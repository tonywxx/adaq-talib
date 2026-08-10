//! 成交量指标（Volume Indicators）。
//!
//! Volume Indicators.
//!
//! 本模块全部函数的数值输出与 [TA-Lib](https://ta-lib.org) 0.7.1 逐项一致（浮点误差容限内，
//! 见 [`crate::utils`] 与 ADR 0005）。前导不稳定期以 [`f64::NAN`] 填充、等长返回（见 ADR 0007）。
//!
//! Every function in this module reproduces the numerical output of TA-Lib 0.7.1 (within the
//! float tolerance in ADR 0005). The leading unstable period is filled with [`f64::NAN`] and
//! returned at equal length (ADR 0007).

use crate::core::defaults::{ADOSC_FAST, ADOSC_SLOW};
use crate::core::{check_eq_len, ema};
use crate::error::{check_period, TaError};

/// 累积/派发线（Accumulation/Distribution Line，TA-Lib `TA_AD`）。
///
/// `CLV = (2*close - high - low) / (high - low)`（收盘价位置因子，Close Location Value）；
/// 若 `high == low` 则 `CLV = 0`。`AD` 为累计量：`AD[i] = AD[i-1] + volume[i]*CLV[i]`，
/// `AD[0] = volume[0]*CLV[0]`。无滞后（lookback 0）。
///
/// # 返回值 / Returns
/// 与输入等长的累计向量；无前导 NaN。
///
/// # 示例 / Example
/// ```
/// use adaq_talib::volume::ad;
/// let high = [10.0, 11.0, 12.0];
/// let low  = [9.0, 9.5, 11.0];
/// let close = [9.5, 10.5, 11.5];
/// let vol = [100.0, 200.0, 300.0];
/// let out = ad(&high, &low, &close, &vol).unwrap();
/// // CLV[0] = (19-10-9)/1 = 0 -> AD[0] = 0
/// assert!(out[0].abs() < 1e-9);
/// ```
pub fn ad(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[high, low, close, volume], "ad")?;
    let n = close.len();
    let mut out = vec![0.0_f64; n];
    let mut prev = 0.0;
    for i in 0..n {
        let clv = if high[i] == low[i] {
            0.0
        } else {
            (2.0 * close[i] - high[i] - low[i]) / (high[i] - low[i])
        };
        out[i] = prev + volume[i] * clv;
        prev = out[i];
    }
    Ok(out)
}

// ──────────────────────────── ADOSC ────────────────────────────

/// 累积/派发震荡器（Chaikin A/D Oscillator，TA-Lib `TA_ADOSC`）。
///
/// 先计算累计 A/D 线（见 [`ad`]），再对其分别做经典 `EMA(fast)` 与 `EMA(slow)`，
/// `ADOSC = EMA(fast) - EMA(slow)`。前导 `slow-1` 个为 [`f64::NAN`]。
///
/// Computes the cumulative A/D line (see [`ad`]), applies a classic `EMA(fast)` and
/// `EMA(slow)` to it, then `ADOSC = EMA(fast) - EMA(slow)`. The leading `slow - 1`
/// positions are [`f64::NAN`].
pub fn adosc(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
    fast_period: usize,
    slow_period: usize,
) -> Result<Vec<f64>, TaError> {
    check_period(fast_period)?;
    check_period(slow_period)?;
    if fast_period >= slow_period {
        return Err(TaError::BadParam("fast_period must be < slow_period".into()));
    }
    let ad_line = ad(high, low, close, volume)?;
    let ef = ema(&ad_line, fast_period);
    let es = ema(&ad_line, slow_period);
    let n = ad_line.len();
    let mut out = vec![f64::NAN; n];
    for i in 0..n {
        if !ef[i].is_nan() && !es[i].is_nan() {
            out[i] = ef[i] - es[i];
        }
    }
    Ok(out)
}

/// `adosc` 便捷版本，默认 3 / 10。/ `adosc` with defaults 3 / 10.
pub fn adosc_default(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<Vec<f64>, TaError> {
    adosc(high, low, close, volume, ADOSC_FAST, ADOSC_SLOW)
}

// ──────────────────────────── OBV ────────────────────────────

/// 能量潮（On Balance Volume，TA-Lib `TA_OBV`）。
///
/// `OBV[0] = volume[0]`；对 `i > 0`：若 `close[i] > close[i-1]` 则 `OBV[i] = OBV[i-1] + volume[i]`，
/// 若 `close[i] < close[i-1]` 则 `OBV[i] = OBV[i-1] - volume[i]`，否则持平。无滞后（lookback 0）。
///
/// # 返回值 / Returns
/// 与输入等长的累计向量；无前导 NaN。
pub fn obv(close: &[f64], volume: &[f64]) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[close, volume], "obv")?;
    let n = close.len();
    let mut out = vec![0.0_f64; n];
    if n == 0 {
        return Ok(out);
    }
    out[0] = volume[0];
    for i in 1..n {
        out[i] = if close[i] > close[i - 1] {
            out[i - 1] + volume[i]
        } else if close[i] < close[i - 1] {
            out[i - 1] - volume[i]
        } else {
            out[i - 1]
        };
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ad_cumulative() {
        let high = [10.0, 11.0, 12.0];
        let low = [9.0, 9.5, 11.0];
        let close = [9.5, 10.5, 11.5];
        let vol = [100.0, 200.0, 300.0];
        let out = ad(&high, &low, &close, &vol).unwrap();
        // CLV[0] = (19-10-9)/1 = 0 -> AD = 0
        // CLV[1] = (21-11-9.5)/1.5 = 0.5/1.5 = 1/3 -> +200/3
        // CLV[2] = (23-12-11)/1 = 0 -> 0
        assert!(out[0].abs() < 1e-12);
        assert!((out[1] - 200.0 / 3.0).abs() < 1e-12);
        assert!((out[2] - 200.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn adosc_shape() {
        let high: Vec<f64> = (0..60).map(|i| 10.0 + i as f64 * 0.1 + 0.5).collect();
        let low: Vec<f64> = (0..60).map(|i| 9.0 + i as f64 * 0.1 - 0.5).collect();
        let close: Vec<f64> = (0..60).map(|i| 9.5 + i as f64 * 0.1).collect();
        let vol: Vec<f64> = (0..60).map(|i| 1000.0 + 100.0 * (i as f64 * 0.7).sin()).collect();
        let out = adosc(&high, &low, &close, &vol, 3, 10).unwrap();
        // lookback = slow-1 = 9
        assert!(out[8].is_nan());
        assert!(!out[20].is_nan());
    }

    #[test]
    fn obv_sign_tracking() {
        let close = [10.0, 11.0, 10.5, 9.0];
        let vol = [100.0, 200.0, 50.0, 80.0];
        let out = obv(&close, &vol).unwrap();
        assert!((out[0] - 100.0).abs() < 1e-12);
        assert!((out[1] - 300.0).abs() < 1e-12); // close 10 -> 11, up
        assert!((out[2] - 250.0).abs() < 1e-12); // close 11 -> 10.5, down -50
        assert!((out[3] - 170.0).abs() < 1e-12); // close 10.5 -> 9, down -80
    }
}
