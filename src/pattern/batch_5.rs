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

use crate::error::TaError;
use super::*;

// ===========================================================================
// cdl_homingpigeon — Homing Pigeon（雌雄鸽 / 家鸽）
// ===========================================================================

/// Homing Pigeon（家鸽）：2-candle 底部反转。第 1 根为长阴线，第 2 根短阴线实体完全
/// 被第 1 根实体包裹（阴孕线形态）。
///
/// 家鸽恒为看涨（`+100`）。`BodyLong` 使用 `OFF=1`（引用 `i−1`），`BodyShort` 使用 `OFF=0`
/// （引用 `i`），`lookback = max(BodyShort, BodyLong) + 1 = 11`。对应 `ta_CDLHOMINGPIGEON.c`。
///
/// Homing Pigeon: 2-candle bottom reversal. 1st a long black candle, 2nd a short black real
/// body totally engulfed by the 1st. Always bullish (`+100`). `BodyLong` `OFF=1` (refs `i−1`),
/// `BodyShort` `OFF=0` (refs `i`), `lookback = 11`.
pub fn cdl_homingpigeon(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_homingpigeon_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_homingpigeon` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_homingpigeon`]。
/// Zero-copy variant of [`cdl_homingpigeon`]: writes results into `out` (length must equal input length).
pub fn cdl_homingpigeon_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_homingpigeon_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_homingpigeon")?;
    let n = open.len();
    let lookback = BODY_SHORT.avg_period.max(BODY_LONG.avg_period) + 1; // 11
    if n <= lookback {
        return Ok(());
    }
    let mut avg_body_long = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 1);
    let mut avg_body_short = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 1], close[i - 1]) == -1.0 // 1st black
            && candle_color(open[i], close[i]) == -1.0 // 2nd black
            && real_body(open[i - 1], close[i - 1]) > avg_body_long.value(i, open, high, low, close) // 1st long
            && real_body(open[i], close[i]) <= avg_body_short.value(i, open, high, low, close) // 2nd short
            && open[i] < open[i - 1] // 2nd engulfed by 1st
            && close[i] > close[i - 1]
        {
            out[i] = 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body_long.advance(i, open, high, low, close);
        avg_body_short.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_identical3crows — Identical Three Crows（三胞胎乌鸦）
// ===========================================================================

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
pub fn cdl_identical3crows(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_identical3crows_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_identical3crows` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_identical3crows`]。
/// Zero-copy variant of [`cdl_identical3crows`]: writes results into `out` (length must equal input length).
pub fn cdl_identical3crows_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_identical3crows_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_identical3crows")?;
    let n = open.len();
    let lookback = SHADOW_VERY_SHORT.avg_period.max(EQUAL.avg_period) + 2; // 12
    if n <= lookback {
        return Ok(());
    }
    let mut avg_sv_2 = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 2);
    let mut avg_sv_1 = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 1);
    let mut avg_sv_0 = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 0);
    let mut avg_eq_2 = CandleAvg::new(EQUAL, open, high, low, close, lookback, 2);
    let mut avg_eq_1 = CandleAvg::new(EQUAL, open, high, low, close, lookback, 1);
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 2], close[i - 2]) == -1.0 // 1st black
            && lower_shadow(open[i - 2], low[i - 2], close[i - 2])
                < avg_sv_2.value(i, open, high, low, close) // very short lower shadow
            && candle_color(open[i - 1], close[i - 1]) == -1.0 // 2nd black
            && lower_shadow(open[i - 1], low[i - 1], close[i - 1])
                < avg_sv_1.value(i, open, high, low, close) // very short lower shadow
            && candle_color(open[i], close[i]) == -1.0 // 3rd black
            && lower_shadow(open[i], low[i], close[i])
                < avg_sv_0.value(i, open, high, low, close) // very short lower shadow
            && close[i - 2] > close[i - 1] // three declining
            && close[i - 1] > close[i]
            && open[i - 1] <= close[i - 2] + avg_eq_2.value(i, open, high, low, close) // 2nd opens very close to 1st close
            && open[i - 1] >= close[i - 2] - avg_eq_2.value(i, open, high, low, close)
            && open[i] <= close[i - 1] + avg_eq_1.value(i, open, high, low, close) // 3rd opens very close to 2nd close
            && open[i] >= close[i - 1] - avg_eq_1.value(i, open, high, low, close)
        {
            out[i] = -100.0;
        } else {
            out[i] = 0.0;
        }
        avg_sv_2.advance(i, open, high, low, close);
        avg_sv_1.advance(i, open, high, low, close);
        avg_sv_0.advance(i, open, high, low, close);
        avg_eq_2.advance(i, open, high, low, close);
        avg_eq_1.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_inneck — In-Neck Pattern（颈内线）
// ===========================================================================

/// In-Neck Pattern（颈内线）：2-candle 看跌延续。第 1 根为长阴线，第 2 根阳线开盘低于
/// 前一根最低价、收盘略微进入前一根实体内部。
///
/// 颈内线恒为看跌（`−100`）。`BodyLong` 与 `Equal` 均使用 `OFF=1`（引用 `i−1`），
/// `lookback = max(Equal, BodyLong) + 1 = 11`。对应 `ta_CDLINNECK.c`。
///
/// In-Neck: 2-candle bearish continuation. 1st a long black candle, 2nd a white candle that
/// opens below the prior low and closes slightly into the prior body. Always bearish (`−100`).
/// `BodyLong` & `Equal` `OFF=1` (ref `i−1`), `lookback = 11`.
pub fn cdl_inneck(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_inneck_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_inneck` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_inneck`]。
/// Zero-copy variant of [`cdl_inneck`]: writes results into `out` (length must equal input length).
pub fn cdl_inneck_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_inneck_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_inneck")?;
    let n = open.len();
    let lookback = EQUAL.avg_period.max(BODY_LONG.avg_period) + 1; // 11
    if n <= lookback {
        return Ok(());
    }
    let mut avg_equal = CandleAvg::new(EQUAL, open, high, low, close, lookback, 1);
    let mut avg_body_long = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 1);
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 1], close[i - 1]) == -1.0 // 1st: black
            && real_body(open[i - 1], close[i - 1]) > avg_body_long.value(i, open, high, low, close) // long
            && candle_color(open[i], close[i]) == 1.0 // 2nd: white
            && open[i] < low[i - 1] // open below prior low
            && close[i] <= close[i - 1] + avg_equal.value(i, open, high, low, close) // close slightly into prior body
            && close[i] >= close[i - 1]
        {
            out[i] = -100.0;
        } else {
            out[i] = 0.0;
        }
        avg_equal.advance(i, open, high, low, close);
        avg_body_long.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_invertedhammer — Inverted Hammer（倒锤头）
// ===========================================================================

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
pub fn cdl_invertedhammer(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_invertedhammer_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_invertedhammer` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_invertedhammer`]。
/// Zero-copy variant of [`cdl_invertedhammer`]: writes results into `out` (length must equal input length).
pub fn cdl_invertedhammer_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_invertedhammer_with_output: out length must equal input length".into()));
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
    let mut avg_body = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 0);
    let mut avg_shadow_long = CandleAvg::new(SHADOW_LONG, open, high, low, close, lookback, 0);
    let mut avg_shadow_vshort = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        if real_body(open[i], close[i]) < avg_body.value(i, open, high, low, close) // small real body
            && upper_shadow(open[i], high[i], close[i]) > avg_shadow_long.value(i, open, high, low, close) // long upper shadow
            && lower_shadow(open[i], low[i], close[i]) < avg_shadow_vshort.value(i, open, high, low, close) // very short lower shadow
            && real_body_gap_down(open[i], close[i], open[i - 1], close[i - 1]) // gap down
        {
            out[i] = 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body.advance(i, open, high, low, close);
        avg_shadow_long.advance(i, open, high, low, close);
        avg_shadow_vshort.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_kicking — Kicking（反挂 / 踢腿）
// ===========================================================================

/// Kicking（反挂 / 踢腿）：2-candle 反转。两根反色光头光脚（marubozu）蜡烛之间存在跳空。
///
/// 输出由第 2 根蜡烛颜色决定：第 2 根为阳线（黑→白向上跳空）→ `+100`，为阴线
/// （白→黑向下跳空）→ `−100`。`BodyLong` 与 `ShadowVeryShort` 均使用 `OFF ∈ {1,0}`
/// （分别引用 `i−1` 与 `i`，对应第 1 / 第 2 根），`lookback = 11`。
/// 对应 `ta_CDLKICKING.c`。
///
/// Kicking: 2-candle reversal. Two opposite-color marubozu with a gap between them. Output is
/// `±100` by the 2nd candle's color. `BodyLong` & `ShadowVeryShort` `OFF ∈ {1,0}`, `lookback = 11`.
pub fn cdl_kicking(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_kicking_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_kicking` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_kicking`]。
/// Zero-copy variant of [`cdl_kicking`]: writes results into `out` (length must equal input length).
pub fn cdl_kicking_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_kicking_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_kicking")?;
    let n = open.len();
    let lookback = SHADOW_VERY_SHORT.avg_period.max(BODY_LONG.avg_period) + 1; // 11
    if n <= lookback {
        return Ok(());
    }
    let mut avg_body_1 = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 1);
    let mut avg_body_0 = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 0);
    let mut avg_sv_1 = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 1);
    let mut avg_sv_0 = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        let c_prev = candle_color(open[i - 1], close[i - 1]);
        let c_cur = candle_color(open[i], close[i]);
        if c_prev == -c_cur // opposite candles
            // 1st marubozu
            && real_body(open[i - 1], close[i - 1]) > avg_body_1.value(i, open, high, low, close)
            && upper_shadow(open[i - 1], high[i - 1], close[i - 1]) < avg_sv_1.value(i, open, high, low, close)
            && lower_shadow(open[i - 1], low[i - 1], close[i - 1]) < avg_sv_1.value(i, open, high, low, close)
            // 2nd marubozu
            && real_body(open[i], close[i]) > avg_body_0.value(i, open, high, low, close)
            && upper_shadow(open[i], high[i], close[i]) < avg_sv_0.value(i, open, high, low, close)
            && lower_shadow(open[i], low[i], close[i]) < avg_sv_0.value(i, open, high, low, close)
            // gap
            && (
                (c_prev == -1.0 && candle_gap_up(low[i], high[i - 1]))
                || (c_prev == 1.0 && candle_gap_down(high[i], low[i - 1]))
            )
        {
            out[i] = c_cur * 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body_1.advance(i, open, high, low, close);
        avg_body_0.advance(i, open, high, low, close);
        avg_sv_1.advance(i, open, high, low, close);
        avg_sv_0.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_kickingbylength — Kicking by Length（按长度踢腿）
// ===========================================================================

/// Kicking by Length（按长度踢腿）：条件与 `cdl_kicking` 完全相同，但输出方向由**两根
/// marubozu 中实体更长者**的颜色决定（而非第 2 根）。阳线更长 → `+100`，阴线更长 → `−100`。
///
/// `BodyLong` 与 `ShadowVeryShort` 均使用 `OFF ∈ {1,0}`，`lookback = 11`。
/// 对应 `ta_CDLKICKINGBYLENGTH.c`。
///
/// Kicking by Length: identical conditions to `cdl_kicking`, but the output sign is set by the
/// color of the *longer* marubozu body (not the 2nd candle). `BodyLong` & `ShadowVeryShort`
/// `OFF ∈ {1,0}`, `lookback = 11`.
pub fn cdl_kickingbylength(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_kickingbylength_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_kickingbylength` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_kickingbylength`]。
/// Zero-copy variant of [`cdl_kickingbylength`]: writes results into `out` (length must equal input length).
pub fn cdl_kickingbylength_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_kickingbylength_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_kickingbylength")?;
    let n = open.len();
    let lookback = SHADOW_VERY_SHORT.avg_period.max(BODY_LONG.avg_period) + 1; // 11
    if n <= lookback {
        return Ok(());
    }
    let mut avg_body_1 = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 1);
    let mut avg_body_0 = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 0);
    let mut avg_sv_1 = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 1);
    let mut avg_sv_0 = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        let c_prev = candle_color(open[i - 1], close[i - 1]);
        let c_cur = candle_color(open[i], close[i]);
        if c_prev == -c_cur // opposite candles
            // 1st marubozu
            && real_body(open[i - 1], close[i - 1]) > avg_body_1.value(i, open, high, low, close)
            && upper_shadow(open[i - 1], high[i - 1], close[i - 1]) < avg_sv_1.value(i, open, high, low, close)
            && lower_shadow(open[i - 1], low[i - 1], close[i - 1]) < avg_sv_1.value(i, open, high, low, close)
            // 2nd marubozu
            && real_body(open[i], close[i]) > avg_body_0.value(i, open, high, low, close)
            && upper_shadow(open[i], high[i], close[i]) < avg_sv_0.value(i, open, high, low, close)
            && lower_shadow(open[i], low[i], close[i]) < avg_sv_0.value(i, open, high, low, close)
            // gap
            && (
                (c_prev == -1.0 && candle_gap_up(low[i], high[i - 1]))
                || (c_prev == 1.0 && candle_gap_down(high[i], low[i - 1]))
            )
        {
            // 较长实体决定方向
            let longer = if real_body(open[i], close[i]) > real_body(open[i - 1], close[i - 1]) {
                i
            } else {
                i - 1
            };
            out[i] = candle_color(open[longer], close[longer]) * 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body_1.advance(i, open, high, low, close);
        avg_body_0.advance(i, open, high, low, close);
        avg_sv_1.advance(i, open, high, low, close);
        avg_sv_0.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_ladderbottom — Ladder Bottom（梯底）
// ===========================================================================

/// Ladder Bottom（梯底）：5-candle 底部反转。前三根阴线开盘与收盘连续走低，第 4 根阴线
/// 带上影线，第 5 根阳线开盘高于前一根实体、收盘高于前一根最高价。
///
/// 梯底恒为看涨（`+100`）。`ShadowVeryShort` 使用 `OFF=1`（引用 `i−1`，对应第 4 根的上影线
/// 比较），`lookback = ShadowVeryShort + 4 = 14`。对应 `ta_CDLLADDERBOTTOM.c`。
///
/// Ladder Bottom: 5-candle bottom reversal. Three declining black candles, a 4th black with an
/// upper shadow, and a 5th white that opens above the prior body and closes above the prior high.
/// Always bullish (`+100`). `ShadowVeryShort` `OFF=1` (refs `i−1`), `lookback = 14`.
pub fn cdl_ladderbottom(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_ladderbottom_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_ladderbottom` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_ladderbottom`]。
/// Zero-copy variant of [`cdl_ladderbottom`]: writes results into `out` (length must equal input length).
pub fn cdl_ladderbottom_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_ladderbottom_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_ladderbottom")?;
    let n = open.len();
    let lookback = SHADOW_VERY_SHORT.avg_period + 4; // 14
    if n <= lookback {
        return Ok(());
    }
    let mut avg_sv = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 1);
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 4], close[i - 4]) == -1.0 // 1st black
            && candle_color(open[i - 3], close[i - 3]) == -1.0 // 2nd black
            && candle_color(open[i - 2], close[i - 2]) == -1.0 // 3rd black
            && open[i - 4] > open[i - 3] // consecutively lower opens
            && open[i - 3] > open[i - 2]
            && close[i - 4] > close[i - 3] // and closes
            && close[i - 3] > close[i - 2]
            && candle_color(open[i - 1], close[i - 1]) == -1.0 // 4th: black with upper shadow
            && upper_shadow(open[i - 1], high[i - 1], close[i - 1]) > avg_sv.value(i, open, high, low, close)
            && candle_color(open[i], close[i]) == 1.0 // 5th: white
            && open[i] > open[i - 1] // opens above prior candle's body
            && close[i] > high[i - 1] // closes above prior candle's high
        {
            out[i] = 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_sv.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}

