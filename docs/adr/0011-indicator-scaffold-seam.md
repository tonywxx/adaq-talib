# ADR 0011: 指标脚手架接缝归一化（零成本 `indicator!` 宏）

- 状态：已采纳（2026-08-11）
- 决策人：用户 + WorkBuddy（架构评审 + grilling 决策会话：`/improve-codebase-architecture` 中文报告 → 候选① → 两轮 grilling，用户采纳全部推荐）
- 关联：ADR 0001（API 模型 B）、ADR 0003（黄金向量）、ADR 0005（误差容限）、ADR 0010（性能策略，D2 原地写入）

## 背景

架构评审（2026-08-11 中文报告）量化了重复脚手架：约 **211 处** 重复胶水（64 个 `_default` + 144 个
`_with_output`）横跨约 **389 个公开函数**；其中单输出（`Vec<f64>`）≈146、多输出（结构体）≈30。

其中最同质的模式是：每个公开 `func` 都手写「分配等长 `f64::NAN` 缓冲 → 调用 `*_with_output` → 包成
`Result<Vec<f64>, TaError>`」。这正是 ADR 0010 D2 引入 `_with_output` 后必然伴随的胶水层——正确但
高度重复、易在复制粘贴中漂移（参数顺序、错误映射、NaN 填充语义）。

候选①（评审 Top 推荐）：用**零成本 `indicator!` 宏** 把该接缝归一化。硬约束（来自项目约定与记忆）：

- Zero-FFI / No-Dependencies（`[dependencies]` 为空）；
- 零偏差（zero-deviation）：数值与 TA-Lib 0.7.1 逐项 1:1；
- **禁用 `dyn Fn`**：优先 `impl Fn` 单态化内联，避免间接调用与每轮分配退化性能；
- **度量前置协议**：任何「消除重复/统一抽象」类重构合并前，必须做 A/B 基准 + 黄金向量 1:1；
- 不写投机接口（不预先添加尚未被使用的 arm / 参数）。

两轮 grilling（Q1–Q9，用户「全部按你推荐」「ok」）锚定了落地路径：**Phase 1a 先试点 `math_trans`
15 个函数**（全库唯一「纯单输入 / 单输出 / 逐元素」模块，零风险），验证双闸门后再推广。

## 决策

### D1 机制：编译期 `macro_rules! indicator`，`pub(crate)` 非导出

- 宏定义于 `src/indicator.rs`，`pub(crate) use indicator;` 对库内可见，**不** `#[macro_export]`
  （避免泄漏为公开宏、规避 SemVer 表面面与跨 crate 宏路径负担）。
- 两条 arm：
  - **无默认 arm（Phase 1a）**：展开为
    `pub fn $fname($len: &[f64] $(, $arg: $argty)*) -> Result<Vec<f64>, TaError> {`
    `let mut out = vec![f64::NAN; $len.len()]; $with_output($len $(, $arg)*, &mut out)?; Ok(out) }`
  - **默认 arm（Phase 1b）**：在无默认 arm 基础上额外生成 `$dname` 转发到
    `$fname(..., $def)`。
- 宏顶部 doc-comment 固化接缝约定与路线图（Phase 1a math_trans → 1b stat → 1c 其余单输出 →
  Phase 2 多输入 + 结构体 arm）。

### D2 零成本保证：宏为文本展开，数值与性能与手写 1:1

- 宏展开体就是手写为完全一致的字面体（无 `dyn Fn`、无间接调用、无每轮分配），二者生成机器码
  逐字节相同。
- 热路径 `*_with_output` 体**保持手写、字节级不变**；宏只替换其外层胶水。
- 因此重构不引入任何数值漂移或性能开销——`indicator!` 是「消除重复」而非「改变算法」。

### D3 接缝约定（写宏调用须遵守）

- `func` 首个参数必须是主序列 `&[f64]`；
- 对应 `_with_output` 签名为 `(主序列, …其他参数, out: &mut [f64]) -> Result<(), TaError>`；
- 若带默认值，默认参数必须置于**最后**一位（默认 arm 才能正确转发）；
- 错误以 `Result<Vec<f64>, TaError>` 返回，NaN 填充前导不稳定期。

### D4 双闸门（度量前置协议落地）

任何模块接入 `indicator!` 前/后必须同时满足：

1. **黄金向量闸门**：`cargo test`（含 `tests/math_trans_test.rs` 等）1:1 通过（ADR 0003 / 0005）；
2. **A/B 性能闸门**：`benches/math_trans_bench.rs`（零依赖 `Instant` harness）测量重构前后
   ns/elem，断言 **median |Δ| ≤ ±5%**。

> 经验修正（Phase 1a 实测）：单发 `Instant` 受 CPU 电源态 / 缓存预热 / 测量顺序影响，单次 Δ 可
> ±10%（曾出现宏反而快 10% 的纯噪声）。故 A/B 须 **预热 + 交错多轮 + 取中位数** 抑制噪声，
> 以 median |Δ| 判定，而非单次 Δ（见 `benches/math_trans_bench.rs`）。

### D5 分阶段推广范围（门控于双闸门，逐模块）

- **Phase 1a（已完成）**：`math_trans` 15 个单输入/单输出/逐元素函数。
- **Phase 1b（已完成）**：`stat` 7 个单输入函数（`stddev` / `var` / `linear_reg` / `linear_reg_angle` /
  `linear_reg_intercept` / `linear_reg_slope` / `tsf`）启用默认臂；宏默认臂由单默认扩展为 **N 末尾默认**
  （以覆盖 `stddev`/`var` 的 2 末尾默认）；`beta` / `correl` 为多输入（2×`&[f64]`），留待 Phase 2。
- **Phase 1c（已完成）**：`math_ops` 9 个单输出函数（`add` / `sub` / `mult` / `div` / `max` / `min` / `sum` /
  `max_index` / `min_index`）+ `volatility` 3 个（`trange` / `atr` / `natr`，其中 `atr`/`natr` 带默认臂）+
  `price_transform` 中须前导 `NaN` 的 `avgdev`；`*_with_output` 体字节级不变。
  `price_transform` 的 `avgprice` / `medprice` / `typprice` / `wclprice` **未纳入宏、已回退手写**
  （见下方「Phase 1c 关键发现」）。
- **Phase 2（待做，暂未实现）**：多输入 + 结构体多输出 arm（`momentum` 的 MACD、`math_ops` 的
  `minmax` / `minmax_index` 等）；当前宏仅覆盖单输出 `Vec<f64>` 形态，多输出须新增 arm，
  **仅在确有模块需要时才扩展**（不写投机接口）。

> **Phase 1c 关键发现（measure-first 协议抓出的真实回归，非噪声）**：宏统一用 `vec![f64::NAN; n]`
> 初始化输出缓冲；而 `avgprice` / `medprice` / `typprice` / `wclprice` 这 4 个**逐元素全覆写、无前导
> `NaN`** 的函数，其手写原版用 `vec![0.0_f64; n]`。实测 NAN 初始化比零初始化慢（非全零位模式无法走
> 零页优化；隔离微基准 `avgprice` NAN vs ZERO = **+34.7%**、`add` = **+22.2%**）。A/B 闸门因此报
> `avgprice` median |Δ| = **16–17%**（远超 ±5%）。这 4 个函数**既无前导 NaN 需求、也无默认参数、
> 宏对其零收益**，仅因统一 NAN 初始化而变慢 —— 故**刻意回退手写**（`0.0_f64` 原生初始化），不纳入宏。
> 其余宏生成函数（含 `trange`/`atr` 等同样全覆写者）因计算/拷贝主导、或本身即需 NAN 初始化，
> median |Δ| 均在 ±0.3% 内，闸门通过。此即「度量前置」协议的价值：未度量会误以为宏零代价。

### D6 不写投机接口

宏的 arm / 参数只增不减地随真实需求扩展；Phase 2 的多输入/结构体 arm 不在 Phase 1 预置。

## 验证（Phase 1a 试点结果）

- **黄金向量闸门**：`cargo test --test math_trans_test` **15/15 通过**（全部 161 函数级 `cargo test`
  绿灯）；宏生成 `func` 与手写数值逐项 1:1。
- **A/B 性能闸门**：`cargo bench --bench math_trans_bench` median 最大 |Δ| = **2.97% ≤ 5%** → PASS
  （预热 + 交错 11 轮 + 中位数；基线记录 15 个函数 ns/call 供后续回归）。
- `benches/math_trans_bench.rs` 已在 `Cargo.toml` 注册 `[[bench]] name = "math_trans_bench" harness = false`。

## 验证（Phase 1b 结果）

- **默认臂扩展**：`indicator!` 默认臂由「单默认」升级为「N 末尾默认」（`=> ( $($def:expr),+ )`），
  以覆盖 `stddev`/`var`（末尾 2 默认）。`stat` 7 个单输入函数改由宏生成，`*_with_output` 体字节级不变；
  `beta`/`correl`（多输入）保持手写（Phase 2）。
- **黄金向量闸门**：`cargo test --test stat_test` **9/9 通过**；全量 `cargo test`（含内联 `mod tests` 与
  `stddev` 宏内 doctest）绿灯——宏生成 `func` 与手写数值逐项 1:1。
- **A/B 性能闸门**：`cargo bench --bench stat_bench` median 最大 |Δ| = **0.11% ≤ 5%** → PASS
  （stddev/var 为 2 默认、linear_reg/tsf 为 1 默认；预热 + 交错 11 轮 + 中位数）。
- 副作用修复：`linear_reg` 等改走 `*_with_output` 后，`linreg_core`（分配式包装）仅被测试引用，
  标 `#[cfg(test)]` 消除 `cargo build` dead_code 警告（保持零警告构建）。
- `benches/stat_bench.rs` 已在 `Cargo.toml` 注册 `[[bench]] name = "stat_bench" harness = false`。

## 验证（Phase 1c 结果）

- **范围**：`math_ops` 9 个、`volatility` 3 个（`atr`/`natr` 带默认臂）、`price_transform` 的 `avgdev`
  共 13 个单输出函数改由 `indicator!` 生成；`avgprice` / `medprice` / `typprice` / `wclprice` 因
  NAN 初始化回归**回退手写**（见 D5 关键发现）。
- **黄金向量闸门**：`cargo test --test price_transform_test / math_ops_test / volatility_test`
  **11 / 5 / 3 通过**（合计 19/19）；全量 `cargo test`（含 `trange` / `atr` / `natr` / `avgdev` 宏内
  doctest）绿灯——宏生成 `func` 与手写数值逐项 1:1。
- **A/B 性能闸门**：`cargo bench --bench phase1c_bench` median 最大 |Δ| = **0.21% ≤ 5%** → PASS
  （add / trange / max / min_index / avgdev / atr；预热 + 交错 11 轮 + 中位数 + 宏与基线双路径预热）。
- 注：`avgprice` 等 4 函数已从 A/B 移除（已回退手写，非宏对照）；其回归由隔离微基准单独确认。
- `benches/phase1c_bench.rs` 已在 `Cargo.toml` 注册 `[[bench]] name = "phase1c_bench" harness = false`。
- 构建纪律：`price_transform` / `math_ops` / `volatility` 三模块 `cargo build` 零新增警告
  （既有 `momentum.rs:18` 的 `rolling_sum` 死导入、`pattern/batch_*.rs` 的 `val_avg_shadow_long_1`
  未使用变量为本次变更前已存在，不在 Phase 1c 范围内，未改动）。

## 权衡

- 优点：消除约 146 处单输出脚手架（Phase 1 全量后），显著降低复制粘贴漂移面与维护成本；
  零数值 / 零性能代价（D2）；`pub(crate)` 不污染 SemVer 表面。
- 缺点：源码中 `func` 来源不如手写直白，宏展开调试略增心智负担；新贡献者需先读宏 doc。
  缓解：宏顶部 doc-comment + `CONTEXT.md` 术语 + 本 ADR。
- 难以回退性：宏为 `pub(crate)` 内部实现，`_with_output` 公开 API 形态不变（ADR 0010 D2），
  回退成本低、无 SemVer 断裂。

## 影响

- 新增 `src/indicator.rs`；`src/lib.rs` 接入 `pub(crate) mod indicator;`。
- `src/math_trans.rs` 15 个 `func` 改由 `indicator!` 生成，`*_with_output` 体字节级不变。
- `CONTEXT.md` 增补术语 **指标脚手架 (indicator scaffold)**。
- `benches/math_trans_bench.rs` + `Cargo.toml` `[[bench]]` 条目。
- 后续 Phase 1b/1c/2 **复用本已固定的模式与双闸门**，不再重复「grilling + 试点」流程；
  每个模块接入时仅跑双闸门即可，无需再次评审候选①本身。
