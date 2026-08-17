# 变更日志 / Changelog

本文件从 `README.zh-CN.md` 抽出为独立变更日志（2026-08-17）。每条均在
[度量前置双闸门](docs/adr/0010-performance-strategy.md)（黄金向量 1:1 + A/B `cargo bench` 中位数）下验证，
零偏差承诺不变（[ADR 0005](docs/adr/0005-error-tolerance.md)）。无新增公开 API、无弃用、无依赖变更（[ADR 0002](docs/adr/0002-release-scope-milestones.md)），除非条目另有说明。

---

### 0.1.8

- **架构深化收尾（度量前置双闸门）**：候选②（Wilder 递推接缝收口，见 0.1.7）在「黄金向量 1:1 + A/B
  `cargo bench` median-of-9」双闸门下**采纳**——Wilder 家族 7 个指标（`rsi`/`cmo`/`plus_di`/`minus_di`/
  `dx`/`adx`/`adxr`）提速 **−12% ~ −46%**。候选③（移除 / 去重默认关闭的 `parallel` 特性）与候选④
  （pattern 运行时 / 文档导航索引）经同一闸门**否定**：二者均为结构重构，不改变库的热计算核，无法证明
  默认构建的性能提升（候选④ 的 kernel 内共享偏移变体还会像候选① CandleAvg 接缝那样回归 170–291%），
  故不采纳。零偏差保持不变（ADR 0005）。
  - Architecture-deepening wrap-up: candidate② (Wilder recurrence consolidation, see 0.1.7) **adopted**
    under the measure-first gate (golden vectors 1:1 + A/B `cargo bench` median-of-9) — the 7 Wilder-family
    indicators are **−12% ~ −46%** faster. Candidate③ (remove/dedup the default-off `parallel` feature) and
    candidate④ (pattern runtime/doc navigation index) are **rejected** by the same gate: both are structural
    refactors that don't touch the hot compute kernels, so they can't demonstrate a default-build perf gain.
- **文档与发布整理**：将 Changelog 抽出为独立 `changelog.zh-CN.md`（本文件，中文）与 `changelog.md`（英文）；性能文档
  （[`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md) §3.6、
  [`docs/perf-verify-report.md`](docs/perf-verify-report.md)、[`benches/BASELINE.md`](benches/BASELINE.md)）
  记录 Wilder 提速与 median-of-9 基准护栏（`benches/momentum_wilder_bench.rs` / `benches/cdl_bench.rs`）；
  README 优化表新增 Wilder 行。
  - Docs & release hygiene: Changelog extracted into standalone `changelog.zh-CN.md` (this file, Chinese) and `changelog.md` (English); the performance docs record the
    Wilder speedup and the median-of-9 bench guard (`benches/momentum_wilder_bench.rs` / `benches/cdl_bench.rs`);
    the README optimization table gains a Wilder row.
- **发布**：版本号提升至 `0.1.8`。无新增公开 API、无弃用、无依赖变更（[ADR 0002](docs/adr/0002-release-scope-milestones.md)）。
  - Release: version bumped to `0.1.8`. No new public API, no deprecations, no dependency changes.

### 0.1.7

- **Wilder 递推接缝收口（架构深化候选②）**：将 `momentum.rs` 中 5 处内联 Wilder 递推收口到 `core::ema` 原语 —— `rsi`/`cmo`/`adx` 用均值形 `wilder_step(prev, x, k)`，`dm_tr`/`adx_adxr_fused`/`dx_from_candles` 的 ±DM/TR 用求和形 `wilder_step_sum(prev, x, k)`（两种形式分别保留，因 `period` 因子仅在 ±DI 比值中相消，盲目统一会破坏 +DI/−DI）；`ema_wilder` 改为委托新增的零拷贝 `wilder_with_output`。在**度量前置双闸门**（黄金向量 1:1 + A/B `cargo bench` median-of-9）下通过：31/31 动量黄金向量逐项 1:1、全量 `cargo test` 全绿（含经重构 `ema_wilder` 的 ATR/NATR）；`rsi`/`cmo`/`plus_di`/`minus_di`/`dx`/`adx`/`adxr` 提速 **−12% ~ −46%**（`momentum_wilder_bench`，N=10 万，9 轮中位数）—— 性能提升来自用预计算的 `k = 1/period` 乘法替代热循环内每步的 `/p` 浮点除法。
- **基准套件**：新增 `benches/momentum_wilder_bench.rs`（Wilder 家族微基准，median-of-9）；`benches/cdl_bench.rs` 由单发 `Instant` 加固为 **median-of-9**（单发噪声可达 ±10%，已证会误报）。
- **发布**：版本号提升至 `0.1.7`。无新增公开 API、无弃用、无依赖变更（[ADR 0002](docs/adr/0002-release-scope-milestones.md)）。面向用户的行为、调用形态与 `cargo test` / `cargo bench` 工作流均不变。

### 0.1.6

- **蜡烛形态内核 —— `real_body` 冗余重算去重（perf(pattern)）**：20 个蜡烛形态内核现复用各条件里已算好的 `cur_avg_*` 滑动窗口值，而非重新计算 `real_body(open[i], close[i])`。纯重排 —— 无算术改动 —— TA-Lib 0.7.1 黄金向量保持逐位一致（全部 144 个蜡烛集成测试通过）。对照原基线的控制校正 A/B（3 轮中位数，借未改动对照组校正环境漂移）：12 个明确提速（如 `cdl_closingmarubozu` −57%、`cdl_marubozu` −36%、`cdl_stalledpattern` −27%、`cdl_counterattack` −24%）、3 个持平（`cdl_belthold` / `cdl_longleggeddoji` / `cdl_eveningstar`）、5 个表观「回归」（`cdl_3starsinsouth` / `cdl_3whitesoldiers` / `cdl_abandonedbaby` / `cdl_eveningdojistar` / `cdl_morningstar`）经判定为环境噪声 —— 去掉一次重算不可能拖慢函数、且黄金向量完全相同，故全部保留。另含 `cdl_harami` CandleAvg 合并（已验证提速）与 `cdl_homingpigeon` / `longline` / `shortline` 影线 + 实体去重。
- **指标脚手架推广（`indicator!` 宏）—— 一致性**：将 `midprice`、`sar`、`sarext`、`avgprice`、`medprice`、`typprice`、`wclprice`、`ad`、`adosc`、`obv` 迁移至零成本 `indicator!` 宏（0.1.5 引入），移除冗余的错误处理 / 输出初始化样板；各函数保留其详尽的中英双语文档注释。模式识别模块一并迁移。输出仍为黄金向量 1:1。
- **蜡烛形态模块 —— 可读性重构**：移除各 batch 文件中算术表达式多余括号，合并均值计算的变量初始化，并为 `pattern/mod.rs` 中未使用赋值 / 变量显式加 `#[allow(...)]`，使严格编译无警告。
- **CI**：`.github/workflows/ci.yml` 与 `release.yml` 中 `actions/checkout` 升级至 **v5**。
- **基准套件**：新增 `benches/cdl_bench.rs` 并扩展 `benches/phase1c_bench.rs` / `benches/poc_bench.rs`；重新生成 `all161_results.csv`。
- **发布**：版本号提升至 `0.1.6`。无新增公开 API、无弃用、无依赖变更（[ADR 0002](docs/adr/0002-release-scope-milestones.md)）。面向用户的行为、调用形态与 `cargo test` / `cargo bench` 工作流均不变。

### 0.1.5

- **指标脚手架（`indicator!` 宏）—— 架构深化候选①（Phase 1a/1b/1c）**：新增 `src/indicator.rs`，以**零成本 `macro_rules! indicator`** 宏统一约 146 个单输出公开函数里重复的「分配等长 `f64::NAN` 缓冲 → 转发到 `*_with_output` 内核」胶水。在**度量前置双闸门**（黄金向量 1:1 + A/B `cargo bench` median |Δ| ≤ ±5%）下分阶段推广：
  - **Phase 1a**：`math_trans` 15 个单输入 / 单输出 / 逐元素函数。
  - **Phase 1b**：`stat` 7 个单输入函数（`stddev`/`var`/`linear_reg`/`linear_reg_angle`/`linear_reg_intercept`/`linear_reg_slope`/`tsf`）经新增的 N 末尾默认臂生成；`beta`/`correl`（多输入）保持手写（阶段二）。
  - **Phase 1c**：`math_ops` 9 个（`add`/`sub`/`mult`/`div`/`sum`/`min`/`max`/`max_index`/`min_index`）+ `volatility` 3 个（`trange`/`atr`/`natr`，其中 2 个带默认臂）+ `price_transform::avgdev`，共 13 个单输出函数改由宏生成；`avgprice`/`medprice`/`typprice`/`wclprice` **刻意回退手写** —— 宏统一的 `vec![f64::NAN; n]` 初始化对它们有回归（隔离微基准：`avgprice` +34.7%、`add` +22.2%；A/B median |Δ| = 16–17% ≫ 5%），而它们既无前导 NaN、也无默认参数、宏对其零收益。
- **零成本保证已验证**：宏在编译期展开为字节级相同的代码（无 `dyn Fn`、无间接调用、无每轮分配）；`*_with_output` 热路径体不变。A/B 结果 —— Phase 1a median 最大 |Δ| = **2.97%**、Phase 1b = **0.11%**、Phase 1c = **0.21%**（均 ≤ 5% → 通过）。黄金向量闸门：全部 **161/161** 函数仍在其容限内复现 TA-Lib 0.7.1；全量 `cargo test` 仍全绿（含宏生成的新 `doctest`）。
- **新增 A/B 基准 harness（方法论）**：新增 `benches/math_trans_bench.rs`、`benches/stat_bench.rs`、`benches/phase1c_bench.rs`（均已在 `Cargo.toml` 注册）—— 零依赖 `Instant` harness，采用**预热 + 交错多轮 + 中位数**抑制单发噪声（单发可达 ±10%）。详见 [`benches/BASELINE.md`](benches/BASELINE.md) 与 [ADR 0011](docs/adr/0011-indicator-scaffold-seam.md)。
- **发布**：版本号提升至 `0.1.5`。无新增公开 API、无弃用、无依赖变更（[ADR 0002](docs/adr/0002-release-scope-milestones.md)）。面向用户的行为、调用形态与 `cargo test` / `cargo bench` 工作流均不变。

### 0.1.4

- **核心模块化（架构深化）**：将单体 `src/core/mod.rs` 拆分为职责单一的模块 —— `ema.rs`（嵌套 EMA 融合）、`extreme.rs`（单调队列滚动极值 / 索引）、`window.rs`（窗口求和 / 方差）、`kernel.rs`（共享内核 helper）。删除冗余的 `check_eq_len` 长度检查 helper（长度检查现紧贴各内核）。纯重构 —— 输出与 TA-Lib 0.7.1 仍逐位 / 黄金向量 1:1，零性能影响。
- **`parallel` 特性升级为一等模块**：原有的重叠播种并行分块（原概念验证）现归入 `src/parallel.rs`，由专属 `tests/parallel_equality.rs` 1:1 相等性测试守护，并由 `benches/parallel_poc.rs` 驱动。5 个 A 类窗口函数（`midpoint`/`minmax`/`minmax_index`/`willr`/`stoch_f`）在默认关闭的 `parallel` 特性下获得多核加速 —— 合计由 **85 更快 / 60 持平 / 16 更慢（几何均值 0.786×）** 变为 **88 / 63 / 10（0.734×）**；对其余 156 个函数该特性为 no-op。
- **性能报告与 161 基准套件刷新**：刷新 [`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md) 与 `all161_results*.csv` 基准数据；并入已定稿的 0.1.3 优化成果（EMA 家族 FMA 收缩补齐 EMA 缺口 —— 见[验证与基准](#验证与基准--verification--benchmarks)）。
- **发布**：版本号提升至 `0.1.4`。无新增公开 API、无弃用、无依赖变更（[ADR 0002](docs/adr/0002-release-scope-milestones.md)）。

### 0.1.3

- **模式识别性能推广**：将 `cdl_hammer` 的内联运行和累加器模板推广到**全部 61 个蜡烛函数**（零偏差 transformer `tools/opt_pattern.py`）；把逐函数的 `CandleAvg::new`+`value`+`advance` 替换为内联 `sum_*`/`trail_*`/`cur_*`/`val_*` 累加器（跳过无 `CandleAvg` 的函数，如 `cdl_engulfing`/`cdl_3outside`/`cdl_hikkake`/`cdl_tristar`）。模式识别几何均值 **Rust/C 由 2.98× → 0.677×**（43 快 / 13 持平 / 5 慢，原为 1/3/57）—— 本次发布的最大单项收益。
- **P2 算法优化（零偏差，0 回退）**：以环形缓冲 `MonoQueue` 替换 `VecDeque` 滚动极值（`min`/`max`/`min_index`/`max_index`，每极值约快 32%）；为 `ht_dcperiod` 增加跳过未用 `compute_dc_phase` 正弦/余弦窗口的循环-IIR 快路径（3.59× → 1.19×，已持平）；在 `compute_dc_phase` 中改用正弦/余弦角度加法递推（`ht_dcphase`/`ht_sine`/`ht_trendmode`）；并将 `mfi` 改写为单遍滑动窗口融合（2.56× → 1.41×）。合计 **82 快 / 54 持平 / 25 慢，几何均值 Rust/C = 0.792×** —— adaq-talib 平均现为 C 的约 1.26× 快（此前为 1.50× 慢）。
- **P3-2b 并行重叠播种（零偏差，0 回退）**：新增默认关闭的 `parallel` 特性，对 5 个可重叠播种的 A 类窗口函数（`midpoint`/`minmax`/`minmax_index`/`willr`/`stoch_f`）采用 `std::thread::scope` + `available_parallelism` 的重叠播种并行分块（纯 `std`，零外部依赖）；每块以 `period-1`（或 `stoch_f` 的 `fk+fd-2`）个前导元素重叠，复用与串行逐字节一致的核，输出 1:1。合计由 **85 快 / 60 持平 / 16 慢（0.786×）** 变为 **88 快 / 63 持平 / 10 慢（0.734×，约 1.36× 快于 C）**；对其余 156 个函数该特性为 no-op。详见 [`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md) §3.5。
- **报告与工具**：更新 [`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md)（新分组 / 逐指标表，三次取中位方法学）与交互式 `docs/benchmarks/adaq-vs-talib-161.html`；新增 `benches/extreme_ab.rs`、`tools/opt_pattern.py` 与 `docs/research/perf-161-analysis.md`。
- **发布**：版本号提升至 `0.1.3`。无新增公开 API、无弃用、无依赖变更（[ADR 0002](docs/adr/0002-release-scope-milestones.md)）。

### 0.1.2

- **全量 161 基准与验证套件**：新增 `benches/all161_bench.rs`（由 `tools/bench/gen_all161.py` 自动生成），对**全部 161** 个指标与原生 TA-Lib C 0.7.1 逐项基准对照，并附带实时数值一致性校验和；配套 `benches/poc_bench.rs` 为概念验证脚手架。统一报告 [`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md)、交互式 `docs/benchmarks/adaq-vs-talib-161.html` 与 `all161_results.csv` 由 `tools/bench/gen_report.py` 生成（双轨方法论见 [ADR 0004](docs/adr/0004-benchmark-dual-track.md)）。
- **黄金向量覆盖扩大**：**222 个黄金向量 fixture 文件**（原 159 个）——补全了完整的模式识别 fixture 集与 `macd_ext` / `macd_fix` fixture。全量测试现为 **326 项测试，0 失败**（原 308），`tools/reconcile.py` 确认 **161/161**。
- **文档完整性**：逐函数表现已列出全部 161 个函数。`accbands`（重叠研究）、`dx` / `imi`（动量）与 `avgdev`（价格变换）此前已实现并计入 161 总数，但被遗漏在明细表之外 —— 现均已补入文档。
- **发布**：版本号提升至 `0.1.2`。除上述外无新增公开 API；无弃用、无依赖变更（[ADR 0002](docs/adr/0002-release-scope-milestones.md)）。

### 0.1.1

- **数学算子 —— O(n) 极值索引函数**：`max_index` / `min_index` / `minmax_index` 现采用单遍单调队列（`core::rolling_extreme_index`），替换原先 O(n·period) 的嵌套扫描 —— 提速约 1.9×，且与 TA-Lib 0.7.1 仍逐项 1:1（见 [ADR 0005](docs/adr/0005-error-tolerance.md)）。新增 `benches/index_bench.rs` 与 `benches/minmax_bench.rs`。
- **`minmax` 收敛**：`math_ops::minmax` 现复用单遍 `core::rolling_minmax` 核（与 `midpoint` 同源），消除重复的极值逻辑。性能中性，精度不变。
- **P2 全阶段性能优化（1:1 验证）**：`dema` / `tema` / `t3` 嵌套 EMA 融合（P2-1）；`midpoint` / `midprice` 单调队列（P2-2）；`wma` O(n) 滑动递推（P2-3）；`bbands` 中轨单遍融合（P2-4）；`linear_reg` 家族 / `correl` / `willr` / `stoch` 滑动 O(n)（P2-5）。详见 [`benches/BASELINE.md`](benches/BASELINE.md)。
- **发布工具与文档**：新增 `.github/workflows/release.yml`（发布自动化）与 CI；修复 doc-comment 与发布 `exclude`；版本号提升至 `0.1.1`。
- **模式识别与数学运算模块**：全部 61 个蜡烛形态与完整的 `math_ops` / `math_trans` 函数面均已实现，并补齐黄金向量 fixture（P4 里程碑 —— 161/161 函数）。

### 0.1.0

- 首个公开里程碑：完整的 TA-Lib 0.7.1 公开函数面 —— 10 大类共 161 个函数，并以零偏差黄金向量验证。
