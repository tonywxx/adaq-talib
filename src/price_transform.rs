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

use crate::core::check_eq_len;
use crate::error::TaError;

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
    let n = high.len();
    let mut out = vec![0.0_f64; n];
    for i in 0..n {
        out[i] = (high[i] + low[i] + close[i] + open[i]) / 4.0;
    }
    Ok(out)
}

/// 中位价（Median Price，TA-Lib `TA_MEDPRICE`）。
///
/// `MEDPRICE = (high + low) / 2`。无滞后（lookback 0）。
pub fn medprice(high: &[f64], low: &[f64]) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[high, low], "medprice")?;
    let n = high.len();
    let mut out = vec![0.0_f64; n];
    for i in 0..n {
        out[i] = (high[i] + low[i]) / 2.0;
    }
    Ok(out)
}

/// 典型价（Typical Price，TA-Lib `TA_TYPPRICE`）。
///
/// `TYPPRICE = (high + low + close) / 3`。无滞后（lookback 0）。
pub fn typprice(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[high, low, close], "typprice")?;
    let n = high.len();
    let mut out = vec![0.0_f64; n];
    for i in 0..n {
        out[i] = (high[i] + low[i] + close[i]) / 3.0;
    }
    Ok(out)
}

/// 加权收盘价（Weighted Close Price，TA-Lib `TA_WCLPRICE`）。
///
/// `WCLPRICE = (high + low + 2*close) / 4`。无滞后（lookback 0）。
pub fn wclprice(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[high, low, close], "wclprice")?;
    let n = high.len();
    let mut out = vec![0.0_f64; n];
    for i in 0..n {
        out[i] = (high[i] + low[i] + 2.0 * close[i]) / 4.0;
    }
    Ok(out)
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
