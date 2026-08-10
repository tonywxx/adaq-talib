//! 价格变换黄金向量比对测试（见 ADR 0003 / ADR 0005）。
//!
//! Price-transform golden-vector comparison tests (ADR 0003 / ADR 0005).
//!
//! 注意：fixture 为 `tools/gen_fixtures/generate.py` 基于 **TA-Lib C 0.7.1** 真实输出生成的
//! 权威黄金向量（2026-08-10 重生成，不再携带 `_note` 字段；见 ADR 0003 / `tools/README.md`）。
//! 此处比对即等价于与原版逐项 **1:1** 校验。
//!
//! NOTE: fixtures are authoritative golden vectors generated from real TA-Lib C 0.7.1
//! output via `tools/gen_fixtures/generate.py` (regenerated 2026-08-10, no `_note` field;
//! see ADR 0003 / `tools/README.md`). These comparisons are 1:1 checks against the original.

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

#[test]
fn avgdev_matches_golden_vector() {
    let (input, expected) = common::load_fixture("avgdev_basic.json").expect("load");
    let out = avgdev(&input, 14).expect("avgdev");
    assert!(approx_eq_slice(&out, &expected), "AVGDEV deviates");
}
