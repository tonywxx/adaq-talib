//! 内部工具函数（非公开模块）。
//!
//! Internal utility functions (private module): output alignment and the
//! approximate-equality helper used by tests to compare against golden vectors
//! (matches the tolerance policy in ADR 0005).

/// 相对容限 / Relative tolerance (ADR 0005).
pub(crate) const REL_TOL: f64 = 1e-8;
/// 绝对下限 / Absolute floor (ADR 0005).
pub(crate) const ABS_TOL: f64 = 1e-10;

/// 按 ADR 0005 容限比较两个浮点：相对 1e-8 + 绝对 1e-10。
///
/// Compare two floats under the ADR 0005 tolerance: relative 1e-8 + absolute 1e-10.
///
/// 两侧同为 `NaN` 视为相等；仅一侧为 `NaN` 视为不等。
/// Both sides `NaN` counts as equal; exactly one `NaN` counts as unequal.
#[inline]
pub fn approx_eq(a: f64, b: f64) -> bool {
    if a.is_nan() || b.is_nan() {
        return a.is_nan() && b.is_nan();
    }
    let diff = (a - b).abs();
    diff <= REL_TOL * a.abs().max(b.abs()) + ABS_TOL
}

/// 按 ADR 0005 容限逐元素比较两个切片（长度须一致）。
///
/// Element-wise compare two slices under ADR 0005 (lengths must match).
#[inline]
pub fn approx_eq_slice(a: &[f64], b: &[f64]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| approx_eq(*x, *y))
}
