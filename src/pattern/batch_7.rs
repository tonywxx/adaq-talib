//! # 形态识别 · 第 7 批 / Pattern Recognition · Batch 7
//!
//! 本批实现 7 个蜡烛形态，全部与 TA-Lib 0.7.1 黄金向量逐项 1:1：
//! `cdl_piercing`、`cdl_rickshawman`、`cdl_risefall3methods`、
//! `cdl_separatinglines`、`cdl_shortline`、`cdl_spinningtop`、
//! `cdl_stalledpattern`。
//!
//! This batch implements 7 candlestick patterns, all bit-identical to TA-Lib 0.7.1 golden
//! vectors: `cdl_piercing`, `cdl_rickshawman`, `cdl_risefall3methods`, `cdl_separatinglines`,
//! `cdl_shortline`, `cdl_spinningtop`, `cdl_stalledpattern`.

use crate::error::TaError;
use super::*;

// ===========================================================================
// cdl_piercing — Piercing Pattern（刺透形态 / 刺穿线）
// ===========================================================================

/// Piercing Pattern（刺透形态）：第 1 根长阴线，第 2 根长阳线、开盘低于前低、收盘深入前一根实体
/// 至少 50%。恒为看涨 `100`。`BodyLong` 用两个 `CandleAvg`：`OFF=1`（引用 `i−1`）与 `OFF=0`（引用 `i`），
/// `lookback = BodyLong + 1 = 11`。
///
/// Piercing Pattern: 1st long black candle, 2nd long white candle opening below the prior low and
/// closing at least 50% into the prior real body. Always bullish `100`. `BodyLong` uses two window
/// offsets (1 and 0), `lookback = 11`.
pub fn cdl_piercing(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_ohlc(open, high, low, close, "cdl_piercing")?;
    let n = open.len();
    let lookback = BODY_LONG.avg_period + 1; // 11
    let mut out = vec![0.0_f64; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut avg_body_long1 = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 1);
    let mut avg_body_long0 = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 1], close[i - 1]) == -1.0 // 1st: black
            && real_body(open[i - 1], close[i - 1]) > avg_body_long1.value(i, open, high, low, close) //      long
            && candle_color(open[i], close[i]) == 1.0 // 2nd: white
            && real_body(open[i], close[i]) > avg_body_long0.value(i, open, high, low, close) //      long
            && open[i] < low[i - 1] //      open below prior low
            && close[i] < open[i - 1] //      close within prior body
            && close[i] > close[i - 1] + real_body(open[i - 1], close[i - 1]) * 0.5 //        above midpoint
        {
            out[i] = 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body_long1.advance(i, open, high, low, close);
        avg_body_long0.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(out)
}

// ===========================================================================
// cdl_rickshawman — Rickshaw Man（轿夫形态 / 长腿十字）
// ===========================================================================

/// Rickshaw Man（轿夫形态）：十字星实体 + 两根长影线 + 实体接近高低幅中点。恒为 `100`（显示犹豫）。
/// `lookback = max(max(BodyDoji, ShadowLong), Near) = 10`，三者 `off=0`。
///
/// Rickshaw Man: doji body with two long shadows and the body near the midpoint of the high-low
/// range. Always `100`. `lookback = 10`.
pub fn cdl_rickshawman(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_ohlc(open, high, low, close, "cdl_rickshawman")?;
    let n = open.len();
    let lookback = BODY_DOJI.avg_period.max(SHADOW_LONG.avg_period).max(NEAR.avg_period); // 10
    let mut out = vec![0.0_f64; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut avg_body_doji = CandleAvg::new(BODY_DOJI, open, high, low, close, lookback, 0);
    let mut avg_shadow_long = CandleAvg::new(SHADOW_LONG, open, high, low, close, lookback, 0);
    let mut avg_near = CandleAvg::new(NEAR, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        let rb = real_body(open[i], close[i]);
        let midpoint = low[i] + high_low_range(high[i], low[i]) / 2.0;
        if rb <= avg_body_doji.value(i, open, high, low, close) // doji
            && lower_shadow(open[i], low[i], close[i]) > avg_shadow_long.value(i, open, high, low, close) // long shadow
            && upper_shadow(open[i], high[i], close[i]) > avg_shadow_long.value(i, open, high, low, close) // long shadow
            && open[i].min(close[i]) <= midpoint + avg_near.value(i, open, high, low, close) // body near midpoint
            && open[i].max(close[i]) >= midpoint - avg_near.value(i, open, high, low, close)
        {
            out[i] = 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body_doji.advance(i, open, high, low, close);
        avg_shadow_long.advance(i, open, high, low, close);
        avg_near.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(out)
}

// ===========================================================================
// cdl_risefall3methods — Rising/Falling Three Methods（上升/下降三法）
// ===========================================================================

/// Rising/Falling Three Methods（上升/下降三法）：5-candle 持续形态。第 1 根长阳（阴）线，接着 3 根
/// 反向小实体蜡烛（被第 1 根实体包裹且依次回落/上升），第 5 根长阳（阴）线高开并收在第 1 根收盘之上。
/// 阳线输出 `+100`，阴线输出 `−100`。`lookback = max(BodyShort, BodyLong) + 4 = 14`。
/// `BodyLong` 用 `OFF=4 / 0`，三个 `BodyShort` 分别用 `OFF=3 / 2 / 1`。
///
/// Rising/Falling Three Methods: 5-candle continuation. 1st long white/black, three opposite-direction
/// small bodies held within the 1st, 5th long white/black closing above the 1st close. White → `+100`,
/// black → `−100`. `lookback = 14`.
pub fn cdl_risefall3methods(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_ohlc(open, high, low, close, "cdl_risefall3methods")?;
    let n = open.len();
    let lookback = BODY_SHORT.avg_period.max(BODY_LONG.avg_period) + 4; // 14
    let mut out = vec![0.0_f64; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut avg_body_long4 = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 4);
    let mut avg_body_short3 = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 3);
    let mut avg_body_short2 = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 2);
    let mut avg_body_short1 = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 1);
    let mut avg_body_long0 = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        let c4 = candle_color(open[i - 4], close[i - 4]);
        if real_body(open[i - 4], close[i - 4]) > avg_body_long4.value(i, open, high, low, close)
            && real_body(open[i - 3], close[i - 3]) < avg_body_short3.value(i, open, high, low, close)
            && real_body(open[i - 2], close[i - 2]) < avg_body_short2.value(i, open, high, low, close)
            && real_body(open[i - 1], close[i - 1]) < avg_body_short1.value(i, open, high, low, close)
            && real_body(open[i], close[i]) > avg_body_long0.value(i, open, high, low, close)
            && candle_color(open[i - 4], close[i - 4]) == -candle_color(open[i - 3], close[i - 3])
            && candle_color(open[i - 3], close[i - 3]) == candle_color(open[i - 2], close[i - 2])
            && candle_color(open[i - 2], close[i - 2]) == candle_color(open[i - 1], close[i - 1])
            && candle_color(open[i - 1], close[i - 1]) == -candle_color(open[i], close[i])
            && open[i - 3].min(close[i - 3]) < high[i - 4] && open[i - 3].max(close[i - 3]) > low[i - 4]
            && open[i - 2].min(close[i - 2]) < high[i - 4] && open[i - 2].max(close[i - 2]) > low[i - 4]
            && open[i - 1].min(close[i - 1]) < high[i - 4] && open[i - 1].max(close[i - 1]) > low[i - 4]
            && close[i - 2] * c4 < close[i - 3] * c4
            && close[i - 1] * c4 < close[i - 2] * c4
            && open[i] * c4 > close[i - 1] * c4
            && close[i] * c4 > close[i - 4] * c4
        {
            out[i] = 100.0 * c4;
        } else {
            out[i] = 0.0;
        }
        avg_body_long4.advance(i, open, high, low, close);
        avg_body_short3.advance(i, open, high, low, close);
        avg_body_short2.advance(i, open, high, low, close);
        avg_body_short1.advance(i, open, high, low, close);
        avg_body_long0.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(out)
}

// ===========================================================================
// cdl_separatinglines — Separating Lines（分离线）
// ===========================================================================

/// Separating Lines（分离线）：第 1 根阴（阳）线，第 2 根同色 belt-hold（相同开盘价、长实体、无对应
/// 影线）。阳线输出 `+100`，阴线输出 `−100`。`lookback = max(max(ShadowVeryShort, BodyLong), Equal) + 1 = 11`。
/// `ShadowVeryShort`/`BodyLong` 用 `OFF=0`，`Equal` 用 `OFF=1`（引用 `i−1`）。
///
/// Separating Lines: 1st black/white candle, 2nd same-color belt-hold with the same open, long body and
/// no corresponding shadow. White → `+100`, black → `−100`. `lookback = 11`. `Equal` uses `OFF=1`.
pub fn cdl_separatinglines(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_ohlc(open, high, low, close, "cdl_separatinglines")?;
    let n = open.len();
    let lookback = SHADOW_VERY_SHORT.avg_period.max(BODY_LONG.avg_period).max(EQUAL.avg_period) + 1; // 11
    let mut out = vec![0.0_f64; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut avg_shadow_vs = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 0);
    let mut avg_body_long = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 0);
    let mut avg_eq = CandleAvg::new(EQUAL, open, high, low, close, lookback, 1);
    let mut i = lookback;
    while i < n {
        let ci = candle_color(open[i], close[i]);
        let same_open = open[i] <= open[i - 1] + avg_eq.value(i, open, high, low, close)
            && open[i] >= open[i - 1] - avg_eq.value(i, open, high, low, close);
        let long_body = real_body(open[i], close[i]) > avg_body_long.value(i, open, high, low, close);
        let shadow_ok = if ci == 1.0 {
            // bullish: no lower shadow
            lower_shadow(open[i], low[i], close[i]) < avg_shadow_vs.value(i, open, high, low, close)
        } else {
            // bearish: no upper shadow
            upper_shadow(open[i], high[i], close[i]) < avg_shadow_vs.value(i, open, high, low, close)
        };
        if candle_color(open[i - 1], close[i - 1]) == -ci && same_open && long_body && shadow_ok {
            out[i] = ci * 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_shadow_vs.advance(i, open, high, low, close);
        avg_body_long.advance(i, open, high, low, close);
        avg_eq.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(out)
}

// ===========================================================================
// cdl_shortline — Short Line Candle（短实体蜡烛）
// ===========================================================================

/// Short Line Candle（短实体蜡烛）：短实体 + 极短上下影线。阳线输出 `+100`，阴线输出 `−100`。
/// `lookback = max(BodyShort, ShadowShort) = 10`，两者 `off=0`。
///
/// Short Line: short real body with very short upper & lower shadows. White → `+100`, black → `−100`.
/// `lookback = 10`.
pub fn cdl_shortline(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_ohlc(open, high, low, close, "cdl_shortline")?;
    let n = open.len();
    let lookback = BODY_SHORT.avg_period.max(SHADOW_SHORT.avg_period); // 10
    let mut out = vec![0.0_f64; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut avg_body = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 0);
    let mut avg_shadow = CandleAvg::new(SHADOW_SHORT, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        if real_body(open[i], close[i]) < avg_body.value(i, open, high, low, close)
            && upper_shadow(open[i], high[i], close[i]) < avg_shadow.value(i, open, high, low, close)
            && lower_shadow(open[i], low[i], close[i]) < avg_shadow.value(i, open, high, low, close)
        {
            out[i] = candle_color(open[i], close[i]) * 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body.advance(i, open, high, low, close);
        avg_shadow.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(out)
}

// ===========================================================================
// cdl_spinningtop — Spinning Top（纺锤线）
// ===========================================================================

/// Spinning Top（纺锤线）：小实体 + 上下影线均长于实体。阳线输出 `+100`，阴线输出 `−100`。
/// `lookback = BodyShort = 10`，`off=0`。
///
/// Spinning Top: small real body with both shadows longer than the real body. White → `+100`,
/// black → `−100`. `lookback = 10`.
pub fn cdl_spinningtop(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_ohlc(open, high, low, close, "cdl_spinningtop")?;
    let n = open.len();
    let lookback = BODY_SHORT.avg_period; // 10
    let mut out = vec![0.0_f64; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut avg_body = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        let rb = real_body(open[i], close[i]);
        if rb < avg_body.value(i, open, high, low, close)
            && upper_shadow(open[i], high[i], close[i]) > rb
            && lower_shadow(open[i], low[i], close[i]) > rb
        {
            out[i] = candle_color(open[i], close[i]) * 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(out)
}

// ===========================================================================
// cdl_stalledpattern — Stalled Pattern（停顿形态 / 残缺三兵）
// ===========================================================================

/// Stalled Pattern（停顿形态）：3 根连续创新高的阳线。第 1、2 根长阳（第 2 根极短上影线、开盘在第 1
/// 根实体内/附近），第 3 根小阳线"骑在"第 2 根实体的肩部。恒为看跌 `−100`。
/// `lookback = max(max(BodyLong, BodyShort), max(ShadowVeryShort, Near)) + 2 = 12`。
/// `BodyLong` 用 `OFF=2 / 1`、`ShadowVeryShort` 用 `OFF=1`、`BodyShort` 用 `OFF=0`、
/// `Near` 用 `OFF=2 / 1`。
///
/// Stalled Pattern: three white candles with consecutively higher closes. 1st & 2nd long white (2nd with
/// very short upper shadow, opening within/near the 1st body), 3rd small white riding on the 2nd's shoulder.
/// Always bearish `−100`. `lookback = 12`.
pub fn cdl_stalledpattern(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    check_ohlc(open, high, low, close, "cdl_stalledpattern")?;
    let n = open.len();
    let lookback = BODY_LONG.avg_period
        .max(BODY_SHORT.avg_period)
        .max(SHADOW_VERY_SHORT.avg_period)
        .max(NEAR.avg_period)
        + 2; // 12
    let mut out = vec![0.0_f64; n];
    if n <= lookback {
        return Ok(out);
    }
    let mut avg_body_long2 = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 2);
    let mut avg_body_long1 = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 1);
    let mut avg_body_short = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 0);
    let mut avg_shadow_vs = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 1);
    let mut avg_near2 = CandleAvg::new(NEAR, open, high, low, close, lookback, 2);
    let mut avg_near1 = CandleAvg::new(NEAR, open, high, low, close, lookback, 1);
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 2], close[i - 2]) == 1.0 // 1st white
            && candle_color(open[i - 1], close[i - 1]) == 1.0 // 2nd white
            && candle_color(open[i], close[i]) == 1.0 // 3rd white
            && close[i] > close[i - 1] && close[i - 1] > close[i - 2] // consecutive higher closes
            && real_body(open[i - 2], close[i - 2]) > avg_body_long2.value(i, open, high, low, close) // 1st long
            && real_body(open[i - 1], close[i - 1]) > avg_body_long1.value(i, open, high, low, close) // 2nd long
            && upper_shadow(open[i - 1], high[i - 1], close[i - 1]) < avg_shadow_vs.value(i, open, high, low, close) // very short upper shadow
            && open[i - 1] > open[i - 2] // opens within 1st real body
            && open[i - 1] <= close[i - 2] + avg_near2.value(i, open, high, low, close)
            && real_body(open[i], close[i]) < avg_body_short.value(i, open, high, low, close) // 3rd small
            && open[i] >= close[i - 1] - real_body(open[i], close[i]) - avg_near1.value(i, open, high, low, close) // rides shoulder
        {
            out[i] = -100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body_long2.advance(i, open, high, low, close);
        avg_body_long1.advance(i, open, high, low, close);
        avg_body_short.advance(i, open, high, low, close);
        avg_shadow_vs.advance(i, open, high, low, close);
        avg_near2.advance(i, open, high, low, close);
        avg_near1.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(out)
}
