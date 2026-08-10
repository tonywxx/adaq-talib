//! 周期类（希尔伯特变换）黄金向量比对测试（MAMA / HT_TRENDLINE）。见 ADR 0003 / 0005。
//!
//! Cycle (Hilbert-transform) golden-vector comparison tests (MAMA / HT_TRENDLINE).
//! See ADR 0003 / 0005.
//!
//! 注意：当前 fixture 为对照 TA-Lib 0.7.1 文档算法的参考值（见各 fixture 内 `_note`），
//! 待用户本机以 `tools/gen_fixtures/generate.py`（需 TA-Lib C）重新生成权威基准后，
//! 此处比对即等价于与原版逐项校验。由于希尔伯特变换累积浮点运算较多，参考实现与 Rust
//! 实现逐项对齐（同序 IEEE-754 运算），差异应在 ADR 0005 容限内。
//!
//! NOTE: fixtures currently hold reference values (see each fixture's `_note`). The two
//! implementations align op-by-op (same-sequence IEEE-754 arithmetic), so the difference
//! should stay within the ADR 0005 tolerance.

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
