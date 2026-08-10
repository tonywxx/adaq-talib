//! 形态识别（Pattern Recognition）黄金向量比对测试，第 2 批。见 ADR 0003 / ADR 0005。
//!
//! Golden-vector comparison tests for Pattern Recognition, batch 2 (ADR 0003 / ADR 0005).
//! Fixtures are authoritative golden vectors generated from the installed TA-Lib 0.7.1 dylib
//! via `tools/gen_fixtures/generate.py` (`tests/fixtures/cdl_*.json`, leading `lookback`
//! positions `0.0`, integer-output convention, ADR 0007).

#[path = "common/mod.rs"]
mod common;

use adaq_talib::pattern::{
    cdl_3blackcrows, cdl_3inside, cdl_3linestrike, cdl_3outside, cdl_3starsinsouth,
    cdl_3whitesoldiers, cdl_abandonedbaby, cdl_advanceblock,
};
use adaq_talib::utils::approx_eq_slice;

/// 通用比对：加载 `cdl_<name>.json`，调用 `f(open, high, low, close)`，与 `expected` 逐项比对。
fn check_cdl(
    name: &str,
    f: fn(&[f64], &[f64], &[f64], &[f64]) -> Result<Vec<f64>, adaq_talib::error::TaError>,
) {
    let fixture = format!("cdl_{name}.json");
    let json = common::load_json(&fixture).expect("load fixture");
    let open = common::load_f64_array(&json, "open").expect("open");
    let high = common::load_f64_array(&json, "high").expect("high");
    let low = common::load_f64_array(&json, "low").expect("low");
    let close = common::load_f64_array(&json, "close").expect("close");
    let expected = common::load_f64_array(&json, "expected").expect("expected");
    let out = f(&open, &high, &low, &close).expect("pattern fn");
    assert_eq!(out.len(), expected.len(), "length mismatch for cdl_{name}");
    assert!(
        approx_eq_slice(&out, &expected),
        "cdl_{name} output deviates from golden vector beyond ADR 0005 tolerance"
    );
}

#[test]
fn cdl_3blackcrows_matches_golden_vector() {
    check_cdl("3blackcrows", cdl_3blackcrows);
}

#[test]
fn cdl_3inside_matches_golden_vector() {
    check_cdl("3inside", cdl_3inside);
}

#[test]
fn cdl_3linestrike_matches_golden_vector() {
    check_cdl("3linestrike", cdl_3linestrike);
}

#[test]
fn cdl_3outside_matches_golden_vector() {
    check_cdl("3outside", cdl_3outside);
}

#[test]
fn cdl_3starsinsouth_matches_golden_vector() {
    check_cdl("3starsinsouth", cdl_3starsinsouth);
}

#[test]
fn cdl_3whitesoldiers_matches_golden_vector() {
    check_cdl("3whitesoldiers", cdl_3whitesoldiers);
}

#[test]
fn cdl_abandonedbaby_matches_golden_vector() {
    check_cdl("abandonedbaby", cdl_abandonedbaby);
}

#[test]
fn cdl_advanceblock_matches_golden_vector() {
    check_cdl("advanceblock", cdl_advanceblock);
}

// ===========================================================================
// 手工构造 OHLC 的逻辑校验 / Crafted OHLC logic checks
// ===========================================================================
//
// 由于带蜡烛设置（CandleAvg）的形态需要 `n > lookback`（12~13）根前置数据做预热，
// 这里先用一段统一基线（每根 o=100,h=103,l=98,c=101，阳线、实体=1、上下影线=2）铺设足够
// 长的历史，再在末尾覆盖 3~4 根形态 K 线。基线使各蜡烛均值稳定（ShadowVeryShort≈0.5、
// BodyLong/BodyShort≈1.0、BodyDoji≈0.5、Near≈1.0、Far≈3.0、ShadowShort≈2.0），便于手工
// 触发/否定形态。
//
// Settings-based patterns need `n > lookback` (12~13) warm-up bars. We lay a uniform
// baseline (o=100,h=103,l=98,c=101: white, body=1, shadows=2) then override the last few
// bars with the pattern candles so the candle averages are well-defined.

/// 生成 `n` 根相同基线 K 线。/ Build `n` identical baseline candles.
fn baseline(n: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut o = Vec::with_capacity(n);
    let mut h = Vec::with_capacity(n);
    let mut l = Vec::with_capacity(n);
    let mut c = Vec::with_capacity(n);
    for _ in 0..n {
        o.push(100.0);
        h.push(103.0);
        l.push(98.0);
        c.push(101.0);
    }
    (o, h, l, c)
}

// Three Black Crows: prior white, then 3 declining blacks with very short lower shadows,
// each opening within the prior body, 1st black closing under the prior white high.
#[test]
fn cdl_3blackcrows_triggers_and_rejects() {
    let (mut o, mut h, mut l, mut c) = baseline(20);
    // bars 20,21,22 = 3 declining blacks
    o.extend_from_slice(&[100.0, 99.8, 99.5]);
    h.extend_from_slice(&[100.5, 99.9, 99.6]);
    l.extend_from_slice(&[99.4, 98.9, 98.4]);
    c.extend_from_slice(&[99.5, 99.0, 98.5]);
    let out = cdl_3blackcrows(&o, &h, &l, &c).unwrap();
    assert_eq!(out[22], -100.0, "three black crows should trigger at i=22");

    // Reject: 3rd black opens ABOVE the 2nd black's open.
    let (mut o2, mut h2, mut l2, mut c2) = baseline(20);
    o2.extend_from_slice(&[100.0, 99.8, 99.9]);
    h2.extend_from_slice(&[100.5, 99.9, 99.6]);
    l2.extend_from_slice(&[99.4, 98.9, 98.4]);
    c2.extend_from_slice(&[99.5, 99.0, 98.5]);
    let out2 = cdl_3blackcrows(&o2, &h2, &l2, &c2).unwrap();
    assert_eq!(out2[22], 0.0, "should not trigger when open[i] not < open[i-1]");
}

// Three Inside Up: long black 1st, short white 2nd engulfed, 3rd white closing above 1st open.
#[test]
fn cdl_3inside_triggers_and_rejects() {
    let (mut o, mut h, mut l, mut c) = baseline(13);
    // bars 13,14,15
    o.extend_from_slice(&[101.0, 99.0, 100.0]);
    h.extend_from_slice(&[101.5, 100.0, 103.0]);
    l.extend_from_slice(&[94.5, 98.0, 99.0]);
    c.extend_from_slice(&[95.0, 99.5, 102.0]);
    let out = cdl_3inside(&o, &h, &l, &c).unwrap();
    assert_eq!(out[15], 100.0, "three inside up (1st black) should be +100");

    // Reject: 3rd does not close above the 1st open.
    let (mut o2, mut h2, mut l2, mut c2) = baseline(13);
    o2.extend_from_slice(&[101.0, 99.0, 100.0]);
    h2.extend_from_slice(&[101.5, 100.0, 103.0]);
    l2.extend_from_slice(&[94.5, 98.0, 99.0]);
    c2.extend_from_slice(&[95.0, 99.5, 100.0]);
    let out2 = cdl_3inside(&o2, &h2, &l2, &c2).unwrap();
    assert_eq!(out2[15], 0.0, "should not trigger without 3rd closing out");
}

// Three-Line Strike (bullish): 3 white soldiers then a black 4th opening above prior close
// and closing below the 1st open.
#[test]
fn cdl_3linestrike_triggers_and_rejects() {
    let (mut o, mut h, mut l, mut c) = baseline(11);
    // bars 11,12,13,14
    o.extend_from_slice(&[100.0, 100.5, 101.5, 104.0]);
    h.extend_from_slice(&[103.0, 104.0, 105.0, 104.5]);
    l.extend_from_slice(&[99.0, 99.5, 100.5, 98.5]);
    c.extend_from_slice(&[101.0, 102.0, 103.0, 99.0]);
    let out = cdl_3linestrike(&o, &h, &l, &c).unwrap();
    assert_eq!(out[14], 100.0, "bullish 3-line strike should be +100");

    // Reject: 4th does not close below the 1st open.
    let (mut o2, mut h2, mut l2, mut c2) = baseline(11);
    o2.extend_from_slice(&[100.0, 100.5, 101.5, 104.0]);
    h2.extend_from_slice(&[103.0, 104.0, 105.0, 104.5]);
    l2.extend_from_slice(&[99.0, 99.5, 100.5, 98.5]);
    c2.extend_from_slice(&[101.0, 102.0, 103.0, 101.0]);
    let out2 = cdl_3linestrike(&o2, &h2, &l2, &c2).unwrap();
    assert_eq!(out2[14], 0.0, "should not trigger when 4th closes inside range");
}

// Three Outside Up: black 1st, white 2nd engulfs it, 3rd closes higher.
// (3-candle pattern: signal emitted at index 3, loop starts at i=lookback=3.)
#[test]
fn cdl_3outside_triggers_and_rejects() {
    let o = [50.0, 100.0, 90.0, 99.5];
    let h = [51.0, 100.5, 101.0, 102.0];
    let l = [49.0, 90.0, 89.0, 99.0];
    let c = [50.5, 91.0, 101.0, 102.0];
    let out = cdl_3outside(&o, &h, &l, &c).unwrap();
    assert_eq!(out[3], 100.0, "three outside up should be +100");

    // Reject: 3rd closes lower instead of higher.
    let c2 = [50.5, 91.0, 101.0, 100.0];
    let out2 = cdl_3outside(&o, &h, &l, &c2).unwrap();
    assert_eq!(out2[3], 0.0, "should not trigger without 3rd higher close");
}

// Three Stars In The South: three blacks, 1st long with long lower shadow, 3rd small marubozu.
#[test]
fn cdl_3starsinsouth_triggers_and_rejects() {
    let (mut o, mut h, mut l, mut c) = baseline(13);
    // bars 13,14,15
    o.extend_from_slice(&[100.0, 98.0, 97.0]);
    h.extend_from_slice(&[100.5, 99.0, 97.4]);
    l.extend_from_slice(&[89.0, 93.0, 96.0]);
    c.extend_from_slice(&[95.0, 94.0, 96.5]);
    let out = cdl_3starsinsouth(&o, &h, &l, &c).unwrap();
    assert_eq!(out[15], 100.0, "three stars in the south should be +100");

    // Reject: 3rd not a small marubozu (has a long upper shadow).
    let (mut o2, mut h2, mut l2, mut c2) = baseline(13);
    o2.extend_from_slice(&[100.0, 98.0, 97.0]);
    h2.extend_from_slice(&[100.5, 99.0, 100.0]);
    l2.extend_from_slice(&[89.0, 93.0, 96.0]);
    c2.extend_from_slice(&[95.0, 94.0, 96.5]);
    let out2 = cdl_3starsinsouth(&o2, &h2, &l2, &c2).unwrap();
    assert_eq!(out2[15], 0.0, "should not trigger with a long upper shadow on 3rd");
}

// Three White Soldiers: three whites, higher closes, very short upper shadows.
#[test]
fn cdl_3whitesoldiers_triggers_and_rejects() {
    let (mut o, mut h, mut l, mut c) = baseline(13);
    // bars 13,14,15
    o.extend_from_slice(&[100.0, 102.0, 104.0]);
    h.extend_from_slice(&[103.2, 105.2, 106.2]);
    l.extend_from_slice(&[99.0, 101.0, 103.0]);
    c.extend_from_slice(&[103.0, 105.0, 106.0]);
    let out = cdl_3whitesoldiers(&o, &h, &l, &c).unwrap();
    assert_eq!(out[15], 100.0, "three white soldiers should be +100");

    // Reject: 2nd has a long upper shadow.
    let (mut o2, mut h2, mut l2, mut c2) = baseline(13);
    o2.extend_from_slice(&[100.0, 102.0, 104.0]);
    h2.extend_from_slice(&[103.2, 108.0, 106.2]);
    l2.extend_from_slice(&[99.0, 101.0, 103.0]);
    c2.extend_from_slice(&[103.0, 105.0, 106.0]);
    let out2 = cdl_3whitesoldiers(&o2, &h2, &l2, &c2).unwrap();
    assert_eq!(out2[15], 0.0, "should not trigger with a long upper shadow");
}

// Abandoned Baby top: long white, doji gapping up, black closing well within 1st body.
#[test]
fn cdl_abandonedbaby_triggers_and_rejects() {
    let (mut o, mut h, mut l, mut c) = baseline(13);
    // bars 13,14,15
    o.extend_from_slice(&[100.0, 107.1, 106.0]);
    h.extend_from_slice(&[106.0, 107.5, 105.5]);
    l.extend_from_slice(&[99.0, 106.1, 102.0]);
    c.extend_from_slice(&[105.0, 107.0, 102.5]);
    let out = cdl_abandonedbaby(&o, &h, &l, &c).unwrap();
    assert_eq!(out[15], -100.0, "abandoned baby top (bearish) should be -100");

    // Reject: the doji's body dips into the 1st candle (no gap up), so no abandonment.
    let (mut o2, mut h2, mut l2, mut c2) = baseline(13);
    o2.extend_from_slice(&[100.0, 107.1, 106.0]);
    h2.extend_from_slice(&[106.0, 107.5, 105.5]);
    l2.extend_from_slice(&[99.0, 105.5, 102.0]);
    c2.extend_from_slice(&[105.0, 107.0, 102.5]);
    let out2 = cdl_abandonedbaby(&o2, &h2, &l2, &c2).unwrap();
    assert_eq!(out2[15], 0.0, "should not trigger without a gap up to the doji");
}

// Advance Block: three whites but the 3rd has a long upper shadow and smaller body (branch 4).
#[test]
fn cdl_advanceblock_triggers_and_rejects() {
    let (mut o, mut h, mut l, mut c) = baseline(13);
    // bars 13,14,15
    o.extend_from_slice(&[100.0, 104.0, 106.5]);
    h.extend_from_slice(&[106.0, 108.0, 112.5]);
    l.extend_from_slice(&[99.0, 103.0, 105.0]);
    c.extend_from_slice(&[105.0, 107.0, 107.5]);
    let out = cdl_advanceblock(&o, &h, &l, &c).unwrap();
    assert_eq!(out[15], -100.0, "advance block with 3rd long upper shadow should be -100");

    // Reject: clean soldiers, no weakening signs.
    let (mut o2, mut h2, mut l2, mut c2) = baseline(13);
    o2.extend_from_slice(&[100.0, 104.0, 105.0]);
    h2.extend_from_slice(&[106.0, 108.0, 109.5]);
    l2.extend_from_slice(&[99.0, 103.0, 104.0]);
    c2.extend_from_slice(&[105.0, 107.0, 109.0]);
    let out2 = cdl_advanceblock(&o2, &h2, &l2, &c2).unwrap();
    assert_eq!(out2[15], 0.0, "should not trigger for healthy advancing soldiers");
}
