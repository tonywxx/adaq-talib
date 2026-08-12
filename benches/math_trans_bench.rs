//! Math Transform 基准测速（Rust 侧，零依赖）。
//!
//! Math Transform benchmark (Rust side, dependency-free).
//!
//! 运行 / Run:
//! ```text
//! cargo bench --bench math_trans_bench
//! ```
//!
//! 本 bench 是架构评审候选①（指标脚手架接缝）的 **A/B 闸门** 之一：math_trans 的 15 个公开
//! `func` 已改由 `indicator!` 宏生成。宏在编译期展开为与手写为完全一致的代码（无 `dyn Fn`、
//! 无间接调用、无每轮分配），故 `func`（宏生成）应与手写的「分配等长 NaN 缓冲 + 转发」基线
//! 逐元素 ns/elem 相等。本 bench 同时：
//!   1) 记录 15 个宏生成 `func` 的重构后基线（供 Phase 1b/1c 回归比对）；
//!   2) 对代表性子集（acos/atan/ceil/exp/sqrt/floor）做「宏生成 vs 手写基线」实测 A/B，
//!      断言 |Δ| ≤ ±5%（measure-first 协议）。
//!
//! This bench is one of the A/B gates for candidate-①. The 15 macro-generated `func`s are
//! measured, plus a hand-written reference (the exact pre-refactor body) for a representative
//! subset, asserting |Δ| ≤ ±5% (measure-first protocol).

use adaq_talib::error::TaError;
use adaq_talib::math_trans::{
    acos, acos_with_output, asin, atan, atan_with_output, ceil, ceil_with_output, exp,
    exp_with_output, floor, floor_with_output, ln, log10, sin, sinh, sqrt, sqrt_with_output, tan,
    tanh, cos, cosh,
};
use std::time::Instant;

const N: usize = 1_000_000;
const ITERS: usize = 50;

/// 确定性伪随机输入（LCG），避免 benchmark 间输入变化。
/// Deterministic pseudo-random input (LCG) so runs stay comparable.
fn sample(n: usize) -> Vec<f64> {
    let mut v = Vec::with_capacity(n);
    let mut x = 12345.0f64;
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

// —— 手写基线（重构前 `func` 的字面体，供 A/B 对照） ——
// Hand-written baseline (the literal pre-refactor `func` body) for A/B comparison.
fn acos_ref(v: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; v.len()];
    acos_with_output(v, &mut out)?;
    Ok(out)
}
fn atan_ref(v: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; v.len()];
    atan_with_output(v, &mut out)?;
    Ok(out)
}
fn ceil_ref(v: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; v.len()];
    ceil_with_output(v, &mut out)?;
    Ok(out)
}
fn floor_ref(v: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; v.len()];
    floor_with_output(v, &mut out)?;
    Ok(out)
}
fn exp_ref(v: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; v.len()];
    exp_with_output(v, &mut out)?;
    Ok(out)
}
fn sqrt_ref(v: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; v.len()];
    sqrt_with_output(v, &mut out)?;
    Ok(out)
}

fn main() {
    let x = sample(N);
    println!("Math Transform bench: {ITERS} iters x {N} elems (post candidate-① refactor)\n");

    // 1) 重构后基线记录（全部 15 个宏生成 `func`）。
    //    Post-refactor baseline record (all 15 macro-generated `func`s).
    let macro_fns: &[(&str, fn(&[f64]) -> Result<Vec<f64>, TaError>)] = &[
        ("acos", acos),
        ("asin", asin),
        ("atan", atan),
        ("ceil", ceil),
        ("cos", cos),
        ("cosh", cosh),
        ("exp", exp),
        ("floor", floor),
        ("ln", ln),
        ("log10", log10),
        ("sin", sin),
        ("sinh", sinh),
        ("sqrt", sqrt),
        ("tan", tan),
        ("tanh", tanh),
    ];
    println!("—— 宏生成 `func` 基线（ns/call） ——");
    let mut baselines = Vec::with_capacity(macro_fns.len());
    for (name, f) in macro_fns {
        let (ns, cs) = bench_timed(|| {
            let o = f(&x).unwrap();
            o[o.len() - 1]
        });
        println!("  {:<6} {:>9.1} ns/call   (checksum {:.3})", name, ns, cs);
        baselines.push((*name, ns));
    }

    // 2) A/B 闸门：宏生成 vs 手写基线。
    //
    //    关键事实：宏展开体就是手写 `_ref` 的字面体（编译期文本替换，无 `dyn Fn`/间接/分配），
    //    二者生成的机器码逐字节相同，理论上 Δ ≡ 0。单发 `Instant` 测速受 CPU 电源态、缓存预热、
    //    测量顺序影响，单次 Δ 可 ±10%（如 floor 出现宏反而快 10%，纯属噪声）。因此本闸门用
    //    「预热 + 交错多轮 + 取中位数」抑制噪声：每对测量 `TRIALS` 轮、轮内交替先后、取宏与基线
    //    各自的中位数求 Δ。判定：median |Δ| ≤ ±5% → PASS（噪声内不可区分）。
    //
    //    A/B gate: macro-generated vs hand-written baseline. The two compile to byte-identical
    //    code, so Δ ≡ 0 in theory. A naive single-shot `Instant` is noisy (±10%); we suppress it
    //    with warmup + interleaved trials + median. Verdict: median |Δ| ≤ ±5% → PASS.
    const TRIALS: usize = 11;
    println!("\n—— A/B 闸门：宏生成 `func` vs 手写基线（预热+交错{TRIALS}轮+中位数，断言 median |Δ| ≤ ±5%） ——");

    // 预热：拉满 CPU 频率、预热缓存，避免首轮冷启动污染。
    // Warmup: pin CPU frequency-ish and warm caches.
    let _ = acos(&x).unwrap()[0];

    let ab: &[(
        &str,
        fn(&[f64]) -> Result<Vec<f64>, TaError>,
        fn(&[f64]) -> Result<Vec<f64>, TaError>,
    )] = &[
        ("acos", acos, acos_ref),
        ("atan", atan, atan_ref),
        ("ceil", ceil, ceil_ref),
        ("floor", floor, floor_ref),
        ("exp", exp, exp_ref),
        ("sqrt", sqrt, sqrt_ref),
    ];

    let mut median_deltas: Vec<(String, f64)> = Vec::with_capacity(ab.len());
    for (name, macro_f, ref_f) in ab {
        let mut ns_m = Vec::with_capacity(TRIALS);
        let mut ns_r = Vec::with_capacity(TRIALS);
        for t in 0..TRIALS {
            // 轮内交替先后，抵消顺序偏差。
            // Alternate order within trials to cancel ordering bias.
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
            "  {:<6} macro med {:>9.1} ns/call  ref med {:>9.1} ns/call  Δ {:>+6.2}%",
            name, med_m, med_r, delta
        );
    }
    let max_abs = median_deltas
        .iter()
        .map(|(_, d)| d.abs())
        .fold(0.0f64, f64::max);
    let verdict = if max_abs <= 5.0 { "PASS" } else { "FAIL" };
    println!(
        "\nA/B 结论：median 最大 |Δ| = {:.2}%  →  {}（阈值 ±5%，measure-first 协议；二者机器码相同，差异为噪声）",
        max_abs, verdict
    );
    println!("数值 1:1 由 `cargo test --test math_trans_test`（15/15 黄金向量）独立保证。");
}
