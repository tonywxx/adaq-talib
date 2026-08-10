//! 周期类指标（希尔伯特变换，Hilbert Transform / Cycle）。
//!
//! Cycle indicators. These use John Ehlers' Hilbert-transform machinery (a
//! weighted-moving-average price smoother plus an even/odd circular-buffer
//! Hilbert transformer) to extract the dominant market cycle. This module
//! implements the following TA-Lib 0.7.1 functions (numeric 1:1):
//!
//! - [`mama`] / [`mama_default`] — MESA 自适应移动平均及其 FAMA / MAMA (with FAMA)
//! - [`ht_trendline`] / [`ht_trendline_default`] — 希尔伯特趋势线 / Hilbert Trendline
//!
//! 数值逐项对齐 TA-Lib 0.7.1；本实现逐行移植官方 C 源码（`ta_MAMA.c` /
//! `ta_HT_TRENDLINE.c`，main 分支）以保证位级一致。
//!
//! Numeric 1:1 with TA-Lib 0.7.1; ported line-for-line from the official C
//! sources so the outputs are bit-close to the reference.

use crate::core::defaults::{MAMA_FAST_LIMIT, MAMA_SLOW_LIMIT};
use crate::error::TaError;

// ───────────────────────── 共享希尔伯特核心 ─────────────────────────
//
// MAMA 与 HT_TRENDLINE 共用同一套「价格平滑器（周期 4 的 WMA）+ 希尔伯特变换」
// 状态机。两者仅在尾部（MAMA 计算自适应 alpha 与 mama/fama；HT_TRENDLINE 计算
// 主导周期与趋势线）不同，故把公共部分抽成内部 `Hilbert` 结构体逐 bar 推进。
//
// Both MAMA and HT_TRENDLINE share the same price-smoother (a period-4 WMA) and
// Hilbert-transform state machine; they differ only in their tails, so the
// common part is factored into the private `Hilbert` struct and advanced per bar.

/// 希尔伯特变换 + 价格平滑器的内部状态机。
/// Internal Hilbert-transform + price-smoother state machine.
struct Hilbert {
    // —— 价格平滑器（周期 4 的 WMA） / WMA price smoother (period 4) ——
    period_wma_sub: f64,
    period_wma_sum: f64,
    trailing_wma_value: f64,
    smoothed_value: f64,
    trailing_wma_idx: usize,

    // —— 希尔伯特变换常量 / Hilbert constants ——
    a: f64,
    b: f64,
    rad2deg: f64,
    hilbert_idx: usize,

    // detrender 的奇/偶环形缓冲与延迟输入 / detrender odd/even ring buffers
    detrender_odd: [f64; 3],
    detrender_even: [f64; 3],
    detrender: f64,
    prev_detrender_odd: f64,
    prev_detrender_even: f64,
    prev_detrender_input_odd: f64,
    prev_detrender_input_even: f64,

    // Q1
    q1_odd: [f64; 3],
    q1_even: [f64; 3],
    q1: f64,
    prev_q1_odd: f64,
    prev_q1_even: f64,
    prev_q1_input_odd: f64,
    prev_q1_input_even: f64,

    // jI
    ji_odd: [f64; 3],
    ji_even: [f64; 3],
    ji: f64,
    prev_ji_odd: f64,
    prev_ji_even: f64,
    prev_ji_input_odd: f64,
    prev_ji_input_even: f64,

    // jQ
    jq_odd: [f64; 3],
    jq_even: [f64; 3],
    jq: f64,
    prev_jq_odd: f64,
    prev_jq_even: f64,
    prev_jq_input_odd: f64,
    prev_jq_input_even: f64,

    // 周期估计 / dominant-cycle period estimation
    period: f64,
    q2: f64,
    i2: f64,
    prev_q2: f64,
    prev_i2: f64,
    re: f64,
    im: f64,

    // I1（detrender 延迟 3 根 K 线）的奇/偶历史 / delayed-detrender histories
    i1_for_odd_prev2: f64,
    i1_for_odd_prev3: f64,
    i1_for_even_prev2: f64,
    i1_for_even_prev3: f64,

    // MAMA 特有 / MAMA-only
    prev_phase: f64,
    mama: f64,
    fama: f64,

    // 当前 bar 的原始输入 / current raw input
    today_value: f64,
}

impl Hilbert {
    fn new() -> Self {
        // a = 0.0962, b = 0.5769（Ehlers 系数）。
        // rad2deg = 180/π，与 C 源码 `180.0/(4.0*atan(1))` 等价。
        Hilbert {
            period_wma_sub: 0.0,
            period_wma_sum: 0.0,
            trailing_wma_value: 0.0,
            smoothed_value: 0.0,
            trailing_wma_idx: 0,
            a: 0.0962,
            b: 0.5769,
            rad2deg: 180.0 / (4.0 * (1.0_f64).atan()),
            hilbert_idx: 0,
            detrender_odd: [0.0; 3],
            detrender_even: [0.0; 3],
            detrender: 0.0,
            prev_detrender_odd: 0.0,
            prev_detrender_even: 0.0,
            prev_detrender_input_odd: 0.0,
            prev_detrender_input_even: 0.0,
            q1_odd: [0.0; 3],
            q1_even: [0.0; 3],
            q1: 0.0,
            prev_q1_odd: 0.0,
            prev_q1_even: 0.0,
            prev_q1_input_odd: 0.0,
            prev_q1_input_even: 0.0,
            ji_odd: [0.0; 3],
            ji_even: [0.0; 3],
            ji: 0.0,
            prev_ji_odd: 0.0,
            prev_ji_even: 0.0,
            prev_ji_input_odd: 0.0,
            prev_ji_input_even: 0.0,
            jq_odd: [0.0; 3],
            jq_even: [0.0; 3],
            jq: 0.0,
            prev_jq_odd: 0.0,
            prev_jq_even: 0.0,
            prev_jq_input_odd: 0.0,
            prev_jq_input_even: 0.0,
            period: 0.0,
            q2: 0.0,
            i2: 0.0,
            prev_q2: 0.0,
            prev_i2: 0.0,
            re: 0.0,
            im: 0.0,
            i1_for_odd_prev2: 0.0,
            i1_for_odd_prev3: 0.0,
            i1_for_even_prev2: 0.0,
            i1_for_even_prev3: 0.0,
            prev_phase: 0.0,
            mama: 0.0,
            fama: 0.0,
            today_value: 0.0,
        }
    }

    /// 初始化价格平滑器（WMA 预热）。返回主循环的起始绝对索引 `first_main`。
    /// Prime the WMA price smoother. Returns the starting absolute index of the
    /// main loop (`first_main`). `wma_init_iters` is the count of the unrolled
    /// loop: 9 for MAMA (lookback 32), 34 for HT_TRENDLINE (lookback 63).
    fn init(&mut self, values: &[f64], lookback_total: usize, wma_init_iters: usize) -> usize {
        // TA-Lib 先把 startIdx 钳到 lookbackTotal（我们的 startIdx 恒为 0）。
        // TA-Lib clamps startIdx up to lookbackTotal (our startIdx is always 0).
        let start_idx = if 0 < lookback_total { lookback_total } else { 0 };
        self.trailing_wma_idx = start_idx - lookback_total; // = 0
        let mut today = self.trailing_wma_idx;

        // 与 C 的 WMA 初始化一致：前 3 个值手工展开，随后 do-while 循环。
        // Mirrors the C WMA init: 3 unrolled reads, then the do-while loop.
        let mut t = values[today];
        today += 1;
        self.period_wma_sub = t;
        self.period_wma_sum = t;

        t = values[today];
        today += 1;
        self.period_wma_sub += t;
        self.period_wma_sum += t * 2.0;

        t = values[today];
        today += 1;
        self.period_wma_sub += t;
        self.period_wma_sum += t * 3.0;

        self.trailing_wma_value = 0.0;

        let mut i = wma_init_iters;
        loop {
            t = values[today];
            today += 1;
            self.period_wma_sub += t;
            self.period_wma_sub -= self.trailing_wma_value;
            self.period_wma_sum += t * 4.0;
            self.trailing_wma_value = values[self.trailing_wma_idx];
            self.trailing_wma_idx += 1;
            self.smoothed_value = self.period_wma_sum * 0.1;
            self.period_wma_sum -= self.period_wma_sub;
            i -= 1;
            if i == 0 {
                break;
            }
        }
        today // = first_main
    }

    /// 推进一 bar：更新 WMA 平滑器，运行奇偶希尔伯特变换，返回相位 `phase`
    /// （`tempReal2`，供 MAMA 计算自适应 alpha）。
    /// Advance one bar: update the WMA smoother, run the parity Hilbert
    /// transform, and return the phase (`tempReal2`, used by MAMA for the
    /// adaptive alpha).
    fn step(&mut self, values: &[f64], today: usize, today_value: f64) -> f64 {
        // adjustedPrevPeriod = 0.075*period + 0.54（用上一 bar 的 period）。
        let adjusted_prev_period = 0.075 * self.period + 0.54;
        self.today_value = today_value;

        // —— WMA 平滑器步进（与 C 的 DO_PRICE_WMA 一致） ——
        self.period_wma_sub += today_value;
        self.period_wma_sub -= self.trailing_wma_value;
        self.period_wma_sum += today_value * 4.0;
        self.trailing_wma_value = values[self.trailing_wma_idx];
        self.trailing_wma_idx += 1;
        self.smoothed_value = self.period_wma_sum * 0.1;
        self.period_wma_sum -= self.period_wma_sub;

        let phase = if today % 2 == 0 {
            // ── 偶数 bar 的希尔伯特变换（使用 Even 缓冲） ──
            let mut h = self.a * self.smoothed_value;
            self.detrender = 0.0 - self.detrender_even[self.hilbert_idx];
            self.detrender_even[self.hilbert_idx] = h;
            self.detrender += h;
            self.detrender -= self.prev_detrender_even;
            self.prev_detrender_even = self.b * self.prev_detrender_input_even;
            self.detrender += self.prev_detrender_even;
            self.prev_detrender_input_even = self.smoothed_value;
            self.detrender *= adjusted_prev_period;

            h = self.a * self.detrender;
            self.q1 = 0.0 - self.q1_even[self.hilbert_idx];
            self.q1_even[self.hilbert_idx] = h;
            self.q1 += h;
            self.q1 -= self.prev_q1_even;
            self.prev_q1_even = self.b * self.prev_q1_input_even;
            self.q1 += self.prev_q1_even;
            self.prev_q1_input_even = self.detrender;
            self.q1 *= adjusted_prev_period;

            h = self.a * self.i1_for_even_prev3;
            self.ji = 0.0 - self.ji_even[self.hilbert_idx];
            self.ji_even[self.hilbert_idx] = h;
            self.ji += h;
            self.ji -= self.prev_ji_even;
            self.prev_ji_even = self.b * self.prev_ji_input_even;
            self.ji += self.prev_ji_even;
            self.prev_ji_input_even = self.i1_for_even_prev3;
            self.ji *= adjusted_prev_period;

            h = self.a * self.q1;
            self.jq = 0.0 - self.jq_even[self.hilbert_idx];
            self.jq_even[self.hilbert_idx] = h;
            self.jq += h;
            self.jq -= self.prev_jq_even;
            self.prev_jq_even = self.b * self.prev_jq_input_even;
            self.jq += self.prev_jq_even;
            self.prev_jq_input_even = self.q1;
            self.jq *= adjusted_prev_period;

            self.hilbert_idx += 1;
            if self.hilbert_idx == 3 {
                self.hilbert_idx = 0;
            }

            self.q2 = 0.2 * (self.q1 + self.ji) + 0.8 * self.prev_q2;
            self.i2 = 0.2 * (self.i1_for_even_prev3 - self.jq) + 0.8 * self.prev_i2;

            // 把当前 detrender 存给「奇数」分支 3 根后使用。
            // Save current detrender for the odd branch 3 bars later.
            self.i1_for_odd_prev3 = self.i1_for_odd_prev2;
            self.i1_for_odd_prev2 = self.detrender;

            let denom = self.i1_for_even_prev3;
            if denom != 0.0 {
                (self.q1 / denom).atan() * self.rad2deg
            } else {
                0.0
            }
        } else {
            // ── 奇数 bar 的希尔伯特变换（使用 Odd 缓冲） ──
            let mut h = self.a * self.smoothed_value;
            self.detrender = 0.0 - self.detrender_odd[self.hilbert_idx];
            self.detrender_odd[self.hilbert_idx] = h;
            self.detrender += h;
            self.detrender -= self.prev_detrender_odd;
            self.prev_detrender_odd = self.b * self.prev_detrender_input_odd;
            self.detrender += self.prev_detrender_odd;
            self.prev_detrender_input_odd = self.smoothed_value;
            self.detrender *= adjusted_prev_period;

            h = self.a * self.detrender;
            self.q1 = 0.0 - self.q1_odd[self.hilbert_idx];
            self.q1_odd[self.hilbert_idx] = h;
            self.q1 += h;
            self.q1 -= self.prev_q1_odd;
            self.prev_q1_odd = self.b * self.prev_q1_input_odd;
            self.q1 += self.prev_q1_odd;
            self.prev_q1_input_odd = self.detrender;
            self.q1 *= adjusted_prev_period;

            h = self.a * self.i1_for_odd_prev3;
            self.ji = 0.0 - self.ji_odd[self.hilbert_idx];
            self.ji_odd[self.hilbert_idx] = h;
            self.ji += h;
            self.ji -= self.prev_ji_odd;
            self.prev_ji_odd = self.b * self.prev_ji_input_odd;
            self.ji += self.prev_ji_odd;
            self.prev_ji_input_odd = self.i1_for_odd_prev3;
            self.ji *= adjusted_prev_period;

            h = self.a * self.q1;
            self.jq = 0.0 - self.jq_odd[self.hilbert_idx];
            self.jq_odd[self.hilbert_idx] = h;
            self.jq += h;
            self.jq -= self.prev_jq_odd;
            self.prev_jq_odd = self.b * self.prev_jq_input_odd;
            self.jq += self.prev_jq_odd;
            self.prev_jq_input_odd = self.q1;
            self.jq *= adjusted_prev_period;

            self.hilbert_idx += 1;
            if self.hilbert_idx == 3 {
                self.hilbert_idx = 0;
            }

            self.q2 = 0.2 * (self.q1 + self.ji) + 0.8 * self.prev_q2;
            self.i2 = 0.2 * (self.i1_for_odd_prev3 - self.jq) + 0.8 * self.prev_i2;

            // 把当前 detrender 存给「偶数」分支 3 根后使用。
            self.i1_for_even_prev3 = self.i1_for_even_prev2;
            self.i1_for_even_prev2 = self.detrender;

            let denom = self.i1_for_odd_prev3;
            if denom != 0.0 {
                (self.q1 / denom).atan() * self.rad2deg
            } else {
                0.0
            }
        };
        phase
    }

    /// 更新 Re/Im 与主导周期（含 [6,50] 钳制与平滑）。MAMA 在尾部之后、HT 在尾部之前调用。
    /// Update Re/Im and the dominant cycle `period` (with [6,50] clamping and
    /// smoothing). Called after the MAMA tail, before the HT_TRENDLINE tail.
    fn update_period(&mut self) {
        self.re = 0.8 * self.re + 0.2 * (self.i2 * self.prev_i2 + self.q2 * self.prev_q2);
        self.im = 0.8 * self.im + 0.2 * (self.i2 * self.prev_q2 - self.q2 * self.prev_i2);
        self.prev_q2 = self.q2;
        self.prev_i2 = self.i2;

        let temp_real = self.period;
        if self.im != 0.0 && self.re != 0.0 {
            self.period = 360.0 / ((self.im / self.re).atan() * self.rad2deg);
        }
        let hi = 1.5 * temp_real;
        if self.period > hi {
            self.period = hi;
        }
        let lo = 0.67 * temp_real;
        if self.period < lo {
            self.period = lo;
        }
        if self.period < 6.0 {
            self.period = 6.0;
        } else if self.period > 50.0 {
            self.period = 50.0;
        }
        self.period = 0.2 * self.period + 0.8 * temp_real;
    }
}

// ───────────────────────────── MAMA ─────────────────────────────

/// MESA 自适应移动平均结果（含 FAMA）。三向量等长。
/// MESA Adaptive Moving Average result (including FAMA). All three vectors are
/// equal-length.
pub struct Mama {
    /// MAMA 主线（自适应 EMA）。/ MAMA line (adaptive EMA).
    pub mama: Vec<f64>,
    /// FAMA（MAMA 的平滑版，alpha 减半）。/ FAMA (smoothed MAMA, half alpha).
    pub fama: Vec<f64>,
}

/// MESA 自适应移动平均（MAMA，TA-Lib `TA_MAMA`）。
///
/// MESA Adaptive Moving Average (MAMA). Produces both `mama` and its smoother
/// companion `fama`. Lookback is 32; the first 32 positions are [`f64::NAN`].
///
/// # 参数 / Parameters
/// - `values`：输入序列（收盘价等）`&[f64]`。/ Input series `&[f64]`.
/// - `fast_limit`：自适应 alpha 上限（TA-Lib 默认 0.5，须 ∈ [0.01, 0.99]）。
/// - `slow_limit`：自适应 alpha 下限（TA-Lib 默认 0.05，须 ∈ [0.01, 0.99]）。
///
/// # 返回值 / Returns
/// [`Mama`]（`mama` 与 `fama` 等长向量，前导 32 个为 [`f64::NAN`]）。
///
/// # 错误 / Errors
/// - [`TaError::BadParam`]：`fast_limit` / `slow_limit` 不在 [0.01, 0.99]。
pub fn mama(values: &[f64], fast_limit: f64, slow_limit: f64) -> Result<Mama, TaError> {
    if !(0.01..=0.99).contains(&fast_limit) {
        return Err(TaError::BadParam(
            "mama: fast_limit must be within [0.01, 0.99]".into(),
        ));
    }
    if !(0.01..=0.99).contains(&slow_limit) {
        return Err(TaError::BadParam(
            "mama: slow_limit must be within [0.01, 0.99]".into(),
        ));
    }
    let n = values.len();
    let lookback = 32;
    let mut out_mama = vec![f64::NAN; n];
    let mut out_fama = vec![f64::NAN; n];
    if n <= lookback {
        return Ok(Mama {
            mama: out_mama,
            fama: out_fama,
        });
    }
    let mut h = Hilbert::new();
    let first_main = h.init(values, lookback, 9); // MAMA 的 WMA 预热循环次数 = 9
    let mut today = first_main;
    while today <= n - 1 {
        let phase = h.step(values, today, values[today]);
        // 自适应 alpha（与 C 的 delta-phase → alpha 推导一致）。
        let delta_phase = h.prev_phase - phase;
        h.prev_phase = phase;
        let mut alpha = delta_phase;
        if alpha < 1.0 {
            alpha = 1.0;
        }
        if alpha > 1.0 {
            alpha = fast_limit / alpha;
            if alpha < slow_limit {
                alpha = slow_limit;
            }
        } else {
            alpha = fast_limit;
        }
        h.mama = (1.0 - alpha) * h.mama + alpha * h.today_value;
        let alpha2 = alpha * 0.5;
        h.fama = (1.0 - alpha2) * h.fama + alpha2 * h.mama;
        h.update_period();
        if today >= lookback {
            // 等长返回：输出写在绝对索引 `today`（≥ lookback），前导保持 NaN。
            // Equal-length return: write at absolute index `today` (>= lookback);
            // the leading positions keep their NaN.
            out_mama[today] = h.mama;
            out_fama[today] = h.fama;
        }
        today += 1;
    }
    Ok(Mama {
        mama: out_mama,
        fama: out_fama,
    })
}

/// MAMA，使用 TA-Lib 默认参数（fast 0.5 / slow 0.05）。
/// MAMA with TA-Lib defaults (fast 0.5 / slow 0.05).
pub fn mama_default(values: &[f64]) -> Result<Mama, TaError> {
    mama(values, MAMA_FAST_LIMIT, MAMA_SLOW_LIMIT)
}

// ─────────────────────────── HT_TRENDLINE ───────────────────────────

/// 希尔伯特趋势线（Hilbert Trendline，TA-Lib `TA_HT_TRENDLINE`）。
///
/// Hilbert Trendline. Uses the same Hilbert transform to estimate the dominant
/// cycle period, then averages the raw price over that period and smooths it
/// with a 4-term weighted filter. Lookback is 63; the first 63 positions are
/// [`f64::NAN`].
///
/// # 参数 / Parameters
/// - `values`：输入序列（收盘价等）`&[f64]`。/ Input series `&[f64]`.
///
/// # 返回值 / Returns
/// 与 `values` 等长的向量，前导 63 个为 [`f64::NAN`]。
/// Equal-length vector; the first 63 positions are [`f64::NAN`].
pub fn ht_trendline(values: &[f64]) -> Result<Vec<f64>, TaError> {
    let n = values.len();
    let lookback = 63;
    let mut out = vec![f64::NAN; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut h = Hilbert::new();
    let first_main = h.init(values, lookback, 34); // HT_TRENDLINE 的 WMA 预热循环次数 = 34
    let mut today = first_main;
    let mut i_trend1 = 0.0_f64;
    let mut i_trend2 = i_trend1;
    let mut i_trend3 = i_trend2;
    let mut smooth_period = 0.0_f64;
    while today <= n - 1 {
        h.step(values, today, values[today]);
        h.update_period();
        smooth_period = 0.67 * smooth_period + 0.33 * h.period;
        // 主导周期（取整）对应的原始价格均值窗口。
        // Dominant-cycle-period (truncated) raw-price averaging window.
        let dc_period = smooth_period + 0.5;
        let dc_period_int = dc_period as i32; // 截断，等价于 C 的 (int)
        let mut sum = 0.0_f64;
        for i in 0..50 {
            // TA-Lib 此处会越界读取（today-i<0），但仅发生在 lookback 之前的预热
            // 段、且其结果被丢弃；安全起见此处跳过负索引项，对正常数据等价。
            // TA-Lib reads out-of-bounds here (today-i<0) but only during the
            // pre-output warmup where the value is discarded; skip negative
            // indices for safety — identical for normal data.
            if i < dc_period_int && today >= i as usize {
                sum += values[today - i as usize];
            }
        }
        if dc_period_int > 0 {
            sum /= dc_period_int as f64;
        }
        // 4 项加权趋势线：trend = (2*ITrend2 + 4*priceAvg + 3*ITrend1 + ITrend3)/10
        let trend = (2.0 * i_trend2 + 4.0 * sum + 3.0 * i_trend1 + i_trend3) / 10.0;
        i_trend3 = i_trend2;
        i_trend2 = i_trend1;
        i_trend1 = sum;
        if today >= lookback {
            // 等长返回：输出写在绝对索引 `today`（≥ lookback）。/ equal-length.
            out[today] = trend;
        }
        today += 1;
    }
    Ok(out)
}

/// 希尔伯特趋势线，使用 TA-Lib 默认参数（无可选参数）。
/// Hilbert Trendline with TA-Lib defaults (no optional inputs).
pub fn ht_trendline_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    ht_trendline(values)
}

// ──────────────────────────── 单元测试 ────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mama_constant_input_converges() {
        // 常数输入：MAMA/FAMA 在预热后应逼近该常数（EMA 收敛）。
        // Constant input: MAMA/FAMA should converge to the constant after warmup.
        let v = vec![1.0; 200];
        let out = mama_default(&v).unwrap();
        assert_eq!(out.mama.len(), 200);
        assert_eq!(out.fama.len(), 200);
        // 前 32 个为 NaN / first 32 are NaN.
        for i in 0..32 {
            assert!(out.mama[i].is_nan(), "mama[{i}] should be NaN");
            assert!(out.fama[i].is_nan(), "fama[{i}] should be NaN");
        }
        // 收敛后接近 1.0 / converges to 1.0.
        for i in 100..200 {
            assert!((out.mama[i] - 1.0).abs() < 1e-9, "mama[{i}] = {}", out.mama[i]);
            assert!((out.fama[i] - 1.0).abs() < 1e-9, "fama[{i}] = {}", out.fama[i]);
        }
    }

    #[test]
    fn mama_invalid_limits() {
        let v = vec![1.0; 200];
        assert!(matches!(
            mama(&v, 0.0, 0.05),
            Err(TaError::BadParam(_))
        ));
        assert!(matches!(
            mama(&v, 0.5, 1.0),
            Err(TaError::BadParam(_))
        ));
    }

    #[test]
    fn mama_short_input_all_nan() {
        let v = vec![1.0, 2.0, 3.0];
        let out = mama_default(&v).unwrap();
        assert!(out.mama.iter().all(|x| x.is_nan()));
        assert!(out.fama.iter().all(|x| x.is_nan()));
    }

    #[test]
    fn ht_trendline_constant_input() {
        let v = vec![2.0; 200];
        let out = ht_trendline_default(&v).unwrap();
        assert_eq!(out.len(), 200);
        for i in 0..63 {
            assert!(out[i].is_nan(), "ht_trendline[{i}] should be NaN");
        }
        for i in 120..200 {
            assert!((out[i] - 2.0).abs() < 1e-9, "ht_trendline[{i}] = {}", out[i]);
        }
    }

    #[test]
    fn ht_trendline_short_input_all_nan() {
        let v = vec![1.0, 2.0, 3.0];
        let out = ht_trendline_default(&v).unwrap();
        assert!(out.iter().all(|x| x.is_nan()));
    }
}
