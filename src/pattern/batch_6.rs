//! # 形态识别 · 第 6 批 / Pattern Recognition · Batch 6
//!
//! 本批实现 7 个蜡烛形态，全部与 TA-Lib 0.7.1 黄金向量逐项 1:1：
//! `cdl_longleggeddoji`、`cdl_longline`、`cdl_matchinglow`、`cdl_mathold`、
//! `cdl_morningdojistar`、`cdl_morningstar`、`cdl_onneck`。
//!
//! This batch implements 7 candlestick patterns, all bit-identical to TA-Lib 0.7.1 golden
//! vectors: `cdl_longleggeddoji`, `cdl_longline`, `cdl_matchinglow`, `cdl_mathold`,
//! `cdl_morningdojistar`, `cdl_morningstar`, `cdl_onneck`.

use crate::error::TaError;
use super::*;

// ===========================================================================
// cdl_longleggeddoji — Long-Legged Doji（长脚十字星）
// ===========================================================================

/// Long-Legged Doji（长脚十字星）：极小的实体 + 至少一根极长影线。
///
/// 恒为 `100`（显示犹豫，本身不判多空）。`ShadowLong` 为 `avgPeriod=0`，直接用当前 K 线实体
/// 比较。`lookback = max(BodyDoji, ShadowLong) = 10`，两者 `off=0`。
///
/// Long-Legged Doji: tiny real body and at least one very long shadow. Always `100`.
/// `ShadowLong` has `avgPeriod=0` (compared against current bar's real body). `lookback = 10`.
pub fn cdl_longleggeddoji(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_longleggeddoji_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_longleggeddoji` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_longleggeddoji`]。
/// Zero-copy variant of [`cdl_longleggeddoji`]: writes results into `out` (length must equal input length).
pub fn cdl_longleggeddoji_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_longleggeddoji_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_longleggeddoji")?;
    let n = open.len();
    let lookback = BODY_DOJI.avg_period.max(SHADOW_LONG.avg_period); // 10
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_body = {
        let mut s = (lookback - 0 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body = (lookback - 0 - 10);
    let mut sum_avg_shadow = {
        let mut s = (lookback - 0 - 0);
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_shadow = (lookback - 0 - 0);
    let mut i = lookback;
    while i < n {
        let cur_avg_body = high_low_range(high[i], low[i]);
        let val_avg_body = sum_avg_body / 10 as f64 * 0.1;
        let cur_avg_shadow = real_body(open[i], close[i]);
        let val_avg_shadow = cur_avg_shadow * 1.0;
        out[i] = if real_body(open[i], close[i]) <= val_avg_body
            && (lower_shadow(open[i], low[i], close[i]) > val_avg_shadow
                || upper_shadow(open[i], high[i], close[i]) > val_avg_shadow)
        { 100.0 } else { 0.0 };
        sum_avg_body += cur_avg_body - high_low_range(high[trail_avg_body], low[trail_avg_body]);
        trail_avg_body += 1;
        sum_avg_shadow += cur_avg_shadow - real_body(open[trail_avg_shadow], close[trail_avg_shadow]);
        trail_avg_shadow += 1;
        i += 1;
    }

    Ok(())
}


// ===========================================================================
// cdl_longline — Long Line Candle（长实体蜡烛）
// ===========================================================================

/// Long Line Candle（长实体蜡烛）：长实体 + 极短上下影线。
///
/// 阳线输出 `+100`，阴线输出 `−100`。`lookback = max(BodyLong, ShadowShort) = 10`，两者 `off=0`。
///
/// Long Line: long real body with very short upper & lower shadows. White → `+100`, black → `−100`.
/// `lookback = 10`.
pub fn cdl_longline(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_longline_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_longline` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_longline`]。
/// Zero-copy variant of [`cdl_longline`]: writes results into `out` (length must equal input length).
pub fn cdl_longline_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_longline_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_longline")?;
    let n = open.len();
    let lookback = BODY_LONG.avg_period.max(SHADOW_SHORT.avg_period); // 10
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
    let mut sum_avg_shadow = {
        let mut s = (lookback - 0 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += (upper_shadow(open[s], high[s], close[s]) + lower_shadow(open[s], low[s], close[s]));
            s += 1;
        }
        acc
    };
    let mut trail_avg_shadow = (lookback - 0 - 10);
    let mut i = lookback;
    while i < n {
        let cur_avg_body = real_body(open[i], close[i]);
        let val_avg_body = sum_avg_body / 10 as f64 * 1.0;
        let cur_avg_shadow = (upper_shadow(open[i], high[i], close[i]) + lower_shadow(open[i], low[i], close[i]));
        let val_avg_shadow = sum_avg_shadow / 10 as f64 * 1.0 / 2.0;
        out[i] = if real_body(open[i], close[i]) > val_avg_body
            && upper_shadow(open[i], high[i], close[i]) < val_avg_shadow
            && lower_shadow(open[i], low[i], close[i]) < val_avg_shadow
        { candle_color(open[i], close[i]) * 100.0 } else { 0.0 };
        sum_avg_body += cur_avg_body - real_body(open[trail_avg_body], close[trail_avg_body]);
        trail_avg_body += 1;
        sum_avg_shadow += cur_avg_shadow - (upper_shadow(open[trail_avg_shadow], high[trail_avg_shadow], close[trail_avg_shadow]) + lower_shadow(open[trail_avg_shadow], low[trail_avg_shadow], close[trail_avg_shadow]));
        trail_avg_shadow += 1;
        i += 1;
    }

    Ok(())
}


// ===========================================================================
// cdl_matchinglow — Matching Low（匹配低价 / 相同低价）
// ===========================================================================

/// Matching Low（匹配低价）：两根连续阴线，且第二根收盘 ≈ 第一根收盘。
///
/// 恒为看涨 `100`。`EQUAL` 使用 `OFF=1`（引用 `i−1`），`lookback = EQUAL + 1 = 6`。
///
/// Matching Low: two consecutive black candles whose closes are (approximately) equal. Always
/// bullish `100`. `EQUAL` uses `OFF=1` (references `i−1`), `lookback = 6`.
pub fn cdl_matchinglow(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_matchinglow_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_matchinglow` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_matchinglow`]。
/// Zero-copy variant of [`cdl_matchinglow`]: writes results into `out` (length must equal input length).
pub fn cdl_matchinglow_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_matchinglow_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_matchinglow")?;
    let n = open.len();
    let lookback = EQUAL.avg_period + 1; // 6
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_eq = {
        let mut s = (lookback - 1 - 5);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_eq = (lookback - 1 - 5);
    let mut i = lookback;
    while i < n {
        let cur_avg_eq = high_low_range(high[(i - 1)], low[(i - 1)]);
        let val_avg_eq = sum_avg_eq / 5 as f64 * 0.05;
        out[i] = if candle_color(open[i - 1], close[i - 1]) == -1.0 // 1st black
            && candle_color(open[i], close[i]) == -1.0 // 2nd black
            && close[i] <= close[i - 1] + val_avg_eq
            && close[i] >= close[i - 1] - val_avg_eq
        { 100.0 } else { 0.0 };
        sum_avg_eq += cur_avg_eq - high_low_range(high[trail_avg_eq], low[trail_avg_eq]);
        trail_avg_eq += 1;
        i += 1;
    }

    Ok(())
}


// ===========================================================================
// cdl_mathold — Mat Hold（铺垫形态 / 梯形持有）
// ===========================================================================

/// Mat Hold（铺垫形态）：5-candle 看涨持续形态。第 1 根长阳线，向上跳空的小阴线（第 2 根），
/// 第 3、4 根回落的小实体蜡烛（被第 1 根实体包裹且高于第一根回调幅度），第 5 根阳线高开并收在
/// 回调日最高价之上。`optInPenetration` 默认 `0.5`（见 `ta_CDLMATHOLD.c`）。
///
/// 恒为看涨 `100`。`lookback = max(BodyShort, BodyLong) + 4 = 14`。`BodyLong` 用 `OFF=4`，
/// 三个 `BodyShort` 分别用 `OFF=3 / 2 / 1`（对应第 2/3/4/5 根实体）。
///
/// Mat Hold: 5-candle bullish continuation. 1st long white, gap-up small black (2nd), two falling
/// small bodies (3rd/4th) held within the 1st, 5th white closing above the reaction-day highs.
/// `optInPenetration` defaults to `0.5`. Always bullish `100`. `lookback = 14`.
pub fn cdl_mathold(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_mathold_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_mathold` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_mathold`]。
/// Zero-copy variant of [`cdl_mathold`]: writes results into `out` (length must equal input length).
pub fn cdl_mathold_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_mathold_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_mathold")?;
    let n = open.len();
    let lookback = BODY_SHORT.avg_period.max(BODY_LONG.avg_period) + 4; // 14
    if n <= lookback {
        return Ok(());
    }
    let penetration = 0.5; // TA_REAL_DEFAULT for CDLMATHOLD
    let mut sum_avg_body_long = {
        let mut s = (lookback - 4 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 4) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_long = (lookback - 4 - 10);
    let mut sum_avg_body_short3 = {
        let mut s = (lookback - 3 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 3) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_short3 = (lookback - 3 - 10);
    let mut sum_avg_body_short2 = {
        let mut s = (lookback - 2 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 2) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_short2 = (lookback - 2 - 10);
    let mut sum_avg_body_short1 = {
        let mut s = (lookback - 1 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_short1 = (lookback - 1 - 10);
    let mut i = lookback;
    while i < n {
        let cur_avg_body_long = real_body(open[(i - 4)], close[(i - 4)]);
        let val_avg_body_long = sum_avg_body_long / 10 as f64 * 1.0;
        let cur_avg_body_short3 = real_body(open[(i - 3)], close[(i - 3)]);
        let val_avg_body_short3 = sum_avg_body_short3 / 10 as f64 * 1.0;
        let cur_avg_body_short2 = real_body(open[(i - 2)], close[(i - 2)]);
        let val_avg_body_short2 = sum_avg_body_short2 / 10 as f64 * 1.0;
        let cur_avg_body_short1 = real_body(open[(i - 1)], close[(i - 1)]);
        let val_avg_body_short1 = sum_avg_body_short1 / 10 as f64 * 1.0;
        out[i] = if real_body(open[i - 4], close[i - 4]) > val_avg_body_long
            && real_body(open[i - 3], close[i - 3]) < val_avg_body_short3
            && real_body(open[i - 2], close[i - 2]) < val_avg_body_short2
            && real_body(open[i - 1], close[i - 1]) < val_avg_body_short1
            && candle_color(open[i - 4], close[i - 4]) == 1.0
            && candle_color(open[i - 3], close[i - 3]) == -1.0
            && candle_color(open[i], close[i]) == 1.0
            && real_body_gap_up(open[i - 3], close[i - 3], open[i - 4], close[i - 4])
            && body_low(open[i - 2], close[i - 2]) < close[i - 4]
            && body_low(open[i - 1], close[i - 1]) < close[i - 4]
            && body_low(open[i - 2], close[i - 2]) > close[i - 4] - real_body(open[i - 4], close[i - 4]) * penetration
            && body_low(open[i - 1], close[i - 1]) > close[i - 4] - real_body(open[i - 4], close[i - 4]) * penetration
            && body_high(open[i - 2], close[i - 2]) < open[i - 3]
            && body_high(open[i - 1], close[i - 1]) < body_high(open[i - 2], close[i - 2])
            && open[i] > close[i - 1]
            && close[i] > high[i - 3].max(high[i - 2]).max(high[i - 1])
        { 100.0 } else { 0.0 };
        sum_avg_body_long += cur_avg_body_long - real_body(open[trail_avg_body_long], close[trail_avg_body_long]);
        trail_avg_body_long += 1;
        sum_avg_body_short3 += cur_avg_body_short3 - real_body(open[trail_avg_body_short3], close[trail_avg_body_short3]);
        trail_avg_body_short3 += 1;
        sum_avg_body_short2 += cur_avg_body_short2 - real_body(open[trail_avg_body_short2], close[trail_avg_body_short2]);
        trail_avg_body_short2 += 1;
        sum_avg_body_short1 += cur_avg_body_short1 - real_body(open[trail_avg_body_short1], close[trail_avg_body_short1]);
        trail_avg_body_short1 += 1;
        i += 1;
    }

    Ok(())
}


// ===========================================================================
// cdl_morningstar — Morning Star（晨星）
// ===========================================================================

/// Morning Star（晨星）：3-candle 底部反转。第 1 根长阴线，第 2 根向下跳空的小实体星线，
/// 第 3 根阳线深入第 1 根实体。`optInPenetration` 默认 `0.3`（见 `ta_CDLMORNINGSTAR.c`）。
///
/// 恒为看涨 `100`。`lookback = max(BodyShort, BodyLong) + 2 = 12`。`BodyLong` 用 `OFF=2`、
/// 第 2 根 `BodyShort` 用 `OFF=1`、第 3 根 `BodyShort` 用 `OFF=0`。
///
/// Morning Star: 3-candle bottom reversal. 1st long black, 2nd short star gapping down, 3rd white
/// closing well within the 1st real body. `optInPenetration` defaults to `0.3`. Always bullish `100`.
/// `lookback = 12`.
pub fn cdl_morningstar(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_morningstar_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_morningstar` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_morningstar`]。
/// Zero-copy variant of [`cdl_morningstar`]: writes results into `out` (length must equal input length).
pub fn cdl_morningstar_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_morningstar_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_morningstar")?;
    let n = open.len();
    let lookback = BODY_SHORT.avg_period.max(BODY_LONG.avg_period) + 2; // 12
    if n <= lookback {
        return Ok(());
    }
    let penetration = 0.3; // TA_REAL_DEFAULT for CDLMORNINGSTAR
    let mut sum_avg_body_long = {
        let mut s = (lookback - 2 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 2) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_long = (lookback - 2 - 10);
    let mut sum_avg_body_short = {
        let mut s = (lookback - 1 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_short = (lookback - 1 - 10);
    let mut sum_avg_body_short2 = {
        let mut s = (lookback - 0 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_short2 = (lookback - 0 - 10);
    let mut i = lookback;
    while i < n {
        let cur_avg_body_long = real_body(open[(i - 2)], close[(i - 2)]);
        let val_avg_body_long = sum_avg_body_long / 10 as f64 * 1.0;
        let cur_avg_body_short = real_body(open[(i - 1)], close[(i - 1)]);
        let val_avg_body_short = sum_avg_body_short / 10 as f64 * 1.0;
        let cur_avg_body_short2 = real_body(open[i], close[i]);
        let val_avg_body_short2 = sum_avg_body_short2 / 10 as f64 * 1.0;
        out[i] = if real_body(open[i - 2], close[i - 2]) > val_avg_body_long
            && candle_color(open[i - 2], close[i - 2]) == -1.0
            && real_body(open[i - 1], close[i - 1]) <= val_avg_body_short
            && real_body_gap_down(open[i - 1], close[i - 1], open[i - 2], close[i - 2])
            && real_body(open[i], close[i]) > val_avg_body_short2
            && candle_color(open[i], close[i]) == 1.0
            && close[i] > close[i - 2] + real_body(open[i - 2], close[i - 2]) * penetration
        { 100.0 } else { 0.0 };
        sum_avg_body_long += cur_avg_body_long - real_body(open[trail_avg_body_long], close[trail_avg_body_long]);
        trail_avg_body_long += 1;
        sum_avg_body_short += cur_avg_body_short - real_body(open[trail_avg_body_short], close[trail_avg_body_short]);
        trail_avg_body_short += 1;
        sum_avg_body_short2 += cur_avg_body_short2 - real_body(open[trail_avg_body_short2], close[trail_avg_body_short2]);
        trail_avg_body_short2 += 1;
        i += 1;
    }

    Ok(())
}


// ===========================================================================
// cdl_morningdojistar — Morning Doji Star（晨星十字）
// ===========================================================================

/// Morning Doji Star（晨星十字）：3-candle 底部反转，第 2 根为十字星（而非普通小实体）。
/// `optInPenetration` 默认 `0.3`（见 `ta_CDLMORNINGDOJISTAR.c`）。
///
/// 恒为看涨 `100`。`lookback = max(max(BodyDoji, BodyLong), BodyShort) + 2 = 12`。
/// `BodyLong` 用 `OFF=2`、`BodyDoji` 用 `OFF=1`、`BodyShort` 用 `OFF=0`。
///
/// Morning Doji Star: like Morning Star but the 2nd candle is a doji. `optInPenetration` defaults
/// to `0.3`. Always bullish `100`. `lookback = 12`.
pub fn cdl_morningdojistar(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_morningdojistar_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_morningdojistar` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_morningdojistar`]。
/// Zero-copy variant of [`cdl_morningdojistar`]: writes results into `out` (length must equal input length).
pub fn cdl_morningdojistar_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_morningdojistar_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_morningdojistar")?;
    let n = open.len();
    let lookback = BODY_DOJI.avg_period.max(BODY_LONG.avg_period).max(BODY_SHORT.avg_period) + 2; // 12
    if n <= lookback {
        return Ok(());
    }
    let penetration = 0.3; // TA_REAL_DEFAULT for CDLMORNINGDOJISTAR
    let mut sum_avg_body_long = {
        let mut s = (lookback - 2 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 2) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_long = (lookback - 2 - 10);
    let mut sum_avg_body_doji = {
        let mut s = (lookback - 1 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_doji = (lookback - 1 - 10);
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
        let cur_avg_body_long = real_body(open[(i - 2)], close[(i - 2)]);
        let val_avg_body_long = sum_avg_body_long / 10 as f64 * 1.0;
        let cur_avg_body_doji = high_low_range(high[(i - 1)], low[(i - 1)]);
        let val_avg_body_doji = sum_avg_body_doji / 10 as f64 * 0.1;
        let cur_avg_body_short = real_body(open[i], close[i]);
        let val_avg_body_short = sum_avg_body_short / 10 as f64 * 1.0;
        out[i] = if real_body(open[i - 2], close[i - 2]) > val_avg_body_long
            && candle_color(open[i - 2], close[i - 2]) == -1.0
            && real_body(open[i - 1], close[i - 1]) <= val_avg_body_doji
            && real_body_gap_down(open[i - 1], close[i - 1], open[i - 2], close[i - 2])
            && real_body(open[i], close[i]) > val_avg_body_short
            && candle_color(open[i], close[i]) == 1.0
            && close[i] > close[i - 2] + real_body(open[i - 2], close[i - 2]) * penetration
        { 100.0 } else { 0.0 };
        sum_avg_body_long += cur_avg_body_long - real_body(open[trail_avg_body_long], close[trail_avg_body_long]);
        trail_avg_body_long += 1;
        sum_avg_body_doji += cur_avg_body_doji - high_low_range(high[trail_avg_body_doji], low[trail_avg_body_doji]);
        trail_avg_body_doji += 1;
        sum_avg_body_short += cur_avg_body_short - real_body(open[trail_avg_body_short], close[trail_avg_body_short]);
        trail_avg_body_short += 1;
        i += 1;
    }

    Ok(())
}


// ===========================================================================
// cdl_onneck — On-Neck Pattern（颈上线）
// ===========================================================================

/// On-Neck Pattern（颈上线）：第 1 根长阴线，第 2 根阳线开盘低于前低、收盘≈前低。
///
/// 恒为看跌 `−100`。`EQUAL` 与 `BodyLong` 均使用 `OFF=1`（引用 `i−1`），
/// `lookback = max(Equal, BodyLong) + 1 = 11`。
///
/// On-Neck: 1st long black candle, 2nd white candle opening below the prior low and closing equal
/// to the prior low. Always bearish `−100`. `EQUAL` and `BodyLong` use `OFF=1`, `lookback = 11`.
pub fn cdl_onneck(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_onneck_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_onneck` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_onneck`]。
/// Zero-copy variant of [`cdl_onneck`]: writes results into `out` (length must equal input length).
pub fn cdl_onneck_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_onneck_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_onneck")?;
    let n = open.len();
    let lookback = EQUAL.avg_period.max(BODY_LONG.avg_period) + 1; // 11
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_eq = {
        let mut s = (lookback - 1 - 5);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_eq = (lookback - 1 - 5);
    let mut sum_avg_body = {
        let mut s = (lookback - 1 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body = (lookback - 1 - 10);
    let mut i = lookback;
    while i < n {
        let cur_avg_eq = high_low_range(high[(i - 1)], low[(i - 1)]);
        let val_avg_eq = sum_avg_eq / 5 as f64 * 0.05;
        let cur_avg_body = real_body(open[(i - 1)], close[(i - 1)]);
        let val_avg_body = sum_avg_body / 10 as f64 * 1.0;
        out[i] = if candle_color(open[i - 1], close[i - 1]) == -1.0 // 1st black
            && real_body(open[i - 1], close[i - 1]) > val_avg_body // long
            && candle_color(open[i], close[i]) == 1.0 // 2nd white
            && open[i] < low[i - 1] // open below prior low
            && close[i] <= low[i - 1] + val_avg_eq
            && close[i] >= low[i - 1] - val_avg_eq
        { -100.0 } else { 0.0 };
        sum_avg_eq += cur_avg_eq - high_low_range(high[trail_avg_eq], low[trail_avg_eq]);
        trail_avg_eq += 1;
        sum_avg_body += cur_avg_body - real_body(open[trail_avg_body], close[trail_avg_body]);
        trail_avg_body += 1;
        i += 1;
    }

    Ok(())
}

