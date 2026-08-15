//! 通用滑动窗口聚合（均值 / 求和 / 方差 / 加权移动平均）。
//!
//! Generic sliding-window aggregations (mean / sum / variance / weighted MA).
//! 与具体指标无关，可独立测试；不依赖 EMA 或极值队列。

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
/// 加权移动平均，零拷贝写入 `out`（与 `values` 等长）。见 [`wma`]。
/// Weighted Moving Average, written zero-copy into `out`. See [`wma`].
pub fn wma_with_output(values: &[f64], period: usize, out: &mut [f64]) {
    debug_assert!(period >= 1);
    debug_assert_eq!(out.len(), values.len());
    for v in out.iter_mut() {
        *v = f64::NAN;
    }
    let n = values.len();
    if n < period {
        return;
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
}

// 公开参考实现；当前 crate 内由 `overlap::wma` 提供带 `Result` 的封装，本函数主要供
// 文档链接与未来直接调用，故显式允许 dead_code（与下方 `wma_naive` 一致）。
// Public reference impl; the crate-internal `overlap::wma` wraps this in a `Result`, so this
// function is currently only a doc-link target / future direct API — allow dead_code
// (consistent with `wma_naive` below).
#[allow(dead_code)]
pub fn wma(values: &[f64], period: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; values.len()];
    wma_with_output(values, period, &mut out);
    out
}

/// 朴素 O(n·period) 加权移动平均，仅作为 [`wma`] 的单元测试对照（非热路径）。
///
/// Naïve O(n·period) weighted moving average — used only as the reference in unit tests for
/// [`wma`]; not on any hot path.
#[allow(dead_code)]
pub(crate) fn wma_naive(values: &[f64], period: usize) -> Vec<f64> {
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
