//! Focused benchmark for the Wilder-smoothed momentum family (candidate② seam).
//! Mirrors benches/cdl_bench.rs: N=100_000, LCG inputs, median over ROUNDS to cancel
//! single-`Instant` noise (ADR 0010/0011). Run: cargo bench --bench momentum_wilder_bench
//!
//! Writes `momentum_wilder_results.csv` (name, adaq_ns_per_elem). Use before/after a
//! deepening to compute median Δ and apply the ±5% gate.
#![allow(dead_code, unused_imports)]

use adaq_talib::error::TaError;
use adaq_talib::momentum::{
    adx_with_output, adxr_with_output, cmo_with_output, dx_with_output, minus_di_with_output,
    plus_di_with_output, rsi_with_output,
};
use std::time::Instant;

const N: usize = 100_000;
const BUDGET_NS: u128 = 400_000_000;
const ROUNDS: usize = 9;

fn make_1d() -> Vec<f64> {
    let mut x = 98765.0f64;
    let mut v = Vec::with_capacity(N);
    let mut prev = 100.0f64;
    for _ in 0..N {
        x = (x * 1103515245.0 + 12345.0) % 1e9;
        prev += (x / 1e9 - 0.5) * 2.0; // 随机游走，避免平坦序列
        v.push(prev);
    }
    v
}

fn make_ohlc() -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut x = 24680.0f64;
    let mut high = Vec::with_capacity(N);
    let mut low = Vec::with_capacity(N);
    let mut close = Vec::with_capacity(N);
    for _ in 0..N {
        x = (x * 1103515245.0 + 12345.0) % 1e9;
        let base = 50.0 + (x / 1e9) * 10.0;
        close.push(base);
        high.push(base + 0.5);
        low.push(base - 0.5);
    }
    (high, low, close)
}

fn measure_1d(
    name: &str,
    f: fn(&[f64], usize, &mut [f64]) -> Result<(), TaError>,
    data: &[f64],
    period: usize,
) -> f64 {
    let mut out = vec![0.0f64; N];
    f(data, period, &mut out).unwrap();
    let mut samples = Vec::with_capacity(ROUNDS);
    for _ in 0..ROUNDS {
        let t0 = Instant::now();
        for _ in 0..3 {
            let _ = f(data, period, &mut out).unwrap();
        }
        let per = t0.elapsed().as_nanos().max(1);
        let iters = ((BUDGET_NS / per) as usize).clamp(10, 400);
        let start = Instant::now();
        let mut ack = 0.0f64;
        for _ in 0..iters {
            let _ = f(data, period, &mut out).unwrap();
            ack += out[N - 1];
        }
        let _ = ack;
        samples.push(start.elapsed().as_nanos() as f64 / (iters as f64 * N as f64));
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = samples[ROUNDS / 2];
    println!("{} {:.4}", name, med);
    med
}

fn measure_3d(
    name: &str,
    f: fn(&[f64], &[f64], &[f64], usize, &mut [f64]) -> Result<(), TaError>,
    h: &[f64],
    l: &[f64],
    c: &[f64],
    period: usize,
) -> f64 {
    let mut out = vec![0.0f64; N];
    f(h, l, c, period, &mut out).unwrap();
    let mut samples = Vec::with_capacity(ROUNDS);
    for _ in 0..ROUNDS {
        let t0 = Instant::now();
        for _ in 0..3 {
            let _ = f(h, l, c, period, &mut out).unwrap();
        }
        let per = t0.elapsed().as_nanos().max(1);
        let iters = ((BUDGET_NS / per) as usize).clamp(10, 400);
        let start = Instant::now();
        let mut ack = 0.0f64;
        for _ in 0..iters {
            let _ = f(h, l, c, period, &mut out).unwrap();
            ack += out[N - 1];
        }
        let _ = ack;
        samples.push(start.elapsed().as_nanos() as f64 / (iters as f64 * N as f64));
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = samples[ROUNDS / 2];
    println!("{} {:.4}", name, med);
    med
}

fn main() {
    let v = make_1d();
    let (h, l, c) = make_ohlc();
    let p = 14usize;
    let mut csv = String::from("name,adaq_ns_per_elem\n");
    for (name, ns) in [
        ("rsi", measure_1d("rsi", rsi_with_output, &v, p)),
        ("cmo", measure_1d("cmo", cmo_with_output, &v, p)),
        (
            "plus_di",
            measure_3d("plus_di", plus_di_with_output, &h, &l, &c, p),
        ),
        (
            "minus_di",
            measure_3d("minus_di", minus_di_with_output, &h, &l, &c, p),
        ),
        ("dx", measure_3d("dx", dx_with_output, &h, &l, &c, p)),
        ("adx", measure_3d("adx", adx_with_output, &h, &l, &c, p)),
        (
            "adxr",
            measure_3d("adxr", adxr_with_output, &h, &l, &c, p),
        ),
    ] {
        csv.push_str(&format!("{},{:.4}\n", name, ns));
    }
    let _ = std::fs::write("momentum_wilder_results.csv", &csv);
    println!("WROTE momentum_wilder_results.csv");
}
