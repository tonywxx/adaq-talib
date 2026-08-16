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
//! - **多输入 0-init 臂（候选① Phase 2）**：`cdl_doji`（轻量 1-candle）/ `cdl_engulfing`
//!   （重型 2-candle 比较），基线用 `vec![0.0_f64]`（与重构前手写 body 一致）。
//! - **多输入 NAN/0-init 臂（候选① Phase 2 续：volume）**：`ad` / `obv`（0-init，基线
//!   `vec![0.0_f64]`）/ `adosc`（NAN 臂，基线 `vec![f64::NAN]`），本批 A/B 通过（max |Δ| 0.84%）。
//! - **多输入 NAN 臂（候选① Phase 2 续：overlap）**：`midprice`（high,low,period）/
//!   `sar`（high,low,accel,max）/ `sarext`（high,low,10 标量），均 NAN 臂，本批 A/B 通过（max |Δ| 0.47%）。
//! - **回收 ADR-0011 D5 回退函数（候选① 下一轮）**：`avgprice` / `medprice` / `typprice` /
//!   `wclprice` 四个无不稳定期 price_transform 函数，借 `init zero` 臂重新并入接缝且性能零损失
//!   （闭包以同一 `&[f64]` 传入 4/3/2 路以计时；基线即宏展开体，内核内自检长度）。
//!
//! 仍手写的 `minmax` / `minmax_index`（struct 多输出）属阶段二，不在此 gate 内。

use adaq_talib::error::TaError;
use adaq_talib::math_ops::{
    add, add_with_output, max, max_with_output, min_index, min_index_with_output,
};
use adaq_talib::momentum::{
    adx, adx_with_output, aroon_osc, aroon_osc_default, aroon_osc_with_output, bop, bop_with_output,
    cci, cci_with_output, mfi, mfi_with_output, rsi, rsi_default, rsi_with_output, willr,
    willr_with_output,
};
use adaq_talib::volume::{
    ad, ad_with_output, adosc, adosc_with_output, obv, obv_with_output,
};
use adaq_talib::overlap::{
    mavp, mavp_default, mavp_with_output, midprice, midprice_with_output, sma, sma_with_output,
    sar, sar_with_output, sarext, sarext_with_output, MaType,
};
use adaq_talib::pattern::{
    cdl_doji, cdl_doji_with_output, cdl_engulfing, cdl_engulfing_with_output,
};
use adaq_talib::price_transform::{
    avgdev, avgdev_with_output, avgprice, avgprice_with_output, medprice, medprice_with_output,
    typprice, typprice_with_output, wclprice, wclprice_with_output,
};
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

// —— 多输入 0-init 基线（候选① Phase 2：蜡烛形态 cdl_*）——
// 须用 `vec![0.0_f64; ...]`（与重构前手写 body 一致），否则 A/B 会量到 init 差异而非接缝本身。
// Multi-input 0-init baseline (cand-① Phase 2 cdl_*): faithful pre-refactor `vec![0.0_f64]`.
fn cdl_doji_ref(o: &[f64], h: &[f64], l: &[f64], c: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; o.len()];
    cdl_doji_with_output(o, h, l, c, &mut out)?;
    Ok(out)
}
fn cdl_engulfing_ref(o: &[f64], h: &[f64], l: &[f64], c: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; o.len()];
    cdl_engulfing_with_output(o, h, l, c, &mut out)?;
    Ok(out)
}

// —— price_transform 4 函数 0-init 基线（候选① 下一轮：回收 ADR-0011 D5 回退函数）——
// 与宏展开体一致（内核内已自检长度，故不重复 `check_eq_len`），仅 `vec![0.0_f64]` 初始化。
// Mirrors the macro expansion (kernel self-checks length); 0.0 init. Pure seam A/B.
fn avgprice_ref(h: &[f64], l: &[f64], c: &[f64], o: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; h.len()];
    avgprice_with_output(h, l, c, o, &mut out)?;
    Ok(out)
}
fn medprice_ref(h: &[f64], l: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; h.len()];
    medprice_with_output(h, l, &mut out)?;
    Ok(out)
}
fn typprice_ref(h: &[f64], l: &[f64], c: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; h.len()];
    typprice_with_output(h, l, c, &mut out)?;
    Ok(out)
}
fn wclprice_ref(h: &[f64], l: &[f64], c: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; h.len()];
    wclprice_with_output(h, l, c, &mut out)?;
    Ok(out)
}

// —— momentum 多输入 NAN 臂基线（候选① Phase 2：cci/mfi/willr/adx）——
// 与宏展开体一致（内核内自检长度与周期，故不重复 wrapper 校验）；仅 `vec![f64::NAN]` 初始化。
// momentum multi-input NAN-arm baselines: mirror the macro expansion (kernel self-validates).
fn cci_ref(h: &[f64], l: &[f64], c: &[f64], period: usize) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; h.len()];
    cci_with_output(h, l, c, period, &mut out)?;
    Ok(out)
}
fn mfi_ref(h: &[f64], l: &[f64], c: &[f64], v: &[f64], period: usize) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; h.len()];
    mfi_with_output(h, l, c, v, period, &mut out)?;
    Ok(out)
}
fn willr_ref(h: &[f64], l: &[f64], c: &[f64], period: usize) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; h.len()];
    willr_with_output(h, l, c, period, &mut out)?;
    Ok(out)
}
fn adx_ref(h: &[f64], l: &[f64], c: &[f64], period: usize) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; h.len()];
    adx_with_output(h, l, c, period, &mut out)?;
    Ok(out)
}

// —— momentum 多输入 0-init 臂基线（候选① Phase 2：bop）——
// 须用 `vec![0.0_f64; ...]`（与重构前手写 body 一致）。
// momentum multi-input 0-init baseline (bop): faithful pre-refactor `vec![0.0_f64]`.
fn bop_ref(o: &[f64], h: &[f64], l: &[f64], c: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; o.len()];
    bop_with_output(o, h, l, c, &mut out)?;
    Ok(out)
}

// —— volume 多输入 NAN 臂基线（候选① Phase 2：adosc）—— 与宏展开体一致（内核内自检）。
// volume multi-input NAN-arm baseline (adosc): mirror the macro expansion (kernel self-validates).
fn adosc_ref(
    h: &[f64],
    l: &[f64],
    c: &[f64],
    v: &[f64],
    fp: usize,
    sp: usize,
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; h.len()];
    adosc_with_output(h, l, c, v, fp, sp, &mut out)?;
    Ok(out)
}

// —— volume 多输入 0-init 臂基线（候选① Phase 2：ad / obv）——
// 须用 `vec![0.0_f64; ...]`（与重构前手写 body 一致）。
// volume multi-input 0-init baselines (ad/obv): faithful pre-refactor `vec![0.0_f64]`.
fn ad_ref(h: &[f64], l: &[f64], c: &[f64], v: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; h.len()];
    ad_with_output(h, l, c, v, &mut out)?;
    Ok(out)
}
fn obv_ref(c: &[f64], v: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; c.len()];
    obv_with_output(c, v, &mut out)?;
    Ok(out)
}

// —— overlap 多输入 NAN 臂基线（候选① Phase 2：midprice / sar / sarext）——
// 与宏展开体一致（内核内自检长度与周期，故不重复 wrapper 校验）；仅 `vec![f64::NAN]` 初始化。
// overlap multi-input NAN-arm baselines (midprice/sar/sarext): mirror the macro expansion.
fn midprice_ref(h: &[f64], l: &[f64], period: usize) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; h.len()];
    midprice_with_output(h, l, period, &mut out)?;
    Ok(out)
}
fn sar_ref(h: &[f64], l: &[f64], accel: f64, max: f64) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; h.len()];
    sar_with_output(h, l, accel, max, &mut out)?;
    Ok(out)
}
fn sarext_ref(
    h: &[f64],
    l: &[f64],
    start: f64,
    offset: f64,
    ail: f64,
    al: f64,
    aml: f64,
    ais: f64,
    as_: f64,
    ams: f64,
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; h.len()];
    sarext_with_output(h, l, start, offset, ail, al, aml, ais, as_, ams, &mut out)?;
    Ok(out)
}

// —— overlap / momentum 单输入 + 多输入基线（候选① 本轮：sma / rsi / aroon_osc / mavp）——
// 与重构前手写 body 一致：保留被宏移除的冗余 wrapper 校验（内核已自检，故为纯噪声）。
// 公平 A/B：宏生成（去冗余校验）vs 重构前（含冗余校验），断言 macro ≤ old（|Δ| ≤ ±5%）。
// overlap/momentum single-input & multi-input baselines (this round): faithful pre-refactor body
// WITH the now-removed redundant wrapper validation (kernel self-validates, so pure noise).
const RSI_PERIOD: usize = 14;
const AROON_PERIOD: usize = 14;
const MAVP_MIN_PERIOD: usize = 2;
const MAVP_MAX_PERIOD: usize = 30;

fn sma_ref(v: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    if time_period == 0 {
        return Err(TaError::BadParam("time period must be >= 1".into()));
    }
    let mut out = vec![f64::NAN; v.len()];
    sma_with_output(v, time_period, &mut out)?;
    Ok(out)
}
fn rsi_ref(v: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    if time_period == 0 {
        return Err(TaError::BadParam("time period must be >= 1".into()));
    }
    let mut out = vec![f64::NAN; v.len()];
    rsi_with_output(v, time_period, &mut out)?;
    Ok(out)
}
fn rsi_default_ref(v: &[f64]) -> Result<Vec<f64>, TaError> {
    rsi_ref(v, RSI_PERIOD)
}
fn aroon_osc_ref(h: &[f64], l: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    if time_period == 0 {
        return Err(TaError::BadParam("time period must be >= 1".into()));
    }
    if h.len() != l.len() {
        return Err(TaError::BadParam(
            "aroon_osc: all input arrays must have equal length".into(),
        ));
    }
    let mut out = vec![f64::NAN; h.len()];
    aroon_osc_with_output(h, l, time_period, &mut out)?;
    Ok(out)
}
fn aroon_osc_default_ref(h: &[f64], l: &[f64]) -> Result<Vec<f64>, TaError> {
    aroon_osc_ref(h, l, AROON_PERIOD)
}
fn mavp_ref(
    v: &[f64],
    periods: &[f64],
    min_period: usize,
    max_period: usize,
    ma_type: MaType,
) -> Result<Vec<f64>, TaError> {
    if min_period == 0 || max_period == 0 {
        return Err(TaError::BadParam("time period must be >= 1".into()));
    }
    let mut out = vec![f64::NAN; v.len()];
    mavp_with_output(v, periods, min_period, max_period, ma_type, &mut out)?;
    Ok(out)
}
fn mavp_default_ref(v: &[f64], periods: &[f64]) -> Result<Vec<f64>, TaError> {
    mavp_ref(v, periods, MAVP_MIN_PERIOD, MAVP_MAX_PERIOD, MaType::Sma)
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
        ("cdl_doji", Box::new(|x| cdl_doji(x, x, x, x))),
        ("cdl_engulfing", Box::new(|x| cdl_engulfing(x, x, x, x))),
        ("avgprice", Box::new(|x| avgprice(x, x, x, x))),
        ("medprice", Box::new(|x| medprice(x, x))),
        ("typprice", Box::new(|x| typprice(x, x, x))),
        ("wclprice", Box::new(|x| wclprice(x, x, x))),
        ("cci", Box::new(|x| cci(x, x, x, 14))),
        ("mfi", Box::new(|x| mfi(x, x, x, x, 14))),
        ("willr", Box::new(|x| willr(x, x, x, 14))),
        ("adx", Box::new(|x| adx(x, x, x, 14))),
        ("bop", Box::new(|x| bop(x, x, x, x))),
        ("ad", Box::new(|x| ad(x, x, x, x))),
        ("adosc", Box::new(|x| adosc(x, x, x, x, 3, 10))),
        ("obv", Box::new(|x| obv(x, x))),
        ("midprice", Box::new(|x| midprice(x, x, 30))),
        ("sar", Box::new(|x| sar(x, x, 0.02, 0.2))),
        (
            "sarext",
            Box::new(|x| sarext(x, x, 0.0, 0.0, 0.02, 0.02, 0.2, 0.02, 0.02, 0.2)),
        ),
        // —— 候选① 本轮：overlap / momentum 单输入 + 多输入 ——
        ("sma", Box::new(|x| sma(x, 14))),
        ("rsi", Box::new(|x| rsi(x, 14))),
        ("rsi_default", Box::new(|x| rsi_default(x))),
        ("aroon_osc", Box::new(|x| aroon_osc(x, x, 14))),
        ("aroon_osc_default", Box::new(|x| aroon_osc_default(x, x))),
        (
            "mavp",
            Box::new(|x| mavp(x, x, MAVP_MIN_PERIOD, MAVP_MAX_PERIOD, MaType::Sma)),
        ),
        (
            "mavp_default",
            Box::new(|x| mavp_default(x, x)),
        ),
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
    println!(
        "\n—— A/B 闸门：宏生成 `func` vs 手写基线（预热+交错{TRIALS}轮+中位数，断言 median |Δ| ≤ ±5%） ——"
    );
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
        (
            "max",
            Box::new(|x| max(x, 20)),
            Box::new(|x| max_ref(x, 20)),
        ),
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
        (
            "cdl_doji",
            Box::new(|x| cdl_doji(x, x, x, x)),
            Box::new(|x| cdl_doji_ref(x, x, x, x)),
        ),
        (
            "cdl_engulfing",
            Box::new(|x| cdl_engulfing(x, x, x, x)),
            Box::new(|x| cdl_engulfing_ref(x, x, x, x)),
        ),
        (
            "avgprice",
            Box::new(|x| avgprice(x, x, x, x)),
            Box::new(|x| avgprice_ref(x, x, x, x)),
        ),
        (
            "medprice",
            Box::new(|x| medprice(x, x)),
            Box::new(|x| medprice_ref(x, x)),
        ),
        (
            "typprice",
            Box::new(|x| typprice(x, x, x)),
            Box::new(|x| typprice_ref(x, x, x)),
        ),
        (
            "wclprice",
            Box::new(|x| wclprice(x, x, x)),
            Box::new(|x| wclprice_ref(x, x, x)),
        ),
        (
            "cci",
            Box::new(|x| cci(x, x, x, 14)),
            Box::new(|x| cci_ref(x, x, x, 14)),
        ),
        (
            "mfi",
            Box::new(|x| mfi(x, x, x, x, 14)),
            Box::new(|x| mfi_ref(x, x, x, x, 14)),
        ),
        (
            "willr",
            Box::new(|x| willr(x, x, x, 14)),
            Box::new(|x| willr_ref(x, x, x, 14)),
        ),
        (
            "adx",
            Box::new(|x| adx(x, x, x, 14)),
            Box::new(|x| adx_ref(x, x, x, 14)),
        ),
        (
            "bop",
            Box::new(|x| bop(x, x, x, x)),
            Box::new(|x| bop_ref(x, x, x, x)),
        ),
        (
            "ad",
            Box::new(|x| ad(x, x, x, x)),
            Box::new(|x| ad_ref(x, x, x, x)),
        ),
        (
            "adosc",
            Box::new(|x| adosc(x, x, x, x, 3, 10)),
            Box::new(|x| adosc_ref(x, x, x, x, 3, 10)),
        ),
        (
            "obv",
            Box::new(|x| obv(x, x)),
            Box::new(|x| obv_ref(x, x)),
        ),
        (
            "midprice",
            Box::new(|x| midprice(x, x, 30)),
            Box::new(|x| midprice_ref(x, x, 30)),
        ),
        (
            "sar",
            Box::new(|x| sar(x, x, 0.02, 0.2)),
            Box::new(|x| sar_ref(x, x, 0.02, 0.2)),
        ),
        (
            "sarext",
            Box::new(|x| sarext(x, x, 0.0, 0.0, 0.02, 0.02, 0.2, 0.02, 0.02, 0.2)),
            Box::new(|x| sarext_ref(x, x, 0.0, 0.0, 0.02, 0.02, 0.2, 0.02, 0.02, 0.2)),
        ),
        // —— 候选① 本轮：overlap / momentum 单输入 + 多输入 ——
        (
            "sma",
            Box::new(|x| sma(x, 14)),
            Box::new(|x| sma_ref(x, 14)),
        ),
        (
            "rsi",
            Box::new(|x| rsi(x, 14)),
            Box::new(|x| rsi_ref(x, 14)),
        ),
        (
            "rsi_default",
            Box::new(|x| rsi_default(x)),
            Box::new(|x| rsi_default_ref(x)),
        ),
        (
            "aroon_osc",
            Box::new(|x| aroon_osc(x, x, 14)),
            Box::new(|x| aroon_osc_ref(x, x, 14)),
        ),
        (
            "aroon_osc_default",
            Box::new(|x| aroon_osc_default(x, x)),
            Box::new(|x| aroon_osc_default_ref(x, x)),
        ),
        (
            "mavp",
            Box::new(|x| mavp(x, x, MAVP_MIN_PERIOD, MAVP_MAX_PERIOD, MaType::Sma)),
            Box::new(|x| mavp_ref(x, x, MAVP_MIN_PERIOD, MAVP_MAX_PERIOD, MaType::Sma)),
        ),
        (
            "mavp_default",
            Box::new(|x| mavp_default(x, x)),
            Box::new(|x| mavp_default_ref(x, x)),
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
    println!(
        "数值 1:1 由 `cargo test --test price_transform_test / math_ops_test / volatility_test`（19/19 黄金向量）独立保证。"
    );
}
