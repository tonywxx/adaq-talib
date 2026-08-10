//! 形态识别（Pattern Recognition）黄金向量比对测试，第 7 批。见 ADR 0003 / ADR 0005。
//!
//! Golden-vector comparison tests for Pattern Recognition, batch 7. See ADR 0003 / ADR 0005.

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

// ===========================================================================
// cdl_piercing
// ===========================================================================

#[test]
fn cdl_piercing_matches_golden_vector() {
    check_cdl("piercing", cdl_piercing);
}

#[test]
fn cdl_piercing_trigger() {
    let (mut o, mut h, mut l, mut c) = base(15);
    // i-1: 长阴线
    o.push(20.0); h.push(21.0); l.push(10.0); c.push(11.0);
    // i: 长阳线，开盘低于前低、收盘深入前实体 50% 以上
    o.push(9.0); h.push(16.0); l.push(8.0); c.push(16.0); // 16 > 11 + 9*0.5 = 15.5
    let out = cdl_piercing(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_piercing_reject() {
    let (mut o, mut h, mut l, mut c) = base(15);
    // i-1: 长阴线
    o.push(20.0); h.push(21.0); l.push(10.0); c.push(11.0);
    // i: 阳线收盘未深入前实体 50% 以上
    o.push(9.0); h.push(13.0); l.push(8.0); c.push(14.0); // 14 < 15.5
    let out = cdl_piercing(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

// ===========================================================================
// cdl_rickshawman
// ===========================================================================

#[test]
fn cdl_rickshawman_matches_golden_vector() {
    check_cdl("rickshawman", cdl_rickshawman);
}

#[test]
fn cdl_rickshawman_trigger() {
    let (mut o, mut h, mut l, mut c) = base(10);
    // doji 实体 + 两根长影线 + 实体接近中点
    o.push(10.0); h.push(15.0); l.push(5.0); c.push(10.1);
    let out = cdl_rickshawman(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_rickshawman_reject() {
    let (mut o, mut h, mut l, mut c) = base(10);
    // 长实体（非十字星）
    o.push(10.0); h.push(15.0); l.push(5.0); c.push(14.0);
    let out = cdl_rickshawman(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

// ===========================================================================
// cdl_risefall3methods
// ===========================================================================

#[test]
fn cdl_risefall3methods_matches_golden_vector() {
    check_cdl("risefall3methods", cdl_risefall3methods);
}

#[test]
fn cdl_risefall3methods_trigger() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // i-4: 长阳线
    o.push(10.0); h.push(20.0); l.push(9.0); c.push(19.0);
    // i-3: 小阴线，被第1根包裹
    o.push(19.0); h.push(19.1); l.push(18.7); c.push(18.8);
    // i-2: 小阴线，收盘更低
    o.push(18.8); h.push(18.9); l.push(18.5); c.push(18.6);
    // i-1: 小阴线，收盘更低
    o.push(18.6); h.push(18.7); l.push(18.3); c.push(18.4);
    // i: 长阳线，高开并收在第1根收盘之上
    o.push(18.5); h.push(21.0); l.push(18.2); c.push(20.0); // 20 > 19
    let out = cdl_risefall3methods(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_risefall3methods_reject() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // 同样结构但第5根收盘未超过第1根收盘
    o.push(10.0); h.push(20.0); l.push(9.0); c.push(19.0);
    o.push(19.0); h.push(19.1); l.push(18.7); c.push(18.8);
    o.push(18.8); h.push(18.9); l.push(18.5); c.push(18.6);
    o.push(18.6); h.push(18.7); l.push(18.3); c.push(18.4);
    o.push(18.5); h.push(20.0); l.push(18.2); c.push(18.0); // 18 < 19
    let out = cdl_risefall3methods(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

// ===========================================================================
// cdl_separatinglines
// ===========================================================================

#[test]
fn cdl_separatinglines_matches_golden_vector() {
    check_cdl("separatinglines", cdl_separatinglines);
}

#[test]
fn cdl_separatinglines_trigger() {
    let (mut o, mut h, mut l, mut c) = base(11);
    // i-1: 阴线
    o.push(10.0); h.push(11.0); l.push(4.0); c.push(5.0);
    // i: 阳线 belt-hold，相同开盘价、极短下影线
    o.push(10.0); h.push(20.0); l.push(9.9); c.push(19.0);
    let out = cdl_separatinglines(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_separatinglines_reject() {
    let (mut o, mut h, mut l, mut c) = base(11);
    // i-1: 阴线
    o.push(10.0); h.push(11.0); l.push(4.0); c.push(5.0);
    // i: 开盘价不同 -> 失败
    o.push(11.0); h.push(20.0); l.push(10.9); c.push(19.0);
    let out = cdl_separatinglines(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

// ===========================================================================
// cdl_shortline
// ===========================================================================

#[test]
fn cdl_shortline_matches_golden_vector() {
    check_cdl("shortline", cdl_shortline);
}

#[test]
fn cdl_shortline_trigger() {
    let (mut o, mut h, mut l, mut c) = base(10);
    // 短实体 + 极短上下影线
    o.push(10.0); h.push(10.8); l.push(9.2); c.push(10.1);
    let out = cdl_shortline(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_shortline_reject() {
    let (mut o, mut h, mut l, mut c) = base(10);
    // 长实体 -> 失败
    o.push(10.0); h.push(15.0); l.push(9.0); c.push(15.0);
    let out = cdl_shortline(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

// ===========================================================================
// cdl_spinningtop
// ===========================================================================

#[test]
fn cdl_spinningtop_matches_golden_vector() {
    check_cdl("spinningtop", cdl_spinningtop);
}

#[test]
fn cdl_spinningtop_trigger() {
    let (mut o, mut h, mut l, mut c) = base(10);
    // 小实体 + 影线长于实体
    o.push(10.0); h.push(15.0); l.push(5.0); c.push(10.1);
    let out = cdl_spinningtop(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_spinningtop_reject() {
    let (mut o, mut h, mut l, mut c) = base(10);
    // 长实体（影线不长于实体）
    o.push(10.0); h.push(16.0); l.push(9.0); c.push(15.0);
    let out = cdl_spinningtop(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

// ===========================================================================
// cdl_stalledpattern
// ===========================================================================

#[test]
fn cdl_stalledpattern_matches_golden_vector() {
    check_cdl("stalledpattern", cdl_stalledpattern);
}

#[test]
fn cdl_stalledpattern_trigger() {
    let (mut o, mut h, mut l, mut c) = base(12);
    // i-2: 长阳线
    o.push(10.0); h.push(20.0); l.push(9.0); c.push(19.0);
    // i-1: 长阳线，极短上影线，开盘在第1根实体内
    o.push(14.0); h.push(20.1); l.push(13.5); c.push(20.0);
    // i: 小阳线，骑在第2根肩部
    o.push(19.5); h.push(21.0); l.push(19.0); c.push(20.5);
    let out = cdl_stalledpattern(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(-100.0));
}

#[test]
fn cdl_stalledpattern_reject() {
    let (mut o, mut h, mut l, mut c) = base(12);
    // 同样结构但第3根收盘未创新高
    o.push(10.0); h.push(20.0); l.push(9.0); c.push(19.0);
    o.push(14.0); h.push(20.1); l.push(13.5); c.push(20.0);
    o.push(19.0); h.push(20.5); l.push(18.8); c.push(19.5); // 19.5 < 20
    let out = cdl_stalledpattern(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}
