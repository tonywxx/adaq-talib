//! 数学变换函数（Math Transform）。
//!
//! Math Transform functions.
//!
//! 本模块函数均为逐元素（element-wise）数学变换，数值输出与 [TA-Lib](https://ta-lib.org)
//! 0.7.1 逐项一致（浮点误差容限内，见 [`crate::utils`] 与 ADR 0005）。输入与输出等长；
//! 对定义域外的输入，输出为 [`f64::NAN`]/[`f64::INFINITY`]，与原版一致。
//!
//! Every function here is an element-wise math transform; the output equals TA-Lib 0.7.1
//! (within the ADR 0005 tolerance). Input and output are equal length; out-of-domain inputs
//! yield `NaN`/`inf`, matching the original.

use crate::error::TaError;

/// 反余弦（TA-Lib `TA_ACOS`）。逐元素 `acos(x)`，定义域 `[-1, 1]`，越界输出 `NaN`。
/// Arccosine (TA-Lib `TA_ACOS`). Element-wise `acos(x)`; domain `[-1, 1]`, out-of-domain → `NaN`.
pub fn acos(values: &[f64]) -> Result<Vec<f64>, TaError> {
    Ok(values.iter().map(|&v| v.acos()).collect())
}

/// 反正弦（TA-Lib `TA_ASIN`）。逐元素 `asin(x)`，定义域 `[-1, 1]`，越界输出 `NaN`。
/// Arcsine (TA-Lib `TA_ASIN`). Element-wise `asin(x)`; domain `[-1, 1]`, out-of-domain → `NaN`.
pub fn asin(values: &[f64]) -> Result<Vec<f64>, TaError> {
    Ok(values.iter().map(|&v| v.asin()).collect())
}

/// 反正切（TA-Lib `TA_ATAN`）。逐元素 `atan(x)`。
/// Arctangent (TA-Lib `TA_ATAN`). Element-wise `atan(x)`.
pub fn atan(values: &[f64]) -> Result<Vec<f64>, TaError> {
    Ok(values.iter().map(|&v| v.atan()).collect())
}

/// 向上取整（TA-Lib `TA_CEIL`）。逐元素 `ceil(x)`。
/// Ceiling (TA-Lib `TA_CEIL`). Element-wise `ceil(x)`.
pub fn ceil(values: &[f64]) -> Result<Vec<f64>, TaError> {
    Ok(values.iter().map(|&v| v.ceil()).collect())
}

/// 余弦（TA-Lib `TA_COS`）。逐元素 `cos(x)`。
/// Cosine (TA-Lib `TA_COS`). Element-wise `cos(x)`.
pub fn cos(values: &[f64]) -> Result<Vec<f64>, TaError> {
    Ok(values.iter().map(|&v| v.cos()).collect())
}

/// 双曲余弦（TA-Lib `TA_COSH`）。逐元素 `cosh(x)`。
/// Hyperbolic cosine (TA-Lib `TA_COSH`). Element-wise `cosh(x)`.
pub fn cosh(values: &[f64]) -> Result<Vec<f64>, TaError> {
    Ok(values.iter().map(|&v| v.cosh()).collect())
}

/// 指数（TA-Lib `TA_EXP`）。逐元素 `exp(x)`。
/// Exponential (TA-Lib `TA_EXP`). Element-wise `exp(x)`.
pub fn exp(values: &[f64]) -> Result<Vec<f64>, TaError> {
    Ok(values.iter().map(|&v| v.exp()).collect())
}

/// 向下取整（TA-Lib `TA_FLOOR`）。逐元素 `floor(x)`。
/// Floor (TA-Lib `TA_FLOOR`). Element-wise `floor(x)`.
pub fn floor(values: &[f64]) -> Result<Vec<f64>, TaError> {
    Ok(values.iter().map(|&v| v.floor()).collect())
}

/// 自然对数（TA-Lib `TA_LN`）。逐元素 `ln(x)`，定义域 `x > 0`，越界输出 `NaN`/`inf`。
/// Natural logarithm (TA-Lib `TA_LN`). Element-wise `ln(x)`; domain `x > 0`, out-of-domain → `NaN`/`inf`.
pub fn ln(values: &[f64]) -> Result<Vec<f64>, TaError> {
    Ok(values.iter().map(|&v| v.ln()).collect())
}

/// 常用对数（TA-Lib `TA_LOG10`）。逐元素 `log10(x)`，定义域 `x > 0`，越界输出 `NaN`/`inf`。
/// Base-10 logarithm (TA-Lib `TA_LOG10`). Element-wise `log10(x)`; domain `x > 0`, out-of-domain → `NaN`/`inf`.
pub fn log10(values: &[f64]) -> Result<Vec<f64>, TaError> {
    Ok(values.iter().map(|&v| v.log10()).collect())
}

/// 正弦（TA-Lib `TA_SIN`）。逐元素 `sin(x)`。
/// Sine (TA-Lib `TA_SIN`). Element-wise `sin(x)`.
pub fn sin(values: &[f64]) -> Result<Vec<f64>, TaError> {
    Ok(values.iter().map(|&v| v.sin()).collect())
}

/// 双曲正弦（TA-Lib `TA_SINH`）。逐元素 `sinh(x)`。
/// Hyperbolic sine (TA-Lib `TA_SINH`). Element-wise `sinh(x)`.
pub fn sinh(values: &[f64]) -> Result<Vec<f64>, TaError> {
    Ok(values.iter().map(|&v| v.sinh()).collect())
}

/// 平方根（TA-Lib `TA_SQRT`）。逐元素 `sqrt(x)`，定义域 `x >= 0`，越界输出 `NaN`。
/// Square root (TA-Lib `TA_SQRT`). Element-wise `sqrt(x)`; domain `x >= 0`, out-of-domain → `NaN`.
pub fn sqrt(values: &[f64]) -> Result<Vec<f64>, TaError> {
    Ok(values.iter().map(|&v| v.sqrt()).collect())
}

/// 正切（TA-Lib `TA_TAN`）。逐元素 `tan(x)`。
/// Tangent (TA-Lib `TA_TAN`). Element-wise `tan(x)`.
pub fn tan(values: &[f64]) -> Result<Vec<f64>, TaError> {
    Ok(values.iter().map(|&v| v.tan()).collect())
}

/// 双曲正切（TA-Lib `TA_TANH`）。逐元素 `tanh(x)`。
/// Hyperbolic tangent (TA-Lib `TA_TANH`). Element-wise `tanh(x)`.
pub fn tanh(values: &[f64]) -> Result<Vec<f64>, TaError> {
    Ok(values.iter().map(|&v| v.tanh()).collect())
}
