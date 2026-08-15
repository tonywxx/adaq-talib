//! P3-2 A-class parallel PoC bench: midpoint / minmax / minmax_index / willr / stoch_f.
//! 用法 / Usage:
//!   cargo bench --bench parallel_poc                 # 仅串行参考
//!   cargo bench --bench parallel_poc --features parallel   # 串行 + 并行对照
//! 各函数两次运行的 ns/elem 之比即为并行加速比（多核 vs 串行单线程）。
//! The ns/elem ratio between the two runs is the parallel speedup (multi-core vs serial).
use adaq_talib::math_ops::{minmax_index_serial, minmax_serial};
use adaq_talib::momentum::{stoch_f_serial, willr_serial};
use adaq_talib::overlap::midpoint_serial;
#[cfg(feature = "parallel")]
use adaq_talib::math_ops::{minmax_index_parallel, minmax_parallel};
#[cfg(feature = "parallel")]
use adaq_talib::momentum::{stoch_f_parallel, willr_parallel};
#[cfg(feature = "parallel")]
use adaq_talib::overlap::{midpoint_parallel, midpoint};
use std::hint::black_box;
use std::time::Instant;

const N: usize = 200_000;
const PERIOD: usize = 5;
const RUNS: usize = 5;

fn lcg_input(n: usize, seed: u64) -> Vec<f64> {
    let mut x = seed;
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

fn report(name: &str, serial_ns: f64) {
    #[cfg(feature = "parallel")]
    {
        let _ = name;
        let _ = serial_ns;
    }
    #[cfg(not(feature = "parallel"))]
    {
        println!("{name:<22}: {serial_ns:8.3} ns/elem (serial reference)");
    }
}

fn main() {
    let data = lcg_input(N, 0x2545F491_4F6CDD1D);
    let high = lcg_input(N, 0xC2B2AE3D27D4EB4F);
    let low = lcg_input(N, 0x27D4EB4FC2B2AE3D);
    let close = lcg_input(N, 0x165667B19E3779F9);

    // ---- midpoint ----
    let _ = black_box(midpoint_serial(&data, PERIOD));
    let mp_s = median_ns(|| midpoint_serial(&data, PERIOD).unwrap());
    report("midpoint", mp_s);
    #[cfg(feature = "parallel")]
    {
        let mp_d = median_ns(|| midpoint(&data, PERIOD).unwrap());
        let mp_p = median_ns(|| midpoint_parallel(&data, PERIOD).unwrap());
        println!("midpoint             : serial {mp_s:8.3} | dispatch {mp_d:8.3} ({:.2}x) | parallel {mp_p:8.3} ({:.2}x)",
                 mp_s / mp_d, mp_s / mp_p);
    }

    // ---- minmax ----
    let _ = black_box(minmax_serial(&data, PERIOD));
    let mm_s = median_ns(|| minmax_serial(&data, PERIOD).unwrap().min);
    report("minmax(.min)", mm_s);
    #[cfg(feature = "parallel")]
    {
        let mm_p = median_ns(|| minmax_parallel(&data, PERIOD).unwrap().min);
        println!("minmax(.min)          : serial {mm_s:8.3} | parallel {mm_p:8.3} ({:.2}x)", mm_s / mm_p);
    }

    // ---- minmax_index ----
    let _ = black_box(minmax_index_serial(&data, PERIOD));
    let mi_s = median_ns(|| minmax_index_serial(&data, PERIOD).unwrap().min_idx);
    report("minmax_index(.min_idx)", mi_s);
    #[cfg(feature = "parallel")]
    {
        let mi_p = median_ns(|| minmax_index_parallel(&data, PERIOD).unwrap().min_idx);
        println!("minmax_index(.min_idx) : serial {mi_s:8.3} | parallel {mi_p:8.3} ({:.2}x)", mi_s / mi_p);
    }

    // ---- willr ----
    let _ = black_box(willr_serial(&high, &low, &close, PERIOD));
    let wr_s = median_ns(|| willr_serial(&high, &low, &close, PERIOD).unwrap());
    report("willr", wr_s);
    #[cfg(feature = "parallel")]
    {
        let wr_p = median_ns(|| willr_parallel(&high, &low, &close, PERIOD).unwrap());
        println!("willr                : serial {wr_s:8.3} | parallel {wr_p:8.3} ({:.2}x)", wr_s / wr_p);
    }

    // ---- stoch_f ----
    let _ = black_box(stoch_f_serial(&high, &low, &close, PERIOD, 3));
    let sf_s = median_ns(|| stoch_f_serial(&high, &low, &close, PERIOD, 3).unwrap().fast_k);
    report("stoch_f(.fast_k)", sf_s);
    #[cfg(feature = "parallel")]
    {
        let sf_p = median_ns(|| stoch_f_parallel(&high, &low, &close, PERIOD, 3).unwrap().fast_k);
        println!("stoch_f(.fast_k)      : serial {sf_s:8.3} | parallel {sf_p:8.3} ({:.2}x)", sf_s / sf_p);
    }
}
