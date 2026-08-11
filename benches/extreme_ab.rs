//! A/B: VecDeque monotonic queue vs ring-buffer monotonic queue for rolling min/max.
//! Self-contained so it does not depend on the crate's current primitive.
//! Run: cargo bench --bench extreme_ab

use std::collections::VecDeque;
use std::time::Instant;

const N: usize = 1_000_000;
const PERIOD: usize = 20;
const ITERS: usize = 30;

fn sample(n: usize) -> Vec<f64> {
    let mut v = Vec::with_capacity(n);
    let mut x = 12345.0f64;
    for _ in 0..n {
        x = (x * 1103515245.0 + 12345.0) % 1e9;
        v.push(50.0 + (x / 1e9) * 10.0);
    }
    v
}

// ---- VecDeque version ----
fn rolling_min_vecdeque(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if n < period { return out; }
    let mut dq: VecDeque<usize> = VecDeque::with_capacity(period);
    for i in 0..n {
        while let Some(&f) = dq.front() {
            if f + period <= i { dq.pop_front(); } else { break; }
        }
        while let Some(&b) = dq.back() {
            if values[b] >= values[i] { dq.pop_back(); } else { break; }
        }
        dq.push_back(i);
        if i >= period - 1 { out[i] = values[*dq.front().unwrap()]; }
    }
    out
}

// ---- Ring-buffer version ----
struct MonoQueue { buf: Vec<usize>, mask: usize, head: usize, tail: usize, len: usize }
impl MonoQueue {
    fn with_capacity(period: usize) -> Self {
        let cap = period.next_power_of_two().max(1);
        MonoQueue { buf: vec![0; cap], mask: cap - 1, head: 0, tail: 0, len: 0 }
    }
    #[inline] fn push_back(&mut self, v: usize) { self.buf[self.tail & self.mask] = v; self.tail += 1; self.len += 1; }
    #[inline] fn pop_back(&mut self) { self.tail -= 1; self.len -= 1; }
    #[inline] fn pop_front(&mut self) { self.head += 1; self.len -= 1; }
    #[inline] fn front(&self) -> usize { self.buf[self.head & self.mask] }
    #[inline] fn back(&self) -> usize { self.buf[(self.tail - 1) & self.mask] }
    #[inline] fn is_empty(&self) -> bool { self.len == 0 }
}
fn rolling_min_ring(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if n < period { return out; }
    let mut dq = MonoQueue::with_capacity(period);
    for i in 0..n {
        while !dq.is_empty() && dq.front() + period <= i { dq.pop_front(); }
        while !dq.is_empty() && values[dq.back()] >= values[i] { dq.pop_back(); }
        dq.push_back(i);
        if i >= period - 1 { out[i] = values[dq.front()]; }
    }
    out
}

fn main() {
    let values = sample(N);
    // warmup
    let _ = rolling_min_vecdeque(&values, PERIOD);
    let _ = rolling_min_ring(&values, PERIOD);

    let mut cs_v = 0.0; let mut t_v = 0u128;
    for _ in 0..ITERS {
        let s = Instant::now();
        let o = rolling_min_vecdeque(&values, PERIOD);
        t_v += s.elapsed().as_nanos();
        cs_v += o[o.len()-1];
    }
    let mut cs_r = 0.0; let mut t_r = 0u128;
    for _ in 0..ITERS {
        let s = Instant::now();
        let o = rolling_min_ring(&values, PERIOD);
        t_r += s.elapsed().as_nanos();
        cs_r += o[o.len()-1];
    }
    let nv = t_v as f64 / ITERS as f64 / N as f64;
    let nr = t_r as f64 / ITERS as f64 / N as f64;
    println!("VecDeque  min ns/elem = {:.3}  (cs={:.3})", nv, cs_v);
    println!("RingBuf   min ns/elem = {:.3}  (cs={:.3})", nr, cs_r);
    println!("ring/vecdeque ratio = {:.3}  (same checksum? {})", nr/nv, (cs_v-cs_r).abs() < 1e-6);
}
