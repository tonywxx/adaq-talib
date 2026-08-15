//! # 形态识别 · 第 4 批 / Pattern Recognition · Batch 4
//!
//! 本批实现 8 个蜡烛图形态，全部逐位对齐 TA-Lib 0.7.1（含已安装 dylib 的修订行为，见 ADR 0005）：
//! `cdl_eveningdojistar`、`cdl_eveningstar`、`cdl_gapsidesidewhite`、`cdl_gravestonedoji`、
//! `cdl_hangingman`、`cdl_haramicross`、`cdl_hikkake`、`cdl_hikkakemod`。
//!
//! This batch implements 8 candlestick patterns, bit-identical to TA-Lib 0.7.1
//! (including the installed dylib's revised behavior, ADR 0005):
//! `cdl_eveningdojistar`, `cdl_eveningstar`, `cdl_gapsidesidewhite`, `cdl_gravestonedoji`,
//! `cdl_hangingman`, `cdl_haramicross`, `cdl_hikkake`, `cdl_hikkakemod`.

use super::*;
use crate::error::TaError;
use crate::indicator::indicator;

// ===========================================================================
// cdl_eveningdojistar — Evening Doji Star（暮星十字）
// ===========================================================================

indicator! {
    /// Evening Doji Star（黄昏十字星）：长阳线 + 向上跳空十字星 + 深陷第一根实体的阴线。
    ///
    /// 恒为看跌（`−100`）。`optInPenetration` 取 TA-Lib 默认值 `0.3`。
    /// `BODY_LONG` 用 `OFF=2`（引用 `i−2`），`BODY_DOJI` 用 `OFF=1`（引用 `i−1`），
    /// `BODY_SHORT` 用 `OFF=0`（引用 `i`）。`lookback = max(max(BodyDoji,BodyLong),BodyShort)+2 = 12`。
    /// 对应 `ta_CDLEVENINGDOJISTAR.c`。
    ///
    /// Evening Doji Star: long white, doji gapping up, then black candle well inside the 1st body.
    /// Always bearish (`−100`), `optInPenetration = 0.3`. `lookback = 12`.
    fn cdl_eveningdojistar(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_eveningdojistar_with_output init zero;
}

/// `cdl_eveningdojistar` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_eveningdojistar`]。
/// Zero-copy variant of [`cdl_eveningdojistar`]: writes results into `out` (length must equal input length).
pub fn cdl_eveningdojistar_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_eveningdojistar_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_eveningdojistar")?;
    let n = open.len();
    let lookback = [
        BODY_DOJI.avg_period,
        BODY_LONG.avg_period,
        BODY_SHORT.avg_period,
    ]
    .iter()
    .max()
    .copied()
    .unwrap()
        + 2; // 12
    let penetration = 0.3;
    if n <= lookback {
        return Ok(());
    }
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
            && candle_color(open[i - 2], close[i - 2]) == 1.0
            && real_body(open[i - 1], close[i - 1]) <= val_avg_body_doji
            && real_body_gap_up(open[i - 1], close[i - 1], open[i - 2], close[i - 2])
            && cur_avg_body_short > val_avg_body_short
            && candle_color(open[i], close[i]) == -1.0
            && close[i] < close[i - 2] - real_body(open[i - 2], close[i - 2]) * penetration
        {
            -100.0
        } else {
            0.0
        };
        sum_avg_body_long +=
            cur_avg_body_long - real_body(open[trail_avg_body_long], close[trail_avg_body_long]);
        trail_avg_body_long += 1;
        sum_avg_body_doji +=
            cur_avg_body_doji - high_low_range(high[trail_avg_body_doji], low[trail_avg_body_doji]);
        trail_avg_body_doji += 1;
        sum_avg_body_short +=
            cur_avg_body_short - real_body(open[trail_avg_body_short], close[trail_avg_body_short]);
        trail_avg_body_short += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_eveningstar — Evening Star（暮星）
// ===========================================================================

indicator! {
    /// Evening Star（黄昏之星）：长阳线 + 向上跳空小星线 + 深陷第一根实体的阴线。
    ///
    /// 恒为看跌（`−100`）。`optInPenetration` 取 TA-Lib 默认值 `0.3`。
    /// `BODY_LONG` 用 `OFF=2`（引用 `i−2`）；`BODY_SHORT` 用 `OFF=1`（引用 `i−1`，第二根星线）
    /// 与 `OFF=0`（引用 `i`，第三根）。`lookback = max(BodyShort,BodyLong)+2 = 12`。对应 `ta_CDLEVENINGSTAR.c`。
    ///
    /// Evening Star: long white, small star gapping up, then black candle well inside the 1st body.
    /// Always bearish (`−100`), `optInPenetration = 0.3`, `lookback = 12`.
    fn cdl_eveningstar(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_eveningstar_with_output init zero;
}

/// `cdl_eveningstar` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_eveningstar`]。
/// Zero-copy variant of [`cdl_eveningstar`]: writes results into `out` (length must equal input length).
pub fn cdl_eveningstar_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_eveningstar_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_eveningstar")?;
    let n = open.len();
    let lookback = BODY_SHORT.avg_period.max(BODY_LONG.avg_period) + 2; // 12
    let penetration = 0.3;
    if n <= lookback {
        return Ok(());
    }
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
            && candle_color(open[i - 2], close[i - 2]) == 1.0
            && real_body(open[i - 1], close[i - 1]) <= val_avg_body_short
            && real_body_gap_up(open[i - 1], close[i - 1], open[i - 2], close[i - 2])
            && cur_avg_body_short2 > val_avg_body_short2
            && candle_color(open[i], close[i]) == -1.0
            && close[i] < close[i - 2] - real_body(open[i - 2], close[i - 2]) * penetration
        {
            -100.0
        } else {
            0.0
        };
        sum_avg_body_long +=
            cur_avg_body_long - real_body(open[trail_avg_body_long], close[trail_avg_body_long]);
        trail_avg_body_long += 1;
        sum_avg_body_short +=
            cur_avg_body_short - real_body(open[trail_avg_body_short], close[trail_avg_body_short]);
        trail_avg_body_short += 1;
        sum_avg_body_short2 += cur_avg_body_short2
            - real_body(open[trail_avg_body_short2], close[trail_avg_body_short2]);
        trail_avg_body_short2 += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_gapsidesidewhite — Up/Down-gap Side-by-Side White Lines（向上/向下跳空并列阳线）
// ===========================================================================

indicator! {
    /// Up/Down-gap Side-by-Side White Lines（向上/向下跳空并列白线）：第一根跳空后连续两根相似白线。
    ///
    /// 向上跳空并列白线输出 `+100`，向下跳空并列白线输出 `−100`。
    /// `NEAR` 与 `EQUAL` 均用 `OFF=1`（引用 `i−1`）。`lookback = max(Near,Equal)+2 = 7`。对应 `ta_CDLGAPSIDESIDEWHITE.c`。
    ///
    /// Up/Down-gap side-by-side white lines: after a gap, two similar white lines.
    /// Gap-up → `+100`, gap-down → `−100`, `lookback = 7`.
    fn cdl_gapsidesidewhite(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_gapsidesidewhite_with_output init zero;
}

/// `cdl_gapsidesidewhite` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_gapsidesidewhite`]。
/// Zero-copy variant of [`cdl_gapsidesidewhite`]: writes results into `out` (length must equal input length).
pub fn cdl_gapsidesidewhite_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_gapsidesidewhite_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_gapsidesidewhite")?;
    let n = open.len();
    let lookback = NEAR.avg_period.max(EQUAL.avg_period) + 2; // 7
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_near = {
        let mut s = (lookback - 1 - 5);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_near = (lookback - 1 - 5);
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
    let mut i = lookback;
    while i < n {
        let cur_avg_near = high_low_range(high[(i - 1)], low[(i - 1)]);
        let val_avg_near = sum_avg_near / 5 as f64 * 0.2;
        let cur_avg_equal = high_low_range(high[(i - 1)], low[(i - 1)]);
        let val_avg_equal = sum_avg_equal / 5 as f64 * 0.05;

        let gap_up = real_body_gap_up(open[i - 1], close[i - 1], open[i - 2], close[i - 2])
            && real_body_gap_up(open[i], close[i], open[i - 2], close[i - 2]);
        let gap_down = real_body_gap_down(open[i - 1], close[i - 1], open[i - 2], close[i - 2])
            && real_body_gap_down(open[i], close[i], open[i - 2], close[i - 2]);
        out[i] = if (gap_up || gap_down)
            && candle_color(open[i - 1], close[i - 1]) == 1.0
            && candle_color(open[i], close[i]) == 1.0
            && real_body(open[i], close[i]) >= real_body(open[i - 1], close[i - 1]) - val_avg_near
            && real_body(open[i], close[i]) <= real_body(open[i - 1], close[i - 1]) + val_avg_near
            && open[i] >= open[i - 1] - val_avg_equal
            && open[i] <= open[i - 1] + val_avg_equal
        {
            if gap_up { 100.0 } else { -100.0 }
        } else {
            0.0
        };
        sum_avg_near += cur_avg_near - high_low_range(high[trail_avg_near], low[trail_avg_near]);
        trail_avg_near += 1;
        sum_avg_equal +=
            cur_avg_equal - high_low_range(high[trail_avg_equal], low[trail_avg_equal]);
        trail_avg_equal += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_gravestonedoji — Gravestone Doji（墓碑十字）
// ===========================================================================

indicator! {
    /// Gravestone Doji（墓碑十字）：十字星实体，开盘/收盘位于当日最低，且有上影线。
    ///
    /// 恒为 `+100`（本身不判多空，需结合趋势）。`BODY_DOJI` 与 `SHADOW_VERY_SHORT` 均用 `OFF=0`
    /// （引用 `i`）。`lookback = max(BodyDoji, ShadowVeryShort) = 10`。对应 `ta_CDLGRAVESTONEDOJI.c`。
    ///
    /// Gravestone Doji: doji body at the low of the day with an upper shadow. Always `+100`, `lookback = 10`.
    fn cdl_gravestonedoji(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_gravestonedoji_with_output init zero;
}

/// `cdl_gravestonedoji` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_gravestonedoji`]。
/// Zero-copy variant of [`cdl_gravestonedoji`]: writes results into `out` (length must equal input length).
pub fn cdl_gravestonedoji_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_gravestonedoji_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_gravestonedoji")?;
    let n = open.len();
    let lookback = BODY_DOJI.avg_period.max(SHADOW_VERY_SHORT.avg_period); // 10
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_body_doji = {
        let mut s = (lookback - 0 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_doji = (lookback - 0 - 10);
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
        let cur_avg_body_doji = high_low_range(high[i], low[i]);
        let val_avg_body_doji = sum_avg_body_doji / 10 as f64 * 0.1;
        let cur_avg_shadow_vshort = high_low_range(high[i], low[i]);
        let val_avg_shadow_vshort = sum_avg_shadow_vshort / 10 as f64 * 0.1;
        out[i] = if real_body(open[i], close[i]) <= val_avg_body_doji
            && lower_shadow(open[i], low[i], close[i]) < val_avg_shadow_vshort
            && upper_shadow(open[i], high[i], close[i]) > val_avg_shadow_vshort
        {
            100.0
        } else {
            0.0
        };
        sum_avg_body_doji +=
            cur_avg_body_doji - high_low_range(high[trail_avg_body_doji], low[trail_avg_body_doji]);
        trail_avg_body_doji += 1;
        sum_avg_shadow_vshort += cur_avg_shadow_vshort
            - high_low_range(high[trail_avg_shadow_vshort], low[trail_avg_shadow_vshort]);
        trail_avg_shadow_vshort += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_hangingman — Hanging Man（上吊线）
// ===========================================================================

indicator! {
    /// Hanging Man（上吊线）：小实体、长下影线、极短/无上影线，实体位于前一根蜡烛高位附近。
    ///
    /// 恒为看跌（`−100`）。`BODY_SHORT`/`SHADOW_LONG`/`SHADOW_VERY_SHORT` 用 `OFF=0`（引用 `i`），
    /// `NEAR` 用 `OFF=1`（引用 `i−1`）。
    /// `lookback = max(max(max(BodyShort,ShadowLong),ShadowVeryShort),Near)+1 = 11`。对应 `ta_CDLHANGINGMAN.c`。
    ///
    /// Hanging Man: small body, long lower shadow, very short upper shadow, body near prior highs.
    /// Always bearish (`−100`), `lookback = 11`.
    fn cdl_hangingman(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_hangingman_with_output init zero;
}

/// `cdl_hangingman` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_hangingman`]。
/// Zero-copy variant of [`cdl_hangingman`]: writes results into `out` (length must equal input length).
pub fn cdl_hangingman_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_hangingman_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_hangingman")?;
    let n = open.len();
    let lookback = [
        BODY_SHORT.avg_period,
        SHADOW_LONG.avg_period,
        SHADOW_VERY_SHORT.avg_period,
        NEAR.avg_period,
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
    let mut sum_avg_near = {
        let mut s = (lookback - 1 - 5);
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_near = (lookback - 1 - 5);
    let mut i = lookback;
    while i < n {
        let cur_avg_body = real_body(open[i], close[i]);
        let val_avg_body = sum_avg_body / 10 as f64 * 1.0;
        let cur_avg_shadow_long = real_body(open[i], close[i]);
        let val_avg_shadow_long = cur_avg_shadow_long * 1.0;
        let cur_avg_shadow_vshort = high_low_range(high[i], low[i]);
        let val_avg_shadow_vshort = sum_avg_shadow_vshort / 10 as f64 * 0.1;
        let cur_avg_near = high_low_range(high[(i - 1)], low[(i - 1)]);
        let val_avg_near = sum_avg_near / 5 as f64 * 0.2;
        out[i] = if real_body(open[i], close[i]) < val_avg_body
            && lower_shadow(open[i], low[i], close[i]) > val_avg_shadow_long
            && upper_shadow(open[i], high[i], close[i]) < val_avg_shadow_vshort
            && open[i].min(close[i]) >= high[i - 1] - val_avg_near
        {
            -100.0
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
        sum_avg_near += cur_avg_near - high_low_range(high[trail_avg_near], low[trail_avg_near]);
        trail_avg_near += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_haramicross — Harami Cross Pattern（十字孕线）
// ===========================================================================

indicator! {
    /// Harami Cross（十字孕线）：第一根长实体，第二根十字星完全被第一根实体包裹。
    ///
    /// 第一根为阳线则看跌（`−100`），为阴线则看涨（`+100`）。`BODY_LONG` 用 `OFF=1`（引用 `i−1`），
    /// `BODY_DOJI` 用 `OFF=0`（引用 `i`）。`lookback = max(BodyDoji,BodyLong)+1 = 11`。对应 `ta_CDLHARAMICROSS.c`。
    ///
    /// Harami Cross: 1st a long real body, 2nd a doji totally engulfed by the 1st.
    /// `BODY_LONG` uses `OFF=1`, `BODY_DOJI` uses `OFF=0`, `lookback = 11`.
    fn cdl_haramicross(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_haramicross_with_output init zero;
}

/// `cdl_haramicross` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_haramicross`]。
/// Zero-copy variant of [`cdl_haramicross`]: writes results into `out` (length must equal input length).
pub fn cdl_haramicross_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_haramicross_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_haramicross")?;
    let n = open.len();
    let lookback = BODY_DOJI.avg_period.max(BODY_LONG.avg_period) + 1; // 11
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
    let mut sum_avg_body_doji = {
        let mut s = (lookback - 0 - 10);
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_doji = (lookback - 0 - 10);
    let mut i = lookback;
    while i < n {
        let cur_avg_body_long = real_body(open[(i - 1)], close[(i - 1)]);
        let val_avg_body_long = sum_avg_body_long / 10 as f64 * 1.0;
        let cur_avg_body_doji = high_low_range(high[i], low[i]);
        let val_avg_body_doji = sum_avg_body_doji / 10 as f64 * 0.1;
        out[i] = if real_body(open[i - 1], close[i - 1]) > val_avg_body_long
            && real_body(open[i], close[i]) <= val_avg_body_doji
            && open[i].max(close[i]) < open[i - 1].max(close[i - 1])
            && open[i].min(close[i]) > open[i - 1].min(close[i - 1])
        {
            -candle_color(open[i - 1], close[i - 1]) * 100.0
        } else {
            0.0
        };
        sum_avg_body_long +=
            cur_avg_body_long - real_body(open[trail_avg_body_long], close[trail_avg_body_long]);
        trail_avg_body_long += 1;
        sum_avg_body_doji +=
            cur_avg_body_doji - high_low_range(high[trail_avg_body_doji], low[trail_avg_body_doji]);
        trail_avg_body_doji += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_hikkake — Hikkake Pattern（陷阱形态）
// ===========================================================================

indicator! {
    /// Hikkake Pattern（陷阱形态）：第 1、2 根为内包线（第 2 根更低高、更高低），第 3 根突破。
    ///
    /// 第 3 根更低高且更低低 → 看涨 `+100`；更高高且更高低 → 看跌 `−100`。
    /// 之后 3 根内若收线突破第 2 根极值，确认输出 `+200` / `−200`。
    /// 无蜡烛设置，`lookback = 5`。含跨根状态机（确认会重置），对应 `ta_CDLHIKKAKE.c`。
    ///
    /// Hikkake: 1st+2nd inside bar, 3rd breaks out. Bullish `+100`/bearish `−100`; confirmation `+200`/`−200`.
    /// No candle settings, `lookback = 5`, carries cross-bar state (ADR 0005).
    fn cdl_hikkake(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_hikkake_with_output init zero;
}

/// `cdl_hikkake` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_hikkake`]。
/// Zero-copy variant of [`cdl_hikkake`]: writes results into `out` (length must equal input length).
pub fn cdl_hikkake_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_hikkake_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_hikkake")?;
    let n = open.len();
    let lookback = 5;
    if n <= lookback {
        return Ok(());
    }
    let mut pattern_idx: i64 = 0;
    let mut pattern_result: f64 = 0.0;
    // 预热状态机（仅更新 patternIdx/patternResult，不输出，与 C 源一致）。
    // Warm-up state machine (updates state only, no output, matching the C source).
    let mut i = lookback - 3;
    while i < lookback {
        if high[i - 1] < high[i - 2]
            && low[i - 1] > low[i - 2]
            && ((high[i] < high[i - 1] && low[i] < low[i - 1])
                || (high[i] > high[i - 1] && low[i] > low[i - 1]))
        {
            pattern_result = 100.0 * if high[i] < high[i - 1] { 1.0 } else { -1.0 };
            pattern_idx = i as i64;
        } else if (i as i64) <= pattern_idx + 3
            && ((pattern_result > 0.0 && close[i] > high[pattern_idx as usize - 1])
                || (pattern_result < 0.0 && close[i] < low[pattern_idx as usize - 1]))
        {
            pattern_idx = 0;
        }
        i += 1;
    }
    // 主循环 / Main loop.
    i = lookback;
    while i < n {
        let val;
        if high[i - 1] < high[i - 2]
            && low[i - 1] > low[i - 2]
            && ((high[i] < high[i - 1] && low[i] < low[i - 1])
                || (high[i] > high[i - 1] && low[i] > low[i - 1]))
        {
            pattern_result = 100.0 * if high[i] < high[i - 1] { 1.0 } else { -1.0 };
            pattern_idx = i as i64;
            val = pattern_result;
        } else if (i as i64) <= pattern_idx + 3
            && ((pattern_result > 0.0 && close[i] > high[pattern_idx as usize - 1])
                || (pattern_result < 0.0 && close[i] < low[pattern_idx as usize - 1]))
        {
            val = pattern_result + 100.0 * if pattern_result > 0.0 { 1.0 } else { -1.0 };
            pattern_idx = 0;
        } else {
            val = 0.0;
        }
        out[i] = val;
        i += 1;
    }
    Ok(())
}

// ===========================================================================
// cdl_hikkakemod — Modified Hikkake Pattern（改良陷阱形态）
// ===========================================================================

indicator! {
    /// Modified Hikkake Pattern（改良陷阱形态）：在 Hikkake 基础上要求第 2 根收盘贴近极值，
    /// 并增加第 1 根前导线，共 4 根确认（第 2 根收盘在低位为看涨、高位为看跌）。
    ///
    /// 第 4 根更低高且更低低（且第 2 根收盘贴近低位）→ 看涨 `+100`；
    /// 更高高且更高低（且第 2 根收盘贴近高位）→ 看跌 `−100`。之后 3 根内确认输出 `+200` / `−200`。
    /// `NEAR` 用 `OFF=2`（引用 `i−2`），`lookback = max(1,Near)+5 = 10`。对应 `ta_CDLHIKKAKEMOD.c`。
    ///
    /// Modified Hikkake: 4-bar setup, 2nd candle's close near the low (bull) / high (bear).
    /// `NEAR` uses `OFF=2`, `lookback = 10`, carries cross-bar confirmation state.
    fn cdl_hikkakemod(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_hikkakemod_with_output init zero;
}

/// `cdl_hikkakemod` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_hikkakemod`]。
/// Zero-copy variant of [`cdl_hikkakemod`]: writes results into `out` (length must equal input length).
pub fn cdl_hikkakemod_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_hikkakemod_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_hikkakemod")?;
    let n = open.len();
    let lookback = NEAR.avg_period.max(1) + 5; // 10
    if n <= lookback {
        return Ok(());
    }
    // `OFF=2` 但预热起点为 `lookback-3`：C 源有独立的 seed 循环 + 识别预热循环，等价于
    // 从 `i=lookback-3` 起每根推进一次滚动窗口（见 ta_CDLHIKKAKEMOD.c）。
    // `OFF=2` with warm-up seeded at `lookback-3`: mirrors the C seed + recognition warm-up loops.
    let mut avg_near = CandleAvg::new(NEAR, open, high, low, close, lookback - 3, 2);
    let mut pattern_idx: i64 = 0;
    let mut pattern_result: f64 = 0.0;
    let mut i = lookback - 3;
    while i < lookback {
        let near = avg_near.value(i, open, high, low, close);
        if high[i - 2] < high[i - 3]
            && low[i - 2] > low[i - 3]
            && high[i - 1] < high[i - 2]
            && low[i - 1] > low[i - 2]
            && ((high[i] < high[i - 1] && low[i] < low[i - 1] && close[i - 2] <= low[i - 2] + near)
                || (high[i] > high[i - 1]
                    && low[i] > low[i - 1]
                    && close[i - 2] >= high[i - 2] - near))
        {
            pattern_result = 100.0 * if high[i] < high[i - 1] { 1.0 } else { -1.0 };
            pattern_idx = i as i64;
        } else if (i as i64) <= pattern_idx + 3
            && ((pattern_result > 0.0 && close[i] > high[pattern_idx as usize - 1])
                || (pattern_result < 0.0 && close[i] < low[pattern_idx as usize - 1]))
        {
            pattern_idx = 0;
        }
        avg_near.advance(i, open, high, low, close);
        i += 1;
    }
    i = lookback;
    while i < n {
        let near = avg_near.value(i, open, high, low, close);
        let val;
        if high[i - 2] < high[i - 3]
            && low[i - 2] > low[i - 3]
            && high[i - 1] < high[i - 2]
            && low[i - 1] > low[i - 2]
            && ((high[i] < high[i - 1] && low[i] < low[i - 1] && close[i - 2] <= low[i - 2] + near)
                || (high[i] > high[i - 1]
                    && low[i] > low[i - 1]
                    && close[i - 2] >= high[i - 2] - near))
        {
            pattern_result = 100.0 * if high[i] < high[i - 1] { 1.0 } else { -1.0 };
            pattern_idx = i as i64;
            val = pattern_result;
        } else if (i as i64) <= pattern_idx + 3
            && ((pattern_result > 0.0 && close[i] > high[pattern_idx as usize - 1])
                || (pattern_result < 0.0 && close[i] < low[pattern_idx as usize - 1]))
        {
            val = pattern_result + 100.0 * if pattern_result > 0.0 { 1.0 } else { -1.0 };
            pattern_idx = 0;
        } else {
            val = 0.0;
        }
        out[i] = val;
        avg_near.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}
