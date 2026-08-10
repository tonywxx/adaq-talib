//! 价格变换黄金向量比对测试（见 ADR 0003 / ADR 0005）。
//!
//! Price-transform golden-vector comparison tests (ADR 0003 / ADR 0005).
//!
//! 注意：当前 fixture 为对照 TA-Lib 0.7.1 文档算法的参考值（见各 fixture 内 `_note`），
//! 待用户本机以 `tools/gen_fixtures/generate.py`（需 TA-Lib C）重新生成权威基准后，
//! 此处比对即等价于与原版逐项校验。
//!
//! NOTE: fixtures currently hold reference values derived from the TA-Lib 0.7.1 documented
//! algorithm (see each fixture's `_note`). Once regenerated authoritatively via
//! `tools/gen_fixtures/generate.py` (requires TA-Lib C), these become 1:1 checks.

#[path = "common/mod.rs"]
mod common;

use adaq_talib::price_transform::*;
use adaq_talib::utils::approx_eq_slice;

#[test]
fn avgprice_matches_golden_vector() {
    let json = common::load_json("avgprice_basic.json").unwrap();
    let open = common::load_f64_array(&json, "open").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &avgprice(&high, &low, &close, &open).unwrap(),
        &expected
    ));
}

#[test]
fn medprice_matches_golden_vector() {
    let json = common::load_json("medprice_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(&medprice(&high, &low).unwrap(), &expected));
}

#[test]
fn typprice_matches_golden_vector() {
    let json = common::load_json("typprice_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &typprice(&high, &low, &close).unwrap(),
        &expected
    ));
}

#[test]
fn wclprice_matches_golden_vector() {
    let json = common::load_json("wclprice_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &wclprice(&high, &low, &close).unwrap(),
        &expected
    ));
}
