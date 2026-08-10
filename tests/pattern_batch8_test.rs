//! 形态识别（Pattern Recognition）黄金向量比对测试，第 8 批。见 ADR 0003 / ADR 0005。
//!
//! Golden-vector comparison tests for Pattern Recognition, batch 8. See ADR 0003 / ADR 0005.

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

/// 构造 `n` 根中性基准 K 线（小实体、稳定范围），用于手工触发/拒绝测试前的预热。
/// Build `n` neutral prior bars (small body, stable range) to warm up the candle averages.
fn base(n: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut o = vec![];
    let mut h = vec![];
    let mut l = vec![];
    let mut c = vec![];
    for _ in 0..n {
        o.push(10.0);
        h.push(12.0);
        l.push(8.0);
        c.push(10.3);
    }
    (o, h, l, c)
}

// ----- cdl_sticksandwich -----------------------------------------------------

#[test]
fn cdl_sticksandwich_matches_golden_vector() {
    check_cdl("sticksandwich", cdl_sticksandwich);
}

#[test]
fn cdl_sticksandwich_trigger() {
    let (mut o, mut h, mut l, mut c) = base(10);
    // i-2: black, close 9.5
    o.push(10.0); h.push(11.0); l.push(9.0); c.push(9.5);
    // i-1: white, low > prior close
    o.push(9.6); h.push(11.0); l.push(9.7); c.push(10.5);
    // i: black, close == first close
    o.push(10.5); h.push(11.0); l.push(9.0); c.push(9.5);
    let out = cdl_sticksandwich(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_sticksandwich_reject() {
    let (mut o, mut h, mut l, mut c) = base(10);
    o.push(10.0); h.push(11.0); l.push(9.0); c.push(9.5);
    o.push(9.6); h.push(11.0); l.push(9.7); c.push(10.5);
    // third close far from first close
    o.push(10.5); h.push(11.0); l.push(9.0); c.push(9.0);
    let out = cdl_sticksandwich(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

// ----- cdl_takuri ------------------------------------------------------------

#[test]
fn cdl_takuri_matches_golden_vector() {
    check_cdl("takuri", cdl_takuri);
}

#[test]
fn cdl_takuri_trigger() {
    let (mut o, mut h, mut l, mut c) = base(12);
    // doji at the high + very long lower shadow
    o.push(20.0); h.push(20.0); l.push(5.0); c.push(20.0);
    let out = cdl_takuri(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_takuri_reject() {
    let (mut o, mut h, mut l, mut c) = base(12);
    // not a doji
    o.push(10.0); h.push(15.0); l.push(8.0); c.push(14.0);
    let out = cdl_takuri(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

// ----- cdl_tasukigap ---------------------------------------------------------

#[test]
fn cdl_tasukigap_matches_golden_vector() {
    check_cdl("tasukigap", cdl_tasukigap);
}

#[test]
fn cdl_tasukigap_trigger() {
    let (mut o, mut h, mut l, mut c) = base(10);
    // i-2: white
    o.push(10.0); h.push(12.0); l.push(8.0); c.push(11.0);
    // i-1: white, gapping up
    o.push(12.5); h.push(14.0); l.push(12.0); c.push(13.5);
    // i: black, opens in 2nd rb, closes under 2nd open but inside gap
    o.push(13.0); h.push(13.2); l.push(11.5); c.push(12.0);
    let out = cdl_tasukigap(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_tasukigap_reject() {
    let (mut o, mut h, mut l, mut c) = base(10);
    // no upside gap between 1st and 2nd
    o.push(10.0); h.push(12.0); l.push(8.0); c.push(11.0);
    o.push(10.5); h.push(12.0); l.push(10.0); c.push(11.5);
    o.push(11.0); h.push(11.2); l.push(9.5); c.push(10.5);
    let out = cdl_tasukigap(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

// ----- cdl_thrusting ---------------------------------------------------------

#[test]
fn cdl_thrusting_matches_golden_vector() {
    check_cdl("thrusting", cdl_thrusting);
}

#[test]
fn cdl_thrusting_trigger() {
    let (mut o, mut h, mut l, mut c) = base(12);
    // i-1: long black
    o.push(20.0); h.push(21.0); l.push(10.0); c.push(10.0);
    // i: white, gaps down, closes into prior body under midpoint
    o.push(9.0); h.push(12.5); l.push(8.5); c.push(12.0);
    let out = cdl_thrusting(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(-100.0));
}

#[test]
fn cdl_thrusting_reject() {
    let (mut o, mut h, mut l, mut c) = base(12);
    // i-1: long black
    o.push(20.0); h.push(21.0); l.push(10.0); c.push(10.0);
    // i: white, but closes beyond midpoint -> reject
    o.push(9.0); h.push(16.5); l.push(8.5); c.push(16.0);
    let out = cdl_thrusting(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

// ----- cdl_tristar -----------------------------------------------------------

#[test]
fn cdl_tristar_matches_golden_vector() {
    check_cdl("tristar", cdl_tristar);
}

#[test]
fn cdl_tristar_trigger() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // i-2: doji
    o.push(15.0); h.push(15.2); l.push(14.8); c.push(15.01);
    // i-1: doji, gaps up
    o.push(15.5); h.push(15.7); l.push(15.49); c.push(15.51);
    // i: doji, not higher than 2nd
    o.push(15.3); h.push(15.4); l.push(15.29); c.push(15.31);
    let out = cdl_tristar(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(-100.0));
}

#[test]
fn cdl_tristar_reject() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // i-2: doji
    o.push(15.0); h.push(15.2); l.push(14.8); c.push(15.01);
    // i-1: doji, gaps up
    o.push(15.5); h.push(15.7); l.push(15.49); c.push(15.51);
    // i: not a doji -> reject
    o.push(15.3); h.push(16.5); l.push(15.29); c.push(16.0);
    let out = cdl_tristar(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

// ----- cdl_unique3river ------------------------------------------------------

#[test]
fn cdl_unique3river_matches_golden_vector() {
    check_cdl("unique3river", cdl_unique3river);
}

#[test]
fn cdl_unique3river_trigger() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // i-2: long black
    o.push(20.0); h.push(21.0); l.push(15.0); c.push(10.0);
    // i-1: black harami with lower low
    o.push(18.0); h.push(19.0); l.push(14.0); c.push(11.0);
    // i: small white, open above prior low
    o.push(15.0); h.push(15.3); l.push(14.9); c.push(15.2);
    let out = cdl_unique3river(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_unique3river_reject() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // i-2: long black
    o.push(20.0); h.push(21.0); l.push(15.0); c.push(10.0);
    // i-1: black but NOT harami (open above 1st open) -> reject
    o.push(21.0); h.push(22.0); l.push(14.0); c.push(11.0);
    // i: small white
    o.push(15.0); h.push(15.3); l.push(14.9); c.push(15.2);
    let out = cdl_unique3river(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

// ----- cdl_upsidegap2crows ---------------------------------------------------

#[test]
fn cdl_upsidegap2crows_matches_golden_vector() {
    check_cdl("upsidegap2crows", cdl_upsidegap2crows);
}

#[test]
fn cdl_upsidegap2crows_trigger() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // i-2: long white
    o.push(10.0); h.push(21.0); l.push(9.0); c.push(20.0);
    // i-1: small black, gapping up
    o.push(21.0); h.push(21.5); l.push(20.7); c.push(20.8);
    // i: black, engulfs prior body, closes above 1st
    o.push(22.0); h.push(22.5); l.push(20.4); c.push(20.5);
    let out = cdl_upsidegap2crows(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(-100.0));
}

#[test]
fn cdl_upsidegap2crows_reject() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // i-2: long white
    o.push(10.0); h.push(21.0); l.push(9.0); c.push(20.0);
    // i-1: black but NO gap up -> reject
    o.push(20.5); h.push(21.0); l.push(19.5); c.push(19.5);
    // i: black
    o.push(22.0); h.push(22.5); l.push(20.4); c.push(20.5);
    let out = cdl_upsidegap2crows(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

// ----- cdl_xsidegap3methods --------------------------------------------------

#[test]
fn cdl_xsidegap3methods_matches_golden_vector() {
    check_cdl("xsidegap3methods", cdl_xsidegap3methods);
}

#[test]
fn cdl_xsidegap3methods_trigger() {
    let (mut o, mut h, mut l, mut c) = base(5);
    // i-2: white
    o.push(10.0); h.push(12.0); l.push(9.0); c.push(11.0);
    // i-1: white, upside gap
    o.push(12.0); h.push(13.0); l.push(12.0); c.push(13.0);
    // i: black, opens in 2nd rb, closes in 1st rb
    o.push(12.5); h.push(12.7); l.push(10.4); c.push(10.5);
    let out = cdl_xsidegap3methods(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_xsidegap3methods_reject() {
    let (mut o, mut h, mut l, mut c) = base(5);
    // i-2: white
    o.push(10.0); h.push(12.0); l.push(9.0); c.push(11.0);
    // i-1: white, NO gap
    o.push(9.0); h.push(10.0); l.push(9.0); c.push(10.0);
    // i: black
    o.push(9.5); h.push(9.7); l.push(10.4); c.push(10.5);
    let out = cdl_xsidegap3methods(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}
