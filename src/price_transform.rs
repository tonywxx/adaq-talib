//! 价格变换（Price Transform）。
//!
//! Price Transform.
//!
//! 本模块全部函数的数值输出与 [TA-Lib](https://ta-lib.org) 0.7.1 逐项一致（浮点误差容限内，
//! 见 [`crate::utils`] 与 ADR 0005）。这些变换无滞后（lookback 0），与输入等长返回。
//!
//! Every function in this module reproduces the numerical output of TA-Lib 0.7.1 (within the
//! float tolerance in ADR 0005). These transforms have no lookback (lookback 0) and are
//! returned at equal length.

use crate::core::{check_eq_len, rolling_mean};
use crate::error::{check_period, TaError};
use crate::indicator::indicator;

/// 平均价（Average Price，TA-Lib `TA_AVGPRICE`）。
///
/// `AVGPRICE = (high + low + close + open) / 4`。无滞后（lookback 0）。
///
/// # 示例 / Example
/// ```
/// use adaq_talib::price_transform::avgprice;
/// let o = [1.0]; let h = [2.0]; let l = [0.5]; let c = [1.5];
/// let out = avgprice(&h, &l, &c, &o).unwrap();
/// assert!((out[0] - (2.0 + 0.5 + 1.5 + 1.0) / 4.0).abs() < 1e-9);
/// ```
pub fn avgprice(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    open: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[high, low, close, open], "avgprice")?;
    let mut out = vec![0.0_f64; high.len()];
    avgprice_with_output(high, low, close, open, &mut out)?;
    Ok(out)
}

/// 平均价，零拷贝写入 `out`（与 `high` 等长）。见 [`avgprice`]。
/// Average Price, written zero-copy into `out`. See [`avgprice`].
pub fn avgprice_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    open: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    check_eq_len(&[high, low, close, open], "avgprice")?;
    if out.len() != high.len() {
        return Err(TaError::BadParam(
            "avgprice_with_output: out length must equal high length".into(),
        ));
    }
    let n = high.len();
    for i in 0..n {
        out[i] = (high[i] + low[i] + close[i] + open[i]) / 4.0;
    }
    Ok(())
}

/// 中位价（Median Price，TA-Lib `TA_MEDPRICE`）。
///
/// `MEDPRICE = (high + low) / 2`。无滞后（lookback 0）。
pub fn medprice(high: &[f64], low: &[f64]) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[high, low], "medprice")?;
    let mut out = vec![0.0_f64; high.len()];
    medprice_with_output(high, low, &mut out)?;
    Ok(out)
}

/// 中位价，零拷贝写入 `out`（与 `high` 等长）。见 [`medprice`]。
/// Median Price, written zero-copy into `out`. See [`medprice`].
pub fn medprice_with_output(
    high: &[f64],
    low: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    check_eq_len(&[high, low], "medprice")?;
    if out.len() != high.len() {
        return Err(TaError::BadParam(
            "medprice_with_output: out length must equal high length".into(),
        ));
    }
    let n = high.len();
    for i in 0..n {
        out[i] = (high[i] + low[i]) / 2.0;
    }
    Ok(())
}

/// 典型价（Typical Price，TA-Lib `TA_TYPPRICE`）。
///
/// `TYPPRICE = (high + low + close) / 3`。无滞后（lookback 0）。
pub fn typprice(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[high, low, close], "typprice")?;
    let mut out = vec![0.0_f64; high.len()];
    typprice_with_output(high, low, close, &mut out)?;
    Ok(out)
}

/// 典型价，零拷贝写入 `out`（与 `high` 等长）。见 [`typprice`]。
/// Typical Price, written zero-copy into `out`. See [`typprice`].
pub fn typprice_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    check_eq_len(&[high, low, close], "typprice")?;
    if out.len() != high.len() {
        return Err(TaError::BadParam(
            "typprice_with_output: out length must equal high length".into(),
        ));
    }
    let n = high.len();
    for i in 0..n {
        out[i] = (high[i] + low[i] + close[i]) / 3.0;
    }
    Ok(())
}

/// 加权收盘价（Weighted Close Price，TA-Lib `TA_WCLPRICE`）。
///
/// `WCLPRICE = (high + low + 2*close) / 4`。无滞后（lookback 0）。
pub fn wclprice(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[high, low, close], "wclprice")?;
    let mut out = vec![0.0_f64; high.len()];
    wclprice_with_output(high, low, close, &mut out)?;
    Ok(out)
}

/// 加权收盘价，零拷贝写入 `out`（与 `high` 等长）。见 [`wclprice`]。
/// Weighted Close Price, written zero-copy into `out`. See [`wclprice`].
pub fn wclprice_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    check_eq_len(&[high, low, close], "wclprice")?;
    if out.len() != high.len() {
        return Err(TaError::BadParam(
            "wclprice_with_output: out length must equal high length".into(),
        ));
    }
    let n = high.len();
    for i in 0..n {
        out[i] = (high[i] + low[i] + 2.0 * close[i]) / 4.0;
    }
    Ok(())
}

indicator! {
    /// 平均偏差（Average Deviation，TA-Lib `TA_AVGDEV`）。
    ///
    /// 在每个长度为 `period` 的窗口上，先求简单移动平均 `MA`，再求价格相对 `MA` 的
    /// 平均绝对偏差：`AVGDEV[i] = mean_j(|x_{i-period+1+j} − MA[i]|)`。前导 `period-1` 为 [`f64::NAN`]。
    ///
    /// Average Deviation: within each trailing window of length `period`, take the SMA `MA`,
    /// then the mean absolute deviation of prices from `MA`. The leading `period - 1` are `NaN`.
    ///
    /// # 示例 / Example
    /// ```
    /// use adaq_talib::price_transform::avgdev;
    /// let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    /// let out = avgdev(&x, 5).unwrap();
    /// // MA = 3.0；|1-3|+|2-3|+|3-3|+|4-3|+|5-3| = 2+1+0+1+2 = 6；avgdev = 6/5 = 1.2
    /// assert!((out[4] - 1.2).abs() < 1e-9);
    /// ```
    fn avgdev(values: &[f64], time_period: usize) -> Vec<f64> with avgdev_with_output
    default avgdev_default(values: &[f64]) => (14)
    /// `avgdev` 便捷版本，默认周期 14（与 TA-Lib 一致）。
    /// `avgdev` with default period (14), matching TA-Lib.
    ;
}

/// 平均偏差，零拷贝写入 `out`（与 `values` 等长）。见 [`avgdev`]。
/// Average Deviation, written zero-copy into `out`. See [`avgdev`].
pub fn avgdev_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "avgdev_with_output: out length must equal values length".into(),
        ));
    }
    let n = values.len();
    if n < time_period {
        return Ok(());
    }
    let mean = rolling_mean(values, time_period);
    let p = time_period as f64;
    for i in (time_period - 1)..n {
        let m = mean[i];
        let mut s = 0.0_f64;
        for j in 0..time_period {
            s += (values[i - j] - m).abs();
        }
        out[i] = s / p;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avgprice_basic() {
        let o = [1.0, 2.0];
        let h = [2.0, 3.0];
        let l = [0.5, 1.0];
        let c = [1.5, 2.5];
        let out = avgprice(&h, &l, &c, &o).unwrap();
        assert!((out[0] - (2.0 + 0.5 + 1.5 + 1.0) / 4.0).abs() < 1e-12);
        assert!((out[1] - (3.0 + 1.0 + 2.5 + 2.0) / 4.0).abs() < 1e-12);
    }

    #[test]
    fn medprice_basic() {
        let h = [2.0, 4.0];
        let l = [1.0, 3.0];
        let out = medprice(&h, &l).unwrap();
        assert!((out[0] - 1.5).abs() < 1e-12);
        assert!((out[1] - 3.5).abs() < 1e-12);
    }

    #[test]
    fn typprice_basic() {
        let h = [2.0, 4.0];
        let l = [1.0, 3.0];
        let c = [1.5, 3.5];
        let out = typprice(&h, &l, &c).unwrap();
        assert!((out[0] - (2.0 + 1.0 + 1.5) / 3.0).abs() < 1e-12);
    }

    #[test]
    fn wclprice_basic() {
        let h = [2.0, 4.0];
        let l = [1.0, 3.0];
        let c = [1.5, 3.5];
        let out = wclprice(&h, &l, &c).unwrap();
        assert!((out[0] - (2.0 + 1.0 + 3.0) / 4.0).abs() < 1e-12);
    }
}
