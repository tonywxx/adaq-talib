//! 重叠研究函数黄金向量比对测试（见 ADR 0003 / ADR 0005）。
//!
//! Overlap-studies golden-vector comparison tests (ADR 0003 / ADR 0005).
//!
//! 注意：当前 fixture 为对照 TA-Lib 0.7.1 文档算法的参考值（见各 fixture 内 `_note`），
//! 待用户本机以 `tools/gen_fixtures/generate.py`（需 TA-Lib C）重新生成权威基准后，
//! 此处比对即等价于与原版逐项校验。
//!
//! NOTE: fixtures currently hold reference values derived from the TA-Lib 0.7.1
//! documented algorithm (see each fixture's `_note`). Once regenerated authoritatively
//! via `tools/gen_fixtures/generate.py` (requires TA-Lib C), these comparisons become
//! 1:1 checks against the original.

#[path = "common/mod.rs"]
mod common;

use adaq_talib::overlap::{dema, ema, midpoint, midprice, tema, wma};
use adaq_talib::utils::approx_eq_slice;

#[test]
fn ema_matches_golden_vector() {
    let (input, expected) = common::load_fixture("ema_basic.json").expect("load fixture");
    let out = ema(&input, 3).expect("ema");
    assert!(
        approx_eq_slice(&out, &expected),
        "EMA output deviates from golden vector beyond ADR 0005 tolerance"
    );
}

#[test]
fn wma_matches_golden_vector() {
    let (input, expected) = common::load_fixture("wma_basic.json").expect("load fixture");
    let out = wma(&input, 3).expect("wma");
    assert!(
        approx_eq_slice(&out, &expected),
        "WMA output deviates from golden vector beyond ADR 0005 tolerance"
    );
}

#[test]
fn dema_matches_golden_vector() {
    let (input, expected) = common::load_fixture("dema_basic.json").expect("load fixture");
    let out = dema(&input, 3).expect("dema");
    assert!(
        approx_eq_slice(&out, &expected),
        "DEMA output deviates from golden vector beyond ADR 0005 tolerance"
    );
}

#[test]
fn tema_matches_golden_vector() {
    let (input, expected) = common::load_fixture("tema_basic.json").expect("load fixture");
    let out = tema(&input, 3).expect("tema");
    assert!(
        approx_eq_slice(&out, &expected),
        "TEMA output deviates from golden vector beyond ADR 0005 tolerance"
    );
}

#[test]
fn midpoint_matches_golden_vector() {
    let (input, expected) = common::load_fixture("midpoint_basic.json").expect("load fixture");
    let out = midpoint(&input, 3).expect("midpoint");
    assert!(
        approx_eq_slice(&out, &expected),
        "MIDPOINT output deviates from golden vector beyond ADR 0005 tolerance"
    );
}

#[test]
fn midprice_matches_golden_vector() {
    let json = common::load_json("midprice_basic.json").expect("load fixture");
    let high = common::load_f64_array(&json, "high").expect("high");
    let low = common::load_f64_array(&json, "low").expect("low");
    let expected = common::load_f64_array(&json, "expected").expect("expected");
    let out = midprice(&high, &low, 3).expect("midprice");
    assert!(
        approx_eq_slice(&out, &expected),
        "MIDPRICE output deviates from golden vector beyond ADR 0005 tolerance"
    );
}
