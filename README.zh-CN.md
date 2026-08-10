# adaq-talib

**AdaQ-TAlib** —— 纯 Rust、Zero-FFI、零依赖的 [TA-Lib](https://ta-lib.org) 0.7.1 技术指标复刻库。

> Pure-Rust, zero-FFI, dependency-free reimplementation of the
> [TA-Lib](https://ta-lib.org) 0.7.1 technical-analysis indicators.
>
> English version: [README.md](./README.md)

---

## 目录 / Contents

- [特性](#特性--features)
- [已实现功能总览](#已实现功能总览--implemented-functions)
- [快速开始](#快速开始--quick-start)
- [安装与依赖](#安装与依赖--installation)
- [使用说明](#使用说明--usage)
- [交互式示例](#交互式示例--interactive-demo)
- [验证与基准](#验证与基准--verification--benchmarks)
- [文档](#文档--documentation)
- [许可证](#许可证--license)
- [路线图](#路线图--roadmap)

---

## 特性 / Features

- **零偏差（zero-deviation）**：每个函数的数值输出与原版 TA-Lib 0.7.1 逐项一致（在浮点误差容限内，见 [ADR 0005](docs/adr/0005-error-tolerance.md)）。权威比对基于 TA-Lib C 0.7.1 生成的黄金向量（见 [ADR 0003](docs/adr/0003-verification-golden-fixtures.md)）。
- **Zero-FFI / No-Dependencies**：发布的库不调用任何 C ABI，`Cargo.toml` 的 `[dependencies]` 为空，全部算法原生手写。
- **惯用 Rust API（模型 B）**：切片入参（`&[f64]`）、`Result<_, TaError>` 出参、多输出以结构体返回；前导不稳定期以 `f64::NAN` 填充、等长返回（见 [ADR 0001](docs/adr/0001-api-fidelity-model.md) / [ADR 0007](docs/adr/0007-unstable-period.md)）。
- **性能优先**：在内存布局、循环分支、数组运算层面做优化；周期类（MAMA / HT_TRENDLINE）逐行移植官方 C 源码以保证位级一致。
- **无需系统依赖即可验证**：`cargo test` 对照已入库的黄金向量，普通用户运行测试**不需要** Python 或 TA-Lib C 库。

---

## 已实现功能总览 / Implemented Functions

截至当前版本，已覆盖 TA-Lib 0.7.1 的 **7 大类、共 65 个**指标函数。下表按模块列出全部公开函数、对应的 TA-Lib 原函数、默认参数与返回形态。

> 约定：每个函数都提供「显式参数」版本与「`_default`」便捷版本（使用 TA-Lib 默认参数，见 [`src/core/defaults.rs`](src/core/defaults.rs)）。

### 重叠研究 / Overlap Studies — `adaq_talib::overlap`

| 函数 | TA-Lib | 默认参数 | 返回 |
| --- | --- | --- | --- |
| `sma` / `sma_default` | `TA_SMA` | period = 30 | `Vec<f64>` |
| `ema` / `ema_default` | `TA_EMA` | period = 30 | `Vec<f64>` |
| `wma` / `wma_default` | `TA_WMA` | period = 30 | `Vec<f64>` |
| `dema` / `dema_default` | `TA_DEMA` | period = 30 | `Vec<f64>` |
| `tema` / `tema_default` | `TA_TEMA` | period = 30 | `Vec<f64>` |
| `midpoint` / `midpoint_default` | `TA_MIDPOINT` | period = 30 | `Vec<f64>` |
| `midprice` / `midprice_default` | `TA_MIDPRICE` | period = 30 | `Vec<f64>` |
| `bbands` / `bbands_default` | `TA_BBANDS` | period = 20, nb_dev = 2.0/2.0, SMA 中轨 | `Bbands { upper, middle, lower }` |
| `trima` / `trima_default` | `TA_TRIMA` | period = 30 | `Vec<f64>` |
| `t3` / `t3_default` | `TA_T3` | period = 5, vfactor = 0.7 | `Vec<f64>` |
| `ma` / `ma_default` | `TA_MA` | period = 30, `MaType::Sma` | `Vec<f64>`（按 `MaType` 派发） |
| `mavp` / `mavp_default` | `TA_MAVP` | min = 2 / max = 30, SMA | `Vec<f64>`（变周期） |
| `kama` / `kama_default` | `TA_KAMA` | period = 30 | `Vec<f64>` |
| `sar` / `sar_default` | `TA_SAR` | accel = 0.02, max = 0.2 | `Vec<f64>` |
| `sarext` / `sarext_default` | `TA_SAREXT` | 多空加速 0.02/0.02/0.2 | `Vec<f64>`（短侧为负值） |
| `MaType` | `TA_MAType` | `Sma/Ema/Wma/Dema/Tema/Trima/Kama/Mama` | 枚举（供 `ma`/`bbands`/`mavp` 选用） |

### 动量指标 / Momentum Indicators — `adaq_talib::momentum`

| 函数 | TA-Lib | 默认参数 | 返回 |
| --- | --- | --- | --- |
| `mom` / `mom_default` | `TA_MOM` | period = 10 | `Vec<f64>` |
| `roc` / `roc_default` | `TA_ROC` | period = 10 | `Vec<f64>` |
| `rocp` / `rocp_default` | `TA_ROCP` | period = 10 | `Vec<f64>` |
| `rocr` / `rocr_default` | `TA_ROCR` | period = 10 | `Vec<f64>` |
| `rocr100` / `rocr100_default` | `TA_ROCR100` | period = 10 | `Vec<f64>` |
| `rsi` / `rsi_default` | `TA_RSI` | period = 14 | `Vec<f64>` |
| `macd` / `macd_default` | `TA_MACD` | fast = 12, slow = 26, signal = 9 | `Macd { macd, signal, hist }` |
| `macd_fix` / `macd_fix_default` | `TA_MACDFIX` | fast = 12, slow = 26, signal = 9 | `Macd` |
| `macd_ext` / `macd_ext_default` | `TA_MACDEXT` | fast = 12, slow = 26, signal = 9 | `Macd`（默认全 EMA） |
| `apo` / `apo_default` | `TA_APO` | fast = 12, slow = 26 | `Vec<f64>` |
| `ppo` / `ppo_default` | `TA_PPO` | fast = 12, slow = 26 | `Vec<f64>` |
| `cmo` / `cmo_default` | `TA_CMO` | period = 14 | `Vec<f64>` |
| `cci` / `cci_default` | `TA_CCI` | period = 20 | `Vec<f64>` |
| `mfi` / `mfi_default` | `TA_MFI` | period = 14 | `Vec<f64>`（需成交量） |
| `willr` / `willr_default` | `TA_WILLR` | period = 14 | `Vec<f64>` |
| `bop` | `TA_BOP` | — | `Vec<f64>`（lookback 0） |
| `ultosc` / `ultosc_default` | `TA_ULTOSC` | 7 / 14 / 28 | `Vec<f64>` |
| `plus_dm` / `plus_dm_default` | `TA_PLUS_DM` | period = 14 | `Vec<f64>` |
| `minus_dm` / `minus_dm_default` | `TA_MINUS_DM` | period = 14 | `Vec<f64>` |
| `plus_di` / `plus_di_default` | `TA_PLUS_DI` | period = 14 | `Vec<f64>` |
| `minus_di` / `minus_di_default` | `TA_MINUS_DI` | period = 14 | `Vec<f64>` |
| `adx` / `adx_default` | `TA_ADX` | period = 14 | `Vec<f64>` |
| `adxr` / `adxr_default` | `TA_ADXR` | period = 14 | `Vec<f64>` |
| `aroon` / `aroon_default` | `TA_AROON` | period = 14 | `Aroon { up, down }` |
| `aroon_osc` / `aroon_osc_default` | `TA_AROONOSC` | period = 14 | `Vec<f64>` |
| `stoch` / `stoch_default` | `TA_STOCH` | fastK = 5, slowK = 3, slowD = 3 | `Stoch { slow_k, slow_d }` |
| `stoch_f` / `stoch_f_default` | `TA_STOCHF` | fastK = 5, fastD = 3 | `StochF { fast_k, fast_d }` |
| `stoch_rsi` / `stoch_rsi_default` | `TA_STOCHRSI` | rsi = 14, period = 14 | `Vec<f64>` |
| `trix` / `trix_default` | `TA_TRIX` | period = 30 | `Vec<f64>` |

### 波动率指标 / Volatility Indicators — `adaq_talib::volatility`

| 函数 | TA-Lib | 默认参数 | 返回 |
| --- | --- | --- | --- |
| `trange` | `TA_TRANGE` | — | `Vec<f64>`（lookback 0） |
| `atr` / `atr_default` | `TA_ATR` | period = 14 | `Vec<f64>` |
| `natr` / `natr_default` | `TA_NATR` | period = 14 | `Vec<f64>` |

### 成交量指标 / Volume Indicators — `adaq_talib::volume`

| 函数 | TA-Lib | 默认参数 | 返回 |
| --- | --- | --- | --- |
| `ad` | `TA_AD` | — | `Vec<f64>`（累计量，lookback 0） |
| `adosc` / `adosc_default` | `TA_ADOSC` | fast = 3, slow = 10 | `Vec<f64>` |
| `obv` | `TA_OBV` | — | `Vec<f64>`（累计量，lookback 0） |

### 价格变换 / Price Transform — `adaq_talib::price_transform`

| 函数 | TA-Lib | 默认参数 | 返回 |
| --- | --- | --- | --- |
| `avgprice` | `TA_AVGPRICE` | — | `Vec<f64>`（(H+L+C+O)/4） |
| `medprice` | `TA_MEDPRICE` | — | `Vec<f64>`（(H+L)/2） |
| `typprice` | `TA_TYPPRICE` | — | `Vec<f64>`（(H+L+C)/3） |
| `wclprice` | `TA_WCLPRICE` | — | `Vec<f64>`（(H+L+2C)/4） |

### 统计函数 / Statistic Functions — `adaq_talib::stat`

| 函数 | TA-Lib | 默认参数 | 返回 |
| --- | --- | --- | --- |
| `stddev` / `stddev_default` | `TA_STDDEV` | period = 5, nb_dev = 1.0 | `Vec<f64>` |
| `var` / `var_default` | `TA_VAR` | period = 5, nb_dev（被忽略） | `Vec<f64>` |
| `linear_reg` / `linear_reg_default` | `TA_LINEARREG` | period = 14 | `Vec<f64>` |
| `linear_reg_angle` / `linear_reg_angle_default` | `TA_LINEARREG_ANGLE` | period = 14 | `Vec<f64>`（角度，度） |
| `linear_reg_intercept` / `linear_reg_intercept_default` | `TA_LINEARREG_INTERCEPT` | period = 14 | `Vec<f64>` |
| `linear_reg_slope` / `linear_reg_slope_default` | `TA_LINEARREG_SLOPE` | period = 14 | `Vec<f64>` |
| `tsf` / `tsf_default` | `TA_TSF` | period = 14 | `Vec<f64>` |
| `beta` / `beta_default` | `TA_BETA` | period = 5 | `Vec<f64>` |
| `correl` / `correl_default` | `TA_CORREL` | period = 5 | `Vec<f64>` |

### 周期类（希尔伯特变换）/ Cycle — `adaq_talib::cycle`

| 函数 | TA-Lib | 默认参数 | 返回 |
| --- | --- | --- | --- |
| `mama` / `mama_default` | `TA_MAMA` | fast = 0.5, slow = 0.05 | `Mama { mama, fama }` |
| `ht_trendline` / `ht_trendline_default` | `TA_HT_TRENDLINE` | — | `Vec<f64>`（lookback 63） |

### 错误类型 / Error Type

- `TaError`（`adaq_talib::TaError`）—— 公开错误枚举，语义映射 TA-Lib `TA_RetCode`：
  `BadParam` / `OutOfRange` / `LibNotInitialized` / `OutOfMemory` / `InternalError`（见 [ADR 0006](docs/adr/0006-type-error-model.md)）。

---

## 快速开始 / Quick Start

```rust
use adaq_talib::overlap::sma;

let prices = [1.0, 2.0, 3.0, 4.0, 5.0];
let out = sma(&prices, 3).unwrap();
// out 与 prices 等长；前导 2 个位置为 NaN，其余为窗口均值。
// `out` has the same length as `prices`; the first 2 positions are NaN, the rest are window means.
assert!(out[0].is_nan());
assert!((out[2] - 2.0).abs() < 1e-9);
```

多输出示例（布林带、MACD）：

```rust
use adaq_talib::overlap::bbands_default;
use adaq_talib::momentum::macd_default;

let close = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0,
                 20.0, 19.0, 18.0, 17.0, 16.0, 15.0, 14.0, 13.0, 12.0, 11.0];

// 布林带：三轨等长，前导不稳定期填 NaN。
// Bollinger Bands: three equal-length bands, leading unstable period is NaN.
let b = bbands_default(&close).unwrap();
let _ = (b.upper, b.middle, b.lower);

// MACD：macd / signal / hist 三列，对齐到同一前导 NaN。
// MACD: macd / signal / hist, aligned to the same leading NaN.
let m = macd_default(&close).unwrap();
let _ = (m.macd, m.signal, m.hist);
```

OHLC 输入示例（ATR，需 high/low/close）：

```rust
use adaq_talib::volatility::atr_default;

// 用 40 根样例序列以便看到首个有效值（前导 period-1 = 13 个为 NaN）。
// A 40-bar sample so the first valid value is observable (leading period-1 = 13 are NaN).
let high:  Vec<f64> = (0..40).map(|i| 10.0 + i as f64 * 0.1 + 0.5).collect();
let low:   Vec<f64> = (0..40).map(|i| 9.0  + i as f64 * 0.1 - 0.5).collect();
let close: Vec<f64> = (0..40).map(|i| 9.5  + i as f64 * 0.1).collect();

let out = atr_default(&high, &low, &close).unwrap();
// 前导 period-1 = 13 个为 NaN；首个有效值 = 前 14 个 TR 的均值（Wilder 平滑种子）。
assert!(out[0].is_nan() && out[12].is_nan());
assert!(!out[13].is_nan()); // 首个有效位于索引 period-1 = 13
```

---

## 安装与依赖 / Installation

### 环境依赖 / Requirements

- **Rust 工具链 ≥ 1.85**（本库 `edition = "2024"`，见 `Cargo.toml` 的 `rust-version`）。安装：[rustup.rs](https://rustup.rs)。
- **无需**任何外部 crate：发布的库 `[dependencies]` 为空。
- **构建与测试无需** TA-Lib C 库或 Python（黄金向量已入库）。

### 添加依赖 / Add the dependency

若已发布到 crates.io：

```bash
cargo add adaq_talib
```

或手动在 `Cargo.toml` 中加入：

```toml
[dependencies]
adaq-talib = "0.1"
```

从源码构建（克隆后）：

```bash
git clone https://github.com/tonywxx/adaq-talib
cd adaq-talib
cargo build --release
```

---

## 使用说明 / Usage

### 1. 调用形态 / Calling conventions

- **单输入序列**：如 `sma(&prices, period)`、`rsi(&close, period)`。
- **OHLC 多输入**：如 `atr(&high, &low, &close, period)`、`cci(&high, &low, &close, period)`、`bop(&open, &high, &low, &close)`。
- **含成交量**：如 `mfi(&high, &low, &close, &volume, period)`、`obv(&close, &volume)`。
- **多输出**：以专用结构体返回，例如 `Macd { macd, signal, hist }`、`Bbands { upper, middle, lower }`、`Aroon { up, down }`、`Mama { mama, fama }`。

### 2. 默认参数便捷函数 / Default-parameter helpers

每个函数都配套 `_default` 版本，使用 TA-Lib 原版默认参数，省去手写：

```rust
use adaq_talib::momentum::rsi_default;
use adaq_talib::volatility::atr_default;

let rsi = rsi_default(&close).unwrap();   // 默认 period = 14
let atr = atr_default(&high, &low, &close).unwrap(); // 默认 period = 14
```

### 3. 错误处理 / Error handling

所有函数返回 `Result<_, TaError>`。常见错误为 `TaError::BadParam`（如 `period == 0`、输入数组长度不一致、`mama` 的 `fast/slow_limit` 不在 [0.01, 0.99]）。

```rust
use adaq_talib::overlap::sma;
use adaq_talib::TaError;

match sma(&[1.0, 2.0], 0) {
    Ok(out) => println!("{out:?}"),
    Err(TaError::BadParam(msg)) => eprintln!("参数错误 / bad param: {msg}"),
    Err(e) => eprintln!("其他错误 / other error: {e}"),
}
```

### 4. 前导不稳定期与 NaN / Unstable period & NaN

与原版 TA-Lib 一致，预热阶段（不稳定期）的输出以 `f64::NAN` 填充，且**返回值与输入等长**。常见 lookback：

- SMA/EMA/WMA/TRIMA 等：`period - 1`
- DEMA：`2*(period-1)`；TEMA：`3*(period-1)`；T3：`6*(period-1)`；TRIX：`3*period-2`
- RSI/CMO/CCI 等：`period`（Wilder 类）
- ADX：`2*period-1`；ADXR：`3*period-2`
- MAMA：`32`；HT_TRENDLINE：`63`
- 无滞后（lookback 0）：`trange`、`ad`、`obv`、价格变换类、`bop`

消费结果时请用 `f64::is_nan()` 跳过不稳定期，再取有效段。

### 5. 在代码库之外复用 / Outside the library

`src/utils.rs` 仅作 `doc(hidden)` 内部实现细节（对齐、范围检查等），不属于公开 API，请勿在外部依赖。

---

## 交互式示例 / Interactive Demo

`src/main.rs` 是一个命令行演示入口，按指标名运行内置示例，**无需编写代码**：

```bash
cargo run -- sma
cargo run -- rsi
cargo run -- macd
cargo run -- bbands
cargo run -- atr
cargo run -- adx
cargo run -- mama
```

支持的全部指标（可直接 `cargo run -- <名称>`，按类别排列）：

```
重叠研究 / Overlap:    sma ema wma dema tema midpoint midprice bbands trima t3 ma mavp kama sar sarext
动量 / Momentum:       rsi cmo trix mom macd cci willr bop ultosc adx aroon stoch mfi
波动率 / Volatility:   trange atr natr
成交量 / Volume:       ad adosc obv
价格变换 / Price:      avgprice medprice typprice wclprice
统计 / Statistic:      stddev var linear_reg linear_reg_angle linear_reg_intercept linear_reg_slope tsf beta correl
周期 / Cycle:          mama ht_trendline
```

> 注：交互式示例覆盖上述指标。其余已实现的指标（如 `roc`/`rocp`/`rocr`/`rocr100`、`apo`/`ppo`、
> `stoch_f`/`stoch_rsi`、`aroon_osc`、`plus_dm`/`minus_dm`/`plus_di`/`minus_di`、`adxr`、
> `macd_fix`/`macd_ext` 等）均可直接在代码中调用，详见上方[功能总览](#已实现功能总览--implemented-functions)。
> / The interactive demo covers the indicators above. The remaining implemented functions
> (e.g. `roc`, `rocp`, `apo`, `ppo`, `stoch_f`, `stoch_rsi`, `aroon_osc`, the directional
> components, `adxr`, `macd_fix`, `macd_ext`, …) are callable directly in code — see the
> [function overview](#implemented-functions) above.

> 未知指标名会打印支持列表并以退出码 2 结束。/ An unknown name prints the supported list and exits with code 2.

---

## 验证与基准 / Verification & Benchmarks

### 正确性验证 / Correctness

- 对照已入库的**黄金向量**（由 TA-Lib C 0.7.1 真实输出生成，位于 `tests/fixtures/`，共 63 个）运行 `cargo test`，普通用户**无需** Python 或 TA-Lib C 库。
- 容限策略：相对 `1e-8` + 绝对 `1e-10`（见 [ADR 0005](docs/adr/0005-error-tolerance.md)）。
- 黄金向量生成工具见 [`tools/gen_fixtures`](tools/)（需系统安装 TA-Lib C 库 + `TA-Lib` Python 绑定，仅供维护者重生成 fixture）。当前 fixture 状态与已知缺口见 [`tools/README.md`](tools/README.md)。

```bash
cargo test                 # 单元 + 黄金向量比对（无需 Python / C）
cargo test --doc           # 仅运行文档示例
```

### 性能基准 / Benchmarks

双轨基准（见 [ADR 0004](docs/adr/0004-benchmark-dual-track.md)）：

```bash
# 1) Rust 侧（默认，零依赖）：std::time 计时，harness = false
cargo bench --bench sma_bench

# 2) 原生 C 对照（可选 feature）：FFI 链接系统 TA-Lib C 库
cargo bench --bench sma_bench --features bench-c
```

> 第 2 种需系统已安装 TA-Lib C 库（`brew install ta-lib` / 源码编译）；`build.rs` 仅在 `bench-c` 下链接，未启用时构建不受影响。报告须明确区分两种口径。

---

## 文档 / Documentation

- 设计决策（ADR 0001–0009）：[`docs/adr/`](docs/adr/)
- 统一 API 写法：[`docs/api-conventions.md`](docs/api-conventions.md)
- 0.1.0 函数范围基线：[`docs/0.1.0-scope.md`](docs/0.1.0-scope.md)
- 术语表：[`CONTEXT.md`](CONTEXT.md)
- 每个公开函数均含中英双语 doc-comment（公式来源、参数、返回值、前导 `NaN`、可运行示例），可直接 `cargo doc --open` 浏览。

---

## 许可证 / License

Apache-2.0（见 [`LICENSE`](LICENSE)）。

---

## 路线图 / Roadmap

采用里程碑式发布（见 [ADR 0002](docs/adr/0002-release-scope-milestones.md)），**最终全量覆盖 TA-Lib 0.7.1 且不删减任何已发布能力**：

- ✅ **0.1.0（当前）**：重叠研究 + 动量 + 波动率 + 成交量 + 价格变换 + 统计 + 周期（MAMA/HT_TRENDLINE），共 65 个函数。
- ⏳ **后续里程碑**：模式识别（蜡烛形态，~61，仅默认 candle settings，见 [ADR 0009](docs/adr/0009-candle-settings-default-only.md)）、数学算子（7：ADD/DIV/MAX/MIN/MULT/SUB/SUM）、数学变换（15：ACOS/ASIN/ATAN/CEIL/COS/COSH/EXP/FLOOR/LN/LOG10/SIN/SINH/SQRT/TAN/TANH）。

完成上述后，adaq-talib 即与 TA-Lib 0.7.1 等价全量覆盖。
