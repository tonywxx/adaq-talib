//! 极值索引基准测速（Rust 侧，零依赖）。
//! Rolling-extreme-index benchmark (Rust side, dependency-free).
//!
//! 运行 / Run:
//! ```text
//! cargo bench --bench index_bench
//! ```
//! 对照原生 C（需系统安装 TA-Lib C 库）:
//! ```text
//! cargo bench --bench index_bench --features bench-c
//! ```

use adaq_talib::math_ops::{max_index, min_index, minmax_index};
use std::time::Instant;

const N: usize = 1_000_000;
const PERIOD: usize = 20;
const ITERS: usize = 20;

/// 确定性伪随机输入（LCG），与仓库其他 bench 同口径。
/// Deterministic pseudo-random input (LCG), same convention as the other benches.
fn sample(n: usize) -> Vec<f64> {
    let mut v = Vec::with_capacity(n);
    let mut x = 12345.0f64;
    for _ in 0..n {
        x = (x * 1103515245.0 + 12345.0) % 1e9;
        v.push(50.0 + (x / 1e9) * 10.0);
    }
    v
}

/// 重构前等价：朴素 O(n·period) 嵌套扫描（最左 tie-break），仅用于对照测速。
/// Pre-change equivalent: naïve O(n·period) nested scan (leftmost), reference only.
fn naive_max_index(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut out = vec![0.0_f64; n];
    if n < period {
        return out;
    }
    for i in (period - 1)..n {
        let mut best = values[i];
        let mut best_idx = i;
        for j in 1..period {
            let v = values[i - j];
            if v >= best {
                best = v;
                best_idx = i - j;
            }
        }
        out[i] = best_idx as f64;
    }
    out
}

fn ns_per_elem(elapsed: std::time::Duration) -> f64 {
    elapsed.as_nanos() as f64 / ITERS as f64 / N as f64
}

fn main() {
    let values = sample(N);

    // MAX_INDEX：单遍单调队列 O(n)（候选③：复用 core::rolling_extreme_index）。
    // MAX_INDEX: single-pass monotonic-queue O(n) (candidate ③).
    let start = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..ITERS {
        let out = max_index(&values, PERIOD).unwrap();
        checksum += out[out.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust MAX_INDEX (single-pass O(n), candidate ③): {ITERS} iters x {N} elems = {elapsed:?}");
    println!("  avg/call: {:?}", elapsed / ITERS as u32);
    println!("  ns/elem : {:.2}", ns_per_elem(elapsed));
    println!("  checksum (anti-optimize): {checksum}\n");

    // 重构前等价：朴素 O(n·period) 嵌套扫描。
    // Pre-change equivalent: naïve O(n·period) nested scan.
    let start = Instant::now();
    let mut checksum2 = 0.0;
    for _ in 0..ITERS {
        let out = naive_max_index(&values, PERIOD);
        checksum2 += out[out.len() - 1];
    }
    let elapsed = start.elapsed();
    println!("Rust MAX_INDEX (naive O(n·period), pre-change equivalent): {ITERS} iters x {N} elems = {elapsed:?}");
    println!("  avg/call: {:?}", elapsed / ITERS as u32);
    println!("  ns/elem : {:.2}", ns_per_elem(elapsed));
    println!("  checksum (anti-optimize): {checksum2}\n");

    // MIN_INDEX（单遍 O(n)）。
    // MIN_INDEX (single-pass O(n)).
    let start = Instant::now();
    let mut c3 = 0.0;
    for _ in 0..ITERS {
        let out = min_index(&values, PERIOD).unwrap();
        c3 += out[out.len() - 1];
    }
    let e = start.elapsed();
    println!("Rust MIN_INDEX (single-pass O(n)): ns/elem {:.2} (checksum {c3})\n", ns_per_elem(e));

    // MINMAX_INDEX（两次 O(n) 遍历）。
    // MINMAX_INDEX (two O(n) passes).
    let start = Instant::now();
    let mut c4 = 0.0;
    for _ in 0..ITERS {
        let out = minmax_index(&values, PERIOD).unwrap();
        c4 += out.max_idx[out.max_idx.len() - 1];
    }
    let e = start.elapsed();
    println!("Rust MINMAX_INDEX (two-pass O(n)): ns/elem {:.2} (checksum {c4})\n", ns_per_elem(e));

    #[cfg(feature = "bench-c")]
    run_c_bench(&values);
}

#[cfg(feature = "bench-c")]
fn run_c_bench(values: &[f64]) {
    unsafe {
        unsafe extern "C" {
            fn TA_Initialize() -> i32;
            fn TA_Shutdown() -> i32;
            fn TA_MAXINDEX(
                start_idx: i32,
                end_idx: i32,
                in_real: *const f64,
                opt_in_time_period: i32,
                out_beg_idx: *mut i32,
                out_nb_element: *mut i32,
                out_real: *mut f64,
            ) -> i32;
            fn TA_MININDEX(
                start_idx: i32,
                end_idx: i32,
                in_real: *const f64,
                opt_in_time_period: i32,
                out_beg_idx: *mut i32,
                out_nb_element: *mut i32,
                out_real: *mut f64,
            ) -> i32;
            fn TA_MINMAXINDEX(
                start_idx: i32,
                end_idx: i32,
                in_real: *const f64,
                opt_in_time_period: i32,
                out_beg_idx: *mut i32,
                out_nb_element: *mut i32,
                out_min_idx: *mut f64,
                out_max_idx: *mut f64,
            ) -> i32;
        }
        assert_eq!(TA_Initialize(), 0, "TA_Initialize failed");
        let n = values.len() as i32;

        let mut beg = 0i32;
        let mut nb = 0i32;
        let mut out = vec![0.0f64; values.len()];
        let start = Instant::now();
        let mut cs = 0.0;
        for _ in 0..ITERS {
            beg = 0;
            nb = 0;
            assert_eq!(
                TA_MAXINDEX(
                    0,
                    n - 1,
                    values.as_ptr(),
                    PERIOD as i32,
                    &mut beg,
                    &mut nb,
                    out.as_mut_ptr()
                ),
                0
            );
            cs += out[(nb - 1) as usize];
        }
        println!(
            "C MAXINDEX (native): ns/elem {:.2} (checksum {cs})\n",
            ns_per_elem(start.elapsed())
        );

        let start = Instant::now();
        let mut cs = 0.0;
        for _ in 0..ITERS {
            beg = 0;
            nb = 0;
            assert_eq!(
                TA_MININDEX(
                    0,
                    n - 1,
                    values.as_ptr(),
                    PERIOD as i32,
                    &mut beg,
                    &mut nb,
                    out.as_mut_ptr()
                ),
                0
            );
            cs += out[(nb - 1) as usize];
        }
        println!(
            "C MININDEX (native): ns/elem {:.2} (checksum {cs})\n",
            ns_per_elem(start.elapsed())
        );

        let mut out_min = vec![0.0f64; values.len()];
        let mut out_max = vec![0.0f64; values.len()];
        let start = Instant::now();
        let mut cs = 0.0;
        for _ in 0..ITERS {
            beg = 0;
            nb = 0;
            assert_eq!(
                TA_MINMAXINDEX(
                    0,
                    n - 1,
                    values.as_ptr(),
                    PERIOD as i32,
                    &mut beg,
                    &mut nb,
                    out_min.as_mut_ptr(),
                    out_max.as_mut_ptr()
                ),
                0
            );
            cs += out_max[(nb - 1) as usize];
        }
        println!(
            "C MINMAXINDEX (native): ns/elem {:.2} (checksum {cs})",
            ns_per_elem(start.elapsed())
        );

        TA_Shutdown();
    }
}
