//! 周期类（希尔伯特变换）黄金向量比对测试（MAMA / HT_TRENDLINE）。见 ADR 0003 / 0005。
//!
//! Cycle (Hilbert-transform) golden-vector comparison tests (MAMA / HT_TRENDLINE).
//! See ADR 0003 / 0005.
//!
//! fixture 均为对照已安装 `talib` 0.7.1 生成的权威黄金向量（MAMA/FAMA、HT_TRENDLINE 均与
//! 原版逐项一致，diff 0.0），此处比对即等价于与原版 1:1 校验。
//!
//! Fixtures are authoritative golden vectors from the installed `talib` 0.7.1 (MAMA/FAMA and
//! HT_TRENDLINE each match the original 1:1, diff 0.0); these comparisons are 1:1 checks.

#[path = "common/mod.rs"]
mod common;

use adaq_talib::cycle::{ht_trendline_default, mama_default};
use adaq_talib::utils::approx_eq_slice;

#[test]
fn mama_matches_golden_vector() {
    let json = common::load_json("mama_basic.json").expect("load fixture");
    let input = common::load_f64_array(&json, "input").expect("input");
    let mama_exp = common::load_f64_array(&json, "mama").expect("mama");
    let fama_exp = common::load_f64_array(&json, "fama").expect("fama");
    let out = mama_default(&input).expect("mama");
    assert!(
        approx_eq_slice(&out.mama, &mama_exp),
        "MAMA line deviates from golden vector beyond ADR 0005 tolerance"
    );
    assert!(
        approx_eq_slice(&out.fama, &fama_exp),
        "FAMA line deviates from golden vector beyond ADR 0005 tolerance"
    );
}

#[test]
fn ht_trendline_matches_golden_vector() {
    let (input, expected) = common::load_fixture("ht_trendline_basic.json").expect("load fixture");
    let out = ht_trendline_default(&input).expect("ht_trendline");
    assert!(
        approx_eq_slice(&out, &expected),
        "HT_TRENDLINE output deviates from golden vector beyond ADR 0005 tolerance"
    );
}
