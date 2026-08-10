//! 数学变换黄金向量比对测试（见 ADR 0003 / ADR 0005）。
//!
//! Math Transform golden-vector comparison tests (ADR 0003 / ADR 0005).
//!
//! 注意：fixture 为 `tools/gen_fixtures/generate.py` 基于 **TA-Lib C 0.7.1** 真实输出生成的
//! 权威黄金向量。此处比对即等价于与原版逐项 **1:1** 校验。
//!
//! NOTE: fixtures are authoritative golden vectors generated from real TA-Lib C 0.7.1 output
//! via `tools/gen_fixtures/generate.py`. These comparisons are 1:1 checks against the original.

#[path = "common/mod.rs"]
mod common;

use adaq_talib::math_trans::{
    acos, asin, atan, ceil, cos, cosh, exp, floor, ln, log10, sin, sinh, sqrt, tan, tanh,
};
use adaq_talib::utils::approx_eq_slice;

macro_rules! mt_test {
    ($name:ident, $fn:ident, $fixture:expr) => {
        #[test]
        fn $name() {
            let (input, expected) = common::load_fixture($fixture).expect("load fixture");
            let out = $fn(&input).expect(stringify!($fn));
            assert!(
                approx_eq_slice(&out, &expected),
                "{} output deviates from golden vector beyond ADR 0005 tolerance",
                stringify!($fn)
            );
        }
    };
}

mt_test!(acos_matches_golden_vector, acos, "acos_basic.json");
mt_test!(asin_matches_golden_vector, asin, "asin_basic.json");
mt_test!(atan_matches_golden_vector, atan, "atan_basic.json");
mt_test!(ceil_matches_golden_vector, ceil, "ceil_basic.json");
mt_test!(cos_matches_golden_vector, cos, "cos_basic.json");
mt_test!(cosh_matches_golden_vector, cosh, "cosh_basic.json");
mt_test!(exp_matches_golden_vector, exp, "exp_basic.json");
mt_test!(floor_matches_golden_vector, floor, "floor_basic.json");
mt_test!(ln_matches_golden_vector, ln, "ln_basic.json");
mt_test!(log10_matches_golden_vector, log10, "log10_basic.json");
mt_test!(sin_matches_golden_vector, sin, "sin_basic.json");
mt_test!(sinh_matches_golden_vector, sinh, "sinh_basic.json");
mt_test!(sqrt_matches_golden_vector, sqrt, "sqrt_basic.json");
mt_test!(tan_matches_golden_vector, tan, "tan_basic.json");
mt_test!(tanh_matches_golden_vector, tanh, "tanh_basic.json");
