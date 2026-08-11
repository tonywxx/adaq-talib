//! # 形态识别 · 第 1 批（验证批）/ Pattern Recognition · Batch 1 (validation batch)
//!
//! 本批为 `CandleAvg` 助手与蜡烛原语的**验证批**，覆盖全部助手特性：
//! 1-candle 运行和（`cdl_doji` / `cdl_marubozu`）、`OFF=1` 的 `Near`（`cdl_hammer`）、
//! 影线非常长（`avgPeriod=0`，`cdl_highwave`）、纯 2-candle 比较（`cdl_engulfing` /
//! `cdl_shootingstar`）、`OFF=1` 的 2-candle（`cdl_harami`）、`OFF=2` 的 3-candle 多根
//! （`cdl_2crows`）。全部与 TA-Lib 0.7.1 黄金向量逐项 1:1。
//!
//! This is the **validation batch** for the [`CandleAvg`](super::CandleAvg) helper and candle
//! primitives, exercising every helper feature: 1-candle running-sum (`cdl_doji` /
//! `cdl_marubozu`), `OFF=1` `Near` (`cdl_hammer`), very-long shadow (`avgPeriod=0`,
//! `cdl_highwave`), pure 2-candle comparison (`cdl_engulfing` / `cdl_shootingstar`),
//! `OFF=1` 2-candle (`cdl_harami`), and `OFF=2` 3-candle multi-bar (`cdl_2crows`). All
//! bit-identical to TA-Lib 0.7.1 golden vectors.

use crate::error::TaError;
use super::*;

// ===========================================================================
// cdl_doji — Doji（十字星）
// ===========================================================================

/// Doji（十字星）：开盘价 ≈ 收盘价（实体极小）。
///
/// `outInteger` 恒为 `100`（十字星表示犹豫，本身不判多空）。
/// 对应 C 源 `ta_CDLDOJI.c`：`realBody(i) <= CANDLEAVERAGE(BodyDoji, i)`。
///
/// Doji: open ≈ close (very small real body). Output is always `100`.
pub fn cdl_doji(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_doji_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_doji` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_doji`]。
/// Zero-copy variant of [`cdl_doji`]: writes results into `out` (length must equal input length).
pub fn cdl_doji_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_doji_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_doji")?;
    let n = open.len();
    let lookback = BODY_DOJI.avg_period; // 10
    if n <= lookback {
        return Ok(());
    }
    let mut avg = CandleAvg::new(BODY_DOJI, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        out[i] = if real_body(open[i], close[i]) <= avg.value(i, open, high, low, close) {
            100.0
        } else {
            0.0
        };
        avg.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_marubozu — Marubozu（光头光脚）
// ===========================================================================

/// Marubozu（光头光脚）：长实体、无/极短影线。
///
/// 阳线（close ≥ open）输出 `+100`，阴线输出 `−100`；否则 `0`。
/// 对应 C 源 `ta_CDLMARUBOZU.c`：`lookback = max(BodyLong, ShadowVeryShort)`。
///
/// Marubozu: long real body with no/very-short shadows. Bullish → `+100`, bearish → `−100`.
pub fn cdl_marubozu(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_marubozu_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_marubozu` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_marubozu`]。
/// Zero-copy variant of [`cdl_marubozu`]: writes results into `out` (length must equal input length).
pub fn cdl_marubozu_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_marubozu_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_marubozu")?;
    let n = open.len();
    let lookback = BODY_LONG.avg_period.max(SHADOW_VERY_SHORT.avg_period); // max(10,10)=10
    if n <= lookback {
        return Ok(());
    }
    let mut avg_body = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 0);
    let mut avg_shadow = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        if real_body(open[i], close[i]) > avg_body.value(i, open, high, low, close)
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
    Ok(())
}


// ===========================================================================
// cdl_hammer — Hammer（锤头）
// ===========================================================================

/// Hammer（锤头）：小实体、长下影线、无/极短上影线，且实体靠近前一根最低价附近。
///
/// 锤头恒为看涨（`+100`）。`Near` 设置使用 `OFF=1`，引用 `i−1`
/// （见 `ta_CDLHAMMER.c`）。`lookback = max(max(max(BodyShort, ShadowLong), ShadowVeryShort), Near) + 1 = 11`。
///
/// Hammer: small body, long lower shadow, no/very-short upper shadow, body near prior low.
/// Always bullish (`+100`). `Near` uses `OFF=1` (references `i−1`), `lookback = 11`.
pub fn cdl_hammer(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_hammer_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_hammer` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_hammer`]。
/// Zero-copy variant of [`cdl_hammer`]: writes results into `out` (length must equal input length).
pub fn cdl_hammer_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_hammer_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_hammer")?;
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
    let mut avg_body = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 0);
    let mut avg_shadow_long = CandleAvg::new(SHADOW_LONG, open, high, low, close, lookback, 0);
    let mut avg_shadow_vshort = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 0);
    // Near 使用 OFF=1：引用 i-1（见 ta_CDLHAMMER.c 的 `inLow[i-1] + TA_CANDLEAVERAGE(Near, ..., i-1)`）。
    let mut avg_near = CandleAvg::new(NEAR, open, high, low, close, lookback, 1);
    let mut i = lookback;
    while i < n {
        if real_body(open[i], close[i]) < avg_body.value(i, open, high, low, close)
            && lower_shadow(open[i], low[i], close[i]) > avg_shadow_long.value(i, open, high, low, close)
            && upper_shadow(open[i], high[i], close[i]) < avg_shadow_vshort.value(i, open, high, low, close)
            && open[i].min(close[i]) <= low[i - 1] + avg_near.value(i, open, high, low, close)
        {
            out[i] = 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body.advance(i, open, high, low, close);
        avg_shadow_long.advance(i, open, high, low, close);
        avg_shadow_vshort.advance(i, open, high, low, close);
        avg_near.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_shootingstar — Shooting Star（射击之星）
// ===========================================================================

/// Shooting Star（射击之星）：小实体、长上影线、无/极短下影线，且相对前一根实体向上跳空。
///
/// 射击之星恒为看跌（`−100`）。`lookback = max(max(BodyShort, ShadowLong), ShadowVeryShort) = 10`。
///
/// Shooting Star: small body, long upper shadow, no/very-short lower shadow, gap up from prior body.
/// Always bearish (`−100`), `lookback = 10`.
pub fn cdl_shootingstar(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_shootingstar_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_shootingstar` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_shootingstar`]。
/// Zero-copy variant of [`cdl_shootingstar`]: writes results into `out` (length must equal input length).
pub fn cdl_shootingstar_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_shootingstar_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_shootingstar")?;
    let n = open.len();
    let lookback = [
        BODY_SHORT.avg_period,
        SHADOW_LONG.avg_period,
        SHADOW_VERY_SHORT.avg_period,
    ]
    .iter()
    .max()
    .copied()
    .unwrap(); // 10
    if n <= lookback {
        return Ok(());
    }
    let mut avg_body = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 0);
    let mut avg_shadow_long = CandleAvg::new(SHADOW_LONG, open, high, low, close, lookback, 0);
    let mut avg_shadow_vshort = CandleAvg::new(SHADOW_VERY_SHORT, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        if real_body(open[i], close[i]) < avg_body.value(i, open, high, low, close)
            && upper_shadow(open[i], high[i], close[i]) > avg_shadow_long.value(i, open, high, low, close)
            && lower_shadow(open[i], low[i], close[i]) < avg_shadow_vshort.value(i, open, high, low, close)
            && real_body_gap_up(open[i], close[i], open[i - 1], close[i - 1])
        {
            out[i] = -100.0;
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
// cdl_engulfing — Engulfing Pattern（吞没形态）
// ===========================================================================

/// Engulfing Pattern（吞没形态）：第 2 根实体吞没第 1 根实体。
///
/// 与已安装的 TA-Lib 0.7.1 dylib 一致（见 ADR 0005），采用**两级**输出：
/// - 完全吞没（`open[i] < close[i-1]`，bull；`open[i] > close[i-1]`，bear）→ `±100`；
/// - 边界弱吞没（`open[i] == close[i-1]`，即第 2 根开盘恰等于前收）→ `±80`。
///
/// 此为 dylib 修订版行为（上游 C 源 `ta_CDLENGULFING.c` 仅返回 `±100`）；黄金向量由 dylib
/// 生成，本实现严格对齐 dylib。纯 2-candle 比较，无蜡烛设置，`lookback = 2`。
///
/// Engulfing with the installed TA-Lib 0.7.1 dylib's **two-tier** output (ADR 0005):
/// full engulf → `±100`; boundary weak engulf (`open[i] == close[i-1]`) → `±80`.
/// Pure 2-candle comparison, no candle settings, `lookback = 2`.
pub fn cdl_engulfing(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_engulfing_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_engulfing` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_engulfing`]。
/// Zero-copy variant of [`cdl_engulfing`]: writes results into `out` (length must equal input length).
pub fn cdl_engulfing_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_engulfing_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_engulfing")?;
    let n = open.len();
    let lookback = 2; // TA_CDLENGULFING_Lookback() returns 2（见 ta_CDLENGULFING.c）
    if n <= lookback {
        return Ok(());
    }
    let mut i = lookback;
    while i < n {
        let cur_color = candle_color(open[i], close[i]);
        let prev_color = candle_color(open[i - 1], close[i - 1]);
        let bull = cur_color == 1.0 && prev_color == -1.0 && close[i] > open[i - 1];
        let bear = cur_color == -1.0 && prev_color == 1.0 && close[i] < open[i - 1];
        if bull && open[i] < close[i - 1] {
            out[i] = 100.0;
        } else if bear && open[i] > close[i - 1] {
            out[i] = -100.0;
        } else if bull && open[i] <= close[i - 1] {
            out[i] = 80.0;
        } else if bear && open[i] >= close[i - 1] {
            out[i] = -80.0;
        } else {
            out[i] = 0.0;
        }
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_harami — Harami Pattern（孕线形态）
// ===========================================================================

/// Harami Pattern（孕线形态）：第 1 根长实体，第 2 根短实体完全被第 1 根实体包裹。
///
/// 第 1 根为阳线则看跌（`−100`），为阴线则看涨（`+100`）。`BodyLong` 使用 `OFF=1`
/// （引用 `i−1`），`lookback = max(BodyLong, BodyShort) + 1 = 11`。对应 `ta_CDLHARAMI.c`。
///
/// Harami: 1st a long real body, 2nd a short body totally engulfed by the 1st.
/// `BodyLong` uses `OFF=1` (references `i−1`), `lookback = 11`.
pub fn cdl_harami(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_harami_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_harami` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_harami`]。
/// Zero-copy variant of [`cdl_harami`]: writes results into `out` (length must equal input length).
pub fn cdl_harami_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_harami_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_harami")?;
    let n = open.len();
    let lookback = BODY_LONG.avg_period.max(BODY_SHORT.avg_period) + 1; // 11
    if n <= lookback {
        return Ok(());
    }
    let mut avg_body_long = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 1);
    let mut avg_body_short = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        if real_body(open[i - 1], close[i - 1]) > avg_body_long.value(i, open, high, low, close)
            && real_body(open[i], close[i]) <= avg_body_short.value(i, open, high, low, close)
            && open[i].max(close[i]) < open[i - 1].max(close[i - 1])
            && open[i].min(close[i]) > open[i - 1].min(close[i - 1])
        {
            out[i] = -candle_color(open[i - 1], close[i - 1]) * 100.0;
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
// cdl_highwave — High-Wave（高空浪）
// ===========================================================================

/// High-Wave（高空浪）：短实体、极长上下影线。
///
/// 阳线输出 `+100`，阴线输出 `−100`（不判多空）。`ShadowVeryLong` 为 `avgPeriod=0`
/// （直接用当前 K 线范围）。`lookback = max(BodyShort, ShadowVeryLong) = 10`。对应 `ta_CDLHIGHWAVE.c`。
///
/// High-Wave: short body, very long upper & lower shadows. White → `+100`, black → `−100`.
/// `ShadowVeryLong` has `avgPeriod=0`; `lookback = 10`.
pub fn cdl_highwave(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_highwave_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_highwave` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_highwave`]。
/// Zero-copy variant of [`cdl_highwave`]: writes results into `out` (length must equal input length).
pub fn cdl_highwave_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_highwave_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_highwave")?;
    let n = open.len();
    let lookback = BODY_SHORT.avg_period.max(SHADOW_VERY_LONG.avg_period); // max(10,0)=10
    if n <= lookback {
        return Ok(());
    }
    let mut avg_body = CandleAvg::new(BODY_SHORT, open, high, low, close, lookback, 0);
    let mut avg_shadow = CandleAvg::new(SHADOW_VERY_LONG, open, high, low, close, lookback, 0);
    let mut i = lookback;
    while i < n {
        if real_body(open[i], close[i]) < avg_body.value(i, open, high, low, close)
            && upper_shadow(open[i], high[i], close[i]) > avg_shadow.value(i, open, high, low, close)
            && lower_shadow(open[i], low[i], close[i]) > avg_shadow.value(i, open, high, low, close)
        {
            out[i] = candle_color(open[i], close[i]) * 100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body.advance(i, open, high, low, close);
        avg_shadow.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}


// ===========================================================================
// cdl_2crows — Two Crows（两只乌鸦）
// ===========================================================================

/// Two Crows（两只乌鸦）：3-candle 顶部反转。第 1 根长阳线，第 2 根向上跳空阴线，
/// 第 3 根阴线开盘在第 2 根实体内、收盘在第 1 根实体内。
///
/// 两只乌鸦恒为看跌（`−100`）。`BodyLong` 使用 `OFF=2`（引用 `i−2`），
/// `lookback = BodyLong + 2 = 12`。对应 `ta_CDL2CROWS.c`。
///
/// Two Crows: 3-candle top reversal. Always bearish (`−100`). `BodyLong` uses `OFF=2`
/// (references `i−2`), `lookback = 12`.
pub fn cdl_2crows(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, TaError> {
    let mut out = vec![0.0_f64; open.len()];
    cdl_2crows_with_output(open, high, low, close, &mut out)?;
    Ok(out)
}

/// `cdl_2crows` 的零拷贝变体：将结果写入 `out`（长度须等于输入长度）。见 [`cdl_2crows`]。
/// Zero-copy variant of [`cdl_2crows`]: writes results into `out` (length must equal input length).
pub fn cdl_2crows_with_output(
    open: &[f64], high: &[f64], low: &[f64], close: &[f64],
    out: &mut [f64],
) -> Result<(), TaError> {
    if out.len() != open.len() {
        return Err(TaError::BadParam("cdl_2crows_with_output: out length must equal input length".into()));
    }

    check_ohlc(open, high, low, close, "cdl_2crows")?;
    let n = open.len();
    let lookback = BODY_LONG.avg_period + 2; // 12
    if n <= lookback {
        return Ok(());
    }
    let mut avg_body_long = CandleAvg::new(BODY_LONG, open, high, low, close, lookback, 2);
    let mut i = lookback;
    while i < n {
        if candle_color(open[i - 2], close[i - 2]) == 1.0 // 1st: white
            && real_body(open[i - 2], close[i - 2]) > avg_body_long.value(i, open, high, low, close)
            && candle_color(open[i - 1], close[i - 1]) == -1.0 // 2nd: black
            && real_body_gap_up(open[i - 1], close[i - 1], open[i - 2], close[i - 2]) // gapping up
            && candle_color(open[i], close[i]) == -1.0 // 3rd: black
            && open[i] < open[i - 1]
            && open[i] > close[i - 1]
            && close[i] > open[i - 2]
            && close[i] < close[i - 2]
        {
            out[i] = -100.0;
        } else {
            out[i] = 0.0;
        }
        avg_body_long.advance(i, open, high, low, close);
        i += 1;
    }
    Ok(())
}

