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

> 采集自 `cargo bench`（Rust 侧，release）。`C (native)` 列：DEMA/TEMA/T3/MIDPRICE/MIDPOINT
> 为 P2 各阶段重测值，SMA/BBANDS/WMA 沿用 P1 基线值，点测 ±5% 波动。Rust 侧 DEMA/TEMA/T3
> 已改为单遍融合核（`core::nested_ema_with_output`，P2-1）；MIDPOINT/MIDPRICE 的窗口极值
> 已改为单调队列 O(n)（`core::rolling_extreme` + 合并 `rolling_minmax`，P2-2）。`Δ` = P2 Rust / P1 Rust。

| 指标 | Rust P1 | Rust P2 | Δ (P2/P1) | C ns/elem (native) | Rust / C (P2) | 状态 |
|------|--------:|--------:|----------:|-------------------:|--------------:|------|
| SMA      | 1.19 | 1.19¹ | 1.00× | 1.92 | 0.61× | 已完成（已快于 C） |
| BBANDS   | 5.61 | **3.02** | **0.54×** | 5.20 | **0.58×** | P2-4 ✅（已快于 C） |
| TEMA     | 10.84 | **3.46** | **0.32×** | 7.44 | **0.47×** | P2-1 ✅（已快于 C） |
| DEMA     | 7.40 | **3.63** | **0.49×** | 4.85 | **0.75×** | P2-1 ✅（已快于 C） |
| T3       | 22.28 | **3.76** | **0.17×** | 2.78 | **1.35×** | P2-1 ✅（P3 NO-GO） |
| MIDPRICE | 22.81 | **7.30** | **0.32×** | 12.25 | **0.60×** | P2-2 ✅（已快于 C） |
| WMA      | 9.93 | **2.11** | **0.21×** | 2.28 | **0.93×** | P2-3 ✅（≈持平） |
| MIDPOINT | 22.55 | **6.88** | **0.30×** | 3.05 | **2.26×** | P2-2 ✅（P3 NO-GO） |
| LINEARREG| — | **2.33** | O(n·period)→O(n) | N/A | N/A | P2-5 ✅（滑动 O(n)，C 未接线） |
| CORREL   | — | **4.81** | O(n·period)→O(n) | N/A | N/A | P2-5 ✅（滑动 O(n)，C 未接线） |
| WILLR    | — | **7.90** | O(n·period)→O(n) | N/A | N/A | P2-5 ✅（单调队列 O(n)，C 未接线） |
| STOCH    | — | **10.99** | O(n·period)→O(n) | N/A | N/A | P2-5 ✅（单调队列 O(n)，C 未接线） |

¹ SMA 在 P2 各阶段未改动，Rust 数值与 P1 基线一致。C 列：SMA/BBANDS 为 P2-4 重测；WMA 为 P2-3 重测（2.28）；其余为 P2 各阶段重测。LINREG/CORREL/WILLR/STOCH 无 P1 单独基线（原为 O(n·period) 朴素扫描），以渐近 O(n·period)→O(n) 报告；其原生 C 对照未接线（`bench-c` 仅覆盖原始 8 项），按零-FFI 精神以 Rust 侧为准并注明。Rust/C 比值基于 2026-08-10 canonical 实测（受管 cargo，无 `-C target-cpu=native`）。

> 说明：SMA/BBANDS 已与 C 持平或更优；**P2-1 已完成**（DEMA/TEMA 甚至快于 C，T3 1.40×）。
> **P2-2 已完成** —— MIDPOINT/MIDPRICE 的窗口极值由 O(n·period) 朴素扫描换为单调队列 O(n)，
> MIDPRICE 已快于 C（0.56×），MIDPOINT 自 7.09× 降至 2.08×（C 的 `TA_MIDPOINT` 内部用单遍
> 双队列 `MINMAXINDEX`，本实现以合并 `rolling_minmax` 单遍双队列对齐，待 P3 视自动向量化评估）。
> **P2-3 已完成** —— WMA 由 O(n·period) 朴素扫描换为 O(n) 滑动递推（`W[i]=W[i-1]+period·x[i]-sw[i-1]`），
> 9.93→2.12（0.94×，已快于 C）。

## P2 优化优先级（按 Rust/C 差距由大到小，更新于 P2-5 后）

1. **MIDPOINT**（2.26×）—— 单调队列 O(n) 已完成；P3 闸门评估结论：**NO-GO**（单遍双队列数据依赖，C 侧 `TA_MIDPOINT` 同为 `MINMAXINDEX`、`无 SIMD`，非缺陷）
2. **T3**（1.35×）—— 单遍融合核已完成；P3 闸门评估结论：**NO-GO**（顺序 EMA 递推 IIR，不可向量化）
3. ~~WMA (3.91×→0.94×)~~ / ~~DEMA (1.48×→0.73×)~~ / ~~TEMA (0.96×→0.46×)~~ / ~~MIDPRICE (1.84×→0.56×)~~ / ~~BBANDS (1.02×→0.58×)~~ —— **P2-1/P2-2/P2-3/P2-4 已完成**
4. **LINREG / CORREL / WILLR / STOCH** —— P2-5 由 O(n·period) 朴素扫描换为滑动 O(n)（LINREG/CORREL 滑动求和/交叉积；WILLR/STOCH 复用单调队列），渐近 ~20×；原生 C 对照未接线（零-FFI 精神）

**P2-3 结论（2026-08-10）**：`core::wma` 由 O(n·period) 朴素扫描换为 O(n) 滑动递推 —— 维护朴素窗口和
`sw`（`sw += x[i]-x[i-period]` 在 O(1) 内滑动），并以闭式 `W[i] = W[i-1] + period·x[i] - sw[i-1]` 更新
加权累加，消除每窗口 `period` 次重复乘加；首个窗口沿用朴素求和作种子（与历史实现逐项对齐）。
9.93→2.11 ns/elem（0.93× C，已≈持平）。新增 `core::tests::wma_matches_naive` 单测，对多组
`(n, period)` 含单调/随机/重复值输入逐位验证与朴素版相等（容差 1e-9），作为零偏差护栏。

**P2-1 结论（2026-08-10）**：新增 `core::nested_ema_with_output` 单遍嵌套 EMA 级联（const-generic `L` + `combine` 闭包），DEMA/TEMA/T3 经 `_with_output` 委托调用，消除 2/3/6 次中间 `Vec` 分配与独立扫描；数值逐项相等（黄金向量 1:1，ADR 0005）。T3 自 7.85× 降至 1.35×。

**P2-2 结论（2026-08-10）**：`core::rolling_extreme` 由 O(n·period) 朴素扫描换为单调队列 O(n)
（`VecDeque`，并列极值取最右以匹配朴素 tie-break，逐位相等），并新增合并 `rolling_minmax`
单遍双队列供 `midpoint` 使用（对齐 TA-Lib `MINMAXINDEX`）。MIDPRICE 22.81→7.30（0.60×，快于 C），
MIDPOINT 22.55→6.88（2.26×）。新增 `core::tests::rolling_extreme_matches_naive` 单测，对多组
`(n, period)` 含重复极值输入逐位验证与朴素版相等，作为零偏差护栏。

**P2-4 结论（2026-08-10）**：`bbands` 中轨由两段（先 `rolling_mean` 再 `rolling_var`）换为单遍融合
`rolling_mean_var`（共享滑动 `sx`+`sxx`，闭式方差），消除第二趟窗口扫描。5.61→3.02 ns/elem
（0.58× C，已快于 C）。新增 `bbands` 零偏差护栏（种子沿用朴素求和、递推仅重排浮点运算顺序，与
既有 `rolling_mean`/`wma`/`rolling_var` 同构）。

**P2-5 结论（2026-08-10）**：LINREG 家族（`linear_reg`/`_angle`/`_intercept`/`_slope`/`tsf` 共享
`linreg_core`）与 `correl` 由每窗口重算求和/交叉积换为滑动 O(n) 递推（`sxy[i]=sxy[i-1]+period·x[i]−sy[i-1]`
等）；`willr` 与 `stoch`/`stoch_f` 改用既有单调队列 `rolling_min`/`rolling_max`（并列取最右，与朴素一致）。
渐近 O(n·period)→O(n)（理论 ~period=20×，实测因新核常数略低）。均通过新增 `*_matches_naive` 单测逐位验证。

**P3 SIMD 评估结论（2026-08-10）**：按 ADR 0010 闸门（自动向量化失败 且 比原生 C 慢 >20%），MIDPOINT(2.26×)
与 T3(1.35×) 虽形式上命中双条件，但瓶颈本质不可向量化 —— MIDPOINT 为数据依赖单调双队列（C 侧 `TA_MIDPOINT`
同为 `MINMAXINDEX`、无 SIMD），T3 为严格顺序 EMA 递推（IIR）。**判定 NO-GO**：不引入 `unsafe`/SIMD/
`nightly`，维持现状；二者为已知结构性权衡，非缺陷。

目标：将剩余热路径 Rust/C 比值压到 **≈1.0**。在 P2-1~P2-5 完成后，8 项 C 接线基准中 6 项已快于/持平 C，
仅 MIDPOINT/T3 因 SIMD NO-GO 维持现状（已知权衡）。P3 评估闭环。

## 架构深化 P1 候选②：MINMAX 单遍化（2026-08-10）

> 源自 `/improve-codebase-architecture` 评审候选②：将 `math_ops::minmax` 的两遍独立极值扫描
> 合并为单遍。改动：`math_ops::minmax` 由 `rolling_min`+`rolling_max` 改为复用
> `core::rolling_minmax`（P2-2 已验证的单遍双队列 O(n)，最右 tie-break、前导 `NaN` 与分别调用逐位相等）。
> 受双基准（精度 1:1 + 性能最大化）约束，实测后诚实记录如下。

- **精度**：1:1 零偏差。通过 `tests/fixtures/minmax_basic.json` 黄金向量 + 全量 `cargo test`
  （0 失败；含 144 模式 + 21 doctest + 各模块单测）。
- **性能实测**（ns/elem，release，`N=1e6`，`PERIOD=20`，`ITERS=20`，点测 ±5%）：

  | 实现 | ns/elem | 说明 |
  |------|--------:|------|
  | Rust MINMAX 单遍（新） | **6.76** | 复用 `core::rolling_minmax` |
  | Rust MAX+MIN 两遍（改动前等价） | **6.96** | 原 `rolling_min`+`rolling_max` |
  | C MINMAX（原生，`bench-c`） | **3.11** | TA-Lib 0.7.1 |

- **结论（诚实）**：设想的"两遍→一遍 2× 提速**未出现**"。`minmax` 的成本在**两个单调队列的维护**
  而非数据遍历；单遍双队列与"两次独立单队列扫描"的总队列操作量相同，少一遍数据扫描省下的开销
  落在 ±5% 噪声内（6.96→6.76，仅 ~3%）。故本改动**性能中性、精度零退化**，真实收益是
  **消除重复极值逻辑、把 `minmax` 钉在已验证的 `core` 核上（与 `midpoint` 同源）**——降低未来
  极值语义漂移风险（服务准确性基准的长期稳定性），**而非性能优化**。
- **双基准判定**：准确性满足（非妥协项）、性能未退化（中性）、且提升核唯一性 → 保留；
  但**不计入性能优化收益**。perf 口径下 Rust/C = 2.17×，仍未达 ≈1.0 目标，且瓶颈同为单调双队列
  （与 MIDPOINT 同类，P3 SIMD 闸门 NO-GO）。

## 架构深化 P3 候选③：极值索引单遍化（2026-08-10）

> 源自 `/improve-codebase-architecture` 评审候选③：消除 `math_ops` 索引变体（`max_index` /
> `min_index` / `minmax_index`）对极值逻辑的本地重推导（`core` 可见性 seam），并把并列 tie-break
> 显式化（索引变体最左、值变体最右，二者现同处 `core`）。改动：新增 `core::rolling_extreme_index`
> （单遍单调队列 O(n)、最左 tie-break、前导 `0.0`），`math_ops` 三个索引函数统一复用之，替换原有
> 朴素 `O(n·period)` 嵌套扫描。受双基准（精度 1:1 + 性能最大化）约束，实测后诚实记录如下。

- **精度**：1:1 零偏差。新增 `core::rolling_extreme_index_matches_naive_leftmost`（对朴素最左扫描逐项相等，
  含重复极值并列场景）；`tests/fixtures/{max_index,min_index,minmax_index}_basic.json` 黄金向量 +
  全量 `cargo test`（0 失败；含 144 模式 + 21 doctest + 各模块单测）。
- **性能实测**（ns/elem，release，`N=1e6`，`PERIOD=20`，`ITERS=20`，点测 ±5%）：

  | 实现 | ns/elem | 说明 |
  |------|--------:|------|
  | Rust MAX_INDEX 单遍（新） | **3.43** | 复用 `core::rolling_extreme_index` |
  | Rust MAX_INDEX 朴素 O(n·period)（改动前等价） | **6.55** | 原本地嵌套扫描 |
  | Rust MIN_INDEX 单遍（新） | **3.31** | 同上 |
  | Rust MINMAX_INDEX 两遍 O(n)（新） | **6.79** | 两次 `rolling_extreme_index` |
  | C MAXINDEX / MININDEX / MINMAXINDEX（原生，`bench-c`） | **2.56 / 2.62 / 3.09** | TA-Lib 0.7.1 |

  （C 对照的 checksum 累加为本机 C 接线读数偏差，仅作时间量级参考；时序数值有效。）
- **结论（诚实）**：本次是**真实性能收益**——单索引 `O(n·period) → O(n)`，实测 **~1.9× 提速**
  （6.55 → 3.43 ns/elem），Rust/C 差距由 ~2.56× 收窄到 ~1.34×。不同于候选②（性能中性），候选③
  把"少一遍遍历"的红利落到了实处，因为旧实现确实是 `period` 倍的嵌套比较、瓶颈就是遍历而非队列。
- **范围/局限**：索引函数属**纯对外 API**，未被 crate 内部任何指标调用，故收益归于直接调用方，
  不在 crate 内部热路径上。`minmax_index` 当前为**两次 O(n) 遍历**（6.79）；若要逼近 C 的 3.09，
  可进一步做单遍双队列（min+max 同扫），但属常数级微优、收益有限，非必要。
- **双基准判定**：准确性满足（非妥协项）、性能提升（真实、已实测）、且修复 `core` 可见性 seam +
  显式 tie-break → 保留并计入性能优化收益。

## 架构深化 候选①：指标脚手架接缝归一化（indicator! 宏，2026-08-11）

> 源自 `/improve-codebase-architecture` 评审候选①：用零成本 `macro_rules! indicator` 把每个指标
> 函数三件套里重复的「等长 `f64::NAN` 缓冲分配 + 转发到 `*_with_output` 内核」胶水归一化。完整决策
> 见 [ADR 0011](docs/adr/0011-indicator-scaffold-seam.md)。改动分阶段（Phase 1a/1b/1c）在
> **度量前置双闸门**（精度 1:1 + 性能 A/B median |Δ| ≤ ±5%）下推广，受项目双重基准（准确性 > 性能）约束。

- **精度**：逐项零偏差。宏仅替换外层胶水，`*_with_output` 热路径体字节级不变；宏生成 `func` 与手写
  数值逐项 1:1（黄金向量闸门：`cargo test` 全量绿灯，161/161；含宏内 `doctest`）。
- **性能实测方法（A/B 闸门，measure-first 协议）**：新增 `benches/math_trans_bench.rs` /
  `benches/stat_bench.rs` / `benches/phase1c_bench.rs`（零依赖 `Instant` harness，已注册 `Cargo.toml`
  `[[bench]]`）。每条 A/B 对「宏生成 `func`」与手写基线（重构前字面 body）做 **预热 + 交错 11 轮
  （TRIALS=11）+ 取中位数**（`N = 1_000_000`，`ITERS = 50`）；断言 **median |Δ| ≤ ±5%**。
  经验修正：单发 `Instant` 受 CPU 电源态 / 缓存预热 / 测量顺序影响，单次 Δ 可 ±10%（曾出现宏反而快
  10% 的纯噪声），故以 median |Δ| 判定而非单次 Δ。

  | Phase | 模块 / 函数 | median 最大 |Δ| | 闸门 |
  |-------|-------------|------------:|------|
  | 1a | `math_trans` 15（单输入/单输出/逐元素） | **2.97%** | PASS |
  | 1b | `stat` 7（单输入，N 末尾默认臂） | **0.11%** | PASS |
  | 1c | `math_ops` 9 + `volatility` 3 + `price_transform::avgdev` 1 | **0.21%** | PASS |

- **Phase 1c 关键发现（measure-first 协议抓出的真实回归，非噪声）**：宏统一用 `vec![f64::NAN; n]`
  初始化输出缓冲；而 `avgprice` / `medprice` / `typprice` / `wclprice` 这 4 个**逐元素全覆写、无前导
  NaN、无默认参数**的函数，其手写原版用 `vec![0.0_f64; n]`。实测 NAN 初始化比零初始化慢（隔离微基准
  `avgprice` NAN vs ZERO = **+34.7%**、`add` = **+22.2%**）；A/B 闸门报 `avgprice` median |Δ| =
  **16–17%**（远超 ±5%）。这 4 个函数**既无前导 NaN 需求、也无默认参数、宏对其零收益**，仅因统一 NAN
  初始化而变慢 —— 故**刻意回退手写**（`0.0_f64` 原生初始化），不纳入宏。其余宏生成函数（含 `trange`/
  `atr` 等同样全覆写者）因计算/拷贝主导、或本身即需 NAN 初始化，median |Δ| 均在 ±0.3% 内，闸门通过。
- **双基准判定**：准确性满足（非妥协项）、性能未退化（median |Δ| 均 ≤ 5%，中性）、且消除约 146 处
  单输出脚手架、降低复制粘贴漂移面 → 保留；**不计入性能优化收益**（宏为「消除重复」而非「改变算法」，
  展开体与手写字节级相同）。`avgprice` 等 4 函数已从 A/B 移除（已回退手写），其回归由隔离微基准单独确认。

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

---

## 状态更新（2026-08-10，末次刷新）

- **P4 里程碑已完成**：adaq-talib 现已实现 TA-Lib 0.7.1 的**全量 161 个对外函数**（含 61 个
  CDL 模式识别），由 `tools/reconcile.py` 自动对账确认 161/161 覆盖、live 交叉校验 0 偏差；
  `cargo test` **308 项测试全绿、0 失败**。
- 本基线快照（ns/elem 数值矩阵）聚焦**数值型热路径**（重叠/动量/波动率等），未含模式识别模块——
  CDL 函数为**整数输出的 O(n) 蜡烛比较**，手写即快、几乎不依赖重原语，不属于当前 ns/elem 优化对象；
  其正确性护栏为黄金向量 1:1（`tests/pattern_*_test.rs`，共 144 项测试全绿），而非性能基准。
- **P2 全阶段完成（P2-1~P2-5）**：共享原语（嵌套 EMA 融合核、单调队列、WMA 滑动递推、BBANDS 单遍
  融合、LINREG/CORREL/WILLR/STOCH 滑动 O(n)、`_with_output` 原地 API）均已落地并通过 `*_matches_naive`
  零偏差护栏。8 项 C 接线基准中 **6 项已快于/持平 C**（SMA/BBANDS/DEMA/TEMA/MIDPRICE 快于 C，WMA ≈ 持平）；
  仅 **MIDPOINT(2.26×)** 与 **T3(1.35×)** 两项仍慢于 C。
- **P3 SIMD 评估闭环（结论 NO-GO）**：MIDPOINT（数据依赖单调双队列）与 T3（顺序 EMA 递推 IIR）虽形式上
  命中 ADR 0010 闸门（自动向量化失败 且 慢 >20%），但瓶颈本质不可向量化（C 侧 `TA_MIDPOINT` 同为
  `MINMAXINDEX`/`无 SIMD`）。判定 **NO-GO**：不引入 `unsafe`/SIMD/`nightly`，维持现状；二者为KNOWN
  结构性权衡，非缺陷。性能优化工作至此收口（见 `docs/perf-verify-report.md`）。

- **2026-08-11 更新（指标脚手架 / 0.1.5）**：新增 0.1.5 的 `indicator!` 宏脚手架（架构深化候选①，
  Phase 1a/1b/1c），在 measure-first 双闸门下推广；A/B 闸门（预热 + 交错 11 轮 + 中位数，median |Δ| ≤ 5%）
  全 PASS（1a 2.97% / 1b 0.11% / 1c 0.21%），黄金向量 161/161 仍 1:1。详见上方「架构深化 候选①」小节
  与 [ADR 0011](docs/adr/0011-indicator-scaffold-seam.md)。此为「消除重复」重构，非性能优化，故不计入
  性能收益，但证明零成本（展开体与手写字节级相同）。
