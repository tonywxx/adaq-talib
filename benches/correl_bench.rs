//! CORREL 基准测速（Rust 侧，零依赖）。/ CORREL benchmark (Rust side, dependency-free).
//!
//! 运行 / Run:
//! ```text
//! cargo bench --bench correl_bench
//! ```

use adaq_talib::stat::{correl, correl_with_output};
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
    let a = sample_prices(N);
    let b = sample_prices(N);

    let start = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..ITERS {
        let out = correl(&a, &b, PERIOD).unwrap();
        checksum += out[out.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust CORREL:  {ITERS} iters x {N} elems = {elapsed:?}");
    println!("  avg/call: {:?}", elapsed / ITERS as u32);
    println!(
        "  ns/elem : {:.2}",
        elapsed.as_nanos() as f64 / ITERS as f64 / N as f64
    );
    println!("  checksum (anti-optimize): {checksum}\n");

    let mut buf = vec![f64::NAN; N];
    let start = Instant::now();
    let mut checksum2 = 0.0;
    for _ in 0..ITERS {
        correl_with_output(&a, &b, PERIOD, &mut buf).unwrap();
        checksum2 += buf[buf.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust CORREL(_with_output):  {ITERS} iters x {N} elems = {elapsed:?}");
    println!(
        "  ns/elem : {:.2}",
        elapsed.as_nanos() as f64 / ITERS as f64 / N as f64
    );
    println!("  checksum (anti-optimize): {checksum2}");
}
