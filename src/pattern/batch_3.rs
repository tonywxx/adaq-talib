//! # 形态识别 · 第 3 批 / Pattern Recognition · Batch 3
//!
//! 本批实现 8 个蜡烛形态，均严格对齐 TA-Lib 0.7.1 黄金向量（见 ADR 0005）：
//! `cdl_belthold`（捉腰带线）、`cdl_breakaway`（脱离形态）、`cdl_closingmarubozu`
//! （收盘缺影线）、`cdl_concealbabyswall`（藏婴吞没）、`cdl_counterattack`（反击线）、
//! `cdl_darkcloudcover`（乌云盖顶）、`cdl_dojistar`（十字星）、`cdl_dragonflydoji`（蜻蜓十字）。
//!
//! This batch implements 8 candlestick patterns, bit-identical to the TA-Lib 0.7.1 golden
//! vectors (ADR 0005): belt-hold, breakaway, closing marubozu, concealing baby swallow,
//! counterattack, dark cloud cover, doji star, and dragonfly doji.

use super::*;
use crate::error::TaError;
use crate::indicator::indicator;

// ===========================================================================
// cdl_belthold — Belt-hold（捉腰带线）
// ===========================================================================

indicator! {
    /// Belt-hold（捉腰带线）：长实体、无/极短影线（阳线无下影、阴线无上影）。
    ///
    /// 阳线输出 `+100`，阴线输出 `−100`。`BodyLong`(avgPeriod=10) 与 `ShadowVeryShort`(avgPeriod=10)
    /// 均 `off=0`，`lookback = max(BodyLong, ShadowVeryShort) = 10`。对应 `ta_CDLBELTHOLD.c`。
    ///
    /// Belt-hold: long real body with no/very-short shadow (white → no lower shadow, black → no upper
    /// shadow). `BodyLong` and `ShadowVeryShort` both `off=0`; `lookback = 10`. Output is the candle's
    /// color × 100.
    fn cdl_belthold(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_belthold_with_output init zero;
}

/// `cdl_belthold` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_belthold`]。
/// Zero-copy variant of [`cdl_belthold`]: writes results into `out` (length must equal input length).
pub fn cdl_belthold_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_belthold_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_belthold")?;
    let n = open.len();
    let lookback = BODY_LONG.avg_period.max(SHADOW_VERY_SHORT.avg_period); // 10
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_body = {
        let mut s = lookback - 0 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body = lookback - 0 - 10;
    let mut sum_avg_vs = {
        let mut s = lookback - 0 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_vs = lookback - 0 - 10;
    let mut i = lookback;
    while i < n {
        let cur_avg_body = real_body(open[i], close[i]);
        let val_avg_body = sum_avg_body / 10 as f64 * 1.0;
        let cur_avg_vs = high_low_range(high[i], low[i]);
        let val_avg_vs = sum_avg_vs / 10 as f64 * 0.1;
        out[i] = if cur_avg_body > val_avg_body
            && ((candle_color(open[i], close[i]) == 1.0
                && lower_shadow(open[i], low[i], close[i]) < val_avg_vs)
                || (candle_color(open[i], close[i]) == -1.0
                    && upper_shadow(open[i], high[i], close[i]) < val_avg_vs))
        {
            candle_color(open[i], close[i]) * 100.0
        } else {
            0.0
        };
        sum_avg_body += cur_avg_body - real_body(open[trail_avg_body], close[trail_avg_body]);
        trail_avg_body += 1;
        sum_avg_vs += cur_avg_vs - high_low_range(high[trail_avg_vs], low[trail_avg_vs]);
        trail_avg_vs += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_breakaway — Breakaway（脱离形态）
// ===========================================================================

indicator! {
    /// Breakaway（脱离形态）：5-candle 反转，首根长实体、后续跳空、末根收盘回填缺口。
    ///
    /// 末根阳线看涨（`+100`），末根阴线看跌（`−100`）。`BodyLong` 使用 `off=4`（引用 `i−4`），
    /// `lookback = BodyLong + 4 = 14`。对应 `ta_CDLBREAKAWAY.c`。
    ///
    /// Breakaway: 5-candle reversal — 1st long body, gaps, 5th closes inside the gap. `BodyLong` uses
    /// `off=4` (references `i−4`); `lookback = 14`. Output is the 5th candle's color × 100.
    fn cdl_breakaway(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_breakaway_with_output init zero;
}

/// `cdl_breakaway` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_breakaway`]。
/// Zero-copy variant of [`cdl_breakaway`]: writes results into `out` (length must equal input length).
pub fn cdl_breakaway_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_breakaway_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_breakaway")?;
    let n = open.len();
    let lookback = BODY_LONG.avg_period + 4; // 14
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_body = {
        let mut s = lookback - 4 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 4) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body = lookback - 4 - 10;
    let mut i = lookback;
    while i < n {
        let cur_avg_body = real_body(open[i - 4], close[i - 4]);
        let val_avg_body = sum_avg_body / 10 as f64 * 1.0;
        out[i] = if real_body(open[i - 4], close[i - 4]) > val_avg_body
            && candle_color(open[i - 4], close[i - 4]) == candle_color(open[i - 3], close[i - 3])
            && candle_color(open[i - 3], close[i - 3]) == candle_color(open[i - 1], close[i - 1])
            && candle_color(open[i - 1], close[i - 1]) == -candle_color(open[i], close[i])
            && ((candle_color(open[i - 4], close[i - 4]) == -1.0
                && real_body_gap_down(open[i - 3], close[i - 3], open[i - 4], close[i - 4])
                && high[i - 2] < high[i - 3]
                && low[i - 2] < low[i - 3]
                && high[i - 1] < high[i - 2]
                && low[i - 1] < low[i - 2]
                && close[i] > open[i - 3]
                && close[i] < close[i - 4])
                || (candle_color(open[i - 4], close[i - 4]) == 1.0
                    && real_body_gap_up(open[i - 3], close[i - 3], open[i - 4], close[i - 4])
                    && high[i - 2] > high[i - 3]
                    && low[i - 2] > low[i - 3]
                    && high[i - 1] > high[i - 2]
                    && low[i - 1] > low[i - 2]
                    && close[i] < open[i - 3]
                    && close[i] > close[i - 4]))
        {
            candle_color(open[i], close[i]) * 100.0
        } else {
            0.0
        };
        sum_avg_body += cur_avg_body - real_body(open[trail_avg_body], close[trail_avg_body]);
        trail_avg_body += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_closingmarubozu — Closing Marubozu（收盘缺影线）
// ===========================================================================

indicator! {
    /// Closing Marubozu（收盘缺影线）：长实体、无/极短影线（阳线无上影、阴线无下影）。
    ///
    /// 阳线输出 `+100`，阴线输出 `−100`。`BodyLong`(off=0) 与 `ShadowVeryShort`(off=0)，
    /// `lookback = max(BodyLong, ShadowVeryShort) = 10`。对应 `ta_CDLCLOSINGMARUBOZU.c`。
    ///
    /// Closing Marubozu: long real body with no/very-short shadow (white → no upper shadow, black → no
    /// lower shadow). `BodyLong` and `ShadowVeryShort` both `off=0`; `lookback = 10`. Output is the
    /// candle's color × 100.
    fn cdl_closingmarubozu(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_closingmarubozu_with_output init zero;
}

/// `cdl_closingmarubozu` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_closingmarubozu`]。
/// Zero-copy variant of [`cdl_closingmarubozu`]: writes results into `out` (length must equal input length).
pub fn cdl_closingmarubozu_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_closingmarubozu_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_closingmarubozu")?;
    let n = open.len();
    let lookback = BODY_LONG.avg_period.max(SHADOW_VERY_SHORT.avg_period); // 10
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_body = {
        let mut s = lookback - 0 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body = lookback - 0 - 10;
    let mut sum_avg_vs = {
        let mut s = lookback - 0 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_vs = lookback - 0 - 10;
    let mut i = lookback;
    while i < n {
        let cur_avg_body = real_body(open[i], close[i]);
        let val_avg_body = sum_avg_body / 10 as f64 * 1.0;
        let cur_avg_vs = high_low_range(high[i], low[i]);
        let val_avg_vs = sum_avg_vs / 10 as f64 * 0.1;
        out[i] = if cur_avg_body > val_avg_body
            && ((candle_color(open[i], close[i]) == 1.0
                && upper_shadow(open[i], high[i], close[i]) < val_avg_vs)
                || (candle_color(open[i], close[i]) == -1.0
                    && lower_shadow(open[i], low[i], close[i]) < val_avg_vs))
        {
            candle_color(open[i], close[i]) * 100.0
        } else {
            0.0
        };
        sum_avg_body += cur_avg_body - real_body(open[trail_avg_body], close[trail_avg_body]);
        trail_avg_body += 1;
        sum_avg_vs += cur_avg_vs - high_low_range(high[trail_avg_vs], low[trail_avg_vs]);
        trail_avg_vs += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_concealbabyswall — Concealing Baby Swallow（藏婴吞没）
// ===========================================================================

indicator! {
    /// Concealing Baby Swallow（藏婴吞没）：4-candle 底部反转，3 根黑孕线 + 第 4 根吞没第 3 根。
    ///
    /// 恒为看涨（`+100`）。`ShadowVeryShort` 用 3 个 `CandleAvg`（off = 3/2/1）评估第 i−3/i−2/i−1
    /// 根影线，`lookback = ShadowVeryShort + 3 = 13`。对应 `ta_CDLCONCEALBABYSWALL.c`。
    ///
    /// Concealing Baby Swallow: 4-candle bullish bottom reversal — three black marubozu-like candles
    /// then a 4th engulfing the 3rd. `ShadowVeryShort` uses 3 `CandleAvg` (off 3/2/1); `lookback = 13`.
    /// Always bullish (`+100`).
    fn cdl_concealbabyswall(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_concealbabyswall_with_output init zero;
}

/// `cdl_concealbabyswall` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_concealbabyswall`]。
/// Zero-copy variant of [`cdl_concealbabyswall`]: writes results into `out` (length must equal input length).
pub fn cdl_concealbabyswall_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_concealbabyswall_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_concealbabyswall")?;
    let n = open.len();
    let lookback = SHADOW_VERY_SHORT.avg_period + 3; // 13
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_vs_3 = {
        let mut s = lookback - 3 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 3) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_vs_3 = lookback - 3 - 10;
    let mut sum_avg_vs_2 = {
        let mut s = lookback - 2 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 2) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_vs_2 = lookback - 2 - 10;
    let mut sum_avg_vs_1 = {
        let mut s = lookback - 1 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_vs_1 = lookback - 1 - 10;
    let mut i = lookback;
    while i < n {
        let cur_avg_vs_3 = high_low_range(high[i - 3], low[i - 3]);
        let val_avg_vs_3 = sum_avg_vs_3 / 10 as f64 * 0.1;
        let cur_avg_vs_2 = high_low_range(high[i - 2], low[i - 2]);
        let val_avg_vs_2 = sum_avg_vs_2 / 10 as f64 * 0.1;
        let cur_avg_vs_1 = high_low_range(high[i - 1], low[i - 1]);
        let val_avg_vs_1 = sum_avg_vs_1 / 10 as f64 * 0.1;
        out[i] = if candle_color(open[i - 3], close[i - 3]) == -1.0
            && candle_color(open[i - 2], close[i - 2]) == -1.0
            && candle_color(open[i - 1], close[i - 1]) == -1.0
            && candle_color(open[i], close[i]) == -1.0
            && lower_shadow(open[i - 3], low[i - 3], close[i - 3]) < val_avg_vs_3
            && upper_shadow(open[i - 3], high[i - 3], close[i - 3]) < val_avg_vs_3
            && lower_shadow(open[i - 2], low[i - 2], close[i - 2]) < val_avg_vs_2
            && upper_shadow(open[i - 2], high[i - 2], close[i - 2]) < val_avg_vs_2
            && real_body_gap_down(open[i - 1], close[i - 1], open[i - 2], close[i - 2])
            && upper_shadow(open[i - 1], high[i - 1], close[i - 1]) > val_avg_vs_1
            && high[i - 1] > close[i - 2]
            && high[i] > high[i - 1]
            && low[i] < low[i - 1]
        {
            100.0
        } else {
            0.0
        };
        sum_avg_vs_3 += cur_avg_vs_3 - high_low_range(high[trail_avg_vs_3], low[trail_avg_vs_3]);
        trail_avg_vs_3 += 1;
        sum_avg_vs_2 += cur_avg_vs_2 - high_low_range(high[trail_avg_vs_2], low[trail_avg_vs_2]);
        trail_avg_vs_2 += 1;
        sum_avg_vs_1 += cur_avg_vs_1 - high_low_range(high[trail_avg_vs_1], low[trail_avg_vs_1]);
        trail_avg_vs_1 += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_counterattack — Counterattack（反击线）
// ===========================================================================

indicator! {
    /// Counterattack（反击线）：2-candle 反转，两根反向长实体、收盘价相等（或近等）。
    ///
    /// 第 2 根阳线看涨（`+100`），阴线看跌（`−100`）。`Equal`(off=1)、`BodyLong` 用 2 个
    /// `CandleAvg`（off = 1/0），`lookback = max(Equal, BodyLong) + 1 = 11`。对应 `ta_CDLCOUNTERATTACK.c`。
    ///
    /// Counterattack: 2-candle reversal — two opposite long bodies closing at (near) the same price.
    /// `Equal` uses `off=1`; `BodyLong` uses 2 `CandleAvg` (off 1/0); `lookback = 11`. Output is the
    /// 2nd candle's color × 100.
    fn cdl_counterattack(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_counterattack_with_output init zero;
}

/// `cdl_counterattack` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_counterattack`]。
/// Zero-copy variant of [`cdl_counterattack`]: writes results into `out` (length must equal input length).
pub fn cdl_counterattack_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_counterattack_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_counterattack")?;
    let n = open.len();
    let lookback = BODY_LONG.avg_period.max(EQUAL.avg_period) + 1; // 11
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_equal = {
        let mut s = lookback - 1 - 5;
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_equal = lookback - 1 - 5;
    let mut sum_avg_body_1 = {
        let mut s = lookback - 1 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_1 = lookback - 1 - 10;
    let mut sum_avg_body_0 = {
        let mut s = lookback - 0 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_0 = lookback - 0 - 10;
    let mut i = lookback;
    while i < n {
        let cur_avg_equal = high_low_range(high[i - 1], low[i - 1]);
        let val_avg_equal = sum_avg_equal / 5 as f64 * 0.05;
        let cur_avg_body_1 = real_body(open[i - 1], close[i - 1]);
        let val_avg_body_1 = sum_avg_body_1 / 10 as f64 * 1.0;
        let cur_avg_body_0 = real_body(open[i], close[i]);
        let val_avg_body_0 = sum_avg_body_0 / 10 as f64 * 1.0;
        out[i] = if candle_color(open[i - 1], close[i - 1]) == -candle_color(open[i], close[i])
            && real_body(open[i - 1], close[i - 1]) > val_avg_body_1
            && cur_avg_body_0 > val_avg_body_0
            && close[i] <= close[i - 1] + val_avg_equal
            && close[i] >= close[i - 1] - val_avg_equal
        {
            candle_color(open[i], close[i]) * 100.0
        } else {
            0.0
        };
        sum_avg_equal +=
            cur_avg_equal - high_low_range(high[trail_avg_equal], low[trail_avg_equal]);
        trail_avg_equal += 1;
        sum_avg_body_1 +=
            cur_avg_body_1 - real_body(open[trail_avg_body_1], close[trail_avg_body_1]);
        trail_avg_body_1 += 1;
        sum_avg_body_0 +=
            cur_avg_body_0 - real_body(open[trail_avg_body_0], close[trail_avg_body_0]);
        trail_avg_body_0 += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_darkcloudcover — Dark Cloud Cover（乌云盖顶）
// ===========================================================================

indicator! {
    /// Dark Cloud Cover（乌云盖顶）：2-candle 顶部反转，长阳线后高开低收的黑线。
    ///
    /// 恒为看跌（`−100`）。`BodyLong` 使用 `off=1`（引用 `i−1`），`penetration` 默认 0.5，
    /// `lookback = BodyLong + 1 = 11`。对应 `ta_CDLDARKCLOUDCOVER.c`。
    ///
    /// Dark Cloud Cover: 2-candle top reversal — long white candle followed by a black candle that
    /// opens above the prior high and closes within the prior body. `BodyLong` uses `off=1`,
    /// `penetration = 0.5`; `lookback = 11`. Always bearish (`−100`).
    fn cdl_darkcloudcover(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_darkcloudcover_with_output init zero;
}

/// `cdl_darkcloudcover` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_darkcloudcover`]。
/// Zero-copy variant of [`cdl_darkcloudcover`]: writes results into `out` (length must equal input length).
pub fn cdl_darkcloudcover_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_darkcloudcover_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_darkcloudcover")?;
    let n = open.len();
    let lookback = BODY_LONG.avg_period + 1; // 11
    let penetration: f64 = 0.5; // TA_CDLDARKCLOUDCOVER default optInPenetration
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_body = {
        let mut s = lookback - 1 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body = lookback - 1 - 10;
    let mut i = lookback;
    while i < n {
        let cur_avg_body = real_body(open[i - 1], close[i - 1]);
        let val_avg_body = sum_avg_body / 10 as f64 * 1.0;
        out[i] = if candle_color(open[i - 1], close[i - 1]) == 1.0
            && real_body(open[i - 1], close[i - 1]) > val_avg_body
            && candle_color(open[i], close[i]) == -1.0
            && open[i] > high[i - 1]
            && close[i] > open[i - 1]
            && close[i] < close[i - 1] - real_body(open[i - 1], close[i - 1]) * penetration
        {
            -100.0
        } else {
            0.0
        };
        sum_avg_body += cur_avg_body - real_body(open[trail_avg_body], close[trail_avg_body]);
        trail_avg_body += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_dojistar — Doji Star（十字星）
// ===========================================================================

indicator! {
    /// Doji Star（十字星）：长实体 + 跳空十字星。
    ///
    /// 首根为阳线、十字星向上跳空 → 看跌（`−100`）；首根为阴线、十字星向下跳空 → 看涨（`+100`）。
    /// `BodyLong` 用 `off=1`（引用 `i−1`），`BodyDoji` 用 `off=0`，`lookback =
    /// max(BodyLong, BodyDoji) + 1 = 11`。对应 `ta_CDLDOJISTAR.c`。
    ///
    /// Doji Star: long real body followed by a gapping doji. White 1st + gap-up doji → bearish
    /// (`−100`; black 1st + gap-down doji → bullish (`+100`). `BodyLong` uses `off=1`, `BodyDoji`
    /// uses `off=0`; `lookback = 11`.
    fn cdl_dojistar(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_dojistar_with_output init zero;
}

/// `cdl_dojistar` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_dojistar`]。
/// Zero-copy variant of [`cdl_dojistar`]: writes results into `out` (length must equal input length).
pub fn cdl_dojistar_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_dojistar_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_dojistar")?;
    let n = open.len();
    let lookback = BODY_LONG.avg_period.max(BODY_DOJI.avg_period) + 1; // 11
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_body = {
        let mut s = lookback - 1 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body = lookback - 1 - 10;
    let mut sum_avg_doji = {
        let mut s = lookback - 0 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_doji = lookback - 0 - 10;
    let mut i = lookback;
    while i < n {
        let cur_avg_body = real_body(open[i - 1], close[i - 1]);
        let val_avg_body = sum_avg_body / 10 as f64 * 1.0;
        let cur_avg_doji = high_low_range(high[i], low[i]);
        let val_avg_doji = sum_avg_doji / 10 as f64 * 0.1;
        out[i] = if real_body(open[i - 1], close[i - 1]) > val_avg_body
            && real_body(open[i], close[i]) <= val_avg_doji
            && ((candle_color(open[i - 1], close[i - 1]) == 1.0
                && real_body_gap_up(open[i], close[i], open[i - 1], close[i - 1]))
                || (candle_color(open[i - 1], close[i - 1]) == -1.0
                    && real_body_gap_down(open[i], close[i], open[i - 1], close[i - 1])))
        {
            -candle_color(open[i - 1], close[i - 1]) * 100.0
        } else {
            0.0
        };
        sum_avg_body += cur_avg_body - real_body(open[trail_avg_body], close[trail_avg_body]);
        trail_avg_body += 1;
        sum_avg_doji += cur_avg_doji - high_low_range(high[trail_avg_doji], low[trail_avg_doji]);
        trail_avg_doji += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_dragonflydoji — Dragonfly Doji（蜻蜓十字）
// ===========================================================================

indicator! {
    /// Dragonfly Doji（蜻蜓十字）：实体极小、开盘/收盘在近高点（无上影）、长下影。
    ///
    /// 恒为 `+100`（需结合趋势判断多空，本函数固定输出）。`BodyDoji`(off=0) 与 `ShadowVeryShort`
    /// (off=0)，`lookback = max(BodyDoji, ShadowVeryShort) = 10`。对应 `ta_CDLDRAGONFLYDOJI.c`。
    ///
    /// Dragonfly Doji: very small body, open/close near the high (no upper shadow), long lower shadow.
    /// Always `+100` (trend context determines bullishness; the function outputs a constant).
    /// `BodyDoji` and `ShadowVeryShort` both `off=0`; `lookback = 10`.
    fn cdl_dragonflydoji(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_dragonflydoji_with_output init zero;
}

/// `cdl_dragonflydoji` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_dragonflydoji`]。
/// Zero-copy variant of [`cdl_dragonflydoji`]: writes results into `out` (length must equal input length).
pub fn cdl_dragonflydoji_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_dragonflydoji_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_dragonflydoji")?;
    let n = open.len();
    let lookback = BODY_DOJI.avg_period.max(SHADOW_VERY_SHORT.avg_period); // 10
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_doji = {
        let mut s = lookback - 0 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_doji = lookback - 0 - 10;
    let mut sum_avg_vs = {
        let mut s = lookback - 0 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_vs = lookback - 0 - 10;
    let mut i = lookback;
    while i < n {
        let cur_avg_doji = high_low_range(high[i], low[i]);
        let val_avg_doji = sum_avg_doji / 10 as f64 * 0.1;
        let cur_avg_vs = high_low_range(high[i], low[i]);
        let val_avg_vs = sum_avg_vs / 10 as f64 * 0.1;
        out[i] = if real_body(open[i], close[i]) <= val_avg_doji
            && upper_shadow(open[i], high[i], close[i]) < val_avg_vs
            && lower_shadow(open[i], low[i], close[i]) > val_avg_vs
        {
            100.0
        } else {
            0.0
        };
        sum_avg_doji += cur_avg_doji - high_low_range(high[trail_avg_doji], low[trail_avg_doji]);
        trail_avg_doji += 1;
        sum_avg_vs += cur_avg_vs - high_low_range(high[trail_avg_vs], low[trail_avg_vs]);
        trail_avg_vs += 1;
        i += 1;
    }

    Ok(())
}
