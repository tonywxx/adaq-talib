//! P3-2 midpoint parallel PoC bench.
//! 用法 / Usage:
//!   cargo bench --bench parallel_poc            # 仅串行参考
//!   cargo bench --bench parallel_poc --features parallel   # 串行 + 并行对照
//! 比较两次运行下 midpoint 的 ns/elem 即可得并行加速比。
//! Compare `midpoint` ns/elem between the two runs to get the parallel speedup.
use adaq_talib::overlap::{midpoint, midpoint_serial};
#[cfg(feature = "parallel")]
use adaq_talib::overlap::midpoint_parallel;
use std::hint::black_box;
use std::time::Instant;

const N: usize = 200_000;
const PERIOD: usize = 5;
const RUNS: usize = 5;

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

/// 取 RUNS 次运行的中位数 ns/elem（与 all161 基准一致：median-of-5）。
/// Median ns/elem over RUNS runs (consistent with the all161 harness: median-of-5).
fn median_ns<F: Fn() -> Vec<f64>>(f: F) -> f64 {
    let mut ts = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let t = Instant::now();
        let _ = black_box(f());
        ts.push(t.elapsed().as_nanos() as f64 / N as f64);
    }
    ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ts[RUNS / 2]
}

fn main() {
    let data = lcg_input(N);
    // warmup
    let _ = black_box(midpoint_serial(&data, PERIOD));

    let serial = median_ns(|| midpoint_serial(&data, PERIOD).unwrap());
    println!("midpoint (serial reference)   : {serial:8.3} ns/elem");

    // 无 parallel feature 时 midpoint_parallel 不存在；此处统一用 midpoint 作为
    // “默认分发”的快照：feature 关闭时为串行，开启时为大输入并行。
    // Without the parallel feature `midpoint_parallel` does not exist; `midpoint` is the
    // dispatch snapshot: serial when the feature is off, parallel for large input when on.
    #[cfg(feature = "parallel")]
    {
        let dispatched = median_ns(|| midpoint(&data, PERIOD).unwrap());
        let parallel = median_ns(|| midpoint_parallel(&data, PERIOD).unwrap());
        let speedup = serial / dispatched;
        println!("midpoint (feature dispatch)   : {dispatched:8.3} ns/elem  (serial/{dispatched} = {speedup:.2}x)");
        println!("midpoint (explicit parallel)  : {parallel:8.3} ns/elem  (serial/parallel = {:.2}x)", serial / parallel);
    }
    #[cfg(not(feature = "parallel"))]
    {
        let dispatched = median_ns(|| midpoint(&data, PERIOD).unwrap());
        println!("midpoint (no parallel feature): {dispatched:8.3} ns/elem  (serial/dispatch = {:.2}x)", serial / dispatched);
    }
}
