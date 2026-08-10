//! 统计函数黄金向量比对测试（见 ADR 0003 / ADR 0005）。
//!
//! Statistic-function golden-vector comparison tests (ADR 0003 / ADR 0005).
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
