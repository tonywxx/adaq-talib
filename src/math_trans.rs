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
//!
//! 本模块是架构评审候选①（指标脚手架接缝）的 **Phase 1a 试点**：每个公开 `func` 不再手写
//! 「分配等长 NaN 缓冲 + 转发」脚手架，改由 `indicator!` 宏生成；逐元素热路径
//! `*_with_output` 体保持手写、字节级不变。见 `src/indicator.rs` 与 ADR-0011。

use crate::error::TaError;
use crate::indicator::indicator;

indicator! {
    /// 反余弦（TA-Lib `TA_ACOS`）。逐元素 `acos(x)`，定义域 `[-1, 1]`，越界输出 `NaN`。
    /// Arccosine (TA-Lib `TA_ACOS`). Element-wise `acos(x)`; domain `[-1, 1]`, out-of-domain → `NaN`.
    fn acos(values: &[f64]) -> Vec<f64> with acos_with_output;
}

/// 反余弦的零拷贝写入变体。见 [`acos`]。
/// Zero-copy write variant of [`acos`]. See [`acos`].
pub fn acos_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "acos_with_output: out length must equal values length".into(),
        ));
    }
    for (o, &v) in out.iter_mut().zip(values.iter()) {
        *o = v.acos();
    }
    Ok(())
}

indicator! {
    /// 反正弦（TA-Lib `TA_ASIN`）。逐元素 `asin(x)`，定义域 `[-1, 1]`，越界输出 `NaN`。
    /// Arcsine (TA-Lib `TA_ASIN`). Element-wise `asin(x)`; domain `[-1, 1]`, out-of-domain → `NaN`.
    fn asin(values: &[f64]) -> Vec<f64> with asin_with_output;
}

/// 反正弦的零拷贝写入变体。见 [`asin`]。
/// Zero-copy write variant of [`asin`]. See [`asin`].
pub fn asin_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "asin_with_output: out length must equal values length".into(),
        ));
    }
    for (o, &v) in out.iter_mut().zip(values.iter()) {
        *o = v.asin();
    }
    Ok(())
}

indicator! {
    /// 反正切（TA-Lib `TA_ATAN`）。逐元素 `atan(x)`。
    /// Arctangent (TA-Lib `TA_ATAN`). Element-wise `atan(x)`.
    fn atan(values: &[f64]) -> Vec<f64> with atan_with_output;
}

/// 反正切的零拷贝写入变体。见 [`atan`]。
/// Zero-copy write variant of [`atan`]. See [`atan`].
pub fn atan_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "atan_with_output: out length must equal values length".into(),
        ));
    }
    for (o, &v) in out.iter_mut().zip(values.iter()) {
        *o = v.atan();
    }
    Ok(())
}

indicator! {
    /// 向上取整（TA-Lib `TA_CEIL`）。逐元素 `ceil(x)`。
    /// Ceiling (TA-Lib `TA_CEIL`). Element-wise `ceil(x)`.
    fn ceil(values: &[f64]) -> Vec<f64> with ceil_with_output;
}

/// 向上取整的零拷贝写入变体。见 [`ceil`]。
/// Zero-copy write variant of [`ceil`]. See [`ceil`].
pub fn ceil_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "ceil_with_output: out length must equal values length".into(),
        ));
    }
    for (o, &v) in out.iter_mut().zip(values.iter()) {
        *o = v.ceil();
    }
    Ok(())
}

indicator! {
    /// 余弦（TA-Lib `TA_COS`）。逐元素 `cos(x)`。
    /// Cosine (TA-Lib `TA_COS`). Element-wise `cos(x)`.
    fn cos(values: &[f64]) -> Vec<f64> with cos_with_output;
}

/// 余弦的零拷贝写入变体。见 [`cos`]。
/// Zero-copy write variant of [`cos`]. See [`cos`].
pub fn cos_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "cos_with_output: out length must equal values length".into(),
        ));
    }
    for (o, &v) in out.iter_mut().zip(values.iter()) {
        *o = v.cos();
    }
    Ok(())
}

indicator! {
    /// 双曲余弦（TA-Lib `TA_COSH`）。逐元素 `cosh(x)`。
    /// Hyperbolic cosine (TA-Lib `TA_COSH`). Element-wise `cosh(x)`.
    fn cosh(values: &[f64]) -> Vec<f64> with cosh_with_output;
}

/// 双曲余弦的零拷贝写入变体。见 [`cosh`]。
/// Zero-copy write variant of [`cosh`]. See [`cosh`].
pub fn cosh_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "cosh_with_output: out length must equal values length".into(),
        ));
    }
    for (o, &v) in out.iter_mut().zip(values.iter()) {
        *o = v.cosh();
    }
    Ok(())
}

indicator! {
    /// 指数（TA-Lib `TA_EXP`）。逐元素 `exp(x)`。
    /// Exponential (TA-Lib `TA_EXP`). Element-wise `exp(x)`.
    fn exp(values: &[f64]) -> Vec<f64> with exp_with_output;
}

/// 指数的零拷贝写入变体。见 [`exp`]。
/// Zero-copy write variant of [`exp`]. See [`exp`].
pub fn exp_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "exp_with_output: out length must equal values length".into(),
        ));
    }
    for (o, &v) in out.iter_mut().zip(values.iter()) {
        *o = v.exp();
    }
    Ok(())
}

indicator! {
    /// 向下取整（TA-Lib `TA_FLOOR`）。逐元素 `floor(x)`。
    /// Floor (TA-Lib `TA_FLOOR`). Element-wise `floor(x)`.
    fn floor(values: &[f64]) -> Vec<f64> with floor_with_output;
}

/// 向下取整的零拷贝写入变体。见 [`floor`]。
/// Zero-copy write variant of [`floor`]. See [`floor`].
pub fn floor_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "floor_with_output: out length must equal values length".into(),
        ));
    }
    for (o, &v) in out.iter_mut().zip(values.iter()) {
        *o = v.floor();
    }
    Ok(())
}

indicator! {
    /// 自然对数（TA-Lib `TA_LN`）。逐元素 `ln(x)`，定义域 `x > 0`，越界输出 `NaN`/`inf`。
    /// Natural logarithm (TA-Lib `TA_LN`). Element-wise `ln(x)`; domain `x > 0`, out-of-domain → `NaN`/`inf`.
    fn ln(values: &[f64]) -> Vec<f64> with ln_with_output;
}

/// 自然对数的零拷贝写入变体。见 [`ln`]。
/// Zero-copy write variant of [`ln`]. See [`ln`].
pub fn ln_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "ln_with_output: out length must equal values length".into(),
        ));
    }
    for (o, &v) in out.iter_mut().zip(values.iter()) {
        *o = v.ln();
    }
    Ok(())
}

indicator! {
    /// 常用对数（TA-Lib `TA_LOG10`）。逐元素 `log10(x)`，定义域 `x > 0`，越界输出 `NaN`/`inf`。
    /// Base-10 logarithm (TA-Lib `TA_LOG10`). Element-wise `log10(x)`; domain `x > 0`, out-of-domain → `NaN`/`inf`.
    fn log10(values: &[f64]) -> Vec<f64> with log10_with_output;
}

/// 常用对数的零拷贝写入变体。见 [`log10`]。
/// Zero-copy write variant of [`log10`]. See [`log10`].
pub fn log10_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "log10_with_output: out length must equal values length".into(),
        ));
    }
    for (o, &v) in out.iter_mut().zip(values.iter()) {
        *o = v.log10();
    }
    Ok(())
}

indicator! {
    /// 正弦（TA-Lib `TA_SIN`）。逐元素 `sin(x)`。
    /// Sine (TA-Lib `TA_SIN`). Element-wise `sin(x)`.
    fn sin(values: &[f64]) -> Vec<f64> with sin_with_output;
}

/// 正弦的零拷贝写入变体。见 [`sin`]。
/// Zero-copy write variant of [`sin`]. See [`sin`].
pub fn sin_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "sin_with_output: out length must equal values length".into(),
        ));
    }
    for (o, &v) in out.iter_mut().zip(values.iter()) {
        *o = v.sin();
    }
    Ok(())
}

indicator! {
    /// 双曲正弦（TA-Lib `TA_SINH`）。逐元素 `sinh(x)`。
    /// Hyperbolic sine (TA-Lib `TA_SINH`). Element-wise `sinh(x)`.
    fn sinh(values: &[f64]) -> Vec<f64> with sinh_with_output;
}

/// 双曲正弦的零拷贝写入变体。见 [`sinh`]。
/// Zero-copy write variant of [`sinh`]. See [`sinh`].
pub fn sinh_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "sinh_with_output: out length must equal values length".into(),
        ));
    }
    for (o, &v) in out.iter_mut().zip(values.iter()) {
        *o = v.sinh();
    }
    Ok(())
}

indicator! {
    /// 平方根（TA-Lib `TA_SQRT`）。逐元素 `sqrt(x)`，定义域 `x >= 0`，越界输出 `NaN`。
    /// Square root (TA-Lib `TA_SQRT`). Element-wise `sqrt(x)`; domain `x >= 0`, out-of-domain → `NaN`.
    fn sqrt(values: &[f64]) -> Vec<f64> with sqrt_with_output;
}

/// 平方根的零拷贝写入变体。见 [`sqrt`]。
/// Zero-copy write variant of [`sqrt`]. See [`sqrt`].
pub fn sqrt_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "sqrt_with_output: out length must equal values length".into(),
        ));
    }
    for (o, &v) in out.iter_mut().zip(values.iter()) {
        *o = v.sqrt();
    }
    Ok(())
}

indicator! {
    /// 正切（TA-Lib `TA_TAN`）。逐元素 `tan(x)`。
    /// Tangent (TA-Lib `TA_TAN`). Element-wise `tan(x)`.
    fn tan(values: &[f64]) -> Vec<f64> with tan_with_output;
}

/// 正切的零拷贝写入变体。见 [`tan`]。
/// Zero-copy write variant of [`tan`]. See [`tan`].
pub fn tan_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "tan_with_output: out length must equal values length".into(),
        ));
    }
    for (o, &v) in out.iter_mut().zip(values.iter()) {
        *o = v.tan();
    }
    Ok(())
}

indicator! {
    /// 双曲正切（TA-Lib `TA_TANH`）。逐元素 `tanh(x)`。
    /// Hyperbolic tangent (TA-Lib `TA_TANH`). Element-wise `tanh(x)`.
    fn tanh(values: &[f64]) -> Vec<f64> with tanh_with_output;
}

/// 双曲正切的零拷贝写入变体。见 [`tanh`]。
/// Zero-copy write variant of [`tanh`]. See [`tanh`].
pub fn tanh_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "tanh_with_output: out length must equal values length".into(),
        ));
    }
    for (o, &v) in out.iter_mut().zip(values.iter()) {
        *o = v.tanh();
    }
    Ok(())
}
