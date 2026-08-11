//! 重叠研究类指标（Overlap Studies）。
//!
//! Overlap Studies indicators. These overlay the price series and are the most
//! widely used technical-analysis primitives (SMA, EMA, WMA, DEMA, TEMA, BBANDS, ...).
//!
//! 本模块实现 TA-Lib 0.7.1 中下列重叠研究函数（数值逐项一致，见 ADR 0001/0006）：
//! This module implements the following Overlap Studies functions from TA-Lib 0.7.1
//! (numeric 1:1, see ADR 0001/0006):
//!
//! - [`sma`] / [`sma_default`] — 简单移动平均 / Simple Moving Average
//! - [`ema`] / [`ema_default`] — 指数移动平均 / Exponential Moving Average
//! - [`wma`] / [`wma_default`] — 加权移动平均 / Weighted Moving Average
//! - [`dema`] / [`dema_default`] — 双指数移动平均 / Double Exponential MA
//! - [`tema`] / [`tema_default`] — 三指数移动平均 / Triple Exponential MA
//! - [`midpoint`] / [`midpoint_default`] — 中点（区间）/ MidPoint over period
//! - [`midprice`] / [`midprice_default`] — 中点价（高低）/ MidPoint Price over period
//! - [`bbands`] / [`bbands_default`] — 布林带 / Bollinger Bands
//! - [`trima`] / [`trima_default`] — 三角移动平均 / Triangular MA
//! - [`t3`] / [`t3_default`] — 三指数移动平均（Tillson）/ T3 (Tillson)
//! - [`ma`] / [`ma_default`] — 通用移动平均（支持 `MaType`）/ General MA (via `MaType`)
//! - [`mavp`] / [`mavp_default`] — 变周期移动平均 / Variable-period MA
//! - [`sar`] / [`sar_default`] — 抛物线转向 / Parabolic SAR
//! - [`sarext`] / [`sarext_default`] — 扩展抛物线转向 / Parabolic SAR Extended
//! - [`kama`] / [`kama_default`] — Kaufman 自适应移动平均 / Kaufman Adaptive MA
//! - [`accbands`] / [`accbands_default`] — 加速带 / Acceleration Bands
//!
//! MESA 自适应移动平均与希尔伯特趋势线（MAMA / HT_TRENDLINE）归入 [`crate::cycle`] 模块。
//! MAMA / HT_TRENDLINE (Hilbert-transform indicators) live in [`crate::cycle`].

use crate::core::defaults::{ACCBANDS_PERIOD, DEFAULT_TIME_PERIOD};
use crate::core::{rolling_max, rolling_mean, rolling_mean_skip, rolling_min};
use crate::error::{check_period, TaError};

// ───────────────────────────── SMA ─────────────────────────────

/// 简单移动平均（Simple Moving Average, SMA）。
///
/// Simple Moving Average (SMA). Replicates TA-Lib `TA_SMA`.
///
/// # 参数 / Parameters
/// - `values`：输入序列（如收盘价），类型 `&[f64]`。/ Input series (e.g. close prices), `&[f64]`.
/// - `time_period`：窗口长度 `period >= 1`。对应 TA-Lib `optInTimePeriod`（默认 30）。
///   Window length `period >= 1`, maps to TA-Lib `optInTimePeriod` (default 30).
///
/// # 返回值 / Returns
/// `Result<Vec<f64>, TaError>`：与 `values` **等长**的向量。前导 `period-1` 个位置为
/// [`f64::NAN`]（不稳定期，见 ADR 0007）；若输入长度 `< period`，则全部为 `NaN`
/// （对应 TA-Lib "0 个输出"）。
///
/// `Result<Vec<f64>, TaError>`: a vector with the **same length** as `values`. The first
/// `period - 1` positions are [`f64::NAN`] (unstable period, see ADR 0007); if the input
/// length `< period`, the whole vector is `NaN` (matches TA-Lib "0 outputs").
///
/// # 公式 / Formula
/// ```text
/// SMA[i] = (1/period) * Σ_{k=i-period+1}^{i} values[k],   i >= period-1
/// ```
/// 来源 / Source: TA-Lib `ta_sma.c`.
///
/// # 错误 / Errors
/// - [`TaError::BadParam`]：`time_period == 0`。
///
/// # 示例 / Example
/// ```rust
/// use adaq_talib::overlap::sma;
/// let out = sma(&[1.0, 2.0, 3.0, 4.0, 5.0], 3).unwrap();
/// assert!(out[0].is_nan());
/// assert!((out[2] - 2.0).abs() < 1e-9);
/// assert!((out[4] - 4.0).abs() < 1e-9);
/// ```
pub fn sma(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; values.len()];
    sma_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 简单移动平均，零拷贝写入 `out`（与 `values` 等长）。见 [`sma`]。
/// Simple Moving Average, written zero-copy into `out`. See [`sma`].
pub fn sma_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "sma_with_output: out length must equal values length".into(),
        ));
    }
    let res = rolling_mean(values, time_period);
    out.copy_from_slice(&res);
    Ok(())
}

/// 简单移动平均，使用 TA-Lib 默认周期（30，对应 `optInTimePeriod`）。
///
/// Simple Moving Average with TA-Lib's default period (30, maps to `optInTimePeriod`).
///
/// 等价于以 [`DEFAULT_TIME_PERIOD`](crate::core::defaults::DEFAULT_TIME_PERIOD) 调用 [`sma`]。
/// Equivalent to calling [`sma`] with [`DEFAULT_TIME_PERIOD`](crate::core::defaults::DEFAULT_TIME_PERIOD).
///
/// # 示例 / Example
/// ```rust
/// use adaq_talib::overlap::sma_default;
/// let out = sma_default(&[1.0, 2.0, 3.0]).unwrap();
/// assert!(out[0].is_nan());
/// ```
pub fn sma_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    sma(values, DEFAULT_TIME_PERIOD)
}

// ───────────────────────────── EMA ─────────────────────────────

/// 指数移动平均（Exponential Moving Average, EMA）。
///
/// Exponential Moving Average (EMA). Replicates TA-Lib `TA_EMA` (default `TA_MA_CLASSIC`).
///
/// # 参数 / Parameters
/// - `values`：输入序列 `&[f64]`。/ Input series `&[f64]`.
/// - `time_period`：窗口长度 `period >= 1`，对应 `optInTimePeriod`（默认 30）。
///   Window length `period >= 1`, maps to `optInTimePeriod` (default 30).
///
/// # 返回值 / Returns
/// 与 `values` 等长的向量，前导 `period-1` 为 [`f64::NAN`]。首个有效值 = 前 `period`
/// 个输入的算术均值（SMA 种子），其后按 `k = 2/(period+1)` 递推（见 [`crate::core::ema`]）。
///
/// A vector with the same length as `values`; the leading `period - 1` positions are
/// [`f64::NAN`]. The first valid value is the SMA seed of the first `period` inputs, then
/// recursed with `k = 2/(period+1)` (see [`crate::core::ema`]).
///
/// # 公式 / Formula
/// ```text
/// seed   = mean(values[0..period])
/// EMA[i] = (values[i] - EMA[i-1]) * 2/(period+1) + EMA[i-1],  i >= period
/// ```
/// 来源 / Source: TA-Lib `ta_ema.c`.
///
/// # 错误 / Errors
/// - [`TaError::BadParam`]：`time_period == 0`。
///
/// # 示例 / Example
/// ```rust
/// use adaq_talib::overlap::ema;
/// let out = ema(&[1.0, 2.0, 4.0, 8.0], 3).unwrap();
/// assert!(out[0].is_nan() && out[1].is_nan());
/// assert!((out[2] - 7.0 / 3.0).abs() < 1e-9); // SMA 种子 / SMA seed
/// ```
pub fn ema(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; values.len()];
    ema_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 指数移动平均，零拷贝写入 `out`（与 `values` 等长）。见 [`ema`]。
/// Exponential Moving Average, written zero-copy into `out`. See [`ema`].
pub fn ema_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "ema_with_output: out length must equal values length".into(),
        ));
    }
    crate::core::ema_with_output(values, time_period, out);
    Ok(())
}

/// 指数移动平均，使用 TA-Lib 默认周期（30）。
/// Exponential Moving Average with TA-Lib's default period (30).
///
/// # 示例 / Example
/// ```rust
/// use adaq_talib::overlap::ema_default;
/// let out = ema_default(&[1.0, 2.0, 3.0]).unwrap();
/// assert!(out[0].is_nan());
/// ```
pub fn ema_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    ema(values, DEFAULT_TIME_PERIOD)
}

// ───────────────────────────── WMA ─────────────────────────────

/// 加权移动平均（Weighted Moving Average, WMA）。
///
/// Weighted Moving Average (WMA). Replicates TA-Lib `TA_WMA`.
///
/// # 参数 / Parameters
/// - `values`：输入序列 `&[f64]`。/ Input series `&[f64]`.
/// - `time_period`：窗口长度 `period >= 1`，对应 `optInTimePeriod`（默认 30）。
///
/// # 返回值 / Returns
/// 与 `values` 等长，前导 `period-1` 为 [`f64::NAN`]。权重 `period..1`（最新价权重最大），
/// 归一化除以 `period*(period+1)/2`（见 [`crate::core::wma`]）。
///
/// Same length as `values`; leading `period - 1` are [`f64::NAN`]. Weights are
/// `period..1` (most-recent highest), normalized by `period*(period+1)/2`
/// (see [`crate::core::wma`]).
///
/// # 公式 / Formula
/// ```text
/// WMA[i] = Σ_{j=0}^{period-1} (period-j) * values[i-j]  /  (period*(period+1)/2),  i >= period-1
/// ```
/// 来源 / Source: TA-Lib `ta_wma.c`.
///
/// # 错误 / Errors
/// - [`TaError::BadParam`]：`time_period == 0`。
///
/// # 示例 / Example
/// ```rust
/// use adaq_talib::overlap::wma;
/// // 权重 3,2,1，归一化 /6；窗口 [1,2,4] -> (3*4 + 2*2 + 1*1)/6 = 17/6
/// let out = wma(&[1.0, 2.0, 4.0], 3).unwrap();
/// assert!((out[2] - 17.0 / 6.0).abs() < 1e-9);
/// ```
pub fn wma(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; values.len()];
    wma_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 加权移动平均，零拷贝写入 `out`（与 `values` 等长）。见 [`wma`]。
/// Weighted Moving Average, written zero-copy into `out`. See [`wma`].
pub fn wma_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "wma_with_output: out length must equal values length".into(),
        ));
    }
    crate::core::wma_with_output(values, time_period, out);
    Ok(())
}

/// 加权移动平均，使用 TA-Lib 默认周期（30）。
/// Weighted Moving Average with TA-Lib's default period (30).
pub fn wma_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    wma(values, DEFAULT_TIME_PERIOD)
}

// ──────────────────────────── DEMA ─────────────────────────────

/// 双指数移动平均（Double Exponential Moving Average, DEMA）。
///
/// Double Exponential Moving Average (DEMA). Replicates TA-Lib `TA_DEMA`.
///
/// # 参数 / Parameters
/// - `values`：输入序列 `&[f64]`。/ Input series `&[f64]`.
/// - `time_period`：窗口长度 `period >= 1`，对应 `optInTimePeriod`（默认 30）。
///
/// # 返回值 / Returns
/// 与 `values` 等长，前导 `2*(period-1)` 为 [`f64::NAN`]。由两层 EMA 组合：
/// `DEMA = 2*EMA(x) - EMA(EMA(x))`，嵌套 EMA 作用于上一层有效（非 NaN）部分
/// （见 [`crate::core::ema`]），与原版对齐一致。
///
/// Same length as `values`; the leading `2*(period-1)` positions are [`f64::NAN`].
/// Composed of two EMAs: `DEMA = 2*EMA(x) - EMA(EMA(x))`, where the nested EMA operates
/// on the valid (non-NaN) portion of the previous EMA (see [`crate::core::ema`]), matching
/// the original.
///
/// # 公式 / Formula
/// ```text
/// E1 = EMA(values, period)
/// E2 = EMA(E1, period)          // 仅作用于 E1 的有效段 / over valid E1
/// DEMA[i] = 2*E1[i] - E2[i]
/// ```
/// 来源 / Source: TA-Lib `ta_dema.c`.
///
/// # 错误 / Errors
/// - [`TaError::BadParam`]：`time_period == 0`。
///
/// # 示例 / Example
/// ```rust
/// use adaq_talib::overlap::dema;
/// let out = dema(&[1.0, 2.0, 4.0, 8.0, 16.0, 32.0], 3).unwrap();
/// assert!(out[0].is_nan() && out[1].is_nan() && out[2].is_nan() && out[3].is_nan());
/// // 首个有效值在索引 2*(period-1) = 4 / first valid at index 2*(period-1) = 4
/// assert!(!out[4].is_nan());
/// ```
/// 双指数移动平均，零拷贝写入 `out`（与 `values` 等长）。
///
/// Double Exponential Moving Average (TA-Lib `TA_DEMA`), written zero-copy into `out`.
/// Reuses the [`crate::core::nested_ema_with_output`] single-pass fused kernel (ADR 0010
/// P2-1), eliminating the 2 independent `ema` scans and `Vec` allocations of the naive
/// version. Numerically identical to [`dema`] (1:1 with TA-Lib).
///
/// `out` 长度必须等于 `values.len()`，否则返回 [`TaError::BadParam`]。
/// `out` must have length equal to `values.len()`; otherwise [`TaError::BadParam`] is returned.
pub fn dema_with_output(values: &[f64], time_period: usize, out: &mut [f64]) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "dema_with_output: out length must equal values length".into(),
        ));
    }
    crate::core::nested_ema_with_output::<2, _>(values, time_period, |e| 2.0 * e[0] - e[1], out);
    Ok(())
}

/// 双指数移动平均（Double Exponential Moving Average, DEMA）。
///
/// Double Exponential Moving Average (DEMA). Replicates TA-Lib `TA_DEMA`.
///
/// # 参数 / Parameters
/// - `values`：输入序列 `&[f64]`。/ Input series `&[f64]`.
/// - `time_period`：窗口长度 `period >= 1`，对应 `optInTimePeriod`（默认 30）。
///
/// # 返回值 / Returns
/// 与 `values` 等长，前导 `2*(period-1)` 为 [`f64::NAN`]。由两层 EMA 组合：
/// `DEMA = 2*EMA(x) - EMA(EMA(x))`，嵌套 EMA 作用于上一层有效（非 NaN）部分
/// （见 [`crate::core::ema`]），与原版对齐一致。
///
/// Same length as `values`; the leading `2*(period-1)` positions are [`f64::NAN`].
/// Composed of two EMAs: `DEMA = 2*EMA(x) - EMA(EMA(x))`, where the nested EMA operates
/// on the valid (non-NaN) portion of the previous EMA (see [`crate::core::ema`]), matching
/// the original.
///
/// # 公式 / Formula
/// ```text
/// E1 = EMA(values, period)
/// E2 = EMA(E1, period)          // 仅作用于 E1 的有效段 / over valid E1
/// DEMA[i] = 2*E1[i] - E2[i]
/// ```
/// 来源 / Source: TA-Lib `ta_dema.c`.
///
/// # 错误 / Errors
/// - [`TaError::BadParam`]：`time_period == 0`。
///
/// # 示例 / Example
/// ```rust
/// use adaq_talib::overlap::dema;
/// let out = dema(&[1.0, 2.0, 4.0, 8.0, 16.0, 32.0], 3).unwrap();
/// assert!(out[0].is_nan() && out[1].is_nan() && out[2].is_nan() && out[3].is_nan());
/// // 首个有效值在索引 2*(period-1) = 4 / first valid at index 2*(period-1) = 4
/// assert!(!out[4].is_nan());
/// ```
pub fn dema(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; values.len()];
    dema_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 双指数移动平均，使用 TA-Lib 默认周期（30）。
/// Double Exponential Moving Average with TA-Lib's default period (30).
pub fn dema_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    dema(values, DEFAULT_TIME_PERIOD)
}

// ──────────────────────────── TEMA ─────────────────────────────

/// 三指数移动平均（Triple Exponential Moving Average, TEMA）。
///
/// Triple Exponential Moving Average (TEMA). Replicates TA-Lib `TA_TEMA`.
///
/// # 参数 / Parameters
/// - `values`：输入序列 `&[f64]`。/ Input series `&[f64]`.
/// - `time_period`：窗口长度 `period >= 1`，对应 `optInTimePeriod`（默认 30）。
///
/// # 返回值 / Returns
/// 与 `values` 等长，前导 `3*(period-1)` 为 [`f64::NAN`]。由三层 EMA 组合：
/// `TEMA = 3*EMA1 - 3*EMA2 + EMA3`，每层嵌套 EMA 作用于上一层有效段
/// （见 [`crate::core::ema`]），与原版对齐一致。
///
/// Same length as `values`; the leading `3*(period-1)` positions are [`f64::NAN`].
/// Composed of three EMAs: `TEMA = 3*EMA1 - 3*EMA2 + EMA3`, each nested EMA over the
/// previous valid portion (see [`crate::core::ema`]), matching the original.
///
/// # 公式 / Formula
/// ```text
/// E1 = EMA(values, period)
/// E2 = EMA(E1, period)
/// E3 = EMA(E2, period)
/// TEMA[i] = 3*E1[i] - 3*E2[i] + E3[i]
/// ```
/// 来源 / Source: TA-Lib `ta_tema.c`.
///
/// # 错误 / Errors
/// - [`TaError::BadParam`]：`time_period == 0`。
///
/// # 示例 / Example
/// ```rust
/// use adaq_talib::overlap::tema;
/// let out = tema(&[1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0], 3).unwrap();
/// // 首个有效值在索引 3*(period-1) = 6 / first valid at index 3*(period-1) = 6
/// assert!(out[5].is_nan());
/// assert!(!out[6].is_nan());
/// ```
/// 三指数移动平均，零拷贝写入 `out`（与 `values` 等长）。
///
/// Triple Exponential Moving Average (TA-Lib `TA_TEMA`), written zero-copy into `out`.
/// Reuses the [`crate::core::nested_ema_with_output`] single-pass fused kernel (ADR 0010
/// P2-1), eliminating the 3 independent `ema` scans and `Vec` allocations of the naive
/// version. Numerically identical to [`tema`] (1:1 with TA-Lib).
///
/// `out` 长度必须等于 `values.len()`，否则返回 [`TaError::BadParam`]。
/// `out` must have length equal to `values.len()`; otherwise [`TaError::BadParam`] is returned.
pub fn tema_with_output(values: &[f64], time_period: usize, out: &mut [f64]) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "tema_with_output: out length must equal values length".into(),
        ));
    }
    crate::core::nested_ema_with_output::<3, _>(values, time_period, |e| 3.0 * e[0] - 3.0 * e[1] + e[2], out);
    Ok(())
}

/// 三指数移动平均（Triple Exponential Moving Average, TEMA）。
///
/// Triple Exponential Moving Average (TEMA). Replicates TA-Lib `TA_TEMA`.
///
/// # 参数 / Parameters
/// - `values`：输入序列 `&[f64]`。/ Input series `&[f64]`.
/// - `time_period`：窗口长度 `period >= 1`，对应 `optInTimePeriod`（默认 30）。
///
/// # 返回值 / Returns
/// 与 `values` 等长，前导 `3*(period-1)` 为 [`f64::NAN`]。由三层 EMA 组合：
/// `TEMA = 3*EMA1 - 3*EMA2 + EMA3`，每层嵌套 EMA 作用于上一层有效段
/// （见 [`crate::core::ema`]），与原版对齐一致。
///
/// Same length as `values`; the leading `3*(period-1)` positions are [`f64::NAN`].
/// Composed of three EMAs: `TEMA = 3*EMA1 - 3*EMA2 + EMA3`, each nested EMA over the
/// previous valid portion (see [`crate::core::ema`]), matching the original.
///
/// # 公式 / Formula
/// ```text
/// E1 = EMA(values, period)
/// E2 = EMA(E1, period)
/// E3 = EMA(E2, period)
/// TEMA[i] = 3*E1[i] - 3*E2[i] + E3[i]
/// ```
/// 来源 / Source: TA-Lib `ta_tema.c`.
///
/// # 错误 / Errors
/// - [`TaError::BadParam`]：`time_period == 0`。
///
/// # 示例 / Example
/// ```rust
/// use adaq_talib::overlap::tema;
/// let out = tema(&[1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0], 3).unwrap();
/// // 首个有效值在索引 3*(period-1) = 6 / first valid at index 3*(period-1) = 6
/// assert!(out[5].is_nan());
/// assert!(!out[6].is_nan());
/// ```
pub fn tema(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; values.len()];
    tema_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 三指数移动平均，使用 TA-Lib 默认周期（30）。
/// Triple Exponential Moving Average with TA-Lib's default period (30).
pub fn tema_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    tema(values, DEFAULT_TIME_PERIOD)
}

// ────────────────────────── MIDPOINT ───────────────────────────

/// 中点（MidPoint over period）。
///
/// MidPoint over period. Replicates TA-Lib `TA_MIDPOINT`.
///
/// # 参数 / Parameters
/// - `values`：输入序列 `&[f64]`（通常为收盘价）。/ Input series `&[f64]` (usually close).
/// - `time_period`：窗口长度 `period >= 1`，对应 `optInTimePeriod`（默认 30）。
///
/// # 返回值 / Returns
/// 与 `values` 等长，前导 `period-1` 为 [`f64::NAN`]。每个位置为窗口内
/// `(max + min) / 2`。
///
/// Same length as `values`; leading `period - 1` are [`f64::NAN`]. Each position is
/// `(max + min) / 2` over the window.
///
/// # 公式 / Formula
/// ```text
/// MIDPOINT[i] = (max(values[i-period+1..=i]) + min(values[i-period+1..=i])) / 2,  i >= period-1
/// ```
/// 来源 / Source: TA-Lib `ta_midpoint.c`.
///
/// # 错误 / Errors
/// - [`TaError::BadParam`]：`time_period == 0`。
///
/// # 示例 / Example
/// ```rust
/// use adaq_talib::overlap::midpoint;
/// let out = midpoint(&[1.0, 5.0, 3.0], 3).unwrap();
/// assert!((out[2] - (5.0 + 1.0) / 2.0).abs() < 1e-9); // (max+min)/2
/// ```
pub fn midpoint(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; values.len()];
    midpoint_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 中点，零拷贝写入 `out`（与 `values` 等长）。见 [`midpoint`]。
/// MidPoint, written zero-copy into `out`. See [`midpoint`].
pub fn midpoint_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "midpoint_with_output: out length must equal values length".into(),
        ));
    }
    // 数据量足够且启用 `parallel` feature 时走多核分块；内核与串行逐字节一致，输出 1:1。
    // Under the `parallel` feature with enough data, use multi-core chunking; the kernel is
    // byte-identical to the serial path, so output is 1:1.
    #[cfg(feature = "parallel")]
    {
        if values.len() >= 8192 {
            return midpoint_parallel_with_output(values, time_period, out);
        }
    }
    midpoint_serial_with_output(values, time_period, out)
}

/// 中点串行内核（与 TA-Lib `TA_MIDPOINT` 逐项 1:1）。见 [`midpoint_with_output`]。
/// Serial kernel for MidPoint (1:1 with TA-Lib `TA_MIDPOINT`). See [`midpoint_with_output`].
fn midpoint_serial_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    let (mx, mn) = crate::core::rolling_minmax(values, time_period);
    for (i, (a, b)) in mx.iter().zip(&mn).enumerate() {
        out[i] = (a + b) / 2.0;
    }
    Ok(())
}

/// 中点串行版本（feature 无关，供并行对照测试作黄金参考）。见 [`midpoint`]。
/// Serial MidPoint (feature-agnostic; golden reference for the parallel equality test). See [`midpoint`].
pub fn midpoint_serial(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; values.len()];
    midpoint_serial_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 中点多核并行版本（需 `parallel` feature）。复用 [`midpoint_serial_with_output`] 内核，
/// 以 `period-1` 前导重叠播种各分块的单调双端队列状态，输出与串行逐项 1:1。
/// Multi-core parallel MidPoint (requires the `parallel` feature). Reuses the
/// [`midpoint_serial_with_output`] kernel with `period-1` leading overlap to seed each chunk's
/// deque state; output is 1:1 with the serial path.
#[cfg(feature = "parallel")]
pub fn midpoint_parallel(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; values.len()];
    midpoint_parallel_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 中点并行内核（零拷贝写入 `out`）。见 [`midpoint_parallel`]。
/// Parallel kernel for MidPoint (zero-copy into `out`). See [`midpoint_parallel`].
#[cfg(feature = "parallel")]
fn midpoint_parallel_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "midpoint_parallel_with_output: out length must equal values length".into(),
        ));
    }
    let period = time_period;
    crate::parallel::parallel_index_map(values.len(), period - 1, out, |start, end| {
        let mut local = vec![f64::NAN; end - start];
        // 复用串行内核处理扩展区间 [start, end)；仅自有区间会被写回 `out`。
        // Reuse the serial kernel on the extended range [start, end); only the owned range is written back.
        let _ = midpoint_serial_with_output(&values[start..end], period, &mut local);
        local
    });
    Ok(())
}

/// 中点，使用 TA-Lib 默认周期（30）。
/// MidPoint with TA-Lib's default period (30).
pub fn midpoint_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    midpoint(values, DEFAULT_TIME_PERIOD)
}

// ────────────────────────── MIDPRICE ───────────────────────────

/// 中点价（MidPoint Price over period）。
///
/// MidPoint Price over period. Replicates TA-Lib `TA_MIDPRICE`.
///
/// # 参数 / Parameters
/// - `high`：最高价序列 `&[f64]`。/ High-price series `&[f64]`.
/// - `low`：最低价序列 `&[f64]`。/ Low-price series `&[f64]`.
/// - `time_period`：窗口长度 `period >= 1`，对应 `optInTimePeriod`（默认 30）。
///
/// # 返回值 / Returns
/// 与输入等长，前导 `period-1` 为 [`f64::NAN`]。每个位置为
/// `(max(high 窗口) + min(low 窗口)) / 2`。`high` 与 `low` 长度须一致。
///
/// Same length as inputs; leading `period - 1` are [`f64::NAN`]. Each position is
/// `(max(high window) + min(low window)) / 2`. `high` and `low` must have equal length.
///
/// # 公式 / Formula
/// ```text
/// MIDPRICE[i] = (max(high[i-period+1..=i]) + min(low[i-period+1..=i])) / 2,  i >= period-1
/// ```
/// 来源 / Source: TA-Lib `ta_midprice.c`.
///
/// # 错误 / Errors
/// - [`TaError::BadParam`]：`time_period == 0` 或 `high.len() != low.len()`。
///
/// # 示例 / Example
/// ```rust
/// use adaq_talib::overlap::midprice;
/// let high = [5.0, 6.0, 7.0];
/// let low  = [1.0, 2.0, 3.0];
/// let out = midprice(&high, &low, 3).unwrap();
/// assert!((out[2] - (7.0 + 1.0) / 2.0).abs() < 1e-9);
/// ```
pub fn midprice(high: &[f64], low: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; high.len()];
    midprice_with_output(high, low, time_period, &mut out)?;
    Ok(out)
}

/// 中点价，零拷贝写入 `out`（与 `high`/`low` 等长）。见 [`midprice`]。
/// MidPoint Price, written zero-copy into `out`. See [`midprice`].
pub fn midprice_with_output(
    high: &[f64],
    low: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if high.len() != low.len() {
        return Err(TaError::BadParam(
            "high and low must have equal length".into(),
        ));
    }
    if out.len() != high.len() {
        return Err(TaError::BadParam(
            "midprice_with_output: out length must equal high/low length".into(),
        ));
    }
    let mx = rolling_max(high, time_period);
    let mn = rolling_min(low, time_period);
    for (i, (a, b)) in mx.iter().zip(&mn).enumerate() {
        out[i] = (a + b) / 2.0;
    }
    Ok(())
}

/// 中点价，使用 TA-Lib 默认周期（30）。
/// MidPoint Price with TA-Lib's default period (30).
pub fn midprice_default(high: &[f64], low: &[f64]) -> Result<Vec<f64>, TaError> {
    midprice(high, low, DEFAULT_TIME_PERIOD)
}

// ───────────────────────── 移动平均类型 ─────────────────────────

/// 移动平均类型（映射 TA-Lib `TA_MAType`），用于 [`ma`] / [`bbands`] / [`mavp`]。
///
/// Moving-average type (maps to TA-Lib `TA_MAType`), used by [`ma`] / [`bbands`] / [`mavp`].
///
/// 取值与 TA-Lib 0.7.1 的整数枚举一致：`SMA=0, EMA=1, WMA=2, DEMA=3, TEMA=4, TRIMA=5,
/// KAMA=6, MAMA=7`。注意：T3 / HMA 等后续版本新增的类型不在 0.7.1 枚举内，不在此列。
///
/// The discriminants match TA-Lib 0.7.1 (`SMA=0 … MAMA=7`). T3 / HMA (added later) are not
/// part of the 0.7.1 enum and are omitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MaType {
    /// 简单移动平均 / Simple Moving Average (`TA_MAType_SMA`, 0).
    Sma = 0,
    /// 指数移动平均 / Exponential Moving Average (`TA_MAType_EMA`, 1).
    Ema = 1,
    /// 加权移动平均 / Weighted Moving Average (`TA_MAType_WMA`, 2).
    Wma = 2,
    /// 双指数移动平均 / Double EMA (`TA_MAType_DEMA`, 3).
    Dema = 3,
    /// 三指数移动平均 / Triple EMA (`TA_MAType_TEMA`, 4).
    Tema = 4,
    /// 三角移动平均 / Triangular MA (`TA_MAType_TRIMA`, 5).
    Trima = 5,
    /// Kaufman 自适应移动平均 / Kaufman Adaptive MA (`TA_MAType_KAMA`, 6).
    Kama = 6,
    /// MESA 自适应移动平均 / MESA Adaptive MA (`TA_MAType_MAMA`, 7).
    Mama = 7,
}

/// 通用移动平均的内部 lookback（对应 TA-Lib `TA_MA_Lookback`）。
/// Internal lookback for the general MA (maps to TA-Lib `TA_MA_Lookback`).
fn ma_lookback(period: usize, ma_type: MaType) -> usize {
    match ma_type {
        MaType::Sma | MaType::Ema | MaType::Wma | MaType::Trima => period - 1,
        MaType::Dema => 2 * (period - 1),
        MaType::Tema => 3 * (period - 1),
        MaType::Kama => period,
        MaType::Mama => 32,
    }
}

// ───────────────────────────── BBANDS ─────────────────────────────

/// 布林带（Bollinger Bands，TA-Lib `TA_BBANDS`）。
///
/// Bollinger Bands (TA-Lib `TA_BBANDS`). The middle band is a moving average of `values`
/// (selected by `ma_type`, SMA by default); the upper/lower bands are the middle band
/// shifted by `nb_dev_up` / `nb_dev_dn` times the population standard deviation over the
/// same window.
///
/// # 参数 / Parameters
/// - `values`：输入序列 `&[f64]`。/ Input series `&[f64]`.
/// - `time_period`：窗口长度 `period >= 2`（TA-Lib 默认 20）。/ Window length `period >= 2`.
/// - `nb_dev_up` / `nb_dev_dn`：上/下偏离倍数（TA-Lib 默认 2.0 / 2.0）。
/// - `ma_type`：中轨所用的移动平均类型（见 [`MaType`]）。
///
/// # 返回值 / Returns
/// [`Bbands`] 结构体，三轨均与 `values` 等长，前导不稳定期为各轨自身的 lookback。
/// A [`Bbands`] whose three bands are equal-length; the leading unstable period is each
/// band's own lookback.
pub fn bbands(
    values: &[f64],
    time_period: usize,
    nb_dev_up: f64,
    nb_dev_dn: f64,
    ma_type: MaType,
) -> Result<Bbands, TaError> {
    let n = values.len();
    let mut out = Bbands {
        upper: vec![f64::NAN; n],
        middle: vec![f64::NAN; n],
        lower: vec![f64::NAN; n],
    };
    bbands_with_output(values, time_period, nb_dev_up, nb_dev_dn, ma_type, &mut out)?;
    Ok(out)
}

/// 布林带，零拷贝写入 `out`（与 `values` 等长的三轨向量）。
///
/// Bollinger Bands, written zero-copy into `out` (three equal-length bands). Reuses the
/// single-pass [`crate::core::rolling_mean_var`] fused kernel (ADR 0010 P2-4) for the default
/// SMA middle band, merging the mean+stddev traversal into one pass; non-SMA middle bands
/// fall back to [`ma`] + [`crate::stat::stddev`]. Numerically identical to [`bbands`].
///
/// `out` 的三条带长度均必须等于 `values.len()`，否则返回 [`TaError::BadParam`]。
/// All three bands of `out` must have length equal to `values.len()`; otherwise
/// [`TaError::BadParam`] is returned.
pub fn bbands_with_output(
    values: &[f64],
    time_period: usize,
    nb_dev_up: f64,
    nb_dev_dn: f64,
    ma_type: MaType,
    out: &mut Bbands,
) -> Result<(), TaError> {
    if time_period < 2 {
        return Err(TaError::BadParam(
            "bbands: time_period must be >= 2".into(),
        ));
    }
    let n = values.len();
    if out.upper.len() != n || out.middle.len() != n || out.lower.len() != n {
        return Err(TaError::BadParam(
            "bbands_with_output: out bands must have length == values length".into(),
        ));
    }
    if ma_type == MaType::Sma {
        // 单遍融合：同一次窗口滑动同时产出 mean 与 var（P2-4，ADR 0010）。
        // Single-pass fused: one window traversal yields both mean and var.
        let (middle, var) = crate::core::rolling_mean_var(values, time_period);
        for i in 0..n {
            out.middle[i] = middle[i];
            if var[i].is_nan() {
                out.upper[i] = f64::NAN;
                out.lower[i] = f64::NAN;
            } else {
                let sd = var[i].sqrt();
                out.upper[i] = middle[i] + nb_dev_up * sd;
                out.lower[i] = middle[i] - nb_dev_dn * sd;
            }
        }
    } else {
        // 非 SMA 中轨：保留原分解（ma + stddev），行为零偏差。
        // Non-SMA middle: keep the original decomposition (ma + stddev), zero deviation.
        let middle = ma(values, time_period, ma_type)?;
        let sd = crate::stat::stddev(values, time_period, 1.0)?;
        for i in 0..n {
            out.middle[i] = middle[i];
            if middle[i].is_nan() || sd[i].is_nan() {
                out.upper[i] = f64::NAN;
                out.lower[i] = f64::NAN;
            } else {
                out.upper[i] = middle[i] + nb_dev_up * sd[i];
                out.lower[i] = middle[i] - nb_dev_dn * sd[i];
            }
        }
    }
    Ok(())
}

/// 布林带结果（三轨等长向量）。/ Bollinger Bands result (three equal-length vectors).
pub struct Bbands {
    /// 上轨 / Upper band.
    pub upper: Vec<f64>,
    /// 中轨（移动平均）/ Middle band (moving average).
    pub middle: Vec<f64>,
    /// 下轨 / Lower band.
    pub lower: Vec<f64>,
}

/// 布林带，使用 TA-Lib 默认参数（周期 20、偏离 2.0/2.0、中轨 SMA）。
/// Bollinger Bands with TA-Lib defaults (period 20, dev 2.0/2.0, SMA middle).
pub fn bbands_default(values: &[f64]) -> Result<Bbands, TaError> {
    use crate::core::defaults::{BBANDS_NB_DEV_DN, BBANDS_NB_DEV_UP, BBANDS_PERIOD};
    bbands(
        values,
        BBANDS_PERIOD,
        BBANDS_NB_DEV_UP,
        BBANDS_NB_DEV_DN,
        MaType::Sma,
    )
}

// ───────────────────────────── ACCBANDS ─────────────────────────────

/// 加速带（Acceleration Bands，TA-Lib `TA_ACCBANDS`）。
///
/// Acceleration Bands. Three SMA bands over a shared window of `period` bars:
/// ```text
/// middle = SMA( close )
/// upper  = SMA( high * (1 + 4·(high − low) / (high + low)) )
/// lower  = SMA( low  * (1 − 4·(high − low) / (high + low)) )
/// ```
/// When `high + low` is within TA-Lib's `TA_IS_ZERO` epsilon (±1e-8), the upper/lower
/// maps degenerate to `high` / `low` (the `1 ± …` term drops out), matching the C source.
/// Lookback is `period − 1` (default 20); the first `period − 1` positions are [`f64::NAN`].
///
/// # 参数 / Parameters
/// - `high` / `low` / `close`：蜡烛数据 `&[f64]`。/ Candle data `&[f64]`.
/// - `period`：窗口周期（TA-Lib 默认 20，需 `period >= 2`）。/ Window period (default 20).
///
/// # 返回值 / Returns
/// [`AccBands`] 结构体，三轨均与输入等长。/ [`AccBands`] with three equal-length bands.
pub fn accbands(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> Result<AccBands, TaError> {
    check_period(period)?;
    let n = high.len();
    let mut out = AccBands {
        upper: vec![f64::NAN; n],
        middle: vec![f64::NAN; n],
        lower: vec![f64::NAN; n],
    };
    accbands_with_output(high, low, close, period, &mut out)?;
    Ok(out)
}

/// 加速带，零拷贝写入 `out`（三轨均与输入等长）。见 [`accbands`]。
/// Acceleration Bands, written zero-copy into `out` (three equal-length bands). See
/// [`accbands`].
pub fn accbands_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
    out: &mut AccBands,
) -> Result<(), TaError> {
    check_period(period)?;
    if period < 2 {
        return Err(TaError::BadParam("accbands: period must be >= 2".into()));
    }
    let n = high.len();
    if n != low.len() || n != close.len() {
        return Err(TaError::BadParam(
            "accbands: high/low/close must share the same length".into(),
        ));
    }
    if out.upper.len() != n || out.middle.len() != n || out.lower.len() != n {
        return Err(TaError::BadParam(
            "accbands_with_output: out bands must have length == input length".into(),
        ));
    }
    let mut upper_map = vec![0.0_f64; n];
    let mut lower_map = vec![0.0_f64; n];
    for i in 0..n {
        let hl = high[i] + low[i];
        // TA-Lib: `if (!TA_IS_ZERO(high+low))` → |hl| >= 1e-8.
        if hl.abs() >= 1e-8 {
            let k = 4.0 * (high[i] - low[i]) / hl;
            upper_map[i] = high[i] * (1.0 + k);
            lower_map[i] = low[i] * (1.0 - k);
        } else {
            upper_map[i] = high[i];
            lower_map[i] = low[i];
        }
    }
    // 三个共享窗口的 SMA，等价于 C 的 running-sum 三带实现。
    // Three SMAs over the shared window; bit-identical to C's running-sum form.
    let upper = rolling_mean(&upper_map, period);
    let middle = rolling_mean(close, period);
    let lower = rolling_mean(&lower_map, period);
    out.upper = upper;
    out.middle = middle;
    out.lower = lower;
    Ok(())
}

/// 加速带结果（三轨等长向量）。/ Acceleration Bands result (three equal-length vectors).
pub struct AccBands {
    /// 上轨 / Upper band.
    pub upper: Vec<f64>,
    /// 中轨（收盘价 SMA）/ Middle band (SMA of close).
    pub middle: Vec<f64>,
    /// 下轨 / Lower band.
    pub lower: Vec<f64>,
}

/// 加速带，使用 TA-Lib 默认参数（周期 20）。/ Acceleration Bands with TA-Lib default (period 20).
pub fn accbands_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<AccBands, TaError> {
    accbands(high, low, close, ACCBANDS_PERIOD)
}

// ───────────────────────────── TRIMA ─────────────────────────────

/// 三角移动平均（Triangular Moving Average, TRIMA，TA-Lib `TA_TRIMA`）。
///
/// Triangular Moving Average (TRIMA). TA-Lib 的 "SMA of a SMA" 实现：
/// 奇数周期 `p` → `SMA(SMA(x, (p+1)/2), (p+1)/2)`；偶数周期 → `SMA(SMA(x, p/2), p/2+1)`。
/// 前导 `period-1` 为 [`f64::NAN`]。
///
/// TA-Lib's "SMA of a SMA": odd `p` → `SMA(SMA(x,(p+1)/2),(p+1)/2)`; even →
/// `SMA(SMA(x,p/2),p/2+1)`. The leading `period - 1` positions are [`f64::NAN`].
///
/// # 示例 / Example
/// ```rust
/// use adaq_talib::overlap::trima;
/// let v = [1.0, 2.0, 3.0, 4.0, 5.0];
/// let out = trima(&v, 4).unwrap();
/// // TRIMA(4): 权重 (1,2,2,1)/6 -> ((1+2*2+2*3+4)/6) = 15/6 = 2.5
/// assert!((out[3] - 2.5).abs() < 1e-9);
/// ```
pub fn trima(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; values.len()];
    trima_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 三角移动平均，零拷贝写入 `out`（与 `values` 等长）。见 [`trima`]。
/// Triangular Moving Average, written zero-copy into `out`. See [`trima`].
pub fn trima_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    let n = values.len();
    if out.len() != n {
        return Err(TaError::BadParam(
            "trima_with_output: out length must equal values length".into(),
        ));
    }
    // 输入长度 < period：原版返回全 NAN（此处 `out` 已由调用方预填 NAN）。
    // Input shorter than period: original returns all-NaN (caller pre-fills NAN).
    if n < time_period {
        return Ok(());
    }
    let (p1, p2) = if time_period % 2 == 1 {
        let h = (time_period + 1) / 2;
        (h, h)
    } else {
        let h = time_period / 2;
        (h, h + 1)
    };
    let inner = rolling_mean(values, p1);
    let res = rolling_mean_skip(&inner, p2);
    out.copy_from_slice(&res);
    Ok(())
}

/// 三角移动平均，使用 TA-Lib 默认周期（30）。
/// Triangular MA with TA-Lib's default period (30).
pub fn trima_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    trima(values, DEFAULT_TIME_PERIOD)
}

// ───────────────────────────── T3 ─────────────────────────────

/// 三指数移动平均（T3，Tillson，TA-Lib `TA_T3`）。
///
/// Triple Exponential Moving Average (T3, Tillson). Six nested EMAs combined with the
/// v-factor coefficients `c1..c4`. The leading `6*(period-1)` positions are [`f64::NAN`].
///
/// # 参数 / Parameters
/// - `time_period`：窗口长度 `period >= 1`（TA-Lib 默认 5）。/ Window length (default 5).
/// - `v_factor`：平滑因子（TA-Lib 默认 0.7）。/ Smoothing factor (default 0.7).
///
/// # 公式 / Formula
/// ```text
/// e1 = EMA(x); e2 = EMA(e1); ... e6 = EMA(e5)   (each period `period`)
/// c1 = -v^3;  c2 = 3v^2 + 3v^3;  c3 = -6v^2 - 3v - 3v^3;  c4 = 3v^2 + 3v + 1 + v^3
/// T3 = c1*e6 + c2*e5 + c3*e4 + c4*e3
/// ```
/// 三指数移动平均（T3，Tillson），零拷贝写入 `out`（与 `values` 等长）。
///
/// Triple Exponential Moving Average (T3, Tillson) — TA-Lib `TA_T3` — written zero-copy
/// into `out`. Reuses the [`crate::core::nested_ema_with_output`] single-pass fused kernel
/// (ADR 0010 P2-1) with `L = 6`, eliminating the 6 independent `ema` scans and `Vec`
/// allocations of the naive version. Numerically identical to [`t3`] (1:1 with TA-Lib).
///
/// `out` 长度必须等于 `values.len()`，否则返回 [`TaError::BadParam`]。
/// `out` must have length equal to `values.len()`; otherwise [`TaError::BadParam`] is returned.
pub fn t3_with_output(
    values: &[f64],
    time_period: usize,
    v_factor: f64,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "t3_with_output: out length must equal values length".into(),
        ));
    }
    let v = v_factor;
    let c1 = -v * v * v;
    let c2 = 3.0 * (v * v - c1);
    let c3 = -6.0 * v * v - 3.0 * (v - c1);
    let c4 = (3.0 * v * v + 3.0 * v + 1.0) - c1;
    crate::core::nested_ema_with_output::<6, _>(
        values,
        time_period,
        |e| c1 * e[5] + c2 * e[4] + c3 * e[3] + c4 * e[2],
        out,
    );
    Ok(())
}

/// 三指数移动平均（T3，Tillson，TA-Lib `TA_T3`）。
///
/// Triple Exponential Moving Average (T3, Tillson). Six nested EMAs combined with the
/// v-factor coefficients `c1..c4`. The leading `6*(period-1)` positions are [`f64::NAN`].
///
/// # 参数 / Parameters
/// - `time_period`：窗口长度 `period >= 1`（TA-Lib 默认 5）。/ Window length (default 5).
/// - `v_factor`：平滑因子（TA-Lib 默认 0.7）。/ Smoothing factor (default 0.7).
///
/// # 公式 / Formula
/// ```text
/// e1 = EMA(x); e2 = EMA(e1); ... e6 = EMA(e5)   (each period `period`)
/// c1 = -v^3;  c2 = 3v^2 + 3v^3;  c3 = -6v^2 - 3v - 3v^3;  c4 = 3v^2 + 3v + 1 + v^3
/// T3 = c1*e6 + c2*e5 + c3*e4 + c4*e3
/// ```
pub fn t3(values: &[f64], time_period: usize, v_factor: f64) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; values.len()];
    t3_with_output(values, time_period, v_factor, &mut out)?;
    Ok(out)
}

/// T3，使用 TA-Lib 默认参数（周期 5、v 因子 0.7）。
/// T3 with TA-Lib defaults (period 5, v-factor 0.7).
pub fn t3_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    use crate::core::defaults::T3_VFACTOR;
    t3(values, 5, T3_VFACTOR)
}

// ───────────────────────────── MA ─────────────────────────────

/// 通用移动平均（General Moving Average，TA-Lib `TA_MA`）。
///
/// General Moving Average (TA-Lib `TA_MA`). Dispatches to the selected [`MaType`].
/// For `Kama`, `time_period` is the efficiency-ratio window; for `Mama`, `time_period` is
/// ignored (the Hilbert transform has no period parameter).
///
/// # 返回值 / Returns
/// 与 `values` 等长的向量，前导不稳定期为所选类型的 lookback（DEMA=2(p-1)、TEMA=3(p-1)、
/// T3=6(p-1)、KAMA=p、MAMA=32，其余 p-1）。
/// Equal-length vector; leading unstable period is the selected type's lookback.
pub fn ma(values: &[f64], time_period: usize, ma_type: MaType) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; values.len()];
    ma_with_output(values, time_period, ma_type, &mut out)?;
    Ok(out)
}

/// 通用移动平均，零拷贝写入 `out`（与 `values` 等长）。见 [`ma`]。
/// General Moving Average, written zero-copy into `out`. See [`ma`].
pub fn ma_with_output(
    values: &[f64],
    time_period: usize,
    ma_type: MaType,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "ma_with_output: out length must equal values length".into(),
        ));
    }
    let res = match ma_type {
        MaType::Sma => sma(values, time_period)?,
        MaType::Ema => ema(values, time_period)?,
        MaType::Wma => wma(values, time_period)?,
        MaType::Dema => dema(values, time_period)?,
        MaType::Tema => tema(values, time_period)?,
        MaType::Trima => trima(values, time_period)?,
        MaType::Kama => kama(values, time_period)?,
        MaType::Mama => crate::cycle::mama_default(values)?.mama,
    };
    out.copy_from_slice(&res);
    Ok(())
}

/// 通用移动平均，使用 TA-Lib 默认参数（周期 30、SMA）。
/// General MA with TA-Lib defaults (period 30, SMA).
pub fn ma_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    ma(values, DEFAULT_TIME_PERIOD, MaType::Sma)
}

// ───────────────────────────── MAVP ─────────────────────────────

/// 变周期移动平均（Variable-period MA，TA-Lib `TA_MAVP`）。
///
/// Variable-period Moving Average (TA-Lib `TA_MAVP`). Each output bar `i` uses the moving
/// average of `values` with period `clamp(round(periods[i]), min_period, max_period)`
/// (floored at 1). The output is contiguous from index `MA_lookback(max_period)` onward
/// (earlier positions are [`f64::NAN`]).
///
/// # 参数 / Parameters
/// - `values`：输入序列 `&[f64]`（与 `periods` 等长）。/ Input series (same length as `periods`).
/// - `periods`：每个 bar 的周期数组 `&[f64]`。/ Per-bar period array.
/// - `min_period` / `max_period`：周期裁剪范围（TA-Lib 默认 2 / 30）。/ Clamp range (default 2/30).
/// - `ma_type`：移动平均类型（见 [`MaType`]）。
pub fn mavp(
    values: &[f64],
    periods: &[f64],
    min_period: usize,
    max_period: usize,
    ma_type: MaType,
) -> Result<Vec<f64>, TaError> {
    check_period(min_period)?;
    check_period(max_period)?;
    let mut out = vec![f64::NAN; values.len()];
    mavp_with_output(values, periods, min_period, max_period, ma_type, &mut out)?;
    Ok(out)
}

/// 变周期移动平均，零拷贝写入 `out`（与 `values` 等长）。见 [`mavp`]。
/// Variable-period Moving Average, written zero-copy into `out`. See [`mavp`].
pub fn mavp_with_output(
    values: &[f64],
    periods: &[f64],
    min_period: usize,
    max_period: usize,
    ma_type: MaType,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(min_period)?;
    check_period(max_period)?;
    if min_period > max_period {
        return Err(TaError::BadParam(
            "mavp: min_period must be <= max_period".into(),
        ));
    }
    let n = values.len();
    if periods.len() != n {
        return Err(TaError::BadParam(
            "mavp: periods must have the same length as values".into(),
        ));
    }
    if out.len() != n {
        return Err(TaError::BadParam(
            "mavp_with_output: out length must equal values length".into(),
        ));
    }
    out.fill(f64::NAN);
    let mut p: Vec<usize> = Vec::with_capacity(n);
    for &x in periods {
        let mut v = x.trunc() as i64;
        if v < min_period as i64 {
            v = min_period as i64;
        } else if v > max_period as i64 {
            v = max_period as i64;
        }
        if v < 1 {
            v = 1;
        }
        p.push(v as usize);
    }
    let lookback = ma_lookback(max_period, ma_type);
    let mut distinct: Vec<usize> = p.iter().copied().collect();
    distinct.sort_unstable();
    distinct.dedup();
    for &pp in &distinct {
        let ma_series = ma(values, pp, ma_type)?;
        for i in lookback..n {
            if p[i] == pp {
                out[i] = ma_series[i];
            }
        }
    }
    Ok(())
}

/// 变周期移动平均，使用 TA-Lib 默认参数（min 2 / max 30、SMA）。
/// Variable-period MA with TA-Lib defaults (min 2 / max 30, SMA).
pub fn mavp_default(values: &[f64], periods: &[f64]) -> Result<Vec<f64>, TaError> {
    use crate::core::defaults::{MAVP_MAX_PERIOD, MAVP_MIN_PERIOD};
    mavp(
        values,
        periods,
        MAVP_MIN_PERIOD,
        MAVP_MAX_PERIOD,
        MaType::Sma,
    )
}

// ───────────────────────────── KAMA ─────────────────────────────

/// Kaufman 自适应移动平均（Kaufman Adaptive MA，TA-Lib `TA_KAMA`）。
///
/// Kaufman Adaptive Moving Average (KAMA). The smoothing constant adapts to the
/// efficiency ratio (ER) between a fast SC (`2/(2+1)`) and a slow SC (`2/(30+1)`); `time_period`
/// is the ER window (TA-Lib default 30). The first output is at index `time_period`
/// (lookback = `time_period`).
///
/// # 参数 / Parameters
/// - `time_period`：效率比窗口 `period >= 1`（TA-Lib 默认 30）。/ ER window (default 30).
///
/// # 示例 / Example
/// ```rust
/// use adaq_talib::overlap::kama;
/// let v: Vec<f64> = (0..60).map(|i| 100.0 + (i as f64)).collect();
/// let out = kama(&v, 30).unwrap();
/// // 前 30 个为 NaN；第 30 个起为有效 KAMA / first 30 are NaN, valid from index 30
/// assert!(out[0].is_nan());
/// assert!(!out[30].is_nan());
/// ```
pub fn kama(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; values.len()];
    kama_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// Kaufman 自适应移动平均，零拷贝写入 `out`（与 `values` 等长）。见 [`kama`]。
/// Kaufman Adaptive MA, written zero-copy into `out`. See [`kama`].
pub fn kama_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    let n = values.len();
    if out.len() != n {
        return Err(TaError::BadParam(
            "kama_with_output: out length must equal values length".into(),
        ));
    }
    out.fill(f64::NAN);
    if time_period == 1 {
        out.copy_from_slice(values);
        return Ok(());
    }
    let p = time_period;
    // 输入长度 <= period：原版返回全 NAN（此处 `out` 已填 NAN）。
    // Input shorter than or equal to period: original returns all-NaN (out is NAN-filled).
    if n <= p {
        return Ok(());
    }
    let const_max = 2.0 / 31.0;
    let const_diff = 2.0 / 3.0 - const_max;
    let mut sum_roc1 = 0.0_f64;
    let mut today = 0usize;
    let mut trailing_idx = 0usize;
    let mut i = p;
    while i > 0 {
        i -= 1;
        let tr = values[today];
        let diff = tr - values[today + 1];
        sum_roc1 += diff.abs();
        today += 1;
    }
    let mut prev_kama = values[today - 1];
    let temp = values[today];
    let temp2 = values[trailing_idx];
    trailing_idx += 1;
    let period_roc = temp - temp2;
    let mut trailing_value = temp2;
    let er = if sum_roc1 <= period_roc || sum_roc1.abs() < 1e-14 {
        1.0
    } else {
        (period_roc / sum_roc1).abs()
    };
    let mut sc = er * const_diff + const_max;
    sc *= sc;
    prev_kama = (values[today] - prev_kama).mul_add(sc, prev_kama);
    today += 1;
    out[p] = prev_kama;
    while today <= n - 1 {
        let temp = values[today];
        let temp2 = values[trailing_idx];
        trailing_idx += 1;
        let period_roc = temp - temp2;
        sum_roc1 -= (trailing_value - temp2).abs();
        sum_roc1 += (temp - values[today - 1]).abs();
        trailing_value = temp2;
        let er = if sum_roc1 <= period_roc || sum_roc1.abs() < 1e-14 {
            1.0
        } else {
            (period_roc / sum_roc1).abs()
        };
        let mut sc = er * const_diff + const_max;
        sc *= sc;
        prev_kama = (values[today] - prev_kama).mul_add(sc, prev_kama);
        out[today] = prev_kama;
        today += 1;
    }
    Ok(())
}

/// Kaufman 自适应移动平均，使用 TA-Lib 默认周期（30）。
/// Kaufman Adaptive MA with TA-Lib's default period (30).
pub fn kama_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    kama(values, DEFAULT_TIME_PERIOD)
}

// ────────────────────────── SAR / SAREXT ──────────────────────────

/// 抛物线转向（Parabolic SAR，TA-Lib `TA_SAR`）。
///
/// Parabolic SAR (TA-Lib `TA_SAR`). Standard Wilder SAR: the initial direction is derived
/// from the directional movement between the first two bars (tie → long), the first SAR is
/// placed at index 1 (lookback = 1), and `acceleration` / `maximum` bound the step factor.
///
/// # 参数 / Parameters
/// - `high` / `low`：最高/最低价序列（等长）。/ High/Low series (equal length).
/// - `acceleration`：加速因子（TA-Lib 默认 0.02）。/ Acceleration factor (default 0.02).
/// - `maximum`：加速因子上限（TA-Lib 默认 0.2）。/ Max acceleration (default 0.2).
///
/// # 返回值 / Returns
/// 与输入等长；`out[0]` 为 [`f64::NAN`]，其余为各 bar 的 SAR。
/// Equal length; `out[0]` is [`f64::NAN`], the rest are per-bar SAR values.
pub fn sar(
    high: &[f64],
    low: &[f64],
    acceleration: f64,
    maximum: f64,
) -> Result<Vec<f64>, TaError> {
    if high.len() != low.len() {
        return Err(TaError::BadParam("sar: high and low must have equal length".into()));
    }
    let mut out = vec![f64::NAN; high.len()];
    sar_with_output(high, low, acceleration, maximum, &mut out)?;
    Ok(out)
}

/// 抛物线转向，零拷贝写入 `out`（与 `high`/`low` 等长）。见 [`sar`]。
/// Parabolic SAR, written zero-copy into `out`. See [`sar`].
pub fn sar_with_output(
    high: &[f64],
    low: &[f64],
    acceleration: f64,
    maximum: f64,
    out: &mut [f64],
) -> Result<(), TaError> {
    if high.len() != low.len() {
        return Err(TaError::BadParam("sar: high and low must have equal length".into()));
    }
    let n = high.len();
    if out.len() != n {
        return Err(TaError::BadParam(
            "sar_with_output: out length must equal high/low length".into(),
        ));
    }
    if n < 2 {
        // 原版返回全 NAN（此处 `out` 已由调用方预填 NAN）。
        // Original returns all-NaN (caller pre-fills NAN).
        return Ok(());
    }
    let mut af = acceleration;
    if af > maximum {
        af = maximum;
    }
    // Initial direction from the first two bars (tie -> long).
    let up_move = high[1] - high[0];
    let down_move = low[0] - low[1];
    let mut is_long = !(down_move > up_move);
    let mut today = 1usize;
    let mut new_high = high[0];
    let mut new_low = low[0];
    let (mut ep, mut sar);
    if is_long {
        ep = high[1];
        sar = new_low;
    } else {
        ep = low[1];
        sar = new_high;
    }
    new_low = low[1];
    new_high = high[1];
    while today <= n - 1 {
        let bar = today;
        let prev_low = new_low;
        let prev_high = new_high;
        new_low = low[today];
        new_high = high[today];
        today += 1;
        if is_long {
            if new_low <= sar {
                is_long = false;
                sar = ep;
                if sar < prev_high {
                    sar = prev_high;
                }
                if sar < new_high {
                    sar = new_high;
                }
                out[bar] = sar;
                af = acceleration;
                ep = new_low;
                sar = af * (ep - sar) + sar;
                if sar < prev_high {
                    sar = prev_high;
                }
                if sar < new_high {
                    sar = new_high;
                }
            } else {
                out[bar] = sar;
                if new_high > ep {
                    ep = new_high;
                    af += acceleration;
                    if af > maximum {
                        af = maximum;
                    }
                }
                sar = af * (ep - sar) + sar;
                if sar > prev_low {
                    sar = prev_low;
                }
                if sar > new_low {
                    sar = new_low;
                }
            }
        } else if new_high >= sar {
            is_long = true;
            sar = ep;
            if sar > prev_low {
                sar = prev_low;
            }
            if sar > new_low {
                sar = new_low;
            }
            out[bar] = sar;
            af = acceleration;
            ep = new_high;
            sar = af * (ep - sar) + sar;
            if sar > prev_low {
                sar = prev_low;
            }
            if sar > new_low {
                sar = new_low;
            }
        } else {
            out[bar] = sar;
            if new_low < ep {
                ep = new_low;
                af += acceleration;
                if af > maximum {
                    af = maximum;
                }
            }
            sar = af * (ep - sar) + sar;
            if sar < prev_high {
                sar = prev_high;
            }
            if sar < new_high {
                sar = new_high;
            }
        }
    }
    Ok(())
}

/// 抛物线转向，使用 TA-Lib 默认参数（加速 0.02 / 上限 0.2）。
/// Parabolic SAR with TA-Lib defaults (acceleration 0.02 / max 0.2).
pub fn sar_default(high: &[f64], low: &[f64]) -> Result<Vec<f64>, TaError> {
    use crate::core::defaults::{SAR_ACCELERATION, SAR_MAX};
    sar(high, low, SAR_ACCELERATION, SAR_MAX)
}

/// 扩展抛物线转向（Parabolic SAR Extended，TA-Lib `TA_SAREXT`）。
///
/// Parabolic SAR Extended (TA-Lib `TA_SAREXT`). Like [`sar`] but with separate long/short
/// acceleration factors and an optional `offset_on_reverse`. **Short-side SAR values are
/// returned as negatives** (so a reversal is distinguishable), matching TA-Lib. Lookback = 1.
///
/// # 参数 / Parameters
/// - `start_value`：强制初始方向/位置；`0` 用默认（DM 判定），`>0` 强制多头于该值，`<0` 强制空头于 `|值|`。
/// - `offset_on_reverse`：反转时的偏移比例（TA-Lib 默认 0）。/ Offset on reversal (default 0).
/// - `accel_init_long` / `accel_long` / `accel_max_long`：多头初始/步进/上限加速（默认 0.02/0.02/0.2）。
/// - `accel_init_short` / `accel_short` / `accel_max_short`：空头对应参数（默认 0.02/0.02/0.2）。
pub fn sarext(
    high: &[f64],
    low: &[f64],
    start_value: f64,
    offset_on_reverse: f64,
    accel_init_long: f64,
    accel_long: f64,
    accel_max_long: f64,
    accel_init_short: f64,
    accel_short: f64,
    accel_max_short: f64,
) -> Result<Vec<f64>, TaError> {
    if high.len() != low.len() {
        return Err(TaError::BadParam(
            "sarext: high and low must have equal length".into(),
        ));
    }
    let mut out = vec![f64::NAN; high.len()];
    sarext_with_output(
        high,
        low,
        start_value,
        offset_on_reverse,
        accel_init_long,
        accel_long,
        accel_max_long,
        accel_init_short,
        accel_short,
        accel_max_short,
        &mut out,
    )?;
    Ok(out)
}

/// 扩展抛物线转向，零拷贝写入 `out`（与 `high`/`low` 等长）。见 [`sarext`]。
/// Parabolic SAR Extended, written zero-copy into `out`. See [`sarext`].
pub fn sarext_with_output(
    high: &[f64],
    low: &[f64],
    start_value: f64,
    offset_on_reverse: f64,
    accel_init_long: f64,
    accel_long: f64,
    accel_max_long: f64,
    accel_init_short: f64,
    accel_short: f64,
    accel_max_short: f64,
    out: &mut [f64],
) -> Result<(), TaError> {
    if high.len() != low.len() {
        return Err(TaError::BadParam(
            "sarext: high and low must have equal length".into(),
        ));
    }
    let n = high.len();
    if out.len() != n {
        return Err(TaError::BadParam(
            "sarext_with_output: out length must equal high/low length".into(),
        ));
    }
    if n < 2 {
        // 原版返回全 NAN（此处 `out` 已由调用方预填 NAN）。
        // Original returns all-NaN (caller pre-fills NAN).
        return Ok(());
    }
    let mut af_long = accel_init_long;
    let mut af_short = accel_init_short;
    if af_long > accel_max_long {
        af_long = accel_max_long;
    }
    if af_short > accel_max_short {
        af_short = accel_max_short;
    }
    let mut is_long: bool;
    if start_value == 0.0 {
        let up_move = high[1] - high[0];
        let down_move = low[0] - low[1];
        is_long = !(down_move > up_move);
    } else if start_value > 0.0 {
        is_long = true;
    } else {
        is_long = false;
    }
    let mut today = 1usize;
    let mut new_high = high[0];
    let mut new_low = low[0];
    let (mut ep, mut sar);
    if start_value == 0.0 {
        if is_long {
            ep = high[1];
            sar = new_low;
        } else {
            ep = low[1];
            sar = new_high;
        }
    } else if start_value > 0.0 {
        ep = high[1];
        sar = start_value;
    } else {
        ep = low[1];
        sar = start_value.abs();
    }
    new_low = low[1];
    new_high = high[1];
    while today <= n - 1 {
        let bar = today;
        let prev_low = new_low;
        let prev_high = new_high;
        new_low = low[today];
        new_high = high[today];
        today += 1;
        if is_long {
            if new_low <= sar {
                is_long = false;
                sar = ep;
                if sar < prev_high {
                    sar = prev_high;
                }
                if sar < new_high {
                    sar = new_high;
                }
                if offset_on_reverse != 0.0 {
                    sar += sar * offset_on_reverse;
                }
                out[bar] = 0.0 - sar;
                af_short = accel_init_short;
                ep = new_low;
                sar = af_short * (ep - sar) + sar;
                if sar < prev_high {
                    sar = prev_high;
                }
                if sar < new_high {
                    sar = new_high;
                }
            } else {
                out[bar] = sar;
                if new_high > ep {
                    ep = new_high;
                    af_long += accel_long;
                    if af_long > accel_max_long {
                        af_long = accel_max_long;
                    }
                }
                sar = af_long * (ep - sar) + sar;
                if sar > prev_low {
                    sar = prev_low;
                }
                if sar > new_low {
                    sar = new_low;
                }
            }
        } else if new_high >= sar {
            is_long = true;
            sar = ep;
            if sar > prev_low {
                sar = prev_low;
            }
            if sar > new_low {
                sar = new_low;
            }
            if offset_on_reverse != 0.0 {
                sar -= sar * offset_on_reverse;
            }
            out[bar] = sar;
            af_long = accel_init_long;
            ep = new_high;
            sar = af_long * (ep - sar) + sar;
            if sar > prev_low {
                sar = prev_low;
            }
            if sar > new_low {
                sar = new_low;
            }
        } else {
            out[bar] = 0.0 - sar;
            if new_low < ep {
                ep = new_low;
                af_short += accel_short;
                if af_short > accel_max_short {
                    af_short = accel_max_short;
                }
            }
            sar = af_short * (ep - sar) + sar;
            if sar < prev_high {
                sar = prev_high;
            }
            if sar < new_high {
                sar = new_high;
            }
        }
    }
    Ok(())
}

/// 扩展抛物线转向，使用 TA-Lib 默认参数。
/// Parabolic SAR Extended with TA-Lib defaults.
pub fn sarext_default(high: &[f64], low: &[f64]) -> Result<Vec<f64>, TaError> {
    use crate::core::defaults::*;
    sarext(
        high,
        low,
        SAREXT_START_VALUE,
        SAREXT_OFFSET_ON_REVERSE,
        SAREXT_ACCEL_INIT_LONG,
        SAREXT_ACCEL_LONG,
        SAREXT_ACCEL_MAX_LONG,
        SAREXT_ACCEL_INIT_SHORT,
        SAREXT_ACCEL_SHORT,
        SAREXT_ACCEL_MAX_SHORT,
    )
}

// ──────────────────────────── 单元测试 ────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sma_basic() {
        let v = [1.0, 2.0, 3.0, 4.0, 5.0];
        let out = sma(&v, 3).unwrap();
        assert!(out[0].is_nan());
        assert!(out[1].is_nan());
        assert!((out[2] - 2.0).abs() < 1e-12);
        assert!((out[3] - 3.0).abs() < 1e-12);
        assert!((out[4] - 4.0).abs() < 1e-12);
    }

    #[test]
    fn sma_period_zero_is_error() {
        assert!(matches!(sma(&[1.0, 2.0], 0), Err(TaError::BadParam(_))));
    }

    #[test]
    fn sma_short_input_all_nan() {
        let out = sma(&[1.0, 2.0], 5).unwrap();
        assert!(out.iter().all(|x| x.is_nan()));
    }

    #[test]
    fn ema_seed_and_recursion() {
        // 输入 powers of two，period=3 -> 种子 = (1+2+4)/3 = 7/3，k = 0.5
        // seed = (1+2+4)/3 = 7/3, k = 0.5
        let v = [1.0, 2.0, 4.0, 8.0, 16.0];
        let out = ema(&v, 3).unwrap();
        assert!(out[0].is_nan() && out[1].is_nan());
        assert!((out[2] - 7.0 / 3.0).abs() < 1e-12);
        // out[3] = (8 - 7/3)*0.5 + 7/3 = 31/6
        assert!((out[3] - 31.0 / 6.0).abs() < 1e-12);
    }

    #[test]
    fn wma_weights() {
        // 窗口 [1,2,4]，权重 3,2,1，归一化 /6 -> (12+4+1)/6 = 17/6
        let v = [1.0, 2.0, 4.0];
        let out = wma(&v, 3).unwrap();
        assert!((out[2] - 17.0 / 6.0).abs() < 1e-12);
    }

    #[test]
    fn dema_first_valid_index() {
        let v: Vec<f64> = (0..12).map(|i| 2f64.powi(i)).collect();
        let out = dema(&v, 3).unwrap();
        let lookback = 2 * (3 - 1);
        for i in 0..lookback {
            assert!(out[i].is_nan(), "index {i} should be NaN");
        }
        assert!(!out[lookback].is_nan());
    }

    #[test]
    fn tema_first_valid_index() {
        let v: Vec<f64> = (0..14).map(|i| 2f64.powi(i)).collect();
        let out = tema(&v, 3).unwrap();
        let lookback = 3 * (3 - 1);
        for i in 0..lookback {
            assert!(out[i].is_nan(), "index {i} should be NaN");
        }
        assert!(!out[lookback].is_nan());
    }

    #[test]
    fn midpoint_min_max_half() {
        let v = [1.0, 5.0, 3.0, 9.0, 2.0];
        let out = midpoint(&v, 3).unwrap();
        assert!(out[0].is_nan() && out[1].is_nan());
        assert!((out[2] - (5.0 + 1.0) / 2.0).abs() < 1e-12);
        assert!((out[3] - (9.0 + 3.0) / 2.0).abs() < 1e-12);
    }

    #[test]
    fn midprice_high_low() {
        let high = [5.0, 6.0, 7.0, 8.0];
        let low = [1.0, 2.0, 3.0, 4.0];
        let out = midprice(&high, &low, 3).unwrap();
        assert!(out[0].is_nan() && out[1].is_nan());
        assert!((out[2] - (7.0 + 1.0) / 2.0).abs() < 1e-12);
    }

    #[test]
    fn midprice_length_mismatch_is_error() {
        assert!(matches!(
            midprice(&[1.0, 2.0], &[1.0], 2),
            Err(TaError::BadParam(_))
        ));
    }
}
