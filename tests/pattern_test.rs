//! 形态识别（Pattern Recognition）黄金向量比对测试，第 1 批（验证批）。见 ADR 0003 / ADR 0005。
//!
//! Golden-vector comparison tests for Pattern Recognition, batch 1 (validation batch).
//! See ADR 0003 / ADR 0005.
//!
//! fixture 为 `tools/gen_fixtures/generate.py` 基于 **TA-Lib C 0.7.1** 真实输出生成的权威黄金向量
//! （`tests/fixtures/cdl_*.json`，前导 `lookback` 位置填 `0.0`，整数输出约定，见 ADR 0007）。
//! 此处比对即等价于与原版逐项 1:1 校验。
//!
//! Fixtures are authoritative golden vectors generated from real TA-Lib C 0.7.1 output via
//! `tools/gen_fixtures/generate.py` (`tests/fixtures/cdl_*.json`, leading `lookback` positions
//! `0.0`, integer-output convention, ADR 0007). These comparisons are 1:1 checks against the original.

#[path = "common/mod.rs"]
mod common;

use adaq_talib::pattern::{
    cdl_2crows, cdl_doji, cdl_engulfing, cdl_hammer, cdl_harami, cdl_highwave, cdl_marubozu,
    cdl_shootingstar,
};
use adaq_talib::utils::approx_eq_slice;

/// 通用比对：加载 `cdl_<name>.json`，调用 `f(open, high, low, close)`，与 `expected` 逐项比对。
/// Generic compare: load `cdl_<name>.json`, call `f(open, high, low, close)`, compare to `expected`.
fn check_cdl(
    name: &str,
    f: fn(&[f64], &[f64], &[f64], &[f64]) -> Result<Vec<f64>, adaq_talib::error::TaError>,
) {
    let fixture = format!("cdl_{name}.json");
    let json = common::load_json(&fixture).expect("load fixture");
    let open = common::load_f64_array(&json, "open").expect("open");
    let high = common::load_f64_array(&json, "high").expect("high");
    let low = common::load_f64_array(&json, "low").expect("low");
    let close = common::load_f64_array(&json, "close").expect("close");
    let expected = common::load_f64_array(&json, "expected").expect("expected");
    let out = f(&open, &high, &low, &close).expect("pattern fn");
    assert_eq!(out.len(), expected.len(), "length mismatch for cdl_{name}");
    assert!(
        approx_eq_slice(&out, &expected),
        "cdl_{name} output deviates from golden vector beyond ADR 0005 tolerance"
    );
}

#[test]
fn cdl_doji_matches_golden_vector() {
    check_cdl("doji", cdl_doji);
}

#[test]
fn cdl_marubozu_matches_golden_vector() {
    check_cdl("marubozu", cdl_marubozu);
}

#[test]
fn cdl_hammer_matches_golden_vector() {
    check_cdl("hammer", cdl_hammer);
}

#[test]
fn cdl_shootingstar_matches_golden_vector() {
    check_cdl("shootingstar", cdl_shootingstar);
}

#[test]
fn cdl_engulfing_matches_golden_vector() {
    check_cdl("engulfing", cdl_engulfing);
}

/// 锁定 Engulfing 的两级输出（100 / 80 / -100 / -80 / 0）与已安装 dylib 一致。
/// Locks the two-tier engulfing outputs against the installed dylib behavior.
#[test]
fn cdl_engulfing_two_tier_outputs() {
    // 看涨：A 完全(100) / B 边界(80) / C 非吞没(0)
    let o = [10.0, 10.0, 7.0, 9.0, 9.0];
    let h = [10.5, 10.5, 11.5, 9.5, 9.5];
    let l = [7.5, 7.5, 6.5, 8.5, 8.5];
    let c = [8.0, 8.0, 11.0, 9.0, 9.0];
    let out = cdl_engulfing(&o, &h, &l, &c).unwrap();
    assert_eq!(out[2], 100.0, "bullish full engulf should be 100");

    let o2 = [10.0, 10.0, 8.0, 9.0, 9.0];
    let c2 = [8.0, 8.0, 11.0, 9.0, 9.0];
    let out2 = cdl_engulfing(&o2, &h, &l, &c2).unwrap();
    assert_eq!(out2[2], 80.0, "bullish boundary engulf should be 80");

    let o3 = [10.0, 10.0, 9.0, 9.0, 9.0];
    let c3 = [8.0, 8.0, 11.0, 9.0, 9.0];
    let out3 = cdl_engulfing(&o3, &h, &l, &c3).unwrap();
    assert_eq!(out3[2], 0.0, "bullish gap-up (open>prev close) should be 0");

    // 看跌：A 完全(-100) / B 边界(-80) / C 非吞没(0)
    let ob = [10.0, 8.0, 11.0, 9.0, 9.0];
    let cb = [8.0, 10.0, 7.0, 9.0, 9.0];
    let outb = cdl_engulfing(&ob, &h, &l, &cb).unwrap();
    assert_eq!(outb[2], -100.0, "bearish full engulf should be -100");

    let ob2 = [10.0, 8.0, 10.0, 9.0, 9.0];
    let cb2 = [8.0, 10.0, 7.0, 9.0, 9.0];
    let outb2 = cdl_engulfing(&ob2, &h, &l, &cb2).unwrap();
    assert_eq!(outb2[2], -80.0, "bearish boundary engulf should be -80");

    let ob3 = [10.0, 8.0, 9.0, 9.0, 9.0];
    let cb3 = [8.0, 10.0, 7.0, 9.0, 9.0];
    let outb3 = cdl_engulfing(&ob3, &h, &l, &cb3).unwrap();
    assert_eq!(outb3[2], 0.0, "bearish gap-down (close>prev open) should be 0");
}

#[test]
fn cdl_harami_matches_golden_vector() {
    check_cdl("harami", cdl_harami);
}

#[test]
fn cdl_highwave_matches_golden_vector() {
    check_cdl("highwave", cdl_highwave);
}

#[test]
fn cdl_2crows_matches_golden_vector() {
    check_cdl("2crows", cdl_2crows);
}
