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
    APO_FAST, APO_SLOW, AROON_PERIOD, ATR_PERIOD, CMO_PERIOD, MACD_FAST, MACD_SIGNAL, MACD_SLOW,
    MFI_PERIOD, MOM_PERIOD, RSI_PERIOD, STOCH_FAST_K, STOCH_SLOW_D, STOCH_SLOW_K, STOCHRSI_PERIOD,
    STOCHRSI_RSI_PERIOD, TRIX_PERIOD, ULTOSC_PERIOD1, ULTOSC_PERIOD2, ULTOSC_PERIOD3,
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
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    for i in time_period..n {
        out[i] = values[i] - values[i - time_period];
    }
    Ok(out)
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
fn rate_of_change(values: &[f64], time_period: usize, mode: u8) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    for i in time_period..n {
        let prev = values[i - time_period];
        let cur = values[i];
        out[i] = if prev == 0.0 {
            0.0
        } else {
            match mode {
                0 => 100.0 * (cur - prev) / prev,
                1 => (cur - prev) / prev,
                2 => cur / prev,
                _ => 100.0 * cur / prev,
            }
        };
    }
    Ok(out)
}

/// 变动率（Rate of Change，`TA_MOM` 的 `ROC` 变体，TA-Lib `TA_ROC`）。
/// `ROC[i] = 100 * (inReal[i] - inReal[i-period]) / inReal[i-period]`。
pub fn roc(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    rate_of_change(values, time_period, 0)
}

/// `roc` 便捷版本，默认周期 10。/ `roc` with default period (10).
pub fn roc_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    roc(values, MOM_PERIOD)
}

/// 变动率（百分比，TA-Lib `TA_ROCP`）。`ROCP[i] = (cur - prev) / prev`。
pub fn rocp(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    rate_of_change(values, time_period, 1)
}

/// `rocp` 便捷版本，默认周期 10。/ `rocp` with default period (10).
pub fn rocp_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    rocp(values, MOM_PERIOD)
}

/// 变动率（比率，TA-Lib `TA_ROCR`）。`ROCR[i] = cur / prev`。
pub fn rocr(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    rate_of_change(values, time_period, 2)
}

/// `rocr` 便捷版本，默认周期 10。/ `rocr` with default period (10).
pub fn rocr_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    rocr(values, MOM_PERIOD)
}

/// 变动率（比率×100，TA-Lib `TA_ROCR100`）。`ROCR100[i] = 100 * cur / prev`。
pub fn rocr100(values: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    rate_of_change(values, time_period, 3)
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
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if n < time_period + 1 {
        return Ok(out);
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
    Ok(out)
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
    check_period(fast_period)?;
    check_period(slow_period)?;
    check_period(signal_period)?;
    if fast_period >= slow_period {
        return Err(TaError::BadParam(
            "fast_period must be < slow_period".into(),
        ));
    }
    let n = values.len();
    let fast_k = 2.0 / (fast_period as f64 + 1.0);
    let slow_k = 2.0 / (slow_period as f64 + 1.0);
    let signal_k = 2.0 / (signal_period as f64 + 1.0);
    let lookback_signal = signal_period - 1;
    // TA-Lib 锁步实现：lookback = slow 的 EMA lookback + signal 的 EMA lookback。
    // TA-Lib lockstep: lookback = EMA-lookback(slow) + EMA-lookback(signal).
    let lookback_total = lookback_signal + (slow_period - 1); // = slow + signal - 2
    let mut macd_line = vec![f64::NAN; n];
    let mut signal = vec![f64::NAN; n];
    let mut hist = vec![f64::NAN; n];
    if n <= lookback_total {
        return Ok(Macd {
            macd: macd_line,
            signal,
            hist,
        });
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
    macd_line[out_idx] = macd_value;
    signal[out_idx] = prev_signal;
    hist[out_idx] = macd_value - prev_signal;
    while today < n {
        temp_real = values[today];
        today += 1;
        prev_fast = (temp_real - prev_fast) * fast_k + prev_fast;
        prev_slow = (temp_real - prev_slow) * slow_k + prev_slow;
        macd_value = prev_fast - prev_slow;
        prev_signal = (macd_value - prev_signal) * signal_k + prev_signal;
        out_idx += 1;
        macd_line[out_idx] = macd_value;
        signal[out_idx] = prev_signal;
        hist[out_idx] = macd_value - prev_signal;
    }
    Ok(Macd {
        macd: macd_line,
        signal,
        hist,
    })
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
    macd(values, fast_period, slow_period, MACD_SIGNAL)
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
    macd(values, fast_period, slow_period, signal_period)
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
    if fast_period >= slow_period {
        return Err(TaError::BadParam("fast_period must be < slow_period".into()));
    }
    let ef = ema(values, fast_period);
    let es = ema(values, slow_period);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    for i in 0..n {
        if !ef[i].is_nan() && !es[i].is_nan() {
            out[i] = ef[i] - es[i];
        }
    }
    Ok(out)
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
    if fast_period >= slow_period {
        return Err(TaError::BadParam("fast_period must be < slow_period".into()));
    }
    let ef = ema(values, fast_period);
    let es = ema(values, slow_period);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    for i in 0..n {
        if !ef[i].is_nan() && !es[i].is_nan() && es[i] != 0.0 {
            out[i] = 100.0 * (ef[i] - es[i]) / es[i];
        }
    }
    Ok(out)
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
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if n <= time_period {
        return Ok(out);
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
    Ok(out)
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
    let n = close.len();
    let tp: Vec<f64> = (0..n).map(|i| (high[i] + low[i] + close[i]) / 3.0).collect();
    let sma_tp = rolling_mean(&tp, time_period);
    let mut out = vec![f64::NAN; n];
    for i in (time_period - 1)..n {
        let mean = sma_tp[i];
        let mut dev = 0.0;
        for j in 0..time_period {
            dev += (tp[i - j] - mean).abs();
        }
        dev /= time_period as f64;
        out[i] = if dev == 0.0 { 0.0 } else { (tp[i] - mean) / (0.015 * dev) };
    }
    Ok(out)
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
    let n = close.len();
    let mut tp = vec![0.0_f64; n];
    let mut mf = vec![0.0_f64; n];
    for i in 0..n {
        tp[i] = (high[i] + low[i] + close[i]) / 3.0;
        if i > 0 {
            mf[i] = tp[i] * volume[i];
        }
    }
    let mut pos = vec![0.0_f64; n];
    let mut neg = vec![0.0_f64; n];
    for i in 1..n {
        if tp[i] > tp[i - 1] {
            pos[i] = mf[i];
        } else if tp[i] < tp[i - 1] {
            neg[i] = mf[i];
        }
    }
    let sp = rolling_sum(&pos, time_period);
    let sn = rolling_sum(&neg, time_period);
    let mut out = vec![f64::NAN; n];
    for i in time_period..n {
        let (p, l) = (sp[i], sn[i]);
        if p.is_nan() || l.is_nan() {
            continue;
        }
        out[i] = if l == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + p / l) };
    }
    Ok(out)
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
    for i in (time_period - 1)..n {
        let mut hh = high[i];
        let mut ll = low[i];
        for j in 1..time_period {
            if high[i - j] > hh {
                hh = high[i - j];
            }
            if low[i - j] < ll {
                ll = low[i - j];
            }
        }
        out[i] = if hh == ll {
            0.0
        } else {
            -100.0 * (hh - close[i]) / (hh - ll)
        };
    }
    Ok(out)
}

/// `willr` 便捷版本，默认周期 14。/ `willr` with default period (14).
pub fn willr_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    willr(high, low, close, ATR_PERIOD)
}

/// 均势（Balance Of Power，TA-Lib `TA_BOP`）。`BOP = (close - open) / (high - low)`。
/// 无滞后（lookback 0）；若 `high == low` 返回 0.0。
pub fn bop(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    check_eq_len(&[open, high, low, close], "bop")?;
    let n = close.len();
    let mut out = vec![0.0_f64; n];
    for i in 0..n {
        let range = high[i] - low[i];
        out[i] = if range == 0.0 {
            0.0
        } else {
            (close[i] - open[i]) / range
        };
    }
    Ok(out)
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
    let n = close.len();
    let mut bp = vec![0.0_f64; n];
    let mut tr = vec![0.0_f64; n];
    for i in 0..n {
        let prev = if i > 0 { close[i - 1] } else { close[i] };
        bp[i] = close[i] - low[i].min(prev);
        tr[i] = high[i].max(prev) - low[i].min(prev);
    }
    let sbp1 = rolling_sum(&bp, period1);
    let str1 = rolling_sum(&tr, period1);
    let sbp2 = rolling_sum(&bp, period2);
    let str2 = rolling_sum(&tr, period2);
    let sbp3 = rolling_sum(&bp, period3);
    let str3 = rolling_sum(&tr, period3);
    let mut out = vec![f64::NAN; n];
    // TA-Lib 以最长周期 `period3` 为 lookback（首个有效值位于 index `period3`，而非 `period3-1`）。
    // TA-Lib uses the longest period `period3` as the lookback (first valid at index `period3`).
    let lookback = period3;
    for i in lookback..n {
        let (a1, a2, a3) = (sbp1[i], sbp2[i], sbp3[i]);
        let (t1, t2, t3) = (str1[i], str2[i], str3[i]);
        if a1.is_nan() || a2.is_nan() || a3.is_nan() || t1.is_nan() || t2.is_nan() || t3.is_nan() {
            continue;
        }
        if t1 == 0.0 || t2 == 0.0 || t3 == 0.0 {
            out[i] = 0.0;
            continue;
        }
        let avg1 = a1 / t1;
        let avg2 = a2 / t2;
        let avg3 = a3 / t3;
        out[i] = 100.0 * (4.0 * avg1 + 2.0 * avg2 + avg3) / 7.0;
    }
    Ok(out)
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
fn directional(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> (
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
) {
    let n = high.len();
    let mut pdm = vec![f64::NAN; n];
    let mut mdm = vec![f64::NAN; n];
    let mut tr = vec![f64::NAN; n];
    let mut pdi = vec![f64::NAN; n];
    let mut mdi = vec![f64::NAN; n];
    let mut adx = vec![f64::NAN; n];
    let mut adxr = vec![f64::NAN; n];

    // 需要至少 period+1 个价格才能产出首个 +DM/-DM（位于索引 period-1）。
    // Need at least period+1 prices to yield the first ±DM (at index period-1).
    if n < period + 1 {
        return (pdm, mdm, pdi, mdi, adx, adxr);
    }
    let p = period as f64;

    // --- +DM/-DM/TR 的 Wilder 平滑流（种子 = 前 period-1 个 DM1/TR1 之和）---
    // Seed = sum of the first `period-1` DM1/TR1 values, then Wilder recursion.
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
        // 该柱的真实波幅（使用前一收盘价）。/ True Range at this bar (uses previous close).
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
    // 种子值置于索引 `period-1`（首个有效 +DM/-DM/TR）。
    // Seed placed at index `period-1` (first valid ±DM/TR).
    pdm[period - 1] = prev_plus_dm;
    mdm[period - 1] = prev_minus_dm;
    tr[period - 1] = prev_tr;
    // 后续 Wilder 递推（prev -= prev/period (+= dm1)）。
    // Subsequent Wilder recursion (prev -= prev/period (+= dm1)).
    // 注意：`today += 1` 后再写入 `pdm[today]`，故循环条件须为 `today < n - 1`，
    // 否则最后一轮会把 `today` 推到 `n` 越界。
    // Note: `today += 1` precedes the `pdm[today]` write, so the guard must stay `today < n - 1`.
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

    // --- +DI/-DI（= 100*DM/TR），首个有效于 `period`（比 DM 多递推一步）。---
    // +DI/-DI (= 100·DM/TR), first valid at `period` (one Wilder step after DM).
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

    // --- DX / ADX ---
    // DX[i] = 100*|pdi-mdi|/(pdi+mdi)，首个有效于 `period`。
    // ADX：种子 = 前 period 个 DX 的均值（位于 `2*period-1`），其后 Wilder 递推。
    let dx_seed_end = 2 * period - 1;
    if n > dx_seed_end {
        let mut sum_dx = 0.0_f64;
        for i in period..=dx_seed_end {
            let denom = pdi[i] + mdi[i];
            if denom != 0.0 {
                sum_dx += 100.0 * (pdi[i] - mdi[i]).abs() / denom;
            }
        }
        let mut prev_adx = sum_dx / p;
        adx[dx_seed_end] = prev_adx;
        for i in (dx_seed_end + 1)..n {
            let denom = pdi[i] + mdi[i];
            let dx = if denom == 0.0 {
                0.0
            } else {
                100.0 * (pdi[i] - mdi[i]).abs() / denom
            };
            prev_adx = (prev_adx * (p - 1.0) + dx) / p;
            adx[i] = prev_adx;
        }
    }

    // --- ADXR = (ADX[i] + ADX[i-(period-1)]) / 2，首个有效于 `3*period-2`。---
    // ADXR = (ADX[i] + ADX[i-(period-1)]) / 2, first valid at `3*period-2`.
    let adxr_start = 3 * period - 2;
    for i in adxr_start..n {
        let earlier = i - (period - 1);
        if !adx[i].is_nan() && !adx[earlier].is_nan() {
            adxr[i] = (adx[i] + adx[earlier]) / 2.0;
        }
    }

    (pdm, mdm, pdi, mdi, adx, adxr)
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
    Ok(directional(high, low, close, time_period).0)
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
    Ok(directional(high, low, close, time_period).1)
}

/// `minus_dm` 便捷版本，默认周期 14。/ `minus_dm` with default period (14).
pub fn minus_dm_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    minus_dm(high, low, close, ATR_PERIOD)
}

/// 正方向性指标（TA-Lib `TA_PLUS_DI`）。前导 `period-1` 个为 [`f64::NAN`]。
pub fn plus_di(high: &[f64], low: &[f64], close: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "plus_di")?;
    Ok(directional(high, low, close, time_period).2)
}

/// `plus_di` 便捷版本，默认周期 14。/ `plus_di` with default period (14).
pub fn plus_di_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    plus_di(high, low, close, ATR_PERIOD)
}

/// 负方向性指标（TA-Lib `TA_MINUS_DI`）。前导 `period-1` 个为 [`f64::NAN`]。
pub fn minus_di(high: &[f64], low: &[f64], close: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "minus_di")?;
    Ok(directional(high, low, close, time_period).3)
}

/// `minus_di` 便捷版本，默认周期 14。/ `minus_di` with default period (14).
pub fn minus_di_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    minus_di(high, low, close, ATR_PERIOD)
}

/// 平均方向性运动指数（TA-Lib `TA_ADX`）。前导 `2*period-1` 个为 [`f64::NAN`]。
pub fn adx(high: &[f64], low: &[f64], close: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "adx")?;
    Ok(directional(high, low, close, time_period).4)
}

/// `adx` 便捷版本，默认周期 14。/ `adx` with default period (14).
pub fn adx_default(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, TaError> {
    adx(high, low, close, ATR_PERIOD)
}

/// 平均方向性运动指数评级（TA-Lib `TA_ADXR`）。前导 `3*period-2` 个为 [`f64::NAN`]。
pub fn adxr(high: &[f64], low: &[f64], close: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[high, low, close], "adxr")?;
    Ok(directional(high, low, close, time_period).5)
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
    let period = time_period as i64;
    // TRUE up/down from the canonical streaming algorithm (before the build's swap).
    let mut true_up = vec![f64::NAN; n];
    let mut true_down = vec![f64::NAN; n];
    if n <= time_period {
        // Return swapped-but-empty so `up`/`down` still line up with the contract.
        return Ok(Aroon {
            up: true_down,
            down: true_up,
        });
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
    Ok(Aroon {
        up: true_down,
        down: true_up,
    })
}

/// `aroon` 便捷版本，默认周期 14。/ `aroon` with default period (14).
pub fn aroon_default(high: &[f64], low: &[f64]) -> Result<Aroon, TaError> {
    aroon(high, low, AROON_PERIOD)
}

/// 阿隆震荡器（TA-Lib `TA_AROONOSC`）。基准构建中 `AROONOSC = 返回的上轨 - 返回的下轨`
/// （由于 `aroon` 的 up/down 已按基准构建互换，故本值等于真实 `down - up`，与黄金向量一致）。
pub fn aroon_osc(high: &[f64], low: &[f64], time_period: usize) -> Result<Vec<f64>, TaError> {
    let a = aroon(high, low, time_period)?;
    // `aroon` 返回的是按基准构建互换后的 up/down（见其文档），故此处用 `down - up`
    // 还原真实 `AROONOSC = 真实上轨 - 真实下轨 = TRUE_up - TRUE_down`，与黄金向量一致。
    let out: Vec<f64> = a
        .down
        .iter()
        .zip(&a.up)
        .map(|(d, u)| {
            if u.is_nan() || d.is_nan() {
                f64::NAN
            } else {
                d - u
            }
        })
        .collect();
    Ok(out)
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
    let mut fastk = vec![f64::NAN; n];
    for i in (fast_k_period - 1)..n {
        let mut hh = high[i];
        let mut ll = low[i];
        for j in 1..fast_k_period {
            if high[i - j] > hh {
                hh = high[i - j];
            }
            if low[i - j] < ll {
                ll = low[i - j];
            }
        }
        fastk[i] = if hh == ll {
            0.0
        } else {
            100.0 * (close[i] - ll) / (hh - ll)
        };
    }
    let mut slow_k = rolling_mean_skip(&fastk, slow_k_period);
    let slow_d = rolling_mean_skip(&slow_k, slow_d_period);
    // 三数组对齐到同一前导不稳定期（lookback = fastK+slowK+slowD-3），见 ADR 0007。
    // Align all three arrays to the same leading unstable period (lookback = fastK+slowK+slowD-3).
    let lookback = fast_k_period + slow_k_period + slow_d_period - 3;
    for i in 0..lookback.min(n) {
        slow_k[i] = f64::NAN;
    }
    Ok(Stoch { slow_k, slow_d })
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
    let mut fastk = vec![f64::NAN; n];
    for i in (fast_k_period - 1)..n {
        let mut hh = high[i];
        let mut ll = low[i];
        for j in 1..fast_k_period {
            if high[i - j] > hh {
                hh = high[i - j];
            }
            if low[i - j] < ll {
                ll = low[i - j];
            }
        }
        fastk[i] = if hh == ll {
            0.0
        } else {
            100.0 * (close[i] - ll) / (hh - ll)
        };
    }
    let fast_d = rolling_mean_skip(&fastk, fast_d_period);
    // 两数组对齐到同一前导不稳定期（lookback = fastK+fastD-2），见 ADR 0007。
    // Align both arrays to the same leading unstable period (lookback = fastK+fastD-2).
    let lookback = fast_k_period + fast_d_period - 2;
    for i in 0..lookback.min(n) {
        fastk[i] = f64::NAN;
    }
    Ok(StochF {
        fast_k: fastk,
        fast_d,
    })
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
    let e1 = ema(values, time_period);
    let e2 = ema(&e1, time_period);
    let e3 = ema(&e2, time_period);
    let n = values.len();
    let lookback = 3 * time_period - 2;
    let mut out = vec![f64::NAN; n];
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
