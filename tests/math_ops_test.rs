//! 数学运算符黄金向量比对测试（见 ADR 0003 / ADR 0005）。
//!
//! Math Operators golden-vector comparison tests (ADR 0003 / ADR 0005).
//!
//! 注意：fixture 为 `tools/gen_fixtures/generate.py` 基于 **TA-Lib C 0.7.1** 真实输出生成的
//! 权威黄金向量。此处比对即等价于与原版逐项 **1:1** 校验。
//!
//! NOTE: fixtures are authoritative golden vectors generated from real TA-Lib C 0.7.1 output
//! via `tools/gen_fixtures/generate.py`. These comparisons are 1:1 checks against the original.

#[path = "common/mod.rs"]
mod common;

use adaq_talib::math_ops::{
    add, div, max, max_index, min, min_index, minmax, minmax_index, mult, sub, sum,
};
use adaq_talib::utils::approx_eq_slice;

#[test]
fn add_matches_golden_vector() {
    let json = common::load_json("add_basic.json").expect("load");
    let a = common::load_f64_array(&json, "real0").expect("real0");
    let b = common::load_f64_array(&json, "real1").expect("real1");
    let expected = common::load_f64_array(&json, "expected").expect("expected");
    let out = add(&a, &b).expect("add");
    assert!(approx_eq_slice(&out, &expected), "ADD deviates");
}

#[test]
fn sub_matches_golden_vector() {
    let json = common::load_json("sub_basic.json").expect("load");
    let a = common::load_f64_array(&json, "real0").expect("real0");
    let b = common::load_f64_array(&json, "real1").expect("real1");
    let expected = common::load_f64_array(&json, "expected").expect("expected");
    let out = sub(&a, &b).expect("sub");
    assert!(approx_eq_slice(&out, &expected), "SUB deviates");
}

#[test]
fn mult_matches_golden_vector() {
    let json = common::load_json("mult_basic.json").expect("load");
    let a = common::load_f64_array(&json, "real0").expect("real0");
    let b = common::load_f64_array(&json, "real1").expect("real1");
    let expected = common::load_f64_array(&json, "expected").expect("expected");
    let out = mult(&a, &b).expect("mult");
    assert!(approx_eq_slice(&out, &expected), "MULT deviates");
}

#[test]
fn div_matches_golden_vector() {
    let json = common::load_json("div_basic.json").expect("load");
    let a = common::load_f64_array(&json, "real0").expect("real0");
    let b = common::load_f64_array(&json, "real1").expect("real1");
    let expected = common::load_f64_array(&json, "expected").expect("expected");
    let out = div(&a, &b).expect("div");
    assert!(approx_eq_slice(&out, &expected), "DIV deviates");
}

#[test]
fn max_matches_golden_vector() {
    let (input, expected) = common::load_fixture("max_basic.json").expect("load");
    let out = max(&input, 5).expect("max");
    assert!(approx_eq_slice(&out, &expected), "MAX deviates");
}

#[test]
fn min_matches_golden_vector() {
    let (input, expected) = common::load_fixture("min_basic.json").expect("load");
    let out = min(&input, 5).expect("min");
    assert!(approx_eq_slice(&out, &expected), "MIN deviates");
}

#[test]
fn sum_matches_golden_vector() {
    let (input, expected) = common::load_fixture("sum_basic.json").expect("load");
    let out = sum(&input, 5).expect("sum");
    assert!(approx_eq_slice(&out, &expected), "SUM deviates");
}

#[test]
fn max_index_matches_golden_vector() {
    let (input, expected) = common::load_fixture("max_index_basic.json").expect("load");
    let out = max_index(&input, 5).expect("max_index");
    assert!(approx_eq_slice(&out, &expected), "MAXINDEX deviates");
}

#[test]
fn min_index_matches_golden_vector() {
    let (input, expected) = common::load_fixture("min_index_basic.json").expect("load");
    let out = min_index(&input, 5).expect("min_index");
    assert!(approx_eq_slice(&out, &expected), "MININDEX deviates");
}

#[test]
fn minmax_matches_golden_vector() {
    let json = common::load_json("minmax_basic.json").expect("load");
    let input = common::load_f64_array(&json, "input").expect("input");
    let mn = common::load_f64_array(&json, "min").expect("min");
    let mx = common::load_f64_array(&json, "max").expect("max");
    let out = minmax(&input, 5).expect("minmax");
    assert!(approx_eq_slice(&out.min, &mn), "MINMAX.min deviates");
    assert!(approx_eq_slice(&out.max, &mx), "MINMAX.max deviates");
}

#[test]
fn minmax_index_matches_golden_vector() {
    let json = common::load_json("minmax_index_basic.json").expect("load");
    let input = common::load_f64_array(&json, "input").expect("input");
    let mni = common::load_f64_array(&json, "min_idx").expect("min_idx");
    let mxi = common::load_f64_array(&json, "max_idx").expect("max_idx");
    let out = minmax_index(&input, 5).expect("minmax_index");
    assert!(approx_eq_slice(&out.min_idx, &mni), "MINMAXINDEX.min_idx deviates");
    assert!(approx_eq_slice(&out.max_idx, &mxi), "MINMAXINDEX.max_idx deviates");
}
