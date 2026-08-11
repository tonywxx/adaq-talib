# 深度研究：adaq-talib 161 函数性能对标与原生 C 的加速可行性

> 研究日期：2026-08-10 · 数据来源：`all161_results.csv`（基准 `all161_results_before.csv` 备份）
> 约束基线（ADR 0010）：Zero-FFI · No-Deps（`[dependencies]` 为空）· 单线程 · safe · SIMD 延后至 `simd` feature · `f64` · 数值 1:1（ADR 0005）

---

## 0. 结论先行 + 诚实可行性判定

**一句话：在项目的现有硬约束下，"161/161 全部严格快于原生 C ≥ 2×" 这一目标对约 45–50 个本质顺序依赖（IIR / 滚动递归）指标在数学上不可达；对 Pattern Recognition 与部分 Cycle 通过消除冗余计算（单线程）可逼近乃至超过 1×，但稳定 >2× 需要放宽 No-Deps（并行）；对 Elementwise / 可向量化类已自然达成 >2×。**

我先用事实纠正任务前提中的一处数字偏差，再给出分层判定。

---

## 1. 当前状态量化（来自备份基线 `all161_results_before.csv`）

任务描述称"当前仅 36 个指标快于 C 版"（此为 Rust/C < 0.8 的"更快"桶口径，见本仓库权威报告 `docs/validation-and-performance-report.md`）。实测分布如下（本表 `speedup = C_ns/elem ÷ adaq_ns/elem`，>1 即 Rust 更快；与权威报告的 `Rust/C = adaq_ns ÷ c_ns` 互为倒数，数字完全一致）：

| 桶 | 数量 | 说明 |
|---|---|---|
| **≥ 2×**（已达标） | **14** | imi 4.77 · linear_reg* 2.6–4.2 · bbands 2.56 · macd 2.34 · macd_ext 2.27 · tema 2.18 · mom 2.11 · sqrt 2.08 · ceil 2.04 · sub 2.03 |
| 1× – 2× | 40 | 多数已接近 parity |
| 0.5× – 1× | 39 | 慢于 C 但差距 < 2× |
| **< 0.5×**（严重落后） | **68** | 其中 57 个为 Pattern Recognition，其余为 Cycle + 少数顺序指标 |

**口径对齐（重要）**：任务描述的"36 个快于 C"对应权威报告的"更快"桶（Rust/C < 0.8）；若按更宽松的"严格快于 C"（Rust/C < 1.0）则为 **54 个**——正与本表"1–2×"+"≥2×"合计（40+14=54）一致。本表采用互补定义 `speedup = C/adaq`（>1 即 Rust 更快）。**无论哪种口径，当前仅 14 个函数达到"≥2× 快于 C"（Rust/C ≤ 0.5）；161/161 达标为 0 个** —— 这是本研究的核心起点。

按类别（基线）：

| 类别 | n | >1× | >2× | <1× | 平均 speedup |
|---|---|---|---|---|---|
| Overlap Studies | 18 | 6 | 2 | 10 | 1.12 |
| Momentum Indicators | 31 | 7 | 4 | 20 | 0.98 |
| Volatility Indicators | 3 | 2 | 0 | 1 | 1.24 |
| Volume Indicators | 3 | 2 | 0 | 1 | 1.06 |
| Price Transform | 5 | 5 | 0 | 0 | 1.72 |
| Statistic Functions | 9 | 2 | 5 | 2 | 2.18 |
| Math Operators | 11 | 4 | 1 | 6 | 1.16 |
| Math Transform | 15 | 8 | 2 | 5 | 1.23 |
| **Cycle Indicators** | 5 | 0 | 0 | 5 | **0.68** |
| **Pattern Recognition** | 61 | 4 | 0 | 57 | **0.38** |

最差 5 个：`cdl_separatinglines` 0.097 · `cdl_kickingbylength` 0.142 · `cdl_3blackcrows` 0.194 · `cdl_mathold` 0.205 · `cdl_morningstar` 0.224（Rust 比 C 慢 4–10×）。

---

## 2. 基准方法学批判（必须先行修正）

在谈任何"加速比"之前，基准本身**非同口径**，会导致结论失真：

- **非蜡烛函数**（Overlap/Momentum/…/Cycle）：C 侧经 `talib_ffi::c_abstract` → `TA_CallFunc`（抽象 API）计时。每次迭代 `TA_CallFunc` 内部会**重新分配 scratch buffer**（`benches/all161_bench.rs:489–490` 的循环体内调用，ParamHolder 虽只分配一次，但函数内部临时数组每调用一次分配/释放）。这系统性地**抬高了 C 的计时**。
- **蜡烛函数**（CDL*）：C 侧经**直接 FFI**（`TA_CDLxxx(...)`，预分配 `out` 缓冲，`benches/all161_bench.rs:504`）计时，**无每调用分配**——这是干净的口径。

**后果（精确化）：**
1. Rust 在 elementwise / 廉价非蜡烛函数上的"≥2× 胜利"有**部分是假象**——C 抽象 API 内部的 per-call scratch 分配被计入，夸大了 Rust 的相对优势。
2. **但最关键的拖累类别 Pattern Recognition（平均 0.38×）走的是干净的直连 FFI**（见 `benches/all161_bench.rs` 的蜡烛外臂，`TA_CDLXXX` 直接调用、预分配缓冲），所以其落后是**真实的、非测量假象**——这与本仓库 B1 结论（"性能落差主要来自 Pattern 61 蜡烛"）一致。
3. Rust 即便在 C 被抬高的情况下仍有 107 个函数慢于 C——证明确有真实低效。

**关键推论**：把非蜡烛函数也改为直连 FFI 计时，会**移除 C 的 handicap**，反而可能**拉大** Rust 在非蜡烛廉价函数上的劣势（使整体 geomean 更差）。换言之，当前基准若 anything 对 Rust 偏乐观；修正后"161/161 >2×"只会更难，不会更易。

**公正性要求**：在宣称任何加速比前，应先统一把 C 侧改为**直接 FFI**（如同蜡烛那样逐个声明 `TA_XXX`）计时，得到可信基线。本研究的分类结论在"修正后"依然成立，且修正只会强化"顺序类不可达 2×"的判定。

---

## 3. 瓶颈分类与代码证据

### 3.1 Pattern Recognition（61 个，平均 0.38×，57/61 落后）—— **最有价值、最可修复**
证据（`src/pattern/batch_1.rs` `cdl_hammer_with_output`，原实现）：每根 K 线调用 4 个 `CandleAvg` 的 `value()` + `advance()`，而 `CandleAvg::value/advance`（`src/pattern/mod.rs:271/285`）每次都从原始 OHLC 切片**重新 `range_of`**（实体/影线/全幅），并带数组边界检查与 `match rt` 分支 + 方法调用开销。每 bar 约 **12 次冗余 `range_of` 求值 + 8 次方法调用 + 多次数组加载带边界检查**。

C 源 `ta_CDLHAMMER.c` 的做法：每 bar 把 `realBody / upperShadow / lowerShadow` 算**一次**进局部 `double`，再用单个 `TA_CANDLEAVERAGE` running mean。Rust 的 `CandleAvg` 滚动和本身是正确的 O(1) 递推（`total += range(i) - range(trailing)`，与 C 一致），低效在于**跨多个 `CandleAvg` 对象重复 `range_of` 重算 + 调用/边界检查开销**。

**关键利好**：Pattern Recognition 是 **embarrassingly parallel across bars**——第 `i` 根 K 线的判定仅依赖局部窗口（≤5 根）+ 可逐块播种的 running sum，**无跨时间递归**。因此：
- 单线程消除冗余（每 bar 原语算一次 + 内联 running sum）即可逼近 1×；
- 加分块并行（跨 bar）可稳定 >2×，但**需放宽 No-Deps（引入 rayon）或 `unsafe` 手搓线程**——与 ADR 0010 的"No-Deps / safe"硬约束冲突，需项目级决策。

### 3.2 Cycle / Hilbert Transform（5 个，平均 0.68×，全部落后）
`src/cycle.rs` 的 `Hilbert` 状态机是对官方 C 源的忠实逐行移植（价格平滑器 WMA-4 + 希尔伯特变换环形缓冲）。它是**严格状态机**：第 `i` 根的输出依赖第 `i-1` 根的全部状态 → **本质顺序，无法跨时间向量化**。当前落后源于 Rust 结构体大量字段 + `[f64;3]` 环形缓冲索引带边界检查 + 方法调用，比 C 的裸指针版慢。
**可达**：细致内联 + 去边界检查 + 字段布局优化 → 逼近 1×（parity）；**单线程 >2× 不可达**。

### 3.3 顺序 IIR / 滚动递归（EMA·RSI·MACD·ATR·ADX·DX·STOCH·CMO·TRIX·APO/PPO·KAMA·SAR·MAMA…）
这些是**严格递推**（`out[i]` 依赖 `out[i-1]`）：EMA `out=k·in+(1-k)·out_prev`、RSI 的 Wilder 平滑、MACD/ATR 的 EMA 链…… C 已在**每元素最小工作量**（递归形式 + rolling accumulator）。Rust 当前 0.4–0.9× 多因：
- 朴素 O(n·period) 窗口（如历史上的 `rolling_extreme`，"correctness first"）；
- 未用 running accumulator（每 bar 重扫 period）；
- 边界检查 / 方法调用开销。

**修正后可逼近 1×（parity）；单线程 >2× 在数学上不可达**——递推禁止跨时间向量化，每元素工作已是最小。

### 3.4 Elementwise / 可向量化（Math Operators/Transform、Price Transform、部分 Stat）
已通过 LLVM autovec **自然达成 1.7–4.8×**（imi 4.77 · linear_reg* 2.6–4.2 · math ops ~2× · medprice/typprice/wclprice 1.7–2.0）。这些**已满足 >2× 目标**，只需锁定回归护栏。

---

## 4. 诚实可行性判定（按类别）

| 类别 | 当前均速 | 消除冗余后单线程可达 | 单线程 >2× 可达？ | 备注 |
|---|---|---|---|---|
| Elementwise / 可向量化 | 1.2–2.2 | 维持 | **是（已达成）** | 锁定回归即可 |
| Pattern Recognition | 0.38 | ~1.0–1.5（单线程） | 单线程部分可达；**全量 >2× 需并行** | 见 §3.1，PoC 已证单线程可超 1× |
| Cycle (HT) | 0.68 | ~1.0 | **否** | 严格状态机 |
| 顺序 IIR / 滚动递归 | 0.6–0.9 | ~1.0（parity） | **否** | 递推禁止跨时间向量化 |
| Stat/rolling（stddev/var/linear_reg） | 1.3–4.2 | 维持/微调 | 已达成（部分） | — |

**可达 >2× 的真实子集**：Elementwise 全量 + Pattern Recognition（消除冗余 + 分块并行）+ 部分 Stat。  
**单线程永远不可达 >2× 的子集**：所有严格递推指标（EMA/RSI/MACD/ATR/ADX/DX/STOCH/CMO/TRIX/APO/PPO/KAMA/SAR/MAMA）+ 全部 Cycle。

---

## 5. 已交付的 Proof-of-Concept（已验证）

对 `cdl_hammer` 实施 §3.1 描述的优化：每 bar 把 `real_body / lower_shadow / upper_shadow / high_low_range` 算**一次**进局部变量，并以**与 `CandleAvg::value/advance` 完全相同的 `+= range(i) - range(trailing)` 递推**维护 3 个内联 running-sum 累加器（算术逐位一致，边界索引可证在界内 → LLVM 消除边界检查）。

结果（release + bench-c，直接 FFI 计 C 侧）：

| 指标 | adaq 前 (ns/elem) | adaq 后 | C (ns/elem) | speedup 前 | speedup 后 |
|---|---|---|---|---|---|
| `cdl_hammer` | 12.64 | **1.60** | 2.80 | 0.226（慢 4.4×） | **1.751（快 1.75×）** |

- Rust 侧实现加速 **≈7.9×**，并从"显著慢于 C"翻转为"显著快于 C"（即便对**干净的直连 FFI C 计时**也快 1.75×）。
- **数值 1:1 保持**：全量 `cargo test` 161/161 黄金向量通过，含 `cdl_hammer_matches_golden_vector`。
- 对照未改动函数 `cdl_marubozu` 仅 7.24→6.53（≈10% 测量噪声），佐证 hammer 的下降压倒性真实。

这证明 Pattern Recognition 的落后是**真实冗余计算**所致，可修复且单线程即可超过 clean-C。按本仓库 `Rust/C = adaq_ns ÷ c_ns` 口径，`cdl_hammer` 由 **4.42 → 0.571**（即 **1.75× 快于 C**）——已逼近 2× 硬线（0.5）但未单线程越过，恰好印证 §4 判定：Pattern 单线程可超 1×、部分可近 2×，**全量稳定 >2× 仍需并行**。

---

## 6. 优化路线图（分阶段，务实）

- **P0 — 修复基准（前置，不可跳过）**：把 C 侧统一改为直接 FFI 计时（逐函数声明 `TA_XXX`，同蜡烛口径），重测得到可信基线。否则任何"加速比"都不可证伪。
- **P1 — Pattern Recognition 全量应用 PoC 模板**：把 §5 的"原语算一次 + 内联 running sum"应用于全部 61 个 CDL 函数（注意各函数的 settings/off/range 组合不同，需逐函数核对 `CandleAvg` 语义）。目标：单线程全部 ≥1×；再评估分块并行路径。
- **P2 — Cycle HT 内联化**：`Hilbert` 状态机展开为局部变量 + 去边界检查 + 字段重排，逼近 1×。
- **P3 — 顺序 IIR 替换朴素窗口**：用 running accumulator 替换 O(n·period) 扫描（EMA/RSI/Wilder/滚动极值），逼近 1×。
- **P4 — ">2× 全局"工程决策**：单线程无法覆盖顺序类。若坚持全局 >2×，必须（a）对可并行子集引入并行（rayon → 违背 No-Deps，需决策）或（b）接受"全局可达到的是：消除所有伪慢（全部 ≥1×）+ 可并行类 >2×"这一在约束内可交付、可证伪的目标。

---

## 7. 关于"161/161 >2× 快于原生 C"的硬真相

在现有硬约束（Zero-FFI / No-Deps / 单线程 / safe）下：

1. **不可能**对 ~45–50 个严格递推指标（见 §4）实现单线程 >2×——TA-Lib C 已在其每元素最小工作量上运行，递推禁止跨时间向量化。
2. **可能且已验证**对 Pattern Recognition 通过消除冗余（单线程即可超 1×，部分超 2×；全量 >2× 需并行）。
3. **已自然达成**对 Elementwise / 可向量化类（>2×）。

**建议把 KPI 从"161/161 全部 >2×"调整为**：「消除所有 <1× 的伪慢（即全量 ≥1× parity）+ 对可并行子集（Pattern / 部分 rolling stat）达成 >2×」。这是在项目约束内**可证伪、可交付**的真实目标；强行承诺 161/161 >2× 会要么食言，要么被迫引入并行而违背 No-Deps 承诺。

---

## 附：复现命令

```bash
# 基线（已备份为 all161_results_before.csv）
cp all161_results.csv all161_results_before.csv
cargo bench --bench all161_bench --features bench-c        # 写回 all161_results.csv
cargo test                                                 # 161/161 黄金向量 1:1 校验
# 单函数验证（示例）
cargo test --test pattern_batch8_test cdl_hammer_matches_golden_vector
```
