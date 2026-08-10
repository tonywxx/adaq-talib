//! 成交量指标黄金向量比对测试（见 ADR 0003 / ADR 0005）。
//!
//! Volume-indicator golden-vector comparison tests (ADR 0003 / ADR 0005).
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

use adaq_talib::utils::approx_eq_slice;
use adaq_talib::volume::*;

#[test]
fn ad_matches_golden_vector() {
    let json = common::load_json("ad_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let volume = common::load_f64_array(&json, "volume").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &ad(&high, &low, &close, &volume).unwrap(),
        &expected
    ));
}

#[test]
fn adosc_matches_golden_vector() {
    let json = common::load_json("adosc_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let volume = common::load_f64_array(&json, "volume").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &adosc_default(&high, &low, &close, &volume).unwrap(),
        &expected
    ));
}

#[test]
fn obv_matches_golden_vector() {
    let json = common::load_json("obv_basic.json").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let volume = common::load_f64_array(&json, "volume").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(&obv(&close, &volume).unwrap(), &expected));
}
