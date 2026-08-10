//! 波动率指标黄金向量比对测试（见 ADR 0003 / ADR 0005）。
//!
//! Volatility-indicator golden-vector comparison tests (ADR 0003 / ADR 0005).
//!
//! fixture 均为对照已安装 `talib` 0.7.1（Cython 绑定 `libta-lib.0.7.1.dylib`）生成的权威黄金向量，
//! 此处比对即等价于与原版逐项 1:1 校验。全部用例已通过（见 `tools/README.md`）。
//!
//! Fixtures are authoritative golden vectors generated from the installed `talib` 0.7.1
//! (Cython binding over `libta-lib.0.7.1.dylib`); these comparisons are 1:1 checks against
//! the original. All cases pass (see `tools/README.md`).

#[path = "common/mod.rs"]
mod common;

use adaq_talib::utils::approx_eq_slice;
use adaq_talib::volatility::*;

#[test]
fn trange_matches_golden_vector() {
    let json = common::load_json("trange_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(&trange(&high, &low, &close).unwrap(), &expected));
}

#[test]
fn atr_matches_golden_vector() {
    let json = common::load_json("atr_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &atr_default(&high, &low, &close).unwrap(),
        &expected
    ));
}

#[test]
fn natr_matches_golden_vector() {
    let json = common::load_json("natr_basic.json").unwrap();
    let high = common::load_f64_array(&json, "high").unwrap();
    let low = common::load_f64_array(&json, "low").unwrap();
    let close = common::load_f64_array(&json, "close").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(
        &natr_default(&high, &low, &close).unwrap(),
        &expected
    ));
}
