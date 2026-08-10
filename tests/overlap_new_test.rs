//! 重叠研究（第二批）黄金向量比对测试（BBANDS / TRIMA / T3 / MA / MAVP / KAMA /
//! SAR / SAREXT）。见 ADR 0003 / ADR 0005。
//!
//! Golden-vector comparison tests for the second batch of Overlap Studies
//! (BBANDS / TRIMA / T3 / MA / MAVP / KAMA / SAR / SAREXT). See ADR 0003 / 0005.
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

use adaq_talib::overlap::{bbands, kama, ma, mavp, sar, sarext, t3, trima, MaType};
use adaq_talib::utils::approx_eq_slice;

#[test]
fn bbands_matches_golden_vector() {
    let json = common::load_json("bbands_basic.json").expect("load fixture");
    let input = common::load_f64_array(&json, "input").expect("input");
    let upper = common::load_f64_array(&json, "upper").expect("upper");
    let middle = common::load_f64_array(&json, "middle").expect("middle");
    let lower = common::load_f64_array(&json, "lower").expect("lower");
    let out = bbands(&input, 20, 2.0, 2.0, MaType::Sma).expect("bbands");
    assert!(
        approx_eq_slice(&out.upper, &upper),
        "BBANDS upper band deviates from golden vector beyond ADR 0005 tolerance"
    );
    assert!(
        approx_eq_slice(&out.middle, &middle),
        "BBANDS middle band deviates from golden vector beyond ADR 0005 tolerance"
    );
    assert!(
        approx_eq_slice(&out.lower, &lower),
        "BBANDS lower band deviates from golden vector beyond ADR 0005 tolerance"
    );
}

#[test]
fn trima_matches_golden_vector() {
    let (input, expected) = common::load_fixture("trima_basic.json").expect("load fixture");
    let out = trima(&input, 30).expect("trima");
    assert!(
        approx_eq_slice(&out, &expected),
        "TRIMA output deviates from golden vector beyond ADR 0005 tolerance"
    );
}

#[test]
fn t3_matches_golden_vector() {
    let (input, expected) = common::load_fixture("t3_basic.json").expect("load fixture");
    let out = t3(&input, 5, 0.7).expect("t3");
    assert!(
        approx_eq_slice(&out, &expected),
        "T3 output deviates from golden vector beyond ADR 0005 tolerance"
    );
}

#[test]
fn ma_matches_golden_vector() {
    let (input, expected) = common::load_fixture("ma_basic.json").expect("load fixture");
    let out = ma(&input, 30, MaType::Sma).expect("ma");
    assert!(
        approx_eq_slice(&out, &expected),
        "MA(SMA) output deviates from golden vector beyond ADR 0005 tolerance"
    );
}

#[test]
fn mavp_matches_golden_vector() {
    let json = common::load_json("mavp_basic.json").expect("load fixture");
    let input = common::load_f64_array(&json, "input").expect("input");
    let periods = common::load_f64_array(&json, "periods").expect("periods");
    let expected = common::load_f64_array(&json, "expected").expect("expected");
    let out = mavp(&input, &periods, 2, 30, MaType::Sma).expect("mavp");
    assert!(
        approx_eq_slice(&out, &expected),
        "MAVP output deviates from golden vector beyond ADR 0005 tolerance"
    );
}

#[test]
fn kama_matches_golden_vector() {
    let (input, expected) = common::load_fixture("kama_basic.json").expect("load fixture");
    let out = kama(&input, 30).expect("kama");
    assert!(
        approx_eq_slice(&out, &expected),
        "KAMA output deviates from golden vector beyond ADR 0005 tolerance"
    );
}

#[test]
fn sar_matches_golden_vector() {
    let json = common::load_json("sar_basic.json").expect("load fixture");
    let high = common::load_f64_array(&json, "high").expect("high");
    let low = common::load_f64_array(&json, "low").expect("low");
    let expected = common::load_f64_array(&json, "expected").expect("expected");
    let out = sar(&high, &low, 0.02, 0.2).expect("sar");
    assert!(
        approx_eq_slice(&out, &expected),
        "SAR output deviates from golden vector beyond ADR 0005 tolerance"
    );
}

#[test]
fn sarext_matches_golden_vector() {
    let json = common::load_json("sarext_basic.json").expect("load fixture");
    let high = common::load_f64_array(&json, "high").expect("high");
    let low = common::load_f64_array(&json, "low").expect("low");
    let expected = common::load_f64_array(&json, "expected").expect("expected");
    let out = sarext(&high, &low, 0.0, 0.0, 0.02, 0.02, 0.2, 0.02, 0.02, 0.2).expect("sarext");
    assert!(
        approx_eq_slice(&out, &expected),
        "SAREXT output deviates from golden vector beyond ADR 0005 tolerance"
    );
}
