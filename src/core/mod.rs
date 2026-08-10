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
/// 采用 **O(n) 滑动递推**（P2-3，ADR 0010）：维护朴素窗口和 `sw`（以 `sw += x[i]-x[i-period]`
/// 在 O(1) 内滑动），并以闭式递推 `W[i] = W[i-1] + period·x[i] - sw[i-1]` 更新加权累加，
/// 消除原朴素实现每窗口 `period` 次重复乘加。首个窗口沿用朴素求和作为种子，保证与历史实现
/// 逐项对齐（数值同黄金向量一致，ADR 0005）。
///
/// Uses an O(n) sliding recurrence (P2-3, ADR 0010): a plain window sum `sw` is slid in O(1)
/// via `sw += x[i] - x[i-period]`, and a closed-form `W[i] = W[i-1] + period·x[i] - sw[i-1]`
/// updates the weighted accumulator — eliminating the naïve per-window `period` multiply-adds.
/// The first window uses the naïve sum as a seed so it stays aligned with the historical impl
/// (1:1 with the golden vector, ADR 0005).
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
    // 种子：首个窗口（i = period-1）的朴素加权和与朴素窗口和。
    // Seed: the naïve weighted sum and naïve window sum of the first window (i = period-1).
    let mut sw = 0.0_f64; // 朴素窗口和 / plain window sum
    for j in 0..period {
        sw += values[j];
    }
    let mut w = 0.0_f64; // 加权累加 / weighted accumulator
    for j in 0..period {
        w += values[(period - 1) - j] * (period - j) as f64;
    }
    out[period - 1] = w / denom;
    for i in period..n {
        // 递推：W[i] = W[i-1] + period·x[i] - sw[i-1]（此时 sw 仍为窗口和至 i-1）。
        // Recur: W[i] = W[i-1] + period·x[i] - sw[i-1] (sw here still ends at i-1).
        w = w + (period as f64) * values[i] - sw;
        out[i] = w / denom;
        // 滑动窗口和：加入右端新元素，剔除左端出窗元素。/ slide window sum.
        sw += values[i] - values[i - period];
    }
    out
}

/// 朴素 O(n·period) 加权移动平均，仅作为 [`wma`] 的单元测试对照（非热路径）。
///
/// Naïve O(n·period) weighted moving average — used only as the reference in unit tests for
/// [`wma`]; not on any hot path.
#[allow(dead_code)]
fn wma_naive(values: &[f64], period: usize) -> Vec<f64> {
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
/// 采用 **单调队列** O(n) 实现（P2-2，ADR 0010）：以双端队列维护窗口内的单调候选，每元素
/// 入队/出队均摊 O(1)。并列极值取**窗口内最右**者（弹出 `<=`/`>=` 候选），与朴素扫描
/// [`rolling_extreme_naive`] 的 tie-break 完全一致，数值逐项相等（零偏差，ADR 0005）。
///
/// Uses an O(n) monotonic-queue (P2-2, ADR 0010): a deque maintains the monotonic candidates
/// inside the window so each element is enqueued/dequeued in amortized O(1). Ties resolve to
/// the **rightmost** occurrence in the window (popping `<=`/`>=` candidates), which matches the
/// tie-break of the naïve [`rolling_extreme_naive`] scan exactly — bit-for-bit equal (ADR 0005).
#[inline]
fn rolling_extreme(values: &[f64], period: usize, take_max: bool) -> Vec<f64> {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if n < period {
        return out;
    }
    let mut dq: std::collections::VecDeque<usize> = std::collections::VecDeque::with_capacity(period);
    for i in 0..n {
        // 移除已滑出窗口的最左候选 / drop the leftmost candidate that left the window
        while let Some(&front) = dq.front() {
            if front + period <= i {
                dq.pop_front();
            } else {
                break;
            }
        }
        if take_max {
            // 弹出 <= 候选者（含相等），使队首为窗口最右最大值
            // pop <= candidates (incl. equal) so the front is the rightmost max
            while let Some(&back) = dq.back() {
                if values[back] <= values[i] {
                    dq.pop_back();
                } else {
                    break;
                }
            }
        } else {
            // 弹出 >= 候选者，使队首为窗口最右最小值
            // pop >= candidates so the front is the rightmost min
            while let Some(&back) = dq.back() {
                if values[back] >= values[i] {
                    dq.pop_back();
                } else {
                    break;
                }
            }
        }
        dq.push_back(i);
        if i >= period - 1 {
            out[i] = values[*dq.front().unwrap()];
        }
    }
    out
}

/// 朴素 O(n·period) 窗口极值扫描，仅作为 [`rolling_extreme`] 的单元测试对照（非热路径）。
///
/// Naïve O(n·period) window-scan extreme — used only as the reference in unit tests for
/// [`rolling_extreme`]; not on any hot path.
#[allow(dead_code)]
fn rolling_extreme_naive(values: &[f64], period: usize, take_max: bool) -> Vec<f64> {
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

/// 滚动窗口的最大与最小，**同一次遍历** O(n)（用于 `MIDPOINT` 的 `(max+min)/2`）。
///
/// Rolling max **and** min in a single O(n) pass — for `MIDPOINT`'s `(max+min)/2`. Two
/// monotonic deques (decreasing for max, increasing for min) advance together; ties resolve to
/// the rightmost extreme (same `<=`/`>=` pop rule as [`rolling_extreme`]), so the per-element
/// `max`/`min` equal the separate calls exactly (ADR 0005).
///
/// 对应 TA-Lib `TA_MIDPOINT` 内部 `MINMAXINDEX` 的单遍双队列思路，将 `midpoint` 的两次窗口
/// 扫描合并为一次，规避重复遍历开销。
///
/// Mirrors TA-Lib `TA_MIDPOINT`'s internal `MINMAXINDEX` single-pass dual-deque approach, merging
/// `midpoint`'s two window scans into one to avoid the redundant traversal.
#[inline]
pub(crate) fn rolling_minmax(values: &[f64], period: usize) -> (Vec<f64>, Vec<f64>) {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut max_out = vec![f64::NAN; n];
    let mut min_out = vec![f64::NAN; n];
    if n < period {
        return (max_out, min_out);
    }
    let mut max_dq: std::collections::VecDeque<usize> =
        std::collections::VecDeque::with_capacity(period);
    let mut min_dq: std::collections::VecDeque<usize> =
        std::collections::VecDeque::with_capacity(period);
    for i in 0..n {
        // 移除已滑出窗口的最左候选（两个队列同步）。/ drop out-of-window leftmost (both deques).
        while let Some(&f) = max_dq.front() {
            if f + period <= i {
                max_dq.pop_front();
            } else {
                break;
            }
        }
        while let Some(&f) = min_dq.front() {
            if f + period <= i {
                min_dq.pop_front();
            } else {
                break;
            }
        }
        // 最大值队列（递减）：弹出 <= 候选者 / max deque (decreasing): pop <= candidates
        while let Some(&b) = max_dq.back() {
            if values[b] <= values[i] {
                max_dq.pop_back();
            } else {
                break;
            }
        }
        // 最小值队列（递增）：弹出 >= 候选者 / min deque (increasing): pop >= candidates
        while let Some(&b) = min_dq.back() {
            if values[b] >= values[i] {
                min_dq.pop_back();
            } else {
                break;
            }
        }
        max_dq.push_back(i);
        min_dq.push_back(i);
        if i >= period - 1 {
            max_out[i] = values[*max_dq.front().unwrap()];
            min_out[i] = values[*min_dq.front().unwrap()];
        }
    }
    (max_out, min_out)
}

/// 滚动窗口极值**索引**，单遍单调队列 O(n)（平局取最左 / leftmost）。
///
/// Rolling-extreme **index** in a single O(n) monotonic-queue pass (leftmost on ties). Returns
/// the absolute (0-based) position of the window extreme (max when `take_max`, min otherwise);
/// the leading `period - 1` positions are `0.0` (matching TA-Lib `TA_MAXINDEX` / `TA_MININDEX`,
/// which emit `0.0`, not `NaN`).
///
/// 与 [`rolling_extreme`]（值变体，最右 tie-break）互为镜像：此处弹出条件用**严格** `<` / `>`
/// 而非 `<=` / `>=`，使并列极值保留更靠左（更小索引）的候选，从而复刻 TA-Lib 索引变体的
/// 最左 tie-break（见 `math_ops::max_index` / `min_index` 文档）。在有限输入上与朴素
/// `O(n·period)` 扫描逐项相等（零偏差，ADR 0005）。
///
/// Mirrors [`rolling_extreme`] (the value variant, rightmost tie-break): here the pop condition
/// is the **strict** `<` / `>` (not `<=` / `>=`), so equal extremes keep the leftmost (smaller
/// index) candidate — reproducing TA-Lib's leftmost tie-break for the index variants. Bit-for-bit
/// equal to the naïve `O(n·period)` scan on finite inputs (ADR 0005).
pub(crate) fn rolling_extreme_index(values: &[f64], period: usize, take_max: bool) -> Vec<f64> {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut out = vec![0.0_f64; n];
    if n < period {
        return out;
    }
    let mut dq: std::collections::VecDeque<usize> = std::collections::VecDeque::with_capacity(period);
    for i in 0..n {
        // 移除已滑出窗口的最左候选 / drop the leftmost candidate that left the window
        while let Some(&front) = dq.front() {
            if front + period <= i {
                dq.pop_front();
            } else {
                break;
            }
        }
        if take_max {
            // 弹出 < 候选者（严格小于），相等者保留 -> 队首为窗口最左最大值
            // pop < candidates (strict), keep equal -> front is the leftmost max
            while let Some(&back) = dq.back() {
                if values[back] < values[i] {
                    dq.pop_back();
                } else {
                    break;
                }
            }
        } else {
            // 弹出 > 候选者（严格大于），相等者保留 -> 队首为窗口最左最小值
            // pop > candidates (strict), keep equal -> front is the leftmost min
            while let Some(&back) = dq.back() {
                if values[back] > values[i] {
                    dq.pop_back();
                } else {
                    break;
                }
            }
        }
        dq.push_back(i);
        if i >= period - 1 {
            out[i] = *dq.front().unwrap() as f64;
        }
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

/// 单遍滚动「均值 + 总体方差」（BBANDS 中轨/标准差融合核，P2-4，ADR 0010）。
///
/// Single-pass rolling mean **and** population variance, fused into one window traversal.
///
/// 维护同一个滑动窗口和 `sx` 与滑动平方和 `sxx`（与 [`rolling_mean`] / [`rolling_var`]
/// 完全相同的递推与加法顺序），一次遍历同时产出算术均值与总体方差，消除 BBANDS 原先
/// 「先 `rolling_mean` 再 `rolling_var`」的两遍扫描。因加法顺序与两个独立原语逐一相同，
/// 产出值与分开调用逐项相等（零偏差，ADR 0005）。
///
/// Maintains the same sliding `sx` and `sxx` (identical recurrence and addition order to
/// [`rolling_mean`] / [`rolling_var`]); one traversal yields both the mean and the population
/// variance, eliminating the two-pass `rolling_mean` + `rolling_var` scan in BBANDS. Outputs
/// are element-wise equal to calling the two primitives separately (ADR 0005).
///
/// 返回 `(mean, var)`，均与 `values` 等长，前导 `period-1` 个为 [`f64::NAN`]。
/// Returns `(mean, var)`, equal-length to `values`; leading `period - 1` are [`f64::NAN`].
///
/// # Panics
/// 调用方须保证 `period >= 1`。/ Caller must ensure `period >= 1`.
#[inline]
pub fn rolling_mean_var(values: &[f64], period: usize) -> (Vec<f64>, Vec<f64>) {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut mean = vec![f64::NAN; n];
    let mut var = vec![f64::NAN; n];
    if n < period {
        return (mean, var);
    }
    let p = period as f64;
    // 种子：首个窗口（i = period-1）的朴素窗口和 `sx` 与朴素平方和 `sxx`。
    // Seed: naïve window sum `sx` and naïve sum-of-squares `sxx` of the first window.
    let mut sx = values[..period].iter().copied().sum::<f64>();
    let mut sxx = values[..period].iter().map(|v| v * v).sum::<f64>();
    mean[period - 1] = sx / p;
    var[period - 1] = (sxx - sx * sx / p) / p;
    for i in period..n {
        // 与 `rolling_mean` / `rolling_var` 完全相同的滑动递推顺序。
        // Same sliding recurrence order as `rolling_mean` / `rolling_var`.
        sx += values[i] - values[i - period];
        sxx += values[i] * values[i] - values[i - period] * values[i - period];
        mean[i] = sx / p;
        var[i] = (sxx - sx * sx / p) / p;
    }
    (mean, var)
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
