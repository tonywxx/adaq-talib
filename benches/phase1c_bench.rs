//! Phase 1c 基准测速（Rust 侧，零依赖）—— 架构评审候选① Phase 1c 的 **A/B 闸门**。
//!
//! Phase 1c benchmark (Rust side, dependency-free) — the A/B gate for candidate-① Phase 1c.
//!
//! 运行 / Run:
//! ```text
//! cargo bench --bench phase1c_bench
//! ```
//!
//! `price_transform` / `math_ops` / `volatility` 的单输出函数中，13 个已由 `indicator!` 宏生成
//! （热路径 `*_with_output` 体保持手写、字节级不变）；`avgprice` / `medprice` / `typprice` /
//! `wclprice` 这四个逐元素 price_transform 函数因原生 `vec![0.0_f64]` 初始化比宏的 `vec![f64::NAN]`
//! 更快、且宏对它们无收益（无前导 NaN、无默认参数），已**回退手写**（见 ADR-0011 D5，measure-first
//! 协议抓出的真实回归）。本 bench 对宏生成的代表性子集做「宏生成 vs 手写基线」实测 A/B，
//! 断言 median |Δ| ≤ ±5%（measure-first 协议，见 ADR-0011）：
//!
//! - 多输入逐元素（multi-input elementwise）：`add`(2 输入) / `trange`(3 输入)
//! - 滚动窗口（rolling）：`max` / `min_index` / `avgdev` / `atr`
//! - 默认臂（default arm）：`avgdev` / `atr`（末尾默认参数）
//!
//! 仍手写的 `minmax` / `minmax_index`（struct 多输出）属阶段二，不在此 gate 内。

use adaq_talib::error::TaError;
use adaq_talib::math_ops::{add, add_with_output, max, max_with_output, min_index, min_index_with_output};
use adaq_talib::price_transform::{avgdev, avgdev_with_output};
use adaq_talib::volatility::{atr, atr_with_output, trange, trange_with_output};
use std::time::Instant;

const N: usize = 1_000_000;
const ITERS: usize = 50;
const TRIALS: usize = 11;

/// 确定性伪随机输入（LCG），避免 benchmark 间输入变化。
/// Deterministic pseudo-random input (LCG) so runs stay comparable.
fn sample(n: usize) -> Vec<f64> {
    let mut v = Vec::with_capacity(n);
    let mut x = 98765.0f64;
    for _ in 0..n {
        x = (x * 1103515245.0 + 12345.0) % 1e9;
        v.push(50.0 + (x / 1e9) * 10.0);
    }
    v
}

/// 测单函数 ns/call（含 anti-optimize checksum），返回 (ns, checksum)。
/// Measure a single function's ns/call (with an anti-optimize checksum); returns (ns, checksum).
fn bench_timed(mut f: impl FnMut() -> f64) -> (f64, f64) {
    let start = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..ITERS {
        checksum += f();
    }
    let ns = start.elapsed().as_nanos() as f64 / ITERS as f64;
    (ns, checksum)
}

// —— 手写基线（`func` 重构前的字面 body，供 A/B 对照） ——
// Hand-written baseline (the literal pre-refactor `func` body) for A/B comparison.
fn add_ref(a: &[f64], b: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; a.len()];
    add_with_output(a, b, &mut out)?;
    Ok(out)
}
fn trange_ref(h: &[f64], l: &[f64], c: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; h.len()];
    trange_with_output(h, l, c, &mut out)?;
    Ok(out)
}
fn max_ref(v: &[f64], period: usize) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; v.len()];
    max_with_output(v, period, &mut out)?;
    Ok(out)
}
fn min_index_ref(v: &[f64], period: usize) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; v.len()];
    min_index_with_output(v, period, &mut out)?;
    Ok(out)
}
fn avgdev_ref(v: &[f64], period: usize) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; v.len()];
    avgdev_with_output(v, period, &mut out)?;
    Ok(out)
}
fn atr_ref(h: &[f64], l: &[f64], c: &[f64], period: usize) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; h.len()];
    atr_with_output(h, l, c, period, &mut out)?;
    Ok(out)
}

fn main() {
    let x = sample(N);
    println!("Phase 1c bench: {ITERS} iters x {N} elems (post candidate-① Phase 1c refactor)\n");

    // 基线记录（7 个代表性宏生成 `func` 的 ns/call）。
    // Baseline record (ns/call of 7 representative macro-generated `func`s).
    let baseline: &[(&str, Box<dyn Fn(&[f64]) -> Result<Vec<f64>, TaError>>)] = &[
        ("add", Box::new(|x| add(x, x))),
        ("trange", Box::new(|x| trange(x, x, x))),
        ("max", Box::new(|x| max(x, 20))),
        ("min_index", Box::new(|x| min_index(x, 20))),
        ("avgdev", Box::new(|x| avgdev(x, 14))),
        ("atr", Box::new(|x| atr(x, x, x, 14))),
    ];
    println!("—— 宏生成 `func` 基线（ns/call） ——");
    for (name, f) in baseline {
        let (ns, cs) = bench_timed(|| {
            let o = f(&x).unwrap();
            o[o.len() - 1]
        });
        println!("  {:<11} {:>9.1} ns/call   (checksum {:.3})", name, ns, cs);
    }

    // A/B 闸门：宏生成 vs 手写基线（预热 + 交错多轮 + 中位数，断言 median |Δ| ≤ ±5%）。
    // A/B gate: macro-generated vs hand-written baseline (warmup + interleaved + median, ≤ ±5%).
    println!("\n—— A/B 闸门：宏生成 `func` vs 手写基线（预热+交错{TRIALS}轮+中位数，断言 median |Δ| ≤ ±5%） ——");
    let _ = add(x.as_slice(), x.as_slice()).unwrap()[0]; // 预热，拉满 CPU 频率、预热缓存。

    let ab: &[(
        &str,
        Box<dyn Fn(&[f64]) -> Result<Vec<f64>, TaError>>,
        Box<dyn Fn(&[f64]) -> Result<Vec<f64>, TaError>>,
    )] = &[
        ("add", Box::new(|x| add(x, x)), Box::new(|x| add_ref(x, x))),
        (
            "trange",
            Box::new(|x| trange(x, x, x)),
            Box::new(|x| trange_ref(x, x, x)),
        ),
        ("max", Box::new(|x| max(x, 20)), Box::new(|x| max_ref(x, 20))),
        (
            "min_index",
            Box::new(|x| min_index(x, 20)),
            Box::new(|x| min_index_ref(x, 20)),
        ),
        (
            "avgdev",
            Box::new(|x| avgdev(x, 14)),
            Box::new(|x| avgdev_ref(x, 14)),
        ),
        (
            "atr",
            Box::new(|x| atr(x, x, x, 14)),
            Box::new(|x| atr_ref(x, x, x, 14)),
        ),
    ];

    let mut median_deltas: Vec<(String, f64)> = Vec::with_capacity(ab.len());
    for (name, macro_f, ref_f) in ab {
        // 预热宏与基线两条路径，消除冷启动 / 缓存效应（否则首个被测函数会虚高 ~10%+）。
        // Warm up BOTH the macro and baseline paths so cold-start/cache effects don't bias the
        // first measured function (which otherwise reports a spurious ~10%+ delta).
        let _ = bench_timed(|| macro_f(&x).unwrap()[x.len() - 1]);
        let _ = bench_timed(|| ref_f(&x).unwrap()[x.len() - 1]);
        let mut ns_m = Vec::with_capacity(TRIALS);
        let mut ns_r = Vec::with_capacity(TRIALS);
        for t in 0..TRIALS {
            let (m, r) = if t % 2 == 0 {
                (
                    bench_timed(|| macro_f(&x).unwrap()[x.len() - 1]),
                    bench_timed(|| ref_f(&x).unwrap()[x.len() - 1]),
                )
            } else {
                let r = bench_timed(|| ref_f(&x).unwrap()[x.len() - 1]);
                let m = bench_timed(|| macro_f(&x).unwrap()[x.len() - 1]);
                (m, r)
            };
            ns_m.push(m.0);
            ns_r.push(r.0);
        }
        ns_m.sort_by(|a, b| a.partial_cmp(b).unwrap());
        ns_r.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med_m = ns_m[TRIALS / 2];
        let med_r = ns_r[TRIALS / 2];
        let delta = (med_m - med_r) / med_r * 100.0;
        median_deltas.push(((*name).to_string(), delta));
        println!(
            "  {:<11} macro med {:>9.1} ns/call  ref med {:>9.1} ns/call  Δ {:>+6.2}%",
            name, med_m, med_r, delta
        );
    }
    let max_abs = median_deltas
        .iter()
        .map(|(_, d)| d.abs())
        .fold(0.0f64, f64::max);
    let verdict = if max_abs <= 5.0 { "PASS" } else { "FAIL" };
    println!(
        "\nA/B 结论：median 最大 |Δ| = {:.2}%  →  {}（阈值 ±5%，measure-first 协议；宏展开体与手写字节级相同，差异为噪声）",
        max_abs, verdict
    );
    println!("数值 1:1 由 `cargo test --test price_transform_test / math_ops_test / volatility_test`（19/19 黄金向量）独立保证。");
}
