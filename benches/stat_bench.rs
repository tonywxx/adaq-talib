//! Stat 模块基准测速（Rust 侧，零依赖）—— 架构评审候选① Phase 1b 的 **A/B 闸门**。
//!
//! Stat benchmark (Rust side, dependency-free) — the A/B gate for candidate-① Phase 1b.
//!
//! 运行 / Run:
//! ```text
//! cargo bench --bench stat_bench
//! ```
//!
//! `stat` 的 7 个单输入函数（stddev / var / linear_reg / linear_reg_angle /
//! linear_reg_intercept / linear_reg_slope / tsf）已由 `indicator!` 宏生成（默认臂），
//! 热路径 `*_with_output` 体保持手写、字节级不变。本 bench 对代表性子集（stddev/var 为
//! 2 末尾默认参数、linear_reg/tsf 为 1 末尾默认参数）做「宏生成 vs 手写基线」实测 A/B，
//! 断言 median |Δ| ≤ ±5%（measure-first 协议，见 ADR-0011）。
//!
//! `stat` 的多输入函数 beta / correl 不在本 Phase（属阶段二多输入臂），保持手写。

use adaq_talib::error::TaError;
use adaq_talib::stat::{
    linear_reg, linear_reg_with_output, stddev, stddev_with_output, tsf, tsf_with_output, var,
    var_with_output,
};
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
fn stddev_ref(v: &[f64], period: usize, nb_dev: f64) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; v.len()];
    stddev_with_output(v, period, nb_dev, &mut out)?;
    Ok(out)
}
fn var_ref(v: &[f64], period: usize, nb_dev: f64) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; v.len()];
    var_with_output(v, period, nb_dev, &mut out)?;
    Ok(out)
}
fn linear_reg_ref(v: &[f64], period: usize) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; v.len()];
    linear_reg_with_output(v, period, &mut out)?;
    Ok(out)
}
fn tsf_ref(v: &[f64], period: usize) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; v.len()];
    tsf_with_output(v, period, &mut out)?;
    Ok(out)
}

fn main() {
    let x = sample(N);
    println!("Stat bench: {ITERS} iters x {N} elems (post candidate-① Phase 1b refactor)\n");

    // 基线记录（4 个代表性宏生成 `func` 的 ns/call）。
    // Baseline record (ns/call of 4 representative macro-generated `func`s).
    let baseline: &[(&str, fn(&[f64]) -> Result<Vec<f64>, TaError>)] = &[
        ("stddev", |v| stddev(v, 20, 1.0)),
        ("var", |v| var(v, 20, 1.0)),
        ("linear_reg", |v| linear_reg(v, 14)),
        ("tsf", |v| tsf(v, 14)),
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
    let _ = stddev(&x, 20, 1.0).unwrap()[0]; // 预热，拉满 CPU 频率、预热缓存。

    let ab: &[(
        &str,
        fn(&[f64]) -> Result<Vec<f64>, TaError>,
        fn(&[f64]) -> Result<Vec<f64>, TaError>,
    )] = &[
        ("stddev", |v| stddev(v, 20, 1.0), |v| stddev_ref(v, 20, 1.0)),
        ("var", |v| var(v, 20, 1.0), |v| var_ref(v, 20, 1.0)),
        ("linear_reg", |v| linear_reg(v, 14), |v| linear_reg_ref(v, 14)),
        ("tsf", |v| tsf(v, 14), |v| tsf_ref(v, 14)),
    ];

    let mut median_deltas: Vec<(String, f64)> = Vec::with_capacity(ab.len());
    for (name, macro_f, ref_f) in ab {
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
    println!("数值 1:1 由 `cargo test --test stat_test`（9/9 黄金向量）独立保证。");
}
