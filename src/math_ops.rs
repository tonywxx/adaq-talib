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
use crate::indicator::indicator;

indicator! {
    /// 逐元素相加（TA-Lib `TA_ADD`）：`out = real0 + real1`，等长返回。
    /// Element-wise addition (TA-Lib `TA_ADD`): `out = real0 + real1`; equal-length.
    fn add(real0: &[f64], real1: &[f64]) -> Vec<f64> with add_with_output;
}

/// 逐元素相加的零拷贝写入变体。见 [`add`]。
/// Zero-copy write variant of [`add`]. See [`add`].
pub fn add_with_output(real0: &[f64], real1: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    check_eq_len(&[real0, real1], "add")?;
    if out.len() != real0.len() {
        return Err(TaError::BadParam(
            "add_with_output: out length must equal real0 length".into(),
        ));
    }
    for ((o, &a), &b) in out.iter_mut().zip(real0.iter()).zip(real1.iter()) {
        *o = a + b;
    }
    Ok(())
}

indicator! {
    /// 逐元素相减（TA-Lib `TA_SUB`）：`out = real0 - real1`，等长返回。
    /// Element-wise subtraction (TA-Lib `TA_SUB`): `out = real0 - real1`; equal-length.
    fn sub(real0: &[f64], real1: &[f64]) -> Vec<f64> with sub_with_output;
}

/// 逐元素相减的零拷贝写入变体。见 [`sub`]。
/// Zero-copy write variant of [`sub`]. See [`sub`].
pub fn sub_with_output(real0: &[f64], real1: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    check_eq_len(&[real0, real1], "sub")?;
    if out.len() != real0.len() {
        return Err(TaError::BadParam(
            "sub_with_output: out length must equal real0 length".into(),
        ));
    }
    for ((o, &a), &b) in out.iter_mut().zip(real0.iter()).zip(real1.iter()) {
        *o = a - b;
    }
    Ok(())
}

indicator! {
    /// 逐元素相乘（TA-Lib `TA_MULT`）：`out = real0 * real1`，等长返回。
    /// Element-wise multiplication (TA-Lib `TA_MULT`): `out = real0 * real1`; equal-length.
    fn mult(real0: &[f64], real1: &[f64]) -> Vec<f64> with mult_with_output;
}

/// 逐元素相乘的零拷贝写入变体。见 [`mult`]。
/// Zero-copy write variant of [`mult`]. See [`mult`].
pub fn mult_with_output(real0: &[f64], real1: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    check_eq_len(&[real0, real1], "mult")?;
    if out.len() != real0.len() {
        return Err(TaError::BadParam(
            "mult_with_output: out length must equal real0 length".into(),
        ));
    }
    for ((o, &a), &b) in out.iter_mut().zip(real0.iter()).zip(real1.iter()) {
        *o = a * b;
    }
    Ok(())
}

indicator! {
    /// 逐元素相除（TA-Lib `TA_DIV`）：`out = real0 / real1`，等长返回；除零产生 `inf`/`NaN`。
    /// Element-wise division (TA-Lib `TA_DIV`): `out = real0 / real1`; equal-length;
    /// division by zero yields `inf`/`NaN`, matching the original.
    fn div(real0: &[f64], real1: &[f64]) -> Vec<f64> with div_with_output;
}

/// 逐元素相除的零拷贝写入变体。见 [`div`]。
/// Zero-copy write variant of [`div`]. See [`div`].
pub fn div_with_output(real0: &[f64], real1: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    check_eq_len(&[real0, real1], "div")?;
    if out.len() != real0.len() {
        return Err(TaError::BadParam(
            "div_with_output: out length must equal real0 length".into(),
        ));
    }
    for ((o, &a), &b) in out.iter_mut().zip(real0.iter()).zip(real1.iter()) {
        *o = a / b;
    }
    Ok(())
}

indicator! {
    /// 滚动窗口最大值（TA-Lib `TA_MAX`）。前导 `period-1` 个为 [`f64::NAN`]。
    /// Rolling maximum (TA-Lib `TA_MAX`). The leading `period - 1` positions are [`f64::NAN`].
    fn max(values: &[f64], time_period: usize) -> Vec<f64> with max_with_output;
}

/// 滚动窗口最大值的零拷贝写入变体。见 [`max`]。
/// Zero-copy write variant of [`max`]. See [`max`].
pub fn max_with_output(values: &[f64], time_period: usize, out: &mut [f64]) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "max_with_output: out length must equal values length".into(),
        ));
    }
    let r = rolling_max(values, time_period);
    out.copy_from_slice(&r);
    Ok(())
}

indicator! {
    /// 滚动窗口最小值（TA-Lib `TA_MIN`）。前导 `period-1` 个为 [`f64::NAN`]。
    /// Rolling minimum (TA-Lib `TA_MIN`). The leading `period - 1` positions are [`f64::NAN`].
    fn min(values: &[f64], time_period: usize) -> Vec<f64> with min_with_output;
}

/// 滚动窗口最小值的零拷贝写入变体。见 [`min`]。
/// Zero-copy write variant of [`min`]. See [`min`].
pub fn min_with_output(values: &[f64], time_period: usize, out: &mut [f64]) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "min_with_output: out length must equal values length".into(),
        ));
    }
    let r = rolling_min(values, time_period);
    out.copy_from_slice(&r);
    Ok(())
}

indicator! {
    /// 滚动窗口求和（TA-Lib `TA_SUM`）。前导 `period-1` 个为 [`f64::NAN`]。
    /// Rolling sum (TA-Lib `TA_SUM`). The leading `period - 1` positions are [`f64::NAN`].
    fn sum(values: &[f64], time_period: usize) -> Vec<f64> with sum_with_output;
}

/// 滚动窗口求和的零拷贝写入变体。见 [`sum`]。
/// Zero-copy write variant of [`sum`]. See [`sum`].
pub fn sum_with_output(values: &[f64], time_period: usize, out: &mut [f64]) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "sum_with_output: out length must equal values length".into(),
        ));
    }
    let r = rolling_sum(values, time_period);
    out.copy_from_slice(&r);
    Ok(())
}

indicator! {
    /// 滚动窗口最大值的**索引**（TA-Lib `TA_MAXINDEX`），返回窗口内最大值的绝对位置
    /// （0 基；平局取最左）。前导 `period-1` 个为 **0.0**（与原版一致，非 `NaN`）。
    /// Index of the rolling-window maximum (TA-Lib `TA_MAXINDEX`), the absolute (0-based) position
    /// of the max in the window (leftmost on ties). The leading `period - 1` positions are `NaN`.
    fn max_index(values: &[f64], time_period: usize) -> Vec<f64> with max_index_with_output;
}

/// 滚动窗口最大值索引的零拷贝写入变体。见 [`max_index`]。
/// Zero-copy write variant of [`max_index`]. See [`max_index`].
pub fn max_index_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "max_index_with_output: out length must equal values length".into(),
        ));
    }
    let r = rolling_extreme_index(values, time_period, true);
    out.copy_from_slice(&r);
    Ok(())
}

indicator! {
    /// 滚动窗口最小值的**索引**（TA-Lib `TA_MININDEX`），返回窗口内最小值的绝对位置
    /// （0 基；平局取最左）。前导 `period-1` 个为 **0.0**（与原版一致，非 `NaN`）。
    /// Index of the rolling-window minimum (TA-Lib `TA_MININDEX`), the absolute (0-based) position
    /// of the min in the window (leftmost on ties). The leading `period - 1` positions are `NaN`.
    fn min_index(values: &[f64], time_period: usize) -> Vec<f64> with min_index_with_output;
}

/// 滚动窗口最小值索引的零拷贝写入变体。见 [`min_index`]。
/// Zero-copy write variant of [`min_index`]. See [`min_index`].
pub fn min_index_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "min_index_with_output: out length must equal values length".into(),
        ));
    }
    let r = rolling_extreme_index(values, time_period, false);
    out.copy_from_slice(&r);
    Ok(())
}

/// 滚动窗口最小/最大值的双向量结果（TA-Lib `TA_MINMAX`）。
/// Two-vector result of the rolling-window min/max (TA-Lib `TA_MINMAX`).
pub struct MinMax {
    /// 窗口最小值 / Window minimum.
    pub min: Vec<f64>,
    /// 窗口最大值 / Window maximum.
    pub max: Vec<f64>,
}

/// 滚动窗口最小/最大值的双向量结果（TA-Lib `TA_MINMAX`），与前导 `period-1` 为 [`f64::NAN`]。
/// Two-vector result of the rolling-window min/max (TA-Lib `TA_MINMAX`), with leading `period - 1` `NaN`.
///
/// 单遍实现：复用 `core::rolling_minmax` 一次遍历同时求得最大与最小（最右 tie-break、前导
/// `NaN` 与分别调用 `rolling_max`/`rolling_min` 逐位相等，见 `core::rolling_minmax` 文档），
/// 将原本的两次独立窗口扫描合并为一次，规避重复遍历开销（P1 候选②，ADR 0005 零偏差）。
pub fn minmax(values: &[f64], time_period: usize) -> Result<MinMax, TaError> {
    check_period(time_period)?;
    let n = values.len();
    let mut out = MinMax {
        min: vec![f64::NAN; n],
        max: vec![f64::NAN; n],
    };
    minmax_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 滚动窗口最小/最大值的零拷贝写入变体。见 [`minmax`]。
/// Zero-copy write variant of [`minmax`]. See [`minmax`].
///
/// `out` 的 `min` / `max` 向量长度均必须等于 `values.len()`，否则返回 [`TaError::BadParam`]。
/// Both `min` and `max` vectors of `out` must have length equal to `values.len()`; otherwise
/// [`TaError::BadParam`] is returned.
pub fn minmax_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut MinMax,
) -> Result<(), TaError> {
    check_period(time_period)?;
    let n = values.len();
    if out.min.len() != n || out.max.len() != n {
        return Err(TaError::BadParam(
            "minmax_with_output: out vectors must have length == values length".into(),
        ));
    }
    // 数据量足够且启用 `parallel` feature 时走多核分块；内核与串行逐字节一致，输出 1:1。
    // Under the `parallel` feature with enough data, use multi-core chunking; the kernel is
    // byte-identical to the serial path, so output is 1:1.
    #[cfg(feature = "parallel")]
    {
        if n >= 8192 {
            return minmax_parallel_with_output(values, time_period, out);
        }
    }
    minmax_serial_with_output(values, time_period, out)
}

/// 滚动窗口最小/最大值串行内核（与 TA-Lib `TA_MINMAX` 逐项 1:1）。见 [`minmax_with_output`]。
/// Serial kernel for rolling min/max (1:1 with TA-Lib `TA_MINMAX`). See [`minmax_with_output`].
fn minmax_serial_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut MinMax,
) -> Result<(), TaError> {
    let (mx, mn) = rolling_minmax(values, time_period);
    out.max.copy_from_slice(&mx);
    out.min.copy_from_slice(&mn);
    Ok(())
}

/// 滚动窗口最小/最大值串行版本（feature 无关，供并行对照测试作黄金参考）。见 [`minmax`]。
/// Serial rolling min/max (feature-agnostic; golden reference for the parallel equality test). See [`minmax`].
pub fn minmax_serial(values: &[f64], time_period: usize) -> Result<MinMax, TaError> {
    check_period(time_period)?;
    let n = values.len();
    let mut out = MinMax {
        min: vec![f64::NAN; n],
        max: vec![f64::NAN; n],
    };
    minmax_serial_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 滚动窗口最小/最大值多核并行版本（需 `parallel` feature）。复用 [`minmax_serial_with_output`]
/// 的单遍双队列内核，以 `period-1` 前导重叠播种各分块的单调双端队列，输出与串行逐项 1:1。
/// Multi-core parallel rolling min/max (requires the `parallel` feature). Reuses the single-pass
/// dual-deque kernel of [`minmax_serial_with_output`] with `period-1` leading overlap to seed each
/// chunk's monotonic deques; output is 1:1 with the serial path.
#[cfg(feature = "parallel")]
pub fn minmax_parallel(values: &[f64], time_period: usize) -> Result<MinMax, TaError> {
    check_period(time_period)?;
    let n = values.len();
    let mut out = MinMax {
        min: vec![f64::NAN; n],
        max: vec![f64::NAN; n],
    };
    minmax_parallel_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 滚动窗口最小/最大值并行内核（零拷贝写入 `out`）。见 [`minmax_parallel`]。
/// Parallel kernel for rolling min/max (zero-copy into `out`). See [`minmax_parallel`].
#[cfg(feature = "parallel")]
fn minmax_parallel_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut MinMax,
) -> Result<(), TaError> {
    check_period(time_period)?;
    let n = values.len();
    if out.min.len() != n || out.max.len() != n {
        return Err(TaError::BadParam(
            "minmax_parallel_with_output: out vectors must have length == values length".into(),
        ));
    }
    let p = time_period;
    // `parallel_index_map_2`：单遍内核一次算 min/max 两路，省去双队列二次并行扫描。
    // `parallel_index_map_2`: one-pass kernel computes both min/max streams, avoiding a second
    // parallel scan of the dual deque.
    crate::parallel::parallel_index_map_2(
        n,
        p - 1,
        &mut out.min,
        &mut out.max,
        |start, end| {
            let (mx, mn) = rolling_minmax(&values[start..end], p);
            (mn, mx)
        },
    );
    Ok(())
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
    let n = values.len();
    let mut out = MinMaxIndex {
        min_idx: vec![f64::NAN; n],
        max_idx: vec![f64::NAN; n],
    };
    minmax_index_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 滚动窗口最小/最大值索引的零拷贝写入变体。见 [`minmax_index`]。
/// Zero-copy write variant of [`minmax_index`]. See [`minmax_index`].
///
/// `out` 的 `min_idx` / `max_idx` 向量长度均必须等于 `values.len()`，否则返回 [`TaError::BadParam`]。
/// Both `min_idx` and `max_idx` vectors of `out` must have length equal to `values.len()`; otherwise
/// [`TaError::BadParam`] is returned.
pub fn minmax_index_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut MinMaxIndex,
) -> Result<(), TaError> {
    check_period(time_period)?;
    let n = values.len();
    if out.min_idx.len() != n || out.max_idx.len() != n {
        return Err(TaError::BadParam(
            "minmax_index_with_output: out vectors must have length == values length".into(),
        ));
    }
    // 数据量足够且启用 `parallel` feature 时走多核分块；内核与串行逐字节一致，输出 1:1。
    // Under the `parallel` feature with enough data, use multi-core chunking; the kernel is
    // byte-identical to the serial path, so output is 1:1.
    #[cfg(feature = "parallel")]
    {
        if n >= 8192 {
            return minmax_index_parallel_with_output(values, time_period, out);
        }
    }
    minmax_index_serial_with_output(values, time_period, out)
}

/// 滚动窗口最小/最大索引串行内核（与 TA-Lib `TA_MINMAXINDEX` 逐项 1:1）。见 [`minmax_index_with_output`]。
/// Serial kernel for rolling min/max indices (1:1 with TA-Lib `TA_MINMAXINDEX`). See [`minmax_index_with_output`].
fn minmax_index_serial_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut MinMaxIndex,
) -> Result<(), TaError> {
    let min_idx = rolling_extreme_index(values, time_period, false);
    let max_idx = rolling_extreme_index(values, time_period, true);
    out.min_idx.copy_from_slice(&min_idx);
    out.max_idx.copy_from_slice(&max_idx);
    Ok(())
}

/// 滚动窗口最小/最大索引串行版本（feature 无关，供并行对照测试作黄金参考）。见 [`minmax_index`]。
/// Serial rolling min/max indices (feature-agnostic; golden reference for the parallel equality test). See [`minmax_index`].
pub fn minmax_index_serial(values: &[f64], time_period: usize) -> Result<MinMaxIndex, TaError> {
    check_period(time_period)?;
    let n = values.len();
    let mut out = MinMaxIndex {
        min_idx: vec![f64::NAN; n],
        max_idx: vec![f64::NAN; n],
    };
    minmax_index_serial_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 滚动窗口最小/最大索引多核并行版本（需 `parallel` feature）。复用 [`minmax_index_serial_with_output`]
/// 的最左 tie-break 单遍单调队列，以 `period-1` 前导重叠播种各分块，输出与串行逐项 1:1。
/// Multi-core parallel rolling min/max indices (requires the `parallel` feature). Reuses the
/// leftmost tie-break single-pass monotonic queue of [`minmax_index_serial_with_output`] with
/// `period-1` leading overlap; output is 1:1 with the serial path.
#[cfg(feature = "parallel")]
pub fn minmax_index_parallel(values: &[f64], time_period: usize) -> Result<MinMaxIndex, TaError> {
    check_period(time_period)?;
    let n = values.len();
    let mut out = MinMaxIndex {
        min_idx: vec![f64::NAN; n],
        max_idx: vec![f64::NAN; n],
    };
    minmax_index_parallel_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 滚动窗口最小/最大索引并行内核（零拷贝写入 `out`）。见 [`minmax_index_parallel`]。
/// Parallel kernel for rolling min/max indices (zero-copy into `out`). See [`minmax_index_parallel`].
#[cfg(feature = "parallel")]
fn minmax_index_parallel_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut MinMaxIndex,
) -> Result<(), TaError> {
    check_period(time_period)?;
    let n = values.len();
    if out.min_idx.len() != n || out.max_idx.len() != n {
        return Err(TaError::BadParam(
            "minmax_index_parallel_with_output: out vectors must have length == values length".into(),
        ));
    }
    let p = time_period;
    crate::parallel::parallel_index_map_2(
        n,
        p - 1,
        &mut out.min_idx,
        &mut out.max_idx,
        |start, end| {
            let off = start as f64;
            let mut min_idx = rolling_extreme_index(&values[start..end], p, false);
            let mut max_idx = rolling_extreme_index(&values[start..end], p, true);
            // 切片使索引变为相对值，须平移回绝对位置；前导 `period-1` 个固定为 0.0（与原版一致）。
            // The slice makes indices relative; shift them back to absolute. The leading
            // `period-1` positions stay 0.0 (matches TA-Lib).
            for (i, v) in min_idx.iter_mut().enumerate() {
                if i >= p - 1 {
                    *v += off;
                }
            }
            for (i, v) in max_idx.iter_mut().enumerate() {
                if i >= p - 1 {
                    *v += off;
                }
            }
            (min_idx, max_idx)
        },
    );
    Ok(())
}
