//! # 形态识别 · 第 5 批 / Pattern Recognition · Batch 5
//!
//! 本批实现 7 个蜡烛图形态，覆盖 2-candle（`cdl_homingpigeon` / `cdl_inneck` /
//! `cdl_kicking` / `cdl_kickingbylength`）、3-candle（`cdl_identical3crows`）、
//! 镜像形态（`cdl_invertedhammer` 为 `cdl_hammer` 的反向）、以及 5-candle 多根
//!（`cdl_ladderbottom`）。全部与 TA-Lib 0.7.1 黄金向量逐项 1:1。
//!
//! This batch implements 7 candlestick patterns spanning 2-candle (`cdl_homingpigeon` /
//! `cdl_inneck` / `cdl_kicking` / `cdl_kickingbylength`), 3-candle (`cdl_identical3crows`),
//! the mirror pattern (`cdl_invertedhammer`, the inverse of `cdl_hammer`), and a 5-candle
//! multi-bar (`cdl_ladderbottom`). All bit-identical to TA-Lib 0.7.1 golden vectors.

use super::*;
use crate::error::TaError;
use crate::indicator::indicator;

// ===========================================================================
// cdl_homingpigeon — Homing Pigeon（雌雄鸽 / 家鸽）
// ===========================================================================

indicator! {
    /// Homing Pigeon（家鸽）：2-candle 底部反转。第 1 根为长阴线，第 2 根短阴线实体完全
    /// 被第 1 根实体包裹（阴孕线形态）。
    ///
    /// 家鸽恒为看涨（`+100`）。`BodyLong` 使用 `OFF=1`（引用 `i−1`），`BodyShort` 使用 `OFF=0`
    /// （引用 `i`），`lookback = max(BodyShort, BodyLong) + 1 = 11`。对应 `ta_CDLHOMINGPIGEON.c`。
    ///
    /// Homing Pigeon: 2-candle bottom reversal. 1st a long black candle, 2nd a short black real
    /// body totally engulfed by the 1st. Always bullish (`+100`). `BodyLong` `OFF=1` (refs `i−1`),
    /// `BodyShort` `OFF=0` (refs `i`), `lookback = 11`.
    fn cdl_homingpigeon(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_homingpigeon_with_output init zero;
}

/// `cdl_homingpigeon` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_homingpigeon`]。
/// Zero-copy variant of [`cdl_homingpigeon`]: writes results into `out` (length must equal input length).
pub fn cdl_homingpigeon_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_homingpigeon_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_homingpigeon")?;
    let n = open.len();
    let lookback = BODY_SHORT.avg_period.max(BODY_LONG.avg_period) + 1; // 11
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_body_long = {
        let mut s = (lookback - 1 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_long = (lookback - 1 - 10);
    let mut sum_avg_body_short = {
        let mut s = (lookback - 0 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_short = (lookback - 0 - 10);
    let mut i = lookback;
    while i < n {
        let cur_avg_body_long = real_body(open[(i - 1)], close[(i - 1)]);
        let val_avg_body_long = sum_avg_body_long / 10 as f64 * 1.0;
        let cur_avg_body_short = real_body(open[i], close[i]);
        let val_avg_body_short = sum_avg_body_short / 10 as f64 * 1.0;
        out[i] = if candle_color(open[i - 1], close[i - 1]) == -1.0 // 1st black
            && candle_color(open[i], close[i]) == -1.0 // 2nd black
            && cur_avg_body_long > val_avg_body_long // 1st long
            && cur_avg_body_short <= val_avg_body_short // 2nd short
            && open[i] < open[i - 1] // 2nd engulfed by 1st
            && close[i] > close[i - 1]
        {
            100.0
        } else {
            0.0
        };
        sum_avg_body_long +=
            cur_avg_body_long - real_body(open[trail_avg_body_long], close[trail_avg_body_long]);
        trail_avg_body_long += 1;
        sum_avg_body_short +=
            cur_avg_body_short - real_body(open[trail_avg_body_short], close[trail_avg_body_short]);
        trail_avg_body_short += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_identical3crows — Identical Three Crows（三胞胎乌鸦）
// ===========================================================================

indicator! {
    /// Identical Three Crows（三胞胎乌鸦）：3-candle 顶部反转。三根连续走低的阴线，
    /// 后两根开盘价非常接近前一根收盘价（无/极短下影线）。
    ///
    /// 恒为看跌（`−100`）。`ShadowVeryShort` 使用 `OFF ∈ {2,1,0}`（分别对应 `i−2`/`i−1`/`i`），
    /// `Equal` 使用 `OFF ∈ {2,1}`（分别对应 `i−2`/`i−1`），
    /// `lookback = max(ShadowVeryShort, Equal) + 2 = 12`。对应 `ta_CDLIDENTICAL3CROWS.c`。
    ///
    /// Identical Three Crows: 3-candle top reversal. Three consecutive declining black candles,
    /// each after the first opening very close to the prior close. Always bearish (`−100`).
    /// `ShadowVeryShort` `OFF ∈ {2,1,0}`, `Equal` `OFF ∈ {2,1}`, `lookback = 12`.
    fn cdl_identical3crows(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_identical3crows_with_output init zero;
}

/// `cdl_identical3crows` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_identical3crows`]。
/// Zero-copy variant of [`cdl_identical3crows`]: writes results into `out` (length must equal input length).
pub fn cdl_identical3crows_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_identical3crows_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_identical3crows")?;
    let n = open.len();
    let lookback = SHADOW_VERY_SHORT.avg_period.max(EQUAL.avg_period) + 2; // 12
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_sv_2 = {
        let mut s = (lookback - 2 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 2) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_sv_2 = (lookback - 2 - 10);
    let mut sum_avg_sv_1 = {
        let mut s = (lookback - 1 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_sv_1 = (lookback - 1 - 10);
    let mut sum_avg_sv_0 = {
        let mut s = (lookback - 0 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_sv_0 = (lookback - 0 - 10);
    let mut sum_avg_eq_2 = {
        let mut s = (lookback - 2 - 5);
        let mut acc = 0.0_f64;
        while s < (lookback - 2) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_eq_2 = (lookback - 2 - 5);
    let mut sum_avg_eq_1 = {
        let mut s = (lookback - 1 - 5);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_eq_1 = (lookback - 1 - 5);
    let mut i = lookback;
    while i < n {
        let cur_avg_sv_2 = high_low_range(high[(i - 2)], low[(i - 2)]);
        let val_avg_sv_2 = sum_avg_sv_2 / 10 as f64 * 0.1;
        let cur_avg_sv_1 = high_low_range(high[(i - 1)], low[(i - 1)]);
        let val_avg_sv_1 = sum_avg_sv_1 / 10 as f64 * 0.1;
        let cur_avg_sv_0 = high_low_range(high[i], low[i]);
        let val_avg_sv_0 = sum_avg_sv_0 / 10 as f64 * 0.1;
        let cur_avg_eq_2 = high_low_range(high[(i - 2)], low[(i - 2)]);
        let val_avg_eq_2 = sum_avg_eq_2 / 5 as f64 * 0.05;
        let cur_avg_eq_1 = high_low_range(high[(i - 1)], low[(i - 1)]);
        let val_avg_eq_1 = sum_avg_eq_1 / 5 as f64 * 0.05;
        out[i] = if candle_color(open[i - 2], close[i - 2]) == -1.0 // 1st black
            && lower_shadow(open[i - 2], low[i - 2], close[i - 2])
                < val_avg_sv_2 // very short lower shadow
            && candle_color(open[i - 1], close[i - 1]) == -1.0 // 2nd black
            && lower_shadow(open[i - 1], low[i - 1], close[i - 1])
                < val_avg_sv_1 // very short lower shadow
            && candle_color(open[i], close[i]) == -1.0 // 3rd black
            && lower_shadow(open[i], low[i], close[i])
                < val_avg_sv_0 // very short lower shadow
            && close[i - 2] > close[i - 1] // three declining
            && close[i - 1] > close[i]
            && open[i - 1] <= close[i - 2] + val_avg_eq_2 // 2nd opens very close to 1st close
            && open[i - 1] >= close[i - 2] - val_avg_eq_2
            && open[i] <= close[i - 1] + val_avg_eq_1 // 3rd opens very close to 2nd close
            && open[i] >= close[i - 1] - val_avg_eq_1
        {
            -100.0
        } else {
            0.0
        };
        sum_avg_sv_2 += cur_avg_sv_2 - high_low_range(high[trail_avg_sv_2], low[trail_avg_sv_2]);
        trail_avg_sv_2 += 1;
        sum_avg_sv_1 += cur_avg_sv_1 - high_low_range(high[trail_avg_sv_1], low[trail_avg_sv_1]);
        trail_avg_sv_1 += 1;
        sum_avg_sv_0 += cur_avg_sv_0 - high_low_range(high[trail_avg_sv_0], low[trail_avg_sv_0]);
        trail_avg_sv_0 += 1;
        sum_avg_eq_2 += cur_avg_eq_2 - high_low_range(high[trail_avg_eq_2], low[trail_avg_eq_2]);
        trail_avg_eq_2 += 1;
        sum_avg_eq_1 += cur_avg_eq_1 - high_low_range(high[trail_avg_eq_1], low[trail_avg_eq_1]);
        trail_avg_eq_1 += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_inneck — In-Neck Pattern（颈内线）
// ===========================================================================

indicator! {
    /// In-Neck Pattern（颈内线）：2-candle 看跌延续。第 1 根为长阴线，第 2 根阳线开盘低于
    /// 前一根最低价、收盘略微进入前一根实体内部。
    ///
    /// 颈内线恒为看跌（`−100`）。`BodyLong` 与 `Equal` 均使用 `OFF=1`（引用 `i−1`），
    /// `lookback = max(Equal, BodyLong) + 1 = 11`。对应 `ta_CDLINNECK.c`。
    ///
    /// In-Neck: 2-candle bearish continuation. 1st a long black candle, 2nd a white candle that
    /// opens below the prior low and closes slightly into the prior body. Always bearish (`−100`).
    /// `BodyLong` & `Equal` `OFF=1` (ref `i−1`), `lookback = 11`.
    fn cdl_inneck(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_inneck_with_output init zero;
}

/// `cdl_inneck` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_inneck`]。
/// Zero-copy variant of [`cdl_inneck`]: writes results into `out` (length must equal input length).
pub fn cdl_inneck_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_inneck_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_inneck")?;
    let n = open.len();
    let lookback = EQUAL.avg_period.max(BODY_LONG.avg_period) + 1; // 11
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_equal = {
        let mut s = (lookback - 1 - 5);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_equal = (lookback - 1 - 5);
    let mut sum_avg_body_long = {
        let mut s = (lookback - 1 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_long = (lookback - 1 - 10);
    let mut i = lookback;
    while i < n {
        let cur_avg_equal = high_low_range(high[(i - 1)], low[(i - 1)]);
        let val_avg_equal = sum_avg_equal / 5 as f64 * 0.05;
        let cur_avg_body_long = real_body(open[(i - 1)], close[(i - 1)]);
        let val_avg_body_long = sum_avg_body_long / 10 as f64 * 1.0;
        out[i] = if candle_color(open[i - 1], close[i - 1]) == -1.0 // 1st: black
            && real_body(open[i - 1], close[i - 1]) > val_avg_body_long // long
            && candle_color(open[i], close[i]) == 1.0 // 2nd: white
            && open[i] < low[i - 1] // open below prior low
            && close[i] <= close[i - 1] + val_avg_equal // close slightly into prior body
            && close[i] >= close[i - 1]
        {
            -100.0
        } else {
            0.0
        };
        sum_avg_equal +=
            cur_avg_equal - high_low_range(high[trail_avg_equal], low[trail_avg_equal]);
        trail_avg_equal += 1;
        sum_avg_body_long +=
            cur_avg_body_long - real_body(open[trail_avg_body_long], close[trail_avg_body_long]);
        trail_avg_body_long += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_invertedhammer — Inverted Hammer（倒锤头）
// ===========================================================================

indicator! {
    /// Inverted Hammer（倒锤头）：小实体、长上影线、无/极短下影线，且相对前一根实体向下跳空。
    ///
    /// 倒锤头恒为看涨（`+100`），是 `cdl_hammer`（看涨锤头，向上结构）的镜像：锤头看下影线、
    /// 倒锤头看上影线。`BodyShort` / `ShadowLong` / `ShadowVeryShort` 均使用 `OFF=0`
    /// （引用 `i`），`lookback = max(max(BodyShort, ShadowLong), ShadowVeryShort) + 1 = 11`。
    /// 对应 `ta_CDLINVERTEDHAMMER.c`。
    ///
    /// Inverted Hammer: small body, long upper shadow, no/very-short lower shadow, gap down from
    /// prior body. Always bullish (`+100`); the mirror of `cdl_hammer`. All three settings `OFF=0`
    /// (ref `i`), `lookback = 11`.
    fn cdl_invertedhammer(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_invertedhammer_with_output init zero;
}

/// `cdl_invertedhammer` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_invertedhammer`]。
/// Zero-copy variant of [`cdl_invertedhammer`]: writes results into `out` (length must equal input length).
pub fn cdl_invertedhammer_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_invertedhammer_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_invertedhammer")?;
    let n = open.len();
    let lookback = [
        BODY_SHORT.avg_period,
        SHADOW_LONG.avg_period,
        SHADOW_VERY_SHORT.avg_period,
    ]
    .iter()
    .max()
    .copied()
    .unwrap()
        + 1; // 11
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_body = {
        let mut s = (lookback - 0 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body = (lookback - 0 - 10);
    let mut sum_avg_shadow_long = {
        let mut s = (lookback - 0 - 0);
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_shadow_long = (lookback - 0 - 0);
    let mut sum_avg_shadow_vshort = {
        let mut s = (lookback - 0 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_shadow_vshort = (lookback - 0 - 10);
    let mut i = lookback;
    while i < n {
        let cur_avg_body = real_body(open[i], close[i]);
        let val_avg_body = sum_avg_body / 10 as f64 * 1.0;
        let cur_avg_shadow_long = real_body(open[i], close[i]);
        let val_avg_shadow_long = cur_avg_shadow_long * 1.0;
        let cur_avg_shadow_vshort = high_low_range(high[i], low[i]);
        let val_avg_shadow_vshort = sum_avg_shadow_vshort / 10 as f64 * 0.1;
        out[i] = if real_body(open[i], close[i]) < val_avg_body // small real body
            && upper_shadow(open[i], high[i], close[i]) > val_avg_shadow_long // long upper shadow
            && lower_shadow(open[i], low[i], close[i]) < val_avg_shadow_vshort // very short lower shadow
            && real_body_gap_down(open[i], close[i], open[i - 1], close[i - 1])
        // gap down
        {
            100.0
        } else {
            0.0
        };
        sum_avg_body += cur_avg_body - real_body(open[trail_avg_body], close[trail_avg_body]);
        trail_avg_body += 1;
        sum_avg_shadow_long += cur_avg_shadow_long
            - real_body(open[trail_avg_shadow_long], close[trail_avg_shadow_long]);
        trail_avg_shadow_long += 1;
        sum_avg_shadow_vshort += cur_avg_shadow_vshort
            - high_low_range(high[trail_avg_shadow_vshort], low[trail_avg_shadow_vshort]);
        trail_avg_shadow_vshort += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_kicking — Kicking（反挂 / 踢腿）
// ===========================================================================

indicator! {
    /// Kicking（反挂 / 踢腿）：2-candle 反转。两根反色光头光脚（marubozu）蜡烛之间存在跳空。
    ///
    /// 输出由第 2 根蜡烛颜色决定：第 2 根为阳线（黑→白向上跳空）→ `+100`，为阴线
    /// （白→黑向下跳空）→ `−100`。`BodyLong` 与 `ShadowVeryShort` 均使用 `OFF ∈ {1,0}`
    /// （分别引用 `i−1` 与 `i`，对应第 1 / 第 2 根），`lookback = 11`。
    /// 对应 `ta_CDLKICKING.c`。
    ///
    /// Kicking: 2-candle reversal. Two opposite-color marubozu with a gap between them. Output is
    /// `±100` by the 2nd candle's color. `BodyLong` & `ShadowVeryShort` `OFF ∈ {1,0}`, `lookback = 11`.
    fn cdl_kicking(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_kicking_with_output init zero;
}

/// `cdl_kicking` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_kicking`]。
/// Zero-copy variant of [`cdl_kicking`]: writes results into `out` (length must equal input length).
pub fn cdl_kicking_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_kicking_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_kicking")?;
    let n = open.len();
    let lookback = SHADOW_VERY_SHORT.avg_period.max(BODY_LONG.avg_period) + 1; // 11
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_body_1 = {
        let mut s = (lookback - 1 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_1 = (lookback - 1 - 10);
    let mut sum_avg_body_0 = {
        let mut s = (lookback - 0 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_0 = (lookback - 0 - 10);
    let mut sum_avg_sv_1 = {
        let mut s = (lookback - 1 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_sv_1 = (lookback - 1 - 10);
    let mut sum_avg_sv_0 = {
        let mut s = (lookback - 0 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_sv_0 = (lookback - 0 - 10);
    let mut i = lookback;
    while i < n {
        let cur_avg_body_1 = real_body(open[(i - 1)], close[(i - 1)]);
        let val_avg_body_1 = sum_avg_body_1 / 10 as f64 * 1.0;
        let cur_avg_body_0 = real_body(open[i], close[i]);
        let val_avg_body_0 = sum_avg_body_0 / 10 as f64 * 1.0;
        let cur_avg_sv_1 = high_low_range(high[(i - 1)], low[(i - 1)]);
        let val_avg_sv_1 = sum_avg_sv_1 / 10 as f64 * 0.1;
        let cur_avg_sv_0 = high_low_range(high[i], low[i]);
        let val_avg_sv_0 = sum_avg_sv_0 / 10 as f64 * 0.1;

        let c_prev = candle_color(open[i - 1], close[i - 1]);
        let c_cur = candle_color(open[i], close[i]);
        out[i] = if c_prev == -c_cur // opposite candles
            // 1st marubozu
            && real_body(open[i - 1], close[i - 1]) > val_avg_body_1
            && upper_shadow(open[i - 1], high[i - 1], close[i - 1]) < val_avg_sv_1
            && lower_shadow(open[i - 1], low[i - 1], close[i - 1]) < val_avg_sv_1
            // 2nd marubozu
            && cur_avg_body_0 > val_avg_body_0
            && upper_shadow(open[i], high[i], close[i]) < val_avg_sv_0
            && lower_shadow(open[i], low[i], close[i]) < val_avg_sv_0
            // gap
            && (
                (c_prev == -1.0 && candle_gap_up(low[i], high[i - 1]))
                || (c_prev == 1.0 && candle_gap_down(high[i], low[i - 1]))
            ) {
            c_cur * 100.0
        } else {
            0.0
        };
        sum_avg_body_1 +=
            cur_avg_body_1 - real_body(open[trail_avg_body_1], close[trail_avg_body_1]);
        trail_avg_body_1 += 1;
        sum_avg_body_0 +=
            cur_avg_body_0 - real_body(open[trail_avg_body_0], close[trail_avg_body_0]);
        trail_avg_body_0 += 1;
        sum_avg_sv_1 += cur_avg_sv_1 - high_low_range(high[trail_avg_sv_1], low[trail_avg_sv_1]);
        trail_avg_sv_1 += 1;
        sum_avg_sv_0 += cur_avg_sv_0 - high_low_range(high[trail_avg_sv_0], low[trail_avg_sv_0]);
        trail_avg_sv_0 += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_kickingbylength — Kicking by Length（按长度踢腿）
// ===========================================================================

indicator! {
    /// Kicking by Length（按长度踢腿）：条件与 `cdl_kicking` 完全相同，但输出方向由**两根
    /// marubozu 中实体更长者**的颜色决定（而非第 2 根）。阳线更长 → `+100`，阴线更长 → `−100`。
    ///
    /// `BodyLong` 与 `ShadowVeryShort` 均使用 `OFF ∈ {1,0}`，`lookback = 11`。
    /// 对应 `ta_CDLKICKINGBYLENGTH.c`。
    ///
    /// Kicking by Length: identical conditions to `cdl_kicking`, but the output sign is set by the
    /// color of the *longer* marubozu body (not the 2nd candle). `BodyLong` & `ShadowVeryShort`
    /// `OFF ∈ {1,0}`, `lookback = 11`.
    fn cdl_kickingbylength(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_kickingbylength_with_output init zero;
}

/// `cdl_kickingbylength` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_kickingbylength`]。
/// Zero-copy variant of [`cdl_kickingbylength`]: writes results into `out` (length must equal input length).
pub fn cdl_kickingbylength_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_kickingbylength_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_kickingbylength")?;
    let n = open.len();
    let lookback = SHADOW_VERY_SHORT.avg_period.max(BODY_LONG.avg_period) + 1; // 11
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_body_1 = {
        let mut s = (lookback - 1 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_1 = (lookback - 1 - 10);
    let mut sum_avg_body_0 = {
        let mut s = (lookback - 0 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_0 = (lookback - 0 - 10);
    let mut sum_avg_sv_1 = {
        let mut s = (lookback - 1 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_sv_1 = (lookback - 1 - 10);
    let mut sum_avg_sv_0 = {
        let mut s = (lookback - 0 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_sv_0 = (lookback - 0 - 10);
    let mut i = lookback;
    while i < n {
        let cur_avg_body_1 = real_body(open[(i - 1)], close[(i - 1)]);
        let val_avg_body_1 = sum_avg_body_1 / 10 as f64 * 1.0;
        let cur_avg_body_0 = real_body(open[i], close[i]);
        let val_avg_body_0 = sum_avg_body_0 / 10 as f64 * 1.0;
        let cur_avg_sv_1 = high_low_range(high[(i - 1)], low[(i - 1)]);
        let val_avg_sv_1 = sum_avg_sv_1 / 10 as f64 * 0.1;
        let cur_avg_sv_0 = high_low_range(high[i], low[i]);
        let val_avg_sv_0 = sum_avg_sv_0 / 10 as f64 * 0.1;

        let c_prev = candle_color(open[i - 1], close[i - 1]);
        let c_cur = candle_color(open[i], close[i]);
        // 较长实体决定方向
        let longer = if cur_avg_body_0 > real_body(open[i - 1], close[i - 1]) {
            i
        } else {
            i - 1
        };
        out[i] = if c_prev == -c_cur // opposite candles
            // 1st marubozu
            && real_body(open[i - 1], close[i - 1]) > val_avg_body_1
            && upper_shadow(open[i - 1], high[i - 1], close[i - 1]) < val_avg_sv_1
            && lower_shadow(open[i - 1], low[i - 1], close[i - 1]) < val_avg_sv_1
            // 2nd marubozu
            && cur_avg_body_0 > val_avg_body_0
            && upper_shadow(open[i], high[i], close[i]) < val_avg_sv_0
            && lower_shadow(open[i], low[i], close[i]) < val_avg_sv_0
            // gap
            && (
                (c_prev == -1.0 && candle_gap_up(low[i], high[i - 1]))
                || (c_prev == 1.0 && candle_gap_down(high[i], low[i - 1]))
            ) {
            candle_color(open[longer], close[longer]) * 100.0
        } else {
            0.0
        };
        sum_avg_body_1 +=
            cur_avg_body_1 - real_body(open[trail_avg_body_1], close[trail_avg_body_1]);
        trail_avg_body_1 += 1;
        sum_avg_body_0 +=
            cur_avg_body_0 - real_body(open[trail_avg_body_0], close[trail_avg_body_0]);
        trail_avg_body_0 += 1;
        sum_avg_sv_1 += cur_avg_sv_1 - high_low_range(high[trail_avg_sv_1], low[trail_avg_sv_1]);
        trail_avg_sv_1 += 1;
        sum_avg_sv_0 += cur_avg_sv_0 - high_low_range(high[trail_avg_sv_0], low[trail_avg_sv_0]);
        trail_avg_sv_0 += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_ladderbottom — Ladder Bottom（梯底）
// ===========================================================================

indicator! {
    /// Ladder Bottom（梯底）：5-candle 底部反转。前三根阴线开盘与收盘连续走低，第 4 根阴线
    /// 带上影线，第 5 根阳线开盘高于前一根实体、收盘高于前一根最高价。
    ///
    /// 梯底恒为看涨（`+100`）。`ShadowVeryShort` 使用 `OFF=1`（引用 `i−1`，对应第 4 根的上影线
    /// 比较），`lookback = ShadowVeryShort + 4 = 14`。对应 `ta_CDLLADDERBOTTOM.c`。
    ///
    /// Ladder Bottom: 5-candle bottom reversal. Three declining black candles, a 4th black with an
    /// upper shadow, and a 5th white that opens above the prior body and closes above the prior high.
    /// Always bullish (`+100`). `ShadowVeryShort` `OFF=1` (refs `i−1`), `lookback = 14`.
    fn cdl_ladderbottom(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_ladderbottom_with_output init zero;
}

/// `cdl_ladderbottom` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_ladderbottom`]。
/// Zero-copy variant of [`cdl_ladderbottom`]: writes results into `out` (length must equal input length).
pub fn cdl_ladderbottom_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_ladderbottom_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_ladderbottom")?;
    let n = open.len();
    let lookback = SHADOW_VERY_SHORT.avg_period + 4; // 14
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_sv = {
        let mut s = (lookback - 1 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_sv = (lookback - 1 - 10);
    let mut i = lookback;
    while i < n {
        let cur_avg_sv = high_low_range(high[(i - 1)], low[(i - 1)]);
        let val_avg_sv = sum_avg_sv / 10 as f64 * 0.1;
        out[i] = if candle_color(open[i - 4], close[i - 4]) == -1.0 // 1st black
            && candle_color(open[i - 3], close[i - 3]) == -1.0 // 2nd black
            && candle_color(open[i - 2], close[i - 2]) == -1.0 // 3rd black
            && open[i - 4] > open[i - 3] // consecutively lower opens
            && open[i - 3] > open[i - 2]
            && close[i - 4] > close[i - 3] // and closes
            && close[i - 3] > close[i - 2]
            && candle_color(open[i - 1], close[i - 1]) == -1.0 // 4th: black with upper shadow
            && upper_shadow(open[i - 1], high[i - 1], close[i - 1]) > val_avg_sv
            && candle_color(open[i], close[i]) == 1.0 // 5th: white
            && open[i] > open[i - 1] // opens above prior candle's body
            && close[i] > high[i - 1]
        // closes above prior candle's high
        {
            100.0
        } else {
            0.0
        };
        sum_avg_sv += cur_avg_sv - high_low_range(high[trail_avg_sv], low[trail_avg_sv]);
        trail_avg_sv += 1;
        i += 1;
    }

    Ok(())
}
