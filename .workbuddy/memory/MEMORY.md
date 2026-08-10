# 项目长期记忆 — adaq-talib

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
