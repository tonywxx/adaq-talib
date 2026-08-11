# 项目长期记忆 — adaq-talib

## 首要根基准则（不可动摇的双重基准 / 一切优化工作的唯一执行依据）
- **声明（用户于 2026-08-10 明确确立）**：本项目的首要且不可动摇的根基准则是——
  所有技术指标的计算结果必须在 **100% 严格对标原官方 TA-Lib 输出** 的前提下，最大化地优化程序性能。
- 准确性（与 TA-Lib 逐项 1:1）与性能（最大化）为本项目的**双重基准**，二者优先级最高、并列不可偏废。
- **任何优化（含架构深化、模块重构、hot-path 改写）均须严格遵循此准则**：
  - 准确性 > 性能：绝不为了性能牺牲对标精度；任何使输出偏离 TA-Lib 的"优化"一律禁止。
  - 性能次之：在精度零退化的前提下追求性能最大化。
  - **验收强制闸**：任何优化变更必须通过现有黄金向量 fixture（tests/fixtures/*，1e-8 相对 + 1e-10 绝对）
    与全部 `cargo test` 的 1:1 校验，方可视为完成。
  - hot-path 重构须避免引入间接调用/每轮分配等会退化性能的写法（如优先 `impl Fn` 单态化内联，禁用 `dyn Fn`）。
- 此准则凌驾于纯"代码整洁/深度"考量之上：候选①（pattern 骨架坍缩为 `run_pattern`）作为纯重构，其价值须
  在"精度零退化 + 性能不退化（力争更优）"的尺度下重新评估，而非仅以可维护性论。
- **度量前置协议（必守，由候选① 实测教训沉淀）**：任何"消除重复/统一抽象"类重构，合并前**必须**做 A/B
  基准——新路径 vs 重构前逐字内联基线（同逻辑、独立局部变量），在 release 下测 ns/elem Δ；仅当 Δ ≤ ±5%
  （点测噪声）方可合并。**不得凭"抽象更少=更快"直觉**。实测反例：候选① 把异质 `CandleAvg` 收拢进 `&[CandleAvg]`
  同质切片，剥离了 per-avg 编译期静态优化（`setting` 分支无法折叠、`total/trailing` 内存往返），致
  +5.8%（4 avg）/ +8.1%（10 avg）退化 → **已 revert**。正确去重复路径是用 `macro_rules!` 展开为独立局部
  avg 变量以保留静态优化。

## 开发规约（grilling 沉淀，详见 docs/adr 与 docs/api-conventions.md）
- 纯 Rust、Zero-FFI、No-Dependencies（`[dependencies]` 为空）。
- API 模型 B：惯用 Rust 封装，数值与 TA-Lib 0.7.1 逐项 1:1。
- 模块按 TA-Lib 类别分（overlap/momentum/volume/volatility/price_transform/cycle/pattern/stat/math_ops/math_trans），私有 `core`/`utils`。
- 单输出返回 `Result<Vec<f64>, TaError>`；多输出返回专用结构体；前导不稳定期填 `f64::NAN`，等长返回。
- 错误类型 `TaError`（映射 `TA_RetCode`）。
- 验证：入库黄金向量（`tools/gen_fixtures` Python 生成），普通 `cargo test` 零 Python 依赖；误差 1e-8 相对 + 1e-10 绝对。
- 基准双轨：`benches/` 可选 feature `bench-c` FFI 对照原生 C；`tools/bench` Python 便捷对照（标注绑定层）。
- 许可证 Apache-2.0（repo: github.com/tonywxx/adaq-talib）。
- 里程碑：0.1.0 先重叠/动量/波动率/成交量约 70 函数，最终全量不删减；模式识别仅默认 candle settings。
- 文档：公开函数中英双语 doc-comment（公式来源/参数/返回值/前导 NaN/示例）；`lib.rs` 顶层文档。
- 0.1.0 函数范围基线：`docs/0.1.0-scope.md`（重叠17/动量30/波动率3/成交量3/价格变换4/统计9，≈66）。
- 已实现 overlap 7 函数：sma/ema/wma/dema/tema/midpoint/midprice（core 原语：rolling_mean/ema/wma/rolling_max/min）。
- 黄金向量 fixture（`tests/fixtures/*.json`，共 63 个）现已是**权威基准**：由已安装 `talib` 0.7.1
  （Cython 绑定 `libta-lib.0.7.1.dylib`）真实输出生成/校验，全部 `cargo test` 1:1 通过；
  普通测试零 Python/C 依赖。生成/校验脚本：`tools/gen_fixtures/generate.py`。

## TA-Lib 0.7.1 与文档/直觉不符的非显然约定（逐项 1:1 必须对照）
- `aroon`/`aroon_osc`：已安装 `talib` 0.7.1 的 `outAroonUp`/`outAroonDown` **互换**（dylib 为不同/带 bug
  修订）；`aroon_osc = up - down` 仍按正确公式。上游 C 源（0.4.0 / 0.7.1 tag / main）实现非互换标准算法。
  Rust 与已安装构建对齐。
- `beta`（`TA_BETA`）：基于相邻价格的**收益率**（相对变化）做线性回归，lookback = period；
  **不是**原始价格的 `cov/var`。C 算法见 `ta_BETA.c`（S_xx/S_xy 累加 + 尾部移除流式）。
- `adosc`（`TA_ADOSC`）：快/慢 EMA 均以**首个 A/D 值**为种子（非 SMA，与 Metastock 一致），
  k = 2/(period+1)。`ad` 线为累计量（AD[0] = vol[0]*CLV[0]）。
- `trange`（`TA_TRANGE`）：索引 0 因无前收盘价输出 `NaN`（lookback 1）；故 `atr`/`natr` 首个
  有效点落在索引 `period`（lookback = period），Wilder 种子 = 前 `period` 个有效 TR 的均值。
- `TA_IS_ZERO(v)` = `(-1e-8 < v) && (v < 1e-8)`（C 侧收益率分母保护），移植时需保留该 epsilon 判定。

## 性能对标可行性结论（2026-08-10 深度研究，权威）
- **161/161 全量 >2× 快于原生 C 在现有硬约束（Zero-FFI/No-Deps/单线程/safe/SIMD 延后）下不可达**：严格递推指标（EMA/RSI/MACD/ATR/ADX/DX/STOCH/CMO/TRIX/APO/PPO/KAMA/SAR/MAMA ~40 + 全部 Cycle 5）单线程触及每元素最小工作量地板，递推禁止跨时间向量化。
- **可达子集**：Elementwise/可向量化（math/price_transform/部分 stat）已自然 >2×；Pattern Recognition 消除冗余后单线程可超 1×、部分近 2×，全量 >2× 需并行（放宽 No-Deps 或 unsafe 手搓线程）；顺序 IIR 至多 parity（1×）。
- **实测现状**（speedup=C/adaq）：≥2× 仅 14、107 慢于 C；Pattern 平均 0.38×（57/61 落后）、Cycle 0.68×。
- **基准方法学坑**：非蜡烛 C 计时走抽象 API `TA_CallFunc`（per-call scratch 分配，抬高 C）；蜡烛走直连 FFI（干净）→ Pattern 0.38× 真实。修正非蜡烛为直连 FFI 只会拉大 Rust 劣势（基准对 Rust 偏乐观）。
- **已验证 PoC**：`cdl_hammer` 改"每 bar 原语算一次 + 内联 running-sum"（与 `CandleAvg` 同递推、逐位一致）→ adaq 12.64→1.60 ns/elem，Rust/C 4.42→0.571（单线程 1.75× 快于 C），黄金向量 1:1 保持。即候选① revert 后"`macro_rules!` 展开独立局部 avg 变量"路径的手写印证；推广至 61 CDL 须逐函数核对 settings/off/range 并 A/B 实测（双基准：Δ≤±5% + 1:1）。
- 建议 KPI：由"161/161 全部 >2×"改为"消除所有 <1× 伪慢（全量 ≥1×）+ 可并行子集 >2×"。
