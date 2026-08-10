//! 内部公共数学原语（非公开模块）。
//!
//! Internal common math primitives (private module). Used by indicator implementations
//! to avoid re-deriving shared numerics (rolling means, EMA, WMA, rolling extremes, ...).

pub mod defaults;

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

/// 计算窗口为 `period` 的滚动均值（简单移动平均的核心）。
///
/// Compute the rolling mean with window `period` — the core of a simple moving average.
///
/// 返回与 `values` 等长的 `Vec<f64>`：前导 `period-1` 个位置填 [`f64::NAN`]
/// （不稳定期，见 ADR 0007），其余位置为对应窗口的算术均值。
///
/// Returns a `Vec<f64>` with the same length as `values`: the leading `period - 1`
/// positions are filled with [`f64::NAN`] (unstable period, see ADR 0007), the rest are
/// the arithmetic mean of the corresponding window.
///
/// # 公式 / Formula
/// ```text
/// SMA[i] = (1/period) * Σ_{k=i-period+1}^{i} values[k],  i >= period-1
/// ```
///
/// # Panics
/// 调用方须保证 `period >= 1`。/ Caller must ensure `period >= 1`.
#[inline]
pub fn rolling_mean(values: &[f64], period: usize) -> Vec<f64> {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if n < period {
        return out;
    }
    let mut sum: f64 = values[..period].iter().copied().sum();
    out[period - 1] = sum / period as f64;
    for i in period..n {
        sum += values[i] - values[i - period];
        out[i] = sum / period as f64;
    }
    out
}

/// 指数移动平均（TA-Lib `TA_EMA`，经典种子），与输入等长的全索引向量。
///
/// Exponential moving average (TA-Lib `TA_EMA`, classic seed), a full-indexed vector
/// with the same length as the input.
///
/// 对齐与种子严格对照 TA-Lib 0.7.1 默认兼容性（`TA_MA_CLASSIC`）：
/// 首个有效值 = 前 `period` 个有限值的算术均值（SMA 种子），后续按
/// `out[i] = (x[i] - out[i-1]) * k + out[i-1]`，其中 `k = 2/(period+1)` 递推。
/// 前导 `period-1` 个位置为 [`f64::NAN`]。
///
/// Alignment & seeding replicate TA-Lib 0.7.1 default compatibility (`TA_MA_CLASSIC`):
/// the first valid value is the SMA of the first `period` finite values (the seed), then
/// `out[i] = (x[i] - out[i-1]) * k + out[i-1]` with `k = 2/(period+1)`. The leading
/// `period - 1` positions are [`f64::NAN`].
///
/// 输入可含前导 [`f64::NAN`]：种子从首个有限值算起（用于 DEMA/TEMA 的嵌套 EMA）。
/// Inputs may carry leading [`f64::NAN`]; the seed starts at the first finite value
/// (used by nested EMAs in DEMA/TEMA).
///
/// # 公式 / Formula
/// ```text
/// seed     = (1/period) * Σ_{k=0}^{period-1} x[start+k]
/// out[i]   = (x[i] - out[i-1]) * 2/(period+1) + out[i-1],  i > start+period-1
/// ```
/// 来源 / Source: TA-Lib `ta_ema.c`（经典种子）。
///
/// # Panics
/// 调用方须保证 `period >= 1`。/ Caller must ensure `period >= 1`.
pub fn ema(values: &[f64], period: usize) -> Vec<f64> {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    let start = match values.iter().position(|&x| !x.is_nan()) {
        Some(s) => s,
        None => return out,
    };
    if n - start < period {
        return out;
    }
    let seed: f64 = values[start..start + period].iter().copied().sum::<f64>() / period as f64;
    out[start + period - 1] = seed;
    let k = 2.0 / (period as f64 + 1.0);
    let mut prev = seed;
    for i in (start + period)..n {
        prev = (values[i] - prev) * k + prev;
        out[i] = prev;
    }
    out
}

/// 嵌套 EMA 级联（单遍融合核，DEMA / TEMA / T3 共用）。
///
/// Nested EMA cascade — a single forward pass that produces `E1..EL` so that DEMA / TEMA /
/// T3 can be combined without allocating `L` intermediate `Vec<f64>` or scanning the series
/// `L` times. At each index `i` we compute `E1[i]`, feed it into `E2`, `E2[i]` into `E3`, …
/// all in one loop, holding only `O(L)` scalar state.
///
/// 每层 `k` 以其输入（上一层输出）的**首个 `period` 个有限值**的算术均值作 SMA 种子，
/// 之后按 `k = 2/(period+1)` 递推 —— 与逐次调用 [`ema`] 完全一致（同一次求和顺序、同一
/// 递推），数值逐项相等、零偏差（ADR 0005）。
///
/// Each level `k` seeds with the SMA of the first `period` finite values of its input (the
/// previous level's output), then recurses with `k = 2/(period+1)` — identical to calling
/// [`ema`] repeatedly (same summation order, same recursion), bit-for-bit equal (ADR 0005).
///
/// `combine` 接收当前索引的全部 `E1..EL`，写出最终组合值（如 DEMA: `2*E1 - E2`）。
/// `combine` receives the full `E1..EL` at the current index and writes the final combined
/// value. The output is [`f64::NAN`] until the deepest level `EL` first becomes valid.
///
/// # Panics
/// 调用方须保证 `period >= 1` 且 `out.len() == values.len()`。/ Caller must ensure
/// `period >= 1` and `out.len() == values.len()`.
#[inline]
pub fn nested_ema_with_output<const L: usize, F>(
    values: &[f64],
    period: usize,
    mut combine: F,
    out: &mut [f64],
) where
    F: FnMut(&[f64; L]) -> f64,
{
    debug_assert!(period >= 1);
    debug_assert_eq!(out.len(), values.len());
    let n = values.len();
    if n == 0 {
        return;
    }
    let k = 2.0 / (period as f64 + 1.0);
    let pk = period as f64;
    let nan = f64::NAN;
    // 每层状态：当前值、递推上一值、种子累加和、种子计数、是否已播种。
    // Per-level state: current value, previous (recursion) value, seed accumulator,
    // seed count, seeded flag.
    let mut e = [nan; L];
    let mut prev = [nan; L];
    let mut seed_sum = [0.0f64; L];
    let mut seed_n = [0usize; L];
    let mut seeded = [false; L];

    for i in 0..n {
        // Level 1: raw input series. Before seeding, only finite values accumulate the seed.
        let v1 = values[i];
        if !seeded[0] {
            if !v1.is_nan() {
                seed_sum[0] += v1;
                seed_n[0] += 1;
                if seed_n[0] == period {
                    e[0] = seed_sum[0] / pk;
                    seeded[0] = true;
                    prev[0] = e[0];
                }
            }
        } else {
            // Once seeded, every index recurses (a NaN input propagates NaN, matching `ema`).
            e[0] = (v1 - prev[0]) * k + prev[0];
            prev[0] = e[0];
        }

        // Levels 2..=L: each consumes the previous level's freshly computed output.
        for l in 1..L {
            let src = e[l - 1];
            if !seeded[l] {
                if !src.is_nan() {
                    seed_sum[l] += src;
                    seed_n[l] += 1;
                    if seed_n[l] == period {
                        e[l] = seed_sum[l] / pk;
                        seeded[l] = true;
                        prev[l] = e[l];
                    }
                }
            } else {
                e[l] = (src - prev[l]) * k + prev[l];
                prev[l] = e[l];
            }
        }

        // Combine only once the deepest level is valid; otherwise keep NaN.
        out[i] = if seeded[L - 1] { combine(&e) } else { nan };
    }
}

/// 加权移动平均（TA-Lib `TA_WMA`），与输入等长的全索引向量。
///
/// Weighted moving average (TA-Lib `TA_WMA`), a full-indexed vector with the same length
/// as the input.
///
/// 权重为 `period, period-1, ..., 1`（最新价权重最大），归一化除以 `period*(period+1)/2`。
/// 前导 `period-1` 个位置为 [`f64::NAN`]。
///
/// Weights are `period, period-1, ..., 1` (most-recent price weighted highest),
/// normalized by the sum `period*(period+1)/2`. The leading `period - 1` positions are
/// [`f64::NAN`].
///
/// # 公式 / Formula
/// ```text
/// WMA[i] = Σ_{j=0}^{period-1} (period-j) * x[i-j]   /   (period*(period+1)/2),  i >= period-1
/// ```
/// 来源 / Source: TA-Lib `ta_wma.c`.
///
/// # Panics
/// 调用方须保证 `period >= 1`。/ Caller must ensure `period >= 1`.
pub fn wma(values: &[f64], period: usize) -> Vec<f64> {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if n < period {
        return out;
    }
    let denom = (period * (period + 1) / 2) as f64;
    for i in (period - 1)..n {
        let mut s = 0.0;
        for j in 0..period {
            s += values[i - j] * (period - j) as f64;
        }
        out[i] = s / denom;
    }
    out
}

/// 滚动窗口极值（最大或最小），与输入等长的全索引向量，前导 `period-1` 为 [`f64::NAN`]。
///
/// Rolling-window extreme (max or min), a full-indexed vector with the same length as the
/// input; the leading `period - 1` positions are [`f64::NAN`].
///
/// 当前为朴素 O(n·period) 窗口扫描，满足正确性优先；后续可在性能敏感路径改用单调队列 O(n)。
/// Currently a naïve O(n·period) window scan (correctness first); a monotonic-queue O(n)
/// variant can be introduced later on hot paths.
#[inline]
fn rolling_extreme(values: &[f64], period: usize, take_max: bool) -> Vec<f64> {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if n < period {
        return out;
    }
    for i in (period - 1)..n {
        let mut acc = values[i];
        for j in 1..period {
            let v = values[i - j];
            if take_max {
                if v > acc {
                    acc = v;
                }
            } else if v < acc {
                acc = v;
            }
        }
        out[i] = acc;
    }
    out
}

/// 滚动窗口最大值（用于 MIDPOINT / MIDPRICE 的 `max` 侧）。
/// Rolling window maximum (the `max` side of MIDPOINT / MIDPRICE).
#[inline]
pub fn rolling_max(values: &[f64], period: usize) -> Vec<f64> {
    rolling_extreme(values, period, true)
}

/// 滚动窗口最小值（用于 MIDPOINT / MIDPRICE 的 `min` 侧）。
/// Rolling window minimum (the `min` side of MIDPOINT / MIDPRICE).
#[inline]
pub fn rolling_min(values: &[f64], period: usize) -> Vec<f64> {
    rolling_extreme(values, period, false)
}

/// 滚动窗口求和，与输入等长的全索引向量，前导 `period-1` 个为 [`f64::NAN`]。
///
/// Rolling window sum; the leading `period - 1` positions are [`f64::NAN`].
///
/// 跳过输入前导 [`f64::NAN`]，种子取首个有限值起的 `period` 个之和（置于 `start+period-1`），
/// 之后滑动递推。用于 CMO / MFI / ULTOSC 等需要窗口内正负变动求和的指标。
///
/// Skips a leading [`f64::NAN`] prefix; the seed is the sum of the first `period` finite
/// values (placed at `start + period - 1`), then slides. Used by CMO / MFI / ULTOSC, which
/// need the windowed sum of positive/negative moves.
#[inline]
pub fn rolling_sum(values: &[f64], period: usize) -> Vec<f64> {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if n < period {
        return out;
    }
    let start = match values.iter().position(|&x| !x.is_nan()) {
        Some(s) => s,
        None => return out,
    };
    if n - start < period {
        return out;
    }
    let mut sum: f64 = values[start..start + period].iter().copied().sum();
    out[start + period - 1] = sum;
    for i in (start + period)..n {
        sum += values[i] - values[i - period];
        out[i] = sum;
    }
    out
}

/// 滚动均值，跳过输入前导 [`f64::NAN`]，置于首个有限值起 `period` 个之后的位置。
///
/// Rolling mean that skips a leading [`f64::NAN`] prefix, placed at `start + period - 1`.
///
/// 当输入本身带有前导不稳定期（如 STOCH 的 `%K` 快速线）时使用：种子取首个有限值起的
/// `period` 个均值，之后滑动递推。
///
/// Used where the input itself carries a leading unstable prefix (e.g. STOCH's fast `%K`):
/// the seed is the mean of the first `period` finite values, then it slides.
#[inline]
pub fn rolling_mean_skip(values: &[f64], period: usize) -> Vec<f64> {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    let start = match values.iter().position(|&x| !x.is_nan()) {
        Some(s) => s,
        None => return out,
    };
    if n - start < period {
        return out;
    }
    let mut sum: f64 = values[start..start + period].iter().copied().sum();
    out[start + period - 1] = sum / period as f64;
    for i in (start + period)..n {
        sum += values[i] - values[i - period];
        out[i] = sum / period as f64;
    }
    out
}

/// Wilder 平滑（SMMA，TA-Lib 方向性运动 / RSI 族所用），跳过前导 [`f64::NAN`]。
///
/// Wilder smoothing (SMMA) as used by TA-Lib's directional-movement and RSI family; skips
/// a leading [`f64::NAN`] prefix.
///
/// 种子 = 首个有限值起的 `period` 个均值（置于 `start+period-1`），之后按
/// `prev = prev + (x - prev) * k`、`k = 1/period` 递推（等同于 `(prev*(period-1) + x)/period`）。
///
/// Seed = mean of the first `period` finite values (placed at `start + period - 1`), then
/// `prev = prev + (x - prev) * k` with `k = 1/period` (equivalently `(prev*(period-1) + x)/period`).
///
/// 与 [`ema`]（经典 `k = 2/(period+1)`）的区别仅在于平滑常数；种子策略一致。
/// Differs from [`ema`] (classic `k = 2/(period+1)`) only in the smoothing constant; the
/// seeding strategy is the same.
///
/// # Panics
/// 调用方须保证 `period >= 1`。/ Caller must ensure `period >= 1`.
#[inline]
pub fn ema_wilder(values: &[f64], period: usize) -> Vec<f64> {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    let start = match values.iter().position(|&x| !x.is_nan()) {
        Some(s) => s,
        None => return out,
    };
    if n - start < period {
        return out;
    }
    let seed: f64 = values[start..start + period].iter().copied().sum::<f64>() / period as f64;
    out[start + period - 1] = seed;
    let k = 1.0 / period as f64;
    let mut prev = seed;
    for i in (start + period)..n {
        prev = prev + (values[i] - prev) * k;
        out[i] = prev;
    }
    out
}

/// 真实波幅（True Range，TA-Lib `TA_TRANGE` 的内核）。
///
/// True Range (kernel of TA-Lib `TA_TRANGE`). Used directly by `trange` and as the
/// input to `atr` / `natr`.
///
/// - 索引 0：`NaN`（`TA_TRANGE` 需要前一收盘价 `close[i-1]`，首根无前收盘价）。
///   Index 0 is `NaN`: TA-Lib's TRANGE requires the previous close `close[i-1]`, which
///   does not exist for the first bar.
/// - `TR[i] = max(high[i], close[i-1]) - min(low[i], close[i-1])`，`i >= 1`.
///
/// 返回值长度与输入一致；若任意相邻长度不一致，以三者最短者为准。
/// Returns a vector with the same length as the inputs (truncated to the shortest).
#[inline]
pub fn true_range(high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> {
    let n = high.len().min(low.len()).min(close.len());
    let mut out = vec![f64::NAN; n];
    if n == 0 {
        return out;
    }
    // 首根需前一收盘价，TA-Lib 此处输出 NaN。The first bar needs a prior close -> NaN.
    out[0] = f64::NAN;
    for i in 1..n {
        let hl = high[i] - low[i];
        let hc = (high[i] - close[i - 1]).abs();
        let lc = (low[i] - close[i - 1]).abs();
        out[i] = hl.max(hc).max(lc);
    }
    out
}

/// 滚动总体方差（population variance），与输入等长的全索引向量，前导 `period-1` 为 [`f64::NAN`]。
///
/// Rolling population variance with the same length as the input; the leading `period - 1`
/// positions are [`f64::NAN`]. Divides by `period` (population), matching TA-Lib `TA_VAR` /
/// `TA_STDDEV` defaults.
///
/// # 公式 / Formula
/// ```text
/// VAR[i] = (Σ x² - (Σ x)² / period) / period,  i >= period-1
/// ```
/// 来源 / Source: TA-Lib `ta_var.c` / `ta_stddev.c`.
#[inline]
pub fn rolling_var(values: &[f64], period: usize) -> Vec<f64> {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if n < period {
        return out;
    }
    let p = period as f64;
    let mut sx = values[..period].iter().copied().sum::<f64>();
    let mut sxx = values[..period].iter().map(|v| v * v).sum::<f64>();
    out[period - 1] = (sxx - sx * sx / p) / p;
    for i in period..n {
        sx += values[i] - values[i - period];
        sxx += values[i] * values[i] - values[i - period] * values[i - period];
        out[i] = (sxx - sx * sx / p) / p;
    }
    out
}
