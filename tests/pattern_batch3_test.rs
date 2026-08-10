//! 形态识别（Pattern Recognition）黄金向量比对测试，第 3 批。
//!
//! Golden-vector comparison tests for Pattern Recognition, batch 3.
//! See ADR 0003 / ADR 0005.

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
fn cdl_belthold_matches_golden_vector() {
    check_cdl("belthold", cdl_belthold);
}

#[test]
fn cdl_breakaway_matches_golden_vector() {
    check_cdl("breakaway", cdl_breakaway);
}

#[test]
fn cdl_closingmarubozu_matches_golden_vector() {
    check_cdl("closingmarubozu", cdl_closingmarubozu);
}

#[test]
fn cdl_concealbabyswall_matches_golden_vector() {
    check_cdl("concealbabyswall", cdl_concealbabyswall);
}

#[test]
fn cdl_counterattack_matches_golden_vector() {
    check_cdl("counterattack", cdl_counterattack);
}

#[test]
fn cdl_darkcloudcover_matches_golden_vector() {
    check_cdl("darkcloudcover", cdl_darkcloudcover);
}

#[test]
fn cdl_dojistar_matches_golden_vector() {
    check_cdl("dojistar", cdl_dojistar);
}

#[test]
fn cdl_dragonflydoji_matches_golden_vector() {
    check_cdl("dragonflydoji", cdl_dragonflydoji);
}

// ---------------------------------------------------------------------------
// 手工构造触发 / 拒绝（当随机 fixture 无信号时校验逻辑）
// Hand-built trigger / reject cases
// ---------------------------------------------------------------------------

// 基线：10 根小实体近十字 K 线（高-低 ≈ 0.4，实体 ≈ 0.1），用于喂蜡烛均值窗口。
// Baseline: 10 small near-doji candles (high-low ≈ 0.4, body ≈ 0.1) for the avg window.
fn baseline_open() -> Vec<f64> {
    vec![10.0; 10]
}
fn baseline_high() -> Vec<f64> {
    vec![10.2; 10]
}
fn baseline_low() -> Vec<f64> {
    vec![9.8; 10]
}
fn baseline_close() -> Vec<f64> {
    vec![10.1; 10]
}

#[test]
fn cdl_belthold_trigger_and_reject() {
    // 触发：长阳线、无下影线 → +100
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(10.0);
    h.push(20.1);
    l.push(10.0); // lower shadow = 0
    c.push(20.0);
    let out = cdl_belthold(&o, &h, &l, &c).unwrap();
    assert_eq!(out[10], 100.0, "bullish belt-hold should be +100");

    // 拒绝：长阳线但有长下影线 → 0
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(10.0);
    h.push(20.1);
    l.push(5.0); // long lower shadow
    c.push(20.0);
    let out = cdl_belthold(&o, &h, &l, &c).unwrap();
    assert_eq!(out[10], 0.0, "belt-hold with long lower shadow should be 0");
}

#[test]
fn cdl_closingmarubozu_trigger_and_reject() {
    // 触发：长阳线、无上影线 → +100
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(10.0);
    h.push(20.03); // upper shadow = 0.03 < avg threshold
    l.push(9.9);
    c.push(20.0);
    let out = cdl_closingmarubozu(&o, &h, &l, &c).unwrap();
    assert_eq!(out[10], 100.0, "bullish closing marubozu should be +100");

    // 拒绝：长阳线但有长上影线 → 0
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(10.0);
    h.push(20.5); // long upper shadow
    l.push(9.9);
    c.push(20.0);
    let out = cdl_closingmarubozu(&o, &h, &l, &c).unwrap();
    assert_eq!(out[10], 0.0, "closing marubozu with long upper shadow should be 0");
}

#[test]
fn cdl_dragonflydoji_trigger_and_reject() {
    // 触发：极小实体、无上影、长下影 → +100
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(10.0);
    h.push(10.03); // upper shadow = 0.03
    l.push(5.0); // long lower shadow
    c.push(10.0);
    let out = cdl_dragonflydoji(&o, &h, &l, &c).unwrap();
    assert_eq!(out[10], 100.0, "dragonfly doji should be +100");

    // 拒绝：下影也极短（普通十字）→ 0
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(10.0);
    h.push(10.03);
    l.push(9.97); // short lower shadow
    c.push(10.0);
    let out = cdl_dragonflydoji(&o, &h, &l, &c).unwrap();
    assert_eq!(out[10], 0.0, "doji without long lower shadow should be 0");
}

#[test]
fn cdl_darkcloudcover_trigger_and_reject() {
    // 触发：长阳线后高开低收黑线 → -100
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(10.0);
    h.push(20.1);
    l.push(9.9);
    c.push(20.0); // 1st long white
    o.push(21.0);
    h.push(21.1);
    l.push(11.9);
    c.push(12.0); // 2nd black, open>20.1, close in (10,15)
    let out = cdl_darkcloudcover(&o, &h, &l, &c).unwrap();
    assert_eq!(out[11], -100.0, "dark cloud cover should be -100");

    // 拒绝：第 2 根收盘未深入实体 → 0
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(10.0);
    h.push(20.1);
    l.push(9.9);
    c.push(20.0);
    o.push(21.0);
    h.push(21.1);
    l.push(17.9);
    c.push(18.0); // close not < 15
    let out = cdl_darkcloudcover(&o, &h, &l, &c).unwrap();
    assert_eq!(out[11], 0.0, "dark cloud cover without penetration should be 0");
}

#[test]
fn cdl_dojistar_trigger_and_reject() {
    // 触发：长阳线 + 向上跳空十字星 → -100（看跌）
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(10.0);
    h.push(20.1);
    l.push(9.9);
    c.push(20.0); // 1st long white
    o.push(25.0);
    h.push(25.1);
    l.push(24.9);
    c.push(25.05); // doji gapping up
    let out = cdl_dojistar(&o, &h, &l, &c).unwrap();
    assert_eq!(out[11], -100.0, "bearish doji star should be -100");

    // 拒绝：十字星未跳空 → 0
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(10.0);
    h.push(20.1);
    l.push(9.9);
    c.push(20.0);
    o.push(15.0);
    h.push(15.1);
    l.push(14.9);
    c.push(15.05); // no gap up
    let out = cdl_dojistar(&o, &h, &l, &c).unwrap();
    assert_eq!(out[11], 0.0, "doji star without gap should be 0");
}

#[test]
fn cdl_counterattack_trigger_and_reject() {
    // 触发：黑线 + 等长收盘白线 → +100
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(12.0);
    h.push(12.1);
    l.push(9.9);
    c.push(10.0); // 1st long black
    o.push(8.0);
    h.push(10.1);
    l.push(7.9);
    c.push(10.0); // 2nd white, equal close
    let out = cdl_counterattack(&o, &h, &l, &c).unwrap();
    assert_eq!(out[11], 100.0, "bullish counterattack should be +100");

    // 拒绝：收盘价不等 → 0
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(12.0);
    h.push(12.1);
    l.push(9.9);
    c.push(10.0);
    o.push(8.0);
    h.push(20.1);
    l.push(7.9);
    c.push(20.0); // far from 10
    let out = cdl_counterattack(&o, &h, &l, &c).unwrap();
    assert_eq!(out[11], 0.0, "counterattack with unequal closes should be 0");
}

#[test]
fn cdl_concealbabyswall_trigger_and_reject() {
    // 触发：四根黑线，藏婴吞没 → +100
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(10.0);
    h.push(10.03);
    l.push(7.97);
    c.push(8.0); // 1st black marubozu
    o.push(8.0);
    h.push(8.03);
    l.push(5.97);
    c.push(6.0); // 2nd black marubozu
    o.push(5.5);
    h.push(6.1); // upper shadow extends into prior body
    l.push(4.9);
    c.push(5.0); // 3rd black, gaps down, has upper shadow
    o.push(5.2);
    h.push(6.5);
    l.push(4.5);
    c.push(4.8); // 4th black engulfing 3rd
    let out = cdl_concealbabyswall(&o, &h, &l, &c).unwrap();
    assert_eq!(out[13], 100.0, "concealing baby swallow should be +100");

    // 拒绝：第 4 根未吞没第 3 根 → 0
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(10.0);
    h.push(10.03);
    l.push(7.97);
    c.push(8.0);
    o.push(8.0);
    h.push(8.03);
    l.push(5.97);
    c.push(6.0);
    o.push(5.5);
    h.push(6.1);
    l.push(4.9);
    c.push(5.0);
    o.push(5.2);
    h.push(6.0); // high not > 6.1
    l.push(4.5);
    c.push(4.8);
    let out = cdl_concealbabyswall(&o, &h, &l, &c).unwrap();
    assert_eq!(out[13], 0.0, "concealing baby swallow without engulf should be 0");
}

#[test]
fn cdl_breakaway_trigger_and_reject() {
    // 触发：五根 K 线脱离形态（看跌末根）→ -100
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(10.0);
    h.push(12.1);
    l.push(9.9);
    c.push(12.0); // 1st white long
    o.push(13.0);
    h.push(14.1);
    l.push(12.9);
    c.push(14.0); // 2nd white gap up
    o.push(13.5);
    h.push(15.1);
    l.push(13.4);
    c.push(15.0); // 3rd white higher
    o.push(14.5);
    h.push(16.1);
    l.push(14.4);
    c.push(16.0); // 4th white higher
    o.push(15.0);
    h.push(15.1);
    l.push(12.4);
    c.push(12.5); // 5th black closes inside gap
    let out = cdl_breakaway(&o, &h, &l, &c).unwrap();
    assert_eq!(out[14], -100.0, "bearish breakaway should be -100");

    // 拒绝：末根未回填缺口 → 0
    let mut o = baseline_open();
    let mut h = baseline_high();
    let mut l = baseline_low();
    let mut c = baseline_close();
    o.push(10.0);
    h.push(12.1);
    l.push(9.9);
    c.push(12.0);
    o.push(13.0);
    h.push(14.1);
    l.push(12.9);
    c.push(14.0);
    o.push(13.5);
    h.push(15.1);
    l.push(13.4);
    c.push(15.0);
    o.push(14.5);
    h.push(16.1);
    l.push(14.4);
    c.push(16.0);
    o.push(15.0);
    h.push(15.1);
    l.push(11.4);
    c.push(11.0); // close 11 < close[10]=12, breaks inside-gap
    let out = cdl_breakaway(&o, &h, &l, &c).unwrap();
    assert_eq!(out[14], 0.0, "breakaway without gap fill should be 0");
}
