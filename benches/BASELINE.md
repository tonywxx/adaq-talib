# 性能基线快照 · Performance Baseline Snapshot

> 回归护栏（见 ADR 0010 D4 / `NEXT-ACTIONS-perf.md` P1）。
> Regression guardrail (ADR 0010 D4 / P1). 任何 P2 优化改动后重跑 `cargo bench`，
> 以本文件中的 **ns/elem** 为对照，确认性能改善且数值（黄金向量）零偏差。

## 环境 / Environment

- 采集日期：2026-08-10
- 机器：Apple Silicon (aarch64-apple-darwin)，macOS
- Rust：`cargo bench`（release profile，优化开启；**未**使用 `-C target-cpu=native` —— 该选项仅用于临时对比，见 ADR 0010 D1）
- 原生 C 对照：`TA-Lib C 0.7.1`（`brew install ta-lib`，Homebrew 命名为 `ta-lib` / `libta-lib.dylib`），经 `bench-c` feature FFI 链接
- Python 绑定对照：`talib` 0.7.1（见 `tools/bench/compare.py`，**非原生 C 口径**，仅量级参考）

## 方法 / Method

- 输入：LCG 确定性伪随机序列，`N = 1_000_000`，`PERIOD = 20`（MIDPRICE/MIDPOINT 由 price 派生 `high=p*1.01, low=p*0.99`）
- `ITERS = 20`，取 `avg/call`；`ns/elem = elapsed / ITERS / N`
- `checksum` 防优化（Rust 侧取末位有效值；C 侧因 TA-Lib 紧凑输出取 `out[nb-1]`）
- 属**点测**（spot measurement），多次运行有 ±5% 波动，仅供趋势对照

## 基线数值 / Baseline numbers

> 采集自 `cargo bench`（Rust 侧，release）。`C (native)` 列：本轮 P2-1 重测值（DEMA/TEMA/T3）
> 与 P1 基线值（其余）并存，点测 ±5% 波动。Rust 侧 DEMA/TEMA/T3 已改为单遍融合核（`core::nested_ema_with_output`，见 P2-1）。
> `Δ` = 本轮 Rust / P1 Rust（越小越好）。

| 指标 | Rust P1 | Rust P2-1 | Δ (P2-1/P1) | C ns/elem (native) | Rust / C (P2-1) | 状态 |
|------|--------:|----------:|-------------:|-------------------:|----------------:|------|
| SMA      | 1.19 | 1.19¹ | 1.00× | 1.99 | 0.60× | 已完成（已快于 C） |
| BBANDS   | 5.61 | 5.61¹ | 1.00× | 5.52 | 1.02× | 已完成（≈持平） |
| TEMA     | 10.84 | **3.48** | **0.32×** | 7.56 | **0.46×** | P2-1 ✅（已快于 C） |
| DEMA     | 7.40 | **3.61** | **0.49×** | 4.93 | **0.73×** | P2-1 ✅（已快于 C） |
| T3       | 22.28 | **3.79** | **0.17×** | 2.70 | **1.40×** | P2-1 ✅（待 P3 评估） |
| MIDPRICE | 22.81 | 22.81¹ | 1.00× | 12.43 | 1.84× | P2-2 待做（单调队列） |
| WMA      | 9.93 | 9.93¹ | 1.00× | 2.54 | 3.91× | P2-3 待做（前缀和） |
| MIDPOINT | 22.55 | 22.55¹ | 1.00× | 3.18 | 7.09× | P2-2 待做（单调队列） |

¹ P2-1 未改动该指标，Rust 数值与 P1 基线一致；C 列沿用 P1 基线值。

> 说明：SMA/BBANDS 已与 C 持平或更优；**P2-1 已完成** —— DEMA/TEMA/T3 经单遍融合核后
> 不仅显著提速，DEMA/TEMA 甚至**快于原生 C**。T3 仍慢于 C（1.40×），是 P3 SIMD 评估的候选
> （见 `NEXT-ACTIONS-perf.md` P3，闸门：自动向量化失败 **且** 慢 >20%）。

## P2 优化优先级（按 Rust/C 差距由大到小，更新于 P2-1 后）

1. **MIDPOINT**（7.09×）—— `rolling_extreme` O(n·period) → 单调队列 O(n) 【下一优先】
2. **WMA**（3.91×）—— 内循环每 `i` 重算 `period` 次 → 滑动前缀和
3. **MIDPRICE**（1.84×）—— 同上 O(n·period) → 单调队列
4. **T3**（1.40×）—— 已做单遍融合核；若 P3 闸门满足（自动向量化失败且 >20%）则评估 SIMD
5. ~~DEMA (1.48×)~~ / ~~TEMA (0.96×)~~ —— **P2-1 已完成**

**P2-1 结论（2026-08-10）**：新增 `core::nested_ema_with_output` 单遍嵌套 EMA 级联（const-generic `L` + `combine` 闭包），DEMA/TEMA/T3 经 `_with_output` 委托调用，消除 2/3/6 次中间 `Vec` 分配与独立扫描；数值与原版**逐项相等**（黄金向量 1:1 通过，ADR 0005）。T3 自 7.85× 降至 1.40×，是 P3 候选。

目标：将剩余热路径 Rust/C 比值压到 **≈1.0**（P3 SIMD 闸门见 ADR 0010 / `NEXT-ACTIONS-perf.md` P3）。

## 复现 / Reproduce

```text
# Rust 侧基线（零依赖，默认）
cargo bench

# 原生 C 双轨对照（需系统安装 TA-Lib C；本机库名 ta-lib）
cargo bench --features bench-c

# Python 绑定对照（口径≠原生 C，仅参考）
python3 tools/bench/compare.py
```

## Python 绑定对照（附录，非原生 C 口径）

来自 `tools/bench/compare.py`（同输入，量级参考）：

| 指标 | Python 绑定 ns/elem |
|------|--------------------:|
| SMA | 2.05 |
| DEMA | 5.05 |
| TEMA | 7.38 |
| T3 | 2.73 |
| WMA | 2.31 |
| MIDPOINT | 2.22 |
| MIDPRICE | 12.46 |
| BBANDS | 5.28 |
