# 性能优化收尾计划 · Final Performance-Optimization Plan (adaq-talib)

> 范围：完成全部 161 个 TA-Lib 0.7.1 对外函数的性能优化收尾、定义正确性+性能验证方法、给出 README 报告大纲。
> 关联：ADR 0010（性能策略）、ADR 0004（双轨基准）、ADR 0005（误差容限）、ADR 0003（黄金向量）、`NEXT-ACTIONS-perf.md`、`benches/BASELINE.md`。
> 硬约束（贯穿全程）：**Zero-FFI（`[dependencies]` 空）/ No-Deps（算法手写）/ 零偏差（ADR 0005）/ 无 `unsafe`·无 `nightly` 发布构建 / `-C target-cpu=native` 仅 bench。**
> 任何改动**不得**使当前全绿的 `cargo test`（161 函数 1:1 黄金向量）退化。

---

## 0. 现状基线（已确认，2026-08-10）

- P2-1 ✅ `nested_ema_with_output` 单遍嵌套 EMA（DEMA 0.73× / TEMA 0.46× / T3 1.40×）。
- P2-2 ✅ 单调队列 `rolling_extreme` + 合并 `rolling_minmax`（MIDPRICE 0.56× / MIDPOINT 2.08×）。
- P2-3 ✅ WMA 滑动递推（0.94×）。
- 当前 Rust/C（ns/elem）：SMA 0.60×、BBANDS 1.02×、TEMA 0.46×、DEMA 0.73×、T3 1.40×、MIDPRICE 0.56×、WMA 0.94×、MIDPOINT 2.08×。
- 仅余 MIDPOINT(2.08×) 与 T3(1.40×) 属 P3 SIMD 评估候选；其余热点已 ≤1.02×。

---

## 1. 已扫描的朴素 O(n·period) 窗口循环（实证清单）

| 位置 | 函数 | 是否可降为 O(n) | 处置 |
|------|------|----------------|------|
| `src/stat.rs:90-97` (`linreg_core`) | LINEARREG 族(5) | **可**（滑动 `sy`=`rolling_sum`，`sxy[i]=sxy[i-1]+period·x[i]−sy[i]`） | **T02 优化** |
| `src/stat.rs:300-314` (`correl_core`) | CORREL | **可**（s0/s1→`rolling_sum`；s00/s11→滚动平方和；s01→滚动叉积，均 O(1) 滑动） | **T03 优化** |
| `src/momentum.rs:650-660` (`willr`) | WILLR | **可**（改用已有 `rolling_max`/`rolling_min`） | **T04 优化** |
| `src/momentum.rs:1162-1178,1216-1230` (`stoch`/`stoch_f`) | STOCH/STOCHF | **可**（同上；STOCHRSI 经 `stoch` 继承） | **T04 优化** |
| `src/overlap.rs:570-598` (`bbands`) | BBANDS | 已 2×O(n) 单遍；可融合为 1 遍（边际） | **T01 融合（低优先）** |
| `src/momentum.rs:570` (`cci`) | CCI | **不可**（Σ\|tp−mean\| 均值随滑动变化，绝对值不可分离） | 保留，诚实说明 |
| `src/price_transform.rs:108` (`avgdev`) | AVGDEV | **不可**（同上，均值绝对偏差非可分离） | 保留，诚实说明 |
| `src/cycle.rs:552` (`for j in 0..50`) | HT_* 族 | 固定 50 步常量循环（非 period 缩放）；改写 HT 重写风险高 | 不动，诚实说明 |
| `src/core/mod.rs:241,245,275` | `wma_naive` 种子 | 仅种子期朴素求和（已优化 `wma` 主体） | 不动 |

**结论**：除已列出的"不可降"项（CCI/AVGDEV 均值绝对偏差、HT 固定 50 步）外，所有数值热点均已 O(n) 化或将被 T01–T04 降为 O(n)。CDL 模式函数（61）为整数 O(n) 蜡烛比较，非 ns/elem 对象，不动。

---

## 2. 有序任务清单 / Ordered Task List

> 每张任务 = 目标函数 + 文件 + 改动 + 依赖 + 验收（黄金测绿 + `cargo bench` 前后 ns/elem 对照 `BASELINE.md`）。

### T01 — BBANDS 融合核 + `bbands_with_output`（P2-4，优先级最低）
- **函数**：`bbands`
- **文件**：`src/overlap.rs`（`bbands` 改写 + 新增 `bbands_with_output(out: &mut BbandsOut)`）、`src/core/mod.rs`（新增 fused `rolling_mean_var`：单遍共享滑动 `sx`+`sxx`，同时产出 mean 与总体方差）、`benches/bbands_bench.rs`
- **改动**：将 `middle=rolling_mean` + `sd=rolling_var` 的两遍合并为单遍（一次窗口滑动同时维护 `sx`、`sxx`）；原 `Result<Bbands>` 委托 `_with_output`（D2）。
- **依赖**：无（独立）
- **验收**：`bbands` 黄金向量 1:1 通过；`cargo bench --bench bbands_bench` 前后对照；预期小幅或持平。若 ns/elem 不优于当前 5.61（≤1.02×C）且 test 绿则保留，否则回退原实现并标记"已验证无显著收益"。

### T02 — LINREG 族滑动 O(n)（P2-6）
- **函数**：`linear_reg` / `linear_reg_angle` / `linear_reg_intercept` / `linear_reg_slope` / `tsf`（共享 `linreg_core`）
- **文件**：`src/stat.rs`（`linreg_core` 改写 + 新增 naive 对比单测）、`benches/linreg_bench.rs`（新增，代表族）
- **改动**：维护滑动 `sy`（= `rolling_sum`，`sy[i]=sy[i-1]+x[i]−x[i-period]`）与 `sxy`，闭式 `sxy[i]=sxy[i-1]+period·x[i]−sy[i]`（推导见附录 A）；首个窗口朴素求和作种子，与历史逐项对齐。按 D2 补 `linear_reg_with_output` 等 5 个原地变体（可选，本任务一并加）。
- **依赖**：无
- **验收**：LINREG 族 5 函数黄金向量 1:1；新增 `linreg_core_matches_naive` 单测（多组 n/period 含单调/随机）逐项相等（容差 1e-9）；`linreg_bench` ns/elem 显著下降。

### T03 — CORREL 滑动 O(n)（P2-6）
- **函数**：`correl`
- **文件**：`src/stat.rs`（`correl_core` 改写 + naive 对比单测）、`src/core/mod.rs`（新增 `rolling_sum_sq` 原语，或在 `correl_core` 内联维护 `s00`/`s11`/`s01` 滑动）、`benches/correl_bench.rs`（新增）
- **改动**：`s0,s1`→`rolling_sum`；`s00=Σx²`、`s11=Σy²`→滚动平方和（`sxx[i]=sxx[i-1]+x[i]²−x[i-period]²`）；`s01=Σxy`→滚动叉积（同类滑动）；去窗口内 `period` 次重算。新增 `correl_with_output`（D2）。
- **依赖**：无
- **验收**：`correl` 黄金向量 1:1；新增 naive 对比单测逐项相等；`correl_bench` ns/elem 下降。

### T04 — WILLR + STOCH/STOCHF 窗口极值 O(n)（P2-6）
- **函数**：`willr`、`stoch`、`stoch_f`（STOCHRSI 经 `stoch` 自动继承）
- **文件**：`src/momentum.rs`（`willr`/`stoch`/`stoch_f` 改写）、`benches/willr_bench.rs`、`benches/stoch_bench.rs`（新增）
- **改动**：以已有且经 1:1 校验的 `rolling_max(high)` / `rolling_min(low)`（单调队列 O(n)，tie-break 取最右与朴素一致）替换每窗朴素 HH/LL 扫描；`fastk` 由 `rolling_min/rolling_max` 取窗口 min/max 后套用公式。补 `willr_with_output` / `stoch_with_output`（D2）。
- **依赖**：无（复用 `core` 已验证原语）
- **验收**：`willr`/`stoch`/`stoch_f` 黄金向量 1:1；`willr_bench`/`stoch_bench` ns/elem 下降（WILLR/STOCH 当前为朴素 O(n·period)，预期明显）。

### T05 — P2-5 微优 + P3 SIMD 裁决 + `_with_output` 扫尾（混合）
- **子项 5a（MIDPOINT 微优尝试）**：将 `rolling_minmax`/`rolling_extreme` 的 `VecDeque` 替换为**内联定长环形缓冲**（`Vec<usize>` + head/tail 索引，零额外堆分配、去 `VecDeque` 内部分支）。测量 ns/elem；**仅当改善且零偏差时保留**，否则回退并记录为"接受的局限"。
- **子项 5b（T3 微优尝试）**：`nested_ema_with_output` 加 `#[inline]`/手动展开 L 层级、`#![optimize(speed)]`（仅 bench）；保留若有改善。
- **子项 5c（P3 裁决 = NO-GO，详见 §3）**：记录证据（llvm-ir/asm 显示 auto-vec 失败，但瓶颈为**非向量化模式**：单调队列 / 顺序 EMA 递推；显式 SIMD 不适用单序列调用）。**不实现 SIMD**，`simd` feature 不引入。
- **子项 5d（`_with_output` 扫尾）**：为 T01–T04 触及的数值热路径补原地变体（已含 bbands/willr/stoch/correl/linreg 族），原 `Result<Vec>` API 形态不变（ADR 0001/D2）；Grep 确认其余 161 无其它 caller 重复 alloc 的低垂果。
- **文件**：`src/core/mod.rs`、`src/overlap.rs`、`src/momentum.rs`、`src/stat.rs`、`docs/NEXT-ACTIONS-perf.md`、`benches/*`
- **依赖**：T01–T04
- **验收**：全 `cargo test` 绿；ns/elem 不退化（±5% 点测噪声内）；P3 记录完整（§3）；`BASELINE.md` 更新。

### T06 — README 报告 + BASELINE 终稿（交付）
- **文件**：`README.md`、`README.zh-CN.md`、`benches/BASELINE.md`
- **改动**：按 §5 大纲补 (a)覆盖率 (b)正确性 (c)性能表 (d)优化小结 (e)已知局限（中英双语）；`BASELINE.md` 回填最终 ns/elem 矩阵。
- **依赖**：T01–T05
- **验收**：文档与最终基准数值一致；161/161 表述正确；列出诚实局限（见 §4）。

---

## 3. P3 显式 SIMD 评估闸门与裁决（MIDPOINT / T3）

**闸门（ADR 0010 P3-1）**：(a) `cargo rustc --release --emit=llvm-ir` 或 `cargo asm` 确认热循环 auto-vec **失败**，**且** (b) ns/elem >20% 慢于原生 C。

| 函数 | (a) auto-vec 失败？ | (b) >20% 慢？ | SIMD 是否适用瓶颈？ | 裁决 |
|------|-------------------|--------------|--------------------|------|
| **MIDPOINT (2.08×)** | 是（单调双队列 `rolling_minmax` 数据依赖、含 `while` 弹出分支，无法向量化） | 是 | **否**——滑动窗口 min/max 单调队列本身非 SIMD 友好；TA-Lib C 的 `TA_MIDPOINT` 同样用单遍 `MINMAXINDEX` 双队列（无 SIMD）。SIMD 需换算法（分块/排序）会改变数值与分支行为，不可能快于 C 同构实现。 | **NO-GO** |
| **T3 (1.40×)** | 是（`nested_ema_with_output` 为严格顺序 EMA 递推 `prev=(x−prev)·k+prev`，跨时间数据依赖，IIR 不可向量化） | 是 | **否**——EMA 递推本质顺序；SIMD 仅能对**多条独立序列（批/SoA）**并行，不改变单序列调用形态，且 TA-Lib C `TA_T3` 同为顺序嵌套 EMA（无 SIMD）。1.40× 差距源于 C 更紧的生成码，非缺失向量化。 | **NO-GO** |

**证据采集**（T05-5c 执行）：对 `rolling_minmax` 与 `nested_ema_with_output` 导出 LLVM IR，确认：
- `rolling_minmax` 内层为含 `VecDeque` 分支与条件弹出的标量循环；
- `nested_ema_with_output` 内层为带 `prev` 回边依赖的标量递推，LLVM 未发射向量指令。

**结论**：两函数**形式上满足闸门 (a)+(b)，但因瓶颈为非向量化算法模式，显式 SIMD 无法解决**，故 **P3 = NO-GO**。实际可收敛差距的杠杆在 P2-5（5a 环形缓冲重写 MIDPOINT、5b 内联/展开 T3），但预期仅能部分收窄，接受诚实状态。不引入 `simd` feature、不引入 `unsafe`/外部 crate。

---

## 4. 验证 + 基准方法论 / Verification & Benchmark Methodology

### 4.1 正确性（零偏差护栏，每次改动必跑）
- **全量 1:1**：`cargo test`（161 函数黄金向量，`tests/fixtures/*.json`，ADR 0005 容限 `rel 1e-8 + abs 1e-10`；STOCH/MACD 等敏感指标按 ADR 0005 放宽至 `1e-6`）。**必须全绿**。
- **原语零偏差单测**：`core::tests::{rolling_extreme_matches_naive, wma_matches_naive}` 已存在；T02/T03 新增 `linreg_core_matches_naive` / `correl_core_matches_naive`（多组 n/period 含单调/随机/重复值，逐项相等 ≤1e-9）。
- **退化判定**：任一测试失败即回退该改动。

### 4.2 性能对照（vs TA-Lib）
| 轨道 | 命令 | 口径 |
|------|------|------|
| Rust（零依赖） | `cargo bench` | 本仓 Rust ns/elem |
| 原生 C 双轨 | `cargo bench --features bench-c` | FFI 链接系统 `TA-Lib C 0.7.1`（`build.rs` 自动探测 `ta-lib`/`ta_lib`），**唯一用于 P3 闸门比较的基准** |
| Python 绑定 | `python3 tools/bench/compare.py` | 经 CPython↔C FFI + ndarray 拷贝，**非原生 C、仅量级参考，不得用于 P3 闸门** |

- **指标（数值型热路径）**：SMA、DEMA、TEMA、T3、WMA、MIDPOINT、MIDPRICE、BBANDS（既有 8）+ 新增 LINREG（族代表）、CORREL、WILLR、STOCH。度量 = **ns/elem** = `elapsed / ITERS / N`，`N=1_000_000`、`PERIOD=20`、`ITERS=20`，带 `checksum` 防优化（见 `benches/*.rs` 写法）。
- **无回归确认（每次改动后）**：
  1. `cargo test` 全绿（零偏差）；
  2. 重跑 `cargo bench`（及 `--features bench-c` 对应项），对照 `benches/BASELINE.md` 该指标条目：ns/elem 须 ≤ 改动前（点测 ±5% 噪声内）；Rust/C 比值不得恶化；
  3. 若 ns/elem 无改善且 test 绿（如 T01 BBANDS），保留并标注"已验证无显著收益"；若恶化 >5% 且无合理理由，**回退**。

---

## 5. README 报告大纲（zh + en，写入 README.md / README.zh-CN.md）

- **(a) 覆盖率 / Coverage**：161/161（按组：重叠 18、动量 31、波动率 3、成交量 3、价格变换 5、统计 9、周期 5、数学算子 11、数学变换 15、模式识别 61），含 61 个 CDL 默认 candle settings（ADR 0009）。
- **(b) 正确性验证小结 / Correctness**：权威黄金向量（TA-Lib C 0.7.1 真实输出，63 fixture，`tests/` 手写 JSON loader）；容限 ADR 0005（rel 1e-8 + abs 1e-10，敏感指标 1e-6）；运行 `cargo test` 全绿；零偏差承诺可证伪。
- **(c) 性能对照表 / Performance**：指标 × Rust ns/elem × 原生 C ns/elem × Rust/C 比值 × 状态（已完成/快于 C/持平/待评估）。数据取自最终 `benches/BASELINE.md`。
- **(d) 优化小结 / Optimization**：P2-1 嵌套 EMA 融合（DEMA/TEMA 快于 C、T3 1.40×）；P2-2 单调队列（MIDPRICE 快于 C、MIDPOINT 2.08×）；P2-3 WMA 滑动递推（0.94×）；P2-4 BBANDS 融合（持平，边际）；P2-6 LINREG/CORREL/WILLR/STOCH 降为 O(n)；P3 裁决 = **NO-GO**（理由见上）。
- **(e) 已知局限 / Known Limitations（诚实）**：
  - MIDPOINT(2.08×)、T3(1.40×) 仍慢于 C，根因为单调队列 / 顺序 EMA 递推非 SIMD 友好，显式 SIMD 不适用单序列（P3 NO-GO）；已尝试 P2-5 微优，部分收窄但接受现状。
  - CCI、AVGDEV 为窗口均值绝对偏差，**数学上不可分离为 O(1) 滑动**，维持 O(n·period)，非优化对象。
  - HT_* 周期指标含固定 50 步 Hilbert 变换循环，改写风险高，维持现状。
  - 61 个 CDL 模式函数为整数输出 O(n) 蜡烛比较，手写即快，不计入 ns/elem 对标。
  - `[dependencies]` 为空（Zero-FFI/No-Deps）；发布构建无 `unsafe`/`nightly`；`-C target-cpu=native` 仅用于 bench。

---

## 附录 A — LINREG `sxy` 滑动递推推导
窗口 `i` 的加权和 `sxy[i]=Σ_{k=0}^{period-1} k·x[i−period+1+k]`，令 `a_m=x[i−period+1+m]`（m=1..period−1）：
`sxy[i]−sxy[i−1] = Σ_{m=1}^{period−1} m(a_m−a_{m−1})`。
经分部求和恒等式得 `= (period−1)·a_{period−1} − Σ_{k=0}^{period−2} a_k`。
其中 `a_{period−1}=x[i]`、`Σ_{k=0}^{period−2} a_k = sy[i] − x[i]`（`sy[i]` 为窗口和，含 `x[i]`）。
故 **`sxy[i] = sxy[i−1] + period·x[i] − sy[i]`**，配合 `sy[i]=sy[i−1]+x[i]−x[i−period]`，每步 O(1)。种子（i=period−1）沿用朴素求和，与历史实现逐项对齐（1:1 黄金向量）。
