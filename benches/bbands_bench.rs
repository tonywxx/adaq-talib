//! BBANDS 基准测速（Rust 侧，零依赖）。/ BBANDS benchmark (Rust side, dependency-free).
//!
//! 运行 / Run:
//! ```text
//! cargo bench --bench bbands_bench
//! ```
//! 对照原生 C（需系统安装 TA-Lib C 库）:
//! ```text
//! cargo bench --bench bbands_bench --features bench-c
//! ```

use adaq_talib::overlap::{bbands, MaType};
use std::time::Instant;

const N: usize = 1_000_000;
const PERIOD: usize = 20;
const DEV_UP: f64 = 2.0;
const DEV_DN: f64 = 2.0;
const ITERS: usize = 20;

fn sample_prices(n: usize) -> Vec<f64> {
    let mut prices = Vec::with_capacity(n);
    let mut x = 12345.0f64;
    for _ in 0..n {
        x = (x * 1103515245.0 + 12345.0) % 1e9;
        prices.push(50.0 + (x / 1e9) * 10.0);
    }
    prices
}

fn main() {
    let prices = sample_prices(N);

    let start = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..ITERS {
        let out = bbands(&prices, PERIOD, DEV_UP, DEV_DN, MaType::Sma).unwrap();
        checksum += out.upper[out.upper.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust BBANDS:  {ITERS} iters x {N} elems = {elapsed:?}");
    println!("  avg/call: {:?}", elapsed / ITERS as u32);
    println!("  ns/elem : {:.2}", elapsed.as_nanos() as f64 / ITERS as f64 / N as f64);
    println!("  checksum (anti-optimize): {checksum}\n");

    #[cfg(feature = "bench-c")]
    run_c_bench(&prices);
}

#[cfg(feature = "bench-c")]
fn run_c_bench(prices: &[f64]) {
    unsafe {
        unsafe extern "C" {
            fn TA_Initialize() -> i32;
            fn TA_Shutdown() -> i32;
            fn TA_BBANDS(
                start_idx: i32,
                end_idx: i32,
                in_real: *const f64,
                opt_in_time_period: i32,
                opt_in_nb_dev_up: f64,
                opt_in_nb_dev_dn: f64,
                opt_in_ma_type: i32,
                out_beg_idx: *mut i32,
                out_nb_element: *mut i32,
                out_real_upper: *mut f64,
                out_real_middle: *mut f64,
                out_real_lower: *mut f64,
            ) -> i32;
        }
        assert_eq!(TA_Initialize(), 0, "TA_Initialize failed");
        let n = prices.len() as i32;
        let mut up = vec![0.0f64; prices.len()];
        let mut mid = vec![0.0f64; prices.len()];
        let mut lo = vec![0.0f64; prices.len()];
        let start = Instant::now();
        let mut checksum = 0.0;
        for _ in 0..ITERS {
            let mut beg = 0i32;
            let mut nb = 0i32;
            let rc = TA_BBANDS(
                0, n - 1, prices.as_ptr(), PERIOD as i32, DEV_UP, DEV_DN, 0, // 0 = SMA
                &mut beg, &mut nb, up.as_mut_ptr(), mid.as_mut_ptr(), lo.as_mut_ptr(),
            );
            assert_eq!(rc, 0, "TA_BBANDS failed");
            checksum += up[(nb - 1) as usize];
        }
        let elapsed = start.elapsed();
        println!("C BBANDS (native): {ITERS} iters x {N} elems = {elapsed:?}");
        println!("  avg/call: {:?}", elapsed / ITERS as u32);
        println!("  ns/elem : {:.2}", elapsed.as_nanos() as f64 / ITERS as f64 / N as f64);
        println!("  checksum (anti-optimize): {checksum}");
        TA_Shutdown();
    }
}
