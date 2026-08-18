//! 滚动窗口极值（最大/最小及其索引），基于环形缓冲单调队列 O(n)。
//!
//! Rolling-window extremes (max / min and their indices), backed by an O(n)
//! ring-buffer monotonic queue.

/// 定长环形缓冲双端队列（容量固定为 ≥ `period` 的 2 的幂），用于 O(1) 单调队列极值。
///
/// Fixed-capacity ring-buffer deque (capacity = next power of two ≥ `period`) backing the
/// monotonic-queue extremes. Compared with `std::collections::VecDeque` it avoids the internal
/// offset arithmetic / bounds checks and any reallocation, giving the compiler a tighter
/// inlinable hot loop (the index is masked with `cap - 1`, a power-of-two, so no division).
pub(crate) struct MonoQueue {
    buf: Vec<usize>,
    mask: usize, // capacity - 1 (capacity is a power of two)
    head: usize, // position of the front element (masked on access)
    tail: usize, // position just past the back element (masked on access)
    len: usize,
}

impl MonoQueue {
    #[inline]
    pub(crate) fn with_capacity(period: usize) -> Self {
        let cap = period.next_power_of_two().max(1);
        MonoQueue {
            buf: vec![0usize; cap],
            mask: cap - 1,
            head: 0,
            tail: 0,
            len: 0,
        }
    }
    #[inline]
    pub(crate) fn push_back(&mut self, v: usize) {
        self.buf[self.tail & self.mask] = v;
        self.tail += 1;
        self.len += 1;
    }
    #[inline]
    pub(crate) fn pop_back(&mut self) {
        self.tail -= 1;
        self.len -= 1;
    }
    #[inline]
    pub(crate) fn pop_front(&mut self) {
        self.head += 1;
        self.len -= 1;
    }
    #[inline]
    pub(crate) fn front(&self) -> usize {
        self.buf[self.head & self.mask]
    }
    #[inline]
    pub(crate) fn back(&self) -> usize {
        self.buf[(self.tail - 1) & self.mask]
    }
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// 滚动窗口极值（最大或最小），与输入等长的全索引向量，前导 `period-1` 为 [`f64::NAN`]。
///
/// Rolling-window extreme (max or min), a full-indexed vector with the same length as the
/// input; the leading `period - 1` positions are [`f64::NAN`].
///
/// 采用 **环形缓冲单调队列** O(n) 实现（P2-2，ADR 0010）：以 [`MonoQueue`] 维护窗口内的单调候选，
/// 每元素入队/出队均摊 O(1)。并列极值取**窗口内最右**者（弹出 `<=`/`>=` 候选），与朴素扫描
/// [`rolling_extreme_naive`] 的 tie-break 完全一致，数值逐项相等（零偏差，ADR 0005）。
///
/// Uses an O(n) ring-buffer monotonic-queue (P2-2, ADR 0010): [`MonoQueue`] maintains the
/// monotonic candidates inside the window so each element is enqueued/dequeued in amortized
/// O(1). Ties resolve to the **rightmost** occurrence in the window (popping `<=`/`>=`
/// candidates), which matches the tie-break of the naïve [`rolling_extreme_naive`] scan exactly
/// — bit-for-bit equal (ADR 0005).
#[inline]
pub(crate) fn rolling_extreme(values: &[f64], period: usize, take_max: bool) -> Vec<f64> {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if n < period {
        return out;
    }
    let mut dq = MonoQueue::with_capacity(period);
    for i in 0..n {
        // 移除已滑出窗口的最左候选 / drop the leftmost candidate that left the window
        while !dq.is_empty() && dq.front() + period <= i {
            dq.pop_front();
        }
        if take_max {
            // 弹出 <= 候选者（含相等），使队首为窗口最右最大值
            // pop <= candidates (incl. equal) so the front is the rightmost max
            while !dq.is_empty() && values[dq.back()] <= values[i] {
                dq.pop_back();
            }
        } else {
            // 弹出 >= 候选者，使队首为窗口最右最小值
            // pop >= candidates so the front is the rightmost min
            while !dq.is_empty() && values[dq.back()] >= values[i] {
                dq.pop_back();
            }
        }
        dq.push_back(i);
        if i >= period - 1 {
            out[i] = values[dq.front()];
        }
    }
    out
}

/// 朴素 O(n·period) 窗口极值扫描，仅作为 [`rolling_extreme`] 的单元测试对照（非热路径）。
///
/// Naïve O(n·period) window-scan extreme — used only as the reference in unit tests for
/// [`rolling_extreme`]; not on any hot path.
#[allow(dead_code)]
pub(crate) fn rolling_extreme_naive(values: &[f64], period: usize, take_max: bool) -> Vec<f64> {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if n < period {
        return out;
    }
    for i in (period - 1)..n {
        let mut acc = values[i];
        for j in 1..period {
            let v = values[i - j];
            if take_max {
                if v > acc {
                    acc = v;
                }
            } else if v < acc {
                acc = v;
            }
        }
        out[i] = acc;
    }
    out
}

/// 滚动窗口的最大与最小，**同一次遍历** O(n)（用于 `MIDPOINT` 的 `(max+min)/2`）。
///
/// Rolling max **and** min in a single O(n) pass — for `MIDPOINT`'s `(max+min)/2`. Two
/// monotonic deques (decreasing for max, increasing for min) advance together; ties resolve to
/// the rightmost extreme (same `<=`/`>=` pop rule as [`rolling_extreme`]), so the per-element
/// `max`/`min` equal the separate calls exactly (ADR 0005).
///
/// 对应 TA-Lib `TA_MIDPOINT` 内部 `MINMAXINDEX` 的单遍双队列思路，将 `midpoint` 的两次窗口
/// 扫描合并为一次，规避重复遍历开销。
///
/// Mirrors TA-Lib `TA_MIDPOINT`'s internal `MINMAXINDEX` single-pass dual-deque approach, merging
/// `midpoint`'s two window scans into one to avoid the redundant traversal.
#[inline]
pub(crate) fn rolling_minmax(values: &[f64], period: usize) -> (Vec<f64>, Vec<f64>) {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut max_out = vec![f64::NAN; n];
    let mut min_out = vec![f64::NAN; n];
    if n < period {
        return (max_out, min_out);
    }
    let mut max_dq = MonoQueue::with_capacity(period);
    let mut min_dq = MonoQueue::with_capacity(period);
    for i in 0..n {
        // 移除已滑出窗口的最左候选（两个队列同步）。/ drop out-of-window leftmost (both deques).
        while !max_dq.is_empty() && max_dq.front() + period <= i {
            max_dq.pop_front();
        }
        while !min_dq.is_empty() && min_dq.front() + period <= i {
            min_dq.pop_front();
        }
        // 最大值队列（递减）：弹出 <= 候选者 / max deque (decreasing): pop <= candidates
        while !max_dq.is_empty() && values[max_dq.back()] <= values[i] {
            max_dq.pop_back();
        }
        // 最小值队列（递增）：弹出 >= 候选者 / min deque (increasing): pop >= candidates
        while !min_dq.is_empty() && values[min_dq.back()] >= values[i] {
            min_dq.pop_back();
        }
        max_dq.push_back(i);
        min_dq.push_back(i);
        if i >= period - 1 {
            max_out[i] = values[max_dq.front()];
            min_out[i] = values[min_dq.front()];
        }
    }
    (max_out, min_out)
}

/// 滚动窗口极值**索引**，单遍单调队列 O(n)（平局取最左 / leftmost）。
///
/// Rolling-extreme **index** in a single O(n) monotonic-queue pass (leftmost on ties). Returns
/// the absolute (0-based) position of the window extreme (max when `take_max`, min otherwise);
/// the leading `period - 1` positions are `NaN`. TA-Lib's `TA_MAXINDEX` / `TA_MININDEX` /
/// `TA_MINMAXINDEX` never write those positions (the caller treats them as unset), so `NaN`
/// is the faithful representation that maps to "no value" in the engine.
///
/// 与 [`rolling_extreme`]（值变体，最右 tie-break）互为镜像：此处弹出条件用**严格** `<` / `>`
/// 而非 `<=` / `>=`，使并列极值保留更靠左（更小索引）的候选，从而复刻 TA-Lib 索引变体的
/// 最左 tie-break（见 `math_ops::max_index` / `min_index` 文档）。在有限输入上与朴素
/// `O(n·period)` 扫描逐项相等（零偏差，ADR 0005）。
///
/// Mirrors [`rolling_extreme`] (the value variant, rightmost tie-break): here the pop condition
/// is the **strict** `<` / `>` (not `<=` / `>=`), so equal extremes keep the leftmost (smaller
/// index) candidate — reproducing TA-Lib's leftmost tie-break for the index variants. Bit-for-bit
/// equal to the naïve `O(n·period)` scan on finite inputs (ADR 0005).
pub(crate) fn rolling_extreme_index(values: &[f64], period: usize, take_max: bool) -> Vec<f64> {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if n < period {
        return out;
    }
    let mut dq = MonoQueue::with_capacity(period);
    for i in 0..n {
        // 移除已滑出窗口的最左候选 / drop the leftmost candidate that left the window
        while !dq.is_empty() && dq.front() + period <= i {
            dq.pop_front();
        }
        if take_max {
            // 弹出 < 候选者（严格小于），相等者保留 -> 队首为窗口最左最大值
            // pop < candidates (strict), keep equal -> front is the leftmost max
            while !dq.is_empty() && values[dq.back()] < values[i] {
                dq.pop_back();
            }
        } else {
            // 弹出 > 候选者（严格大于），相等者保留 -> 队首为窗口最左最小值
            // pop > candidates (strict), keep equal -> front is the leftmost min
            while !dq.is_empty() && values[dq.back()] > values[i] {
                dq.pop_back();
            }
        }
        dq.push_back(i);
        if i >= period - 1 {
            out[i] = dq.front() as f64;
        }
    }
    out
}

/// 滚动窗口最大值（用于 MIDPOINT / MIDPRICE 的 `max` 侧）。
/// Rolling window maximum (the `max` side of MIDPOINT / MIDPRICE).
#[inline]
pub fn rolling_max(values: &[f64], period: usize) -> Vec<f64> {
    rolling_extreme(values, period, true)
}

/// 滚动窗口最小值（用于 MIDPOINT / MIDPRICE 的 `min` 侧）。
/// Rolling window minimum (the `min` side of MIDPOINT / MIDPRICE).
#[inline]
pub fn rolling_min(values: &[f64], period: usize) -> Vec<f64> {
    rolling_extreme(values, period, false)
}
