# tools/ — 开发工具（非发布产物）

本目录下的脚本是**开发工具**，不属于发布的 crate，不进入 `dependencies`。

## gen_fixtures/ — 黄金向量生成（见 ADR 0003）

生成与 TA-Lib 原版逐项一致的参考输出（黄金向量），入库为 `tests/fixtures/*.json`，
供 `cargo test` 比对。**普通用户运行 `cargo test` 无需 Python / TA-Lib。**

### 前置要求

- 系统已安装 TA-Lib C 库（`brew install ta-lib` / 或从源码编译）。
- Python 3 + `pip install TA-Lib`（注意：PyPI 包名是 `TA-Lib`，导入名 `talib`，
  且必须先装 C 库才能安装/运行）。

### 运行

```
python tools/gen_fixtures/generate.py
```

### 版本口径（重要）

黄金向量的"零偏差"基准是 TA-Lib **0.7.1**。生成前请确认所用 C 库版本，并在本文件登记：

- 当前登记 C 库版本：0.7.1

重建 C 库版本或换机器后，必须重生成 fixture 并评审差异。

### 当前 fixture 状态（重要）

- 全部 `tests/fixtures/*.json`（共 63 个）现已由 `generate.py` 基于 **TA-Lib C 0.7.1**
  （`talib` Python 绑定，已确认版本 0.7.1）真实输出生成，是**权威黄金向量**，
  不再携带 `_note: REFERENCE` 字段。`cargo test` 对它们的比对即等价于与原版逐项 1:1 校验。
- 普通用户运行 `cargo test` 仍**无需** Python / TA-Lib —— fixture 已入库。
- 已知缺口（已全部闭合）：A0.1 范围内的全部函数（Overlap 17 / Momentum 30 / Volatility 3 /
  Volume 3 / Price Transform 4 / Stat 9）现已在权威黄金向量下 1:1 通过 `cargo test`。
  此前偏离原版的 13 个动量函数（`cmo`、`macd`、`stoch_rsi`、`ultosc`、`plus_dm`、`minus_dm`、
  `plus_di`、`minus_di`、`adx`、`adxr`、`aroon`、`aroon_osc`、`trix`）均已对照 TA-Lib C 源修正。
- 与原版保持兼容的几个非显然约定（已在对应函数 doc-comment 中标注）：
  - `aroon` / `aroon_osc`：已安装 `talib` 0.7.1 构建的 `outAroonUp`/`outAroonDown` 输出**互换**
    （`aroon_osc = up - down` 仍按正确公式），Rust 与之逐项对齐；上游 C 源（0.4.0 / 0.7.1 tag /
    main）实现的是非互换的标准算法。
  - `beta`（TA-Lib `TA_BETA`）：基于相邻价格的**收益率**（相对变化）做回归，lookback = period；
    BETA 不是原始价格的 `cov/var`。
  - `adosc`（TA-Lib `TA_ADOSC`）：快/慢 EMA 均以**首个 A/D 值**为种子（非 SMA），与 Metastock 一致。
  - `trange`（TA-Lib `TA_TRANGE`）：索引 0 因无前收盘价输出 `NaN`（lookback 1）；
    故下游 `atr`/`natr` 的首个有效点落在索引 `period`。

## bench/ — Python 便捷基准对照（见 ADR 0004）

（规划中：调用 TA-Lib Python 绑定计时，生成对照报告，如实标注为 "TA-Lib Python binding"。
具体实现随基准需求落地。）
