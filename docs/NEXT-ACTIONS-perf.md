# 下一步行动项清单 · 高性能 Rust 版 TA-Lib

> 状态：已与用户对齐（grilling /grill-with-docs，2026-08-10，全按建议确认）
> 关联：ADR 0010（性能策略）、ADR 0002（里程碑）、ADR 0003（黄金向量）、ADR 0004（双轨基准）、ADR 0005（误差容限）、ADR 0009（candle settings）
> 硬约束（贯穿全程，不可破）：**性能第一 / Zero-FFI（`[dependencies]` 为空）/ No-Deps（不引外部 crate，算法原生手写）/ 零偏差（ADR 0005）**

---

## 0. 总览与排序原则

性能重构铁律（ADR 0010）：**先正确性基线 → 再测量 → 后优化**。任何优化改动必须以 P0 权威黄金向量 + P1 ns/elem 基线为护栏，零偏差承诺方可证伪。

```
P0 正确性基线（阻塞） → P1 测量底座 → P2 算法+编译器优化 → P3 显式SIMD(门控评估) → P4 后续里程碑
```

六决策落地映射：Q1→P0-2/3、Q2→P0-1、Q3→P0-4、Q4→P2、Q5→P1、Q6→P3。

## 战略决策（用户确认，2026-08-10，执行中 / recorded & in progress）

**结论：混合路线——先建共享优化原语（于现有 65 上验证 + 测量），再在其上实现剩余 96。**
不采用"先全实现 96 再优化"（会在原语层二次返工、拖延性能证明），也不采用"先对 65 做逐函数微优化"（优化错层，原语就绪后仍需返工）。

- **理由**：ADR 0010 的优化本质是**共享基础设施**——EMA 状态机、单调队列、`rolling` 前缀和、`_with_output` 原地写入 API、少量热核 SIMD。这些原语被 65 与 96 共同复用；现有 65 已覆盖最难的原语类型（嵌套 EMA 链、滚动极值、WMA、BBANDS），恰适合先在其上验证与测量（P1 护栏）。
- **剩余 96 的构成**：数学算子(11) + 数学变换(15) + 模式识别(61) = 87 基本为元素级 / 整数比较运算，手写即快、几乎不依赖重原语；真正吃原语的是重叠/动量/周期的少量补缺（本版类别内 4 + Cycle 5 = 9）。
- **执行顺序**：P0（✅ 已完成）→ **P1 测量底座（✅ 已完成，见 `benches/BASELINE.md`）** → **P2 共享原语优化**（在 65 上验证，黄金向量 + ns/elem 护栏）→ **P4 在高速原语上实现剩余 96**（P4-0 先补本版类别内 4 个，再 Cycle / 数学 / 模式）。
- 执行启动于 2026-08-10（用户指令 "ok 开始"）。

---

## P0 — 正确性基线（性能重构的先决条件，阻塞性）

### P0-1 安装 TA-Lib C 工具链（本机）【已授权 Q2】
- `brew install ta-lib`；随后 `python3 -m pip install TA-Lib`（PyPI 包名 `TA-Lib`，导入名 `talib`，须先装 C 库）。
- 验证：`python3 -c "import talib; print(talib.__version__)"` 输出 `0.7.1`。
- 复核 `tools/README.md` 登记的 C 库版本（已写 0.7.1，确认即可）。
- **约束自检**：该工具链仅用于 `tools/` 开发侧生成 fixture，**不进 `[dependencies]`**，Zero-FFI / No-Deps 不受影响。

### P0-2 全量重生成权威黄金向量【Q1】
- 运行 `python tools/gen_fixtures/generate.py` 重建全部 63 个 fixture。
- 删除各 fixture 内残留的 `_note` / `REFERENCE` 字段，确保 fixture 为**真实 TA-Lib C 0.7.1 输出**（非文档算法参考值）。
- 全量 `cargo test` 必须 1:1 通过（ADR 0005 容限 `1e-8` + `1e-10`；STOCH/MACD 内部 EMA/线性回归等敏感指标按 ADR 0005 逐指标放宽至 `1e-6`）。
- **验收**：63 个 fixture 均为权威向量；`cargo test` 全绿。

### P0-3 统一文档口径（消除当前自相矛盾）【已完成 · 2026-08-10】
- 现状矛盾：`tools/README.md` 称 63 个 fixture 已是权威真实 C 输出；`ADR 0010` D3 与 `tests/overlap_test.rs`/`overlap_new_test.rs` 注释称 overlap fixture 仍是"文档算法参考值"。两套说法互斥，且 `overlap_test.rs` 引用的 `_note` 字段在 fixture 中已不可见。
- 修订 `ADR 0010` D3 措辞（"当前非权威"→ 注明已重生成权威 + 时间线）。
- 修订 `tests/overlap_test.rs` / `tests/overlap_new_test.rs` 注释，去掉"对照文档算法参考值"说法。
- 使 `tools/README.md` / `ADR 0010` / 测试注释三处一致，均为同一真相。

### P0-4 函数计数对账闭环【已完成 · 2026-08-10】
- 写脚本（如 `tools/count_pub_fns.py` 或 grep）自动枚举 7 个模块（`overlap`/`momentum`/`volatility`/`volume`/`price_transform`/`stat`/`cycle`）的 `pub fn`，剔除 `_default` 便捷变体与 `core`/`utils` 内部原语，断言对外函数 = **65**（动量 29）。
- 把 `ADR 0010` 中仍挂起的 P0-A0.2 待办划掉；在 `0.1.0-scope.md` 对账记录追加 2026-08-10 条目。
- **验收**：自动计数 = 65；文档一致。

---

## P1 — 测量底座（无测量不优化）【Q5】

### P1-1 建立热路径 bench 矩阵【已完成 · 2026-08-10】
- 为待优化热路径新增：`benches/dema_bench.rs` `tema_bench.rs` `t3_bench.rs` `midprice_bench.rs` `wma_bench.rs` `bbands_bench.rs`。
- 沿用 `benches/sma_bench.rs` 的 LCG 输入 + checksum 防优化写法；记录 **ns/elem** 基线快照（落 `benches/BASELINE.md` 或 CI 轻量门禁）。
- **验收**：每个待优化指标有可复现 ns/elem 基线。

### P1-2 落地双轨基准（ADR 0004）【已完成 · 2026-08-10】
- `bench-c` feature 下补齐 FFI 对照（当前仅 SMA），覆盖上述热路径。
- `tools/bench/` 写 Python 便捷对照脚本，明确标注口径为 **"vs TA-Lib Python binding"**。
- **验收**：每个待优化指标至少有一个 FFI 对照点 + 一个 Python 对照报告。

---

## P2 — 算法 + 编译器优化（核心性能收益）【Q4，ADR 0010 D1/D2】

### P2-1 嵌套 EMA 融合核（DEMA / TEMA / T3）【已完成 · 2026-08-10】
- 现状：`overlap.rs` 中 `dema` 做 3 次独立 `ema()` 全量扫描各分配 `Vec`；`tema` 4 次；`t3` 5 次（行 232-286、582-586）。
- 新增 `core` 状态式 EMA 原语，改为**单遍**产出 E1/E2/E3（DEMA）、E1–E4（TEMA）、E1–E5（T3），复用 EMA 状态，消除 3–5 次 `Vec` 分配与独立扫描。
- 新增 `dema_with_output` / `tema_with_output` / `t3_with_output(values, …, out: &mut [f64])`；原 `Result<Vec<f64>>` 公开 API 委托之（D2，不改 ADR 0001 形态、不破 SemVer）。
- **验收（已通过）**：黄金向量 1:1 通过（`cargo test` 全绿，dema/tema/t3 黄金向量 + 单元测试 + doctest）；bench ns/elem 显著下降：
  - DEMA 7.40 → **3.61** ns/elem（0.49×，Rust/C = 0.73×，**快于 C**）
  - TEMA 10.84 → **3.48** ns/elem（0.32×，Rust/C = 0.46×，**快于 C**）
  - T3 22.28 → **3.79** ns/elem（0.17×，Rust/C = 1.40×，原 7.85×）
  - 实现：`src/core/mod.rs` 的 `nested_ema_with_output<const L, F>`（const-generic 级联 + `combine` 闭包，bit-for-bit 等同逐次 `ema`）；`src/overlap.rs` 三个 `_with_output` 委托。
  - T3 仍慢于 C（1.40×）→ 满足 P3 闸门条件之一（>20%），留待 P3 评估（需再满足"自动向量化失败"）。

### P2-2 单调队列替换 rolling_extreme（MIDPOINT / MIDPRICE）【Q4】
- `core/mod.rs` 的 `rolling_extreme` 现为 O(n·period) 朴素扫描（自注 "correctness first"）。
- 新增 O(n) 单调队列实现 `rolling_max`/`rolling_min`，保留朴素版作 fallback / 校验。
- 新增 `midpoint_with_output` / `midprice_with_output`。
- **验收**：1:1 通过；大 `period` 下 ns/elem 明显改善。

### P2-3 WMA 前缀和点积化【Q4】
- 现状：`wma` 内循环每 `i` 重算 `period` 次（`core/mod.rs` 行 127-143）。
- 用滑动前缀和或权重点积累积消除每 `i` 的 `period` 次重算；新增 `wma_with_output`。
- **验收**：1:1 通过；ns/elem 改善。

### P2-4 BBANDS 融合核【Q4】
- 合并 middle(SMA) + 总体标准差为单遍（复用 `rolling_mean` + 滚动二阶矩，参考 `rolling_var`）；新增 `bbands_with_output`（写入结构体 `Bbands { upper, middle, lower }`）。
- **验收**：1:1 通过；ns/elem 改善。

### P2-5 编译器自动向量化调优（D1）
- 关键核加 `#[inline]` / `#[inline(always)]`；消除热循环 bounds-check（已验证边界处用迭代器重写或谨慎 `get_unchecked`）。
- 实验 `#![optimize(speed)]`（仅 bench 配置）；`-C target-cpu=native` **仅用于 bench 对比，不进发布产物**。
- **验收**：`cargo-asm` 确认热循环自动向量化；发布构建无 `unsafe` / 不依赖 `nightly`（MSRV 1.85 不变）。

---

## P3 — 显式 SIMD 评估（门控，不默认实现）【Q6】

### P3-1 评估闸门（已固化，可执行判据）
- 触发条件：经 P2 后，某 0.1.0 热循环满足 **(a)** 用 `cargo-asm` / `llvm-mca` 确认编译器自动向量化**失败**，**且 (b)** 其 `ns/elem` 比 `bench-c` 等价 FFI 实现慢 **>20%**。
- 若触发：新增 `simd` feature（默认**关闭**），用 `core::arch` intrinsics 实现该核并门控编译；默认构建保持安全稳定。
- **验收**：仅当闸门满足才实现 SIMD；否则 P3 标记"未触发 / 跳过"并记录理由。

---

## P4 — 后续里程碑（不删减，ADR 0002）

每项同样遵循 P0→P1→P2 流程（先权威向量 → 测量 → 优化）：
- **P4-0** 本版类别内补缺（4）：`ACCBANDS`（Overlap）、`DX` / `IMI`（Momentum）、`AVGDEV`（Price Transform）——优先补齐，保持已覆盖类别完整（详见 `0.1.0-scope.md`）。
- **P4-1** 数学算子（11）：ADD / DIV / MAX / MIN / MULT / SUB / SUM / MAXINDEX / MININDEX / MINMAX / MINMAXINDEX
- **P4-2** 数学变换（15）：ACOS / ASIN / ATAN / CEIL / COS / COSH / EXP / FLOOR / LN / LOG10 / SIN / SINH / SQRT / TAN / TANH
- **P4-3** 周期指标（5）：HT_DCPERIOD / HT_DCPHASE / HT_PHASOR / HT_SINE / HT_TRENDMODE
- **P4-4** 模式识别（~61，仅默认 candle settings，ADR 0009）

---

## 硬约束复查清单（每次 PR 必查）
- [ ] `[dependencies]` 为空（Zero-FFI / No-Deps）
- [ ] 未链接任何 C TA-Lib ABI；仅 `tools/` 开发侧用 TA-Lib C 生成 fixture
- [ ] 全部优化通过权威黄金向量 1:1（ADR 0005 容限）
- [ ] 新增 `_with_output` 为公开 SemVer 契约；原 `Result<Vec>` API 形态不变（ADR 0001）
- [ ] 发布构建无 `unsafe` / 不依赖 `nightly`；`-C target-cpu=native` 仅 bench 用

---

## 当前已确认事实（审计基线，2026-08-10 更新）
- **全量对标目标 = TA-Lib 0.7.1 共 161 个对外函数**（本机 `/opt/homebrew/bin/python3` 的 `talib` 0.7.1 实测）；0.1.0 已实现 **65**，剩余 **96**。
- 0.1.0 实现分布（按 TA-Lib 权威原生分组 `info['group']，2026-08-10 实测对账）：重叠 17/18（缺 `ACCBANDS`）、动量 29/31（缺 `DX`/`IMI`）、波动率 3/3、成交量 3/3、价格变换 4/5（缺 `AVGDEV`）、统计 **9/9（全量完成）**、周期 **0/5**；数学算子/变换/模式识别均 0。合计 **65 / 161**，剩余 **96**。
- 热点已核实：DEMA/TEMA/T3 嵌套 EMA 多分配；`rolling_extreme` O(n·period)；WMA 内循环重算；无 `_with_output`、无 SIMD/`unsafe`/`target-cpu`；bench 仅 `sma_bench.rs`。
- 本机**已安装** TA-Lib C + PyPI `TA-Lib` 0.7.1（P0-1 完成；`generate.py` 已成功跑通、63 fixture 重生成、权威黄金向量护栏就绪）→ P0-2 已完成，P0-3/P0-4 已完成（测试注释 + ADR 0010 D3 已对齐；`tools/reconcile.py` 自动对账 65/161/96 通过）。
- **P1 测量底座已完成**（2026-08-10）：新增 6 个热路径 bench（`dema/tema/t3/wma/midprice/bbands`）+ 沿用 `sma_bench`；`bench-c` feature 下 FFI 链接原生 TA-Lib C 0.7.1 双轨对照（`build.rs` 自动探测 `ta-lib`/`ta_lib`）；`tools/bench/compare.py` 提供 Python 绑定对照。基线快照写入 `benches/BASELINE.md`。
- **基线结论（Rust vs 原生 C，ns/elem）**：SMA 0.60×、BBANDS 1.02× 已持平/更优；P2-1 后 DEMA 0.73×、TEMA 0.46× 已快于 C，T3 1.40×（原 7.85×）；**剩余差距主要在** MIDPOINT 7.09×、WMA 3.91×、MIDPRICE 1.84×。→ **P2 下一步优先级**：MIDPOINT > WMA > MIDPRICE（对应 P2-2 单调队列 / P2-3 WMA 前缀和）；T3 留待 P3 SIMD 评估（已满足 >20% 闸门条件之一）。
- 硬约束自检：`build.rs` 仅在 `bench-c` 下链接 C 库，且仅作用于 bench 二进制；默认 `cargo build`/`cargo test` 仍零 C 依赖（Zero-FFI 不变）。
