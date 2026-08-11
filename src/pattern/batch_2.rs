//! # 形态识别 · 第 2 批 / Pattern Recognition · Batch 2
//!
//! 本批实现 8 个 3-candle / 4-candle 复合形态，均严格对齐 TA-Lib 0.7.1 黄金向量
//!（见 ADR 0005）。其中 `cdl_abandonedbaby` 采用默认 `penetration = 0.3`（与
//! `tools/gen_fixtures/generate.py` 调用 `talib.CDLABANDONEDBABY` 时的默认一致）。
//!
//! This batch implements 8 multi-candle composite patterns, bit-identical to the
//! TA-Lib 0.7.1 golden vectors (ADR 0005). `cdl_abandonedbaby` uses the default
//! `penetration = 0.3` (matching `talib.CDLABANDONEDBABY`'s default in the fixture
//! generator).

use crate::error::TaError;
use super::*;

// ===========================================================================
// cdl_3blackcrows — Three Black Crows（三只黑乌鸦）
// ===========================================================================

/// Three Black Crows（三只黑乌鸦）：3-candle 顶部反转，三根连续下跌阴线。
///
/// 恒为看跌（`−100`）。`ShadowVeryShort` 用 3 个 `CandleAvg`（off = 0/1/2）分别评估
/// 第 i / i−1 / i−2 根下影线。`lookback = ShadowVeryShort + 3 = 13`。对应 `ta_CDL3BLACKCROWS.c`。
///
/// Three Black Crows: 3-candle top reversal, three consecutive declining black candles.
/// Always bearish (`−100`). `ShadowVeryShort` uses 3 `CandleAvg` (off 0/1/2); `lookback = 13`.
pub fn cdl_3blackcrows(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_3blackcrows_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_3blackcrows` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_3blackcrows`]。
/// Zero-copy variant of [`cdl_3blackcrows`]: writes results into `out` (length must equal input length).
pub fn cdl_3blackcrows_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_3blackcrows_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_3blackcrows")?;
    let n = open.len();
    let lookback = SHADOW_VERY_SHORT.avg_period + 3; // 13
    if n <= lookback {
        return Ok(());
    }
    let mut avg_vs_0 = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 0);
    let mut avg_vs_1 = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 1);
    let mut avg_vs_2 = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 2);
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 3], close[i - 3]) == 1.0 // prior white
            && candle_color(open[i - 2], close[i - 2]) == -1.0 // 1st black
            && lower_shadow(open[i - 2], low[i - 2], close[i - 2])
                < avg_vs_2.value(i, open, high, low, close) // very short lower shadow
            && candle_color(open[i - 1], close[i - 1]) == -1.0 // 2nd black
            && lower_shadow(open[i - 1], low[i - 1], close[i - 1])
                < avg_vs_1.value(i, open, high, low, close)
            && candle_color(open[i], close[i]) == -1.0 // 3rd black
            && lower_shadow(open[i], low[i], close[i]) < avg_vs_0.value(i, open, high, low, close)
            && open[i - 1] < open[i - 2]
            && open[i - 1] > close[i - 2] // 2nd opens within 1st rb
            && open[i] < open[i - 1]
            && open[i] > close[i - 1] // 3rd opens within 2nd rb
            && high[i - 3] > close[i - 2] // 1st black closes under prior high
            && close[i - 2] > close[i - 1]
            && close[i - 1] > close[i] // three declining
        {
            out[i] = -100.0;
        } else {
            out[i] = 0.0;
        }
        avg_vs_0.advance(i, open, high, low, close);
        avg_vs_1.advance(i, open, high, low, close);
        avg_vs_2.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_3inside — Three Inside Up/Down（内困三合一）
// ===========================================================================

/// Three Inside Up/Down（内困三合一）：第 1 根长实体，第 2 根短实体被完全包裹，
/// 第 3 根反向并突破第 1 根开盘价。
///
/// 第 1 根为阳则看跌（`−100`），为阴则看涨（`+100`）。`BodyLong` off=2，`BodyShort` off=1，
/// `lookback = max(BodyLong, BodyShort) + 2 = 12`。对应 `ta_CDL3INSIDE.c`。
///
/// Three Inside Up/Down: 1st long body, 2nd short body engulfed by the 1st, 3rd opposite
/// closing past the 1st's open. `BodyLong` off=2, `BodyShort` off=1, `lookback = 12`.
pub fn cdl_3inside(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_3inside_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_3inside` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_3inside`]。
/// Zero-copy variant of [`cdl_3inside`]: writes results into `out` (length must equal input length).
pub fn cdl_3inside_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_3inside_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_3inside")?;
    let n = open.len();
    let lookback = BODY_SHORT.avg_period.max(BODY_LONG.avg_period) + 2; // 12
    if n <= lookback {
        return Ok(());
    }
    let mut avg_body_long = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 2);
    let mut avg_body_short = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 1);
    let mut i = lookback;
    while i < n {
        if real_body(open[i - 2], close[i - 2]) > avg_body_long.value(i, open, high, low, close) // 1st long
            && real_body(open[i - 1], close[i - 1]) <= avg_body_short.value(i, open, high, low, close) // 2nd short
            && open[i - 1].max(close[i - 1]) < open[i - 2].max(close[i - 2]) // engulfed by 1st
            && open[i - 1].min(close[i - 1]) > open[i - 2].min(close[i - 2])
            && ((candle_color(open[i - 2], close[i - 2]) == 1.0
                && candle_color(open[i], close[i]) == -1.0
                && close[i] < open[i - 2])
                || (candle_color(open[i - 2], close[i - 2]) == -1.0
                    && candle_color(open[i], close[i]) == 1.0
                    && close[i] > open[i - 2]))
        {
            out[i] = -candle_color(open[i - 2], close[i - 2]) * 100.0;
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
// cdl_3linestrike — Three-Line Strike（三线打击）
// ===========================================================================

/// Three-Line Strike（三线打击）：三连阳（阴）后第 4 根反向大阴（阳）线。
///
/// 三连阳则看涨（`+100`），三连阴则看跌（`−100`）。`Near` 用 2 个 `CandleAvg`（off = 2/3）
/// 评估第 2、3 根开盘的相对位置，`lookback = Near + 3 = 8`。对应 `ta_CDL3LINESTRIKE.c`。
///
/// Three-Line Strike: three white (black) soldiers followed by a 4th opposite black (white).
/// `Near` uses 2 `CandleAvg` (off 2/3); `lookback = 8`. Output `±100` by the soldiers' color.
pub fn cdl_3linestrike(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_3linestrike_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_3linestrike` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_3linestrike`]。
/// Zero-copy variant of [`cdl_3linestrike`]: writes results into `out` (length must equal input length).
pub fn cdl_3linestrike_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_3linestrike_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_3linestrike")?;
    let n = open.len();
    let lookback = NEAR.avg_period + 3; // 8
    if n <= lookback {
        return Ok(());
    }
    let mut avg_near_3 = CandleAvg::new(NEAR, open, high, low, close, lookback, 3);
    let mut avg_near_2 = CandleAvg::new(NEAR, open, high, low, close, lookback, 2);
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 3], close[i - 3]) == candle_color(open[i - 2], close[i - 2])
            && candle_color(open[i - 2], close[i - 2]) == candle_color(open[i - 1], close[i - 1])
            && candle_color(open[i], close[i]) == -candle_color(open[i - 1], close[i - 1]) // 4th opposite
            // 2nd opens within/near 1st rb
            && open[i - 2] >= open[i - 3].min(close[i - 3]) - avg_near_3.value(i, open, high, low, close)
            && open[i - 2] <= open[i - 3].max(close[i - 3]) + avg_near_3.value(i, open, high, low, close)
            // 3rd opens within/near 2nd rb
            && open[i - 1] >= open[i - 2].min(close[i - 2]) - avg_near_2.value(i, open, high, low, close)
            && open[i - 1] <= open[i - 2].max(close[i - 2]) + avg_near_2.value(i, open, high, low, close)
            && ((candle_color(open[i - 1], close[i - 1]) == 1.0
                && close[i - 1] > close[i - 2]
                && close[i - 2] > close[i - 3] // consecutive higher closes
                && open[i] > close[i - 1] // 4th opens above prior close
                && close[i] < open[i - 3]) // 4th closes below 1st open
                || (candle_color(open[i - 1], close[i - 1]) == -1.0
                    && close[i - 1] < close[i - 2]
                    && close[i - 2] < close[i - 3] // consecutive lower closes
                    && open[i] < close[i - 1] // 4th opens below prior close
                    && close[i] > open[i - 3]))
        {
            out[i] = candle_color(open[i - 1], close[i - 1]) * 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_near_3.advance(i, open, high, low, close);
        avg_near_2.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_3outside — Three Outside Up/Down（外侧三合一）
// ===========================================================================

/// Three Outside Up/Down（外侧三合一）：第 2 根吞没第 1 根，第 3 根顺势收盘。
///
/// 三外升看涨（`+100`），三外降看跌（`−100`）。纯 3-candle 比较，无蜡烛设置，
/// `lookback = 3`。对应 `ta_CDL3OUTSIDE.c`。
///
/// Three Outside Up/Down: 2nd engulfs the 1st, 3rd closes in the same direction.
/// Pure 3-candle comparison, no candle settings, `lookback = 3`. Output `±100`.
pub fn cdl_3outside(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_3outside_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_3outside` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_3outside`]。
/// Zero-copy variant of [`cdl_3outside`]: writes results into `out` (length must equal input length).
pub fn cdl_3outside_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_3outside_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_3outside")?;
    let n = open.len();
    let lookback = 3; // TA_CDL3OUTSIDE_Lookback() returns 3
    if n <= lookback {
        return Ok(());
    }
    let mut i = lookback;
    while i < n {
        if (candle_color(open[i - 1], close[i - 1]) == 1.0
            && candle_color(open[i - 2], close[i - 2]) == -1.0 // white engulfs black
            && close[i - 1] > open[i - 2]
            && open[i - 1] < close[i - 2]
            && close[i] > close[i - 1]) // third higher
            || (candle_color(open[i - 1], close[i - 1]) == -1.0
                && candle_color(open[i - 2], close[i - 2]) == 1.0 // black engulfs white
                && open[i - 1] > close[i - 2]
                && close[i - 1] < open[i - 2]
                && close[i] < close[i - 1])
        {
            out[i] = candle_color(open[i - 1], close[i - 1]) * 100.0;
        } else {
            out[i] = 0.0;
        }
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_3starsinsouth — Three Stars In The South（南方三星）
// ===========================================================================

/// Three Stars In The South（南方三星）：底部反转，三根阴线，第 3 根为小墓碑线被前一根包裹。
///
/// 恒为看涨（`+100`）。使用 `BodyLong`(off=2)、`ShadowLong`(off=2, avgPeriod=0)、
/// `ShadowVeryShort`(off=1/0)、`BodyShort`(off=0)。`lookback = max(max(ShadowVeryShort,
/// ShadowLong), max(BodyLong, BodyShort)) + 2 = 12`。对应 `ta_CDL3STARSSOUTH.c`。
///
/// Three Stars In The South: bullish bottom reversal, three black candles, 3rd a small
/// marubozu engulfed by the 2nd. `lookback = 12`. Always bullish (`+100`).
pub fn cdl_3starsinsouth(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_3starsinsouth_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_3starsinsouth` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_3starsinsouth`]。
/// Zero-copy variant of [`cdl_3starsinsouth`]: writes results into `out` (length must equal input length).
pub fn cdl_3starsinsouth_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_3starsinsouth_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_3starsinsouth")?;
    let n = open.len();
    let lookback = [
        SHADOW_VERY_SHORT.avg_period.max(SHADOW_LONG.avg_period),
        BODY_LONG.avg_period.max(BODY_SHORT.avg_period),
    ]
    .iter()
    .max()
    .copied()
    .unwrap()
        + 2; // 12
    if n <= lookback {
        return Ok(());
    }
    let mut avg_body_long = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 2);
    let mut avg_shadow_long = CandleAvg::new(SHADOW_LONG, open, high, low, close, lookback, 2);
    let mut avg_vs_1 = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 1);
    let mut avg_vs_0 = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 0);
    let mut avg_body_short = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 2], close[i - 2]) == -1.0 // 1st black
            && candle_color(open[i - 1], close[i - 1]) == -1.0 // 2nd black
            && candle_color(open[i], close[i]) == -1.0 // 3rd black
            // 1st: long with long lower shadow
            && real_body(open[i - 2], close[i - 2]) > avg_body_long.value(i, open, high, low, close)
            && lower_shadow(open[i - 2], low[i - 2], close[i - 2])
                > avg_shadow_long.value(i, open, high, low, close)
            && real_body(open[i - 1], close[i - 1]) < real_body(open[i - 2], close[i - 2]) // 2nd smaller
            && open[i - 1] > close[i - 2]
            && open[i - 1] <= high[i - 2] // opens higher but within 1st range
            && low[i - 1] < close[i - 2] // trades lower than 1st close
            && low[i - 1] >= low[i - 2] // but not lower than 1st low
            // 2nd has a lower shadow
            && lower_shadow(open[i - 1], low[i - 1], close[i - 1])
                > avg_vs_1.value(i, open, high, low, close)
            // 3rd: small marubozu engulfed by 2nd range
            && real_body(open[i], close[i]) < avg_body_short.value(i, open, high, low, close)
            && lower_shadow(open[i], low[i], close[i]) < avg_vs_0.value(i, open, high, low, close)
            && upper_shadow(open[i], high[i], close[i]) < avg_vs_0.value(i, open, high, low, close)
            && low[i] > low[i - 1]
            && high[i] < high[i - 1]
        {
            out[i] = 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body_long.advance(i, open, high, low, close);
        avg_shadow_long.advance(i, open, high, low, close);
        avg_vs_1.advance(i, open, high, low, close);
        avg_vs_0.advance(i, open, high, low, close);
        avg_body_short.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_3whitesoldiers — Three Advancing White Soldiers（三白兵）
// ===========================================================================

/// Three Advancing White Soldiers（三白兵）：三连阳、收盘递升、上影线极短。
///
/// 恒为看涨（`+100`）。使用 `ShadowVeryShort`(off=0/1/2)、`Near`(off=1/2)、`Far`(off=1/2)、
/// `BodyShort`(off=0)。`lookback = max(max(ShadowVeryShort, BodyShort), max(Far, Near)) + 2 = 12`。
/// 对应 `ta_CDL3WHITESOLDIERS.c`。
///
/// Three Advancing White Soldiers: three white candles, higher closes, very short upper shadows.
/// Always bullish (`+100`). `lookback = 12`.
pub fn cdl_3whitesoldiers(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_3whitesoldiers_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_3whitesoldiers` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_3whitesoldiers`]。
/// Zero-copy variant of [`cdl_3whitesoldiers`]: writes results into `out` (length must equal input length).
pub fn cdl_3whitesoldiers_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_3whitesoldiers_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_3whitesoldiers")?;
    let n = open.len();
    let lookback = [
        SHADOW_VERY_SHORT.avg_period.max(BODY_SHORT.avg_period),
        FAR.avg_period.max(NEAR.avg_period),
    ]
    .iter()
    .max()
    .copied()
    .unwrap()
        + 2; // 12
    if n <= lookback {
        return Ok(());
    }
    let mut avg_vs_2 = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 2);
    let mut avg_vs_1 = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 1);
    let mut avg_vs_0 = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 0);
    let mut avg_near_2 = CandleAvg::new(NEAR, open, high, low, close, lookback, 2);
    let mut avg_near_1 = CandleAvg::new(NEAR, open, high, low, close, lookback, 1);
    let mut avg_far_2 = CandleAvg::new(FAR, open, high, low, close, lookback, 2);
    let mut avg_far_1 = CandleAvg::new(FAR, open, high, low, close, lookback, 1);
    let mut avg_body_short = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 2], close[i - 2]) == 1.0 // 1st white
            && upper_shadow(open[i - 2], high[i - 2], close[i - 2])
                < avg_vs_2.value(i, open, high, low, close) // very short upper shadow
            && candle_color(open[i - 1], close[i - 1]) == 1.0 // 2nd white
            && upper_shadow(open[i - 1], high[i - 1], close[i - 1])
                < avg_vs_1.value(i, open, high, low, close)
            && candle_color(open[i], close[i]) == 1.0 // 3rd white
            && upper_shadow(open[i], high[i], close[i]) < avg_vs_0.value(i, open, high, low, close)
            && close[i] > close[i - 1]
            && close[i - 1] > close[i - 2] // consecutive higher closes
            && open[i - 1] > open[i - 2] // 2nd opens within/near 1st rb
            && open[i - 1] <= close[i - 2] + avg_near_2.value(i, open, high, low, close)
            && open[i] > open[i - 1] // 3rd opens within/near 2nd rb
            && open[i] <= close[i - 1] + avg_near_1.value(i, open, high, low, close)
            && real_body(open[i - 1], close[i - 1])
                > real_body(open[i - 2], close[i - 2]) - avg_far_2.value(i, open, high, low, close) // 2nd not far shorter
            && real_body(open[i], close[i])
                > real_body(open[i - 1], close[i - 1]) - avg_far_1.value(i, open, high, low, close) // 3rd not far shorter
            && real_body(open[i], close[i]) > avg_body_short.value(i, open, high, low, close) // not short
        {
            out[i] = 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_vs_2.advance(i, open, high, low, close);
        avg_vs_1.advance(i, open, high, low, close);
        avg_vs_0.advance(i, open, high, low, close);
        avg_near_2.advance(i, open, high, low, close);
        avg_near_1.advance(i, open, high, low, close);
        avg_far_2.advance(i, open, high, low, close);
        avg_far_1.advance(i, open, high, low, close);
        avg_body_short.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_abandonedbaby — Abandoned Baby（弃婴形态）
// ===========================================================================

/// Abandoned Baby（弃婴形态）：第 1 根长实体，第 2 根十字星（与两侧跳空），第 3 根反向实体。
///
/// 底部弃婴看涨（`+100`），顶部弃婴看跌（`−100`）。`BodyLong`(off=2)、`BodyDoji`(off=1)、
/// `BodyShort`(off=0)，`penetration` 固定 0.3（与 fixture 生成一致）。`lookback =
/// max(max(BodyDoji, BodyLong), BodyShort) + 2 = 12`。对应 `ta_CDLABANDONEDBABY.c`。
///
/// Abandoned Baby: long body, doji gapping both sides, opposite-color 3rd body. Output is the
/// 3rd candle's color × 100. `penetration = 0.3`; `lookback = 12`.
pub fn cdl_abandonedbaby(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_abandonedbaby_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_abandonedbaby` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_abandonedbaby`]。
/// Zero-copy variant of [`cdl_abandonedbaby`]: writes results into `out` (length must equal input length).
pub fn cdl_abandonedbaby_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_abandonedbaby_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_abandonedbaby")?;
    let n = open.len();
    let lookback = [
        BODY_DOJI.avg_period.max(BODY_LONG.avg_period),
        BODY_SHORT.avg_period,
    ]
    .iter()
    .max()
    .copied()
    .unwrap()
        + 2; // 12
    let penetration: f64 = 0.3; // TA_CDLABANDONEDBABY default optInPenetration
    if n <= lookback {
        return Ok(());
    }
    let mut avg_body_long = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 2);
    let mut avg_body_doji = CandleAvg::new(BODY_DOJI, open, high, low, close, lookback, 1);
    let mut avg_body_short = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        if real_body(open[i - 2], close[i - 2]) > avg_body_long.value(i, open, high, low, close) // 1st long
            && real_body(open[i - 1], close[i - 1]) <= avg_body_doji.value(i, open, high, low, close) // 2nd doji
            && real_body(open[i], close[i]) > avg_body_short.value(i, open, high, low, close) // 3rd not short
            && ((candle_color(open[i - 2], close[i - 2]) == 1.0 // 1st white, baby bottom top
                && candle_color(open[i], close[i]) == -1.0 // 3rd black
                && close[i] < close[i - 2] - real_body(open[i - 2], close[i - 2]) * penetration
                && candle_gap_up(low[i - 1], high[i - 2]) // upside gap 1st-2nd
                && candle_gap_down(high[i], low[i - 1])) // downside gap 2nd-3rd
                || (candle_color(open[i - 2], close[i - 2]) == -1.0 // 1st black
                    && candle_color(open[i], close[i]) == 1.0 // 3rd white
                    && close[i] > close[i - 2] + real_body(open[i - 2], close[i - 2]) * penetration
                    && candle_gap_down(high[i - 1], low[i - 2]) // downside gap 1st-2nd
                    && candle_gap_up(low[i], high[i - 1])))
        {
            out[i] = candle_color(open[i], close[i]) * 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body_long.advance(i, open, high, low, close);
        avg_body_doji.advance(i, open, high, low, close);
        avg_body_short.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_advanceblock — Advance Block（推进块）
// ===========================================================================

/// Advance Block（推进块）：三连阳但出现疲弱信号（实体渐小 / 上影线变长）。
///
/// 恒为看跌（`−100`）。使用 `ShadowShort`(off=0/1/2)、`ShadowLong`(off=0/1, avgPeriod=0)、
/// `Near`(off=1/2)、`Far`(off=1/2)、`BodyLong`(off=2)。`lookback =
/// max(max(max(ShadowLong, ShadowShort), max(Far, Near)), BodyLong) + 2 = 12`。对应
/// `ta_CDLADVANCEBLOCK.c`。注意：C 源第 2 个分支用 `Near` 而非 `Far`（与原版 dylib 一致）。
///
/// Advance Block: three white candles showing weakness (shrinking bodies / longer upper shadows).
/// Always bearish (`−100`). `lookback = 12`. Note the 2nd sub-branch uses `Near` per the C source.
pub fn cdl_advanceblock(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_advanceblock_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_advanceblock` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_advanceblock`]。
/// Zero-copy variant of [`cdl_advanceblock`]: writes results into `out` (length must equal input length).
pub fn cdl_advanceblock_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_advanceblock_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_advanceblock")?;
    let n = open.len();
    let lookback = [
        SHADOW_LONG.avg_period.max(SHADOW_SHORT.avg_period),
        FAR.avg_period.max(NEAR.avg_period),
        BODY_LONG.avg_period,
    ]
    .iter()
    .max()
    .copied()
    .unwrap()
        + 2; // 12
    if n <= lookback {
        return Ok(());
    }
    let mut avg_shadow_short_2 = CandleAvg::new(SHADOW_SHORT, open, high, low, close, lookback, 2);
    let mut avg_shadow_short_1 = CandleAvg::new(SHADOW_SHORT, open, high, low, close, lookback, 1);
    let mut avg_shadow_short_0 = CandleAvg::new(SHADOW_SHORT, open, high, low, close, lookback, 0);
    let mut avg_shadow_long_1 = CandleAvg::new(SHADOW_LONG, open, high, low, close, lookback, 1);
    let mut avg_shadow_long_0 = CandleAvg::new(SHADOW_LONG, open, high, low, close, lookback, 0);
    let mut avg_near_2 = CandleAvg::new(NEAR, open, high, low, close, lookback, 2);
    let mut avg_near_1 = CandleAvg::new(NEAR, open, high, low, close, lookback, 1);
    let mut avg_far_2 = CandleAvg::new(FAR, open, high, low, close, lookback, 2);
    let mut avg_far_1 = CandleAvg::new(FAR, open, high, low, close, lookback, 1);
    let mut avg_body_long = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 2);
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 2], close[i - 2]) == 1.0 // 1st white
            && candle_color(open[i - 1], close[i - 1]) == 1.0 // 2nd white
            && candle_color(open[i], close[i]) == 1.0 // 3rd white
            && close[i] > close[i - 1]
            && close[i - 1] > close[i - 2] // consecutive higher closes
            && open[i - 1] > open[i - 2] // 2nd opens within/near 1st rb
            && open[i - 1] <= close[i - 2] + avg_near_2.value(i, open, high, low, close)
            && open[i] > open[i - 1] // 3rd opens within/near 2nd rb
            && open[i] <= close[i - 1] + avg_near_1.value(i, open, high, low, close)
            && real_body(open[i - 2], close[i - 2]) > avg_body_long.value(i, open, high, low, close) // 1st long
            && upper_shadow(open[i - 2], high[i - 2], close[i - 2])
                < avg_shadow_short_2.value(i, open, high, low, close) // 1st short upper shadow
            && ((real_body(open[i - 1], close[i - 1])
                < real_body(open[i - 2], close[i - 2]) - avg_far_2.value(i, open, high, low, close)
                && real_body(open[i], close[i])
                    < real_body(open[i - 1], close[i - 1])
                        + avg_near_1.value(i, open, high, low, close)) // 2nd far smaller
                || (real_body(open[i], close[i])
                    < real_body(open[i - 1], close[i - 1])
                        - avg_far_1.value(i, open, high, low, close)) // 3rd far smaller
                || (real_body(open[i], close[i]) < real_body(open[i - 1], close[i - 1])
                    && real_body(open[i - 1], close[i - 1]) < real_body(open[i - 2], close[i - 2])
                    && (upper_shadow(open[i], high[i], close[i])
                        > avg_shadow_short_0.value(i, open, high, low, close)
                        || upper_shadow(open[i - 1], high[i - 1], close[i - 1])
                            > avg_shadow_short_1.value(i, open, high, low, close)))
                || (real_body(open[i], close[i]) < real_body(open[i - 1], close[i - 1])
                    && upper_shadow(open[i], high[i], close[i])
                        > avg_shadow_long_0.value(i, open, high, low, close)))
        {
            out[i] = -100.0;
        } else {
            out[i] = 0.0;
        }
        avg_shadow_short_2.advance(i, open, high, low, close);
        avg_shadow_short_1.advance(i, open, high, low, close);
        avg_shadow_short_0.advance(i, open, high, low, close);
        avg_shadow_long_1.advance(i, open, high, low, close);
        avg_shadow_long_0.advance(i, open, high, low, close);
        avg_near_2.advance(i, open, high, low, close);
        avg_near_1.advance(i, open, high, low, close);
        avg_far_2.advance(i, open, high, low, close);
        avg_far_1.advance(i, open, high, low, close);
        avg_body_long.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}

