//! DEMA 基准测速（Rust 侧，零依赖）。/ DEMA benchmark (Rust side, dependency-free).
//!
//! 运行 / Run:
//! ```text
//! cargo bench --bench dema_bench
//! ```
//! 对照原生 C（需系统安装 TA-Lib C 库）:
//! ```text
//! cargo bench --bench dema_bench --features bench-c
//! ```

use adaq_talib::overlap::dema;
use std::time::Instant;

const N: usize = 1_000_000;
const PERIOD: usize = 20;
const ITERS: usize = 20;

/// 确定性伪随机输入（LCG），避免 benchmark 间输入变化。
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
        let out = dema(&prices, PERIOD).unwrap();
        checksum += out[out.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust DEMA:  {ITERS} iters x {N} elems = {elapsed:?}");
    println!("  avg/call: {:?}", elapsed / ITERS as u32);
    println!("  ns/elem : {:.2}", elapsed.as_nanos() as f64 / ITERS as f64 / N as f64);
    println!("  checksum (anti-optimize): {checksum}\n");

    #[cfg(feature = "bench-c")]
    run_c_bench(&prices);
}

/// FFI 对照原生 TA-Lib C。仅在 `bench-c` feature 下编译，需系统安装 `libta_lib`。
#[cfg(feature = "bench-c")]
fn run_c_bench(prices: &[f64]) {
    unsafe {
        unsafe extern "C" {
            fn TA_Initialize() -> i32;
            fn TA_Shutdown() -> i32;
            fn TA_DEMA(
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
        let n = prices.len() as i32;
        let mut out = vec![0.0f64; prices.len()];
        let start = Instant::now();
        let mut checksum = 0.0;
        for _ in 0..ITERS {
            let mut beg = 0i32;
            let mut nb = 0i32;
            let rc = TA_DEMA(
                0,
                n - 1,
                prices.as_ptr(),
                PERIOD as i32,
                &mut beg,
                &mut nb,
                out.as_mut_ptr(),
            );
            assert_eq!(rc, 0, "TA_DEMA failed");
            checksum += out[(nb - 1) as usize];
        }
        let elapsed = start.elapsed();
        println!("C DEMA (native): {ITERS} iters x {N} elems = {elapsed:?}");
        println!("  avg/call: {:?}", elapsed / ITERS as u32);
        println!("  ns/elem : {:.2}", elapsed.as_nanos() as f64 / ITERS as f64 / N as f64);
        println!("  checksum (anti-optimize): {checksum}");
        TA_Shutdown();
    }
}
