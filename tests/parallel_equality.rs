//! P3-2 parallel correctness gate: parallel output must equal serial (golden) output 1:1.
//! 仅 `parallel` feature 下编译运行。/ Compiled & run only under the `parallel` feature.
#![cfg(feature = "parallel")]

use adaq_talib::math_ops::{minmax_index_parallel, minmax_index_serial, minmax_parallel, minmax_serial};
use adaq_talib::momentum::{stoch_f_parallel, stoch_f_serial, willr_parallel, willr_serial};
use adaq_talib::overlap::{midpoint_parallel, midpoint_serial};

/// 确定性 LCG 输入，避免随机性导致测试不可复现。`seed` 用于生成相互独立的序列（如 high/low/close）。
/// Deterministic LCG input so the test is reproducible. `seed` lets us generate independent
/// series (e.g. high/low/close).
fn lcg_input(n: usize, seed: u64) -> Vec<f64> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            f64::from_bits((x >> 11) | 0x3FF0000000000000) - 1.0
        })
        .collect()
}

/// 逐项 1:1 比较，显式处理前导 NaN（NaN ≠ NaN，不能直接 `==`）。
/// Element-wise 1:1 comparison with explicit NaN handling (NaN != NaN in Rust).
fn assert_eq_1to1(name: &str, period: usize, a: &[f64], b: &[f64]) {
    assert_eq!(a.len(), b.len(), "{name}: length mismatch");
    for (i, (x, y)) in a.iter().zip(b).enumerate() {
        if x.is_nan() && y.is_nan() {
            continue; // 两侧均为前导 NaN → 一致 / both leading NaN → equal
        }
        assert!(
            !(x.is_nan() ^ y.is_nan()),
            "{name} period={period} idx={i}: one side NaN (serial={x}, parallel={y})"
        );
        let tol = 1e-8 * x.abs().max(y.abs()) + 1e-10;
        assert!(
            (x - y).abs() <= tol,
            "{name} period={period} idx={i}: serial={x} parallel={y} (diff={})",
            (x - y).abs()
        );
    }
}

#[test]
fn midpoint_parallel_matches_serial() {
    let data = lcg_input(50_000, 0x2545F491_4F6CDD1D);
    // 多种周期，验证重叠播种在边界处的正确性。
    // Multiple periods to exercise overlap seeding at chunk boundaries.
    for period in [1usize, 2, 3, 5, 10, 14, 30, 64] {
        let serial = midpoint_serial(&data, period).unwrap();
        let parallel = midpoint_parallel(&data, period).unwrap();
        assert_eq_1to1("midpoint", period, &serial, &parallel);
    }
}

#[test]
fn midpoint_parallel_small_input_is_serial() {
    // 小输入（< 8192）应直接回退到串行，仍与串行参考一致。
    // Small input (< 8192) falls back to serial and still matches the reference.
    let data = lcg_input(1024, 0x2545F491_4F6CDD1D);
    let serial = midpoint_serial(&data, 5).unwrap();
    let parallel = midpoint_parallel(&data, 5).unwrap();
    assert_eq_1to1("midpoint", 5, &serial, &parallel);
}

#[test]
fn minmax_parallel_matches_serial() {
    let data = lcg_input(50_000, 0x9E3779B97F4A7C15);
    for period in [1usize, 2, 3, 5, 10, 14, 30, 64] {
        let s = minmax_serial(&data, period).unwrap();
        let p = minmax_parallel(&data, period).unwrap();
        assert_eq_1to1("minmax.min", period, &s.min, &p.min);
        assert_eq_1to1("minmax.max", period, &s.max, &p.max);
    }
}

#[test]
fn minmax_index_parallel_matches_serial() {
    let data = lcg_input(50_000, 0xBF58476D1CE4E5B9);
    for period in [1usize, 2, 3, 5, 10, 14, 30, 64] {
        let s = minmax_index_serial(&data, period).unwrap();
        let p = minmax_index_parallel(&data, period).unwrap();
        assert_eq_1to1("minmax_index.min_idx", period, &s.min_idx, &p.min_idx);
        assert_eq_1to1("minmax_index.max_idx", period, &s.max_idx, &p.max_idx);
    }
}

#[test]
fn willr_parallel_matches_serial() {
    let high = lcg_input(50_000, 0xC2B2AE3D27D4EB4F);
    let low = lcg_input(50_000, 0x27D4EB4FC2B2AE3D);
    let close = lcg_input(50_000, 0x165667B19E3779F9);
    for period in [5usize, 14, 30] {
        let s = willr_serial(&high, &low, &close, period).unwrap();
        let p = willr_parallel(&high, &low, &close, period).unwrap();
        assert_eq_1to1("willr", period, &s, &p);
    }
}

#[test]
fn stoch_f_parallel_matches_serial() {
    let high = lcg_input(50_000, 0x7F4A7C159E3779B9);
    let low = lcg_input(50_000, 0x1CE4E5B9BF58476D);
    let close = lcg_input(50_000, 0xD1CE4E5B97F4A7C1);
    for (fk, fd) in [(5usize, 3usize), (14, 3), (30, 5)] {
        let s = stoch_f_serial(&high, &low, &close, fk, fd).unwrap();
        let p = stoch_f_parallel(&high, &low, &close, fk, fd).unwrap();
        assert_eq_1to1("stoch_f.fast_k", fk, &s.fast_k, &p.fast_k);
        assert_eq_1to1("stoch_f.fast_d", fk, &s.fast_d, &p.fast_d);
    }
}
