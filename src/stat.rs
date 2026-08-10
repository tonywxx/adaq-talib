//! 统计函数（Statistic Functions）。
//!
//! Statistic Functions.
//!
//! 本模块全部函数的数值输出与 [TA-Lib](https://ta-lib.org) 0.7.1 逐项一致（浮点误差容限内，
//! 见 [`crate::utils`] 与 ADR 0005）。前导不稳定期以 [`f64::NAN`] 填充、等长返回（见 ADR 0007）。
//!
//! Every function in this module reproduces the numerical output of TA-Lib 0.7.1 (within the
//! float tolerance in ADR 0005). The leading unstable period is filled with [`f64::NAN`] and
//! returned at equal length (ADR 0007).

use crate::core::defaults::{BETA_PERIOD, CORREL_PERIOD, LINEARREG_PERIOD, STDDEV_NB_DEV, STDDEV_PERIOD};
use crate::core::{check_eq_len, rolling_var};
use crate::error::{check_period, TaError};

/// 标准差（Standard Deviation，TA-Lib `TA_STDDEV`）。
///
/// `STDDEV = nb_dev * sqrt(population_variance)`，`population_variance` 为窗口内总体方差
/// （除以 `period`，见 [`crate::core::rolling_var`]）。前导 `period-1` 个为 [`f64::NAN`]。
///
/// `STDDEV = nb_dev * sqrt(population variance)`. The leading `period - 1` positions are
/// [`f64::NAN`].
///
/// # 示例 / Example
/// ```
/// use adaq_talib::stat::stddev;
/// let x = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
/// let out = stddev(&x, 8, 1.0).unwrap();
/// // 总体方差 = 32/8 = 4 -> 标准差 = 2
/// assert!((out[7] - 2.0).abs() < 1e-9);
/// ```
pub fn stddev(values: &[f64], time_period: usize, nb_dev: f64) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let var = rolling_var(values, time_period);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    for i in 0..n {
        if !var[i].is_nan() {
            out[i] = nb_dev * var[i].sqrt();
        }
    }
    Ok(out)
}

/// `stddev` 便捷版本，默认周期 5、偏离倍数 1.0。/ `stddev` with defaults (5, 1.0).
pub fn stddev_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    stddev(values, STDDEV_PERIOD, STDDEV_NB_DEV)
}

/// 方差（Variance，TA-Lib `TA_VAR`）。
///
/// 窗口内总体方差（除以 `period`，见 [`crate::core::rolling_var`）；`nb_dev` 参数与 TA-Lib
/// 一致被忽略（TA-Lib `TA_VAR` 不对方差应用偏离倍数）。前导 `period-1` 个为 [`f64::NAN`]。
///
/// Population variance (divide by `period`). Like TA-Lib `TA_VAR`, the `nb_dev` argument is
/// accepted but **not applied** to the variance (TA-Lib ignores it for `TA_VAR`). The leading
/// `period - 1` positions are [`f64::NAN`].
pub fn var(values: &[f64], time_period: usize, _nb_dev: f64) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    Ok(rolling_var(values, time_period))
}

/// `var` 便捷版本，默认周期 5。/ `var` with default period (5).
pub fn var_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    var(values, STDDEV_PERIOD, STDDEV_NB_DEV)
}

// ---------------------------------------------------------------------------
// 线性回归族 / Linear Regression: LINEARREG, _ANGLE, _INTERCEPT, _SLOPE, TSF
// ---------------------------------------------------------------------------

/// 线性回归族的内核（最小二乘）。
/// Shared core for the linear-regression family (least squares).
///
/// - `mode = 0`：LINEARREG —— 回归线在窗口右端（位置 `period-1`）的预测值。
/// - `mode = 1`：LINEARREG_ANGLE —— 斜率的反正切（角度，度）。
/// - `mode = 2`：LINEARREG_INTERCEPT —— 截距。
/// - `mode = 3`：LINEARREG_SLOPE —— 斜率。
/// - `mode = 4`：TSF —— 回归线在窗口右端再外推一步（位置 `period`）的预测值。
fn linreg_core(values: &[f64], period: usize, mode: u8) -> Vec<f64> {
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if n < period {
        return out;
    }
    let p = period as f64;
    let sx = (period * (period - 1)) as f64 / 2.0; // Σ k, k=0..period-1
    let sxx = (period * (period - 1) * (2 * period - 1)) as f64 / 6.0; // Σ k^2
    let denom = p * sxx - sx * sx;
    for i in (period - 1)..n {
        let mut sy = 0.0_f64;
        let mut sxy = 0.0_f64;
        for k in 0..period {
            let x = values[i - (period - 1) + k];
            sy += x;
            sxy += (k as f64) * x;
        }
        let slope = (p * sxy - sx * sy) / denom;
        let intercept = (sy - slope * sx) / p;
        out[i] = match mode {
            1 => slope.atan() * 180.0 / std::f64::consts::PI, // ANGLE (degrees)
            2 => intercept,                                   // INTERCEPT
            3 => slope,                                       // SLOPE
            4 => intercept + slope * p,                       // TSF (project one step beyond)
            _ => intercept + slope * (p - 1.0),               // LINEARREG (right edge)
        };
    }
    out
}

/// 线性回归（Linear Regression，TA-Lib `TA_LINEARREG`）。
///
/// 在每个窗口（长度 `period`）上做最小二乘拟合，返回回归线在窗口右端（当前 bar）的预测值。
/// 前导 `period-1` 个为 [`f64::NAN`]。
pub fn linear_reg(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    Ok(linreg_core(values, time_period, 0))
}

/// `linear_reg` 便捷版本，默认周期 14。/ `linear_reg` with default period (14).
pub fn linear_reg_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    linear_reg(values, LINEARREG_PERIOD)
}

/// 线性回归角度（TA-Lib `TA_LINEARREG_ANGLE`）。
///
/// 返回回归线斜率的反正切，单位为**度**（TA-Lib 约定）。前导 `period-1` 个为 [`f64::NAN`]。
pub fn linear_reg_angle(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    Ok(linreg_core(values, time_period, 1))
}

/// `linear_reg_angle` 便捷版本，默认周期 14。/ `linear_reg_angle` with default period (14).
pub fn linear_reg_angle_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    linear_reg_angle(values, LINEARREG_PERIOD)
}

/// 线性回归截距（TA-Lib `TA_LINEARREG_INTERCEPT`）。前导 `period-1` 个为 [`f64::NAN`]。
pub fn linear_reg_intercept(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    Ok(linreg_core(values, time_period, 2))
}

/// `linear_reg_intercept` 便捷版本，默认周期 14。
pub fn linear_reg_intercept_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    linear_reg_intercept(values, LINEARREG_PERIOD)
}

/// 线性回归斜率（TA-Lib `TA_LINEARREG_SLOPE`）。前导 `period-1` 个为 [`f64::NAN`]。
pub fn linear_reg_slope(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    Ok(linreg_core(values, time_period, 3))
}

/// `linear_reg_slope` 便捷版本，默认周期 14。
pub fn linear_reg_slope_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    linear_reg_slope(values, LINEARREG_PERIOD)
}

/// 时间序列预测（Time Series Forecast，TA-Lib `TA_TSF`）。
///
/// 在窗口上拟合回归线后，外推一步（窗口右端再 +1）得到预测值。前导 `period-1` 个为 [`f64::NAN`]。
pub fn tsf(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    Ok(linreg_core(values, time_period, 4))
}

/// `tsf` 便捷版本，默认周期 14。
pub fn tsf_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    tsf(values, LINEARREG_PERIOD)
}

// ---------------------------------------------------------------------------
// BETA / CORREL
// ---------------------------------------------------------------------------

/// 协方差/相关系数族的内核（总体口径）。
/// Shared core for BETA / CORREL (population basis).
///
/// - `mode = 0`：BETA —— `cov(real0, real1) / var(real0)`（以 `real0` 为自变量）。
/// - `mode = 1`：CORREL —— 皮尔逊相关系数。
fn beta_corr_core(real0: &[f64], real1: &[f64], period: usize, mode: u8) -> Vec<f64> {
    let n = real0.len();
    let mut out = vec![f64::NAN; n];
    if n < period {
        return out;
    }
    let p = period as f64;
    for i in (period - 1)..n {
        let mut s0 = 0.0_f64;
        let mut s1 = 0.0_f64;
        let mut s00 = 0.0_f64;
        let mut s11 = 0.0_f64;
        let mut s01 = 0.0_f64;
        for k in 0..period {
            let a = real0[i - k];
            let b = real1[i - k];
            s0 += a;
            s1 += b;
            s00 += a * a;
            s11 += b * b;
            s01 += a * b;
        }
        let cov = (s01 - s0 * s1 / p) / p;
        let v0 = (s00 - s0 * s0 / p) / p;
        let v1 = (s11 - s1 * s1 / p) / p;
        out[i] = if mode == 0 {
            if v0 == 0.0 {
                0.0
            } else {
                cov / v0
            }
        } else if v0 == 0.0 || v1 == 0.0 {
            0.0
        } else {
            cov / (v0 * v1).sqrt()
        };
    }
    out
}

/// 贝塔系数（Beta，TA-Lib `TA_BETA`）。
///
/// `BETA = cov(real0, real1) / var(real0)`（以 `real0` 为自变量，`real1` 为因变量），
/// 总体协方差/方差口径。前导 `period-1` 个为 [`f64::NAN`]。
pub fn beta(
    real0: &[f64],
    real1: &[f64],
    time_period: usize,
) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[real0, real1], "beta")?;
    Ok(beta_corr_core(real0, real1, time_period, 0))
}

/// `beta` 便捷版本，默认周期 5。/ `beta` with default period (5).
pub fn beta_default(real0: &[f64], real1: &[f64]) -> Result<Vec<f64>, TaError> {
    beta(real0, real1, BETA_PERIOD)
}

/// 皮尔逊相关系数（Pearson Correlation Coefficient，TA-Lib `TA_CORREL`）。
///
/// `CORREL = cov(real0, real1) / sqrt(var(real0) * var(real1))`，总体口径。
/// 前导 `period-1` 个为 [`f64::NAN`]。
pub fn correl(
    real0: &[f64],
    real1: &[f64],
    time_period: usize,
) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[real0, real1], "correl")?;
    Ok(beta_corr_core(real0, real1, time_period, 1))
}

/// `correl` 便捷版本，默认周期 5。/ `correl` with default period (5).
pub fn correl_default(real0: &[f64], real1: &[f64]) -> Result<Vec<f64>, TaError> {
    correl(real0, real1, CORREL_PERIOD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stddev_population() {
        let x = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        // 均值 = 40/8 = 5；Σ(x-5)^2 = 9+1+1+1+0+0+4+16 = 32；总体方差 = 4 -> 标准差 2
        let out = stddev(&x, 8, 1.0).unwrap();
        assert!((out[7] - 2.0).abs() < 1e-9);
        // nb_dev = 2 -> 4
        let out2 = stddev(&x, 8, 2.0).unwrap();
        assert!((out2[7] - 4.0).abs() < 1e-9);
    }

    #[test]
    fn var_matches_stddev_squared() {
        let x: Vec<f64> = (0..30).map(|i| (i as f64).sin() + 2.0).collect();
        let v = var(&x, 10, 1.0).unwrap();
        let s = stddev(&x, 10, 1.0).unwrap();
        let i = 29;
        assert!((v[i] - s[i] * s[i]).abs() < 1e-9);
    }

    #[test]
    fn linear_reg_through_points() {
        // 完美线性序列 y = 2x+1，period=3 -> 斜率 2，截距 1，LINEARREG 在位置 2 = 5
        let y = [1.0, 3.0, 5.0, 7.0, 9.0];
        let lr = linear_reg(&y, 3).unwrap();
        assert!((lr[2] - 5.0).abs() < 1e-9);
        let slope = linear_reg_slope(&y, 3).unwrap();
        assert!((slope[2] - 2.0).abs() < 1e-9);
        let intercept = linear_reg_intercept(&y, 3).unwrap();
        assert!((intercept[2] - 1.0).abs() < 1e-9);
        let tsf_out = tsf(&y, 3).unwrap();
        // TSF 外推一步：位置 3 = 1 + 2*3 = 7
        assert!((tsf_out[2] - 7.0).abs() < 1e-9);
        let angle = linear_reg_angle(&y, 3).unwrap();
        // atan(2)*180/π
        assert!((angle[2] - (2.0_f64).atan() * 180.0 / std::f64::consts::PI).abs() < 1e-9);
    }

    #[test]
    fn correl_perfect_positive() {
        let a: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let b: Vec<f64> = (0..20).map(|i| 2.0 * i as f64 + 1.0).collect();
        let c = correl(&a, &b, 10).unwrap();
        assert!((c[19] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn beta_of_scaled_series() {
        // b = 3*a + 5 -> beta(a, b) = 3
        let a: Vec<f64> = (0..20).map(|i| (i as f64).sin()).collect();
        let b: Vec<f64> = a.iter().map(|&v| 3.0 * v + 5.0).collect();
        let bt = beta(&a, &b, 10).unwrap();
        assert!((bt[19] - 3.0).abs() < 1e-9);
    }

    #[test]
    fn beta_length_mismatch_is_error() {
        assert!(matches!(
            beta(&[1.0, 2.0], &[1.0], 1),
            Err(TaError::BadParam(_))
        ));
    }
}
