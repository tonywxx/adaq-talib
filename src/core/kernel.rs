//! TA 特定数值核（依赖 OHLC 三元组的指标内核）。
//!
//! TA-specific numeric kernels (indicator kernels that depend on the OHLC triple).

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
