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

use super::*;
use crate::error::TaError;
use crate::indicator::indicator;

// ===========================================================================
// cdl_piercing — Piercing Pattern（刺透形态 / 刺穿线）
// ===========================================================================

indicator! {
    /// Piercing Pattern（刺透形态）：第 1 根长阴线，第 2 根长阳线、开盘低于前低、收盘深入前一根实体
    /// 至少 50%。恒为看涨 `100`。`BodyLong` 用两个 `CandleAvg`：`OFF=1`（引用 `i−1`）与 `OFF=0`（引用 `i`），
    /// `lookback = BodyLong + 1 = 11`。
    ///
    /// Piercing Pattern: 1st long black candle, 2nd long white candle opening below the prior low and
    /// closing at least 50% into the prior real body. Always bullish `100`. `BodyLong` uses two window
    /// offsets (1 and 0), `lookback = 11`.
    fn cdl_piercing(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_piercing_with_output init zero;
}

/// `cdl_piercing` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_piercing`]。
/// Zero-copy variant of [`cdl_piercing`]: writes results into `out` (length must equal input length).
pub fn cdl_piercing_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_piercing_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_piercing")?;
    let n = open.len();
    let lookback = BODY_LONG.avg_period + 1; // 11
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_body_long1 = {
        let mut s = lookback - 1 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_long1 = lookback - 1 - 10;
    let mut sum_avg_body_long0 = {
        let mut s = lookback - 0 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_long0 = lookback - 0 - 10;
    let mut i = lookback;
    while i < n {
        let cur_avg_body_long1 = real_body(open[i - 1], close[i - 1]);
        let val_avg_body_long1 = sum_avg_body_long1 / 10 as f64 * 1.0;
        let cur_avg_body_long0 = real_body(open[i], close[i]);
        let val_avg_body_long0 = sum_avg_body_long0 / 10 as f64 * 1.0;
        out[i] = if candle_color(open[i - 1], close[i - 1]) == -1.0 // 1st: black
            && real_body(open[i - 1], close[i - 1]) > val_avg_body_long1 //      long
            && candle_color(open[i], close[i]) == 1.0 // 2nd: white
            && cur_avg_body_long0 > val_avg_body_long0 //      long
            && open[i] < low[i - 1] //      open below prior low
            && close[i] < open[i - 1] //      close within prior body
            && close[i] > close[i - 1] + real_body(open[i - 1], close[i - 1]) * 0.5
        //        above midpoint
        {
            100.0
        } else {
            0.0
        };
        sum_avg_body_long1 +=
            cur_avg_body_long1 - real_body(open[trail_avg_body_long1], close[trail_avg_body_long1]);
        trail_avg_body_long1 += 1;
        sum_avg_body_long0 +=
            cur_avg_body_long0 - real_body(open[trail_avg_body_long0], close[trail_avg_body_long0]);
        trail_avg_body_long0 += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_rickshawman — Rickshaw Man（轿夫形态 / 长腿十字）
// ===========================================================================

indicator! {
    /// Rickshaw Man（轿夫形态）：十字星实体 + 两根长影线 + 实体接近高低幅中点。恒为 `100`（显示犹豫）。
    /// `lookback = max(max(BodyDoji, ShadowLong), Near) = 10`，三者 `off=0`。
    ///
    /// Rickshaw Man: doji body with two long shadows and the body near the midpoint of the high-low
    /// range. Always `100`. `lookback = 10`.
    fn cdl_rickshawman(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_rickshawman_with_output init zero;
}

/// `cdl_rickshawman` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_rickshawman`]。
/// Zero-copy variant of [`cdl_rickshawman`]: writes results into `out` (length must equal input length).
pub fn cdl_rickshawman_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_rickshawman_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_rickshawman")?;
    let n = open.len();
    let lookback = BODY_DOJI
        .avg_period
        .max(SHADOW_LONG.avg_period)
        .max(NEAR.avg_period); // 10
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_body_doji = {
        let mut s = lookback - 0 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_doji = lookback - 0 - 10;
    let mut sum_avg_shadow_long = {
        let mut s = lookback - 0 - 0;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_shadow_long = lookback - 0 - 0;
    let mut sum_avg_near = {
        let mut s = lookback - 0 - 5;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_near = lookback - 0 - 5;
    let mut i = lookback;
    while i < n {
        let cur_avg_body_doji = high_low_range(high[i], low[i]);
        let val_avg_body_doji = sum_avg_body_doji / 10 as f64 * 0.1;
        let cur_avg_shadow_long = real_body(open[i], close[i]);
        let val_avg_shadow_long = cur_avg_shadow_long * 1.0;
        let cur_avg_near = high_low_range(high[i], low[i]);
        let val_avg_near = sum_avg_near / 5 as f64 * 0.2;

        let rb = real_body(open[i], close[i]);
        let midpoint = low[i] + high_low_range(high[i], low[i]) / 2.0;
        out[i] = if rb <= val_avg_body_doji // doji
            && lower_shadow(open[i], low[i], close[i]) > val_avg_shadow_long // long shadow
            && upper_shadow(open[i], high[i], close[i]) > val_avg_shadow_long // long shadow
            && open[i].min(close[i]) <= midpoint + val_avg_near // body near midpoint
            && open[i].max(close[i]) >= midpoint - val_avg_near
        {
            100.0
        } else {
            0.0
        };
        sum_avg_body_doji +=
            cur_avg_body_doji - high_low_range(high[trail_avg_body_doji], low[trail_avg_body_doji]);
        trail_avg_body_doji += 1;
        sum_avg_shadow_long += cur_avg_shadow_long
            - real_body(open[trail_avg_shadow_long], close[trail_avg_shadow_long]);
        trail_avg_shadow_long += 1;
        sum_avg_near += cur_avg_near - high_low_range(high[trail_avg_near], low[trail_avg_near]);
        trail_avg_near += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_risefall3methods — Rising/Falling Three Methods（上升/下降三法）
// ===========================================================================

indicator! {
    /// Rising/Falling Three Methods（上升/下降三法）：5-candle 持续形态。第 1 根长阳（阴）线，接着 3 根
    /// 反向小实体蜡烛（被第 1 根实体包裹且依次回落/上升），第 5 根长阳（阴）线高开并收在第 1 根收盘之上。
    /// 阳线输出 `+100`，阴线输出 `−100`。`lookback = max(BodyShort, BodyLong) + 4 = 14`。
    /// `BodyLong` 用 `OFF=4 / 0`，三个 `BodyShort` 分别用 `OFF=3 / 2 / 1`。
    ///
    /// Rising/Falling Three Methods: 5-candle continuation. 1st long white/black, three opposite-direction
    /// small bodies held within the 1st, 5th long white/black closing above the 1st close. White → `+100`,
    /// black → `−100`. `lookback = 14`.
    fn cdl_risefall3methods(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_risefall3methods_with_output init zero;
}

/// `cdl_risefall3methods` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_risefall3methods`]。
/// Zero-copy variant of [`cdl_risefall3methods`]: writes results into `out` (length must equal input length).
pub fn cdl_risefall3methods_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_risefall3methods_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_risefall3methods")?;
    let n = open.len();
    let lookback = BODY_SHORT.avg_period.max(BODY_LONG.avg_period) + 4; // 14
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_body_long4 = {
        let mut s = lookback - 4 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 4) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_long4 = lookback - 4 - 10;
    let mut sum_avg_body_short3 = {
        let mut s = lookback - 3 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 3) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_short3 = lookback - 3 - 10;
    let mut sum_avg_body_short2 = {
        let mut s = lookback - 2 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 2) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_short2 = lookback - 2 - 10;
    let mut sum_avg_body_short1 = {
        let mut s = lookback - 1 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_short1 = lookback - 1 - 10;
    let mut sum_avg_body_long0 = {
        let mut s = lookback - 0 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_long0 = lookback - 0 - 10;
    let mut i = lookback;
    while i < n {
        let cur_avg_body_long4 = real_body(open[i - 4], close[i - 4]);
        let val_avg_body_long4 = sum_avg_body_long4 / 10 as f64 * 1.0;
        let cur_avg_body_short3 = real_body(open[i - 3], close[i - 3]);
        let val_avg_body_short3 = sum_avg_body_short3 / 10 as f64 * 1.0;
        let cur_avg_body_short2 = real_body(open[i - 2], close[i - 2]);
        let val_avg_body_short2 = sum_avg_body_short2 / 10 as f64 * 1.0;
        let cur_avg_body_short1 = real_body(open[i - 1], close[i - 1]);
        let val_avg_body_short1 = sum_avg_body_short1 / 10 as f64 * 1.0;
        let cur_avg_body_long0 = real_body(open[i], close[i]);
        let val_avg_body_long0 = sum_avg_body_long0 / 10 as f64 * 1.0;

        let c4 = candle_color(open[i - 4], close[i - 4]);
        out[i] = if real_body(open[i - 4], close[i - 4]) > val_avg_body_long4
            && real_body(open[i - 3], close[i - 3]) < val_avg_body_short3
            && real_body(open[i - 2], close[i - 2]) < val_avg_body_short2
            && real_body(open[i - 1], close[i - 1]) < val_avg_body_short1
            && cur_avg_body_long0 > val_avg_body_long0
            && candle_color(open[i - 4], close[i - 4]) == -candle_color(open[i - 3], close[i - 3])
            && candle_color(open[i - 3], close[i - 3]) == candle_color(open[i - 2], close[i - 2])
            && candle_color(open[i - 2], close[i - 2]) == candle_color(open[i - 1], close[i - 1])
            && candle_color(open[i - 1], close[i - 1]) == -candle_color(open[i], close[i])
            && open[i - 3].min(close[i - 3]) < high[i - 4]
            && open[i - 3].max(close[i - 3]) > low[i - 4]
            && open[i - 2].min(close[i - 2]) < high[i - 4]
            && open[i - 2].max(close[i - 2]) > low[i - 4]
            && open[i - 1].min(close[i - 1]) < high[i - 4]
            && open[i - 1].max(close[i - 1]) > low[i - 4]
            && close[i - 2] * c4 < close[i - 3] * c4
            && close[i - 1] * c4 < close[i - 2] * c4
            && open[i] * c4 > close[i - 1] * c4
            && close[i] * c4 > close[i - 4] * c4
        {
            100.0 * c4
        } else {
            0.0
        };
        sum_avg_body_long4 +=
            cur_avg_body_long4 - real_body(open[trail_avg_body_long4], close[trail_avg_body_long4]);
        trail_avg_body_long4 += 1;
        sum_avg_body_short3 += cur_avg_body_short3
            - real_body(open[trail_avg_body_short3], close[trail_avg_body_short3]);
        trail_avg_body_short3 += 1;
        sum_avg_body_short2 += cur_avg_body_short2
            - real_body(open[trail_avg_body_short2], close[trail_avg_body_short2]);
        trail_avg_body_short2 += 1;
        sum_avg_body_short1 += cur_avg_body_short1
            - real_body(open[trail_avg_body_short1], close[trail_avg_body_short1]);
        trail_avg_body_short1 += 1;
        sum_avg_body_long0 +=
            cur_avg_body_long0 - real_body(open[trail_avg_body_long0], close[trail_avg_body_long0]);
        trail_avg_body_long0 += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_separatinglines — Separating Lines（分离线）
// ===========================================================================

indicator! {
    /// Separating Lines（分离线）：第 1 根阴（阳）线，第 2 根同色 belt-hold（相同开盘价、长实体、无对应
    /// 影线）。阳线输出 `+100`，阴线输出 `−100`。`lookback = max(max(ShadowVeryShort, BodyLong), Equal) + 1 = 11`。
    /// `ShadowVeryShort`/`BodyLong` 用 `OFF=0`，`Equal` 用 `OFF=1`（引用 `i−1`）。
    ///
    /// Separating Lines: 1st black/white candle, 2nd same-color belt-hold with the same open, long body and
    /// no corresponding shadow. White → `+100`, black → `−100`. `lookback = 11`. `Equal` uses `OFF=1`.
    fn cdl_separatinglines(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_separatinglines_with_output init zero;
}

/// `cdl_separatinglines` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_separatinglines`]。
/// Zero-copy variant of [`cdl_separatinglines`]: writes results into `out` (length must equal input length).
pub fn cdl_separatinglines_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_separatinglines_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_separatinglines")?;
    let n = open.len();
    let lookback = SHADOW_VERY_SHORT
        .avg_period
        .max(BODY_LONG.avg_period)
        .max(EQUAL.avg_period)
        + 1; // 11
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_shadow_vs = {
        let mut s = lookback - 0 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_shadow_vs = lookback - 0 - 10;
    let mut sum_avg_body_long = {
        let mut s = lookback - 0 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_long = lookback - 0 - 10;
    let mut sum_avg_eq = {
        let mut s = lookback - 1 - 5;
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_eq = lookback - 1 - 5;
    let mut i = lookback;
    while i < n {
        let cur_avg_shadow_vs = high_low_range(high[i], low[i]);
        let val_avg_shadow_vs = sum_avg_shadow_vs / 10 as f64 * 0.1;
        let cur_avg_body_long = real_body(open[i], close[i]);
        let val_avg_body_long = sum_avg_body_long / 10 as f64 * 1.0;
        let cur_avg_eq = high_low_range(high[i - 1], low[i - 1]);
        let val_avg_eq = sum_avg_eq / 5 as f64 * 0.05;

        let ci = candle_color(open[i], close[i]);
        let same_open = open[i] <= open[i - 1] + val_avg_eq && open[i] >= open[i - 1] - val_avg_eq;
        let long_body = cur_avg_body_long > val_avg_body_long;
        let shadow_ok = if ci == 1.0 {
            // bullish: no lower shadow
            lower_shadow(open[i], low[i], close[i]) < val_avg_shadow_vs
        } else {
            // bearish: no upper shadow
            upper_shadow(open[i], high[i], close[i]) < val_avg_shadow_vs
        };
        out[i] = if candle_color(open[i - 1], close[i - 1]) == -ci
            && same_open
            && long_body
            && shadow_ok
        {
            ci * 100.0
        } else {
            0.0
        };
        sum_avg_shadow_vs +=
            cur_avg_shadow_vs - high_low_range(high[trail_avg_shadow_vs], low[trail_avg_shadow_vs]);
        trail_avg_shadow_vs += 1;
        sum_avg_body_long +=
            cur_avg_body_long - real_body(open[trail_avg_body_long], close[trail_avg_body_long]);
        trail_avg_body_long += 1;
        sum_avg_eq += cur_avg_eq - high_low_range(high[trail_avg_eq], low[trail_avg_eq]);
        trail_avg_eq += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_shortline — Short Line Candle（短实体蜡烛）
// ===========================================================================

indicator! {
    /// Short Line Candle（短实体蜡烛）：短实体 + 极短上下影线。阳线输出 `+100`，阴线输出 `−100`。
    /// `lookback = max(BodyShort, ShadowShort) = 10`，两者 `off=0`。
    ///
    /// Short Line: short real body with very short upper & lower shadows. White → `+100`, black → `−100`.
    /// `lookback = 10`.
    fn cdl_shortline(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_shortline_with_output init zero;
}

/// `cdl_shortline` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_shortline`]。
/// Zero-copy variant of [`cdl_shortline`]: writes results into `out` (length must equal input length).
pub fn cdl_shortline_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_shortline_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_shortline")?;
    let n = open.len();
    let lookback = BODY_SHORT.avg_period.max(SHADOW_SHORT.avg_period); // 10
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
    let mut sum_avg_shadow = {
        let mut s = lookback - 0 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += upper_shadow(open[s], high[s], close[s])
                + lower_shadow(open[s], low[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_shadow = lookback - 0 - 10;
    let mut i = lookback;
    while i < n {
        let cur_avg_body = real_body(open[i], close[i]);
        let val_avg_body = sum_avg_body / 10 as f64 * 1.0;
        let cur_upper = upper_shadow(open[i], high[i], close[i]);
        let cur_lower = lower_shadow(open[i], low[i], close[i]);
        let cur_avg_shadow = cur_upper + cur_lower;
        let val_avg_shadow = sum_avg_shadow / 10 as f64 * 1.0 / 2.0;
        out[i] = if cur_avg_body < val_avg_body
            && cur_upper < val_avg_shadow
            && cur_lower < val_avg_shadow
        {
            candle_color(open[i], close[i]) * 100.0
        } else {
            0.0
        };
        sum_avg_body += cur_avg_body - real_body(open[trail_avg_body], close[trail_avg_body]);
        trail_avg_body += 1;
        sum_avg_shadow += cur_avg_shadow
            - (upper_shadow(
                open[trail_avg_shadow],
                high[trail_avg_shadow],
                close[trail_avg_shadow],
            ) + lower_shadow(
                open[trail_avg_shadow],
                low[trail_avg_shadow],
                close[trail_avg_shadow],
            ));
        trail_avg_shadow += 1;
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// cdl_spinningtop — Spinning Top（纺锤线）
// ===========================================================================

indicator! {
    /// Spinning Top（纺锤线）：小实体 + 上下影线均长于实体。阳线输出 `+100`，阴线输出 `−100`。
    /// `lookback = BodyShort = 10`，`off=0`。
    ///
    /// Spinning Top: small real body with both shadows longer than the real body. White → `+100`,
    /// black → `−100`. `lookback = 10`.
    fn cdl_spinningtop(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_spinningtop_with_output init zero;
}

/// `cdl_spinningtop` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_spinningtop`]。
/// Zero-copy variant of [`cdl_spinningtop`]: writes results into `out` (length must equal input length).
pub fn cdl_spinningtop_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_spinningtop_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_spinningtop")?;
    let n = open.len();
    let lookback = BODY_SHORT.avg_period; // 10
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
    let mut i = lookback;
    while i < n {
        let cur_avg_body = real_body(open[i], close[i]);
        let val_avg_body = sum_avg_body / 10 as f64 * 1.0;

        let rb = real_body(open[i], close[i]);
        out[i] = if rb < val_avg_body
            && upper_shadow(open[i], high[i], close[i]) > rb
            && lower_shadow(open[i], low[i], close[i]) > rb
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
// cdl_stalledpattern — Stalled Pattern（停顿形态 / 残缺三兵）
// ===========================================================================

indicator! {
    /// Stalled Pattern（停顿形态）：3 根连续创新高的阳线。第 1、2 根长阳（第 2 根极短上影线、开盘在第 1
    /// 根实体内/附近），第 3 根小阳线"骑在"第 2 根实体的肩部。恒为看跌 `−100`。
    /// `lookback = max(max(BodyLong, BodyShort), max(ShadowVeryShort, Near)) + 2 = 12`。
    /// `BodyLong` 用 `OFF=2 / 1`、`ShadowVeryShort` 用 `OFF=1`、`BodyShort` 用 `OFF=0`、
    /// `Near` 用 `OFF=2 / 1`。
    ///
    /// Stalled Pattern: three white candles with consecutively higher closes. 1st & 2nd long white (2nd with
    /// very short upper shadow, opening within/near the 1st body), 3rd small white riding on the 2nd's shoulder.
    /// Always bearish `−100`. `lookback = 12`.
    fn cdl_stalledpattern(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> with cdl_stalledpattern_with_output init zero;
}

/// `cdl_stalledpattern` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_stalledpattern`]。
/// Zero-copy variant of [`cdl_stalledpattern`]: writes results into `out` (length must equal input length).
pub fn cdl_stalledpattern_with_output(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam(
            "cdl_stalledpattern_with_output: out length must equal input length".into(),
        ));
    }

    check_ohlc(open, high, low, close, "cdl_stalledpattern")?;
    let n = open.len();
    let lookback = BODY_LONG
        .avg_period
        .max(BODY_SHORT.avg_period)
        .max(SHADOW_VERY_SHORT.avg_period)
        .max(NEAR.avg_period)
        + 2; // 12
    if n <= lookback {
        return Ok(());
    }
    let mut sum_avg_body_long2 = {
        let mut s = lookback - 2 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 2) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_long2 = lookback - 2 - 10;
    let mut sum_avg_body_long1 = {
        let mut s = lookback - 1 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_long1 = lookback - 1 - 10;
    let mut sum_avg_body_short = {
        let mut s = lookback - 0 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 0) {
            acc += real_body(open[s], close[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_body_short = lookback - 0 - 10;
    let mut sum_avg_shadow_vs = {
        let mut s = lookback - 1 - 10;
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_shadow_vs = lookback - 1 - 10;
    let mut sum_avg_near2 = {
        let mut s = lookback - 2 - 5;
        let mut acc = 0.0_f64;
        while s < (lookback - 2) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_near2 = lookback - 2 - 5;
    let mut sum_avg_near1 = {
        let mut s = lookback - 1 - 5;
        let mut acc = 0.0_f64;
        while s < (lookback - 1) {
            acc += high_low_range(high[s], low[s]);
            s += 1;
        }
        acc
    };
    let mut trail_avg_near1 = lookback - 1 - 5;
    let mut i = lookback;
    while i < n {
        let cur_avg_body_long2 = real_body(open[i - 2], close[i - 2]);
        let val_avg_body_long2 = sum_avg_body_long2 / 10 as f64 * 1.0;
        let cur_avg_body_long1 = real_body(open[i - 1], close[i - 1]);
        let val_avg_body_long1 = sum_avg_body_long1 / 10 as f64 * 1.0;
        let cur_avg_body_short = real_body(open[i], close[i]);
        let val_avg_body_short = sum_avg_body_short / 10 as f64 * 1.0;
        let cur_avg_shadow_vs = high_low_range(high[i - 1], low[i - 1]);
        let val_avg_shadow_vs = sum_avg_shadow_vs / 10 as f64 * 0.1;
        let cur_avg_near2 = high_low_range(high[i - 2], low[i - 2]);
        let val_avg_near2 = sum_avg_near2 / 5 as f64 * 0.2;
        let cur_avg_near1 = high_low_range(high[i - 1], low[i - 1]);
        let val_avg_near1 = sum_avg_near1 / 5 as f64 * 0.2;
        out[i] = if candle_color(open[i - 2], close[i - 2]) == 1.0 // 1st white
            && candle_color(open[i - 1], close[i - 1]) == 1.0 // 2nd white
            && candle_color(open[i], close[i]) == 1.0 // 3rd white
            && close[i] > close[i - 1] && close[i - 1] > close[i - 2] // consecutive higher closes
            && real_body(open[i - 2], close[i - 2]) > val_avg_body_long2 // 1st long
            && real_body(open[i - 1], close[i - 1]) > val_avg_body_long1 // 2nd long
            && upper_shadow(open[i - 1], high[i - 1], close[i - 1]) < val_avg_shadow_vs // very short upper shadow
            && open[i - 1] > open[i - 2] // opens within 1st real body
            && open[i - 1] <= close[i - 2] + val_avg_near2
            && cur_avg_body_short < val_avg_body_short // 3rd small
            && open[i] >= close[i - 1] - cur_avg_body_short - val_avg_near1
        // rides shoulder
        {
            -100.0
        } else {
            0.0
        };
        sum_avg_body_long2 +=
            cur_avg_body_long2 - real_body(open[trail_avg_body_long2], close[trail_avg_body_long2]);
        trail_avg_body_long2 += 1;
        sum_avg_body_long1 +=
            cur_avg_body_long1 - real_body(open[trail_avg_body_long1], close[trail_avg_body_long1]);
        trail_avg_body_long1 += 1;
        sum_avg_body_short +=
            cur_avg_body_short - real_body(open[trail_avg_body_short], close[trail_avg_body_short]);
        trail_avg_body_short += 1;
        sum_avg_shadow_vs +=
            cur_avg_shadow_vs - high_low_range(high[trail_avg_shadow_vs], low[trail_avg_shadow_vs]);
        trail_avg_shadow_vs += 1;
        sum_avg_near2 +=
            cur_avg_near2 - high_low_range(high[trail_avg_near2], low[trail_avg_near2]);
        trail_avg_near2 += 1;
        sum_avg_near1 +=
            cur_avg_near1 - high_low_range(high[trail_avg_near1], low[trail_avg_near1]);
        trail_avg_near1 += 1;
        i += 1;
    }

    Ok(())
}
