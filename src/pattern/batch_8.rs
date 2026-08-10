//! # 形态识别 · 第 8 批 / Pattern Recognition · Batch 8
//!
//! 本批实现 8 个蜡烛形态，全部与 TA-Lib 0.7.1 黄金向量逐项 1:1：
//! `cdl_sticksandwich`、`cdl_takuri`、`cdl_tasukigap`、`cdl_thrusting`、
//! `cdl_tristar`、`cdl_unique3river`、`cdl_upsidegap2crows`、`cdl_xsidegap3methods`。
//!
//! This batch implements 8 candlestick patterns, all bit-identical to TA-Lib 0.7.1 golden
//! vectors: `cdl_sticksandwich`, `cdl_takuri`, `cdl_tasukigap`, `cdl_thrusting`,
//! `cdl_tristar`, `cdl_unique3river`, `cdl_upsidegap2crows`, `cdl_xsidegap3methods`.

use crate::error::TaError;
use super::*;

// ===========================================================================
// cdl_sticksandwich — Stick Sandwich（棍子三明治）
// ===========================================================================

/// Stick Sandwich（棍子三明治）：第 1 根阴线、第 2 根阳线（低点高于前收）、第 3 根阴线收盘≈第 1 根收盘。
///
/// 恒为看涨 `100`。`EQUAL` 引用 `i−2`，`lookback = EQUAL + 2 = 7`，`off = 2`。
///
/// Stick Sandwich: 1st black, 2nd white (low above prior close), 3rd black closing equal to the
/// 1st close. Always bullish `100`. `EQUAL` references `i−2`, `lookback = 7`, `off = 2`.
pub fn cdl_sticksandwich(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_ohlc(open, high, low, close, "cdl_sticksandwich")?;
    let n = open.len();
    let lookback = EQUAL.avg_period + 2; // 7
    let mut out = vec![0.0_f64; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut avg_eq = CandleAvg::new(EQUAL, open, high, low, close, lookback, 2);
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 2], close[i - 2]) == -1.0 // 1st black
            && candle_color(open[i - 1], close[i - 1]) == 1.0 // 2nd white
            && candle_color(open[i], close[i]) == -1.0 // 3rd black
            && low[i - 1] > close[i - 2] // 2nd low > prior close
            && close[i] <= close[i - 2] + avg_eq.value(i, open, high, low, close) // 1st & 3rd same close
            && close[i] >= close[i - 2] - avg_eq.value(i, open, high, low, close)
        {
            out[i] = 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_eq.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(out)
}

// ===========================================================================
// cdl_takuri — Takuri (Dragonfly Doji with very long lower shadow)
// ===========================================================================

/// Takuri（蜻蜓十字星）：极小的实体 + 开盘/收盘都在当日最高（上影线极短）+ 极长下影线。
///
/// 恒为 `100`（相对趋势判断，本身不判多空）。三个设置均引用当前 K 线 `i`，`off = 0`；
/// `lookback = max(BodyDoji, ShadowVeryShort, ShadowVeryLong) = 10`。
///
/// Takuri: doji body, open & close at the high (very short upper shadow), very long lower shadow.
/// Always `100`. All three settings reference `i`, `off = 0`; `lookback = 10`.
pub fn cdl_takuri(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_ohlc(open, high, low, close, "cdl_takuri")?;
    let n = open.len();
    let lookback = BODY_DOJI
        .avg_period
        .max(SHADOW_VERY_SHORT.avg_period)
        .max(SHADOW_VERY_LONG.avg_period); // 10
    let mut out = vec![0.0_f64; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut avg_body = CandleAvg::new(BODY_DOJI, open, high, low, close, lookback, 0);
    let mut avg_us = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 0);
    let mut avg_ls = CandleAvg::new(SHADOW_VERY_LONG, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        if real_body(open[i], close[i]) <= avg_body.value(i, open, high, low, close)
            && upper_shadow(open[i], high[i], close[i]) < avg_us.value(i, open, high, low, close)
            && lower_shadow(open[i], low[i], close[i]) > avg_ls.value(i, open, high, low, close)
        {
            out[i] = 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body.advance(i, open, high, low, close);
        avg_us.advance(i, open, high, low, close);
        avg_ls.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(out)
}

// ===========================================================================
// cdl_tasukigap — Tasuki Gap（跳空并列阴阳线）
// ===========================================================================

/// Tasuki Gap（跳空并列阴阳线）：向上/向下跳空后的并列两实体，第 2 根反向且实体大小相近。
///
/// 看涨方向输出 `+100`，看跌方向输出 `−100`。`NEAR` 引用 `i−1`，`lookback = NEAR + 2 = 7`，`off = 1`。
///
/// Tasuki Gap: a gapped pair followed by a counter-coloured candle whose real body stays inside
/// the gap and is near the same size as the prior body. Bullish → `+100`, bearish → `−100`.
/// `NEAR` references `i−1`, `lookback = 7`, `off = 1`.
pub fn cdl_tasukigap(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_ohlc(open, high, low, close, "cdl_tasukigap")?;
    let n = open.len();
    let lookback = NEAR.avg_period + 2; // 7
    let mut out = vec![0.0_f64; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut avg_near = CandleAvg::new(NEAR, open, high, low, close, lookback, 1);
    let mut i = lookback;
    while i < n {
        if (real_body_gap_up(open[i - 1], close[i - 1], open[i - 2], close[i - 2]) // upside gap
            && candle_color(open[i - 1], close[i - 1]) == 1.0 // 1st white
            && candle_color(open[i], close[i]) == -1.0 // 2nd black
            && open[i] < close[i - 1] && open[i] > open[i - 1] // opens within white body
            && close[i] < open[i - 1] // closes under white body
            && close[i] > f64::max(close[i - 2], open[i - 2]) // inside the gap
            && (real_body(open[i - 1], close[i - 1]) - real_body(open[i], close[i])).abs()
                < avg_near.value(i, open, high, low, close)
        ) || (real_body_gap_down(open[i - 1], close[i - 1], open[i - 2], close[i - 2]) // downside gap
            && candle_color(open[i - 1], close[i - 1]) == -1.0 // 1st black
            && candle_color(open[i], close[i]) == 1.0 // 2nd white
            && open[i] < open[i - 1] && open[i] > close[i - 1] // opens within black body
            && close[i] > open[i - 1] // closes above black body
            && close[i] < f64::min(close[i - 2], open[i - 2]) // inside the gap
            && (real_body(open[i - 1], close[i - 1]) - real_body(open[i], close[i])).abs()
                < avg_near.value(i, open, high, low, close)
        ) {
            out[i] = candle_color(open[i - 1], close[i - 1]) * 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_near.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(out)
}

// ===========================================================================
// cdl_thrusting — Thrusting Pattern（插入线 / 推力形态）
// ===========================================================================

/// Thrusting Pattern（推力形态）：长阴线后，阳线跳空低开、收盘深入前阴实体但未过中点。
///
/// 恒为看跌 `−100`（与颈上线 in-neck 类似，但收盘不等于阴线收盘）。`EQUAL` 与 `BODY_LONG` 均引用
/// `i−1`，`lookback = max(Equal, BodyLong) + 1 = 11`，`off = 1`。
///
/// Thrusting Pattern: long black candle, then a white candle gapping down that closes into the
/// prior black body but under its midpoint. Always bearish `−100`. `EQUAL` and `BODY_LONG`
/// reference `i−1`, `lookback = 11`, `off = 1`.
pub fn cdl_thrusting(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_ohlc(open, high, low, close, "cdl_thrusting")?;
    let n = open.len();
    let lookback = EQUAL.avg_period.max(BODY_LONG.avg_period) + 1; // 11
    let mut out = vec![0.0_f64; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut avg_eq = CandleAvg::new(EQUAL, open, high, low, close, lookback, 1);
    let mut avg_body = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 1);
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 1], close[i - 1]) == -1.0 // 1st black
            && real_body(open[i - 1], close[i - 1]) > avg_body.value(i, open, high, low, close) // long
            && candle_color(open[i], close[i]) == 1.0 // 2nd white
            && open[i] < low[i - 1] // open below prior low
            && close[i] > close[i - 1] + avg_eq.value(i, open, high, low, close) // close into prior body
            && close[i] <= close[i - 1] + real_body(open[i - 1], close[i - 1]) * 0.5 // under midpoint
        {
            out[i] = -100.0;
        } else {
            out[i] = 0.0;
        }
        avg_eq.advance(i, open, high, low, close);
        avg_body.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(out)
}

// ===========================================================================
// cdl_tristar — Tristar Pattern（三星形态）
// ===========================================================================

/// Tristar Pattern（三星形态）：连续 3 根十字星，第 2 根为星线（跳空）。
///
/// 第 2 根向上跳空且第 3 根不高于第 2 根 → 看跌 `−100`；向下跳空且第 3 根不低于第 2 根 → 看涨 `+100`；
/// 否则 `0`。`BODY_DOJI` 引用 `i−2`，`lookback = BodyDoji + 2 = 12`，`off = 2`。
///
/// Tristar: three consecutive doji, the 2nd being a star (gapped). 2nd gaps up & 3rd not higher
/// → `−100`; 2nd gaps down & 3rd not lower → `+100`; else `0`. `BODY_DOJI` references `i−2`,
/// `lookback = 12`, `off = 2`.
pub fn cdl_tristar(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_ohlc(open, high, low, close, "cdl_tristar")?;
    let n = open.len();
    let lookback = BODY_DOJI.avg_period + 2; // 12
    let mut out = vec![0.0_f64; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut avg_body = CandleAvg::new(BODY_DOJI, open, high, low, close, lookback, 2);
    let mut i = lookback;
    while i < n {
        if real_body(open[i - 2], close[i - 2]) <= avg_body.value(i, open, high, low, close) // 1st doji
            && real_body(open[i - 1], close[i - 1]) <= avg_body.value(i, open, high, low, close) // 2nd doji
            && real_body(open[i], close[i]) <= avg_body.value(i, open, high, low, close) // 3rd doji
        {
            out[i] = 0.0;
            if real_body_gap_up(open[i - 1], close[i - 1], open[i - 2], close[i - 2]) // 2nd gaps up
                && f64::max(open[i], close[i]) < f64::max(open[i - 1], close[i - 1])
            {
                out[i] = -100.0;
            }
            if real_body_gap_down(open[i - 1], close[i - 1], open[i - 2], close[i - 2]) // 2nd gaps down
                && f64::min(open[i], close[i]) > f64::min(open[i - 1], close[i - 1])
            {
                out[i] = 100.0;
            }
        } else {
            out[i] = 0.0;
        }
        avg_body.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(out)
}

// ===========================================================================
// cdl_unique3river — Unique 3 River（独特三河床）
// ===========================================================================

/// Unique 3 River（独特三河床）：长阴线、更低低的黑色抱线星、不破前低的白色小实体。
///
/// 恒为看涨 `100`。`BODY_LONG` 引用 `i−2`（`off = 2`）；`BODY_SHORT` 引用当前 `i`（`off = 0`）；
/// `lookback = max(BodyShort, BodyLong) + 2 = 12`。
///
/// Unique 3 River: long black, a lower-low black harami star, a small white real body whose open
/// is not below the prior low. Always bullish `100`. `BODY_LONG` references `i−2` (`off = 2`);
/// `BODY_SHORT` references `i` (`off = 0`); `lookback = 12`.
pub fn cdl_unique3river(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_ohlc(open, high, low, close, "cdl_unique3river")?;
    let n = open.len();
    let lookback = BODY_SHORT.avg_period.max(BODY_LONG.avg_period) + 2; // 12
    let mut out = vec![0.0_f64; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut avg_body_long = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 2);
    let mut avg_body_short = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        if real_body(open[i - 2], close[i - 2]) > avg_body_long.value(i, open, high, low, close) // 1st long
            && candle_color(open[i - 2], close[i - 2]) == -1.0 // black
            && candle_color(open[i - 1], close[i - 1]) == -1.0 // 2nd black
            && close[i - 1] > close[i - 2] && open[i - 1] <= open[i - 2] // harami
            && low[i - 1] < low[i - 2] // lower low
            && real_body(open[i], close[i]) < avg_body_short.value(i, open, high, low, close) // 3rd short
            && candle_color(open[i], close[i]) == 1.0 // white
            && open[i] > low[i - 1] // open not lower
        {
            out[i] = 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body_long.advance(i, open, high, low, close);
        avg_body_short.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(out)
}

// ===========================================================================
// cdl_upsidegap2crows — Upside Gap Two Crows（向上跳空两只乌鸦）
// ===========================================================================

/// Upside Gap Two Crows（向上跳空两只乌鸦）：长阳线、向上跳空的小黑线、吞没前实体且收在首阳之上的黑线。
///
/// 恒为看跌 `−100`。`BODY_LONG` 引用 `i−2`（`off = 2`）；`BODY_SHORT` 引用 `i−1`（`off = 1`）；
/// `lookback = max(BodyShort, BodyLong) + 2 = 12`。
///
/// Upside Gap Two Crows: long white, a small black real body gapping up, then a black candle
/// engulfing the prior body and closing above the white close. Always bearish `−100`.
/// `BODY_LONG` references `i−2` (`off = 2`); `BODY_SHORT` references `i−1` (`off = 1`); `lookback = 12`.
pub fn cdl_upsidegap2crows(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_ohlc(open, high, low, close, "cdl_upsidegap2crows")?;
    let n = open.len();
    let lookback = BODY_SHORT.avg_period.max(BODY_LONG.avg_period) + 2; // 12
    let mut out = vec![0.0_f64; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut avg_body_long = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 2);
    let mut avg_body_short = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 1);
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 2], close[i - 2]) == 1.0 // 1st white
            && real_body(open[i - 2], close[i - 2]) > avg_body_long.value(i, open, high, low, close) // long
            && candle_color(open[i - 1], close[i - 1]) == -1.0 // 2nd black
            && real_body(open[i - 1], close[i - 1]) <= avg_body_short.value(i, open, high, low, close) // short
            && real_body_gap_up(open[i - 1], close[i - 1], open[i - 2], close[i - 2]) // gapping up
            && candle_color(open[i], close[i]) == -1.0 // 3rd black
            && open[i] > open[i - 1] && close[i] < close[i - 1] // 3rd engulfs prior body
            && close[i] > close[i - 2] // closes above 1st
        {
            out[i] = -100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body_long.advance(i, open, high, low, close);
        avg_body_short.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(out)
}

// ===========================================================================
// cdl_xsidegap3methods — Upside/Downside Gap Three Methods（上升/下降跳空三法）
// ===========================================================================

/// Upside/Downside Gap Three Methods（上升/下降跳空三法）：同向两根蜡烛跳空，第 3 根反向、开在
/// 第 2 根实体内、收在第 1 根实体内。
///
/// 首根为阳 → 看涨 `+100`；首根为阴 → 看跌 `−100`。无蜡烛均值设置，`lookback = 2`。
///
/// Upside/Downside Gap Three Methods: two same-colour candles gapping, then a counter-colour
/// candle opening inside the 2nd real body and closing inside the 1st real body. 1st white →
/// `+100`, 1st black → `−100`. No candle-average settings; `lookback = 2`.
pub fn cdl_xsidegap3methods(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_ohlc(open, high, low, close, "cdl_xsidegap3methods")?;
    let n = open.len();
    let lookback = 2; // CDLXSIDEGAP3METHODS_Lookback
    let mut out = vec![0.0_f64; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 2], close[i - 2]) == candle_color(open[i - 1], close[i - 1]) // 1st & 2nd same
            && candle_color(open[i - 1], close[i - 1]) == -candle_color(open[i], close[i]) // 3rd opposite
            && open[i] < f64::max(close[i - 1], open[i - 1]) && open[i] > f64::min(close[i - 1], open[i - 1]) // opens in 2nd rb
            && close[i] < f64::max(close[i - 2], open[i - 2]) && close[i] > f64::min(close[i - 2], open[i - 2]) // closes in 1st rb
            && ((candle_color(open[i - 2], close[i - 2]) == 1.0
                && real_body_gap_up(open[i - 1], close[i - 1], open[i - 2], close[i - 2])) // upside gap
                || (candle_color(open[i - 2], close[i - 2]) == -1.0
                    && real_body_gap_down(open[i - 1], close[i - 1], open[i - 2], close[i - 2]))) // downside gap
        {
            out[i] = candle_color(open[i - 2], close[i - 2]) * 100.0;
        } else {
            out[i] = 0.0;
        }
        i += 1;
    }
    Ok(out)
}
