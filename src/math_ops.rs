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

use crate::core::{check_eq_len, rolling_max, rolling_min, rolling_sum};
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
/// （0 基；平局取最左）。前导 `period-1` 个为 [`f64::NAN`]。
/// Index of the rolling-window maximum (TA-Lib `TA_MAXINDEX`), the absolute (0-based) position
/// of the max in the window (leftmost on ties). The leading `period - 1` positions are `NaN`.
pub fn max_index(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    Ok(rolling_extreme_index(values, time_period, true))
}

/// 滚动窗口最小值的**索引**（TA-Lib `TA_MININDEX`），返回窗口内最小值的绝对位置
/// （0 基；平局取最左）。前导 `period-1` 个为 [`f64::NAN`]。
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
pub fn minmax(values: &[f64], time_period: usize) -> Result<MinMax, TaError> {
    check_period(time_period)?;
    let mn = rolling_min(values, time_period);
    let mx = rolling_max(values, time_period);
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

/// 滚动窗口最小/最大值索引（TA-Lib `TA_MINMAXINDEX`）。前导 `period-1` 为 [`f64::NAN`]。
/// Rolling window min/max indices (TA-Lib `TA_MINMAXINDEX`). The leading `period - 1` positions are `NaN`.
pub fn minmax_index(values: &[f64], time_period: usize) -> Result<MinMaxIndex, TaError> {
    check_period(time_period)?;
    let n = values.len();
    // 前导 `period-1` 个位置 TA-Lib 返回 **0.0**（与原版一致）。
    // The leading `period - 1` positions return **0.0** in TA-Lib (not `NaN`).
    let mut min_idx = vec![0.0; n];
    let mut max_idx = vec![0.0; n];
    if n >= time_period {
        for i in (time_period - 1)..n {
            let mut bmin = values[i];
            let mut bmini = i;
            let mut bmax = values[i];
            let mut bmaxi = i;
            for j in 1..time_period {
                let v = values[i - j];
                // 平局取最左（leftmost）。/ Leftmost on ties.
                if v <= bmin {
                    bmin = v;
                    bmini = i - j;
                }
                if v >= bmax {
                    bmax = v;
                    bmaxi = i - j;
                }
            }
            min_idx[i] = bmini as f64;
            max_idx[i] = bmaxi as f64;
        }
    }
    Ok(MinMaxIndex { min_idx, max_idx })
}

/// 滚动窗口极值索引的内部实现（共享于 `max_index` / `min_index`）。平局取最左。
/// 前导 `period-1` 个位置 TA-Lib 返回 **0.0**（与原版一致，非 `NaN`）。
/// Shared rolling-extreme-index core for `max_index` / `min_index`. Leftmost on ties.
/// The leading `period - 1` positions return **0.0** in TA-Lib (not `NaN`), matching the original.
fn rolling_extreme_index(values: &[f64], period: usize, take_max: bool) -> Vec<f64> {
    let n = values.len();
    let mut out = vec![0.0; n];
    if n < period {
        return out;
    }
    for i in (period - 1)..n {
        let mut best = values[i];
        let mut best_idx = i;
        for j in 1..period {
            let v = values[i - j];
            // 平局取最左（leftmost）：遇到相等极值也更新到更小索引。
            // Leftmost on ties: update even on an equal extreme to keep the smaller index.
            let better = if take_max { v >= best } else { v <= best };
            if better {
                best = v;
                best_idx = i - j;
            }
        }
        out[i] = best_idx as f64;
    }
    out
}
