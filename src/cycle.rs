//! 周期类指标（希尔伯特变换，Hilbert Transform / Cycle）。
//!
//! Cycle indicators. These use John Ehlers' Hilbert-transform machinery (a
//! weighted-moving-average price smoother plus an even/odd circular-buffer
//! Hilbert transformer) to extract the dominant market cycle. This module
//! implements the following TA-Lib 0.7.1 functions (numeric 1:1):
//!
//! - [`mama`] / [`mama_default`] — MESA 自适应移动平均及其 FAMA / MAMA (with FAMA)
//! - [`ht_trendline`] / [`ht_trendline_default`] — 希尔伯特趋势线 / Hilbert Trendline
//! - [`ht_dcperiod`] / [`ht_dcperiod_default`] — 希尔伯特主导周期 / Dominant Cycle Period
//! - [`ht_dcphase`] / [`ht_dcphase_default`] — 希尔伯特主导周期相位 / Dominant Cycle Phase
//! - [`ht_phasor`] / [`ht_phasor_default`] — 希尔伯特相量（同相/正交）/ Phasor (in-phase/quadrature)
//! - [`ht_sine`] / [`ht_sine_default`] — 希尔伯特正弦波 / SineWave
//! - [`ht_trendmode`] / [`ht_trendmode_default`] — 希尔伯特趋势模态 / Trend vs Cycle Mode
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

    // —— 周期主导 / 相位状态（DCPERIOD/DCPHASE/SINE/TRENDMODE 共用） ——
    // Dominant-cycle / phase state (shared by the period-based HT_* functions).
    smooth_period: f64,
    // 最近 50 个平滑价的环形缓冲（Ehlers DC 相位窗）。
    // Circular buffer of the last 50 smoothed prices (Ehlers DC-phase window).
    smooth_price: [f64; 50],
    smooth_price_idx: usize,
    dc_phase: f64,
    // deg2rad = 1/rad2deg = atan(1)/45；const_deg2rad_by360 = 8*atan(1) = 2π。
    deg2rad: f64,
    const_deg2rad_by360: f64,

    // —— TRENDMODE 专属趋势状态 / TRENDMODE-only trend state ——
    sine: f64,
    prev_sine: f64,
    lead_sine: f64,
    prev_lead_sine: f64,
    days_in_trend: i32,
    trend: i32,
    prev_dc_phase: f64,
    i_trend1: f64,
    i_trend2: f64,
    i_trend3: f64,
    trendline: f64,

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
            smooth_period: 0.0,
            smooth_price: [0.0; 50],
            smooth_price_idx: 0,
            dc_phase: 0.0,
            deg2rad: (1.0_f64).atan() / 45.0,
            const_deg2rad_by360: (1.0_f64).atan() * 8.0,
            sine: 0.0,
            prev_sine: 0.0,
            lead_sine: 0.0,
            prev_lead_sine: 0.0,
            days_in_trend: 0,
            trend: 0,
            prev_dc_phase: 0.0,
            i_trend1: 0.0,
            i_trend2: 0.0,
            i_trend3: 0.0,
            trendline: 0.0,
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

    /// 计算主导周期相位（DCPhase）。逐个 bar 推进 `advance_full` 后调用。
    /// 从 `smooth_price` 环形缓冲按主导周期窗做正交/同相累加，得到 `dc_phase`。
    /// 与 C 的 `ta_HT_DCPHASE.c` / `ta_HT_SINE.c` / `ta_HT_TRENDMODE.c` 逐行一致。
    ///
    /// Compute the dominant-cycle phase (DCPhase). Called after each bar's
    /// `advance_full`; accumulates in-phase/quadrature over the dominant-cycle
    /// window from the `smooth_price` circular buffer. Bit-faithful to the C
    /// `ta_HT_DCPHASE.c` / `ta_HT_SINE.c` / `ta_HT_TRENDMODE.c` sources.
    fn compute_dc_phase(&mut self) {
        let dc_period = self.smooth_period + 0.5;
        let dc_period_int = dc_period as i32;
        let mut real_part = 0.0_f64;
        let mut imag_part = 0.0_f64;
        // idx 从「刚刚写入」的位置开始，向历史回退（环形缓冲，容量 50）。
        // idx starts at the just-written slot and walks backward in time.
        let mut idx = self.smooth_price_idx;
        if dc_period_int > 0 {
            // 角度为 `2π·i/P`，用角度加法递推（每个 bar 仅 1 次 sin/cos 求步长，
            // 之后每步 4 次乘加）替代每步 2 次超越函数，速度约 10× 且误差 ~1e-13
            // （远低于 ADR 0005 的 1e-8 容差）。与 C 直接 sin/cos 在数学上等价。
            // Angles are `2π·i/P`; use angle-addition recurrence (one sin/cos per bar
            // for the step, then 4 mults per step) instead of 2 transcendentals per
            // step — ~10× faster, error ~1e-13 (well under the 1e-8 tolerance).
            // Mathematically equivalent to the C direct sin/cos.
            let p = dc_period_int as f64;
            let w = self.const_deg2rad_by360 / p; // = 2π / P
            let cw = w.cos();
            let sw = w.sin();
            let mut s = 0.0_f64; // sin(0)
            let mut c = 1.0_f64; // cos(0)
            let mut i = 0_i32;
            while i < dc_period_int {
                let temp_real2 = self.smooth_price[idx];
                real_part += s * temp_real2;
                imag_part += c * temp_real2;
                // 角度推进一步：sin/cos(i·w + w)。/ advance angle by w.
                let ns = s * cw + c * sw; // sin(θ+w) = sinθ·cosw + cosθ·sinw
                let nc = c * cw - s * sw; // cos(θ+w) = cosθ·cosw − sinθ·sinw
                s = ns;
                c = nc;
                if idx == 0 {
                    idx = 49;
                } else {
                    idx -= 1;
                }
                i += 1;
            }
        }
        let temp_real = imag_part.abs();
        if temp_real > 0.0 {
            self.dc_phase = (real_part / imag_part).atan() * self.rad2deg;
        } else if temp_real <= 0.01 {
            if real_part < 0.0 {
                self.dc_phase -= 90.0;
            } else if real_part > 0.0 {
                self.dc_phase += 90.0;
            }
        }
        self.dc_phase += 90.0;
        // 补偿 WMA 的一 bar 滞后 / compensate one-bar WMA lag.
        self.dc_phase += 360.0 / self.smooth_period;
        if imag_part < 0.0 {
            self.dc_phase += 180.0;
        }
        if self.dc_phase > 315.0 {
            self.dc_phase -= 360.0;
        }
    }

    /// 周期类 HT_* 函数的逐 bar 推进（DCPERIOD/DCPHASE/SINE/TRENDMODE）。
    /// 等价于 C 主循环：WMA 步进 → 希尔伯特变换 → 周期估计 → 平滑周期 →
    /// 写入 smoothPrice 缓冲 → 计算 DCPhase → 计算 sine/leadSine。
    /// 不输出；调用方按 `today >= lookback` 自行取 `smooth_period` / `dc_phase` /
    /// `sine` / `lead_sine`。
    ///
    /// Per-bar advance for the period-based HT_* functions. Mirrors the C main
    /// loop: WMA step → Hilbert transform → period estimation → smooth period →
    /// write `smooth_price` buffer → compute DCPhase → compute sine/leadSine.
    /// Emits nothing; the caller reads `smooth_period` / `dc_phase` / `sine` /
    /// `lead_sine` once `today >= lookback`.
    fn advance_full(&mut self, values: &[f64], today: usize, today_value: f64) {
        self.step(values, today, today_value);
        self.update_period();
        self.smooth_period = 0.67 * self.smooth_period + 0.33 * self.period;
        // 把当前平滑价写入环形缓冲（覆盖 C 的 `smoothPrice[smoothPrice_Idx] = smoothedValue`）。
        // Write the current smoothed price into the circular buffer.
        self.smooth_price[self.smooth_price_idx] = self.smoothed_value;
        self.compute_dc_phase();
        // 计算 sine / leadSine（SINE 输出与 TRENDMODE 穿越检测共用）。
        // Compute sine / leadSine (shared by SINE output and TRENDMODE crossing).
        self.prev_sine = self.sine;
        self.prev_lead_sine = self.lead_sine;
        self.sine = (self.dc_phase * self.deg2rad).sin();
        self.lead_sine = ((self.dc_phase + 45.0) * self.deg2rad).sin();
        // 推进环形缓冲写指针。/ advance the circular-buffer write pointer.
        self.smooth_price_idx += 1;
        if self.smooth_price_idx > 49 {
            self.smooth_price_idx = 0;
        }
    }

    /// 仅推进主导周期（DCPERIOD 专属，省去 DCPhase 的 sin/cos 窗与 smoothPrice 缓冲）。
    ///
    /// HT_DCPERIOD only needs `smooth_period`; it never emits `dc_phase`/`sine`/
    /// `lead_sine`, nor reads the `smooth_price` history. TA-Lib's C `ta_HT_DCPERIOD`
    /// correspondingly does NOT run the DCPhase correlation, so skipping it here
    /// keeps the output bit-identical while removing the dominant per-bar cost
    /// (an O(dominantCycle ≈ 50) `sin`/`cos` loop). This is the single biggest
    /// lever for `ht_dcperiod` and is 1:1 with the reference.
    ///
    /// Advance only the dominant cycle (DCPERIOD-only): no DCPhase `sin`/`cos`
    /// window, no `smooth_price` buffer. Bit-faithful to C `ta_HT_DCPERIOD`.
    fn advance_period_only(&mut self, values: &[f64], today: usize, today_value: f64) {
        self.step(values, today, today_value);
        self.update_period();
        self.smooth_period = 0.67 * self.smooth_period + 0.33 * self.period;
    }

    /// TRENDMODE 的专属趋势状态机（在 `advance_full` 之后调用）。
    /// 依据 sine/leadSine 交叉、`daysInTrend`、DCPhase 变化率与趋势线偏离判定
    /// 整数 `trend`（1=趋势，0=区间）。与 C 的 `ta_HT_TRENDMODE.c` 尾部一致。
    ///
    /// TRENDMODE's trend state machine (call after `advance_full`). Decides the
    /// integer `trend` (1=trend, 0=cycle) from the sine/leadSine crossing,
    /// `daysInTrend`, the DCPhase rate-of-change, and trendline deviation.
    /// Bit-faithful to the tail of `ta_HT_TRENDMODE.c`.
    fn advance_trend(&mut self, values: &[f64], today: usize) {
        // 默认假设趋势。/ assume trend by default.
        self.trend = 1;
        // 由 SineWave 指标线交叉测量趋势持续根数。/ crossing of the SineWave lines.
        let crossed = (self.sine > self.lead_sine && self.prev_sine <= self.prev_lead_sine)
            || (self.sine < self.lead_sine && self.prev_sine >= self.prev_lead_sine);
        if crossed {
            self.days_in_trend = 0;
            self.trend = 0;
        }
        self.days_in_trend += 1;
        if (self.days_in_trend as f64) < 0.5 * self.smooth_period {
            self.trend = 0;
        }
        let temp_real = self.dc_phase - self.prev_dc_phase;
        if self.smooth_period != 0.0
            && temp_real > 0.67 * 360.0 / self.smooth_period
            && temp_real < 1.5 * 360.0 / self.smooth_period
        {
            self.trend = 0;
        }
        // 原始价格按主导周期的均值窗 + 4 项加权趋势线（与 HT_TRENDLINE 同式）。
        // Raw-price dominant-cycle average window + 4-term weighted trendline
        // (same formula as HT_TRENDLINE).
        let dc_period = self.smooth_period + 0.5;
        let dc_period_int = dc_period as i32;
        let mut temp = 0.0_f64;
        for j in 0..50 {
            if j < dc_period_int && today >= j as usize {
                temp += values[today - j as usize];
            }
        }
        if dc_period_int > 0 {
            temp /= dc_period_int as f64;
        }
        self.trendline =
            (2.0 * self.i_trend2 + 4.0 * temp + 3.0 * self.i_trend1 + self.i_trend3) / 10.0;
        self.i_trend3 = self.i_trend2;
        self.i_trend2 = self.i_trend1;
        self.i_trend1 = temp;
        // 平滑价相对趋势线偏离 ≥ 1.5% → 判定为趋势。注意：`advance_full` 已在末尾把
        // `smooth_price_idx` 前移，故此处取「刚写入」的平滑价 = `smoothed_value`（与 C 的
        // `smoothPrice[smoothPrice_Idx]` 等价，该读取发生在 `smoothPrice_Idx++` 之前）。
        // Deviation >= 1.5% => trend. Note: `advance_full` advanced `smooth_price_idx`,
        // so the just-written price is `smoothed_value` (== C's `smoothPrice[smoothPrice_Idx]`,
        // which C reads BEFORE its `smoothPrice_Idx++`).
        let latest_smooth = self.smoothed_value;
        if self.trendline != 0.0
            && ((latest_smooth - self.trendline) / self.trendline).abs() >= 0.015
        {
            self.trend = 1;
        }
        self.prev_dc_phase = self.dc_phase;
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
    let mut out = vec![f64::NAN; values.len()];
    ht_trendline_with_output(values, &mut out)?;
    Ok(out)
}

/// 希尔伯特趋势线，零拷贝写入 `out`（与 `values` 等长，前导 63 个为 [`f64::NAN`]）。
/// 见 [`ht_trendline`]。
///
/// Hilbert Trendline, written zero-copy into `out` (equal-length to `values`;
/// the first 63 positions are [`f64::NAN`]). See [`ht_trendline`].
///
/// `out` 长度必须等于 `values.len()`，否则返回 [`TaError::BadParam`]。
/// `out` must have length equal to `values.len()`; otherwise [`TaError::BadParam`]
/// is returned.
pub fn ht_trendline_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    let n = values.len();
    if out.len() != n {
        return Err(TaError::BadParam(
            "ht_trendline_with_output: out length must equal values length".into(),
        ));
    }
    let lookback = 63;
    if n <= lookback {
        return Ok(());
    }
    // 前缀和：把内层 O(min(50, dc)) 的逐窗求和降到 O(1)（P3-2 性能优化）。窗口为最近
    // `terms` 个价格 [today-terms+1, today]；前缀和相减与原始逐窗求和的数值差异在机器精度
    // 量级（≪ 1e-8 黄金容差），通过黄金向量校验即为 1:1（ADR 0005）。
    let mut prefix = vec![0.0_f64; n];
    if n > 0 {
        prefix[0] = values[0];
        for i in 1..n {
            prefix[i] = prefix[i - 1] + values[i];
        }
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
        // 窗口大小：最多 50 项（TA-Lib 硬编码 0..50），并被 `today` 截断于预热段。
        // Window size: at most 50 terms (TA-Lib hardcodes 0..50), truncated by `today` in warmup.
        let k = if dc_period_int > 50 { 50 } else { dc_period_int };
        let terms = if (today as i32) < k - 1 {
            today as i32 + 1
        } else {
            k
        } as usize;
        let lo = today - terms + 1;
        let mut sum = if lo > 0 {
            prefix[today] - prefix[lo - 1]
        } else {
            prefix[today]
        };
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
    Ok(())
}

/// 希尔伯特趋势线，使用 TA-Lib 默认参数（无可选参数）。
/// Hilbert Trendline with TA-Lib defaults (no optional inputs).
pub fn ht_trendline_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    ht_trendline(values)
}

// ───────────────────────── HT_DCPERIOD ─────────────────────────

/// 希尔伯特主导周期（HT_DCPERIOD，TA-Lib `TA_HT_DCPERIOD`）。
///
/// Hilbert Transform — Dominant Cycle Period. After the shared Hilbert
/// transform and dominant-cycle estimation, outputs the smoothed dominant
/// period `smoothPeriod = 0.67*smoothPeriod + 0.33*period`. Lookback is 32;
/// the first 32 positions are [`f64::NAN`].
///
/// # 参数 / Parameters
/// - `values`：输入序列（收盘价等）`&[f64]`。/ Input series `&[f64]`.
///
/// # 返回值 / Returns
/// 与 `values` 等长的向量，前导 32 个为 [`f64::NAN`]。
/// Equal-length vector; the first 32 positions are [`f64::NAN`].
pub fn ht_dcperiod(values: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; values.len()];
    ht_dcperiod_with_output(values, &mut out)?;
    Ok(out)
}

/// 希尔伯特主导周期，零拷贝写入 `out`（与 `values` 等长，前导 32 个为 [`f64::NAN`]）。
/// 见 [`ht_dcperiod`]。
///
/// Hilbert Transform — Dominant Cycle Period, written zero-copy into `out`
/// (equal-length to `values`; the first 32 positions are [`f64::NAN`]). See
/// [`ht_dcperiod`].
///
/// `out` 长度必须等于 `values.len()`，否则返回 [`TaError::BadParam`]。
/// `out` must have length equal to `values.len()`; otherwise [`TaError::BadParam`]
/// is returned.
pub fn ht_dcperiod_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    let n = values.len();
    if out.len() != n {
        return Err(TaError::BadParam(
            "ht_dcperiod_with_output: out length must equal values length".into(),
        ));
    }
    let lookback = 32;
    if n <= lookback {
        return Ok(());
    }
    let mut h = Hilbert::new();
    let first_main = h.init(values, lookback, 9); // HT_DCPERIOD 的 WMA 预热循环次数 = 9
    let mut today = first_main;
    while today <= n - 1 {
        // DCPERIOD 只需主导周期，跳过 DCPhase 的 sin/cos 窗与 smoothPrice 缓冲。
        // DCPERIOD only needs the dominant cycle — skip the DCPhase machinery.
        h.advance_period_only(values, today, values[today]);
        if today >= lookback {
            out[today] = h.smooth_period;
        }
        today += 1;
    }
    Ok(())
}

/// HT_DCPERIOD，使用 TA-Lib 默认参数（无可选参数）。
/// HT_DCPERIOD with TA-Lib defaults (no optional inputs).
pub fn ht_dcperiod_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    ht_dcperiod(values)
}

// ───────────────────────── HT_DCPHASE ─────────────────────────

/// 希尔伯特主导周期相位（HT_DCPHASE，TA-Lib `TA_HT_DCPHASE`）。
///
/// Hilbert Transform — Dominant Cycle Phase. Computes the `DCPhase` from the
/// smoothed-price circular buffer over the dominant cycle (see `compute_dc_phase`).
/// Lookback is 63; the first 63 positions are [`f64::NAN`].
///
/// # 参数 / Parameters
/// - `values`：输入序列（收盘价等）`&[f64]`。/ Input series `&[f64]`.
///
/// # 返回值 / Returns
/// 与 `values` 等长的向量，前导 63 个为 [`f64::NAN`]。
/// Equal-length vector; the first 63 positions are [`f64::NAN`].
pub fn ht_dcphase(values: &[f64]) -> Result<Vec<f64>, TaError> {
    let mut out = vec![f64::NAN; values.len()];
    ht_dcphase_with_output(values, &mut out)?;
    Ok(out)
}

/// 希尔伯特主导周期相位，零拷贝写入 `out`（与 `values` 等长，前导 63 个为
/// [`f64::NAN`]）。见 [`ht_dcphase`]。
///
/// Hilbert Transform — Dominant Cycle Phase, written zero-copy into `out`
/// (equal-length to `values`; the first 63 positions are [`f64::NAN`]). See
/// [`ht_dcphase`].
///
/// `out` 长度必须等于 `values.len()`，否则返回 [`TaError::BadParam`]。
/// `out` must have length equal to `values.len()`; otherwise [`TaError::BadParam`]
/// is returned.
pub fn ht_dcphase_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    let n = values.len();
    if out.len() != n {
        return Err(TaError::BadParam(
            "ht_dcphase_with_output: out length must equal values length".into(),
        ));
    }
    let lookback = 63;
    if n <= lookback {
        return Ok(());
    }
    let mut h = Hilbert::new();
    let first_main = h.init(values, lookback, 34); // HT_DCPHASE 的 WMA 预热循环次数 = 34
    let mut today = first_main;
    while today <= n - 1 {
        h.advance_full(values, today, values[today]);
        if today >= lookback {
            out[today] = h.dc_phase;
        }
        today += 1;
    }
    Ok(())
}

/// HT_DCPHASE，使用 TA-Lib 默认参数（无可选参数）。
/// HT_DCPHASE with TA-Lib defaults (no optional inputs).
pub fn ht_dcphase_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    ht_dcphase(values)
}

// ───────────────────────── HT_PHASOR ─────────────────────────

/// 希尔伯特变换相量（HT_PHASOR，TA-Lib `TA_HT_PHASOR`）结果。两向量等长。
/// Hilbert Transform Phasor result (in-phase & quadrature). Equal-length.
pub struct HtPhasor {
    /// 同相分量（延迟 3 根 K 线的 detrender，`I1ForPrev3`）。/ In-phase (delayed detrender).
    pub in_phase: Vec<f64>,
    /// 正交分量（`Q1`）。/ Quadrature.
    pub quadrature: Vec<f64>,
}

/// 希尔伯特变换相量（HT_PHASOR，TA-Lib `TA_HT_PHASOR`）。
///
/// Hilbert Transform — Phasor Components. Unlike the other HT_* functions,
/// this does NOT estimate the dominant cycle: it directly emits the in-phase
/// component (`I1ForPrev3`, the detrender delayed 3 bars) and the quadrature
/// component (`Q1`) from the shared Hilbert transform. Lookback is 32; the
/// first 32 positions are [`f64::NAN`].
///
/// # 参数 / Parameters
/// - `values`：输入序列（收盘价等）`&[f64]`。/ Input series `&[f64]`.
///
/// # 返回值 / Returns
/// [`HtPhasor`]（`in_phase` 与 `quadrature` 等长向量，前导 32 个为 [`f64::NAN`]）。
pub fn ht_phasor(values: &[f64]) -> Result<HtPhasor, TaError> {
    let n = values.len();
    let mut out = HtPhasor {
        in_phase: vec![f64::NAN; n],
        quadrature: vec![f64::NAN; n],
    };
    ht_phasor_with_output(values, &mut out)?;
    Ok(out)
}

/// 希尔伯特变换相量，零拷贝写入 `out`（与 `values` 等长的两轨向量，前导 32 个为
/// [`f64::NAN`]）。见 [`ht_phasor`]。
///
/// Hilbert Transform — Phasor Components, written zero-copy into `out` (two
/// equal-length bands; the first 32 positions are [`f64::NAN`]). See
/// [`ht_phasor`].
///
/// `out.in_phase` 与 `out.quadrature` 长度均必须等于 `values.len()`，否则返回
/// [`TaError::BadParam`]。
/// Both `out.in_phase` and `out.quadrature` must have length equal to
/// `values.len()`; otherwise [`TaError::BadParam`] is returned.
pub fn ht_phasor_with_output(values: &[f64], out: &mut HtPhasor) -> Result<(), TaError> {
    let n = values.len();
    if out.in_phase.len() != n || out.quadrature.len() != n {
        return Err(TaError::BadParam(
            "ht_phasor_with_output: out field lengths must equal values length".into(),
        ));
    }
    let lookback = 32;
    if n <= lookback {
        return Ok(());
    }
    let mut h = Hilbert::new();
    let first_main = h.init(values, lookback, 9); // HT_PHASOR 的 WMA 预热循环次数 = 9
    let mut today = first_main;
    while today <= n - 1 {
        let parity_even = today % 2 == 0;
        h.step(values, today, values[today]);
        // PHASOR 同样维护主导周期（影响 `adjustedPrevPeriod` 阻尼），必须调用。
        // PHASOR also maintains the dominant cycle (drives the `adjustedPrevPeriod`
        // damping), so update_period() is required.
        h.update_period();
        let quadrature = h.q1;
        // 同相分量：偶数 bar 取 i1_for_even_prev3，奇数 bar 取 i1_for_odd_prev3。
        // In-phase: even bar -> i1_for_even_prev3, odd bar -> i1_for_odd_prev3.
        let in_phase = if parity_even {
            h.i1_for_even_prev3
        } else {
            h.i1_for_odd_prev3
        };
        if today >= lookback {
            out.in_phase[today] = in_phase;
            out.quadrature[today] = quadrature;
        }
        today += 1;
    }
    Ok(())
}

/// HT_PHASOR，使用 TA-Lib 默认参数（无可选参数）。
/// HT_PHASOR with TA-Lib defaults (no optional inputs).
pub fn ht_phasor_default(values: &[f64]) -> Result<HtPhasor, TaError> {
    ht_phasor(values)
}

// ───────────────────────── HT_SINE ─────────────────────────

/// 希尔伯特正弦波（HT_SINE，TA-Lib `TA_HT_SINE`）结果。两向量等长。
/// Hilbert Transform Sine Wave result (sine & lead sine). Equal-length.
pub struct HtSine {
    /// 正弦波（`sin(DCPhase * deg2rad)`）。/ Sine wave.
    pub sine: Vec<f64>,
    /// 领先正弦波（`sin((DCPhase + 45) * deg2rad)`）。/ Lead sine wave.
    pub lead_sine: Vec<f64>,
}

/// 希尔伯特正弦波（HT_SINE，TA-Lib `TA_HT_SINE`）。
///
/// Hilbert Transform — SineWave. Emits the sine and the 45°-leading sine of
/// the dominant-cycle phase. Lookback is 63; the first 63 positions are
/// [`f64::NAN`].
///
/// # 参数 / Parameters
/// - `values`：输入序列（收盘价等）`&[f64]`。/ Input series `&[f64]`.
///
/// # 返回值 / Returns
/// [`HtSine`]（`sine` 与 `lead_sine` 等长向量，前导 63 个为 [`f64::NAN`]）。
pub fn ht_sine(values: &[f64]) -> Result<HtSine, TaError> {
    let n = values.len();
    let mut out = HtSine {
        sine: vec![f64::NAN; n],
        lead_sine: vec![f64::NAN; n],
    };
    ht_sine_with_output(values, &mut out)?;
    Ok(out)
}

/// 希尔伯特正弦波，零拷贝写入 `out`（与 `values` 等长的两轨向量，前导 63 个为
/// [`f64::NAN`]）。见 [`ht_sine`]。
///
/// Hilbert Transform — SineWave, written zero-copy into `out` (two
/// equal-length bands; the first 63 positions are [`f64::NAN`]). See
/// [`ht_sine`].
///
/// `out.sine` 与 `out.lead_sine` 长度均必须等于 `values.len()`，否则返回
/// [`TaError::BadParam`]。
/// Both `out.sine` and `out.lead_sine` must have length equal to
/// `values.len()`; otherwise [`TaError::BadParam`] is returned.
pub fn ht_sine_with_output(values: &[f64], out: &mut HtSine) -> Result<(), TaError> {
    let n = values.len();
    if out.sine.len() != n || out.lead_sine.len() != n {
        return Err(TaError::BadParam(
            "ht_sine_with_output: out field lengths must equal values length".into(),
        ));
    }
    let lookback = 63;
    if n <= lookback {
        return Ok(());
    }
    let mut h = Hilbert::new();
    let first_main = h.init(values, lookback, 34); // HT_SINE 的 WMA 预热循环次数 = 34
    let mut today = first_main;
    while today <= n - 1 {
        h.advance_full(values, today, values[today]);
        if today >= lookback {
            out.sine[today] = h.sine;
            out.lead_sine[today] = h.lead_sine;
        }
        today += 1;
    }
    Ok(())
}

/// HT_SINE，使用 TA-Lib 默认参数（无可选参数）。
/// HT_SINE with TA-Lib defaults (no optional inputs).
pub fn ht_sine_default(values: &[f64]) -> Result<HtSine, TaError> {
    ht_sine(values)
}

// ───────────────────────── HT_TRENDMODE ─────────────────────────

/// 希尔伯特趋势模态（HT_TRENDMODE，TA-Lib `TA_HT_TRENDMODE`）。
///
/// Hilbert Transform — Trend vs. Cycle Mode. Runs the shared Hilbert/DCPhase
/// machinery plus a trend state machine, emitting an integer `trend` per bar
/// (1 = trending, 0 = cycling). For equal-length convention the trend is
/// returned as `1.0` / `0.0`; the leading 63 positions are `0.0` (matching
/// TA-Lib, which zero-fills the unstable period for this integer output — the
/// same quirk as the `*INDEX` functions).
///
/// # 参数 / Parameters
/// - `values`：输入序列（收盘价等）`&[f64]`。/ Input series `&[f64]`.
///
/// # 返回值 / Returns
/// 与 `values` 等长的向量，趋势为 1.0/0.0，前导 63 个为 `0.0`。
/// Equal-length vector; trend is 1.0/0.0, the first 63 positions are `0.0`.
pub fn ht_trendmode(values: &[f64]) -> Result<Vec<f64>, TaError> {
    // TA-Lib 对整数输出的不稳定期填 0.0（与 *INDEX 函数一致）。
    // TA-Lib zero-fills the unstable period for integer outputs (like *INDEX).
    let mut out = vec![0.0; values.len()];
    ht_trendmode_with_output(values, &mut out)?;
    Ok(out)
}

/// 希尔伯特趋势模态，零拷贝写入 `out`（与 `values` 等长，趋势为 1.0/0.0，前导 63 个为
/// `0.0`）。见 [`ht_trendmode`]。
///
/// Hilbert Transform — Trend vs. Cycle Mode, written zero-copy into `out`
/// (equal-length to `values`; trend is 1.0/0.0, the first 63 positions are
/// `0.0`). See [`ht_trendmode`].
///
/// `out` 长度必须等于 `values.len()`，否则返回 [`TaError::BadParam`]。
/// `out` must have length equal to `values.len()`; otherwise [`TaError::BadParam`]
/// is returned.
pub fn ht_trendmode_with_output(values: &[f64], out: &mut [f64]) -> Result<(), TaError> {
    let n = values.len();
    if out.len() != n {
        return Err(TaError::BadParam(
            "ht_trendmode_with_output: out length must equal values length".into(),
        ));
    }
    let lookback = 63;
    if n <= lookback {
        return Ok(());
    }
    let mut h = Hilbert::new();
    let first_main = h.init(values, lookback, 34); // HT_TRENDMODE 的 WMA 预热循环次数 = 34
    let mut today = first_main;
    while today <= n - 1 {
        h.advance_full(values, today, values[today]);
        h.advance_trend(values, today);
        if today >= lookback {
            out[today] = h.trend as f64;
        }
        today += 1;
    }
    Ok(())
}

/// HT_TRENDMODE，使用 TA-Lib 默认参数（无可选参数）。
/// HT_TRENDMODE with TA-Lib defaults (no optional inputs).
pub fn ht_trendmode_default(values: &[f64]) -> Result<Vec<f64>, TaError> {
    ht_trendmode(values)
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
