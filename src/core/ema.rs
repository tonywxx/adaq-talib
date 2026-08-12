//! 指数平滑族（EMA / Wilder / 嵌套 EMA 级联）。
//!
//! Exponential-smoothing family (EMA / Wilder / nested-EMA cascade).

/// 指数移动平均（TA-Lib `TA_EMA`，经典种子），与输入等长的全索引向量。
///
/// Exponential moving average (TA-Lib `TA_EMA`, classic seed), a full-indexed vector
/// with the same length as the input.
///
/// 对齐与种子严格对照 TA-Lib 0.7.1 默认兼容性（`TA_MA_CLASSIC`）：
/// 首个有效值 = 前 `period` 个有限值的算术均值（SMA 种子），后续按
/// `out[i] = (x[i] - out[i-1]) * k + out[i-1]`，其中 `k = 2/(period+1)` 递推。
/// 前导 `period-1` 个位置为 [`f64::NAN`]。
///
/// Alignment & seeding replicate TA-Lib 0.7.1 default compatibility (`TA_MA_CLASSIC`):
/// the first valid value is the SMA of the first `period` finite values (the seed), then
/// `out[i] = (x[i] - out[i-1]) * k + out[i-1]` with `k = 2/(period+1)`. The leading
/// `period - 1` positions are [`f64::NAN`].
///
/// 输入可含前导 [`f64::NAN`]：种子从首个有限值算起（用于 DEMA/TEMA 的嵌套 EMA）。
/// Inputs may carry leading [`f64::NAN`]; the seed starts at the first finite value
/// (used by nested EMAs in DEMA/TEMA).
///
/// # 公式 / Formula
/// ```text
/// seed     = (1/period) * Σ_{k=0}^{period-1} x[start+k]
/// out[i]   = (x[i] - out[i-1]) * 2/(period+1) + out[i-1],  i > start+period-1
/// ```
/// 来源 / Source: TA-Lib `ta_ema.c`（经典种子）。
///
/// # Panics
/// 调用方须保证 `period >= 1`。/ Caller must ensure `period >= 1`.
/// 指数移动平均，零拷贝写入 `out`（与 `values` 等长）。见 [`ema`]。
/// Exponential Moving Average, written zero-copy into `out`. See [`ema`].
pub fn ema_with_output(values: &[f64], period: usize, out: &mut [f64]) {
    debug_assert!(period >= 1);
    debug_assert_eq!(out.len(), values.len());
    let n = values.len();
    let start = match values.iter().position(|&x| !x.is_nan()) {
        Some(s) => s,
        None => {
            for v in out.iter_mut() {
                *v = f64::NAN;
            }
            return;
        }
    };
    if n - start < period {
        for v in out.iter_mut() {
            *v = f64::NAN;
        }
        return;
    }
    // 仅填前导不稳定期（其余由递推覆盖），消除对整段输出的 O(n) NaN 填充
    // （P3-2 性能优化；EMA 是 ema/apo/ppo/t3/dema/tema/macd 的公共热路径）。
    // Fill only the leading unstable region; the rest is overwritten by the recursion,
    // removing the full O(n) NaN pass (high-leverage: EMA underpins many indicators).
    for v in out[..start + period - 1].iter_mut() {
        *v = f64::NAN;
    }
    let seed: f64 = values[start..start + period].iter().copied().sum::<f64>() / period as f64;
    out[start + period - 1] = seed;
    let k = 2.0 / (period as f64 + 1.0);
    let mut prev = seed;
    // 以切片视图递推：编译器可证明 `v[i]` / `o[i]` 不越界，从而消除 `out[i]` 的边界检查
    // （release 下 `out.len()==values.len()` 仅由 debug_assert 保证，无法被证明），这是 EMA
    // 相对原生 C 偏慢的主因之一。数值与逐元素索引版本逐项相等、零偏差（ADR 0005）。
    let v = &values[start + period..];
    let o = &mut out[start + period..];
    let len = v.len().min(o.len());
    // 硬件 FMA：把 `(v-prev)*k+prev` 收缩为单条融合乘加指令（与 GCC -O2 默认
    // `-ffp-contract=fast` 下 TA-Lib C 的 EMA 一致），既提速（1 次运算而非乘+加）
    // 又更贴近 C 的数值（同一次舍入）。与黄金向量逐项在 1e-8/1e-10 容差内一致（ADR 0005）。
    // Hardware FMA: contract `(v-prev)*k+prev` into a single fused multiply-add, matching
    // TA-Lib's C EMA under GCC -O2 default `-ffp-contract=fast`. Bit-for-bit within the
    // 1e-8 / 1e-10 golden tolerance (ADR 0005).
    for i in 0..len {
        prev = (v[i] - prev).mul_add(k, prev);
        o[i] = prev;
    }
}

pub fn ema(values: &[f64], period: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; values.len()];
    ema_with_output(values, period, &mut out);
    out
}

/// 嵌套 EMA 级联（单遍融合核，DEMA / TEMA / T3 共用）。
///
/// Nested EMA cascade — a single forward pass that produces `E1..EL` so that DEMA / TEMA /
/// T3 can be combined without allocating `L` intermediate `Vec<f64>` or scanning the series
/// `L` times. At each index `i` we compute `E1[i]`, feed it into `E2`, `E2[i]` into `E3`, …
/// all in one loop, holding only `O(L)` scalar state.
///
/// 每层 `k` 以其输入（上一层输出）的**首个 `period` 个有限值**的算术均值作 SMA 种子，
/// 之后按 `k = 2/(period+1)` 递推 —— 与逐次调用 [`ema`] 完全一致（同一次求和顺序、同一
/// 递推），数值逐项相等、零偏差（ADR 0005）。
///
/// Each level `k` seeds with the SMA of the first `period` finite values of its input (the
/// previous level's output), then recurses with `k = 2/(period+1)` — identical to calling
/// [`ema`] repeatedly (same summation order, same recursion), bit-for-bit equal (ADR 0005).
///
/// `combine` 接收当前索引的全部 `E1..EL`，写出最终组合值（如 DEMA: `2*E1 - E2`）。
/// `combine` receives the full `E1..EL` at the current index and writes the final combined
/// value. The output is [`f64::NAN`] until the deepest level `EL` first becomes valid.
///
/// # Panics
/// 调用方须保证 `period >= 1` 且 `out.len() == values.len()`。/ Caller must ensure
/// `period >= 1` and `out.len() == values.len()`.
#[inline]
pub fn nested_ema_with_output<const L: usize, F>(
    values: &[f64],
    period: usize,
    mut combine: F,
    out: &mut [f64],
) where
    F: FnMut(&[f64; L]) -> f64,
{
    debug_assert!(period >= 1);
    debug_assert_eq!(out.len(), values.len());
    let n = values.len();
    if n == 0 {
        return;
    }
    let k = 2.0 / (period as f64 + 1.0);
    let pk = period as f64;
    let nan = f64::NAN;
    // 每层状态：当前值、递推上一值、种子累加和、种子计数、是否已播种。
    // Per-level state: current value, previous (recursion) value, seed accumulator,
    // seed count, seeded flag.
    let mut e = [nan; L];
    let mut prev = [nan; L];
    let mut seed_sum = [0.0f64; L];
    let mut seed_n = [0usize; L];
    let mut seeded = [false; L];

    for i in 0..n {
        // Level 1: raw input series. Before seeding, only finite values accumulate the seed.
        let v1 = values[i];
        if !seeded[0] {
            if !v1.is_nan() {
                seed_sum[0] += v1;
                seed_n[0] += 1;
                if seed_n[0] == period {
                    e[0] = seed_sum[0] / pk;
                    seeded[0] = true;
                    prev[0] = e[0];
                }
            }
        } else {
            // Once seeded, every index recurses (a NaN input propagates NaN, matching `ema`).
            e[0] = (v1 - prev[0]).mul_add(k, prev[0]);
            prev[0] = e[0];
        }

        // Levels 2..=L: each consumes the previous level's freshly computed output.
        for l in 1..L {
            let src = e[l - 1];
            if !seeded[l] {
                if !src.is_nan() {
                    seed_sum[l] += src;
                    seed_n[l] += 1;
                    if seed_n[l] == period {
                        e[l] = seed_sum[l] / pk;
                        seeded[l] = true;
                        prev[l] = e[l];
                    }
                }
            } else {
                e[l] = (src - prev[l]).mul_add(k, prev[l]);
                prev[l] = e[l];
            }
        }

        // Combine only once the deepest level is valid; otherwise keep NaN.
        out[i] = if seeded[L - 1] { combine(&e) } else { nan };
    }
}

/// Wilder 平滑（SMMA，TA-Lib 方向性运动 / RSI 族所用），跳过前导 [`f64::NAN`]。
///
/// Wilder smoothing (SMMA) as used by TA-Lib's directional-movement and RSI family; skips
/// a leading [`f64::NAN`] prefix.
///
/// 种子 = 首个有限值起的 `period` 个均值（置于 `start+period-1`），之后按
/// `prev = prev + (x - prev) * k`、`k = 1/period` 递推（等同于 `(prev*(period-1) + x)/period`）。
///
/// Seed = mean of the first `period` finite values (placed at `start + period - 1`), then
/// `prev = prev + (x - prev) * k` with `k = 1/period` (equivalently `(prev*(period-1) + x)/period`).
///
/// 与 [`ema`]（经典 `k = 2/(period+1)`）的区别仅在于平滑常数；种子策略一致。
/// Differs from [`ema`] (classic `k = 2/(period+1)`) only in the smoothing constant; the
/// seeding strategy is the same.
///
/// # Panics
/// 调用方须保证 `period >= 1`。/ Caller must ensure `period >= 1`.
#[inline]
pub fn ema_wilder(values: &[f64], period: usize) -> Vec<f64> {
    debug_assert!(period >= 1);
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    let start = match values.iter().position(|&x| !x.is_nan()) {
        Some(s) => s,
        None => return out,
    };
    if n - start < period {
        return out;
    }
    let seed: f64 = values[start..start + period].iter().copied().sum::<f64>() / period as f64;
    out[start + period - 1] = seed;
    let k = 1.0 / period as f64;
    let mut prev = seed;
    for i in (start + period)..n {
        prev = prev + (values[i] - prev) * k;
        out[i] = prev;
    }
    out
}
