//! MINMAX 基准测速（Rust 侧，零依赖）。
//! MINMAX benchmark (Rust side, dependency-free).
//!
//! 运行 / Run:
//! ```text
//! cargo bench --bench minmax_bench
//! ```
//! 对照原生 C（需系统安装 TA-Lib C 库）:
//! ```text
//! cargo bench --bench minmax_bench --features bench-c
//! ```

use adaq_talib::math_ops::{max, min, minmax};
use std::time::Instant;

const N: usize = 1_000_000;
const PERIOD: usize = 20;
const ITERS: usize = 20;

/// 确定性伪随机输入（LCG），与仓库其他 bench 同口径。
/// Deterministic pseudo-random input (LCG), same convention as the other benches.
fn sample(n: usize) -> Vec<f64> {
    let mut v = Vec::with_capacity(n);
    let mut x = 12345.0f64;
    for _ in 0..n {
        x = (x * 1103515245.0 + 12345.0) % 1e9;
        v.push(50.0 + (x / 1e9) * 10.0);
    }
    v
}

fn main() {
    let values = sample(N);

    // MINMAX: 单遍双队列（P1 候选②：复用 core::rolling_minmax，原为两遍 rolling_min + rolling_max）。
    // MINMAX: single-pass dual-deque (P1 candidate ②: reuses core::rolling_minmax; was two
    // separate passes rolling_min + rolling_max).
    let start = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..ITERS {
        let out = minmax(&values, PERIOD).unwrap();
        checksum += out.max[out.max.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust MINMAX (single-pass): {ITERS} iters x {N} elems = {elapsed:?}");
    println!("  avg/call: {:?}", elapsed / ITERS as u32);
    println!(
        "  ns/elem : {:.2}",
        elapsed.as_nanos() as f64 / ITERS as f64 / N as f64
    );
    println!("  checksum (anti-optimize): {checksum}\n");

    // 旧成本等价：两次独立极值扫描（max + min），即重构前两遍开销。
    // Pre-change equivalent: two independent extreme scans (max + min).
    let start = Instant::now();
    let mut checksum2 = 0.0;
    for _ in 0..ITERS {
        let mx = max(&values, PERIOD).unwrap();
        let mn = min(&values, PERIOD).unwrap();
        checksum2 += mx[mx.len() - 1] + mn[mn.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust MAX+MIN (two-pass, pre-change equivalent): {ITERS} iters x {N} elems = {elapsed:?}");
    println!("  avg/call: {:?}", elapsed / ITERS as u32);
    println!(
        "  ns/elem : {:.2}",
        elapsed.as_nanos() as f64 / ITERS as f64 / N as f64
    );
    println!("  checksum (anti-optimize): {checksum2}\n");

    #[cfg(feature = "bench-c")]
    run_c_bench(&values);
}

#[cfg(feature = "bench-c")]
fn run_c_bench(values: &[f64]) {
    unsafe {
        unsafe extern "C" {
            fn TA_Initialize() -> i32;
            fn TA_Shutdown() -> i32;
            fn TA_MINMAX(
                start_idx: i32,
                end_idx: i32,
                in_real: *const f64,
                opt_in_time_period: i32,
                out_beg_idx: *mut i32,
                out_nb_element: *mut i32,
                out_min: *mut f64,
                out_max: *mut f64,
            ) -> i32;
        }
        assert_eq!(TA_Initialize(), 0, "TA_Initialize failed");
        let n = values.len() as i32;
        let mut out_min = vec![0.0f64; values.len()];
        let mut out_max = vec![0.0f64; values.len()];
        let start = Instant::now();
        let mut checksum = 0.0;
        for _ in 0..ITERS {
            let mut beg = 0i32;
            let mut nb = 0i32;
            let rc = TA_MINMAX(
                0,
                n - 1,
                values.as_ptr(),
                PERIOD as i32,
                &mut beg,
                &mut nb,
                out_min.as_mut_ptr(),
                out_max.as_mut_ptr(),
            );
            assert_eq!(rc, 0, "TA_MINMAX failed");
            checksum += out_max[(nb - 1) as usize];
        }
        let elapsed = start.elapsed();
        println!("C MINMAX (native): {ITERS} iters x {N} elems = {elapsed:?}");
        println!("  avg/call: {:?}", elapsed / ITERS as u32);
        println!(
            "  ns/elem : {:.2}",
            elapsed.as_nanos() as f64 / ITERS as f64 / N as f64
        );
        println!("  checksum (anti-optimize): {checksum}");
        TA_Shutdown();
    }
}
