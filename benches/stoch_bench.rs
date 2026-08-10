//! STOCH 基准测速（Rust 侧，零依赖）。/ STOCH benchmark (Rust side, dependency-free).
//!
//! 运行 / Run:
//! ```text
//! cargo bench --bench stoch_bench
//! ```

use adaq_talib::momentum::{stoch, stoch_with_output, Stoch};
use std::time::Instant;

const N: usize = 1_000_000;
const FAST_K: usize = 20;
const SLOW_K: usize = 3;
const SLOW_D: usize = 3;
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
        let out = stoch(&high, &low, &close, FAST_K, SLOW_K, SLOW_D).unwrap();
        checksum += out.slow_k[out.slow_k.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust STOCH:  {ITERS} iters x {N} elems = {elapsed:?}");
    println!("  avg/call: {:?}", elapsed / ITERS as u32);
    println!(
        "  ns/elem : {:.2}",
        elapsed.as_nanos() as f64 / ITERS as f64 / N as f64
    );
    println!("  checksum (anti-optimize): {checksum}\n");

    let mut buf = Stoch {
        slow_k: vec![f64::NAN; N],
        slow_d: vec![f64::NAN; N],
    };
    let start = Instant::now();
    let mut checksum2 = 0.0;
    for _ in 0..ITERS {
        stoch_with_output(&high, &low, &close, FAST_K, SLOW_K, SLOW_D, &mut buf).unwrap();
        checksum2 += buf.slow_k[buf.slow_k.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust STOCH(_with_output):  {ITERS} iters x {N} elems = {elapsed:?}");
    println!(
        "  ns/elem : {:.2}",
        elapsed.as_nanos() as f64 / ITERS as f64 / N as f64
    );
    println!("  checksum (anti-optimize): {checksum2}");
}
