//! MIDPRICE / MIDPOINT 基准测速（Rust 侧，零依赖）。
//! MIDPRICE / MIDPOINT benchmark (Rust side, dependency-free).
//!
//! 运行 / Run:
//! ```text
//! cargo bench --bench midprice_bench
//! ```
//! 对照原生 C（需系统安装 TA-Lib C 库）:
//! ```text
//! cargo bench --bench midprice_bench --features bench-c
//! ```

use adaq_talib::overlap::{midpoint, midprice};
use std::time::Instant;

const N: usize = 1_000_000;
const PERIOD: usize = 20;
const ITERS: usize = 20;

/// 确定性伪随机输入（LCG）。high/low 由 price 派生，保证确定性且互异。
fn sample_hl(n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut high = Vec::with_capacity(n);
    let mut low = Vec::with_capacity(n);
    let mut x = 12345.0f64;
    for _ in 0..n {
        x = (x * 1103515245.0 + 12345.0) % 1e9;
        let p = 50.0 + (x / 1e9) * 10.0;
        high.push(p * 1.01);
        low.push(p * 0.99);
    }
    (high, low)
}

fn main() {
    let (high, low) = sample_hl(N);

    // MIDPRICE: rolling (high+low)/2 over period — current hot path uses O(n·period).
    let start = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..ITERS {
        let out = midprice(&high, &low, PERIOD).unwrap();
        checksum += out[out.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust MIDPRICE:  {ITERS} iters x {N} elems = {elapsed:?}");
    println!("  avg/call: {:?}", elapsed / ITERS as u32);
    println!("  ns/elem : {:.2}", elapsed.as_nanos() as f64 / ITERS as f64 / N as f64);
    println!("  checksum (anti-optimize): {checksum}\n");

    // MIDPOINT: rolling (max+min)/2 over period. Same naive extreme-scan hot path.
    let start = Instant::now();
    let mut checksum2 = 0.0;
    for _ in 0..ITERS {
        let out = midpoint(&high, PERIOD).unwrap();
        checksum2 += out[out.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust MIDPOINT:  {ITERS} iters x {N} elems = {elapsed:?}");
    println!("  avg/call: {:?}", elapsed / ITERS as u32);
    println!("  ns/elem : {:.2}", elapsed.as_nanos() as f64 / ITERS as f64 / N as f64);
    println!("  checksum (anti-optimize): {checksum2}\n");

    #[cfg(feature = "bench-c")]
    run_c_bench(&high, &low);
}

#[cfg(feature = "bench-c")]
fn run_c_bench(high: &[f64], low: &[f64]) {
    unsafe {
        unsafe extern "C" {
            fn TA_Initialize() -> i32;
            fn TA_Shutdown() -> i32;
            fn TA_MIDPRICE(
                start_idx: i32,
                end_idx: i32,
                in_high: *const f64,
                in_low: *const f64,
                opt_in_time_period: i32,
                out_beg_idx: *mut i32,
                out_nb_element: *mut i32,
                out_real: *mut f64,
            ) -> i32;
            fn TA_MIDPOINT(
                start_idx: i32,
                end_idx: i32,
                in_real: *const f64,
                opt_in_time_period: i32,
                out_beg_idx: *mut i32,
                out_nb_element: *mut i32,
                out_real: *mut f64,
            ) -> i32;
        }
        assert_eq!(TA_Initialize(), 0, "TA_Initialize failed");
        let n = high.len() as i32;
        let mut out = vec![0.0f64; high.len()];
        let start = Instant::now();
        let mut checksum = 0.0;
        for _ in 0..ITERS {
            let mut beg = 0i32;
            let mut nb = 0i32;
            let rc = TA_MIDPRICE(
                0, n - 1, high.as_ptr(), low.as_ptr(), PERIOD as i32,
                &mut beg, &mut nb, out.as_mut_ptr(),
            );
            assert_eq!(rc, 0, "TA_MIDPRICE failed");
            checksum += out[(nb - 1) as usize];
        }
        let elapsed = start.elapsed();
        println!("C MIDPRICE (native): {ITERS} iters x {N} elems = {elapsed:?}");
        println!("  avg/call: {:?}", elapsed / ITERS as u32);
        println!("  ns/elem : {:.2}", elapsed.as_nanos() as f64 / ITERS as f64 / N as f64);
        println!("  checksum (anti-optimize): {checksum}");

        let start = Instant::now();
        let mut checksum2 = 0.0;
        for _ in 0..ITERS {
            let mut beg = 0i32;
            let mut nb = 0i32;
            let rc = TA_MIDPOINT(
                0, n - 1, high.as_ptr(), PERIOD as i32,
                &mut beg, &mut nb, out.as_mut_ptr(),
            );
            assert_eq!(rc, 0, "TA_MIDPOINT failed");
            checksum2 += out[(nb - 1) as usize];
        }
        let elapsed = start.elapsed();
        println!("C MIDPOINT (native): {ITERS} iters x {N} elems = {elapsed:?}");
        println!("  avg/call: {:?}", elapsed / ITERS as u32);
        println!("  ns/elem : {:.2}", elapsed.as_nanos() as f64 / ITERS as f64 / N as f64);
        println!("  checksum (anti-optimize): {checksum2}");
        TA_Shutdown();
    }
}
