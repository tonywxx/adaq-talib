//! 数学运算符函数（Math Operators）。
//!
//! Math Operators.
//!
//! 本模块提供逐元素二元运算（ADD / SUB / MULT / DIV）与滚动窗口运算
//! （MAX / MIN / SUM 及其索引变体 MINMAX / MINMAXINDEX），数值逐项对齐
//! [TA-Lib](https://ta-lib.org) 0.7.1（浮点误差容限内，见 [`crate::utils`] 与 ADR 0005）。
//! 滚动窗口函数的前导 `period-1` 个位置为 [`f64::NAN`]，与输入等长返回。
//!
//! This module provides element-wise binary ops (ADD / SUB / MULT / DIV) and rolling-window
//! ops (MAX / MIN / SUM and their index variants MINMAX / MINMAXINDEX), numerically 1:1 with
//! TA-Lib 0.7.1 (within ADR 0005). Rolling-window outputs carry `period - 1` leading `NaN`s.

use crate::core::{
    check_eq_len, rolling_extreme_index, rolling_max, rolling_min, rolling_minmax, rolling_sum,
};
use crate::error::{check_period, TaError};

/// 逐元素相加（TA-Lib `TA_ADD`）：`out = real0 + real1`，等长返回。
/// Element-wise addition (TA-Lib `TA_ADD`): `out = real0 + real1`; equal-length.
pub fn add(real0: &[f64], real1: &[f64]) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[real0, real1], "add")?;
    Ok(real0.iter().zip(real1).map(|(&a, &b)| a + b).collect())
}

/// 逐元素相减（TA-Lib `TA_SUB`）：`out = real0 - real1`，等长返回。
/// Element-wise subtraction (TA-Lib `TA_SUB`): `out = real0 - real1`; equal-length.
pub fn sub(real0: &[f64], real1: &[f64]) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[real0, real1], "sub")?;
    Ok(real0.iter().zip(real1).map(|(&a, &b)| a - b).collect())
}

/// 逐元素相乘（TA-Lib `TA_MULT`）：`out = real0 * real1`，等长返回。
/// Element-wise multiplication (TA-Lib `TA_MULT`): `out = real0 * real1`; equal-length.
pub fn mult(real0: &[f64], real1: &[f64]) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[real0, real1], "mult")?;
    Ok(real0.iter().zip(real1).map(|(&a, &b)| a * b).collect())
}

/// 逐元素相除（TA-Lib `TA_DIV`）：`out = real0 / real1`，等长返回；除零产生 `inf`/`NaN`。
/// Element-wise division (TA-Lib `TA_DIV`): `out = real0 / real1`; equal-length;
/// division by zero yields `inf`/`NaN`, matching the original.
pub fn div(real0: &[f64], real1: &[f64]) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[real0, real1], "div")?;
    Ok(real0.iter().zip(real1).map(|(&a, &b)| a / b).collect())
}

/// 滚动窗口最大值（TA-Lib `TA_MAX`）。前导 `period-1` 个为 [`f64::NAN`]。
/// Rolling maximum (TA-Lib `TA_MAX`). The leading `period - 1` positions are [`f64::NAN`].
pub fn max(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    Ok(rolling_max(values, time_period))
}

/// 滚动窗口最小值（TA-Lib `TA_MIN`）。前导 `period-1` 个为 [`f64::NAN`]。
/// Rolling minimum (TA-Lib `TA_MIN`). The leading `period - 1` positions are [`f64::NAN`].
pub fn min(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    Ok(rolling_min(values, time_period))
}

/// 滚动窗口求和（TA-Lib `TA_SUM`）。前导 `period-1` 个为 [`f64::NAN`]。
/// Rolling sum (TA-Lib `TA_SUM`). The leading `period - 1` positions are [`f64::NAN`].
pub fn sum(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    Ok(rolling_sum(values, time_period))
}

/// 滚动窗口最大值的**索引**（TA-Lib `TA_MAXINDEX`），返回窗口内最大值的绝对位置
/// （0 基；平局取最左）。前导 `period-1` 个为 **0.0**（与原版一致，非 `NaN`）。
/// Index of the rolling-window maximum (TA-Lib `TA_MAXINDEX`), the absolute (0-based) position
/// of the max in the window (leftmost on ties). The leading `period - 1` positions are `NaN`.
pub fn max_index(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    Ok(rolling_extreme_index(values, time_period, true))
}

/// 滚动窗口最小值的**索引**（TA-Lib `TA_MININDEX`），返回窗口内最小值的绝对位置
/// （0 基；平局取最左）。前导 `period-1` 个为 **0.0**（与原版一致，非 `NaN`）。
/// Index of the rolling-window minimum (TA-Lib `TA_MININDEX`), the absolute (0-based) position
/// of the min in the window (leftmost on ties). The leading `period - 1` positions are `NaN`.
pub fn min_index(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    Ok(rolling_extreme_index(values, time_period, false))
}

/// 滚动窗口最小/最大值的双向量结果（TA-Lib `TA_MINMAX`）。
/// Two-vector result of the rolling-window min/max (TA-Lib `TA_MINMAX`).
pub struct MinMax {
    /// 窗口最小值 / Window minimum.
    pub min: Vec<f64>,
    /// 窗口最大值 / Window maximum.
    pub max: Vec<f64>,
}

/// 滚动窗口最小/最大值（TA-Lib `TA_MINMAX`）。等长双向量，前导 `period-1` 为 [`f64::NAN`]。
/// Rolling window min/max (TA-Lib `TA_MINMAX`); two equal-length vectors, leading `period - 1` `NaN`.
///
/// 单遍实现：复用 `core::rolling_minmax` 一次遍历同时求得最大与最小（最右 tie-break、前导
/// `NaN` 与分别调用 `rolling_max`/`rolling_min` 逐位相等，见 `core::rolling_minmax` 文档），
/// 将原本的两次独立窗口扫描合并为一次，规避重复遍历开销（P1 候选②，ADR 0005 零偏差）。
pub fn minmax(values: &[f64], time_period: usize) -> Result<MinMax, TaError> {
    check_period(time_period)?;
    let (mx, mn) = rolling_minmax(values, time_period);
    Ok(MinMax { min: mn, max: mx })
}

/// 滚动窗口最小/最大值索引的双向量结果（TA-Lib `TA_MINMAXINDEX`），平局取最左。
/// Two-vector result of the rolling-window min/max indices (TA-Lib `TA_MINMAXINDEX`); leftmost on ties.
pub struct MinMaxIndex {
    /// 窗口最小值的绝对位置（0 基）/ Absolute (0-based) position of the window min.
    pub min_idx: Vec<f64>,
    /// 窗口最大值的绝对位置（0 基）/ Absolute (0-based) position of the window max.
    pub max_idx: Vec<f64>,
}

/// 滚动窗口最小/最大值索引（TA-Lib `TA_MINMAXINDEX`）。前导 `period-1` 为 **0.0**（与原版一致）。
/// Rolling window min/max indices (TA-Lib `TA_MINMAXINDEX`). The leading `period - 1` positions are **0.0**.
///
/// 复用 `core::rolling_extreme_index` 单遍单调队列（最左 tie-break）分别求最小/最大索引，
/// 将原本的 O(n·period) 嵌套扫描合并为两次 O(n) 遍历（候选③，ADR 0005 零偏差）。
/// Reuses `core::rolling_extreme_index` (single-pass, leftmost) for min and max — replacing the
/// naïve O(n·period) nested scan with two O(n) passes (candidate ③, ADR 0005 zero-deviation).
pub fn minmax_index(values: &[f64], time_period: usize) -> Result<MinMaxIndex, TaError> {
    check_period(time_period)?;
    let min_idx = rolling_extreme_index(values, time_period, false);
    let max_idx = rolling_extreme_index(values, time_period, true);
    Ok(MinMaxIndex { min_idx, max_idx })
}
