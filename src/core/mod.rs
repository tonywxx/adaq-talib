//! 内部公共数学原语（非公开模块）。
//!
//! Internal common math primitives (private module). Used by indicator implementations
//! to avoid re-deriving shared numerics (rolling means, EMA, WMA, rolling extremes, ...).
//!
//! 本模块按职责拆分为子模块，但全部符号经 `pub(crate) use ...::*` 在 `crate::core`
//! 命名空间内重新导出，因此各指标模块的 `use crate::core::{...}` 路径保持不变：
//! - [`window`]：通用滑动窗口聚合（均值 / 求和 / 方差 / 加权移动平均）。
//! - [`ema`]：指数平滑族（EMA / Wilder / 嵌套 EMA 级联）。
//! - [`extreme`]：环形缓冲单调队列与滚动极值（最大 / 最小 / 索引）。
//! - [`kernel`]：TA 特定核（依赖 OHLC 三元组的 true_range）。
//!
//! Split by responsibility into submodules, but every symbol is re-exported into the
//! `crate::core` namespace via `pub(crate) use ...::*`, so indicator modules' existing
//! `use crate::core::{...}` paths are unchanged.

pub mod defaults;

pub(crate) mod window;
pub(crate) mod ema;
pub(crate) mod extreme;
pub(crate) mod kernel;

pub(crate) use window::*;
pub(crate) use ema::*;
pub(crate) use extreme::*;
pub(crate) use kernel::*;

/// 校验多数组长度一致（对应 TA-Lib 多输入函数的长度约束）。
/// Validate that several slices share the same length.
pub(crate) fn check_eq_len(lists: &[&[f64]], name: &str) -> Result<(), crate::error::TaError> {
    let len = lists[0].len();
    for l in lists.iter().skip(1) {
        if l.len() != len {
            return Err(crate::error::TaError::BadParam(format!(
                "{name}: all input arrays must have equal length"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 单调队列实现必须与朴素 O(n·period) 扫描逐项相等（含并列极值的 tie-break）。
    /// The monotonic-queue impl must equal the naïve scan element-wise (incl. tie-breaks).
    #[test]
    fn rolling_extreme_matches_naive() {
        // 确定性 LCG：覆盖随机序列与重复极值（制造并列 tie-break 场景）。
        // Deterministic LCG covering random series and duplicate extremes (tie-break cases).
        let mut x: u64 = 0x1234_5678_9abc_def0;
        let mut lcg = || {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((x >> 11) as f64 / (1u64 << 53) as f64) * 100.0 - 50.0
        };
        for &n in &[0usize, 1, 2, 5, 20, 137, 1000] {
            for &p in &[1usize, 2, 3, 7, 20, 64] {
                if n < p {
                    continue;
                }
                let mut v = Vec::with_capacity(n);
                for _ in 0..n {
                    // 1/8 概率注入与外部极值相同的重复值，专测并列最右 tie-break。
                    // 1/8 chance of a duplicate value to exercise the rightmost tie-break.
                    let r = lcg();
                    if (r as usize) % 8 == 0 {
                        v.push(42.0);
                    } else {
                        v.push(r);
                    }
                }
                for &take_max in &[true, false] {
                    let fast = rolling_extreme(&v, p, take_max);
                    let naive = rolling_extreme_naive(&v, p, take_max);
                    for i in 0..n {
                        assert!(
                            (fast[i].is_nan() && naive[i].is_nan())
                                || (fast[i] - naive[i]).abs() < 1e-12,
                            "mismatch @ n={n} p={p} max={take_max} i={i}: {} vs {}",
                            fast[i],
                            naive[i]
                        );
                    }
                }
            }
        }
    }

    /// 单遍单调队列索引实现必须与朴素 O(n·period) 扫描逐项相等，且平局取最左。
    /// The single-pass index impl must equal the naïve scan element-wise, leftmost on ties.
    #[test]
    fn rolling_extreme_index_matches_naive_leftmost() {
        // 确定性 LCG：覆盖随机序列与重复极值（制造并列最左 tie-break 场景）。
        // Deterministic LCG covering random series and duplicate extremes (leftmost ties).
        let mut x: u64 = 0x1234_5678_9abc_def0;
        let mut lcg = || {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((x >> 11) as f64 / (1u64 << 53) as f64) * 100.0 - 50.0
        };
        // 朴素最左极值索引：窗口 [i-period+1, i]，平局取最小索引。
        // Naïve leftmost extreme index over window [i-period+1, i].
        let naive_index = |v: &[f64], p: usize, take_max: bool| -> Vec<f64> {
            let n = v.len();
            let mut out = vec![0.0_f64; n];
            if n < p {
                return out;
            }
            for i in (p - 1)..n {
                let mut best = v[i];
                let mut best_idx = i;
                for j in 1..p {
                    let val = v[i - j];
                    let better = if take_max { val >= best } else { val <= best };
                    if better {
                        best = val;
                        best_idx = i - j;
                    }
                }
                out[i] = best_idx as f64;
            }
            out
        };
        for &n in &[0usize, 1, 2, 5, 20, 137, 1000] {
            for &p in &[1usize, 2, 3, 7, 20, 64] {
                if n < p {
                    continue;
                }
                let mut v = Vec::with_capacity(n);
                for _ in 0..n {
                    // 1/8 概率注入重复极值，专测并列最左 tie-break。
                    // 1/8 chance of a duplicate extreme to exercise the leftmost tie-break.
                    let r = lcg();
                    if (r as usize) % 8 == 0 {
                        v.push(42.0);
                    } else {
                        v.push(r);
                    }
                }
                for &take_max in &[true, false] {
                    let fast = rolling_extreme_index(&v, p, take_max);
                    let naive = naive_index(&v, p, take_max);
                    assert_eq!(
                        fast, naive,
                        "mismatch @ n={n} p={p} max={take_max}: index impl diverges from naïve"
                    );
                }
            }
        }
    }

    /// 滑动递推 WMA 必须与朴素 O(n·period) 扫描逐项相等（含不同窗口长度与序列形态）。
    /// The sliding-recurrence WMA must equal the naïve O(n·period) scan element-wise
    /// (across window sizes and series shapes).
    #[test]
    fn wma_matches_naive() {
        // 确定性 LCG：覆盖随机序列、单调递增/递减、含重复值等形态。
        // Deterministic LCG covering random, monotonic, and duplicate-value shapes.
        let mut x: u64 = 0x9e37_79b9_7f4a_7c15;
        let mut lcg = || {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((x >> 11) as f64 / (1u64 << 53) as f64) * 100.0 - 50.0
        };
        for &n in &[0usize, 1, 2, 5, 20, 137, 1000] {
            for &p in &[1usize, 2, 3, 7, 20, 64, 200] {
                if n < p {
                    continue;
                }
                let mut v = Vec::with_capacity(n);
                for _ in 0..n {
                    v.push(lcg());
                }
                let fast = wma(&v, p);
                let naive = wma_naive(&v, p);
                for i in 0..n {
                    assert!(
                        (fast[i].is_nan() && naive[i].is_nan())
                            || (fast[i] - naive[i]).abs() < 1e-9,
                        "mismatch @ n={n} p={p} i={i}: {} vs {}",
                        fast[i],
                        naive[i]
                    );
                }
            }
        }
    }
}
