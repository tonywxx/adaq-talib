//! Focused benchmark for the 61 candlestick (cdl_*) pattern functions.
//! Mirrors benches/all161_bench.rs data-gen + harness (N=100_000, same LCG inputs,
//! same per-elem timing) so ns/elem are directly comparable to the all161 baseline.
//! Run: cargo bench --bench cdl_bench
#![allow(dead_code, unused_imports)]

use adaq_talib::error::TaError;
use std::time::Instant;

const N: usize = 100_000;
const BUDGET_NS: u128 = 400_000_000;

type CdlFn = fn(&[f64], &[f64], &[f64], &[f64], &mut [f64]) -> Result<(), TaError>;

fn make_inputs() -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut x = 12345.0f64;
    let mut open = Vec::with_capacity(N);
    let mut high = Vec::with_capacity(N);
    let mut low = Vec::with_capacity(N);
    let mut close = Vec::with_capacity(N);
    for _ in 0..N {
        x = (x * 1103515245.0 + 12345.0) % 1e9;
        let base = 50.0 + (x / 1e9) * 10.0;
        let c = base;
        close.push(c);
        high.push(c + 0.5);
        low.push(c - 0.5);
        open.push(c + 0.1);
    }
    (open, high, low, close)
}

fn bench_one(_name: &str, f: CdlFn, o: &[f64], h: &[f64], l: &[f64], c: &[f64]) -> f64 {
    let mut out = vec![0.0f64; N];
    f(o, h, l, c, &mut out).unwrap();
    let t0 = Instant::now();
    for _ in 0..3 { let _ = f(o, h, l, c, &mut out).unwrap(); }
    let per = t0.elapsed().as_nanos().max(1);
    let iters = ((BUDGET_NS / per) as usize).clamp(10, 400);
    let start = Instant::now();
    let mut ack = 0.0f64;
    for _ in 0..iters { let _ = f(o, h, l, c, &mut out).unwrap(); ack += out[N - 1]; }
    let _ = ack;
    start.elapsed().as_nanos() as f64 / (iters as f64 * N as f64)
}

fn main() {
    let (open, high, low, close) = make_inputs();
    let funcs: Vec<(&str, CdlFn)> = vec![
        ("cdl_2crows", adaq_talib::pattern::cdl_2crows_with_output),
        ("cdl_3blackcrows", adaq_talib::pattern::cdl_3blackcrows_with_output),
        ("cdl_3inside", adaq_talib::pattern::cdl_3inside_with_output),
        ("cdl_3linestrike", adaq_talib::pattern::cdl_3linestrike_with_output),
        ("cdl_3outside", adaq_talib::pattern::cdl_3outside_with_output),
        ("cdl_3starsinsouth", adaq_talib::pattern::cdl_3starsinsouth_with_output),
        ("cdl_3whitesoldiers", adaq_talib::pattern::cdl_3whitesoldiers_with_output),
        ("cdl_abandonedbaby", adaq_talib::pattern::cdl_abandonedbaby_with_output),
        ("cdl_advanceblock", adaq_talib::pattern::cdl_advanceblock_with_output),
        ("cdl_belthold", adaq_talib::pattern::cdl_belthold_with_output),
        ("cdl_breakaway", adaq_talib::pattern::cdl_breakaway_with_output),
        ("cdl_closingmarubozu", adaq_talib::pattern::cdl_closingmarubozu_with_output),
        ("cdl_concealbabyswall", adaq_talib::pattern::cdl_concealbabyswall_with_output),
        ("cdl_counterattack", adaq_talib::pattern::cdl_counterattack_with_output),
        ("cdl_darkcloudcover", adaq_talib::pattern::cdl_darkcloudcover_with_output),
        ("cdl_doji", adaq_talib::pattern::cdl_doji_with_output),
        ("cdl_dojistar", adaq_talib::pattern::cdl_dojistar_with_output),
        ("cdl_dragonflydoji", adaq_talib::pattern::cdl_dragonflydoji_with_output),
        ("cdl_engulfing", adaq_talib::pattern::cdl_engulfing_with_output),
        ("cdl_eveningdojistar", adaq_talib::pattern::cdl_eveningdojistar_with_output),
        ("cdl_eveningstar", adaq_talib::pattern::cdl_eveningstar_with_output),
        ("cdl_gapsidesidewhite", adaq_talib::pattern::cdl_gapsidesidewhite_with_output),
        ("cdl_gravestonedoji", adaq_talib::pattern::cdl_gravestonedoji_with_output),
        ("cdl_hammer", adaq_talib::pattern::cdl_hammer_with_output),
        ("cdl_hangingman", adaq_talib::pattern::cdl_hangingman_with_output),
        ("cdl_harami", adaq_talib::pattern::cdl_harami_with_output),
        ("cdl_haramicross", adaq_talib::pattern::cdl_haramicross_with_output),
        ("cdl_highwave", adaq_talib::pattern::cdl_highwave_with_output),
        ("cdl_hikkake", adaq_talib::pattern::cdl_hikkake_with_output),
        ("cdl_hikkakemod", adaq_talib::pattern::cdl_hikkakemod_with_output),
        ("cdl_homingpigeon", adaq_talib::pattern::cdl_homingpigeon_with_output),
        ("cdl_identical3crows", adaq_talib::pattern::cdl_identical3crows_with_output),
        ("cdl_inneck", adaq_talib::pattern::cdl_inneck_with_output),
        ("cdl_invertedhammer", adaq_talib::pattern::cdl_invertedhammer_with_output),
        ("cdl_kicking", adaq_talib::pattern::cdl_kicking_with_output),
        ("cdl_kickingbylength", adaq_talib::pattern::cdl_kickingbylength_with_output),
        ("cdl_ladderbottom", adaq_talib::pattern::cdl_ladderbottom_with_output),
        ("cdl_longleggeddoji", adaq_talib::pattern::cdl_longleggeddoji_with_output),
        ("cdl_longline", adaq_talib::pattern::cdl_longline_with_output),
        ("cdl_marubozu", adaq_talib::pattern::cdl_marubozu_with_output),
        ("cdl_matchinglow", adaq_talib::pattern::cdl_matchinglow_with_output),
        ("cdl_mathold", adaq_talib::pattern::cdl_mathold_with_output),
        ("cdl_morningdojistar", adaq_talib::pattern::cdl_morningdojistar_with_output),
        ("cdl_morningstar", adaq_talib::pattern::cdl_morningstar_with_output),
        ("cdl_onneck", adaq_talib::pattern::cdl_onneck_with_output),
        ("cdl_piercing", adaq_talib::pattern::cdl_piercing_with_output),
        ("cdl_rickshawman", adaq_talib::pattern::cdl_rickshawman_with_output),
        ("cdl_risefall3methods", adaq_talib::pattern::cdl_risefall3methods_with_output),
        ("cdl_separatinglines", adaq_talib::pattern::cdl_separatinglines_with_output),
        ("cdl_shootingstar", adaq_talib::pattern::cdl_shootingstar_with_output),
        ("cdl_shortline", adaq_talib::pattern::cdl_shortline_with_output),
        ("cdl_spinningtop", adaq_talib::pattern::cdl_spinningtop_with_output),
        ("cdl_stalledpattern", adaq_talib::pattern::cdl_stalledpattern_with_output),
        ("cdl_sticksandwich", adaq_talib::pattern::cdl_sticksandwich_with_output),
        ("cdl_takuri", adaq_talib::pattern::cdl_takuri_with_output),
        ("cdl_tasukigap", adaq_talib::pattern::cdl_tasukigap_with_output),
        ("cdl_thrusting", adaq_talib::pattern::cdl_thrusting_with_output),
        ("cdl_tristar", adaq_talib::pattern::cdl_tristar_with_output),
        ("cdl_unique3river", adaq_talib::pattern::cdl_unique3river_with_output),
        ("cdl_upsidegap2crows", adaq_talib::pattern::cdl_upsidegap2crows_with_output),
        ("cdl_xsidegap3methods", adaq_talib::pattern::cdl_xsidegap3methods_with_output),
    ];
    let mut csv = String::from("name,adaq_ns_per_elem\n");
    for (name, f) in &funcs {
        let ns = bench_one(name, *f, &open, &high, &low, &close);
        println!("{} {:.4}", name, ns);
        csv.push_str(&format!("{},{:.4}\n", name, ns));
    }
    let _ = std::fs::write("cdl_results.csv", &csv);
    println!("WROTE cdl_results.csv ({} funcs)", funcs.len());
}
