//! WILLR 基准测速（Rust 侧，零依赖）。/ WILLR benchmark (Rust side, dependency-free).
//!
//! 运行 / Run:
//! ```text
//! cargo bench --bench willr_bench
//! ```

use adaq_talib::momentum::{willr, willr_with_output};
use std::time::Instant;

const N: usize = 1_000_000;
const PERIOD: usize = 20;
const ITERS: usize = 20;

/// 生成合理的高/低/收序列（保证 high[i] >= low[i]）。/ OHLC series (high[i] >= low[i]).
fn sample_ohlc(n: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut high = Vec::with_capacity(n);
    let mut low = Vec::with_capacity(n);
    let mut close = Vec::with_capacity(n);
    let mut x = 12345.0f64;
    for _ in 0..n {
        x = (x * 1103515245.0 + 12345.0) % 1e9;
        let base = 50.0 + (x / 1e9) * 10.0;
        x = (x * 1103515245.0 + 12345.0) % 1e9;
        let h = base + (x / 1e9) * 2.0;
        x = (x * 1103515245.0 + 12345.0) % 1e9;
        let l = base - (x / 1e9) * 2.0;
        high.push(h);
        low.push(l);
        close.push(base);
    }
    (high, low, close)
}

fn main() {
    let (high, low, close) = sample_ohlc(N);

    let start = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..ITERS {
        let out = willr(&high, &low, &close, PERIOD).unwrap();
        checksum += out[out.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust WILLR:  {ITERS} iters x {N} elems = {elapsed:?}");
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
        willr_with_output(&high, &low, &close, PERIOD, &mut buf).unwrap();
        checksum2 += buf[buf.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust WILLR(_with_output):  {ITERS} iters x {N} elems = {elapsed:?}");
    println!(
        "  ns/elem : {:.2}",
        elapsed.as_nanos() as f64 / ITERS as f64 / N as f64
    );
    println!("  checksum (anti-optimize): {checksum2}");
}
