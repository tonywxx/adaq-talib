//! 多核并行分块（默认关闭，需 `parallel` feature）。仅用于可重叠播种的窗口类指标。
//!
//! Multi-core parallel chunking (default-off, requires the `parallel` feature). Intended only
//! for window-based indicators that admit overlap seeding (e.g. the rolling min/max monotonic
//! deques used by `midpoint` / `minmax`).
#![cfg(feature = "parallel")]

use std::thread;

/// 将长度为 `n` 的输出 `out` 按 `num_cpus` 切分为连续分块并行计算。
///
/// Split `out` (length `n`) into `num_cpus` contiguous chunks and compute them in parallel.
///
/// - 每块除首块外，以 `overlap` 个**前导元素**与其前一块重叠，用于把单调双端队列（或其它
///   仅依赖有限前窗的状态机）的状态正确“播种”到分块边界——这样每块复用的 `worker`
///   内核与串行路径逐字节一致，输出与原生串行逐项 1:1（容差见 ADR 0005）。
/// - 每块在独立线程中执行 `worker(start, end)`，须计算扩展区间 `[start, end)` 的**全量**
///   输出（长度 `end - start`）；本函数随后仅保留其“自有区间” `[cs, ce)` 写回 `out`
///   （重叠部分被相邻块覆盖为相同值，无数据竞争）。
/// - `worker` 必须 `Sync`（跨线程共享），且不修改任何被捕获状态（纯函数式）。
///
/// Each chunk (except the first) overlaps the previous one by `overlap` **leading elements** to
/// correctly seed the monotonic-deque (or any finite-prefix state machine) state at the chunk
/// boundary — so the reused `worker` kernel is bit-for-bit identical to the serial path and the
/// output matches native serial 1:1 (ADR 0005 tolerance). Each chunk runs `worker(start, end)`
/// in its own thread, which must compute the FULL extended output for `[start, end)` (length
/// `end - start`); only its owned range `[cs, ce)` is written back to `out` (the overlap region
/// is overwritten by the neighbour with the same value — no data race on the final result).
///
/// 单核或数据量过小时直接串行，避免线程开销反而变慢。
/// Falls back to serial for a single core or tiny input to avoid thread overhead.
pub(crate) fn parallel_index_map(
    n: usize,
    overlap: usize,
    out: &mut [f64],
    worker: impl Fn(usize, usize) -> Vec<f64> + Sync,
) {
    let ncpus = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    if ncpus <= 1 || n < 8192 {
        let full = worker(0, n);
        out.copy_from_slice(&full);
        return;
    }
    let chunk = (n + ncpus - 1) / ncpus;
    let mut specs: Vec<(usize, usize, usize)> = Vec::new();
    for c in 0..ncpus {
        let cs = c * chunk;
        let ce = ((c + 1) * chunk).min(n);
        if cs >= ce {
            continue;
        }
        // 首块无重叠；其余块向前重叠 `overlap` 个元素以播种前窗状态。
        // First chunk: no overlap. Others overlap `overlap` leading elements to seed prefix state.
        let start = if c == 0 { 0 } else { cs.saturating_sub(overlap) };
        specs.push((cs, ce, start));
    }
    let results: Vec<(usize, usize, usize, Vec<f64>)> = thread::scope(|s| {
        let mut handles = Vec::with_capacity(specs.len());
        for (cs, ce, start) in specs {
            // `worker` 为 `Sync`，跨线程共享其引用（Copy 的 `&F`，不移动本体），
            // `start/ce/cs` 为 Copy，随 `move` 闭包按值进入各线程。
            // `worker` is `Sync`; share a (Copy) reference to it across threads instead of moving
            // the body. `start/ce/cs` are `Copy` and move into each thread by value.
            let w = &worker;
            let h = s.spawn(move || {
                let local = w(start, ce);
                (cs, ce, start, local)
            });
            handles.push(h);
        }
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect()
    });
    for (cs, ce, start, local) in results {
        // 仅写回自有区间 [cs, ce)；局部缓冲中对应偏移为 [cs-start, ce-start)。
        // Write back only the owned range [cs, ce); its offset in the local buffer is [cs-start, ce-start).
        let owned = &local[(cs - start)..(ce - start)];
        out[cs..ce].copy_from_slice(owned);
    }
}

/// 双输出变体（如 `minmax` / `minmax_index` / `stoch_f` 的两路极值或 fastK/fastD）。
/// 语义与 [`parallel_index_map`] 完全一致：`worker(start, end)` 返回扩展区间 `[start, end)`
/// 的**两个**全量输出 `(a, b)`，各自仅写回自有区间到 `out_a` / `out_b`。单遍内核一次算出
/// 两路，避免对双队列做两次独立并行扫描（相比分两次调用 [`parallel_index_map`] 省一半计算）。
///
/// Two-output variant (e.g. the two extremes of `minmax`/`minmax_index` or fastK/fastD of
/// `stoch_f`). Identical contract to [`parallel_index_map`]: `worker(start, end)` returns the
/// TWO full outputs `(a, b)` for the extended range `[start, end)`, each writing back only its
/// owned range to `out_a` / `out_b`. The kernel computes both streams in one pass, avoiding two
/// independent parallel scans of the dual deque.
pub(crate) fn parallel_index_map_2(
    n: usize,
    overlap: usize,
    out_a: &mut [f64],
    out_b: &mut [f64],
    worker: impl Fn(usize, usize) -> (Vec<f64>, Vec<f64>) + Sync,
) {
    let ncpus = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    if ncpus <= 1 || n < 8192 {
        let (a, b) = worker(0, n);
        out_a.copy_from_slice(&a);
        out_b.copy_from_slice(&b);
        return;
    }
    let chunk = (n + ncpus - 1) / ncpus;
    let mut specs: Vec<(usize, usize, usize)> = Vec::new();
    for c in 0..ncpus {
        let cs = c * chunk;
        let ce = ((c + 1) * chunk).min(n);
        if cs >= ce {
            continue;
        }
        let start = if c == 0 { 0 } else { cs.saturating_sub(overlap) };
        specs.push((cs, ce, start));
    }
    let results: Vec<(usize, usize, usize, Vec<f64>, Vec<f64>)> = thread::scope(|s| {
        let mut handles = Vec::with_capacity(specs.len());
        for (cs, ce, start) in specs {
            let w = &worker;
            let h = s.spawn(move || {
                let (a, b) = w(start, ce);
                (cs, ce, start, a, b)
            });
            handles.push(h);
        }
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect()
    });
    for (cs, ce, start, a, b) in results {
        out_a[cs..ce].copy_from_slice(&a[(cs - start)..(ce - start)]);
        out_b[cs..ce].copy_from_slice(&b[(cs - start)..(ce - start)]);
    }
}
