//! 统计函数黄金向量比对测试（见 ADR 0003 / ADR 0005）。
//!
//! Statistic-function golden-vector comparison tests (ADR 0003 / ADR 0005).
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

use adaq_talib::stat::*;
use adaq_talib::utils::approx_eq_slice;

#[test]
fn stddev_matches_golden_vector() {
    let (input, expected) = common::load_fixture("stddev_basic.json").unwrap();
    assert!(approx_eq_slice(&stddev(&input, 5, 1.0).unwrap(), &expected));
}

#[test]
fn var_matches_golden_vector() {
    let (input, expected) = common::load_fixture("var_basic.json").unwrap();
    assert!(approx_eq_slice(&var(&input, 5, 1.0).unwrap(), &expected));
}

#[test]
fn linear_reg_matches_golden_vector() {
    let (input, expected) = common::load_fixture("linear_reg_basic.json").unwrap();
    assert!(approx_eq_slice(&linear_reg(&input, 14).unwrap(), &expected));
}

#[test]
fn linear_reg_angle_matches_golden_vector() {
    let (input, expected) = common::load_fixture("linear_reg_angle_basic.json").unwrap();
    assert!(approx_eq_slice(
        &linear_reg_angle(&input, 14).unwrap(),
        &expected
    ));
}

#[test]
fn linear_reg_intercept_matches_golden_vector() {
    let (input, expected) = common::load_fixture("linear_reg_intercept_basic.json").unwrap();
    assert!(approx_eq_slice(
        &linear_reg_intercept(&input, 14).unwrap(),
        &expected
    ));
}

#[test]
fn linear_reg_slope_matches_golden_vector() {
    let (input, expected) = common::load_fixture("linear_reg_slope_basic.json").unwrap();
    assert!(approx_eq_slice(
        &linear_reg_slope(&input, 14).unwrap(),
        &expected
    ));
}

#[test]
fn tsf_matches_golden_vector() {
    let (input, expected) = common::load_fixture("tsf_basic.json").unwrap();
    assert!(approx_eq_slice(&tsf(&input, 14).unwrap(), &expected));
}

#[test]
fn beta_matches_golden_vector() {
    let json = common::load_json("beta_basic.json").unwrap();
    let real0 = common::load_f64_array(&json, "real0").unwrap();
    let real1 = common::load_f64_array(&json, "real1").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(&beta(&real0, &real1, 5).unwrap(), &expected));
}

#[test]
fn correl_matches_golden_vector() {
    let json = common::load_json("correl_basic.json").unwrap();
    let real0 = common::load_f64_array(&json, "real0").unwrap();
    let real1 = common::load_f64_array(&json, "real1").unwrap();
    let expected = common::load_f64_array(&json, "expected").unwrap();
    assert!(approx_eq_slice(&correl(&real0, &real1, 5).unwrap(), &expected));
}
