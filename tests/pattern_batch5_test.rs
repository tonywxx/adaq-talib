//! 形态识别（Pattern Recognition）黄金向量比对测试，第 5 批。
//!
//! Golden-vector comparison tests for Pattern Recognition, batch 5. See ADR 0003 / ADR 0005.

#[path = "common/mod.rs"]
mod common;

use adaq_talib::pattern::*;
use adaq_talib::utils::approx_eq_slice;

/// 通用比对：加载 `cdl_<name>.json`，调用 `f(open, high, low, close)`，与 `expected` 逐项比对。
/// Generic compare: load `cdl_<name>.json`, call `f(open, high, low, close)`, compare to `expected`.
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

// ---------------------------------------------------------------------------
// 黄金向量比对 / Golden-vector comparisons
// ---------------------------------------------------------------------------

#[test]
fn cdl_homingpigeon_matches_golden_vector() {
    check_cdl("homingpigeon", cdl_homingpigeon);
}

#[test]
fn cdl_identical3crows_matches_golden_vector() {
    check_cdl("identical3crows", cdl_identical3crows);
}

#[test]
fn cdl_inneck_matches_golden_vector() {
    check_cdl("inneck", cdl_inneck);
}

#[test]
fn cdl_invertedhammer_matches_golden_vector() {
    check_cdl("invertedhammer", cdl_invertedhammer);
}

#[test]
fn cdl_kicking_matches_golden_vector() {
    check_cdl("kicking", cdl_kicking);
}

#[test]
fn cdl_kickingbylength_matches_golden_vector() {
    check_cdl("kickingbylength", cdl_kickingbylength);
}

#[test]
fn cdl_ladderbottom_matches_golden_vector() {
    check_cdl("ladderbottom", cdl_ladderbottom);
}

// ---------------------------------------------------------------------------
// 手工构造触发 / 拒绝用例 / Hand-built trigger & reject cases
// ---------------------------------------------------------------------------

/// 构造 `count` 根中性小实体阴线（用于校准蜡烛均值），返回 (o,h,l,c) 拼接后的四段。
/// Build `count` neutral small-body black candles (calibrates candle averages) appended to the
/// running (o,h,l,c) arrays.
fn seed(o: &mut Vec<f64>, h: &mut Vec<f64>, l: &mut Vec<f64>, c: &mut Vec<f64>, count: usize) {
    for _ in 0..count {
        o.push(100.0);
        h.push(120.0);
        l.push(98.0);
        c.push(99.0);
    }
}

#[test]
fn cdl_homingpigeon_trigger() {
    let mut o = Vec::new();
    let mut h = Vec::new();
    let mut l = Vec::new();
    let mut c = Vec::new();
    seed(&mut o, &mut h, &mut l, &mut c, 20);
    // 1st: long black
    o.push(100.0);
    h.push(100.5);
    l.push(89.5);
    c.push(90.0);
    // 2nd: short black engulfed by 1st
    o.push(99.0);
    h.push(99.2);
    l.push(98.0);
    c.push(98.5);
    let out = cdl_homingpigeon(&o, &h, &l, &c).unwrap();
    assert_eq!(out[out.len() - 1], 100.0, "homingpigeon should trigger");
}

#[test]
fn cdl_homingpigeon_reject() {
    let mut o = Vec::new();
    let mut h = Vec::new();
    let mut l = Vec::new();
    let mut c = Vec::new();
    seed(&mut o, &mut h, &mut l, &mut c, 20);
    o.push(100.0);
    h.push(100.5);
    l.push(89.5);
    c.push(90.0);
    // 2nd opens ABOVE 1st open -> not engulfed
    o.push(101.0);
    h.push(101.2);
    l.push(98.0);
    c.push(99.0);
    let out = cdl_homingpigeon(&o, &h, &l, &c).unwrap();
    assert_eq!(out[out.len() - 1], 0.0, "homingpigeon should not trigger");
}

#[test]
fn cdl_identical3crows_trigger() {
    let mut o = Vec::new();
    let mut h = Vec::new();
    let mut l = Vec::new();
    let mut c = Vec::new();
    seed(&mut o, &mut h, &mut l, &mut c, 20);
    // 3 consecutive declining black, very short lower shadow, opens near prior close
    o.push(100.0);
    h.push(120.0);
    l.push(97.5);
    c.push(98.0);
    o.push(98.0);
    h.push(118.0);
    l.push(97.5);
    c.push(96.0);
    o.push(96.0);
    h.push(116.0);
    l.push(93.5);
    c.push(94.0);
    let out = cdl_identical3crows(&o, &h, &l, &c).unwrap();
    assert_eq!(out[out.len() - 1], -100.0, "identical3crows should trigger");
}

#[test]
fn cdl_identical3crows_reject() {
    let mut o = Vec::new();
    let mut h = Vec::new();
    let mut l = Vec::new();
    let mut c = Vec::new();
    seed(&mut o, &mut h, &mut l, &mut c, 20);
    o.push(100.0);
    h.push(120.0);
    l.push(97.5);
    c.push(98.0);
    o.push(98.0);
    h.push(118.0);
    l.push(97.5);
    c.push(96.0);
    // 3rd opens far below prior close -> not "very close"
    o.push(90.0);
    h.push(116.0);
    l.push(93.5);
    c.push(94.0);
    let out = cdl_identical3crows(&o, &h, &l, &c).unwrap();
    assert_eq!(out[out.len() - 1], 0.0, "identical3crows should not trigger");
}

#[test]
fn cdl_inneck_trigger() {
    let mut o = Vec::new();
    let mut h = Vec::new();
    let mut l = Vec::new();
    let mut c = Vec::new();
    seed(&mut o, &mut h, &mut l, &mut c, 20);
    // 1st: long black
    o.push(100.0);
    h.push(100.5);
    l.push(89.5);
    c.push(90.0);
    // 2nd: white, open below prior low, close slightly into prior body
    o.push(88.0);
    h.push(92.0);
    l.push(87.0);
    c.push(91.0);
    let out = cdl_inneck(&o, &h, &l, &c).unwrap();
    assert_eq!(out[out.len() - 1], -100.0, "inneck should trigger");
}

#[test]
fn cdl_inneck_reject() {
    let mut o = Vec::new();
    let mut h = Vec::new();
    let mut l = Vec::new();
    let mut c = Vec::new();
    seed(&mut o, &mut h, &mut l, &mut c, 20);
    o.push(100.0);
    h.push(100.5);
    l.push(89.5);
    c.push(90.0);
    // 2nd opens ABOVE prior low -> fails "open below previous day low"
    o.push(91.0);
    h.push(92.0);
    l.push(87.0);
    c.push(91.0);
    let out = cdl_inneck(&o, &h, &l, &c).unwrap();
    assert_eq!(out[out.len() - 1], 0.0, "inneck should not trigger");
}

#[test]
fn cdl_invertedhammer_trigger() {
    let mut o = Vec::new();
    let mut h = Vec::new();
    let mut l = Vec::new();
    let mut c = Vec::new();
    seed(&mut o, &mut h, &mut l, &mut c, 20);
    // gap down then small body, long upper shadow, very short lower shadow
    o.push(95.0);
    h.push(110.0);
    l.push(94.5);
    c.push(94.8);
    let out = cdl_invertedhammer(&o, &h, &l, &c).unwrap();
    assert_eq!(out[out.len() - 1], 100.0, "invertedhammer should trigger");
}

#[test]
fn cdl_invertedhammer_reject() {
    let mut o = Vec::new();
    let mut h = Vec::new();
    let mut l = Vec::new();
    let mut c = Vec::new();
    seed(&mut o, &mut h, &mut l, &mut c, 20);
    // no gap down (opens above prior close) -> fails
    o.push(101.0);
    h.push(116.0);
    l.push(100.5);
    c.push(100.8);
    let out = cdl_invertedhammer(&o, &h, &l, &c).unwrap();
    assert_eq!(out[out.len() - 1], 0.0, "invertedhammer should not trigger");
}

#[test]
fn cdl_kicking_trigger() {
    let mut o = Vec::new();
    let mut h = Vec::new();
    let mut l = Vec::new();
    let mut c = Vec::new();
    seed(&mut o, &mut h, &mut l, &mut c, 20);
    // black marubozu
    o.push(100.0);
    h.push(100.3);
    l.push(90.0);
    c.push(90.0);
    // white marubozu gapping up
    o.push(101.0);
    h.push(112.0);
    l.push(101.0);
    c.push(112.0);
    let out = cdl_kicking(&o, &h, &l, &c).unwrap();
    assert_eq!(out[out.len() - 1], 100.0, "kicking should trigger bullish");
}

#[test]
fn cdl_kicking_reject() {
    let mut o = Vec::new();
    let mut h = Vec::new();
    let mut l = Vec::new();
    let mut c = Vec::new();
    seed(&mut o, &mut h, &mut l, &mut c, 20);
    o.push(100.0);
    h.push(100.3);
    l.push(90.0);
    c.push(90.0);
    // white candle but opens WITHIN prior body (no gap) -> fails
    o.push(95.0);
    h.push(105.0);
    l.push(95.0);
    c.push(105.0);
    let out = cdl_kicking(&o, &h, &l, &c).unwrap();
    assert_eq!(out[out.len() - 1], 0.0, "kicking should not trigger");
}

#[test]
fn cdl_kickingbylength_trigger() {
    let mut o = Vec::new();
    let mut h = Vec::new();
    let mut l = Vec::new();
    let mut c = Vec::new();
    seed(&mut o, &mut h, &mut l, &mut c, 20);
    // black marubozu (shorter body)
    o.push(100.0);
    h.push(100.3);
    l.push(95.0);
    c.push(95.0);
    // white marubozu (longer body) gapping up
    o.push(101.0);
    h.push(112.0);
    l.push(101.0);
    c.push(112.0);
    let out = cdl_kickingbylength(&o, &h, &l, &c).unwrap();
    assert_eq!(out[out.len() - 1], 100.0, "kickingbylength bullish by longer body");
}

#[test]
fn cdl_kickingbylength_reject() {
    let mut o = Vec::new();
    let mut h = Vec::new();
    let mut l = Vec::new();
    let mut c = Vec::new();
    seed(&mut o, &mut h, &mut l, &mut c, 20);
    o.push(100.0);
    h.push(100.3);
    l.push(95.0);
    c.push(95.0);
    // white candle but no gap -> fails
    o.push(98.0);
    h.push(113.0);
    l.push(98.0);
    c.push(113.0);
    let out = cdl_kickingbylength(&o, &h, &l, &c).unwrap();
    assert_eq!(out[out.len() - 1], 0.0, "kickingbylength should not trigger");
}

#[test]
fn cdl_ladderbottom_trigger() {
    let mut o = Vec::new();
    let mut h = Vec::new();
    let mut l = Vec::new();
    let mut c = Vec::new();
    seed(&mut o, &mut h, &mut l, &mut c, 20);
    // 3 declining black
    o.push(100.0);
    h.push(101.0);
    l.push(99.0);
    c.push(98.0);
    o.push(98.0);
    h.push(99.0);
    l.push(97.0);
    c.push(96.0);
    o.push(96.0);
    h.push(97.0);
    l.push(95.0);
    c.push(94.0);
    // 4th black with upper shadow
    o.push(94.0);
    h.push(98.0);
    l.push(93.5);
    c.push(93.6);
    // 5th white, opens above prior body, closes above prior high
    o.push(94.5);
    h.push(99.0);
    l.push(93.8);
    c.push(98.5);
    let out = cdl_ladderbottom(&o, &h, &l, &c).unwrap();
    assert_eq!(out[out.len() - 1], 100.0, "ladderbottom should trigger");
}

#[test]
fn cdl_ladderbottom_reject() {
    let mut o = Vec::new();
    let mut h = Vec::new();
    let mut l = Vec::new();
    let mut c = Vec::new();
    seed(&mut o, &mut h, &mut l, &mut c, 20);
    o.push(100.0);
    h.push(101.0);
    l.push(99.0);
    c.push(98.0);
    o.push(98.0);
    h.push(99.0);
    l.push(97.0);
    c.push(96.0);
    o.push(96.0);
    h.push(97.0);
    l.push(95.0);
    c.push(94.0);
    o.push(94.0);
    h.push(98.0);
    l.push(93.5);
    c.push(93.6);
    // 5th white but closes BELOW prior high -> fails
    o.push(94.0);
    h.push(97.5);
    l.push(93.8);
    c.push(97.0);
    let out = cdl_ladderbottom(&o, &h, &l, &c).unwrap();
    assert_eq!(out[out.len() - 1], 0.0, "ladderbottom should not trigger");
}
