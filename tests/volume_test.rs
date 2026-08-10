//! 成交量指标黄金向量比对测试（见 ADR 0003 / ADR 0005）。
//!
//! Volume-indicator golden-vector comparison tests (ADR 0003 / ADR 0005).
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
