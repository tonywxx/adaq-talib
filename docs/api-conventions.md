# API 约定（adaq-talib）

> 由 grilling 会话沉淀的可执行约定。ADR 记录难以回退的决策，本文记录统一写法。

## 1. 模块布局（关联 ADR 0001/0002）
- 按 TA-Lib 类别分模块：`overlap` / `momentum` / `volume` / `volatility` / `price_transform` / `cycle` / `pattern` / `stat` / `math_ops` / `math_trans`。
- 私有模块：`core`（公共数学：SMA、方差、线性回归、EMA 状态机等）、`utils`（对齐、范围检查等）。
- 公共 API 以模块路径为主（`adaq_talib::overlap::sma`），并在 crate 根对高频函数 re-export。

## 2. 命名
- 函数名采用 Rust `snake_case`，取自 TA-Lib 原名去 `TA_` 前缀：`TA_SMA`→`sma`，`TA_RSI`→`rsi`，`TA_MACD`→`macd`，`TA_BBANDS`→`bbands`。
- 多输出结构体字段用小写蛇形、语义明确：`Bbands { upper, middle, lower }`。

## 3. 返回值（关联 ADR 0006/0007）
- 单输出：直接 `Result<Vec<f64>, TaError>`。
- 多输出：专用结构体，`Result<Struct, TaError>`；前导不稳定期填 `NaN`（见 ADR 0007）。

## 4. 可选入参（optIn*）
- 函数显式接收全部参数；另提供使用 TA-Lib 默认常量的便捷函数（如 `macd(prices)` = `macd_with(prices, 12, 26, 9)`）。
- 默认常量集中定义于 `core::defaults`。

## 5. 注释与文档（满足 crates.io 展示标准）
- 每个公开函数含中英双语 doc-comment：公式来源（如 "Wilder's smoothing, TA-Lib ref"）、参数释义、返回值、前导 `NaN` 说明、可运行示例。
- 关键算法逻辑处加中英双语行内注释，标注公式/推导来源。
- crate 根 `lib.rs` 含顶层库文档：定位、覆盖范围、已覆盖/待覆盖清单、零偏差说明、性能与 Zero-FFI/No-Deps 声明。

## 6. 黄金向量 fixture 格式（关联 ADR 0003/0005）
- 输入数据集：JSON（人可读、便于重生成）。
- 黄金输出向量：二进制 `f64` 数组（紧凑、精确），配套 JSON 元数据（指标名、参数、C 库版本、长度）。
- 存放于 `tests/fixtures/`。

## 7. examples/demo.rs 交互式示例入口
- `examples/demo.rs` 作为可运行演示入口：基于命令行参数选择指标类别/函数并运行示例（非 REPL），覆盖每一类技术指标。
- 内置小型样本数据，运行 `cargo run --example demo -- <indicator>` 即可演示。
