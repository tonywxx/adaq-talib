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
- 已知缺口：截至本轮，动量类 13 个函数在权威向量下仍 FAIL（Rust 实现与原版存在偏差，
  需对照 TA-Lib C 源逐项修正）：`cmo`、`macd`(signal 种子)、`stoch_rsi`/`ultosc`(NaN 对齐)、
  以及 `plus_dm`/`minus_dm`/`plus_di`/`minus_di`/`adx`/`adxr`/`aroon`/`aroon_osc`/`trix`
  （Wilder/EMA/周期约定）。详见对应 issue。

## bench/ — Python 便捷基准对照（见 ADR 0004）

（规划中：调用 TA-Lib Python 绑定计时，生成对照报告，如实标注为 "TA-Lib Python binding"。
具体实现随基准需求落地。）
