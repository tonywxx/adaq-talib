//! 动量指标黄金向量比对测试（见 ADR 0003 / ADR 0005）。
//!
//! Momentum-indicator golden-vector comparison tests (ADR 0003 / ADR 0005).
//!
//! 当前 fixture 已由 `tools/gen_fixtures/generate.py`（TA-Lib C 0.7.1，对照已安装 `talib` 0.7.1
//! 校验）生成为权威黄金向量，此处比对即等价于与原版逐项 1:1 校验。全部动量用例均已通过
//! （见 `tools/README.md`）。
//!
//! Fixtures are authoritative TA-Lib C 0.7.1 golden vectors (cross-checked against the installed
//! `talib` 0.7.1). All momentum cases now pass (see `tools/README.md`).

#[path = "common/mod.rs"]
mod common;

use adaq_talib::momentum::*;
use adaq_talib::utils::approx_eq_slice;

#[test]
fn mom_matches_golden_vector() {
    let (input, expected) = common::load_fixture("mom_basic.json").unwrap();
    assert!(approx_eq_slice(&mom(&input, 10).unwrap(), &expected));
}

#[test]
fn roc_matches_golden_vector() {
    let (input, expected) = common::load_fixture("roc_basic.json").unwrap();
    assert!(approx_eq_slice(&roc(&input, 10).unwrap(), &expected));
}

#[test]
fn rocp_matches_golden_vector() {
    let (input, expected) = common::load_fixture("rocp_basic.json").unwrap();
    assert!(approx_eq_slice(&rocp(&input, 10).unwrap(), &expected));
}

#[test]
fn rocr_matches_golden_vector() {
    let (input, expected) = common::load_fixture("rocr_basic.json").unwrap();
    assert!(approx_eq_slice(&rocr(&input, 10).unwrap(), &expected));
}

#[test]
fn rocr100_matches_golden_vector() {
    let (input, expected) = common::load_fixture("rocr100_basic.json").unwrap();
    assert!(approx_eq_slice(&rocr100(&input, 10).unwrap(), &expected));
}

#[test]
fn rsi_matches_golden_vector() {
    let (input, expected) = common::load_fixture("rsi_basic.json").unwrap();
    assert!(approx_eq_slice(&rsi(&input, 14).unwrap(), &expected));
}

#[test]
fn cmo_matches_golden_vector() {
    let (input, expected) = common::load_fixture("cmo_basic.json").unwrap();
    assert!(approx_eq_slice(&cmo(&input, 14).unwrap(), &expected));
}

#[test]
fn trix_matches_golden_vector() {
    let (input, expected) = common::load_fixture("trix_basic.json").unwrap();
    assert!(approx_eq_slice(&trix(&input, 30).unwrap(), &expected));
}

#[test]
fn stoch_rsi_matches_golden_vector() {
    let (input, expected) = common::load_fixture("stoch_rsi_basic.json").unwrap();
    assert!(approx_eq_slice(&stoch_rsi(&input, 14, 14).unwrap(), &expected));
}

#[test]
fn apo_matches_golden_vector() {
    let (input, expected) = common::load_fixture("apo_basic.json").unwrap();
    assert!(approx_eq_slice(&apo(&input, 12, 26).unwrap(), &expected));
}

#[test]
fn ppo_matches_golden_vector() {
    let (input, expected) = common::load_fixture("ppo_basic.json").unwrap();
    assert!(approx_eq_slice(&ppo(&input, 12, 26).unwrap(), &expected));
}

#[test]
fn macd_matches_golden_vector() {
    let json = common::load_json("macd_basic.json").unwrap();
    let input = common::load_f64_array(&json, "input").unwrap();
    let e_macd = common::load_f64_array(&json, "macd").unwrap();
    let e_sig = common::load_f64_array(&json, "signal").unwrap();
    let e_hist = common::load_f64_array(&json, "hist").unwrap();
    let m = macd_default(&input).unwrap();
    assert!(approx_eq_slice(&m.macd, &e_macd));
    assert!(approx_eq_slice(&m.signal, &e_sig));
    assert!(approx_eq_slice(&m.hist, &e_hist));
}

#[test]
fn cci_matches_golden_vector() {
    let json = common::load_json("cci_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(&cci_default(&high, &low, &close).unwrap(), &expected));
}

#[test]
fn mfi_matches_golden_vector() {
    let json = common::load_json("mfi_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let volume = common::load_f64_array(&json, "volume").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &mfi_default(&high, &low, &close, &volume).unwrap(),
        &expected
    ));
}

#[test]
fn willr_matches_golden_vector() {
    let json = common::load_json("willr_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &willr_default(&high, &low, &close).unwrap(),
        &expected
    ));
}

#[test]
fn bop_matches_golden_vector() {
    let json = common::load_json("bop_basic.json").unwrap();
    let open = common::load_f64_array(&json, "open").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &bop(&open, &high, &low, &close).unwrap(),
        &expected
    ));
}

#[test]
fn ultosc_matches_golden_vector() {
    let json = common::load_json("ultosc_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &ultosc_default(&high, &low, &close).unwrap(),
        &expected
    ));
}

#[test]
fn plus_dm_matches_golden_vector() {
    let json = common::load_json("plus_dm_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &plus_dm_default(&high, &low, &close).unwrap(),
        &expected
    ));
}

#[test]
fn minus_dm_matches_golden_vector() {
    let json = common::load_json("minus_dm_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &minus_dm_default(&high, &low, &close).unwrap(),
        &expected
    ));
}

#[test]
fn plus_di_matches_golden_vector() {
    let json = common::load_json("plus_di_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &plus_di_default(&high, &low, &close).unwrap(),
        &expected
    ));
}

#[test]
fn minus_di_matches_golden_vector() {
    let json = common::load_json("minus_di_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &minus_di_default(&high, &low, &close).unwrap(),
        &expected
    ));
}

#[test]
fn adx_matches_golden_vector() {
    let json = common::load_json("adx_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(&adx_default(&high, &low, &close).unwrap(), &expected));
}

#[test]
fn adxr_matches_golden_vector() {
    let json = common::load_json("adxr_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &adxr_default(&high, &low, &close).unwrap(),
        &expected
    ));
}

#[test]
fn aroon_matches_golden_vector() {
    let json = common::load_json("aroon_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let up = common::load_f64_array(&json, "up").unwrap();
    let down = common::load_f64_array(&json, "down").unwrap();
    let a = aroon_default(&high, &low).unwrap();
    assert!(approx_eq_slice(&a.up, &up));
    assert!(approx_eq_slice(&a.down, &down));
}

#[test]
fn aroon_osc_matches_golden_vector() {
    let json = common::load_json("aroon_osc_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &aroon_osc_default(&high, &low).unwrap(),
        &expected
    ));
}

#[test]
fn stoch_matches_golden_vector() {
    let json = common::load_json("stoch_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let slow_k = common::load_f64_array(&json, "slow_k").unwrap();
    let slow_d = common::load_f64_array(&json, "slow_d").unwrap();
    let s = stoch_default(&high, &low, &close).unwrap();
    assert!(approx_eq_slice(&s.slow_k, &slow_k));
    assert!(approx_eq_slice(&s.slow_d, &slow_d));
}

#[test]
fn stoch_f_matches_golden_vector() {
    let json = common::load_json("stoch_f_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let fast_k = common::load_f64_array(&json, "fast_k").unwrap();
    let fast_d = common::load_f64_array(&json, "fast_d").unwrap();
    let s = stoch_f_default(&high, &low, &close).unwrap();
    assert!(approx_eq_slice(&s.fast_k, &fast_k));
    assert!(approx_eq_slice(&s.fast_d, &fast_d));
}

#[test]
fn dx_matches_golden_vector() {
    let json = common::load_json("dx_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(&dx_default(&high, &low, &close).unwrap(), &expected));
}

#[test]
fn imi_matches_golden_vector() {
    let json = common::load_json("imi_basic.json").unwrap();
    let open = common::load_f64_array(&json, "open").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(&imi_default(&open, &close).unwrap(), &expected));
}
