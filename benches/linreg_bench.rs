//! LINEARREG 基准测速（Rust 侧，零依赖）。/ LINEARREG benchmark (Rust side, dependency-free).
//!
//! 代表整个线性回归族（LINEARREG / _ANGLE / _INTERCEPT / _SLOPE / TSF），共享 `linreg_core`。
//! Representative of the whole linear-regression family (LINEARREG / _ANGLE / _INTERCEPT /
//! _SLOPE / TSF), which share `linreg_core`.
//!
//! 运行 / Run:
//! ```text
//! cargo bench --bench linreg_bench
//! ```

use adaq_talib::stat::{linear_reg, linear_reg_with_output};
use std::time::Instant;

const N: usize = 1_000_000;
const PERIOD: usize = 20;
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
        let out = linear_reg(&prices, PERIOD).unwrap();
        checksum += out[out.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust LINEARREG:  {ITERS} iters x {N} elems = {elapsed:?}");
    println!("  avg/call: {:?}", elapsed / ITERS as u32);
    println!(
        "  ns/elem : {:.2}",
        elapsed.as_nanos() as f64 / ITERS as f64 / N as f64
    );
    println!("  checksum (anti-optimize): {checksum}\n");

    // 原地变体对照（确认零偏差且性能一致）。/ in-place variant sanity check.
    let mut buf = vec![f64::NAN; N];
    let start = Instant::now();
    let mut checksum2 = 0.0;
    for _ in 0..ITERS {
        linear_reg_with_output(&prices, PERIOD, &mut buf).unwrap();
        checksum2 += buf[buf.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust LINEARREG(_with_output):  {ITERS} iters x {N} elems = {elapsed:?}");
    println!(
        "  ns/elem : {:.2}",
        elapsed.as_nanos() as f64 / ITERS as f64 / N as f64
    );
    println!("  checksum (anti-optimize): {checksum2}");
}
