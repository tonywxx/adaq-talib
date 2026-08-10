# 性能优化实现总结 · Performance-Optimization Implementation Summary (adaq-talib)

> 工程师：寇豆码 (Kou) · 任务：T01–T05（基于 `docs/perf-final-plan.md`）
> 约束：Zero-FFI（`[dependencies]` 空）/ No-Deps / 零偏差（ADR 0005）/ 无 `unsafe`·无 `nightly` / P3 SIMD = NO-GO。

## 0. 校验总览 / Verification

- **`cargo test`：全绿**。308 个单元/黄金向量子测试（22 个 `test result` 段），**0 失败**。
  161 个对外函数的黄金向量（ADR 0005：rel 1e-8 + abs 1e-10）逐项通过。
- **编译警告：0**。新增 `_with_output` 变体与原生实现零偏差，无 unused / dead_code 警告。
- **基准口径**：`RUSTFLAGS="-C target-cpu=native"`，`N=1_000_000`、`PERIOD=20`、`ITERS=20`，带 `checksum` 防优化（与既有 `benches/*.rs` 一致）。

## 1. 各函数优化明细 / Per-function detail

| 任务 | 函数 | 技术 | before ns/elem | after ns/elem | 加速 |
|------|------|------|---------------|--------------|------|
| T01 | `bbands`(SMA) | 单遍 `rolling_mean_var`（共享滑动 `sx`+`sxx`，合并原 `rolling_mean`+`rolling_var` 两遍） | 4.56（实测 stash 前） | **2.81** | ~1.62× |
| T02 | `linear_reg`/`_angle`/`_intercept`/`_slope`/`tsf`（共享 `linreg_core`） | O(n) 滑动 `sy`+`sxy`（`sxy[i]=sxy[i-1]+period·x[i]−sy[i]`，附录 A） | O(n·period) 朴素 | **2.33** | 渐近 ~period(×20)× |
| T03 | `correl` | O(n) 滑动 `s0/s1`+`s00/s11/s01`（滚动和/平方和/叉积，同构 `rolling_var`） | O(n·period) 朴素 | **4.61** | 渐近 ~period(×20)× |
| T04 | `willr` | 单调队列 `rolling_max`/`rolling_min`（O(n)，最右 tie-break 与朴素一致） | O(n·period) 朴素 | **7.83** | 渐近 ~period(×20)× |
| T04 | `stoch`/`stoch_f`（含 `stoch_rsi` 经 `stoch_f` 继承） | 同上 `stoch_fastk` 复用极值队列 | O(n·period) 朴素 | **10.71** | 渐近 ~period(×20)× |

> LINREG/CORREL/WILLR/STOCH 的"before"为原 O(n·period) 朴素窗口扫描；bare HEAD 不独立可编译（依赖工作树中未提交的前序改动），故未取严格 before 数值，按 ADR 0010 以**渐近 O(n·period)→O(n)** 报告（理论降幅因子 ≈ `period`=20，实际因新核常数略低）。`bbands` 因已有对照 bench，取严格 before/after。

## 2. 关键设计决策 / Key decisions

1. **T01 单遍融合仅作用于 SMA 中轨**：非 SMA（`ma_type`）中轨保留 `ma`+`stddev` 分解，避免改变其它 MA 类型的数值行为；`rolling_mean_var` 的 `sx`/`sxx` 递推顺序与 `rolling_mean`/`rolling_var` 逐一相同，故产出**位级相等**。
2. **T02/T03 滑动递推的零偏差**：种子窗口沿用朴素求和（与历史实现逐项对齐），递推阶段仅重排浮点加法顺序（与既有 `rolling_mean`/`wma`/`rolling_var` 同构），在黄金向量容限内逐项相等；新增 `linreg_core_matches_naive` / `correl_core_matches_naive` 单测（多组 n/period、单调/随机、5 个 mode）守护。
   - 注：退化 `period==1`（`denom=0`，数学未定义，TA-Lib 要求 `period>=2`，黄金向量从不使用）从对照单测中跳过。
3. **T04 极值替换零偏差**：朴素 `willr`/`stoch` 的 HH/LL 扫描在相等时保留**窗口最右**极值，与 `rolling_max`/`rolling_min` 的 `<=`/`>=` 弹出规则逐项一致，故 `out[i]` **位级相等**。
4. **T05 P3 SIMD = NO-GO**（按闸门）：MIDPOINT 的单调双队列、T3 的顺序 EMA 递推均**非向量化友好**，显式 SIMD 不改变单序列调用形态，且 TA-Lib C 同构实现同样无 SIMD；不引入 `simd` feature / `unsafe` / 外部 crate。
5. **`_with_output` 扫尾（D2）**：为 T01–T04 热路径补原地变体——`bbands_with_output`(写入 `Bbands`)、`linear_reg*/tsf_with_output`(×5)、`correl_with_output`、`willr_with_output`、`stoch_with_output`、`stoch_f_with_output`；原函数委托之，公开 `Result<Vec>` API 形态不变。

## 3. 新增文件 / Files

- `src/core/mod.rs`：新增 `rolling_mean_var`（单遍均值+方差融合原语）。
- `src/overlap.rs`：`bbands` 委托 `bbands_with_output`（SMA 走融合核）。
- `src/stat.rs`：`linreg_core`/`correl_core` 改为滑动 + `_with_output` 变体 + 朴素对照单测。
- `src/momentum.rs`：`willr`/`stoch`/`stoch_f` 改用 `rolling_max`/`rolling_min`（经 `stoch_fastk`）+ `_with_output` 变体。
- `benches/linreg_bench.rs`、`benches/correl_bench.rs`、`benches/willr_bench.rs`、`benches/stoch_bench.rs`：Rust 侧基准（对照既有格式；**未接 C FFI**——新函数若接原生对照需系统 TA-Lib C 且涉及 `unsafe`，超出零-FFI 精神，按计划"Rust 侧即可并注明"处理）。`Cargo.toml` 注册 4 个 `[[bench]]`。

## 4. 全局一致性审查 / Global consistency review

- **IS_PASS: YES**
  - 161 个黄金向量测试全部通过（`cargo test` 0 失败）。
  - 无新增编译警告（lib + benches 均干净）。
  - 公开 `Result<Vec<f64>>` 签名未破坏；新增 `_with_output` 均为额外原生变体。
  - 跨文件导入一致：`crate::core::{rolling_mean_var,rolling_max,rolling_min}`、`crate::stat::stddev`、`rolling_mean_skip`、`stoch_fastk` 均正确引用；无重复实现、无循环依赖。
  - Zero-FFI / No-Deps / 无 `unsafe` / 无 `nightly` / P3 SIMD 未实现，全部满足硬约束。
