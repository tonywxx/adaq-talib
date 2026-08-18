//! # 形态识别（Pattern Recognition）
//!
//! 61 个蜡烛图（K 线）形态识别函数，数值与 [TA-Lib](https://ta-lib.org) 0.7.1 逐项一致
//! （在浮点误差容限内，见 ADR 0005）。每个函数接收 `open/high/low/close` 四根等长切片，
//! 返回与输入等长的整数向量（`-100` 看跌 / `0` 中性 / `+100` 看涨）；前导 `lookback` 个
//! 位置填 `0.0`（TA-Lib 整数输出的约定，见 ADR 0007）。
//!
//! 61 candlestick pattern-recognition functions, bit-identical to TA-Lib 0.7.1 (ADR 0005).
//! Each takes equal-length `open/high/low/close` slices and returns an equal-length integer
//! vector (`-100` bearish / `0` neutral / `+100` bullish); the leading `lookback` positions
//! are `0.0` (TA-Lib integer-output convention, ADR 0007).
//!
//! ## 实现说明 / Implementation notes
//!
//! TA-Lib 的蜡烛形态逻辑由一组共享的"蜡烛设置"（candle settings）驱动，本模块完整复刻其
//! 默认的 11 项设置（见 `ta_global.c` 的 `TA_CandleDefaultSettings`）。`CandleAvg` 助手精确
//! 复刻了 TA-Lib 每个形态内部的"滚动求和"窗口：种子段（warm-up）与每根 K 线后的 `+= range(i)
//! - range(trailing)` 推进顺序均与 C 源一致，从而与黄金向量逐项 1:1。
//!
//! TA-Lib drives its candlestick logic with shared "candle settings"; this module reproduces
//! all 11 default settings (see `TA_CandleDefaultSettings` in `ta_global.c`). The [`CandleAvg`]
//! helper reproduces TA-Lib's per-pattern running-sum window exactly — both the warm-up seed
//! and the `+= range(i) - range(trailing)` advance order match the C source, yielding 1:1
//! golden-vector agreement.
//!
//! ### 助手 API（供各 `batch_*.rs` 实现使用）/ Helper API (for `batch_*.rs` impls)
//!
//! - 蜡烛原语：`real_body` / `upper_shadow` / `lower_shadow` / `high_low_range` /
//!   `candle_color` / `body_high` / `body_low` / `body_center`
//!   （对应 C 宏 `TA_REALBODY` / `TA_UPPERSHADOW` / `TA_LOWERSHADOW` / `TA_HIGHLOWRANGE` /
//!   `TA_CANDLECOLOR` / `max(o,c)` / `min(o,c)` / `(o+c)/2`）。
//! - 跳空原语：`real_body_gap_up` / `real_body_gap_down` / `candle_gap_up` /
//!   `candle_gap_down`
//!   （对应 `TA_REALBODYGAPUP` / `TA_REALBODYGAPDOWN` / `TA_CANDLEGAPUP` / `TA_CANDLEGAPDOWN`，
//!   参数为 `(当前蜡烛 o/c, 前一根蜡烛 o/c)` 或 `(当前 low, 前一根 high)`）。
//! - [`CandleAvg`]：`new(setting, o, h, l, c, start_idx, off)` 预热滚动和；
//!   `value(i, o, h, l, c)` 取当前 K 线的蜡烛均值（推进前调用）；
//!   `advance(i, o, h, l, c)` 推进窗口（每根 K 线判定后调用一次）。
//!   `off` = 形态的结构性前置偏移 = `lookback − max(所用设置的 avgPeriod)`，等价于 C 源中
//!   `TrailingIdx = startIdx − OFF − avgPeriod` 的 `OFF`。
//!
//! 每个形态的标准主循环骨架：
//!
//! ```text
//! let lookback = ...;            // = TA_CDLxxx_Lookback() 的整数值
//! let off = ...;                 // = lookback − max(所用设置的 avgPeriod)
//! let mut out = vec![0.0; n];
//! if n <= lookback { return Ok(out); }
//! let mut avg_body = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, off);
//! let mut i = lookback;
//! while i < n {
//!     if real_body(open[i], close[i]) > avg_body.value(i, open, high, low, close) { ... }
//!     out[i] = if hit { candle_color(open[i], close[i]) * 100.0 } else { 0.0 };
//!     avg_body.advance(i, open, high, low, close);
//!     i += 1;
//! }
//! Ok(out)
//! ```

use crate::error::TaError;

// ===========================================================================
// 蜡烛原语 / Candle primitives (mirror ta_utility.h TA_* macros)
// ===========================================================================

/// 实体长度 `|close − open|`（C 宏 `TA_REALBODY`）。
#[inline]
pub fn real_body(open: f64, close: f64) -> f64 {
    (close - open).abs()
}

/// 上影线 `high − max(open, close)`（C 宏 `TA_UPPERSHADOW`）。
#[inline]
pub fn upper_shadow(open: f64, high: f64, close: f64) -> f64 {
    high - open.max(close)
}

/// 下影线 `min(open, close) − low`（C 宏 `TA_LOWERSHADOW`）。
#[inline]
pub fn lower_shadow(open: f64, low: f64, close: f64) -> f64 {
    open.min(close) - low
}

/// 全幅 `high − low`（C 宏 `TA_HIGHLOWRANGE`）。
#[inline]
pub fn high_low_range(high: f64, low: f64) -> f64 {
    high - low
}

/// 蜡烛颜色：阳线（close ≥ open）为 `+1`，阴线为 `−1`（C 宏 `TA_CANDLECOLOR`）。
#[inline]
pub fn candle_color(open: f64, close: f64) -> f64 {
    if close >= open {
        1.0
    } else {
        -1.0
    }
}

/// 实体高点 `max(open, close)`（C 宏 `TA_BodyHigh`）。
#[inline]
pub fn body_high(open: f64, close: f64) -> f64 {
    open.max(close)
}

/// 实体低点 `min(open, close)`（C 宏 `TA_BodyLow`）。
#[inline]
pub fn body_low(open: f64, close: f64) -> f64 {
    open.min(close)
}

/// 实体中心 `(open + close) / 2`（C 宏 `TA_BodyCenter`）。
#[inline]
pub fn body_center(open: f64, close: f64) -> f64 {
    (open + close) * 0.5
}

/// 实体向上跳空：`min(o_cur,c_cur) > max(o_prev,c_prev)`（C 宏 `TA_REALBODYGAPUP(IDX2,IDX1)`）。
#[inline]
pub fn real_body_gap_up(
    o_cur: f64,
    c_cur: f64,
    o_prev: f64,
    c_prev: f64,
) -> bool {
    o_cur.min(c_cur) > o_prev.max(c_prev)
}

/// 实体向下跳空：`max(o_cur,c_cur) < min(o_prev,c_prev)`（C 宏 `TA_REALBODYGAPDOWN(IDX2,IDX1)`）。
#[inline]
pub fn real_body_gap_down(
    o_cur: f64,
    c_cur: f64,
    o_prev: f64,
    c_prev: f64,
) -> bool {
    o_cur.max(c_cur) < o_prev.min(c_prev)
}

/// 蜡烛向上跳空：`low_cur > high_prev`（C 宏 `TA_CANDLEGAPUP(IDX2,IDX1)`）。
#[inline]
pub fn candle_gap_up(low_cur: f64, high_prev: f64) -> bool {
    low_cur > high_prev
}

/// 蜡烛向下跳空：`high_cur < low_prev`（C 宏 `TA_CANDLEGAPDOWN(IDX2,IDX1)`）。
#[inline]
pub fn candle_gap_down(high_cur: f64, low_prev: f64) -> bool {
    high_cur < low_prev
}

// ===========================================================================
// 蜡烛设置 / Candle settings (ta_global.c TA_CandleDefaultSettings)
// ===========================================================================

/// 蜡烛范围类型（对应 C 的 `TA_RangeType`）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RangeType {
    /// 实体长度 `|close − open|`（C `TA_RangeType_RealBody`）。
    RealBody,
    /// 全幅 `high − low`（C `TA_RangeType_HighLow`）。
    HighLow,
    /// 影线总长 `upper + lower shadow`（C `TA_RangeType_Shadows`）。
    Shadows,
}

/// 单个蜡烛设置：范围类型、平均周期、系数（对应 C 的 `TA_CandleSetting`）。
#[derive(Clone, Copy, Debug)]
pub struct CandleSetting {
    /// 范围类型 / range type.
    pub range_type: RangeType,
    /// 平均周期（0 表示仅用当前 K 线，不取均值）/ averaging period (0 = current bar only).
    pub avg_period: usize,
    /// 系数 / scale factor.
    pub factor: f64,
}

// 11 个默认蜡烛设置（索引顺序与 TA-Lib 一致，见 ta_global.c）。
// The 11 default candle settings (order matches TA-Lib, see ta_global.c).
pub const BODY_LONG: CandleSetting =
    CandleSetting { range_type: RangeType::RealBody, avg_period: 10, factor: 1.0 };
pub const BODY_VERY_LONG: CandleSetting =
    CandleSetting { range_type: RangeType::RealBody, avg_period: 10, factor: 3.0 };
pub const BODY_SHORT: CandleSetting =
    CandleSetting { range_type: RangeType::RealBody, avg_period: 10, factor: 1.0 };
pub const BODY_DOJI: CandleSetting =
    CandleSetting { range_type: RangeType::HighLow, avg_period: 10, factor: 0.1 };
pub const SHADOW_LONG: CandleSetting =
    CandleSetting { range_type: RangeType::RealBody, avg_period: 0, factor: 1.0 };
pub const SHADOW_VERY_LONG: CandleSetting =
    CandleSetting { range_type: RangeType::RealBody, avg_period: 0, factor: 2.0 };
pub const SHADOW_SHORT: CandleSetting =
    CandleSetting { range_type: RangeType::Shadows, avg_period: 10, factor: 1.0 };
pub const SHADOW_VERY_SHORT: CandleSetting =
    CandleSetting { range_type: RangeType::HighLow, avg_period: 10, factor: 0.1 };
pub const NEAR: CandleSetting =
    CandleSetting { range_type: RangeType::HighLow, avg_period: 5, factor: 0.2 };
pub const FAR: CandleSetting =
    CandleSetting { range_type: RangeType::HighLow, avg_period: 5, factor: 0.6 };
pub const EQUAL: CandleSetting =
    CandleSetting { range_type: RangeType::HighLow, avg_period: 5, factor: 0.05 };

/// 计算某范围类型在单根 K 线上的取值（C 宏 `TA_CANDLERANGE`）。
#[inline]
fn range_of(rt: RangeType, open: f64, high: f64, low: f64, close: f64) -> f64 {
    match rt {
        RangeType::RealBody => real_body(open, close),
        RangeType::HighLow => high_low_range(high, low),
        RangeType::Shadows => upper_shadow(open, high, close) + lower_shadow(open, low, close),
    }
}

// ===========================================================================
// 蜡烛均值滚动窗口 / Candle-average running window (mirrors TA-Lib per-pattern loop)
// ===========================================================================

/// TA-Lib 蜡烛均值的滚动求和窗口助手。
///
/// Mirrors TA-Lib's per-pattern candle-average running sum. `new` seeds the warm-up total
/// over the `avg_period` bars ending `off` bars before `start_idx`; `value` reads the average
/// (before advancing); `advance` does `total += range(i) - range(trailing); trailing++`.
///
/// 用法：每个 `CandleSetting` 建一个 `CandleAvg`，主循环每根 K 线先 `value(i)` 取均值参与判定，
/// 再 `advance(i)` 推进窗口。
pub struct CandleAvg {
    setting: CandleSetting,
    total: f64,
    trailing: usize,
    /// 结构性前置偏移 `OFF = lookback − max(所用设置的 avg_period)`（HAMMER 的 `Near` 为 1，
    /// 2CROWS 的 `BodyLong` 为 2，多数设置为 0）。`value`/`advance` 引用 `i − off`，
    /// 与 C 源 `TA_CANDLEAVERAGE(SET, SUM, i − OFF)` 及 `+= range(i − OFF) − range(trailing)`
    /// 完全一致（见 `ta_CDLHAMMER.c` / `ta_CDL2CROWS.c`）。
    off: usize,
}

impl CandleAvg {
    /// 预热滚动和：对 `k ∈ [start_idx − off − p, start_idx − off)` 累加到 `total`
    ///（与 C 源 `TrailingIdx = start_idx − OFF − avgPeriod` 的 warm-up 一致）。
    pub fn new(
        setting: CandleSetting,
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
        start_idx: usize,
        off: usize,
    ) -> Self {
        let p = setting.avg_period;
        // start_idx = lookback >= off + p，故不会下溢。
        let begin = start_idx - off - p;
        let end = start_idx - off; // 不含
        let mut total = 0.0_f64;
        let mut k = begin;
        while k < end {
            total += range_of(setting.range_type, open[k], high[k], low[k], close[k]);
            k += 1;
        }
        CandleAvg {
            setting,
            total,
            trailing: begin,
            off,
        }
    }

    /// 当前 K 线 `i` 的蜡烛均值（推进前调用）。对应 C 宏
    /// `TA_CANDLEAVERAGE(SET, SUM, i − off)`：
    /// - `avgPeriod == 0` 时用当前（偏移后）K 线范围 `range(i − off)`；
    /// - 否则用 `total / avgPeriod`（running sum）；
    /// Shadows 范围再除以 2。
    #[inline(always)]
    pub fn value(&self, i: usize, open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> f64 {
        let rt = self.setting.range_type;
        let idx = i - self.off;
        let raw = if self.setting.avg_period == 0 {
            range_of(rt, open[idx], high[idx], low[idx], close[idx])
        } else {
            self.total / self.setting.avg_period as f64
        };
        raw * self.setting.factor / if rt == RangeType::Shadows { 2.0 } else { 1.0 }
    }

    /// 推进滚动窗口（每根 K 线判定后调用一次）。对应 C 的
    /// `total += range(i − off) − range(trailing); trailing++`
    ///（与 `ta_CDLHAMMER.c` `NearPeriodTotal += TA_CANDLERANGE(Near, i−1) − …` 一致）。
    #[inline(always)]
    pub fn advance(&mut self, i: usize, open: &[f64], high: &[f64], low: &[f64], close: &[f64]) {
        let rt = self.setting.range_type;
        let idx = i - self.off;
        self.total += range_of(rt, open[idx], high[idx], low[idx], close[idx])
            - range_of(
                rt,
                open[self.trailing],
                high[self.trailing],
                low[self.trailing],
                close[self.trailing],
            );
        self.trailing += 1;
    }
}

// ===========================================================================
// 批处理模块 / Per-batch modules (each implemented in its own file)
// ===========================================================================

// 蜡烛形态内核为对照/基准实现，部分滚动累加器（如 `sum_avg_shadow*`）当前被计算但
// 未参与最终判定；这些 `unused_assignments` / `unused_variables` 是预期内脚手架，
// 故在每个 batch 模块上显式允许，以免在 `-D warnings` 严格环境下阻碍编译。
// Candle-pattern kernels keep some running accumulators (e.g. `sum_avg_shadow*`) that are
// computed but not yet consumed by the final decision; those `unused_assignments` /
// `unused_variables` are intentional scaffolding, so we allow them per batch module to keep
// a `-D warnings` strict build compiling.
#[allow(unused_assignments, unused_variables)]
pub mod batch_1;
#[allow(unused_assignments, unused_variables)]
pub mod batch_2;
#[allow(unused_assignments, unused_variables)]
pub mod batch_3;
#[allow(unused_assignments, unused_variables)]
pub mod batch_4;
#[allow(unused_assignments, unused_variables)]
pub mod batch_5;
#[allow(unused_assignments, unused_variables)]
pub mod batch_6;
#[allow(unused_assignments, unused_variables)]
pub mod batch_7;
#[allow(unused_assignments, unused_variables)]
pub mod batch_8;

// 统一再导出，便于以 `adaq_talib::pattern::cdlxxx` 调用。
// Re-export so callers use `adaq_talib::pattern::cdlxxx`.
pub use batch_1::*;
pub use batch_2::*;
pub use batch_3::*;
pub use batch_4::*;
pub use batch_5::*;
pub use batch_6::*;
pub use batch_7::*;
pub use batch_8::*;

/// 通用长度校验：四根蜡烛序列必须等长。/ Validate equal length of OHLC inputs.
#[inline]
fn check_ohlc(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    name: &str,
) -> Result<(), TaError> {
    let n = open.len();
    if high.len() != n || low.len() != n || close.len() != n {
        return Err(TaError::BadParam(format!(
            "{name}: open/high/low/close must share the same length"
        )));
    }
    Ok(())
}

// ===========================================================================
// 就近单测层 / Near-code unit tests (candidate ⑤)
//
// 形态识别此前仅由 tests/fixtures/*.json 黄金向量校验（远端 Oracle）。本模块在源码近处
// 补一层手算向量单测，覆盖蜡烛原语、CandleAvg 滚动窗口（含 avgPeriod=0 与 OFF≠0 路径）
// 及有代表性的形态（0 setting / 多 setting / 2-candle 两级 / 跳空引用前一根 / OFF=1 NEAR）。
// 输出为整数（0 / ±100 / ±80），用精确相等断言；与黄金向量互不替代，互为补充安全网。
//
// Pattern recognition was only validated by the remote golden-vector fixtures. This near-code
// layer adds hand-computed unit tests for the candle primitives, the CandleAvg running window
// (incl. avgPeriod=0 and OFF≠0), and representative patterns. Outputs are integers, asserted
// exactly. Complements — does not replace — the golden vectors.
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 蜡烛原语 / candle primitives (hand-derived) ----
    #[test]
    fn candle_primitives() {
        assert_eq!(real_body(10.0, 20.0), 10.0);
        assert_eq!(upper_shadow(10.0, 21.0, 20.0), 1.0);
        assert_eq!(lower_shadow(10.0, 9.0, 20.0), 1.0);
        assert_eq!(high_low_range(21.0, 9.0), 12.0);
        assert_eq!(candle_color(10.0, 20.0), 1.0);
        assert_eq!(candle_color(20.0, 10.0), -1.0);
        assert_eq!(body_high(10.0, 20.0), 20.0);
        assert_eq!(body_low(10.0, 20.0), 10.0);
        assert_eq!(body_center(10.0, 20.0), 15.0);
        assert!(real_body_gap_up(5.0, 6.0, 1.0, 2.0));
        assert!(real_body_gap_down(1.0, 2.0, 5.0, 6.0));
        assert!(candle_gap_up(5.0, 3.0));
        assert!(candle_gap_down(3.0, 5.0));
    }

    // ---- CandleAvg 滚动窗口 / running window (hand-derived) ----
    // close[k]-open[k] = k（open 全 0）-> 单根实体长度 = k。
    fn bodies() -> ([f64; 8], [f64; 8], [f64; 8], [f64; 8]) {
        let open = [0.0; 8];
        let close = [0.0, 0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0];
        let high = close;
        let low = open;
        (open, high, low, close)
    }

    #[test]
    fn candle_avg_running_window() {
        let (open, high, low, close) = bodies();
        let s = CandleSetting { range_type: RangeType::RealBody, avg_period: 3, factor: 1.0 };
        let mut a = CandleAvg::new(s, &open, &high, &low, &close, 5, 0);
        assert_eq!(a.value(5, &open, &high, &low, &close), 4.0); // avg(2,4,6)
        a.advance(5, &open, &high, &low, &close);
        assert_eq!(a.value(6, &open, &high, &low, &close), 6.0); // avg(4,6,8)
        a.advance(6, &open, &high, &low, &close);
        assert_eq!(a.value(7, &open, &high, &low, &close), 8.0); // avg(6,8,10)
    }

    #[test]
    fn candle_avg_avg_period_zero() {
        let (open, high, low, close) = bodies();
        let s = CandleSetting { range_type: RangeType::RealBody, avg_period: 0, factor: 2.0 };
        let a = CandleAvg::new(s, &open, &high, &low, &close, 5, 0);
        // avgPeriod==0 -> 偏移后当前根范围 * factor = real_body(8)*2 = 16
        assert_eq!(a.value(5, &open, &high, &low, &close), 16.0);
    }

    #[test]
    fn candle_avg_off_one() {
        let (open, high, low, close) = bodies();
        let s = CandleSetting { range_type: RangeType::RealBody, avg_period: 3, factor: 1.0 };
        let mut a = CandleAvg::new(s, &open, &high, &low, &close, 5, 1);
        // off=1 -> 窗口 [1,4) = 实体长度 0,2,4 -> 均值 2
        assert_eq!(a.value(5, &open, &high, &low, &close), 2.0);
        a.advance(5, &open, &high, &low, &close);
        // 推进后窗口 [2,5) -> 2,4,6 -> 均值 4
        assert_eq!(a.value(6, &open, &high, &low, &close), 4.0);
    }

    // ---- cdl_doji：0 setting，lookback=10 ----
    #[test]
    fn cdl_doji_detects() {
        // 10 根预热（high-low=100），随后 2 根十字星（open==close）。
        let o = vec![0.0; 12];
        let mut h = vec![0.0; 12];
        let l = vec![0.0; 12];
        let c = vec![0.0; 12]; // open==close -> 全为十字星
        for i in 0..12 {
            h[i] = 100.0;
        }
        let out = cdl_doji(&o, &h, &l, &c).unwrap();
        assert_eq!(out.len(), 12);
        for i in 0..10 {
            assert_eq!(out[i], 0.0, "leading index {i} must be 0");
        }
        assert_eq!(out[10], 100.0);
        assert_eq!(out[11], 100.0);
    }

    // ---- cdl_marubozu：2 setting，看涨 / 看跌 / 不触发 ----
    #[test]
    fn cdl_marubozu_bullish() {
        let mut o = vec![0.0; 11];
        let mut h = vec![0.0; 11];
        let mut l = vec![0.0; 11];
        let mut c = vec![0.0; 11];
        for i in 0..10 {
            h[i] = 100.0; // 预热：实体 0、全幅 100
        }
        o[10] = 10.0;
        c[10] = 20.0; // 实体 10 > 0
        h[10] = 21.0; // 上影 1
        l[10] = 9.0; // 下影 1（均 < 0.1*100 = 10）
        let out = cdl_marubozu(&o, &h, &l, &c).unwrap();
        assert_eq!(out[10], 100.0);
    }

    #[test]
    fn cdl_marubozu_bearish() {
        let mut o = vec![0.0; 11];
        let mut h = vec![0.0; 11];
        let mut l = vec![0.0; 11];
        let mut c = vec![0.0; 11];
        for i in 0..10 {
            h[i] = 100.0;
        }
        o[10] = 20.0;
        c[10] = 10.0;
        h[10] = 21.0;
        l[10] = 9.0;
        let out = cdl_marubozu(&o, &h, &l, &c).unwrap();
        assert_eq!(out[10], -100.0);
    }

    #[test]
    fn cdl_marubozu_no_trigger_big_shadows() {
        let mut o = vec![0.0; 11];
        let mut h = vec![0.0; 11];
        let mut l = vec![0.0; 11];
        let mut c = vec![0.0; 11];
        for i in 0..10 {
            h[i] = 100.0;
        }
        o[10] = 10.0;
        c[10] = 20.0;
        h[10] = 100.0; // 上影 80
        l[10] = 0.0; // 下影 80（均 > 10 -> 不触发）
        let out = cdl_marubozu(&o, &h, &l, &c).unwrap();
        assert_eq!(out[10], 0.0);
    }

    // ---- cdl_engulfing：纯 2-candle，两级 ±80/±100 ----
    #[test]
    fn cdl_engulfing_full_bull() {
        // 第1根阴线（20->10），第2根阳线完全吞没（5->25）。
        let o = [0.0, 20.0, 5.0];
        let h = [0.0, 20.0, 25.0];
        let l = [0.0, 10.0, 5.0];
        let c = [0.0, 10.0, 25.0];
        let out = cdl_engulfing(&o, &h, &l, &c).unwrap();
        assert_eq!(out[0], 0.0);
        assert_eq!(out[1], 0.0); // lookback=2，主循环从 i=2 起
        assert_eq!(out[2], 100.0);
    }

    #[test]
    fn cdl_engulfing_weak_bull() {
        // 第2根开盘恰等于前收（open==close[i-1]）-> 弱吞没 +80。
        let o = [0.0, 20.0, 10.0];
        let h = [0.0, 20.0, 25.0];
        let l = [0.0, 10.0, 10.0];
        let c = [0.0, 10.0, 25.0];
        let out = cdl_engulfing(&o, &h, &l, &c).unwrap();
        assert_eq!(out[2], 80.0);
    }

    #[test]
    fn cdl_engulfing_full_bear() {
        let o = [0.0, 10.0, 25.0];
        let h = [0.0, 20.0, 25.0];
        let l = [0.0, 10.0, 5.0];
        let c = [0.0, 20.0, 5.0];
        let out = cdl_engulfing(&o, &h, &l, &c).unwrap();
        assert_eq!(out[2], -100.0);
    }

    #[test]
    fn cdl_engulfing_none() {
        // 两根同为阳线 -> 不吞没。
        let o = [0.0, 10.0, 12.0];
        let h = [0.0, 20.0, 25.0];
        let l = [0.0, 10.0, 12.0];
        let c = [0.0, 20.0, 22.0];
        let out = cdl_engulfing(&o, &h, &l, &c).unwrap();
        assert_eq!(out[2], 0.0);
    }

    // ---- cdl_shootingstar：小实体 + 长上影 + 相对前一根实体向上跳空，看跌 ----
    #[test]
    fn cdl_shootingstar_bearish() {
        // 11 根预热（lookback=11）：open=0,close=5（实体5）,high=100,low=0（全幅100）。
        let mut o = vec![0.0; 12];
        let mut h = vec![0.0; 12];
        let mut l = vec![0.0; 12];
        let mut c = vec![0.0; 12];
        for i in 0..11 {
            o[i] = 0.0;
            c[i] = 5.0;
            h[i] = 100.0;
            l[i] = 0.0;
        }
        // 测试根 i=11：小实体、长上影、较前一根实体跳空向上。
        o[11] = 10.0;
        c[11] = 11.0; // 实体 1
        h[11] = 13.0; // 上影 2
        l[11] = 9.0; // 下影 1
        let out = cdl_shootingstar(&o, &h, &l, &c).unwrap();
        assert_eq!(out[11], -100.0);
    }

    // ---- cdl_hammer：OFF=1 的 NEAR 设置，看涨 ----
    #[test]
    fn cdl_hammer_bullish() {
        // 11 根预热（lookback=11）：open=0,close=5,high=100,low=0。
        let mut o = vec![0.0; 12];
        let mut h = vec![0.0; 12];
        let mut l = vec![0.0; 12];
        let mut c = vec![0.0; 12];
        for i in 0..11 {
            o[i] = 0.0;
            c[i] = 5.0;
            h[i] = 100.0;
            l[i] = 0.0;
        }
        // 测试根 i=11：小实体、长下影、短上影、实体靠近前最低价。
        o[11] = 10.0;
        c[11] = 11.0; // 实体 1 < BODY_SHORT 均值 5
        h[11] = 20.0; // 上影 9 < 0.1*100 = 10
        l[11] = 8.0; // 下影 2 > 实体 1
        let out = cdl_hammer(&o, &h, &l, &c).unwrap();
        assert_eq!(out[11], 100.0);
    }

    #[test]
    fn cdl_hammer_no_trigger_long_upper_shadow() {
        let mut o = vec![0.0; 12];
        let mut h = vec![0.0; 12];
        let mut l = vec![0.0; 12];
        let mut c = vec![0.0; 12];
        for i in 0..11 {
            o[i] = 0.0;
            c[i] = 5.0;
            h[i] = 100.0;
            l[i] = 0.0;
        }
        o[11] = 10.0;
        c[11] = 11.0;
        h[11] = 100.0; // 上影 89 > 10 -> 不触发
        l[11] = 8.0;
        let out = cdl_hammer(&o, &h, &l, &c).unwrap();
        assert_eq!(out[11], 0.0);
    }
}
