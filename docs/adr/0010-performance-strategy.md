# ADR 0010: 性能优化策略（性能为第一优先级）

- 状态：已采纳（2026-08-09）
- 决策人：用户 + WorkBuddy（grilling 会话，/grill-with-docs）
- 关联：ADR 0001（API 模型 B）、ADR 0002（里程碑）、ADR 0003（黄金向量）、ADR 0004（基准双轨）、ADR 0005（误差容限）

## 背景

用户要求"性能为第一优先级"，在内存布局、循环与分支、数组运算层面做极致优化；同时硬约束为
Zero-FFI、No-Dependencies（`[dependencies]` 为空）、`f64`、API 模型 B、零偏差（zero-deviation）。
审计 `src/` 与 `docs/` 后发现：

1. 既有的 9 个 ADR 没有任何一个把"如何优化"固化为决策；仅 ADR 0004 规定了基准方法，
   且 benchmark 底座几乎空白（仅 `benches/sma_bench.rs`）。
2. 当前实现为"正确性优先"：每调用分配 `Vec<f64>`；DEMA/TEMA 各做 3 次独立全量 EMA
   扫描并各分配 `Vec`；`rolling_extreme` 为 O(n·period) 朴素窗口（`src/core/mod.rs` 自注
   "correctness first"）；WMA 内循环每 `i` 重算 `period` 次；无 SIMD、无 `unsafe`、仅少量 `#[inline]`。
3. 黄金向量已于 **2026-08-10** 由 `tools/gen_fixtures/generate.py` 基于真实 TA-Lib C 0.7.1
   输出全量重生成（共 63 个 fixture），是**权威黄金向量**，不再携带 `_note` 字段
   （见 `tools/README.md` / ADR 0003）。性能重构的数值漂移风险由全量黄金向量比对兜底。

## 决策

性能优化以"**先正确性基线、再测量、后优化**"为铁律，手段边界如下。

### D1 优化层级：稳定版 + 安全代码 + 自动向量化 + 算法优化（不引入 unsafe / 不依赖 nightly）

- 保持 MSRV 1.85、Zero-FFI、No-Deps 不变。
- 性能提升主要来自：
  - **算法改进**：单调队列、融合单遍核、减少分配；
  - **内存布局**：`f64` 数组对齐（32/64 字节）；
  - **编译器自动向量化**：`#[inline]` / `#[inline(always)]` 关键核、消除热循环内
    bounds-check、必要时 `#![optimize(speed)]` 实验；
  - `-C target-cpu=native` **仅用于 bench 对比**，不进入发布产物。
- 显式 SIMD（`core::arch` intrinsics / nightly `std::simd`）**不在 0.1.0 性能计划内**；
  仅当 P1 测量证明某热点自动向量化无法覆盖时，作为后续升级项评估，并门控于 `simd`
  feature，默认构建保持安全稳定（见 P3）。

### D2 分配与 API 形态：新增原地写入变体，公开分配式 API 不变

- 为每个热路径指标新增
  `fn <name>_with_output(values, …, out: &mut [f64]) -> Result<(), TaError>`；
  原 `Result<Vec<f64>, TaError>` 公开 API 作为糖，内部委托给 `_with_output`。
- 多输出指标（BBANDS 等）同样提供写入式结构体变体。
- **不采用**"以调用方缓冲区为主 API"——避免改变 ADR 0001 既定形态、避免 SemVer 断裂。

### D3 正确性前置：性能重构前先生成权威黄金向量【已完成 · 2026-08-10】

- **状态**：本机已安装 TA-Lib C 0.7.1 + PyPI `TA-Lib`；`tools/gen_fixtures/generate.py`
  已运行，全量重生成 63 个 fixture 为**真实 0.7.1 输出**的权威黄金向量（不再含 `_note`）；
  C 库版本登记于 `tools/README.md`（0.7.1）。全量 `cargo test` 1:1 通过（ADR 0005 容限）。
- 任何性能改动须通过全量黄金向量比对（ADR 0003 / ADR 0005）；数值敏感指标按 ADR 0005
  逐指标放宽容限（如 `1e-6`）以吸收合法的浮点重排（重排顺序 ≠ 偏差）。

### D4 测量底座：无测量不优化

- 建立跨类别 benchmark 矩阵（`benches/*_bench.rs`，沿用 LCG 输入 + checksum 防优化）。
- 兑现 ADR 0004 双轨：`bench-c` feature 下补齐 FFI 对照（不止 SMA）；`tools/bench/` 写
  Python 便捷对照脚本并明确标注"vs TA-Lib Python binding"口径。
- 记录 ns/elem 基线作为回归护栏（本地基线快照或 CI 轻量门禁）。

## 权衡

- 优点：在不变更硬约束、不破坏 API / SemVer 的前提下拿到绝大部分性能收益；优化有
  真·Oracle 与测量底座兜底，零偏差承诺**可证伪**。
- 缺点：放弃显式 SIMD 的峰值收益（可由后续升级项补回）；新增 `_with_output` 公开面。
- 难以回退：性能策略是长期工程取向；公开 `_with_output` API 属 SemVer 契约。

## 影响

- 新建本 ADR；`CONTEXT.md` 增补性能术语（单调队列 / 融合单遍核 / 原地写入 / 自动向量化 / 对齐）。
- 下一步行动项（见本会话产出）：P0 正确性基线 → P1 测量底座 → P2 算法 + 编译器优化
  → P3（评估项）SIMD → P4 后续里程碑（数学类 + 模式识别）。
- `0.1.0-scope.md` 的函数计数出入（历史宣称 66 / 动量 30 vs 实际 63 / 动量 27）已于 2026-08-10
  对账闭合：实测 **65 / 161**（对外函数 65，TA-Lib 0.7.1 共 161），详见 P0-A0.2 与 `0.1.0-scope.md` 对账日志。
