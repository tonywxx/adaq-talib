//! 形态识别（Pattern Recognition）黄金向量比对测试，第 4 批。
//!
//! Golden-vector comparison tests for Pattern Recognition, batch 4.
//! `check_cdl` compares against the installed TA-Lib 0.7.1 dylib fixtures (ADR 0005);
//! each pattern also has hand-built trigger / reject tests so logic is validated even
//! when the random fixture has no signal.

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

// ===========================================================================
// 黄金向量比对 / Golden-vector comparisons
// ===========================================================================

#[test]
fn cdl_eveningdojistar_matches_golden_vector() {
    check_cdl("eveningdojistar", cdl_eveningdojistar);
}

#[test]
fn cdl_eveningstar_matches_golden_vector() {
    check_cdl("eveningstar", cdl_eveningstar);
}

#[test]
fn cdl_gapsidesidewhite_matches_golden_vector() {
    check_cdl("gapsidesidewhite", cdl_gapsidesidewhite);
}

#[test]
fn cdl_gravestonedoji_matches_golden_vector() {
    check_cdl("gravestonedoji", cdl_gravestonedoji);
}

#[test]
fn cdl_hangingman_matches_golden_vector() {
    check_cdl("hangingman", cdl_hangingman);
}

#[test]
fn cdl_haramicross_matches_golden_vector() {
    check_cdl("haramicross", cdl_haramicross);
}

#[test]
fn cdl_hikkake_matches_golden_vector() {
    check_cdl("hikkake", cdl_hikkake);
}

#[test]
fn cdl_hikkakemod_matches_golden_vector() {
    check_cdl("hikkakemod", cdl_hikkakemod);
}

// ===========================================================================
// 手工构造触发 / 拒绝用例 / Hand-built trigger & reject tests
// ===========================================================================

#[test]
fn cdl_eveningdojistar_trigger_and_reject() {
    // 长阳线(12) + 跳空十字星(13) + 深陷实体阴线(14)
    let mut o = vec![10.0; 20];
    let mut h = vec![10.5; 20];
    let mut l = vec![9.5; 20];
    let mut c = vec![10.1; 20];
    o[12] = 10.0; h[12] = 20.0; l[12] = 9.5; c[12] = 20.0;
    o[13] = 20.5; h[13] = 21.0; l[13] = 20.0; c[13] = 20.6;
    o[14] = 20.4; h[14] = 21.0; l[14] = 14.0; c[14] = 15.0;
    let out = cdl_eveningdojistar(&o, &h, &l, &c).unwrap();
    assert_eq!(out[14], -100.0, "eveningdojistar should fire -100 at bar 14");
    assert_eq!(out[0], 0.0, "leading lookback positions stay 0");

    // 拒绝：第三根收盘未深陷第一根实体（> 第一根收盘 - 实体*0.3 = 17）
    c[14] = 19.0;
    let out = cdl_eveningdojistar(&o, &h, &l, &c).unwrap();
    assert_eq!(out[14], 0.0, "should reject when close not well within 1st body");
}

#[test]
fn cdl_eveningstar_trigger_and_reject() {
    let mut o = vec![10.0; 20];
    let mut h = vec![10.5; 20];
    let mut l = vec![9.5; 20];
    let mut c = vec![10.1; 20];
    o[12] = 10.0; h[12] = 20.0; l[12] = 9.5; c[12] = 20.0;
    o[13] = 20.5; h[13] = 21.0; l[13] = 20.0; c[13] = 20.6;
    o[14] = 20.4; h[14] = 21.0; l[14] = 14.0; c[14] = 15.0;
    let out = cdl_eveningstar(&o, &h, &l, &c).unwrap();
    assert_eq!(out[14], -100.0, "eveningstar should fire -100 at bar 14");

    // 拒绝：第三根未深陷实体
    c[14] = 19.0;
    let out = cdl_eveningstar(&o, &h, &l, &c).unwrap();
    assert_eq!(out[14], 0.0, "should reject when close not well within 1st body");
}

#[test]
fn cdl_gapsidesidewhite_trigger_and_reject() {
    let mut o = vec![10.0; 15];
    let mut h = vec![10.5; 15];
    let mut l = vec![9.5; 15];
    let mut c = vec![10.1; 15];
    o[6] = 10.0; h[6] = 11.0; l[6] = 9.5; c[6] = 10.5;
    o[7] = 11.5; h[7] = 12.0; l[7] = 11.2; c[7] = 11.8;
    o[8] = 11.5; h[8] = 12.0; l[8] = 11.2; c[8] = 11.8;
    let out = cdl_gapsidesidewhite(&o, &h, &l, &c).unwrap();
    assert_eq!(out[8], 100.0, "upside gap side-by-side white should fire +100");

    // 拒绝：第三根不为阳线
    c[8] = 11.4;
    let out = cdl_gapsidesidewhite(&o, &h, &l, &c).unwrap();
    assert_eq!(out[8], 0.0, "should reject when 3rd candle is not white");
}

#[test]
fn cdl_gravestonedoji_trigger_and_reject() {
    let mut o = vec![10.0; 14];
    let mut h = vec![10.5; 14];
    let mut l = vec![9.5; 14];
    let mut c = vec![10.1; 14];
    o[10] = 10.0; h[10] = 20.0; l[10] = 10.0; c[10] = 10.0;
    let out = cdl_gravestonedoji(&o, &h, &l, &c).unwrap();
    assert_eq!(out[10], 100.0, "gravestone doji should fire +100");

    // 拒绝：下影线不极短
    l[10] = 5.0;
    let out = cdl_gravestonedoji(&o, &h, &l, &c).unwrap();
    assert_eq!(out[10], 0.0, "should reject when lower shadow is not very short");
}

#[test]
fn cdl_hangingman_trigger_and_reject() {
    let mut o = vec![9.5; 15];
    let mut h = vec![9.9; 15];
    let mut l = vec![9.4; 15];
    let mut c = vec![9.6; 15];
    o[11] = 10.0; h[11] = 10.05; l[11] = 4.0; c[11] = 10.05;
    let out = cdl_hangingman(&o, &h, &l, &c).unwrap();
    assert_eq!(out[11], -100.0, "hanging man should fire -100");

    // 拒绝：上影线不极短
    h[11] = 15.0;
    let out = cdl_hangingman(&o, &h, &l, &c).unwrap();
    assert_eq!(out[11], 0.0, "should reject when upper shadow is not very short");
}

#[test]
fn cdl_haramicross_trigger_and_reject() {
    let mut o = vec![9.5; 14];
    let mut h = vec![9.9; 14];
    let mut l = vec![9.4; 14];
    let mut c = vec![9.6; 14];
    o[11] = 10.0; h[11] = 20.0; l[11] = 9.0; c[11] = 20.0;
    o[12] = 14.0; h[12] = 15.0; l[12] = 13.0; c[12] = 14.1;
    let out = cdl_haramicross(&o, &h, &l, &c).unwrap();
    assert_eq!(out[12], -100.0, "harami cross (white 1st) should fire -100");

    // 拒绝：第二根未被第一根实体包裹
    c[12] = 21.0;
    let out = cdl_haramicross(&o, &h, &l, &c).unwrap();
    assert_eq!(out[12], 0.0, "should reject when 2nd body is not engulfed");
}

#[test]
fn cdl_hikkake_trigger_and_reject() {
    let o = [15.0, 15.0, 15.0, 15.0, 15.0, 15.0, 15.0, 15.0];
    let h = [20.0, 21.0, 22.0, 20.0, 19.0, 18.0, 19.0, 19.0];
    let l = [10.0, 13.0, 14.0, 10.0, 11.0, 9.0, 10.0, 10.0];
    let c = [14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0];
    let out = cdl_hikkake(&o, &h, &l, &c).unwrap();
    assert_eq!(out[5], 100.0, "bullish hikkake should fire +100 at bar 5");

    // 拒绝：第 3 根未形成任一方向的突破（高低均与前一根持平）
    let h2 = [20.0, 21.0, 22.0, 20.0, 19.0, 19.0, 19.0, 19.0];
    let l2 = [10.0, 13.0, 14.0, 10.0, 11.0, 11.0, 10.0, 10.0];
    let out = cdl_hikkake(&o, &h2, &l2, &c).unwrap();
    assert_eq!(out[5], 0.0, "should reject when 3rd breaks the inside-bar breakout");
}

#[test]
fn cdl_hikkakemod_trigger_and_reject() {
    let mut o = vec![10.0; 15];
    let mut h = vec![20.0; 15];
    let mut l = vec![19.0; 15];
    let mut c = vec![10.1; 15];
    o[9] = 10.0; h[9] = 25.0; l[9] = 15.0; c[9] = 10.1;
    o[10] = 10.0; h[10] = 24.0; l[10] = 16.0; c[10] = 16.1;
    o[11] = 10.0; h[11] = 23.0; l[11] = 17.0; c[11] = 10.1;
    o[12] = 10.0; h[12] = 22.0; l[12] = 16.0; c[12] = 10.1;
    let out = cdl_hikkakemod(&o, &h, &l, &c).unwrap();
    assert_eq!(out[12], 100.0, "bullish modified hikkake should fire +100 at bar 12");

    // 拒绝：第 4 根未更低高/更低低
    h[12] = 23.0; l[12] = 17.0;
    let out = cdl_hikkakemod(&o, &h, &l, &c).unwrap();
    assert_eq!(out[12], 0.0, "should reject when 4th breaks the inside-bar breakout");
}
