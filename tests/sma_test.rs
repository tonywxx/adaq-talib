//! SMA 黄金向量比对测试（见 ADR 0003 / ADR 0005）。
//!
//! SMA golden-vector comparison test (ADR 0003 / ADR 0005).

#[path = "common/mod.rs"]
mod common;

use adaq_talib::overlap::sma;
use adaq_talib::utils::approx_eq_slice;

#[test]
fn sma_matches_golden_vector() {
    let (input, expected) = common::load_fixture("sma_basic.json").expect("load fixture");
    let out = sma(&input, 3).expect("sma");
    assert!(
        approx_eq_slice(&out, &expected),
        "SMA output deviates from golden vector beyond ADR 0005 tolerance"
    );
}
