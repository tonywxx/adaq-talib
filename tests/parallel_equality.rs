//! P3-2 parallel correctness gate: parallel output must equal serial (golden) output 1:1.
//! 仅 `parallel` feature 下编译运行。/ Compiled & run only under the `parallel` feature.
#![cfg(feature = "parallel")]

use adaq_talib::overlap::{midpoint_parallel, midpoint_serial};

/// 确定性 LCG 输入，避免随机性导致测试不可复现。
/// Deterministic LCG input so the test is reproducible.
fn lcg_input(n: usize) -> Vec<f64> {
    let mut x: u64 = 0x2545F491_4F6CDD1D;
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
    let data = lcg_input(50_000);
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
    let data = lcg_input(1024);
    let serial = midpoint_serial(&data, 5).unwrap();
    let parallel = midpoint_parallel(&data, 5).unwrap();
    assert_eq_1to1("midpoint", 5, &serial, &parallel);
}
