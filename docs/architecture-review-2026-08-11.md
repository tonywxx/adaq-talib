# adaq-talib 架构评审与改进建议报告

- **日期**：2026-08-11
- **范围**：`src/` 全部模块、`Cargo.toml`、CLI（`examples/demo.rs`，原 `src/main.rs`）、测试与基准布局
- **方法**：以"深模块"（Deep Module）视角审视——关注**接口**相对于**实现**的杠杆（Depth）、**缝**（Seam）的位置、**适配器**（Adapter）的复用，以及**局部性**（Locality）。结论先行：当前架构整体健康、已具备良好的深度；以下是在边际上进一步提升深度与局部性的具体建议，按优先级排序。

---

## 1. 总体评估

adaq-talib 是一个**纯 Rust、零依赖、零 FFI** 的 TA-Lib 0.7.1 指标复刻库。分层清晰：

| 层 | 内容 | 角色 |
|---|---|---|
| 公开指标模块 | `overlap` / `momentum` / `stat` / `volatility` / `volume` / `price_transform` / `cycle` / `math_ops` / `math_trans` / `pattern` | 库的对外接口（约 80+ 公开函数） |
| 内部原语模块 | `core`（含 `defaults`、`MonoQueue`、滚动聚合、EMA 族、极值队列） | `pub(crate)` 共享数学内核 |
| 错误模型 | `error::TaError` | 对应 `TA_RetCode` 的语义映射 |
| CLI | `examples/demo.rs` | demo 示例（非默认二进制），约 50 个指标演示 |
| 验证 | `tests/` 集成测试 + `benches/`（17 个基准） | 黄金向量对照 + 性能 |

规模：`src/` 约 8900 行；`core/mod.rs` 单文件 ~930 行承载绝大多数共享数值内核。

**结论**：模块边界基本合理，公开接口小（切片入参、`Result<_, TaError>` 出参）、实现深（大量手写的零偏差数值算法），已符合"深模块"原则。主要可改进点集中在**局部性**（重复定义）、**浅模块占位**（defaults）、**接口表面积管理**（三连发 API）与**内部缝的进一步切分**。

---

## 2. 用"深模块"语言重述现有结构（亮点）

以下几点已经做对了，作为后续改进的基线，不应破坏：

- **零拷贝输出缝（`*_with_output`）**：公开函数成对提供 `fn x(&[f64], p) -> Result<Vec<_>>` 与 `x_with_output(..., out: &mut [f64])`。后者让调用方预分配缓冲、库内以切片视图递推，既消除热路径分配又使编译器能消掉边界检查（见 `core/mod.rs` 中 EMA/WMA 的 slice-view 注释）。这是一个**真实的深模块缝**——调用方与测试穿过同一 seam，且性能收益由同一接口兑现。✅
- **`nested_ema_with_output<const L, F>`**：一个 `const` 泛型 + `combine` 闭包的核，统一服务 DEMA / TEMA / T3，单遍 O(n) 持有 O(L) 标量状态。这是**高杠杆**抽象——一处实现，多指标复用，数值逐项相等。✅
- **`parallel.rs`**：接口仅 `parallel_index_map` / `parallel_index_map_2`（带重叠播种），实现深（分块、thread::scope、边界写回）。已真实接入 `overlap` / `momentum` / `math_ops` 的窗口极值路径，并非死代码。✅
- **`pattern` 模块**：61 个 K 线形态经共享 `CandleAvg` 助手与蜡烛原语驱动，小接口（`open/high/low/close -> Vec<f64>`）、深实现。✅

---

## 3. 发现与改进建议（按优先级）

### P1 — `momentum.rs` 重复定义 `check_eq_len`（局部性违例）

- **现状**：`crate::core::check_eq_len` 已是 `pub(crate)`，并被 `volatility`、`stat`、`volume`、`math_ops`、`price_transform` 共 5 个模块复用。但 `momentum.rs:23` 又**重新实现了一份签名、逻辑完全相同的私有 `check_eq_len`**（`grep` 确认：全仓仅两处定义，momentum 未复用 core 的）。
- **问题**：同一逻辑存在于两处。一旦长度校验语义需要微调（例如允许 `out` 长度不一致时自动 resize，或新增更精确的错误文案），必须记得同时改两处——局部性被破坏。
- **改进**：删除 `momentum.rs:23` 的副本，改为 `use crate::core::check_eq_len;`（文件顶部已 `use crate::core::{...}`，仅漏引 `check_eq_len`）。
- **收益**：消除重复、单点维护；无公开 API 变化、零风险。
- **验证**：`cargo test` 全绿即可。

### P2 — `core/defaults.rs` 是浅模块 / 占位噪声

> **执行情况更正（2026-08-11）**：实际核查后，原"约 95 个占位"判断**不成立**——`defaults.rs` 共 52 个常量，**全部**已被引用（已发布指标的 `*_default` 入口，以及测试/基准套件）。原文件头注释"仅 `DEFAULT_TIME_PERIOD` 被使用、其余为占位"已**过时**。因此 P2 的实际改动只是**修正过时注释**（说明常量均由 `*_default` 或测试/基准引用，并保留模块级 `#![allow(dead_code)]` 以容忍仅被测试/基准使用的少数常量），**未删除任何常量**。该模块并非浅模块，原 P2 的"占位噪声"前提被推翻。

- **现状（修正后）**：`defaults.rs` 集中定义 TA-Lib `optIn*` 默认值（52 个常量），每个都被对应指标的 `*_default` 引用，或被测试/基准引用。模块级 `#![allow(dead_code)]` 仅用于容忍"仅被测试/基准（独立 crate）引用"的少数常量。
- **改进（已执行）**：删除过时的"占位"注释，改写为准确的引用说明；保持常量集中、零删除。
- **收益**：消除误导性的过时文档，避免读者误以为存在大量死代码。
- **风险/权衡**：无。公开 API 不变。

### P3 — `core/mod.rs` 是"上帝原语模块"（内部缝可切分）

- **现状**：`core/mod.rs` 单文件混合了 4 类职责：
  1. 窗口聚合：`rolling_mean` / `rolling_sum` / `rolling_var` / `rolling_mean_var`
  2. EMA 族：`ema` / `ema_with_output` / `nested_ema_with_output` / `ema_wilder`
  3. 单调队列极值：`MonoQueue` + `rolling_extreme` / `rolling_minmax` / `rolling_extreme_index`
  4. TA 特定核：`true_range`
- **问题**：四类逻辑共享"internal seam"但彼此独立。单文件 ~930 行，单测也集中于此，定位与聚焦成本高。
- **改进（纯内部重构，公开 API 不变）**：拆为
  - `core::window` —— 通用滑动窗口聚合（mean/sum/var，与"指标"无关，可独立测试）
  - `core::ema` —— EMA / Wilder / nested_ema
  - `core::extreme` —— `MonoQueue` + 极值/索引/双队列（含 `rolling_minmax`）
  - `core::kernel` —— `true_range`（TA 专属，耦合 OHLC 三元组）
  - `core/mod.rs` 仅 `pub use` + 保留 `check_eq_len`
- **收益**：提升**局部性**——改 EMA 种子策略不波及极值队列；各子模块单测聚焦；新内核有明确落点。
- **风险/权衡**：低风险（全部 `pub(crate)`，不触公开接口）；需同步调整各指标模块的 `use crate::core::{...}` 路径。建议配合 P1 一并做。

### P4 — 公开 API 的三连发表面积（`fn` / `_with_output` / `_default`）

- **现状**：几乎每个指标暴露 3 个入口：`sma`、`sma_with_output`、`sma_default`（约 80 个指标 × 3 ≈ 240 个公开符号）。这是**有意的**设计（ADR 0001 模型 B + ADR 0007）：`_with_output` 给零拷贝热路径，`_default` 给 TA-Lib 默认参数人体工学。
- **张力**：从"深度"看，接口比必要的大 3 倍。从"杠杆"看，三者各自兑现真实收益（性能 / 易用性），且被同一批测试覆盖——并非浅层透传。
- **建议（增量、可逆，非强制）**：
  - **保持现状**作为主路径；当前设计合理，不建议破坏性大改。
  - 若希望在不破坏兼容的前提下收敛表面，可新增一层"选项结构体"入口作为**补充**而非替代，例如：
    ```rust
    #[derive(Default)]
    pub struct SmaOpt { pub period: usize, /* 可选预分配 out */ }
    pub fn sma_with(values: &[f64], opt: SmaOpt) -> Result<Vec<f64>, TaError>;
    ```
    让 `_default` 等于 `sma_with(v, SmaOpt::default())`、`_with_output` 复用其内部。这把"默认参数"语义从**每个函数一份副本**收敛到**一个 `Default` 派生**，消除默认值散落。
- **收益（若采用）**：默认参数集中、少一处漂移；公开符号可按需逐步迁移。
- **风险/权衡**：属于 API 演进，需决策是否纳入公开契约；在达成里程碑（全量指标）后做一次统一收口更划算，当前不必急于动手。

### P5 — `main.rs` CLI 的"双重登记"耦合

- **现状**：每新增一个指标，要在 `main.rs` 的 `match` 中加一个分支，**且**在末尾 `other =>` 的 usage 字符串中手动补列一次支持列表（两处必须同步）。`main.rs` 已 475 行、约 50 个分支。
- **问题**：典型局部性漏洞——同一事实（"支持哪些指标"）存在于两处，易漂移（忘了更新 usage 串）。
- **改进**：表驱动注册，把"派发"与"列出支持集"合并为单一数据源：
  ```rust
  struct Demo { name: &'static str, run: fn(&[f64]) -> Vec<f64> /* 或泛型 runner */ }
  const REGISTRY: &[(&str, Demo)] = &[ ("sma", ...), ("ema", ...), ... ];
  ```
  `match` 改为查表，`unknown` 分支直接 `REGISTRY` 打印全部名字。
- **收益**：加指标只改一处；usage 永远与派发一致。
- **风险/权衡**：`demo_*` 辅助函数签名不一（单输入 / OHLC / OHLCV / MACD 等），表驱动需一层 `enum Runner` 或 trait 归一；中等工作量。因是 demo 二进制，**优先级中**。

**本次处置（2026-08-15）**：用户选择"深化候选 4 = 移除 `src/main.rs` 并迁至 `examples/demo.rs`"。该动作把 demo 从默认 `cargo build` 与发布二进制中剥离，库构建不再耦合此 CLI；"双重登记"问题随 demo 不再是库契约而降级（非阻塞）。表驱动注册（单一数据源 REGISTRY）仍是可选后续优化，但优先级进一步下降——demo 仅由 `cargo run --example demo -- <name>` 触发，不影响库交付。用法文档（README / README.zh-CN / api-conventions）已同步为 `examples/demo.rs` 与 `cargo run --example demo -- <name>`。

### P6 — `pattern` 模块：确认是否为代码生成产物

- **现状**：61 个形态分布在 `batch_1.rs`..`batch_8.rs`（每文件 600–1000 行），依赖 `mod.rs` 的 `CandleAvg` 与蜡烛原语。结构良好、深模块。
- **观察**：如此大规模、机械性强的 1:1 映射，强烈暗示由脚本从 TA-Lib 源生成。但仓库 `tools/` 下未见对应的生成器。
- **建议**：若确为生成，请将生成器纳入 `tools/` 并提交，保证可复现、避免手工改动后与原版漂移；并在 `pattern/mod.rs` 标注"此文件由 tools/xxx 生成，勿手工编辑"。若是手写，则在 ADR 中记录"为何不生成"以保留决策依据。
- **收益**：长期可维护性、与上游 TA-Lib 的安全同步。
- **风险/权衡**：纯流程改进，无运行风险。

### 附：测试/基准布局（总体良好，小建议）

`tests/` 已有 21 个集成测试 + `core` 内联测试 + 17 个 `benches/`，覆盖黄金向量对照。两点小观察：
- 多处 `*_with_output` 重复做 `out.len() != values.len() -> BadParam` 校验（如 `sma_with_output`）。可考虑在 `core` 提供一个 `checked_out(values, out)` 包装，统一该校验与 `vec![NAN; n]` 初始化样板，减少各公开函数内的重复。收益小、风险低，可并入 P3 一并处理。
- `benches/` 与 `tools/` 的对照脚本职责清晰（ADR 0003/0004），保持不变。

---

## 4. 建议落地顺序与执行状态

| 顺序 | 项 | 风险 | 收益类型 | 公开 API 是否变化 | 状态 |
|---|---|---|---|---|---|
| 1 | P1 删 `momentum.rs` 的 `check_eq_len` 副本 | 极低 | 局部性 | 否 | ✅ 已执行 |
| 2 | P3 切分 `core/mod.rs` 为 `window`/`ema`/`extreme`/`kernel` | 低 | 局部性 | 否 | ✅ 已执行 |
| 3 | P2 修正 `defaults.rs` 过时注释（非删占位） | 低 | 可读性 | 否 | ✅ 已执行 |
| 4 | P5 迁出 `main.rs` → `examples/demo.rs`（表驱动注册留待可选） | 中 | 局部性 | 否（demo 非库契约） | ✅ 已执行（2026-08-15） |
| 5 | P6 落实 pattern 生成器入仓 | 低 | 可维护性 | 否 | 待办 |
| 6 | P4 选项结构体入口（达全量里程碑后统一收口） | 中 | 接口收敛 | 是（新增，非破坏） | 待办 |

P1–P3 已在一个 PR 内完成（纯内部重构，全部 `pub(crate)`），并以 `cargo build` / `cargo test` / `cargo build --features parallel` / `cargo build --features bench-c` 验证：**零偏差不变、警告数与基线一致（504）、全部测试通过（默认 + parallel 特性）**。

### 深化候选 ①-⑤ 执行状态（/improve-codebase-architecture 2026-08-15 会话）

> 本会话用 /improve-codebase-architecture 对库做深化扫描，产出 5 个候选。逐项核对真实代码与 `all161_results_final.csv` 后处置如下。

| 候选 | 内容 | 判定 | 状态 |
|---|---|---|---|
| ① | `indicator!` 宏铺开（momentum/overlap 手写胶水迁宏） | 零风险、去重 | ✅ 已执行（2026-08-15 深夜；黄金向量 + 代表 A/B 双闸通过） |
| ② | core 原语补全 + Wilder 递推收口（`wilder_step` / `wilder_step_sum` / `wilder_with_output`） | 安全中等 | ✅ **已执行（2026-08-17）**：momentum 的 5 处内联 Wilder 递推收口到 `core::ema` 原语（均值形 `wilder_step` 用于 rsi/cmo/adx；求和形 `wilder_step_sum` 用于 dm_tr/adx_adxr_fused/dx_from_candles 的 ±DM/TR，两形分别保留以保证 +DI/−DI 逐位一致）；`ema_wilder` 改为委托 `wilder_with_output`。黄金向量 31/31 1:1 + 全量测试绿；性能 −12%~−46%（median-of-9，`momentum_wilder_bench`），详见 `docs/validation-and-performance-report.md` §3.6 |
| ③ | pattern 前导和 `macro_rules!` 去重 + 文档对齐 | 前提不成立 | ❌ **已关闭（2026-08-15）**：9 个慢 pattern 中 7 个已内联却仍 <1×，余下差距为硬约束（safe/无 SIMD/单线程）下 Rust-vs-C codegen 地板，macro 不针对成因（slow-9 内仅 `cdl_harami` 真用 `CandleAvg`）；`cdl_engulfing` 0.43× 经核对为 C 侧测量异常（C 0.93 ns/elem 异常偏低，Rust 2.18 正常）。KPI 重定义为「消除伪慢 + 可并行子集 >2×」 |
| ④ | `main.rs` → `examples/demo.rs` | 局部性 | ✅ 已执行（2026-08-15；库构建不再耦合 demo CLI） |
| ⑤ | `parallel.rs` 两分块原语去重 | 默认关闭、无交付影响 | ⏸ 跳过（skip，低优先级） |

> ③ 关闭依据：跑 `cargo bench --bench cdl_bench`（Rust-only，N=100k）对照 `all161_results_final.csv` 的 c_ns，确认 8 个已内联 pattern 的 <1× 是 C 在轻计算分支循环上 1.3–1.8× 更快的真实 codegen 差距；蜡烛原语（`real_body`/`upper_shadow`/`lower_shadow`/`high_low_range`/`candle_color`）均已 `#[inline]` 单行宏，无安全可消除的冗余。收回差距需 `unsafe get_unchecked` 或 SIMD/并行轨，均超出当前硬约束。

---

## 5. 一句话总结

架构已经"深"，主要欠账在**局部性**（重复定义、双重登记）与**内部缝的粒度**（`core` 单文件过载，已拆分）；`defaults.rs` 经核查并非浅模块、仅注释过时已修正；公开 API 的"三连发"是有意且合理的性能/人体工学缝，不宜破坏性改动，仅在全量落地后考虑用选项结构体做增量收口。
