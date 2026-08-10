//! 形态识别（Pattern Recognition）黄金向量比对测试，第 6 批。见 ADR 0003 / ADR 0005。
//!
//! Golden-vector comparison tests for Pattern Recognition, batch 6. See ADR 0003 / ADR 0005.

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

#[test]
fn cdl_longleggeddoji_matches_golden_vector() {
    check_cdl("longleggeddoji", cdl_longleggeddoji);
}

#[test]
fn cdl_longleggeddoji_trigger() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // doji 实体极小 + 一根极长影线
    o.push(10.0);
    h.push(30.0);
    l.push(5.0);
    c.push(10.02);
    let out = cdl_longleggeddoji(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_longleggeddoji_reject() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // 实体极小但影线也极短 -> 不满足长影线
    o.push(10.0);
    h.push(10.04);
    l.push(9.98);
    c.push(10.02);
    let out = cdl_longleggeddoji(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

#[test]
fn cdl_longline_matches_golden_vector() {
    check_cdl("longline", cdl_longline);
}

#[test]
fn cdl_longline_trigger() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // 长阳线 + 极短影线
    o.push(10.0);
    h.push(21.0);
    l.push(9.0);
    c.push(20.0);
    let out = cdl_longline(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_longline_reject() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // 长实体但上影线过长
    o.push(10.0);
    h.push(25.0);
    l.push(9.5);
    c.push(20.0);
    let out = cdl_longline(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

#[test]
fn cdl_matchinglow_matches_golden_vector() {
    check_cdl("matchinglow", cdl_matchinglow);
}

#[test]
fn cdl_matchinglow_trigger() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // 两根连续阴线，收盘相等
    o.push(10.0);
    h.push(11.0);
    l.push(8.0);
    c.push(9.0);
    o.push(10.0);
    h.push(11.0);
    l.push(8.0);
    c.push(9.0);
    let out = cdl_matchinglow(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_matchinglow_reject() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // 两根连续阴线，收盘不相等
    o.push(10.0);
    h.push(11.0);
    l.push(8.0);
    c.push(9.0);
    o.push(10.0);
    h.push(11.0);
    l.push(8.0);
    c.push(8.0);
    let out = cdl_matchinglow(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

#[test]
fn cdl_mathold_matches_golden_vector() {
    check_cdl("mathold", cdl_mathold);
}

#[test]
fn cdl_mathold_trigger() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // i-4: 长阳线
    o.push(10.0); h.push(21.0); l.push(9.0); c.push(20.0);
    // i-3: 向上跳空的小阴线
    o.push(20.5); h.push(21.0); l.push(20.0); c.push(20.3);
    // i-2: 回落小实体，被第1根实体包裹
    o.push(20.2); h.push(20.3); l.push(19.8); c.push(19.95);
    // i-1: 继续回落小实体
    o.push(20.0); h.push(20.1); l.push(19.7); c.push(19.8);
    // i: 阳线，高开并收在回调日最高价之上
    o.push(20.0); h.push(22.0); l.push(19.9); c.push(22.0);
    let out = cdl_mathold(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_mathold_reject() {
    let (mut o, mut h, mut l, mut c) = base(14);
    // 同样结构但第2根不再向上跳空 -> 失败
    o.push(10.0); h.push(21.0); l.push(9.0); c.push(20.0);
    o.push(19.9); h.push(20.1); l.push(19.8); c.push(20.3); // 不跳空
    o.push(20.2); h.push(20.3); l.push(19.8); c.push(19.95);
    o.push(20.0); h.push(20.1); l.push(19.7); c.push(19.8);
    o.push(20.0); h.push(22.0); l.push(19.9); c.push(22.0);
    let out = cdl_mathold(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

#[test]
fn cdl_morningdojistar_matches_golden_vector() {
    check_cdl("morningdojistar", cdl_morningdojistar);
}

#[test]
fn cdl_morningdojistar_trigger() {
    let (mut o, mut h, mut l, mut c) = base(12);
    // i-2: 长阴线
    o.push(10.0); h.push(11.0); l.push(4.0); c.push(4.0);
    // i-1: 向下跳空的十字星
    o.push(3.5); h.push(3.6); l.push(3.3); c.push(3.4);
    // i: 阳线深入第1根实体
    o.push(4.0); h.push(7.0); l.push(3.0); c.push(6.0);
    let out = cdl_morningdojistar(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_morningdojistar_reject() {
    let (mut o, mut h, mut l, mut c) = base(12);
    // 第3根未深入第1根实体
    o.push(10.0); h.push(11.0); l.push(4.0); c.push(4.0);
    o.push(3.5); h.push(3.6); l.push(3.3); c.push(3.4);
    o.push(4.0); h.push(5.2); l.push(3.0); c.push(5.0); // 5.0 <= 4+5*0.3=5.5 失败
    let out = cdl_morningdojistar(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

#[test]
fn cdl_morningstar_matches_golden_vector() {
    check_cdl("morningstar", cdl_morningstar);
}

#[test]
fn cdl_morningstar_trigger() {
    let (mut o, mut h, mut l, mut c) = base(12);
    // i-2: 长阴线
    o.push(10.0); h.push(11.0); l.push(4.0); c.push(4.0);
    // i-1: 向下跳空的小实体星线
    o.push(3.5); h.push(3.6); l.push(3.2); c.push(3.3);
    // i: 阳线深入第1根实体
    o.push(4.0); h.push(7.0); l.push(3.0); c.push(6.0);
    let out = cdl_morningstar(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(100.0));
}

#[test]
fn cdl_morningstar_reject() {
    let (mut o, mut h, mut l, mut c) = base(12);
    // 第3根未深入第1根实体
    o.push(10.0); h.push(11.0); l.push(4.0); c.push(4.0);
    o.push(3.5); h.push(3.6); l.push(3.2); c.push(3.3);
    o.push(4.0); h.push(5.2); l.push(3.0); c.push(5.0); // 5.0 <= 5.5 失败
    let out = cdl_morningstar(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}

#[test]
fn cdl_onneck_matches_golden_vector() {
    check_cdl("onneck", cdl_onneck);
}

#[test]
fn cdl_onneck_trigger() {
    let (mut o, mut h, mut l, mut c) = base(11);
    // i-1: 长阴线
    o.push(10.0); h.push(11.0); l.push(4.0); c.push(5.0);
    // i: 阳线，开盘低于前低、收盘≈前低
    o.push(3.0); h.push(4.5); l.push(2.8); c.push(4.0);
    let out = cdl_onneck(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(-100.0));
}

#[test]
fn cdl_onneck_reject() {
    let (mut o, mut h, mut l, mut c) = base(11);
    // i-1: 长阴线
    o.push(10.0); h.push(11.0); l.push(4.0); c.push(5.0);
    // 第2根开盘不低于前低 -> 失败
    o.push(5.0); h.push(5.5); l.push(4.8); c.push(5.0);
    let out = cdl_onneck(&o, &h, &l, &c).unwrap();
    assert_eq!(out.last().copied(), Some(0.0));
}
