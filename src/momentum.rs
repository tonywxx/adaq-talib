//! 动量指标（Momentum Indicators）。
//!
//! Momentum Indicators.
//!
//! 本模块全部函数的数值输出与 [TA-Lib](https://ta-lib.org) 0.7.1 逐项一致（浮点误差容限内，
//! 见 [`crate::utils`] 与 ADR 0005）。前导不稳定期以 [`f64::NAN`] 填充、等长返回（见 ADR 0007）。
//!
//! Every function in this module reproduces the numerical output of TA-Lib 0.7.1 (within the
//! float tolerance in ADR 0005). The leading unstable period is filled with [`f64::NAN`] and
//! returned at equal length (ADR 0007).

use crate::core::defaults::{
    APO_FAST, APO_SLOW, AROON_PERIOD, ATR_PERIOD, CMO_PERIOD, DX_PERIOD, IMI_PERIOD, MACD_FAST,
    MACD_SIGNAL, MACD_SLOW, MFI_PERIOD, MOM_PERIOD, RSI_PERIOD, STOCH_FAST_K, STOCH_SLOW_D,
    STOCH_SLOW_K, STOCHRSI_PERIOD, STOCHRSI_RSI_PERIOD, TRIX_PERIOD, ULTOSC_PERIOD1,
    ULTOSC_PERIOD2, ULTOSC_PERIOD3,
};
use crate::core::{ema, rolling_mean, rolling_mean_skip, rolling_sum};
use crate::error::{check_period, TaError};

/// 校验多数组长度一致（对应 TA-Lib 多输入函数的长度约束）。
/// Validate that several slices share the same length.
fn check_eq_len(lists: &[&[f64]], name: &str) -> Result<(), TaError> {
    let len = lists[0].len();
    for l in lists.iter().skip(1) {
        if l.len() != len {
            return Err(TaError::BadParam(format!(
                "{name}: all input arrays must have equal length"
            )));
        }
    }
    Ok(())
}

/// MACD 多输出结果（与 TA-Lib `TA_MACD` 三数组一一对应）。
///
/// MACD multi-output, mapping 1:1 to TA-Lib `TA_MACD`'s three output arrays.
///
/// - `macd`：快慢 EMA 之差（`macd = EMA(fast) - EMA(slow)`）。
/// - `signal`：对 `macd` 再做一次 EMA（`signal = EMA(macd, signal_period)`）。
/// - `hist`：柱（差值，`hist = macd - signal`）。
#[derive(Debug, Clone)]
pub struct Macd {
    /// 快慢 EMA 之差 / MACD line = `EMA(fast) - EMA(slow)`.
    pub macd: Vec<f64>,
    /// 信号线（MACD 的 EMA）/ Signal line (EMA of MACD).
    pub signal: Vec<f64>,
    /// 柱（MACD − 信号）/ Histogram (MACD − Signal).
    pub hist: Vec<f64>,
}

/// AROON 多输出结果（与 TA-Lib `TA_AROON` 两数组一一对应）。
///
/// AROON multi-output, mapping 1:1 to TA-Lib `TA_AROON`'s two output arrays.
#[derive(Debug, Clone)]
pub struct Aroon {
    /// 阿隆上线（创 N 周期内最高价距离）/ Aroon-Up (distance since N-period high).
    pub up: Vec<f64>,
    /// 阿隆下线（创 N 周期内最低价距离）/ Aroon-Down (distance since N-period low).
    pub down: Vec<f64>,
}

/// STOCH（慢速随机）多输出结果（与 TA-Lib `TA_STOCH` 两数组一一对应）。
///
/// STOCH (slow stochastic) multi-output, mapping 1:1 to TA-Lib `TA_STOCH`'s two arrays.
#[derive(Debug, Clone)]
pub struct Stoch {
    /// 慢速 %K（快速 %K 的简单移动平均）/ Slow %K (SMA of fast %K).
    pub slow_k: Vec<f64>,
    /// 慢速 %D（慢速 %K 的简单移动平均）/ Slow %D (SMA of slow %K).
    pub slow_d: Vec<f64>,
}

/// STOCHF（快速随机）多输出结果（与 TA-Lib `TA_STOCHF` 两数组一一对应）。
///
/// STOCHF (fast stochastic) multi-output, mapping 1:1 to TA-Lib `TA_STOCHF`'s two arrays.
#[derive(Debug, Clone)]
pub struct StochF {
    /// 快速 %K / Fast %K.
    pub fast_k: Vec<f64>,
    /// 快速 %D（快速 %K 的简单移动平均）/ Fast %D (SMA of fast %K).
    pub fast_d: Vec<f64>,
}

// ---------------------------------------------------------------------------
// 简单比率族 / Simple ratio family: MOM, ROC, ROCP, ROCR, ROCR100
// ---------------------------------------------------------------------------

/// 动量（Momentum，TA-Lib `TA_MOM`）。
///
/// `MOM[i] = inReal[i] - inReal[i - time_period]`，前导 `time_period` 个为 NaN。
/// 返回与输入等长、前导填 [`f64::NAN`] 的向量（见 ADR 0007）。
///
/// # 示例 / Example
/// ```
/// use adaq_talib::momentum::mom;
/// let p = [1.0, 2.0, 4.0, 8.0, 16.0];
/// let out = mom(&p, 2).unwrap();
/// assert!(out[0].is_nan() && out[1].is_nan());
/// // MOM[2] = p[2] - p[0] = 4 - 1 = 3
/// assert!((out[2] - 3.0).abs() < 1e-9);
/// ```
pub fn mom(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; values.len()];
    mom_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 动量，零拷贝写入 `out`（与 `values` 等长）。见 [`mom`]。
/// Momentum, written zero-copy into `out`. See [`mom`].
pub fn mom_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "mom_with_output: out length must equal values length".into(),
        ));
    }
    let n = values.len();
    for i in time_period..n {
        out[i] = values[i] - values[i - time_period];
    }
    Ok(())
}

/// `mom` 便捷版本，使用 TA-Lib 默认周期 10。/ `mom` with the TA-Lib default period (10).
pub fn mom_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    mom(values, MOM_PERIOD)
}

/// 比率变化族的内核（ROC / ROCP / ROCR / ROCR100 共用）。
/// Shared core for the rate-of-change family (ROC / ROCP / ROCR / ROCR100).
///
/// - `mode = 0`：`ROC = 100 * (cur - prev) / prev`
/// - `mode = 1`：`ROCP = (cur - prev) / prev`
/// - `mode = 2`：`ROCR = cur / prev`
/// - `mode = 3`：`ROCR100 = 100 * cur / prev`
/// 变动率族的共享计算内核（与 TA-Lib 0.7.1 `TA_ROC*` 逐项一致）。
///
/// 零拷贝写入调用方提供的 `out`（长度必须等于 `values.len()`），避免临时向量分配与
/// `copy_from_slice` 往返；并按 `mode` 将分支提出热循环外（P3-2，ADR 0010）。
/// `mode`：0=`ROC`（×100 差值比）、1=`ROCP`（差值比）、2=`ROCR`（比值）、3=`ROCR100`（×100 比值）。
/// 前导 `time_period` 个位置填 [`f64::NAN`]；除数为零时输出 `0.0`（与 TA-Lib 一致）。
fn rate_of_change(values: &[f64], time_period: usize, mode: u8, out: &mut [f64]) -> Result<(), TaError> {
    check_period(time_period)?;
    let n = values.len();
    if out.len() != n {
        return Err(TaError::BadParam(
            "rate_of_change: out length must equal values length".into(),
        ));
    }
    for v in out.iter_mut().take(time_period) {
        *v = f64::NAN;
    }
    let p = time_period;
    match mode {
        0 => {
            for i in p..n {
                let prev = values[i - p];
                out[i] = if prev == 0.0 { 0.0 } else { 100.0 * (values[i] - prev) / prev };
            }
        }
        1 => {
            for i in p..n {
                let prev = values[i - p];
                out[i] = if prev == 0.0 { 0.0 } else { (values[i] - prev) / prev };
            }
        }
        2 => {
            for i in p..n {
                let prev = values[i - p];
                out[i] = if prev == 0.0 { 0.0 } else { values[i] / prev };
            }
        }
        _ => {
            for i in p..n {
                let prev = values[i - p];
                out[i] = if prev == 0.0 { 0.0 } else { 100.0 * values[i] / prev };
            }
        }
    }
    Ok(())
}

/// 变动率（Rate of Change，`TA_MOM` 的 `ROC` 变体，TA-Lib `TA_ROC`）。
/// `ROC[i] = 100 * (inReal[i] - inReal[i-period]) / inReal[i-period]`。
pub fn roc(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; values.len()];
    roc_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 变动率，零拷贝写入 `out`（与 `values` 等长）。见 [`roc`]。
/// Rate of Change, written zero-copy into `out`. See [`roc`].
pub fn roc_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "roc_with_output: out length must equal values length".into(),
        ));
    }
    rate_of_change(values, time_period, 0, out)
}

/// `roc` 便捷版本，默认周期 10。/ `roc` with default period (10).
pub fn roc_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    roc(values, MOM_PERIOD)
}

/// 变动率（百分比，TA-Lib `TA_ROCP`）。`ROCP[i] = (cur - prev) / prev`。
pub fn rocp(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; values.len()];
    rocp_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 变动率（百分比），零拷贝写入 `out`（与 `values` 等长）。见 [`rocp`]。
/// Rate of Change (percent), written zero-copy into `out`. See [`rocp`].
pub fn rocp_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "rocp_with_output: out length must equal values length".into(),
        ));
    }
    rate_of_change(values, time_period, 1, out)
}

/// `rocp` 便捷版本，默认周期 10。/ `rocp` with default period (10).
pub fn rocp_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    rocp(values, MOM_PERIOD)
}

/// 变动率（比率，TA-Lib `TA_ROCR`）。`ROCR[i] = cur / prev`。
pub fn rocr(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; values.len()];
    rocr_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 变动率（比率），零拷贝写入 `out`（与 `values` 等长）。见 [`rocr`]。
/// Rate of Change (ratio), written zero-copy into `out`. See [`rocr`].
pub fn rocr_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "rocr_with_output: out length must equal values length".into(),
        ));
    }
    rate_of_change(values, time_period, 2, out)
}

/// `rocr` 便捷版本，默认周期 10。/ `rocr` with default period (10).
pub fn rocr_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    rocr(values, MOM_PERIOD)
}

/// 变动率（比率×100，TA-Lib `TA_ROCR100`）。`ROCR100[i] = 100 * cur / prev`。
pub fn rocr100(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; values.len()];
    rocr100_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 变动率（比率×100），零拷贝写入 `out`（与 `values` 等长）。见 [`rocr100`]。
/// Rate of Change (ratio × 100), written zero-copy into `out`. See [`rocr100`].
pub fn rocr100_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "rocr100_with_output: out length must equal values length".into(),
        ));
    }
    rate_of_change(values, time_period, 3, out)
}

/// `rocr100` 便捷版本，默认周期 10。/ `rocr100` with default period (10).
pub fn rocr100_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    rocr100(values, MOM_PERIOD)
}

// ---------------------------------------------------------------------------
// RSI
// ---------------------------------------------------------------------------

/// 相对强弱指数（Relative Strength Index，TA-Lib `TA_RSI`）。
///
/// Wilder 平滑：首值 = 前 `period` 个涨跌幅的均值（种子），其后按
/// `prev = (prev*(period-1) + x)/period` 递推。前导 `period` 个位置为 [`f64::NAN`]。
///
/// Wilder smoothing: the first value is the mean of the first `period` gains/losses (seed),
/// then `prev = (prev*(period-1) + x)/period`. The leading `period` positions are
/// [`f64::NAN`].
///
/// # 公式 / Formula
/// ```text
/// RS    = SMMA(gain) / SMMA(loss)
/// RSI   = 100 - 100 / (1 + RS)
/// ```
#[allow(clippy::needless_return)]
pub fn rsi(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; values.len()];
    rsi_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// 相对强弱指数，零拷贝写入 `out`（与 `values` 等长，前导 `period` 为 NaN）。见 [`rsi`]。
///
/// Relative Strength Index, written zero-copy into `out`. See [`rsi`]. Numerically identical
/// to [`rsi`].
pub fn rsi_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    let n = values.len();
    if out.len() != n {
        return Err(TaError::BadParam(
            "rsi_with_output: out length must equal values length".into(),
        ));
    }
    if n < time_period + 1 {
        return Ok(());
    }
    let p = time_period as f64;
    // 种子：前 `period` 个涨跌幅（bars 1..period）。
    // Seed over the first `period` deltas (bars 1..period).
    let mut gain = 0.0;
    let mut loss = 0.0;
    for i in 1..=time_period {
        let d = values[i] - values[i - 1];
        if d >= 0.0 {
            gain += d;
        } else {
            loss -= d;
        }
    }
    gain /= p;
    loss /= p;
    let rs = gain / loss;
    out[time_period] = 100.0 - 100.0 / (1.0 + rs);
    for i in (time_period + 1)..n {
        let d = values[i] - values[i - 1];
        let g = if d >= 0.0 { d } else { 0.0 };
        let l = if d >= 0.0 { 0.0 } else { -d };
        gain = (gain * (p - 1.0) + g) / p;
        loss = (loss * (p - 1.0) + l) / p;
        out[i] = if loss == 0.0 {
            100.0
        } else {
            100.0 - 100.0 / (1.0 + gain / loss)
        };
    }
    Ok(())
}

/// `rsi` 便捷版本，默认周期 14。/ `rsi` with default period (14).
pub fn rsi_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    rsi(values, RSI_PERIOD)
}

// ---------------------------------------------------------------------------
// MACD / MACDFIX / MACDEXT
// ---------------------------------------------------------------------------

/// 指数平滑异同移动平均（Moving Average Convergence/Divergence，TA-Lib `TA_MACD`）。
///
/// `macd = EMA(fast) - EMA(slow)`，`signal = EMA(macd, signal_period)`，
/// `hist = macd - signal`。三个数组等长、对齐到同一不稳定期的前导 NaN。
///
/// `macd = EMA(fast) - EMA(slow)`, `signal = EMA(macd, signal_period)`,
/// `hist = macd - signal`. All three arrays are equal-length and aligned to the same
/// leading `NaN` unstable period.
///
/// 注意：`MACDEXT` 在 TA-Lib 中可选 MA 类型（MAType）。本实现默认全部使用 EMA，
/// 与 `MACD` 数值一致；非 EMA 的 MAType 选择为后续待补能力（见 `docs/0.1.0-scope.md`）。
///
/// Note: TA-Lib's `MACDEXT` allows selecting the MA type (MAType). This implementation
/// defaults all three to EMA (numerically identical to `MACD`); non-EMA MAType selection
/// is a pending capability (see `docs/0.1.0-scope.md`).
pub fn macd(
    values: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> Result<Macd, TaError> {
    let n = values.len();
    let mut out = Macd {
        macd: vec![f64::NAN; n],
        signal: vec![f64::NAN; n],
        hist: vec![f64::NAN; n],
    };
    macd_with_output(values, fast_period, slow_period, signal_period, &mut out)?;
    Ok(out)
}

/// 指数平滑异同移动平均，零拷贝写入 `out`（三轨与 `values` 等长）。
/// 见 [`macd`]。
///
/// Moving Average Convergence/Divergence, written zero-copy into `out` (three equal-length
/// arrays). See [`macd`]. Numerically identical to [`macd`].
pub fn macd_with_output(
    values: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    out: &mut Macd,
) -> Result<(), TaError> {
    check_period(fast_period)?;
    check_period(slow_period)?;
    check_period(signal_period)?;
    if fast_period >= slow_period {
        return Err(TaError::BadParam(
            "fast_period must be < slow_period".into(),
        ));
    }
    let n = values.len();
    if out.macd.len() != n || out.signal.len() != n || out.hist.len() != n {
        return Err(TaError::BadParam(
            "macd_with_output: out vectors must have length == values length".into(),
        ));
    }
    let fast_k = 2.0 / (fast_period as f64 + 1.0);
    let slow_k = 2.0 / (slow_period as f64 + 1.0);
    let signal_k = 2.0 / (signal_period as f64 + 1.0);
    let lookback_signal = signal_period - 1;
    // TA-Lib 锁步实现：lookback = slow 的 EMA lookback + signal 的 EMA lookback。
    // TA-Lib lockstep: lookback = EMA-lookback(slow) + EMA-lookback(signal).
    let lookback_total = lookback_signal + (slow_period - 1); // = slow + signal - 2
    if n <= lookback_total {
        return Ok(());
    }
    // 单遍锁步：两条价格 EMA 独立递推，差值为 MACD 线，MACD 线立即喂入信号 EMA。
    // Single lockstep pass: both price EMAs advance independently, their difference is the
    // MACD line, and each MACD-line value is immediately fed into the signal EMA.
    let mut today = 0usize;
    let mut temp_real = 0.0_f64;
    // 快线种子是慢线窗口的尾部：先消费 `slow-fast` 个仅属慢线的柱，再以共享柱累加两者。
    // Fast seed is the tail of the slow window: consume the `slow-fast` slow-only bars, then
    // accumulate both over the shared bars.
    let mut i = slow_period - fast_period;
    while i > 0 {
        temp_real += values[today];
        today += 1;
        i -= 1;
    }
    let mut prev_fast = 0.0_f64;
    i = fast_period;
    while i > 0 {
        prev_fast += values[today];
        temp_real += values[today];
        today += 1;
        i -= 1;
    }
    let mut prev_slow = temp_real / slow_period as f64;
    prev_fast = prev_fast / fast_period as f64;
    // 推进两条 EMA 穿过各自不稳定期，直至首个 MACD 线柱。
    // Advance both EMAs through their unstable period up to the first MACD bar.
    while today <= lookback_total - lookback_signal {
        temp_real = values[today];
        today += 1;
        prev_fast = (temp_real - prev_fast) * fast_k + prev_fast;
        prev_slow = (temp_real - prev_slow) * slow_k + prev_slow;
    }
    let mut macd_value = prev_fast - prev_slow;
    // 信号线以首个 `signal_period` 个 MACD 值的简单均值作种子。
    // Seed the signal EMA with the simple average of the first `signal_period` MACD values.
    let mut prev_signal = 0.0_f64;
    prev_signal += macd_value;
    i = signal_period - 1;
    while i > 0 {
        temp_real = values[today];
        today += 1;
        prev_fast = (temp_real - prev_fast) * fast_k + prev_fast;
        prev_slow = (temp_real - prev_slow) * slow_k + prev_slow;
        macd_value = prev_fast - prev_slow;
        prev_signal += macd_value;
        i -= 1;
    }
    prev_signal = prev_signal / signal_period as f64;
    // 推进穿过信号线的不稳定期，直至首个输出柱。
    // Advance through the signal EMA unstable period up to the first output bar.
    while today <= lookback_total {
        temp_real = values[today];
        today += 1;
        prev_fast = (temp_real - prev_fast) * fast_k + prev_fast;
        prev_slow = (temp_real - prev_slow) * slow_k + prev_slow;
        macd_value = prev_fast - prev_slow;
        prev_signal = (macd_value - prev_signal) * signal_k + prev_signal;
    }
    // 稳定区：写入三个等长、对齐到同一前导 NaN 的输出。
    // Stable zone: write the three equal-length, leading-NaN-aligned outputs.
    let mut out_idx = lookback_total;
    out.macd[out_idx] = macd_value;
    out.signal[out_idx] = prev_signal;
    out.hist[out_idx] = macd_value - prev_signal;
    while today < n {
        temp_real = values[today];
        today += 1;
        prev_fast = (temp_real - prev_fast) * fast_k + prev_fast;
        prev_slow = (temp_real - prev_slow) * slow_k + prev_slow;
        macd_value = prev_fast - prev_slow;
        prev_signal = (macd_value - prev_signal) * signal_k + prev_signal;
        out_idx += 1;
        out.macd[out_idx] = macd_value;
        out.signal[out_idx] = prev_signal;
        out.hist[out_idx] = macd_value - prev_signal;
    }
    Ok(())
}

/// `macd` 便捷版本，使用 TA-Lib 默认 12 / 26 / 9。/ `macd` with defaults 12 / 26 / 9.
pub fn macd_default(values: &[f64]) -> Result<Macd, TaError> {
    macd(values, MACD_FAST, MACD_SLOW, MACD_SIGNAL)
}

/// 快捷 MACD（固定信号周期 9，TA-Lib `TA_MACDFIX`）。
/// Convenience MACD with a fixed signal period of 9 (TA-Lib `TA_MACDFIX`).
pub fn macd_fix(
    values: &[f64],
    fast_period: usize,
    slow_period: usize,
) -> Result<Macd, TaError> {
    let n = values.len();
    let mut out = Macd {
        macd: vec![f64::NAN; n],
        signal: vec![f64::NAN; n],
        hist: vec![f64::NAN; n],
    };
    macd_fix_with_output(values, fast_period, slow_period, &mut out)?;
    Ok(out)
}

/// 快捷 MACD（固定信号周期 9），零拷贝写入 `out`。见 [`macd_fix`]。
///
/// Convenience MACD (fixed signal period 9), written zero-copy into `out`. See [`macd_fix`].
/// Numerically identical to [`macd_fix`].
pub fn macd_fix_with_output(
    values: &[f64],
    fast_period: usize,
    slow_period: usize,
    out: &mut Macd,
) -> Result<(), TaError> {
    macd_with_output(values, fast_period, slow_period, MACD_SIGNAL, out)
}

/// `macd_fix` 便捷版本，默认 12 / 26。/ `macd_fix` with defaults 12 / 26.
pub fn macd_fix_default(values: &[f64]) -> Result<Macd, TaError> {
    macd(values, MACD_FAST, MACD_SLOW, MACD_SIGNAL)
}

/// 扩展 MACD（TA-Lib `TA_MACDEXT`，默认全 EMA）。参见 `macd` 的 MAType 说明。
/// Extended MACD (TA-Lib `TA_MACDEXT`, all-EMA default). See `macd` for the MAType note.
pub fn macd_ext(
    values: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> Result<Macd, TaError> {
    let n = values.len();
    let mut out = Macd {
        macd: vec![f64::NAN; n],
        signal: vec![f64::NAN; n],
        hist: vec![f64::NAN; n],
    };
    macd_ext_with_output(values, fast_period, slow_period, signal_period, &mut out)?;
    Ok(out)
}

/// 扩展 MACD，零拷贝写入 `out`。见 [`macd_ext`]。
///
/// Extended MACD, written zero-copy into `out`. See [`macd_ext`]. Numerically identical to
/// [`macd_ext`].
pub fn macd_ext_with_output(
    values: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    out: &mut Macd,
) -> Result<(), TaError> {
    macd_with_output(values, fast_period, slow_period, signal_period, out)
}

/// `macd_ext` 便捷版本，默认 12 / 26 / 9。/ `macd_ext` with defaults 12 / 26 / 9.
pub fn macd_ext_default(values: &[f64]) -> Result<Macd, TaError> {
    macd(values, MACD_FAST, MACD_SLOW, MACD_SIGNAL)
}

// ---------------------------------------------------------------------------
// APO / PPO
// ---------------------------------------------------------------------------

/// 绝对价格震荡器（Absolute Price Oscillator，TA-Lib `TA_APO`）。
/// `APO = EMA(fast) - EMA(slow)`，前导 `slow-1` 个为 NaN。
pub fn apo(values: &[f64], fast_period: usize, slow_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(fast_period)?;
    check_period(slow_period)?;
    let mut out = vec![f64::NAN; values.len()];
    apo_with_output(values, fast_period, slow_period, &mut out)?;
    Ok(out)
}

/// 绝对价格震荡器，零拷贝写入 `out`（与 `values` 等长，前导 `slow-1` 为 NaN）。见 [`apo`]。
///
/// Absolute Price Oscillator, written zero-copy into `out`. See [`apo`]. Numerically identical
/// to [`apo`].
pub fn apo_with_output(
    values: &[f64],
    fast_period: usize,
    slow_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(fast_period)?;
    check_period(slow_period)?;
    if fast_period >= slow_period {
        return Err(TaError::BadParam("fast_period must be < slow_period".into()));
    }
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "apo_with_output: out length must equal values length".into(),
        ));
    }
    let ef = ema(values, fast_period);
    let es = ema(values, slow_period);
    let n = values.len();
    for i in 0..n {
        if !ef[i].is_nan() && !es[i].is_nan() {
            out[i] = ef[i] - es[i];
        }
    }
    Ok(())
}

/// `apo` 便捷版本，默认 12 / 26。/ `apo` with defaults 12 / 26.
pub fn apo_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    apo(values, APO_FAST, APO_SLOW)
}

/// 百分比价格震荡器（Percentage Price Oscillator，TA-Lib `TA_PPO`）。
/// `PPO = 100 * (EMA(fast) - EMA(slow)) / EMA(slow)`。
pub fn ppo(values: &[f64], fast_period: usize, slow_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(fast_period)?;
    check_period(slow_period)?;
    let mut out = vec![f64::NAN; values.len()];
    ppo_with_output(values, fast_period, slow_period, &mut out)?;
    Ok(out)
}

/// 百分比价格震荡器，零拷贝写入 `out`（与 `values` 等长，前导 `slow-1` 为 NaN）。见 [`ppo`]。
///
/// Percentage Price Oscillator, written zero-copy into `out`. See [`ppo`]. Numerically
/// identical to [`ppo`].
pub fn ppo_with_output(
    values: &[f64],
    fast_period: usize,
    slow_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(fast_period)?;
    check_period(slow_period)?;
    if fast_period >= slow_period {
        return Err(TaError::BadParam("fast_period must be < slow_period".into()));
    }
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "ppo_with_output: out length must equal values length".into(),
        ));
    }
    let ef = ema(values, fast_period);
    let es = ema(values, slow_period);
    let n = values.len();
    for i in 0..n {
        if !ef[i].is_nan() && !es[i].is_nan() && es[i] != 0.0 {
            out[i] = 100.0 * (ef[i] - es[i]) / es[i];
        }
    }
    Ok(())
}

/// `ppo` 便捷版本，默认 12 / 26。/ `ppo` with defaults 12 / 26.
pub fn ppo_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    ppo(values, APO_FAST, APO_SLOW)
}

// ---------------------------------------------------------------------------
// CMO / CCI
// ---------------------------------------------------------------------------

/// Chande 动量震荡器（Chande Momentum Oscillator，TA-Lib `TA_CMO`）。
///
/// TA-Lib 算法（非朴素滚动和）：先用前 `period` 个涨跌幅做种子（增益/损失各自取均值），
/// 其后按 Wilder 方式递推 `prev = (prev*(period-1) + x)/period`——但涨跌互不影响对侧累加器：
/// 涨柱会同时按 `delta` 增大增益、减小损失；跌柱反之。`CMO = 100*(gain-loss)/(gain+loss)`。
/// 前导 `period` 个为 [`f64::NAN`]。
///
/// TA-Lib algorithm (not a naïve rolling sum): seed from the first `period` deltas (gain/loss
/// each averaged), then Wilder recursion `prev = (prev*(period-1) + x)/period` — but a gain bar
/// raises the gain accumulator *and lowers* the loss accumulator (and vice versa).
/// `CMO = 100*(gain-loss)/(gain+loss)`. The leading `period` positions are [`f64::NAN`].
pub fn cmo(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let mut out = vec![f64::NAN; values.len()];
    cmo_with_output(values, time_period, &mut out)?;
    Ok(out)
}

/// Chande 动量震荡器，零拷贝写入 `out`（与 `values` 等长，前导 `period` 为 NaN）。见 [`cmo`]。
///
/// Chande Momentum Oscillator, written zero-copy into `out`. See [`cmo`]. Numerically
/// identical to [`cmo`].
pub fn cmo_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    let n = values.len();
    if out.len() != n {
        return Err(TaError::BadParam(
            "cmo_with_output: out length must equal values length".into(),
        ));
    }
    if n <= time_period {
        return Ok(());
    }
    let p = time_period as f64;
    // 种子：前 `period` 个涨跌幅（bars 1..period），增益/损失各自求和后取均值。
    // Seed over the first `period` deltas (bars 1..period), gain/loss summed then averaged.
    let mut prev_value = values[0];
    let mut prev_gain = 0.0_f64;
    let mut prev_loss = 0.0_f64;
    for t in 1..=time_period {
        let tv = values[t];
        let delta = tv - prev_value;
        prev_value = tv;
        if delta < 0.0 {
            prev_loss -= delta;
        } else {
            prev_gain += delta;
        }
    }
    prev_gain /= p;
    prev_loss /= p;
    // 首个有效值（索引 `period`）直接由种子得出，无需新涨跌幅。
    // First valid value (index `period`) from the seed alone.
    let denom0 = prev_gain + prev_loss;
    out[time_period] = if denom0 == 0.0 {
        0.0
    } else {
        100.0 * (prev_gain - prev_loss) / denom0
    };
    // 后续 Wilder 递推并输出。/ Subsequent Wilder recursion and output.
    for t in (time_period + 1)..n {
        let tv = values[t];
        let delta = tv - prev_value;
        prev_value = tv;
        prev_loss *= p - 1.0;
        prev_gain *= p - 1.0;
        if delta < 0.0 {
            prev_loss -= delta;
        } else {
            prev_gain += delta;
        }
        prev_loss /= p;
        prev_gain /= p;
        let denom = prev_gain + prev_loss;
        out[t] = if denom == 0.0 {
            0.0
        } else {
            100.0 * (prev_gain - prev_loss) / denom
        };
    }
    Ok(())
}

/// `cmo` 便捷版本，默认周期 14。/ `cmo` with default period (14).
pub fn cmo_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    cmo(values, CMO_PERIOD)
}

/// 商品通道指数（Commodity Channel Index，TA-Lib `TA_CCI`）。
///
/// `CCI = (TP - SMA(TP)) / (0.015 * MeanDev(TP))`，`TP = (H+L+C)/3`，
/// 均值偏差为窗口内 `|TP - SMA(TP)|` 的均值。前导 `period-1` 个为 [`f64::NAN`]。
pub fn cci(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: usize,
) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "cci")?;
    let mut out = vec![f64::NAN; close.len()];
    cci_with_output(high, low, close, time_period, &mut out)?;
    Ok(out)
}

/// 商品通道指数，零拷贝写入 `out`（与 `close` 等长，前导 `period-1` 为 NaN）。见 [`cci`]。
///
/// Commodity Channel Index, written zero-copy into `out`. See [`cci`]. Numerically identical
/// to [`cci`].
pub fn cci_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "cci")?;
    if out.len() != close.len() {
        return Err(TaError::BadParam(
            "cci_with_output: out length must equal close length".into(),
        ));
    }
    let n = close.len();
    let tp: Vec<f64> = (0..n).map(|i| (high[i] + low[i] + close[i]) / 3.0).collect();
    let sma_tp = rolling_mean(&tp, time_period);
    for i in (time_period - 1)..n {
        let mean = sma_tp[i];
        let mut dev = 0.0;
        for j in 0..time_period {
            dev += (tp[i - j] - mean).abs();
        }
        dev /= time_period as f64;
        out[i] = if dev == 0.0 { 0.0 } else { (tp[i] - mean) / (0.015 * dev) };
    }
    Ok(())
}

/// `cci` 便捷版本，默认周期 20。/ `cci` with default period (20).
pub fn cci_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    cci(high, low, close, 20)
}

// ---------------------------------------------------------------------------
// MFI / WILLR / BOP
// ---------------------------------------------------------------------------

/// 资金流量指数（Money Flow Index，TA-Lib `TA_MFI`），含成交量。
///
/// `MFI = 100 - 100 / (1 + posFlow/negFlow)`，典型价 `TP=(H+L+C)/3`，
/// 正/负资金流为窗口内 `TP*volume` 按涨跌方向求和。前导 `period` 个为 [`f64::NAN`]。
pub fn mfi(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
    time_period: usize,
) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close, volume], "mfi")?;
    let mut out = vec![f64::NAN; close.len()];
    mfi_with_output(high, low, close, volume, time_period, &mut out)?;
    Ok(out)
}

/// 资金流量指数，零拷贝写入 `out`（与 `close` 等长，前导 `period` 为 NaN）。见 [`mfi`]。
///
/// Money Flow Index, written zero-copy into `out`. See [`mfi`]. Numerically identical to
/// [`mfi`].
pub fn mfi_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close, volume], "mfi")?;
    if out.len() != close.len() {
        return Err(TaError::BadParam(
            "mfi_with_output: out length must equal close length".into(),
        ));
    }
    let n = close.len();
    if n == 0 {
        return Ok(());
    }
    // 单遍滑动窗口（O(n)）：维护正负资金流的滚动和，消除原实现的多趟扫描与多次分配
    // （tp/mf/pos/neg/sp/sn 共 6 次分配 + 5 趟），与原逐窗口求和在数值上逐项一致
    // （黄金向量 1:1）。TA-Lib C 的 MFI 同为单遍运行和。
    // Single-pass sliding window (O(n)): maintain running positive/negative money-flow
    // sums, eliminating the original multi-pass scan and extra allocations. Numerically
    // 1:1 with the per-window sum (golden vector). TA-Lib's C MFI is also single-pass.
    let p = time_period;
    let mut pos_ring = vec![0.0_f64; p]; // 窗口 pos 环形缓冲，供滑窗剔除左端
    let mut neg_ring = vec![0.0_f64; p]; // 窗口 neg 环形缓冲
    let mut pos_sum = 0.0_f64;
    let mut neg_sum = 0.0_f64;
    let mut idx = 0_usize;
    let mut tp_prev = (high[0] + low[0] + close[0]) / 3.0;
    for i in 1..n {
        let tp = (high[i] + low[i] + close[i]) / 3.0;
        let mf = tp * volume[i];
        let (pos_i, neg_i) = if tp > tp_prev {
            (mf, 0.0)
        } else if tp < tp_prev {
            (0.0, mf)
        } else {
            (0.0, 0.0)
        };
        // 窗口已满（i >= p）：剔除左端离开窗口的元素（bar i-p，pos[0]=neg[0]=0 由未写槽位表示）。
        // Window full (i >= p): evict the left element leaving the window (bar i-p;
        // pos[0]=neg[0]=0 is represented by the never-written slot).
        if i >= p {
            pos_sum -= pos_ring[idx];
            neg_sum -= neg_ring[idx];
        }
        pos_sum += pos_i;
        neg_sum += neg_i;
        pos_ring[idx] = pos_i;
        neg_ring[idx] = neg_i;
        // 首个有效输出在 i = p（窗口 [1..p]，含 p 个资金流），与 TA-Lib lookback = period 一致。
        // First valid output at i = p (window [1..p], p money flows), matching TA-Lib's
        // lookback = period.
        if i >= p {
            out[i] = if neg_sum == 0.0 {
                100.0
            } else {
                100.0 - 100.0 / (1.0 + pos_sum / neg_sum)
            };
        }
        tp_prev = tp;
        idx = (idx + 1) % p;
    }
    Ok(())
}

/// `mfi` 便捷版本，默认周期 14。/ `mfi` with default period (14).
pub fn mfi_default(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<Vec<f64>, TaError> {
    mfi(high, low, close, volume, MFI_PERIOD)
}

/// Williams' %R（TA-Lib `TA_WILLR`）。
/// `WILLR = -100 * (HH - close) / (HH - LL)`，HH/LL 为窗口内最高/最低。
/// 前导 `period-1` 个为 [`f64::NAN`]。
pub fn willr(high: &[f64], low: &[f64], close: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "willr")?;
    let n = close.len();
    let mut out = vec![f64::NAN; n];
    willr_with_output(high, low, close, time_period, &mut out)?;
    Ok(out)
}

/// Williams' %R，零拷贝写入 `out`（与 `close` 等长）。见 [`willr`]。
///
/// Williams' %R, written zero-copy into `out`. See [`willr`]. Uses the already 1:1-verified
/// monotonic-queue [`crate::core::rolling_max`] / [`crate::core::rolling_min`] (O(n))
/// instead of the per-window O(n·period) scan, with the same rightmost tie-break.
pub fn willr_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "willr")?;
    if out.len() != close.len() {
        return Err(TaError::BadParam(
            "willr_with_output: out length must equal close length".into(),
        ));
    }
    // 数据量足够且启用 `parallel` feature 时走多核分块；内核与串行逐字节一致，输出 1:1。
    // Under the `parallel` feature with enough data, use multi-core chunking; the kernel is
    // byte-identical to the serial path, so output is 1:1.
    #[cfg(feature = "parallel")]
    {
        if close.len() >= 8192 {
            return willr_parallel_with_output(high, low, close, time_period, out);
        }
    }
    willr_serial_with_output(high, low, close, time_period, out)
}

/// Williams' %R 串行内核（与 TA-Lib `TA_WILLR` 逐项 1:1）。见 [`willr_with_output`]。
/// Serial kernel for Williams' %R (1:1 with TA-Lib `TA_WILLR`). See [`willr_with_output`].
fn willr_serial_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    let n = close.len();
    if n < time_period {
        for v in out.iter_mut() {
            *v = f64::NAN;
        }
        return Ok(());
    }
    // 单遍融合：用两个单调队列（high 的最大值、low 的最小值）在一次遍历中同时求得
    // HHV(high) 与 LLV(low) 并直接写出 WILLR，消除原先 `rolling_max` + `rolling_min` 的两次
    // 独立扫描、两次 `Vec` 分配与合并趟（P3-3，ADR 0005 零偏差：极值取值与分别调用一致）。
    let mut max_dq = crate::core::MonoQueue::with_capacity(time_period);
    let mut min_dq = crate::core::MonoQueue::with_capacity(time_period);
    for i in 0..n {
        while !max_dq.is_empty() && max_dq.front() + time_period <= i {
            max_dq.pop_front();
        }
        while !min_dq.is_empty() && min_dq.front() + time_period <= i {
            min_dq.pop_front();
        }
        while !max_dq.is_empty() && high[max_dq.back()] <= high[i] {
            max_dq.pop_back();
        }
        while !min_dq.is_empty() && low[min_dq.back()] >= low[i] {
            min_dq.pop_back();
        }
        max_dq.push_back(i);
        min_dq.push_back(i);
        if i >= time_period - 1 {
            let hh = high[max_dq.front()];
            let ll = low[min_dq.front()];
            out[i] = if hh == ll {
                0.0
            } else {
                -100.0 * (hh - close[i]) / (hh - ll)
            };
        } else {
            out[i] = f64::NAN;
        }
    }
    Ok(())
}

/// Williams' %R 串行版本（feature 无关，供并行对照测试作黄金参考）。见 [`willr`]。
/// Serial Williams' %R (feature-agnostic; golden reference for the parallel equality test). See [`willr`].
pub fn willr_serial(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: usize,
) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "willr")?;
    let mut out = vec![f64::NAN; close.len()];
    willr_serial_with_output(high, low, close, time_period, &mut out)?;
    Ok(out)
}

/// Williams' %R 多核并行版本（需 `parallel` feature）。复用 [`willr_serial_with_output`] 的单遍
/// 双队列内核，以 `period-1` 前导重叠播种各分块的单调双端队列，输出与串行逐项 1:1。
/// Multi-core parallel Williams' %R (requires the `parallel` feature). Reuses the single-pass
/// dual-deque kernel of [`willr_serial_with_output`] with `period-1` leading overlap; output is
/// 1:1 with the serial path.
#[cfg(feature = "parallel")]
pub fn willr_parallel(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: usize,
) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "willr")?;
    let mut out = vec![f64::NAN; close.len()];
    willr_parallel_with_output(high, low, close, time_period, &mut out)?;
    Ok(out)
}

/// Williams' %R 并行内核（零拷贝写入 `out`）。见 [`willr_parallel`]。
/// Parallel kernel for Williams' %R (zero-copy into `out`). See [`willr_parallel`].
#[cfg(feature = "parallel")]
fn willr_parallel_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "willr")?;
    if out.len() != close.len() {
        return Err(TaError::BadParam(
            "willr_parallel_with_output: out length must equal close length".into(),
        ));
    }
    let p = time_period;
    crate::parallel::parallel_index_map(close.len(), p - 1, out, |start, end| {
        let mut local = vec![f64::NAN; end - start];
        let _ = willr_serial_with_output(
            &high[start..end],
            &low[start..end],
            &close[start..end],
            p,
            &mut local,
        );
        local
    });
    Ok(())
}

/// `willr` 便捷版本，默认周期 14。/ `willr` with default period (14).
pub fn willr_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    willr(high, low, close, ATR_PERIOD)
}

/// 均势（Balance Of Power，TA-Lib `TA_BOP`）。`BOP = (close - open) / (high - low)`。
/// 无滞后（lookback 0）；若 `high == low` 返回 0.0。
pub fn bop(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[open, high, low, close], "bop")?;
    let mut out = vec![0.0_f64; close.len()];
    bop_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// 均势，零拷贝写入 `out`（与 `close` 等长，无前导 NaN）。见 [`bop`]。
///
/// Balance Of Power, written zero-copy into `out`. See [`bop`]. Numerically identical to
/// [`bop`].
pub fn bop_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    check_eq_len(&[open, high, low, close], "bop")?;
    if out.len() != close.len() {
        return Err(TaError::BadParam(
            "bop_with_output: out length must equal close length".into(),
        ));
    }
    let n = close.len();
    for i in 0..n {
        let range = high[i] - low[i];
        out[i] = if range == 0.0 {
            0.0
        } else {
            (close[i] - open[i]) / range
        };
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// ULTOSC
// ---------------------------------------------------------------------------

/// 终极震荡器（Ultimate Oscillator，TA-Lib `TA_ULTOSC`）。
///
/// `ULTOSC = 100 * (4*avg1 + 2*avg2 + avg3) / 7`，
/// `avgX = sum(BP, periodX) / sum(TR, periodX)`，`BP = close - min(low, prevClose)`，
/// `TR = max(high,prevClose) - min(low,prevClose)`。前导 `p3` 个为 [`f64::NAN`]
/// （TA-Lib 取最长周期 `period3` 为 lookback，首个有效值位于索引 `period3`）。
pub fn ultosc(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period1: usize,
    period2: usize,
    period3: usize,
) -> Result<Vec<f64>, TaError> {
    check_period(period1)?;
    check_period(period2)?;
    check_period(period3)?;
    check_eq_len(&[high, low, close], "ultosc")?;
    let mut out = vec![f64::NAN; close.len()];
    ultosc_with_output(high, low, close, period1, period2, period3, &mut out)?;
    Ok(out)
}

/// 终极震荡器，零拷贝写入 `out`（与 `close` 等长，前导 `period3` 为 NaN）。见 [`ultosc`]。
///
/// Ultimate Oscillator, written zero-copy into `out`. See [`ultosc`]. Numerically identical
/// to [`ultosc`].
pub fn ultosc_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period1: usize,
    period2: usize,
    period3: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(period1)?;
    check_period(period2)?;
    check_period(period3)?;
    check_eq_len(&[high, low, close], "ultosc")?;
    if out.len() != close.len() {
        return Err(TaError::BadParam(
            "ultosc_with_output: out length must equal close length".into(),
        ));
    }
    let n = close.len();
    if n == 0 {
        return Ok(());
    }
    // 单遍融合：bp/tr 的三档窗口求和（period1/2/3）在一次前向扫描内完成，仅用
    // O(period3) 的环形缓冲保存滞后元素，避免 6 次独立 `rolling_sum` 的全遍历与分配
    // （P3-2 性能优化）。与分段实现逐项一致（ADR 0005）：每个窗口的滑窗递推
    // `sum += x - x[lag]` 与 `rolling_sum` 逐字节相同。
    let cap = period3 + 1;
    let mut ring_bp = vec![0.0_f64; cap];
    let mut ring_tr = vec![0.0_f64; cap];
    let mut w = 0usize;
    let mut sbp1 = 0.0_f64;
    let mut sbp2 = 0.0_f64;
    let mut sbp3 = 0.0_f64;
    let mut str1 = 0.0_f64;
    let mut str2 = 0.0_f64;
    let mut str3 = 0.0_f64;
    for i in 0..n {
        let prev = if i > 0 { close[i - 1] } else { close[i] };
        let bpi = close[i] - low[i].min(prev);
        let tri = high[i].max(prev) - low[i].min(prev);
        ring_bp[w] = bpi;
        ring_tr[w] = tri;
        // 滑窗累加（与 `rolling_sum` 逐字节一致：先加当前、滞后减去窗口外元素）。
        sbp1 += bpi;
        sbp2 += bpi;
        sbp3 += bpi;
        str1 += tri;
        str2 += tri;
        str3 += tri;
        if i >= period1 {
            let k = (w + cap - period1) % cap;
            sbp1 -= ring_bp[k];
            str1 -= ring_tr[k];
        }
        if i >= period2 {
            let k = (w + cap - period2) % cap;
            sbp2 -= ring_bp[k];
            str2 -= ring_tr[k];
        }
        if i >= period3 {
            let k = (w + cap - period3) % cap;
            sbp3 -= ring_bp[k];
            str3 -= ring_tr[k];
            // TA-Lib 以最长周期 `period3` 为 lookback（首个有效值位于 index `period3`）。
            // TA-Lib uses the longest period `period3` as the lookback (first valid at index `period3`).
            if str1 == 0.0 || str2 == 0.0 || str3 == 0.0 {
                out[i] = 0.0;
            } else {
                let a1 = sbp1 / str1;
                let a2 = sbp2 / str2;
                let a3 = sbp3 / str3;
                out[i] = 100.0 * (4.0 * a1 + 2.0 * a2 + a3) / 7.0;
            }
        }
        w = (w + 1) % cap;
    }
    Ok(())
}

/// `ultosc` 便捷版本，默认 7 / 14 / 28。/ `ultosc` with defaults 7 / 14 / 28.
pub fn ultosc_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    ultosc(
        high,
        low,
        close,
        ULTOSC_PERIOD1,
        ULTOSC_PERIOD2,
        ULTOSC_PERIOD3,
    )
}

// ---------------------------------------------------------------------------
// 方向性运动族 / Directional Movement: PLUS_DM, MINUS_DM, PLUS_DI, MINUS_DI, ADX, ADXR
// ---------------------------------------------------------------------------

/// 方向性运动族的共享计算（与 TA-Lib 0.7.1 逐项一致）。
/// Shared directional-movement computation, 1:1 with TA-Lib 0.7.1.
///
/// 返回六个等长向量（前导不稳定期填 [`f64::NAN`]，对齐到 TA-Lib 各自 lookback）：
/// - `pdm`/`mdm`：`+DM`/`-DM`（Wilder 平滑，TA_PLUS_DM/MINUS_DM），前导 `period-1` 为 NaN。
/// - `pdi`/`mdi`：`+DI`/`-DI`（`= 100*DM/TR`），前导 `period` 为 NaN。
/// - `adx`：`ADX`（DX 的 Wilder 平滑），前导 `2*period-1` 为 NaN。
/// - `adxr`：`ADXR`（`(ADX[i]+ADX[i-(period-1)])/2`），前导 `3*period-2` 为 NaN。
///
/// Returns six equal-length vectors (leading unstable period filled with [`f64::NAN`], aligned
/// to each TA-Lib lookback): `pdm`/`mdm` (Wilder-smoothed ±DM), `pdi`/`mdi` (= 100·DM/TR),
/// `adx` (Wilder-smoothed DX), `adxr` ((ADX[i]+ADX[i-(period-1)])/2).
#[allow(clippy::too_many_arguments)]
/// Wilder-smoothed +DM / -DM / TR pass — the shared directional-movement kernel.
///
/// Seed = sum of the first `period-1` DM1/TR1 values (placed at index `period-1`), then
/// Wilder recursion. Byte-for-byte identical to the historical `directional` kernel (P3-2).
#[allow(clippy::too_many_arguments)]
fn dm_tr(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = high.len();
    let mut pdm = vec![f64::NAN; n];
    let mut mdm = vec![f64::NAN; n];
    let mut tr = vec![f64::NAN; n];
    if n < period + 1 {
        return (pdm, mdm, tr);
    }
    let p = period as f64;
    let mut today = 0usize;
    let mut prev_high = high[0];
    let mut prev_low = low[0];
    let mut prev_close = close[0];
    let mut prev_plus_dm = 0.0_f64;
    let mut prev_minus_dm = 0.0_f64;
    let mut prev_tr = 0.0_f64;
    let mut i = period - 1;
    while i > 0 {
        today += 1;
        let tp = high[today];
        let diff_p = tp - prev_high;
        prev_high = tp;
        let tl = low[today];
        let diff_m = prev_low - tl;
        prev_low = tl;
        if diff_m > 0.0 && diff_p < diff_m {
            prev_minus_dm += diff_m;
        } else if diff_p > 0.0 && diff_p > diff_m {
            prev_plus_dm += diff_p;
        }
        let mut range = prev_high - prev_low;
        let mut tmp = (prev_high - prev_close).abs();
        if tmp > range {
            range = tmp;
        }
        tmp = (prev_low - prev_close).abs();
        if tmp > range {
            range = tmp;
        }
        prev_tr += range;
        prev_close = close[today];
        i -= 1;
    }
    pdm[period - 1] = prev_plus_dm;
    mdm[period - 1] = prev_minus_dm;
    tr[period - 1] = prev_tr;
    while today < n - 1 {
        today += 1;
        let tp = high[today];
        let diff_p = tp - prev_high;
        prev_high = tp;
        let tl = low[today];
        let diff_m = prev_low - tl;
        prev_low = tl;
        prev_minus_dm -= prev_minus_dm / p;
        prev_plus_dm -= prev_plus_dm / p;
        if diff_m > 0.0 && diff_p < diff_m {
            prev_minus_dm += diff_m;
        } else if diff_p > 0.0 && diff_p > diff_m {
            prev_plus_dm += diff_p;
        }
        let mut range = prev_high - prev_low;
        let mut tmp = (prev_high - prev_close).abs();
        if tmp > range {
            range = tmp;
        }
        tmp = (prev_low - prev_close).abs();
        if tmp > range {
            range = tmp;
        }
        prev_tr = prev_tr - prev_tr / p + range;
        prev_close = close[today];
        pdm[today] = prev_plus_dm;
        mdm[today] = prev_minus_dm;
        tr[today] = prev_tr;
    }
    (pdm, mdm, tr)
}

/// +DI / -DI from Wilder-smoothed DM/TR: `= 100 * DM / TR`. First valid at `period`.
fn di_from_dm_tr(pdm: &[f64], mdm: &[f64], tr: &[f64], period: usize) -> (Vec<f64>, Vec<f64>) {
    let n = tr.len();
    let mut pdi = vec![f64::NAN; n];
    let mut mdi = vec![f64::NAN; n];
    for i in period..n {
        if !tr[i].is_nan() {
            if tr[i] == 0.0 {
                pdi[i] = 0.0;
                mdi[i] = 0.0;
            } else {
                pdi[i] = 100.0 * pdm[i] / tr[i];
                mdi[i] = 100.0 * mdm[i] / tr[i];
            }
        }
    }
    (pdi, mdi)
}

/// 单遍融合核：Wilder ±DM/TR → +DI/-DI → 纯 DX（`denom==0` 取 0）→ ADX（Wilder 种子缓冲）
/// → ADXR（环形缓冲取 `adx[i-(period-1)]`）。与分段实现 [`dm_tr`] + [`di_from_dm_tr`] +
/// `adx_adxr_from_di` 逐项一致（ADR 0005），但只走一遍前向扫描、仅写 `adx`/`adxr` 两份输出，
/// 把内存流量从 5 份降到 2 份，消除 `adx` 调用里对 `adxr` 的冗余计算（P3-2 性能优化）。
///
/// Single forward pass: Wilder ±DM/TR → +DI/-DI → pure DX (0 on zero denom) → ADX
/// (Wilder seed buffer) → ADXR (ring buffer for `adx[i-(period-1)]`). Bit-for-bit equal to
/// the staged `dm_tr` + `di_from_dm_tr` + `adx_adxr_from_di` (ADR 0005), but a single scan
/// that writes only `adx`/`adxr`. `adx` no longer pays for the `adxr` pass (P3-2).
fn adx_adxr_fused(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> (Vec<f64>, Vec<f64>) {
    let n = high.len();
    let mut adx = vec![f64::NAN; n];
    let mut adxr = vec![f64::NAN; n];
    let dx_seed_end = 2 * period - 1;
    // ADX 第一个有效点落在 `2*period-1`；需要 `n > dx_seed_end`。
    // The first valid ADX lands at `2*period-1`; require `n > dx_seed_end`.
    if n < 2 * period {
        return (adx, adxr);
    }
    let p = period as f64;
    let mut today = 0usize;
    let mut prev_high = high[0];
    let mut prev_low = low[0];
    let mut prev_close = close[0];
    let mut prev_plus_dm = 0.0_f64;
    let mut prev_minus_dm = 0.0_f64;
    let mut prev_tr = 0.0_f64;
    // 种子累积（前 `period-1` 根 K 的裸 ±DM/TR 求和），与 [`dm_tr`] 逐字节一致。
    // Seed accumulation (raw ±DM/TR of the first `period-1` bars), byte-identical to `dm_tr`.
    let mut i = period - 1;
    while i > 0 {
        today += 1;
        let tp = high[today];
        let diff_p = tp - prev_high;
        prev_high = tp;
        let tl = low[today];
        let diff_m = prev_low - tl;
        prev_low = tl;
        if diff_m > 0.0 && diff_p < diff_m {
            prev_minus_dm += diff_m;
        } else if diff_p > 0.0 && diff_p > diff_m {
            prev_plus_dm += diff_p;
        }
        let mut range = prev_high - prev_low;
        let mut tmp = (prev_high - prev_close).abs();
        if tmp > range {
            range = tmp;
        }
        tmp = (prev_low - prev_close).abs();
        if tmp > range {
            range = tmp;
        }
        prev_tr += range;
        prev_close = close[today];
        i -= 1;
    }
    let mut sum_dx = 0.0_f64;
    let mut seeded_adx = false;
    let mut prev_adx = 0.0_f64;
    // 仅保留最近 `period` 个 ADX 值，用于 ADXR = (ADX[i] + ADX[i-(period-1)])/2。
    // Keep only the last `period` ADX values for ADXR = (ADX[i] + ADX[i-(period-1)])/2.
    let mut adx_ring: std::collections::VecDeque<f64> = std::collections::VecDeque::with_capacity(period);
    while today < n - 1 {
        today += 1;
        let tp = high[today];
        let diff_p = tp - prev_high;
        prev_high = tp;
        let tl = low[today];
        let diff_m = prev_low - tl;
        prev_low = tl;
        prev_minus_dm -= prev_minus_dm / p;
        prev_plus_dm -= prev_plus_dm / p;
        if diff_m > 0.0 && diff_p < diff_m {
            prev_minus_dm += diff_m;
        } else if diff_p > 0.0 && diff_p > diff_m {
            prev_plus_dm += diff_p;
        }
        let mut range = prev_high - prev_low;
        let mut tmp = (prev_high - prev_close).abs();
        if tmp > range {
            range = tmp;
        }
        tmp = (prev_low - prev_close).abs();
        if tmp > range {
            range = tmp;
        }
        prev_tr = prev_tr - prev_tr / p + range;
        prev_close = close[today];
        // +DI / -DI（与 [`di_from_dm_tr`] 一致）。+DI / -DI (matches `di_from_dm_tr`).
        let (pdi, mdi) = if prev_tr == 0.0 {
            (0.0, 0.0)
        } else {
            (
                100.0 * prev_plus_dm / prev_tr,
                100.0 * prev_minus_dm / prev_tr,
            )
        };
        // 纯 DX：与 `adx_adxr_from_di` 相同，`denom==0` 取 0（无前向填充）。
        // Pure DX: same as `adx_adxr_from_di`, 0 on zero denom (no carry-forward).
        let denom = pdi + mdi;
        let dx = if denom != 0.0 {
            100.0 * (pdi - mdi).abs() / denom
        } else {
            0.0
        };
        // ADX：前 `period` 个 DX 求均值作种子，之后 Wilder 递推（与 `adx_adxr_from_di` 一致）。
        // ADX: mean of the first `period` DX as seed, then Wilder recursion (matches the staged impl).
        if !seeded_adx {
            sum_dx += dx;
            if today >= dx_seed_end {
                prev_adx = sum_dx / p;
                adx[today] = prev_adx;
                adx_ring.push_back(prev_adx);
                seeded_adx = true;
            }
        } else {
            prev_adx = (prev_adx * (p - 1.0) + dx) / p;
            adx[today] = prev_adx;
            adx_ring.push_back(prev_adx);
            if adx_ring.len() > period {
                adx_ring.pop_front();
            }
        }
        // ADXR = (ADX[i] + ADX[i-(period-1)])/2；环形缓冲队首即 ADX[i-(period-1)]。
        // ADXR = (ADX[i] + ADX[i-(period-1)])/2; the ring front is exactly ADX[i-(period-1)].
        let adxr_start = 3 * period - 2;
        if today >= adxr_start && adx_ring.len() == period {
            let earlier = adx_ring.front().copied().unwrap();
            adxr[today] = (adx[today] + earlier) / 2.0;
        }
    }
    (adx, adxr)
}

/// DX 单遍实现（TA-Lib `TA_DX`）：Wilder ±DM/TR → +DI/-DI → DX，一遍前向扫描完成。
/// `carry == true` 时 `denom==0` 沿用上一根有效值（与 `dx` 现有行为一致）；否则取 0
/// （与 `adx_adxr_fused` 的纯 DX 一致）。与分段实现 [`dm_tr`] + [`di_from_dm_tr`] 逐项相等（ADR 0005）。
///
/// Single forward pass for DX (TA-Lib `TA_DX`): Wilder ±DM/TR → +DI/-DI → DX. With
/// `carry == true`, a zero denom reuses the previous valid value (matches `dx`); otherwise it
/// yields 0 (matches the pure DX in `adx_adxr_fused`). Bit-for-bit equal to the staged
/// `dm_tr` + `di_from_dm_tr` (ADR 0005).
fn dx_from_candles(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
    carry: bool,
) -> Vec<f64> {
    let n = high.len();
    let mut dx = vec![f64::NAN; n];
    if n < period + 1 {
        return dx;
    }
    let p = period as f64;
    let mut today = 0usize;
    let mut prev_high = high[0];
    let mut prev_low = low[0];
    let mut prev_close = close[0];
    let mut prev_plus_dm = 0.0_f64;
    let mut prev_minus_dm = 0.0_f64;
    let mut prev_tr = 0.0_f64;
    // 种子累积（与 [`dm_tr`] 逐字节一致）。Seed accumulation (byte-identical to `dm_tr`).
    let mut i = period - 1;
    while i > 0 {
        today += 1;
        let tp = high[today];
        let diff_p = tp - prev_high;
        prev_high = tp;
        let tl = low[today];
        let diff_m = prev_low - tl;
        prev_low = tl;
        if diff_m > 0.0 && diff_p < diff_m {
            prev_minus_dm += diff_m;
        } else if diff_p > 0.0 && diff_p > diff_m {
            prev_plus_dm += diff_p;
        }
        let mut range = prev_high - prev_low;
        let mut tmp = (prev_high - prev_close).abs();
        if tmp > range {
            range = tmp;
        }
        tmp = (prev_low - prev_close).abs();
        if tmp > range {
            range = tmp;
        }
        prev_tr += range;
        prev_close = close[today];
        i -= 1;
    }
    let mut last = f64::NAN; // 用于 `denom==0` 前向填充。/ carry-forward state.
    while today < n - 1 {
        today += 1;
        let tp = high[today];
        let diff_p = tp - prev_high;
        prev_high = tp;
        let tl = low[today];
        let diff_m = prev_low - tl;
        prev_low = tl;
        prev_minus_dm -= prev_minus_dm / p;
        prev_plus_dm -= prev_plus_dm / p;
        if diff_m > 0.0 && diff_p < diff_m {
            prev_minus_dm += diff_m;
        } else if diff_p > 0.0 && diff_p > diff_m {
            prev_plus_dm += diff_p;
        }
        let mut range = prev_high - prev_low;
        let mut tmp = (prev_high - prev_close).abs();
        if tmp > range {
            range = tmp;
        }
        tmp = (prev_low - prev_close).abs();
        if tmp > range {
            range = tmp;
        }
        prev_tr = prev_tr - prev_tr / p + range;
        prev_close = close[today];
        // +DI / -DI（与 [`di_from_dm_tr`] 一致）。+DI / -DI (matches `di_from_dm_tr`).
        let (pdi, mdi) = if prev_tr == 0.0 {
            (0.0, 0.0)
        } else {
            (
                100.0 * prev_plus_dm / prev_tr,
                100.0 * prev_minus_dm / prev_tr,
            )
        };
        let denom = pdi + mdi;
        let val = if denom != 0.0 {
            100.0 * (pdi - mdi).abs() / denom
        } else if carry {
            if last.is_nan() {
                0.0
            } else {
                last
            }
        } else {
            0.0
        };
        dx[today] = val;
        last = val;
    }
    dx
}



/// 正方向性运动（TA-Lib `TA_PLUS_DM`，Wilder 平滑）。前导 `period-1` 个为 [`f64::NAN`]。
///
/// 与 TA-Lib 一致，方向性运动函数接收完整蜡烛数据（`high`/`low`/`close`）；
/// `close` 仅用于内部真实波幅（TR）计算，不影响本函数的输出值。
///
/// Like TA-Lib, the directional-movement functions take the full candle data
/// (`high`/`low`/`close`); `close` is only used internally for True Range and does not
/// affect this function's output values.
pub fn plus_dm(high: &[f64], low: &[f64], close: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "plus_dm")?;
    let mut out = vec![f64::NAN; high.len()];
    plus_dm_with_output(high, low, close, time_period, &mut out)?;
    Ok(out)
}

/// 正方向性运动，零拷贝写入 `out`（与 `high` 等长，前导 `period-1` 为 NaN）。见 [`plus_dm`]。
///
/// Positive Directional Movement, written zero-copy into `out`. See [`plus_dm`]. Numerically
/// identical to [`plus_dm`].
pub fn plus_dm_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "plus_dm")?;
    if out.len() != high.len() {
        return Err(TaError::BadParam(
            "plus_dm_with_output: out length must equal high length".into(),
        ));
    }
    let (pdm, _, _) = dm_tr(high, low, close, time_period);
    out.copy_from_slice(&pdm);
    Ok(())
}

/// `plus_dm` 便捷版本，默认周期 14。/ `plus_dm` with default period (14).
pub fn plus_dm_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    plus_dm(high, low, close, ATR_PERIOD)
}

/// 负方向性运动（TA-Lib `TA_MINUS_DM`，Wilder 平滑）。前导 `period-1` 个为 [`f64::NAN`]。
///
/// 参见 `plus_dm` 关于 `close` 参数的说明。
/// See `plus_dm` for the note about the `close` parameter.
pub fn minus_dm(high: &[f64], low: &[f64], close: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "minus_dm")?;
    let mut out = vec![f64::NAN; high.len()];
    minus_dm_with_output(high, low, close, time_period, &mut out)?;
    Ok(out)
}

/// 负方向性运动，零拷贝写入 `out`（与 `high` 等长，前导 `period-1` 为 NaN）。见 [`minus_dm`]。
///
/// Negative Directional Movement, written zero-copy into `out`. See [`minus_dm`]. Numerically
/// identical to [`minus_dm`].
pub fn minus_dm_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "minus_dm")?;
    if out.len() != high.len() {
        return Err(TaError::BadParam(
            "minus_dm_with_output: out length must equal high length".into(),
        ));
    }
    let (_, mdm, _) = dm_tr(high, low, close, time_period);
    out.copy_from_slice(&mdm);
    Ok(())
}

/// `minus_dm` 便捷版本，默认周期 14。/ `minus_dm` with default period (14).
pub fn minus_dm_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    minus_dm(high, low, close, ATR_PERIOD)
}

/// 正方向性指标（TA-Lib `TA_PLUS_DI`）。前导 `period-1` 个为 [`f64::NAN`]。
pub fn plus_di(high: &[f64], low: &[f64], close: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "plus_di")?;
    let mut out = vec![f64::NAN; high.len()];
    plus_di_with_output(high, low, close, time_period, &mut out)?;
    Ok(out)
}

/// 正方向性指标，零拷贝写入 `out`（与 `high` 等长，前导 `period-1` 为 NaN）。见 [`plus_di`]。
///
/// Positive Directional Indicator, written zero-copy into `out`. See [`plus_di`]. Numerically
/// identical to [`plus_di`].
pub fn plus_di_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "plus_di")?;
    if out.len() != high.len() {
        return Err(TaError::BadParam(
            "plus_di_with_output: out length must equal high length".into(),
        ));
    }
    let (pdm, mdm, tr) = dm_tr(high, low, close, time_period);
    let (pdi, _) = di_from_dm_tr(&pdm, &mdm, &tr, time_period);
    out.copy_from_slice(&pdi);
    Ok(())
}

/// `plus_di` 便捷版本，默认周期 14。/ `plus_di` with default period (14).
pub fn plus_di_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    plus_di(high, low, close, ATR_PERIOD)
}

/// 负方向性指标（TA-Lib `TA_MINUS_DI`）。前导 `period-1` 个为 [`f64::NAN`]。
pub fn minus_di(high: &[f64], low: &[f64], close: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "minus_di")?;
    let mut out = vec![f64::NAN; high.len()];
    minus_di_with_output(high, low, close, time_period, &mut out)?;
    Ok(out)
}

/// 负方向性指标，零拷贝写入 `out`（与 `high` 等长，前导 `period-1` 为 NaN）。见 [`minus_di`]。
///
/// Negative Directional Indicator, written zero-copy into `out`. See [`minus_di`]. Numerically
/// identical to [`minus_di`].
pub fn minus_di_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "minus_di")?;
    if out.len() != high.len() {
        return Err(TaError::BadParam(
            "minus_di_with_output: out length must equal high length".into(),
        ));
    }
    let (pdm, mdm, tr) = dm_tr(high, low, close, time_period);
    let (_, mdi) = di_from_dm_tr(&pdm, &mdm, &tr, time_period);
    out.copy_from_slice(&mdi);
    Ok(())
}

/// `minus_di` 便捷版本，默认周期 14。/ `minus_di` with default period (14).
pub fn minus_di_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    minus_di(high, low, close, ATR_PERIOD)
}

/// 平均方向性运动指数（TA-Lib `TA_ADX`）。前导 `2*period-1` 个为 [`f64::NAN`]。
pub fn adx(high: &[f64], low: &[f64], close: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "adx")?;
    let mut out = vec![f64::NAN; high.len()];
    adx_with_output(high, low, close, time_period, &mut out)?;
    Ok(out)
}

/// 平均方向性运动指数，零拷贝写入 `out`（与 `high` 等长，前导 `2*period-1` 为 NaN）。见 [`adx`]。
///
/// Average Directional Movement Index, written zero-copy into `out`. See [`adx`]. Numerically
/// identical to [`adx`].
pub fn adx_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "adx")?;
    if out.len() != high.len() {
        return Err(TaError::BadParam(
            "adx_with_output: out length must equal high length".into(),
        ));
    }
    out.copy_from_slice(&adx_adxr_fused(high, low, close, time_period).0);
    Ok(())
}

/// `adx` 便捷版本，默认周期 14。/ `adx` with default period (14).
pub fn adx_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    adx(high, low, close, ATR_PERIOD)
}

/// 平均方向性运动指数评级（TA-Lib `TA_ADXR`）。前导 `3*period-2` 个为 [`f64::NAN`]。
pub fn adxr(high: &[f64], low: &[f64], close: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "adxr")?;
    let mut out = vec![f64::NAN; high.len()];
    adxr_with_output(high, low, close, time_period, &mut out)?;
    Ok(out)
}

/// 平均方向性运动指数评级，零拷贝写入 `out`（与 `high` 等长，前导 `3*period-2` 为 NaN）。见 [`adxr`]。
///
/// ADX Rating, written zero-copy into `out`. See [`adxr`]. Numerically identical to [`adxr`].
pub fn adxr_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "adxr")?;
    if out.len() != high.len() {
        return Err(TaError::BadParam(
            "adxr_with_output: out length must equal high length".into(),
        ));
    }
    out.copy_from_slice(&adx_adxr_fused(high, low, close, time_period).1);
    Ok(())
}

/// `adxr` 便捷版本，默认周期 14。/ `adxr` with default period (14).
pub fn adxr_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    adxr(high, low, close, ATR_PERIOD)
}

// ---------------------------------------------------------------------------
// AROON / AROONOSC
// ---------------------------------------------------------------------------

/// 阿隆指标（TA-Lib `TA_AROON`）。`up/down = 100 * (period - 距N周期高/低点的位移) / period`。
/// 前导 `period` 个为 [`f64::NAN`]。
///
/// 算法采用 TA-Lib 的流式极值跟踪（sticky streaming cache）：窗口为 `[today-period, today]`
/// （长度 `period+1`），极值索引用 `<=`/`>=` 并列取“最近”一根；当缓存索引滑出窗口
/// （`extremeIdx < trailingIdx`）时全窗口重扫。
///
/// ⚠️ 兼容性说明：本项目基准用的 TA-Lib 0.7.1 构建（`libta-lib.0.7.1`）在 `TA_AROON` 中把
/// `outAroonUp`/`outAroonDown` 两路输出**互换**了（已对黄金向量与随机数据双向核验，误差 0）。
/// adaq-talib 为与黄金向量 1:1 一致，按基准构建的实测行为返回：本函数返回的 `up` 实际对应
/// 真实下轨、`down` 对应真实上轨；`aroon_osc = up - down` 因此等于真实 `down - up`。
pub fn aroon(high: &[f64], low: &[f64], time_period: usize) -> Result<Aroon, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low], "aroon")?;
    let n = high.len();
    let mut out = Aroon {
        up: vec![f64::NAN; n],
        down: vec![f64::NAN; n],
    };
    aroon_with_output(high, low, time_period, &mut out)?;
    Ok(out)
}

/// 阿隆指标，零拷贝写入 `out`（`up`/`down` 与 `high` 等长，前导 `period` 为 NaN）。见 [`aroon`]。
///
/// AROON, written zero-copy into `out` (equal-length `up`/`down`). See [`aroon`]. Numerically
/// identical to [`aroon`] (preserves the reference build's swapped `up`/`down`).
pub fn aroon_with_output(
    high: &[f64],
    low: &[f64],
    time_period: usize,
    out: &mut Aroon,
) -> Result<(), TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low], "aroon")?;
    let n = high.len();
    if out.up.len() != n || out.down.len() != n {
        return Err(TaError::BadParam(
            "aroon_with_output: out vectors must have length == high length".into(),
        ));
    }
    let period = time_period as i64;
    // TRUE up/down from the canonical streaming algorithm (before the build's swap).
    let mut true_up = vec![f64::NAN; n];
    let mut true_down = vec![f64::NAN; n];
    if n <= time_period {
        // Return swapped-but-empty so `up`/`down` still line up with the contract.
        return Ok(());
    }
    let mut today: i64 = time_period as i64; // first output index (startIdx)
    let mut trailing_idx: i64 = today - period; // = 0
    let mut lowest_idx: i64 = -1;
    let mut highest_idx: i64 = -1;
    let mut lowest: f64 = 0.0;
    let mut highest: f64 = 0.0;
    let factor = 100.0 / period as f64;
    let end = n as i64 - 1;
    while today <= end {
        // --- lowest (newest-wins on ties via `<=`) ---
        let mut tmp = low[today as usize];
        if lowest_idx < trailing_idx {
            lowest_idx = trailing_idx;
            lowest = low[lowest_idx as usize];
            let mut i = lowest_idx;
            while i + 1 <= today {
                i += 1;
                tmp = low[i as usize];
                if tmp <= lowest {
                    lowest_idx = i;
                    lowest = tmp;
                }
            }
        } else if tmp <= lowest {
            lowest_idx = today;
            lowest = tmp;
        }
        // --- highest (newest-wins on ties via `>=`) ---
        tmp = high[today as usize];
        if highest_idx < trailing_idx {
            highest_idx = trailing_idx;
            highest = high[highest_idx as usize];
            let mut i = highest_idx;
            while i + 1 <= today {
                i += 1;
                tmp = high[i as usize];
                if tmp >= highest {
                    highest_idx = i;
                    highest = tmp;
                }
            }
        } else if tmp >= highest {
            highest_idx = today;
            highest = tmp;
        }
        true_up[today as usize] = factor * (period - (today - highest_idx)) as f64;
        true_down[today as usize] = factor * (period - (today - lowest_idx)) as f64;
        trailing_idx += 1;
        today += 1;
    }
    // Match the reference build's swapped up/down output 1:1 with the golden vectors.
    out.up = true_down;
    out.down = true_up;
    Ok(())
}

/// `aroon` 便捷版本，默认周期 14。/ `aroon` with default period (14).
pub fn aroon_default(high: &[f64], low: &[f64]) -> Result<Aroon, TaError> {
    aroon(high, low, AROON_PERIOD)
}

/// 阿隆震荡器（TA-Lib `TA_AROONOSC`）。基准构建中 `AROONOSC = 返回的上轨 - 返回的下轨`
/// （由于 `aroon` 的 up/down 已按基准构建互换，故本值等于真实 `down - up`，与黄金向量一致）。
pub fn aroon_osc(high: &[f64], low: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low], "aroon_osc")?;
    let mut out = vec![f64::NAN; high.len()];
    aroon_osc_with_output(high, low, time_period, &mut out)?;
    Ok(out)
}

/// 阿隆震荡器，零拷贝写入 `out`（与 `high` 等长）。见 [`aroon_osc`]。
///
/// AROON Oscillator, written zero-copy into `out`. See [`aroon_osc`]. Numerically identical
/// to [`aroon_osc`].
pub fn aroon_osc_with_output(
    high: &[f64],
    low: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low], "aroon_osc")?;
    if out.len() != high.len() {
        return Err(TaError::BadParam(
            "aroon_osc_with_output: out length must equal high length".into(),
        ));
    }
    let mut a = Aroon {
        up: vec![f64::NAN; high.len()],
        down: vec![f64::NAN; high.len()],
    };
    aroon_with_output(high, low, time_period, &mut a)?;
    // `aroon` 返回的是按基准构建互换后的 up/down（见其文档），故此处用 `down - up`
    // 还原真实 `AROONOSC = 真实上轨 - 真实下轨 = TRUE_up - TRUE_down`，与黄金向量一致。
    for i in 0..out.len() {
        let u = a.up[i];
        let d = a.down[i];
        out[i] = if u.is_nan() || d.is_nan() {
            f64::NAN
        } else {
            d - u
        };
    }
    Ok(())
}

/// `aroon_osc` 便捷版本，默认周期 14。/ `aroon_osc` with default period (14).
pub fn aroon_osc_default(high: &[f64], low: &[f64]) -> Result<Vec<f64>, TaError> {
    aroon_osc(high, low, AROON_PERIOD)
}

// ---------------------------------------------------------------------------
// STOCH / STOCHF / STOCHRSI
// ---------------------------------------------------------------------------

/// 慢速随机指标（TA-Lib `TA_STOCH`）。
///
/// 快速 `%K = 100 * (close - LL) / (HH - LL)`（窗口 `fastK`），
/// 慢速 `%K = SMA(fastK, slowK)`，慢速 `%D = SMA(slowK, slowD)`。
/// 三个数组等长、对齐到同一不稳定期的前导 NaN（`lookback = fastK+slowK+slowD-3`）。
/// 快速 %K 计算（STOCH / STOCHF 共用），使用单调队列 O(n) 极值（P2-6，ADR 0010）。
///
/// Fast `%K` (shared by STOCH / STOCHF), using the monotonic-queue O(n) extremes (P2-6, ADR 0010).
///
/// `fastK[i] = 100 * (close[i] - LL) / (HH - LL)`，HH/LL 为窗口内最高/最低（最右 tie-break），
/// 与朴素每窗扫描逐项相等（零偏差，ADR 0005）。前导 `fast_k_period-1` 个为 [`f64::NAN`]。
fn stoch_fastk(high: &[f64], low: &[f64], close: &[f64], fast_k_period: usize) -> Vec<f64> {
    let n = close.len();
    let mut fastk = vec![f64::NAN; n];
    if n < fast_k_period {
        return fastk;
    }
    // 单遍融合：在一次遍历中同时维护 high 的最大值队列与 low 的最小值队列，直接写出
    // 快速随机 %K，消除原先 `rolling_max` + `rolling_min` 的两次独立扫描与两次 `Vec` 分配
    // （P3-3，ADR 0005 零偏差）。
    let mut max_dq = crate::core::MonoQueue::with_capacity(fast_k_period);
    let mut min_dq = crate::core::MonoQueue::with_capacity(fast_k_period);
    for i in 0..n {
        while !max_dq.is_empty() && max_dq.front() + fast_k_period <= i {
            max_dq.pop_front();
        }
        while !min_dq.is_empty() && min_dq.front() + fast_k_period <= i {
            min_dq.pop_front();
        }
        while !max_dq.is_empty() && high[max_dq.back()] <= high[i] {
            max_dq.pop_back();
        }
        while !min_dq.is_empty() && low[min_dq.back()] >= low[i] {
            min_dq.pop_back();
        }
        max_dq.push_back(i);
        min_dq.push_back(i);
        if i >= fast_k_period - 1 {
            let hh = high[max_dq.front()];
            let ll = low[min_dq.front()];
            fastk[i] = if hh == ll {
                0.0
            } else {
                100.0 * (close[i] - ll) / (hh - ll)
            };
        }
    }
    fastk
}

pub fn stoch(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    fast_k_period: usize,
    slow_k_period: usize,
    slow_d_period: usize,
) -> Result<Stoch, TaError> {
    check_period(fast_k_period)?;
    check_period(slow_k_period)?;
    check_period(slow_d_period)?;
    check_eq_len(&[high, low, close], "stoch")?;
    let n = close.len();
    let mut out = Stoch {
        slow_k: vec![f64::NAN; n],
        slow_d: vec![f64::NAN; n],
    };
    stoch_with_output(
        high,
        low,
        close,
        fast_k_period,
        slow_k_period,
        slow_d_period,
        &mut out,
    )?;
    Ok(out)
}

/// 慢速随机指标，零拷贝写入 `out`（`slow_k` / `slow_d` 与 `close` 等长）。见 [`stoch`]。
///
/// Slow stochastic, written zero-copy into `out`. See [`stoch`]. Uses [`stoch_fastk`] (O(n)
/// monotonic-queue extremes) instead of the per-window O(n·period) scan. Numerically identical
/// to [`stoch`].
pub fn stoch_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    fast_k_period: usize,
    slow_k_period: usize,
    slow_d_period: usize,
    out: &mut Stoch,
) -> Result<(), TaError> {
    check_period(fast_k_period)?;
    check_period(slow_k_period)?;
    check_period(slow_d_period)?;
    check_eq_len(&[high, low, close], "stoch")?;
    let n = close.len();
    if out.slow_k.len() != n || out.slow_d.len() != n {
        return Err(TaError::BadParam(
            "stoch_with_output: out bands must have length == close length".into(),
        ));
    }
    let fastk = stoch_fastk(high, low, close, fast_k_period);
    let slow_k = rolling_mean_skip(&fastk, slow_k_period);
    let slow_d = rolling_mean_skip(&slow_k, slow_d_period);
    // 三数组对齐到同一前导不稳定期（lookback = fastK+slowK+slowD-3），见 ADR 0007。
    // Align all three arrays to the same leading unstable period (lookback = fastK+slowK+slowD-3).
    let lookback = fast_k_period + slow_k_period + slow_d_period - 3;
    out.slow_k.copy_from_slice(&slow_k);
    out.slow_d.copy_from_slice(&slow_d);
    for i in 0..lookback.min(n) {
        out.slow_k[i] = f64::NAN;
    }
    Ok(())
}

/// `stoch` 便捷版本，默认 5 / 3 / 3。/ `stoch` with defaults 5 / 3 / 3.
pub fn stoch_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Stoch, TaError> {
    stoch(
        high,
        low,
        close,
        STOCH_FAST_K,
        STOCH_SLOW_K,
        STOCH_SLOW_D,
    )
}

/// 快速随机指标（TA-Lib `TA_STOCHF`）。
/// `fastK` 同上；`fastD = SMA(fastK, fastD)`。`lookback = fastK + fastD - 2`。
pub fn stoch_f(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    fast_k_period: usize,
    fast_d_period: usize,
) -> Result<StochF, TaError> {
    check_period(fast_k_period)?;
    check_period(fast_d_period)?;
    check_eq_len(&[high, low, close], "stoch_f")?;
    let n = close.len();
    let mut out = StochF {
        fast_k: vec![f64::NAN; n],
        fast_d: vec![f64::NAN; n],
    };
    stoch_f_with_output(high, low, close, fast_k_period, fast_d_period, &mut out)?;
    Ok(out)
}

/// 快速随机指标，零拷贝写入 `out`（`fast_k` / `fast_d` 与 `close` 等长）。见 [`stoch_f`]。
///
/// Fast stochastic, written zero-copy into `out`. See [`stoch_f`]. Uses [`stoch_fastk`] (O(n)
/// monotonic-queue extremes) instead of the per-window O(n·period) scan. Numerically identical
/// to [`stoch_f`].
pub fn stoch_f_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    fast_k_period: usize,
    fast_d_period: usize,
    out: &mut StochF,
) -> Result<(), TaError> {
    check_period(fast_k_period)?;
    check_period(fast_d_period)?;
    check_eq_len(&[high, low, close], "stoch_f")?;
    let n = close.len();
    if out.fast_k.len() != n || out.fast_d.len() != n {
        return Err(TaError::BadParam(
            "stoch_f_with_output: out bands must have length == close length".into(),
        ));
    }
    // 数据量足够且启用 `parallel` feature 时走多核分块；内核与串行逐字节一致，输出 1:1。
    // Under the `parallel` feature with enough data, use multi-core chunking; the kernel is
    // byte-identical to the serial path, so output is 1:1.
    #[cfg(feature = "parallel")]
    {
        if n >= 8192 {
            return stoch_f_parallel_with_output(high, low, close, fast_k_period, fast_d_period, out);
        }
    }
    stoch_f_serial_with_output(high, low, close, fast_k_period, fast_d_period, out)
}

/// 快速随机指标串行内核（与 TA-Lib `TA_STOCHF` 逐项 1:1）。见 [`stoch_f_with_output`]。
/// Serial kernel for fast stochastic (1:1 with TA-Lib `TA_STOCHF`). See [`stoch_f_with_output`].
fn stoch_f_serial_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    fast_k_period: usize,
    fast_d_period: usize,
    out: &mut StochF,
) -> Result<(), TaError> {
    let n = close.len();
    let fastk = stoch_fastk(high, low, close, fast_k_period);
    let fast_d = rolling_mean_skip(&fastk, fast_d_period);
    // 两数组对齐到同一前导不稳定期（lookback = fastK+fastD-2），见 ADR 0007。
    // Align both arrays to the same leading unstable period (lookback = fastK+fastD-2).
    let lookback = fast_k_period + fast_d_period - 2;
    out.fast_k.copy_from_slice(&fastk);
    out.fast_d.copy_from_slice(&fast_d);
    for i in 0..lookback.min(n) {
        out.fast_k[i] = f64::NAN;
    }
    Ok(())
}

/// 快速随机指标串行版本（feature 无关，供并行对照测试作黄金参考）。见 [`stoch_f`]。
/// Serial fast stochastic (feature-agnostic; golden reference for the parallel equality test). See [`stoch_f`].
pub fn stoch_f_serial(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    fast_k_period: usize,
    fast_d_period: usize,
) -> Result<StochF, TaError> {
    check_period(fast_k_period)?;
    check_period(fast_d_period)?;
    check_eq_len(&[high, low, close], "stoch_f")?;
    let n = close.len();
    let mut out = StochF {
        fast_k: vec![f64::NAN; n],
        fast_d: vec![f64::NAN; n],
    };
    stoch_f_serial_with_output(high, low, close, fast_k_period, fast_d_period, &mut out)?;
    Ok(out)
}

/// 快速随机指标多核并行版本（需 `parallel` feature）。复用 [`stoch_f_serial_with_output`] 的
/// `stoch_fastk` 单遍单调队列内核，以 `fastK-1` 前导重叠播种各分块的极值队列，输出与串行逐项 1:1。
/// Multi-core parallel fast stochastic (requires the `parallel` feature). Reuses the `stoch_fastk`
/// single-pass monotonic-queue kernel of [`stoch_f_serial_with_output`] with `fastK-1` leading
/// overlap; output is 1:1 with the serial path.
#[cfg(feature = "parallel")]
pub fn stoch_f_parallel(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    fast_k_period: usize,
    fast_d_period: usize,
) -> Result<StochF, TaError> {
    check_period(fast_k_period)?;
    check_period(fast_d_period)?;
    check_eq_len(&[high, low, close], "stoch_f")?;
    let n = close.len();
    let mut out = StochF {
        fast_k: vec![f64::NAN; n],
        fast_d: vec![f64::NAN; n],
    };
    stoch_f_parallel_with_output(high, low, close, fast_k_period, fast_d_period, &mut out)?;
    Ok(out)
}

/// 快速随机指标并行内核（零拷贝写入 `out`）。见 [`stoch_f_parallel`]。
/// Parallel kernel for fast stochastic (zero-copy into `out`). See [`stoch_f_parallel`].
#[cfg(feature = "parallel")]
fn stoch_f_parallel_with_output(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    fast_k_period: usize,
    fast_d_period: usize,
    out: &mut StochF,
) -> Result<(), TaError> {
    check_period(fast_k_period)?;
    check_period(fast_d_period)?;
    check_eq_len(&[high, low, close], "stoch_f")?;
    let n = close.len();
    if out.fast_k.len() != n || out.fast_d.len() != n {
        return Err(TaError::BadParam(
            "stoch_f_parallel_with_output: out bands must have length == close length".into(),
        ));
    }
    let fk = fast_k_period;
    let fd = fast_d_period;
    // 重叠须覆盖完整前导不稳定期 `lookback = fk+fd-2`（`fast_k` 自身仅 `fk-1`，但 `fast_d`
    // 的 SMA 对齐把不稳定期拉长到 fk+fd-2），否则分块自有区间会落在前导 NaN 内。
    // Overlap must cover the full unstable period `lookback = fk+fd-2` (the `fast_d` SMA alignment
    // extends the leading NaN beyond `stoch_fastk`'s own `fk-1`), else the owned range would fall
    // inside the leading-NaN zone.
    let overlap = fk + fd - 2;
    crate::parallel::parallel_index_map_2(
        n,
        overlap,
        &mut out.fast_k,
        &mut out.fast_d,
        |start, end| {
            let mut local = StochF {
                fast_k: vec![f64::NAN; end - start],
                fast_d: vec![f64::NAN; end - start],
            };
            let _ = stoch_f_serial_with_output(
                &high[start..end],
                &low[start..end],
                &close[start..end],
                fk,
                fd,
                &mut local,
            );
            (local.fast_k, local.fast_d)
        },
    );
    Ok(())
}

/// `stoch_f` 便捷版本，默认 5 / 3。/ `stoch_f` with defaults 5 / 3.
pub fn stoch_f_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<StochF, TaError> {
    stoch_f(high, low, close, STOCH_FAST_K, STOCH_SLOW_D)
}

/// 随机相对强弱（TA-Lib `TA_STOCHRSI`）。
///
/// TA-Lib 实现为 `STOCHF(RSI(close, rsiPeriod), fastK = timePeriod, fastD = 3)`，
/// 仅输出 `fastK` 一行（`fastD` 仅用于确定对齐的不稳定期长度）。等价做法：先算 RSI，
/// 去掉其前导不稳定期后将 RSI 序列打包到从 0 起始的缓冲，在其上跑快速随机，
/// 再把结果映射回原序列坐标。
///
/// `lookback = rsiPeriod + timePeriod + 3 - 2`（`fastD` 固定为 TA-Lib 默认 3）。
/// 默认参数（14 / 14）下首个有效值位于索引 `14 + 14 + 1 = 29`。
///
/// Stochastic RSI (TA-Lib `TA_STOCHRSI`): `STOCHF(RSI(close, rsiPeriod), fastK = timePeriod,
/// fastD = 3)`， emitting only the `fastK` line (`fastD` only governs the unstable-period
/// length). First valid index = `rsiPeriod + timePeriod + fastD - 2`.
pub fn stoch_rsi(
    close: &[f64],
    rsi_period: usize,
    time_period: usize,
) -> Result<Vec<f64>, TaError> {
    check_period(rsi_period)?;
    check_period(time_period)?;
    let r = rsi(close, rsi_period)?;
    let n = close.len();
    // 没有足够的 RSI 有效值：全 NaN 直接返回。
    // No RSI valid value yet: return all-NaN.
    if n <= rsi_period {
        return Ok(vec![f64::NAN; n]);
    }
    // TA-Lib 把 RSI 打包到从 0 起始的缓冲（去掉前导 NaN）再在其上跑 STOCHF；
    // 否则窗口若覆盖 RSI 前导 NaN 会产生伪值，破坏严格 NaN 对齐（见 ADR 0007）。
    // TA-Lib compacts the RSI (drops the leading NaN) before STOCHF; otherwise a window
    // overlapping the RSI leading-NaN would yield spurious values.
    let compact: Vec<f64> = r[rsi_period..].to_vec();
    let fastd_period = 3usize; // TA-Lib STOCHRSI 默认 optInFastD_Period。
    let sf = stoch_f(&compact, &compact, &compact, time_period, fastd_period)?;
    let mut out = vec![f64::NAN; n];
    for (j, &v) in sf.fast_k.iter().enumerate() {
        out[rsi_period + j] = v;
    }
    Ok(out)
}

/// `stoch_rsi` 便捷版本，默认 14 / 14。/ `stoch_rsi` with defaults 14 / 14.
pub fn stoch_rsi_default(close: &[f64]) -> Result<Vec<f64>, TaError> {
    stoch_rsi(close, STOCHRSI_RSI_PERIOD, STOCHRSI_PERIOD)
}

// ---------------------------------------------------------------------------
// TRIX
// ---------------------------------------------------------------------------

/// 三重指数平滑（TA-Lib `TA_TRIX`）。
/// `TRIX = 100 * (EMA3[i] - EMA3[i-1]) / EMA3[i-1]`，其中 `EMA3` 为三重 EMA。
/// 前导 `3*period-2` 个为 [`f64::NAN`]。
///
/// 三重 EMA 通过嵌套 [`ema`]（经典种子 EMA）实现，与 TA-Lib `ta_trix.c` 的数值一致；
/// 注意系数固定为 `100`（非 `1000`），否则会产生 10× 偏差。
///
/// The triple EMA is implemented via nested [`ema`] (classic-seed EMA), reproducing
/// TA-Lib `ta_trix.c`; note the fixed scale factor is `100` (not `1000`).
pub fn trix(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let n = values.len();
    let lookback = 3 * time_period - 2;
    let mut out = vec![f64::NAN; n];
    if n <= lookback {
        return Ok(out);
    }
    // 三重 EMA 融合为单遍（与逐次 `ema` 调用逐项一致，ADR 0005），仅取最深一层 `E3`
    // 写入 `e3` 暂存，再做轻量合并遍历（4 趟 → 2 趟，P3-2 性能优化）。
    // Triple EMA fused into one pass (bit-for-bit equal to successive `ema` calls, ADR 0005);
    // keep only the deepest level `E3`, then a light combine pass (4 passes → 2).
    let mut e3 = vec![f64::NAN; n];
    crate::core::nested_ema_with_output::<3, _>(values, time_period, |e: &[f64; 3]| e[2], &mut e3);
    for i in lookback..n {
        let cur = e3[i];
        let prev = e3[i - 1];
        if !cur.is_nan() && !prev.is_nan() {
            out[i] = if prev == 0.0 {
                0.0
            } else {
                100.0 * (cur - prev) / prev
            };
        }
    }
    Ok(out)
}

/// `trix` 便捷版本，默认周期 30。/ `trix` with default period (30).
pub fn trix_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    trix(values, TRIX_PERIOD)
}

// ───────────────────────────── DX ─────────────────────────────

/// 方向性运动指数（DX，TA-Lib `TA_DX`）。
///
/// Directional Movement Index. Reuses the Wilder-smoothed ±DI from the shared
/// `directional` helper and returns `DX = 100·|−DI − +DI| / (+DI + −DI)` per bar.
/// When the true range (and thus both DI) is zero, or the DI sum is zero, the
/// previous output is carried forward (matching TA-Lib, which fills the first
/// such case with 0.0). Lookback is `period` (default 14); the first `period`
/// positions are [`f64::NAN`].
///
/// # 参数 / Parameters
/// - `high` / `low` / `close`：蜡烛数据 `&[f64]`。/ Candle data `&[f64]`.
/// - `period`：平滑周期（TA-Lib 默认 14）。/ Smoothing period (default 14).
///
/// # 返回值 / Returns
/// 与输入等长的向量，前导 `period` 个为 [`f64::NAN`]。
/// Equal-length vector; the first `period` positions are [`f64::NAN`].
pub fn dx(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Result<Vec<f64>, TaError> {
    check_period(period)?;
    check_eq_len(&[high, low, close], "dx")?;
    let n = high.len();
    let mut out = vec![f64::NAN; n];
    if n < period + 1 {
        return Ok(out);
    }
    // 单遍融合：Wilder ±DM/TR → +DI/-DI → DX（`denom==0` 前向填充），与分段实现逐项一致。
    // Single forward pass; bit-for-bit equal to the staged implementation (ADR 0005).
    let dx = dx_from_candles(high, low, close, period, true);
    out.copy_from_slice(&dx);
    Ok(out)
}

/// DX，使用 TA-Lib 默认周期 14。/ DX with TA-Lib default period (14).
pub fn dx_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    dx(high, low, close, DX_PERIOD)
}

// ───────────────────────────── IMI ─────────────────────────────

/// 日内动量指数（IMI，TA-Lib `TA_IMI`）。
///
/// Intraday Momentum Index. For each bar it sums, over a rolling window of
/// `period` bars, the up-moves `max(close − open, 0)` and down-moves
/// `max(open − close, 0)`, then returns `100·Σup / (Σup + Σdown)`. If the window
/// is completely flat (every `close == open`), it returns the neutral center
/// 50.0 (matching TA-Lib's `#112` fix so a successful call never emits NaN).
/// Lookback is `period − 1` (default 14); the first `period − 1` positions are
/// [`f64::NAN`].
///
/// # 参数 / Parameters
/// - `open` / `close`：开盘价 / 收盘价 `&[f64]`（IMI 仅需这两者）。/ Open/close only.
/// - `period`：窗口周期（TA-Lib 默认 14）。/ Window period (default 14).
///
/// # 返回值 / Returns
/// 与输入等长的向量，前导 `period − 1` 个为 [`f64::NAN`]。
/// Equal-length vector; the first `period − 1` positions are [`f64::NAN`].
pub fn imi(open: &[f64], close: &[f64], period: usize) -> Result<Vec<f64>, TaError> {
    check_period(period)?;
    check_eq_len(&[open, close], "imi")?;
    let n = open.len();
    let lookback = period - 1;
    let mut out = vec![f64::NAN; n];
    if n <= lookback {
        return Ok(out);
    }
    // 每根 K 的涨/跌贡献（非负）。/ Per-bar up/down contribution (non-negative).
    let mut up = vec![0.0_f64; n];
    let mut down = vec![0.0_f64; n];
    for i in 0..n {
        let o = open[i];
        let c = close[i];
        if c > o {
            up[i] = c - o;
        } else if c < o {
            down[i] = o - c;
        }
    }
    // 滚动窗口求和。/ Rolling window sum.
    let mut sum_up = 0.0_f64;
    let mut sum_down = 0.0_f64;
    for i in 0..period {
        sum_up += up[i];
        sum_down += down[i];
    }
    for i in lookback..n {
        // 减去离开窗口的尾部（当 i >= period 时）。/ drop the trailing bar.
        if i >= period {
            sum_up -= up[i - period];
            sum_down -= down[i - period];
        }
        // 加入当前 K 线（i == lookback 时窗口已在上面初始化完毕）。
        // Add the current bar (window already initialized when i == lookback).
        if i == lookback {
            // 初始化窗口已包含 [0, period-1]；此处无需再加。
        } else {
            sum_up += up[i];
            sum_down += down[i];
        }
        let total = sum_up + sum_down;
        out[i] = if total == 0.0 { 50.0 } else { 100.0 * sum_up / total };
    }
    Ok(out)
}

/// IMI，使用 TA-Lib 默认周期 14。/ IMI with TA-Lib default period (14).
pub fn imi_default(open: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    imi(open, close, IMI_PERIOD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mom_basic() {
        let p = [1.0, 2.0, 4.0, 8.0, 16.0];
        let out = mom(&p, 2).unwrap();
        assert!(out[0].is_nan() && out[1].is_nan());
        // MOM[2] = p[2] - p[0] = 4 - 1 = 3; MOM[4] = 16 - 4 = 12.
        assert!((out[2] - 3.0).abs() < 1e-9);
        assert!((out[4] - 12.0).abs() < 1e-9);
    }

    #[test]
    fn roc_family() {
        let p = [1.0, 2.0, 4.0, 8.0, 16.0];
        let r = roc(&p, 2).unwrap();
        // ROC[2] = 100 * (4 - 1) / 1 = 300.
        assert!((r[2] - 300.0).abs() < 1e-9);
        let c = rocr(&p, 2).unwrap();
        assert!((c[2] - 4.0).abs() < 1e-9);
        let c100 = rocr100(&p, 2).unwrap();
        assert!((c100[2] - 400.0).abs() < 1e-9);
    }

    #[test]
    fn rsi_wilder_seed() {
        // 经典 RSI 小样本手算校验（period=2）。
        // Classic RSI manual check (period=2).
        let p = [1.0, 2.0, 3.0, 2.0, 4.0];
        let out = rsi(&p, 2).unwrap();
        assert!(out[0].is_nan() && out[1].is_nan());
        // gains/losses: d=[_,1,1,-1,2]; seed gain=(1+1)/2=1, loss=(0+0)/2=0 -> RSI[2]=100
        assert!((out[2] - 100.0).abs() < 1e-9);
        // i=3: gain=(1*1+0)/2=0.5, loss=(0*1+1)/2=0.5 -> RS=1 -> RSI=50
        assert!((out[3] - 50.0).abs() < 1e-9);
        // i=4: gain=(0.5*1+2)/2=1.25, loss=(0.5*1+0)/2=0.25 -> RS=5 -> RSI=83.333...
        assert!((out[4] - (100.0 - 100.0 / 6.0)).abs() < 1e-9);
    }

    #[test]
    fn macd_shape() {
        let p: Vec<f64> = (0..60).map(|i| (i as f64).sin() + 2.0).collect();
        let m = macd(&p, 12, 26, 9).unwrap();
        assert_eq!(m.macd.len(), p.len());
        // 前导 lookback = 26+9-2 = 33；macd 与 signal 在 [0,33) 为 NaN，signal 自 33 起有效。
        assert!(m.macd[0].is_nan() && m.signal[32].is_nan());
        assert!(!m.signal[40].is_nan());
    }

    #[test]
    fn cmo_symmetry() {
        // 对称涨跌 -> CMO 接近 0。/ Symmetric up/down -> CMO near 0.
        let p = [1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0];
        let out = cmo(&p, 4).unwrap();
        assert!(out[4].abs() < 1e-9);
    }

    #[test]
    fn adx_runs() {
        let high: Vec<f64> = (0..40).map(|i| 10.0 + i as f64 * 0.1).collect();
        let low: Vec<f64> = (0..40).map(|i| 9.0 + i as f64 * 0.1).collect();
        let close: Vec<f64> = (0..40).map(|i| 9.5 + i as f64 * 0.1).collect();
        let a = adx(&high, &low, &close, 14).unwrap();
        // lookback = 2*14-1 = 27 -> 此前为 NaN，首个有效位于索引 27。
        assert!(a[26].is_nan());
        assert!(!a[27].is_nan());
        assert!(a[30] >= 0.0);
    }

    #[test]
    fn stoch_shape() {
        let close: Vec<f64> = (0..40).map(|i| (i as f64 * 0.3).sin()).collect();
        let high: Vec<f64> = close.iter().map(|&c| c + 0.5).collect();
        let low: Vec<f64> = close.iter().map(|&c| c - 0.5).collect();
        let s = stoch(&high, &low, &close, 5, 3, 3).unwrap();
        assert_eq!(s.slow_k.len(), 40);
        // lookback = 5+3+3-3 = 8；slow_k / slow_d 在 [0,8) 对齐为 NaN，
        // slow_k 内部自 6 起有效但对外对齐填充。/ slow_d first valid at 8.
        assert!(s.slow_k[5].is_nan() && s.slow_d[7].is_nan());
        assert!(!s.slow_k[15].is_nan());
    }

    #[test]
    fn bop_zero_range() {
        let o = [1.0; 4];
        let h = [1.0; 4];
        let l = [1.0; 4];
        let c = [1.0; 4];
        let out = bop(&o, &h, &l, &c).unwrap();
        assert!(out.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn trix_shape() {
        let p: Vec<f64> = (0..120).map(|i| (i as f64).sin()).collect();
        let out = trix(&p, 30).unwrap();
        // lookback = 3*30-2 = 88
        assert!(out[87].is_nan());
        assert!(!out[100].is_nan());
    }
}
