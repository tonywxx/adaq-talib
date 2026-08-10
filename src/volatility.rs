//! 波动率指标（Volatility Indicators）。
//!
//! Volatility Indicators.
//!
//! 本模块全部函数的数值输出与 [TA-Lib](https://ta-lib.org) 0.7.1 逐项一致（浮点误差容限内，
//! 见 [`crate::utils`] 与 ADR 0005）。前导不稳定期以 [`f64::NAN`] 填充、等长返回（见 ADR 0007）。
//!
//! Every function in this module reproduces the numerical output of TA-Lib 0.7.1 (within the
//! float tolerance in ADR 0005). The leading unstable period is filled with [`f64::NAN`] and
//! returned at equal length (ADR 0007).

use crate::core::defaults::ATR_PERIOD;
use crate::core::{check_eq_len, ema_wilder, true_range};
use crate::error::{check_period, TaError};

// ──────────────────────────── TRANGE ────────────────────────────

/// 真实波幅（True Range，TA-Lib `TA_TRANGE`）。
///
/// `TR[0] = high[0] - low[0]`；`TR[i] = max(high[i], close[i-1]) - min(low[i], close[i-1])`。
/// 无滞后（lookback 0），与输入等长；首个值即为 `high[0]-low[0]`（不含前导 NaN）。
///
/// # 参数 / Parameters
/// - `high` / `low` / `close`：最高/最低/收盘价序列，长度须一致。
///   High/Low/Close series, equal length required.
///
/// # 返回值 / Returns
/// 与输入等长的向量；无前导 NaN（TA-Lib `TA_TRANGE` lookback 为 0）。
///
/// # 示例 / Example
/// ```
/// use adaq_talib::volatility::trange;
/// let high = [10.0, 11.0, 12.0];
/// let low  = [9.0, 9.5, 11.0];
/// let close = [9.5, 10.5, 11.5];
/// let tr = trange(&high, &low, &close).unwrap();
/// assert!((tr[0] - 1.0).abs() < 1e-9); // high[0]-low[0]
/// ```
pub fn trange(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[high, low, close], "trange")?;
    Ok(true_range(high, low, close))
}

// ──────────────────────────── ATR ────────────────────────────

/// 平均真实波幅（Average True Range，TA-Lib `TA_ATR`）。
///
/// 对真实波幅（TR）做 Wilder 平滑（SMMA，`k = 1/period`）：首个有效值 = 前 `period`
/// 个 TR 的算术均值（种子），其后按 `prev = prev + (tr - prev)/period` 递推。
/// 前导 `period-1` 个为 [`f64::NAN`]。
///
/// Wilder-smoothed (SMMA) average of True Range; the first valid value is the mean of the
/// first `period` TR values (seed), then recursed. The leading `period - 1` positions are
/// [`f64::NAN`].
///
/// # 示例 / Example
/// ```
/// use adaq_talib::volatility::atr;
/// let high = [10.0, 11.0, 12.0, 13.0, 14.0];
/// let low  = [9.0, 9.5, 11.0, 12.0, 13.0];
/// let close = [9.5, 10.5, 11.5, 12.5, 13.5];
/// let out = atr(&high, &low, &close, 3).unwrap();
/// assert!(out[0].is_nan() && out[1].is_nan());
/// assert!(!out[2].is_nan()); // 种子 = TR[0..2] 的均值 / seed = mean of TR[0..2]
/// ```
pub fn atr(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: usize,
) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "atr")?;
    let tr = true_range(high, low, close);
    Ok(ema_wilder(&tr, time_period))
}

/// `atr` 便捷版本，默认周期 14。/ `atr` with default period (14).
pub fn atr_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    atr(high, low, close, ATR_PERIOD)
}

// ──────────────────────────── NATR ────────────────────────────

/// 归一化平均真实波幅（Normalized ATR，TA-Lib `TA_NATR`）。
///
/// `NATR = 100 * ATR / close`（ATR 同样为 Wilder 平滑）。前导 `period-1` 个为 [`f64::NAN`]；
/// 若某位置 `close == 0`，对应 `NATR` 为 0.0（与 TA-Lib 一致，避免除零）。
///
/// `NATR = 100 * ATR / close`. The leading `period - 1` positions are [`f64::NAN`]; a zero
/// `close` yields `0.0` (matches TA-Lib, avoids division by zero).
pub fn natr(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: usize,
) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "natr")?;
    let tr = true_range(high, low, close);
    let atr_line = ema_wilder(&tr, time_period);
    let n = close.len();
    let mut out = vec![f64::NAN; n];
    for i in 0..n {
        if atr_line[i].is_nan() {
            continue;
        }
        out[i] = if close[i] == 0.0 {
            0.0
        } else {
            100.0 * atr_line[i] / close[i]
        };
    }
    Ok(out)
}

/// `natr` 便捷版本，默认周期 14。/ `natr` with default period (14).
pub fn natr_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    natr(high, low, close, ATR_PERIOD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trange_basic() {
        let high = [10.0, 11.0, 12.0];
        let low = [9.0, 9.5, 11.0];
        let close = [9.5, 10.5, 11.5];
        let tr = trange(&high, &low, &close).unwrap();
        // TR[0] = 10-9 = 1
        assert!((tr[0] - 1.0).abs() < 1e-12);
        // TR[1] = max(11,9.5)-min(9.5,10.5) = 11 - 9.5 = 1.5
        assert!((tr[1] - 1.5).abs() < 1e-12);
        // TR[2] = max(12,10.5)-min(11,10.5) = 12 - 10.5 = 1.5
        assert!((tr[2] - 1.5).abs() < 1e-12);
    }

    #[test]
    fn atr_wilder_seed() {
        // TR = [1.0, 1.5, 1.5, 1.5, 1.5]; period=3 -> 种子 = (1.0+1.5+1.5)/3 = 4/3
        let high = [10.0, 11.0, 12.0, 13.0, 14.0];
        let low = [9.0, 9.5, 11.0, 12.0, 13.0];
        let close = [9.5, 10.5, 11.5, 12.5, 13.5];
        let out = atr(&high, &low, &close, 3).unwrap();
        assert!(out[0].is_nan() && out[1].is_nan());
        let seed = (1.0 + 1.5 + 1.5) / 3.0;
        assert!((out[2] - seed).abs() < 1e-12);
        // out[3] = prev + (tr[3]-prev)/3 = seed + (1.5 - seed)/3
        let exp = seed + (1.5 - seed) / 3.0;
        assert!((out[3] - exp).abs() < 1e-12);
    }

    #[test]
    fn natr_ratio() {
        let high: Vec<f64> = (0..40).map(|i| 10.0 + i as f64 * 0.1).collect();
        let low: Vec<f64> = (0..40).map(|i| 9.0 + i as f64 * 0.1).collect();
        let close: Vec<f64> = (0..40).map(|i| 9.5 + i as f64 * 0.1).collect();
        let a = atr(&high, &low, &close, 14).unwrap();
        let n = natr(&high, &low, &close, 14).unwrap();
        // NATR = 100 * ATR / close where both valid
        let i = 30;
        assert!((n[i] - 100.0 * a[i] / close[i]).abs() < 1e-9);
    }

    #[test]
    fn trange_length_mismatch_is_error() {
        assert!(matches!(
            trange(&[1.0, 2.0], &[1.0], &[1.0, 2.0]),
            Err(TaError::BadParam(_))
        ));
    }
}
