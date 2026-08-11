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
- [已知问题与非推荐特性](#已知问题与非推荐特性--known-issues--deprecations)
- [文档](#文档--documentation)
- [许可证](#许可证--license)
- [路线图](#路线图--roadmap)
- [变更日志](#变更日志--changelog)

---

## 特性 / Features

- **零偏差（zero-deviation）**：每个函数的数值输出与原版 TA-Lib 0.7.1 逐项一致（在浮点误差容限内，见 [ADR 0005](docs/adr/0005-error-tolerance.md)）。权威比对基于 TA-Lib C 0.7.1 生成的黄金向量（见 [ADR 0003](docs/adr/0003-verification-golden-fixtures.md)）。
- **Zero-FFI / No-Dependencies**：发布的库不调用任何 C ABI，`Cargo.toml` 的 `[dependencies]` 为空，全部算法原生手写。
- **惯用 Rust API（模型 B）**：切片入参（`&[f64]`）、`Result<_, TaError>` 出参、多输出以结构体返回；前导不稳定期以 `f64::NAN` 填充、等长返回（见 [ADR 0001](docs/adr/0001-api-fidelity-model.md) / [ADR 0007](docs/adr/0007-unstable-period.md)）。
- **性能优先**：在内存布局、循环分支、数组运算层面做优化；周期类（MAMA / HT_TRENDLINE）逐行移植官方 C 源码以保证位级一致。
- **无需系统依赖即可验证**：`cargo test` 对照已入库的黄金向量，普通用户运行测试**不需要** Python 或 TA-Lib C 库。

---

## 已实现功能总览 / Implemented Functions

adaq-talib 现已实现完整的 TA-Lib 0.7.1 公开函数面 —— **10 大类、共 161 个**函数，零偏差（逐项对照黄金向量验证，见[验证与基准](#验证与基准--verification--benchmarks)）。下表列出每个公开函数、对应的 TA-Lib 原函数、默认参数与返回形态。

| 类别 | 模块 | 数量 | TA-Lib 分组 |
| --- | --- | ---: | --- |
| 重叠研究 | `overlap` | 18 | Overlap Studies |
| 动量指标 | `momentum` | 31 | Momentum Indicators |
| 波动率指标 | `volatility` | 3 | Volatility Indicators |
| 成交量指标 | `volume` | 3 | Volume Indicators |
| 价格变换 | `price_transform` | 5 | Price Transform |
| 统计函数 | `stat` | 9 | Statistic Functions |
| 周期（希尔伯特变换） | `cycle` | 7\* | Cycle Indicators (5) + Overlap (2)† |
| 数学算子 | `math_ops` | 11 | Math Operators |
| 数学变换 | `math_trans` | 15 | Math Transform |
| 模式识别 | `pattern` | 61 | Pattern Recognition |
| **合计** | | **161** | **161** |

\* `cycle` 模块含 7 个函数；TA-Lib 将其中 5 个归入 *Cycle Indicators*（`HT_DCPERIOD`/`HT_DCPHASE`/`HT_PHASOR`/`HT_SINE`/`HT_TRENDMODE`），2 个归入 *Overlap Studies*（`MAMA`/`HT_TRENDLINE`）。† 分组依据 TA-Lib 权威 `info['group']`。

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
| `accbands` / `accbands_default` | `TA_ACCBANDS` | period = 20 | `AccBands { upper, middle, lower }` |
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
| `imi` / `imi_default` | `TA_IMI` | period = 14 | `Vec<f64>`（开/收） |
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
| `dx` / `dx_default` | `TA_DX` | period = 14 | `Vec<f64>`（OHLC） |
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
| `avgdev` / `avgdev_default` | `TA_AVGDEV` | period = 14 | `Vec<f64>` |
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
| `ht_dcperiod` / `ht_dcperiod_default` | `TA_HT_DCPERIOD` | — | `Vec<f64>`（主导周期） |
| `ht_dcphase` / `ht_dcphase_default` | `TA_HT_DCPHASE` | — | `Vec<f64>`（主导相位） |
| `ht_phasor` / `ht_phasor_default` | `TA_HT_PHASOR` | — | `Phasor { in_phase, quadrature }` |
| `ht_sine` / `ht_sine_default` | `TA_HT_SINE` | — | `HtSine { sine, lead_sine }` |
| `ht_trendmode` / `ht_trendmode_default` | `TA_HT_TRENDMODE` | — | `Vec<f64>`（0/1 趋势模式） |

### 数学算子 / Math Operators — `adaq_talib::math_ops`

对单条或两条等长序列做逐元素 / 数组运算，全部返回 `Vec<f64>`（等长；lookback 0）。二元算子入参 `(&[f64], &[f64])`；`maxindex`/`minindex`/`minmax`/`minmaxindex` 在滑动窗口上做归约。

| 函数 | TA-Lib | 签名 | 返回 |
| --- | --- | --- | --- |
| `add` / `add_default` | `TA_ADD` | `(a, b)` | `Vec<f64>` |
| `sub` / `sub_default` | `TA_SUB` | `(a, b)` | `Vec<f64>` |
| `mult` / `mult_default` | `TA_MULT` | `(a, b)` | `Vec<f64>` |
| `div` / `div_default` | `TA_DIV` | `(a, b)` | `Vec<f64>` |
| `sum` / `sum_default` | `TA_SUM` | `(a, period)` | `Vec<f64>` |
| `min` / `min_default` | `TA_MIN` | `(a, period)` | `Vec<f64>` |
| `max` / `max_default` | `TA_MAX` | `(a, period)` | `Vec<f64>` |
| `min_index` / `min_index_default` | `TA_MININDEX` | `(a, period)` | `Vec<f64>`（最小值索引） |
| `max_index` / `max_index_default` | `TA_MAXINDEX` | `(a, period)` | `Vec<f64>`（最大值索引） |
| `minmax` / `minmax_default` | `TA_MINMAX` | `(a, period)` | `MinMax { min, max }` |
| `minmax_index` / `minmax_index_default` | `TA_MINMAXINDEX` | `(a, period)` | `MinMaxIndex { min_idx, max_idx }` |

### 数学变换 / Math Transform — `adaq_talib::math_trans`

对单条序列做逐元素超越 / 取整变换，全部返回 `Vec<f64>`（等长；lookback 0）。

| 函数 | TA-Lib | 返回 |
| --- | --- | --- |
| `acos` / `acos_default` | `TA_ACOS` | `Vec<f64>` |
| `asin` / `asin_default` | `TA_ASIN` | `Vec<f64>` |
| `atan` / `atan_default` | `TA_ATAN` | `Vec<f64>` |
| `ceil` / `ceil_default` | `TA_CEIL` | `Vec<f64>` |
| `cos` / `cos_default` | `TA_COS` | `Vec<f64>` |
| `cosh` / `cosh_default` | `TA_COSH` | `Vec<f64>` |
| `exp` / `exp_default` | `TA_EXP` | `Vec<f64>` |
| `floor` / `floor_default` | `TA_FLOOR` | `Vec<f64>` |
| `ln` / `ln_default` | `TA_LN` | `Vec<f64>` |
| `log10` / `log10_default` | `TA_LOG10` | `Vec<f64>` |
| `sin` / `sin_default` | `TA_SIN` | `Vec<f64>` |
| `sinh` / `sinh_default` | `TA_SINH` | `Vec<f64>` |
| `sqrt` / `sqrt_default` | `TA_SQRT` | `Vec<f64>` |
| `tan` / `tan_default` | `TA_TAN` | `Vec<f64>` |
| `tanh` / `tanh_default` | `TA_TANH` | `Vec<f64>` |

### 模式识别 / Pattern Recognition — `adaq_talib::pattern`

全部 **61 个蜡烛形态**（TA-Lib *Pattern Recognition* 组）。每个函数入参
`(&[f64] open, &[f64] high, &[f64] low, &[f64] close)`，返回等长 `Vec<f64>` 整数信号：
`+100` 看多 / `0` 中性 / `−100` 看空；前导 `lookback` 个位置为 `0.0`（与 TA-Lib 整数输出约定一致，ADR 0007）。
仅实现默认 candle settings（ADR 0009）。

| 函数 | TA-Lib | 函数 | TA-Lib |
| --- | --- | --- | --- |
| `cdl_2crows` | `CDL2CROWS` | `cdl_identical3crows` | `CDLIDENTICAL3CROWS` |
| `cdl_3blackcrows` | `CDL3BLACKCROWS` | `cdl_inneck` | `CDLINNECK` |
| `cdl_3inside` | `CDL3INSIDE` | `cdl_invertedhammer` | `CDLINVERTEDHAMMER` |
| `cdl_3linestrike` | `CDL3LINESTRIKE` | `cdl_kicking` | `CDLKICKING` |
| `cdl_3outside` | `CDL3OUTSIDE` | `cdl_kickingbylength` | `CDLKICKINGBYLENGTH` |
| `cdl_3starsinsouth` | `CDL3STARSINSOUTH` | `cdl_ladderbottom` | `CDLLADDERBOTTOM` |
| `cdl_3whitesoldiers` | `CDL3WHITESOLDIERS` | `cdl_longleggeddoji` | `CDLLONGLEGGEDDOJI` |
| `cdl_abandonedbaby` | `CDLABANDONEDBABY` | `cdl_longline` | `CDLLONGLINE` |
| `cdl_advanceblock` | `CDLADVANCEBLOCK` | `cdl_marubozu` | `CDLMARUBOZU` |
| `cdl_belthold` | `CDLBELTHOLD` | `cdl_matchinglow` | `CDLMATCHINGLOW` |
| `cdl_breakaway` | `CDLBREAKAWAY` | `cdl_mathold` | `CDLMATHOLD` |
| `cdl_closingmarubozu` | `CDLCLOSINGMARUBOZU` | `cdl_morningdojistar` | `CDLMORNINGDOJISTAR` |
| `cdl_concealbabyswall` | `CDLCONCEALBABYSWALL` | `cdl_morningstar` | `CDLMORNINGSTAR` |
| `cdl_counterattack` | `CDLCOUNTERATTACK` | `cdl_onneck` | `CDLONNECK` |
| `cdl_darkcloudcover` | `CDLDARKCLOUDCOVER` | `cdl_piercing` | `CDLPIERCING` |
| `cdl_doji` | `CDLDOJI` | `cdl_rickshawman` | `CDLRICKSHAWMAN` |
| `cdl_dojistar` | `CDLDOJISTAR` | `cdl_risefall3methods` | `CDLRISEFALL3METHODS` |
| `cdl_dragonflydoji` | `CDLDRAGONFLYDOJI` | `cdl_separatinglines` | `CDLSEPARATINGLINES` |
| `cdl_engulfing` | `CDLENGULFING` | `cdl_shootingstar` | `CDLSHOOTINGSTAR` |
| `cdl_eveningdojistar` | `CDLEVENINGDOJISTAR` | `cdl_shortline` | `CDLSHORTLINE` |
| `cdl_eveningstar` | `CDLEVENINGSTAR` | `cdl_spinningtop` | `CDLSPINNINGTOP` |
| `cdl_gapsidesidewhite` | `CDLGAPSIDESIDEWHITE` | `cdl_stalledpattern` | `CDLSTALLEDPATTERN` |
| `cdl_gravestonedoji` | `CDLGRAVESTONEDOJI` | `cdl_sticksandwich` | `CDLSTICKSANDWICH` |
| `cdl_hammer` | `CDLHAMMER` | `cdl_takuri` | `CDLTAKURI` |
| `cdl_hangingman` | `CDLHANGINGMAN` | `cdl_tasukigap` | `CDLTASUKIGAP` |
| `cdl_harami` | `CDLHARAMI` | `cdl_thrusting` | `CDLTHRUSTING` |
| `cdl_haramicross` | `CDLHARAMICROSS` | `cdl_tristar` | `CDLTRISTAR` |
| `cdl_highwave` | `CDLHIGHWAVE` | `cdl_unique3river` | `CDLUNIQUE3RIVER` |
| `cdl_hikkake` | `CDLHIKKAKE` | `cdl_upsidegap2crows` | `CDLUPSIDEGAP2CROWS` |
| `cdl_hikkakemod` | `CDLHIKKAKEMOD` | `cdl_xsidegap3methods` | `CDLXSIDEGAP3METHODS` |
| `cdl_homingpigeon` | `CDLHOMINGPIGEON` | | |

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
cargo add adaq-talib
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

`src/utils.rs` 与 `src/core/` 仅作 `doc(hidden)` 内部实现细节（对齐、范围检查与共享滚动原语等），不属于公开 API，请勿在外部依赖。

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
重叠研究 / Overlap:    sma ema wma dema tema midpoint midprice bbands trima t3 ma mavp kama sar sarext accbands
动量 / Momentum:       rsi cmo trix mom macd cci willr bop ultosc adx aroon stoch mfi dx imi
波动率 / Volatility:   trange atr natr
成交量 / Volume:       ad adosc obv
价格变换 / Price:      avgprice medprice typprice wclprice avgdev
统计 / Statistic:      stddev var linear_reg linear_reg_angle linear_reg_intercept linear_reg_slope tsf beta correl
周期 / Cycle:          mama ht_trendline
```

> 注：交互式示例覆盖上述指标。其余已实现的指标（如 `roc`/`rocp`/`rocr`/`rocr100`、`apo`/`ppo`、
> `stoch_f`/`stoch_rsi`、`aroon_osc`、`plus_dm`/`minus_dm`/`plus_di`/`minus_di`、`adxr`、
> `macd_fix`/`macd_ext`，以及 `math_ops` 数学算子、`math_trans` 数学变换、`pattern` 全部 61 个蜡烛形态等）
> 均可直接在代码中调用，详见上方[功能总览](#已实现功能总览--implemented-functions)。
> / The interactive demo covers the indicators above. The remaining implemented functions
> (e.g. `roc`, `rocp`, `apo`, `ppo`, `stoch_f`, `stoch_rsi`, `aroon_osc`, the directional
> components, `adxr`, `macd_fix`, `macd_ext`, the math operators in `math_ops`, the math
> transforms in `math_trans`, and all 61 candlestick patterns in `pattern`, …) are callable
> directly in code — see the [function overview](#implemented-functions) above.

> 未知指标名会打印支持列表并以退出码 2 结束。/ An unknown name prints the supported list and exits with code 2.

---

## 验证与基准 / Verification & Benchmarks

### 正确性验证 / Correctness (1:1 黄金向量)

- 对照已入库的**黄金向量**（由 TA-Lib C 0.7.1 真实输出生成，位于 `tests/fixtures/`，**222 个黄金向量 fixture 文件**，覆盖全部 161 个函数面）运行 `cargo test`，普通用户**无需** Python 或 TA-Lib C 库。
- 容限策略：相对 `1e-8` + 绝对 `1e-10`（见 [ADR 0005](docs/adr/0005-error-tolerance.md)）。
- 全量测试：**326 项测试，0 失败**；`tools/reconcile.py` 确认 **161/161** 对外函数 1:1 对应 TA-Lib 0.7.1（exit 0）。涵盖全部 161 个指标的完整 1:1 验证与性能对照报告见 [`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md)。
- 黄金向量生成工具见 [`tools/gen_fixtures`](tools/)（需系统安装 TA-Lib C 库 + `TA-Lib` Python 绑定，仅供维护者重生成 fixture）。当前 fixture 状态与已知缺口见 [`tools/README.md`](tools/README.md)。

```bash
cargo test                 # 单元 + 黄金向量比对（无需 Python / C）
cargo test --doc           # 仅运行文档示例
```

### 性能对照 / Performance (Rust vs TA-Lib C)

全部 **161 / 161** 个指标均与原生 TA-Lib C 0.7.1 逐项对照基准（双轨，见 [ADR 0004](docs/adr/0004-benchmark-dual-track.md)；C 侧在 `--features bench-c` 下 FFI 链接系统 TA-Lib C）。环境：Apple Silicon aarch64，每个指标 **N = 100,000** 个元素；`ns/elem = elapsed / ITERS / N`；`Rust/C = Rust_ns/elem ÷ C_ns/elem`。最终数值取 **5 次运行的中位数**以抑制逐指标噪声（约 20–40%）。判定：比值 < 0.8 → 更快，0.8–1.2 → 持平，> 1.2 → 更慢。

**总览：** 相比原生 C，**85 个更快**、**60 个持平**、**16 个更慢**；几何均值 **Rust/C = 0.786×**（adaq-talib 平均约为 C 的 **1.27× 快**；在 0.1.3 优化之前为 1.50× 慢）。其中 **145 / 161 个指标与 C 持平或更优**（Rust/C ≤ 1.2）；仅 16 个仍慢于 C，且均为孤立个案。

**可选 `parallel` 特性：** `parallel` 特性（默认关闭）对 5 个 A 类窗口函数（`midpoint`、`minmax`、`minmax_index`、`willr`、`stoch_f`）采用重叠播种并行分块，将其移出“更慢”桶。在 `--features parallel` 下，合计变为 **88 更快 / 63 持平 / 10 更慢**（几何均值 **Rust/C = 0.734×**，约为 C 的 1.36× 快）；默认（串行）构建仍为 85/60/16（0.786×）。对其余 156 个函数该特性为 no-op。详见已落地优化表（P3-2b）与 `docs/validation-and-performance-report.md` §3.5。

| TA-Lib 分组 | 指标数 | 更快 (<0.8) | 持平 (0.8–1.2) | 更慢 (>1.2) | 几何均值 Rust/C |
|---|---:|---:|---:|---:|---:|
| 周期 / 希尔伯特变换 | 5 | 2 | 2 | 1 | 0.980× |
| 数学算子 | 11 | 7 | 2 | 2 | 0.805× |
| 数学变换 | 15 | 4 | 11 | 0 | 0.858× |
| 动量 | 31 | 8 | 20 | 3 | 0.852× |
| 重叠研究 | 18 | 6 | 10 | 2 | 0.842× |
| 模式识别 | 61 | 43 | 13 | 5 | 0.677× |
| 价格变换 | 5 | 5 | 0 | 0 | 0.599× |
| 统计函数 | 9 | 7 | 1 | 1 | 0.548× |
| 波动率 | 3 | 2 | 0 | 1 | 0.841× |
| 成交量 | 3 | 1 | 1 | 1 | 0.994× |
| **合计** | **161** | **85** | **60** | **16** | **0.786×** |

adaq-talib 现已在 **全部 10 个分组上平均快于 C**（每个分组的几何均值 Rust/C < 1）—— 周期、数学算子、数学变换、动量、重叠研究、模式识别、价格变换、统计函数、波动率与成交量；**没有任何分组在平均意义上慢于 C**。最显著的变化是模式识别：经 0.1.3 对全部 61 个蜡烛函数做内联累加器推广后，该组由最慢（几何均值 2.98× 慢）一跃成为最快之一（0.677×）。仍慢于 C 的 **16** 个指标为孤立个案，属真实的单线程递推 / 双极值下限：`midpoint`、`minmax`、`minmax_index`、`mfi`、`willr`、`stoch_f`、`correl`、`adosc`、`trange`、`ht_phasor`、`ht_trendline`，以及形态蜡烛判定分支 `cdl_engulfing`/`cdl_separatinglines`/`cdl_harami`/`cdl_longline`/`cdl_shortline`。EMA 家族的缺口（EMA/KAMA/APO/PPO/T3/TRIX/ULTOSC/ADX/ADXR/DX）已被 P3-6 FMA 收缩阶段补齐（见下方已落地优化表）。完整逐指标表 —— 全部 161 个，含 Rust/C 比值、状态与实时 TA-Lib 一致性校验和 —— 见 [`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md)。相关注意事项（如 `stoch_rsi` 仅返回 `fastk` 线，故其基准校验和不同 —— 属基准观测假象，非正确性缺口）详见该报告。

### 已落地的性能优化 / Optimizations applied

所有下述优化均**零偏差** —— 逐项对照 TA-Lib 0.7.1 黄金向量验证（完整逐指标说明见 [`benches/BASELINE.md`](benches/BASELINE.md)）。`ns/elem` 采集于 Apple Silicon aarch64，`N = 1_000_000`，`PERIOD = 20`，`ITERS = 20`（点测，±5% 波动）。

| 阶段 | 函数 | 技术 | 结果 (Rust/C 或 ns/elem) | 相对先前 |
|------|------|------|--------------------------:|-----------|
| P3-1 (0.1.3) | 模式识别（全部 61 个 CDL） | 内联 `CandleAvg` 运行和 / 拖尾和累加器（`tools/opt_pattern.py`，零偏差） | 几何均值 **2.98× → 0.677×**；43 快 / 13 持平 / 5 慢 | 最大单项收益 |
| P3-2 (0.1.3) | `min` / `max` / `min_index` / `max_index` | 环形缓冲 `MonoQueue`（掩码索引，无边界检查）替换 `VecDeque` | 每极值 3.447 → 2.347 ns/elem；`min` 1.17→0.76、`max` 1.57→0.99、`min_index` 1.14→0.77、`max_index` 1.54→1.01 | 每极值约快 32% |
| P3-3 (0.1.3) | `ht_dcperiod` | 循环-IIR 快路径跳过未用的 `compute_dc_phase` 正弦/余弦窗口 | 3.589× → 1.191×（已持平） | — |
| P3-4 (0.1.3) | `ht_dcphase` / `ht_sine` / `ht_trendmode` | 正弦/余弦角度加法递推（`sin(θ+w)`、`cos(θ+w)`） | 1.216→0.786 / 0.840→0.687 / 1.432→1.122 | — |
| P3-5 (0.1.3) | `mfi` | 单遍滑动窗口融合（两个环形缓冲运行和） | 2.563× → 1.406×（仍慢 —— 逐 bar 除法主导） | 约 1.8× 接近 C |
| P3-6 (0.1.3) | `ema` / `kama` / `apo` / `ppo` / `t3` / `adosc`（递推点） | 在每一处递推显式使用 `.mul_add()` FMA（与 GCC `-ffp-contract=fast` 等价） | `ema` 1.488→0.977、`kama` 1.484→1.069、`apo` 1.529→1.085、`ppo` 1.425→1.077、`t3` 1.325→0.999（均达持平）；传递性使 `trix`/`ultosc` 更快、`adx`/`adxr`/`dx` 持平 | 补齐 EMA 家族缺口 |
| P3-2b (0.1.3) | `midpoint` / `minmax` / `minmax_index` / `willr` / `stoch_f` | 重叠播种并行分块（`std::thread::scope` + `available_parallelism`，零依赖，默认关闭 `parallel` 特性） | `midpoint` 1.620→0.901、`minmax` 1.523→0.844、`minmax_index` 1.434→0.915（均达持平）；`willr` 1.455→0.748、`stoch_f` 1.228→0.579（更快）；合计 85/60/16 → 88/63/10，几何均值 0.786×→0.734× | 将 5 个可播种的 A 类下限移出“更慢” |
| P2-1 | `dema` / `tema` / `t3` | 单遍嵌套 EMA 融合核（`core::nested_ema_with_output`） | 3.63 / 3.46 / 3.76 ns/elem | ~2× / ~3× / ~6×（相对朴素） |
| P2-2 | `midpoint` / `midprice` | 单调队列 `core::rolling_extreme` O(n) | 6.88 / 7.30 | ~3× / ~3× |
| P2-3 | `wma` | O(n) 滑动递推（`W[i] = W[i-1] + period·x[i] − sw[i-1]`） | 2.11 | ~4.7× |
| P2-4 | `bbands`（SMA 中轨） | 单遍 `rolling_mean_var` 融合 | 3.02 | ~1.5–1.6× |
| P2-5 | `linear_reg` 家族 / `correl` | O(n) 滑动求和 / 交叉积 | 2.33 / 4.81 | ~20×（渐近） |
| P2-5 | `willr` / `stoch` / `stoch_f` | 复用单调极值队列 O(n) | 7.90 / 10.99 | ~20×（渐近） |
| P1② | `minmax` | 复用单遍 `core::rolling_minmax`（收敛；性能中性） | 6.76 | ≈（仅精度收益） |
| P1③ | `max_index` / `min_index` / `minmax_index` | 单遍 `core::rolling_extreme_index` O(n) | 3.43 / 3.31 / 6.79 | ~1.9×（索引） |

† `midpoint` / `midprice`（P2-2）与 `max_index` / `min_index` / `minmax_index`（P1③）现已运行在 0.1.3 引入的同一环形缓冲 `MonoQueue` 上（P3-2）。

每个优化都保持完整 `cargo test` 套件全绿（326/326），且每个被重构的函数仍在其容限内复现 TA-Lib 0.7.1 黄金向量（见 [ADR 0005](docs/adr/0005-error-tolerance.md)）。完整的 QA 报告（方法论、残差缺口、Python 绑定参考数值）见 [`docs/perf-verify-report.md`](docs/perf-verify-report.md)。

### 性能基准（如何运行）/ Benchmarks (how to run)

```bash
# 1) Rust 侧（默认，零依赖）：std::time 计时，harness = false
cargo bench --bench sma_bench

# 2) 原生 C 对照（可选 feature）：FFI 链接系统 TA-Lib C 库
cargo bench --bench sma_bench --features bench-c

# 3) 全部 161 个指标对照原生 C（自动生成套件）：
cargo bench --bench all161_bench
cargo bench --bench all161_bench --features bench-c   # 含 C 参考口径
cargo bench --bench all161_bench --features bench-c,parallel   # 并行重叠播种阶段
```

> 第 2 种需系统已安装 TA-Lib C 库（`brew install ta-lib` / 源码编译）；`build.rs` 仅在 `bench-c` 下链接，未启用时构建不受影响。报告须明确区分两种口径。

---

## 已知问题与非推荐特性 / Known Issues & Deprecations

### 已知问题 / Known issues
- **16 个指标仍慢于原生 TA-Lib C** —— 全部为真实的单线程递推 / 双极值下限，非正确性缺口。最典型的结构性个案是 `MIDPOINT`（约 1.62×），一种数据依赖的单调结构，代价约为 C 单次 MINMAX 扫描的 2×；`minmax`/`minmax_index`（约 1.52×/1.43×）承担同样的双队列代价。严格递推的 `ht_phasor`/`ht_trendline`（约 1.24×/1.27×）、滑动窗口 `mfi`/`willr`/`stoch_f`/`adosc`/`correl`（约 1.23–1.55×）、`trange`（1.22×），以及形态蜡烛判定分支 `cdl_engulfing`/`cdl_separatinglines`/`cdl_harami`/`cdl_longline`/`cdl_shortline`（约 1.30–2.00×）组成完整名单（完整列表见性能报告 §4）。原本落后的 EMA 家族（`ema`/`kama`/`apo`/`ppo`/`t3`/`trix`/`ultosc`/`adx`/`adxr`/`dx`）已被 P3-6 FMA 收缩阶段补齐至持平 / 更快。规划中的 P3 SIMD 阶段对这些递推下限为已记录的 **NO-GO**（见 [ADR 0010](docs/adr/0010-performance-strategy.md)）—— 单线程微优化已触顶；>2× 的路径是并行化，受 `NEXT-ACTIONS-perf.md` 门槛约束。这是已知且已接受的权衡，**并非缺陷**。
- **`linear_reg` / `correl` / `willr` / `stoch` 未接原生 C 对照** —— 其 Rust 侧数值即权威参考。若要接 C 对照需引入 `unsafe` 与系统 TA-Lib C 库，违背零-FFI 设计；因此以 Rust 结果为准。
- **模式识别仅采用 TA-Lib 默认 candle settings**（见 [ADR 0009](docs/adr/0009-candle-settings-default-only.md)），不暴露配置 API。针对 TA-Lib 0.7.1 **无任何功能性覆盖缺口** —— 全部 61 个蜡烛形态均已实现。
- **`aroon` / `aroon_osc` 输出顺序** —— adaq-talib 遵循权威 TA-Lib C 0.7.1 的 `outAroonUp` / `outAroonDown` 顺序（即权威黄金向量）。若与 `talib` Python 绑定（0.7.1）交叉核对，需注意该构建 historically 将二者互换；见 [ADR 0003](docs/adr/0003-verification-golden-fixtures.md)。

### 依赖 / Dependencies
- **无运行时依赖。** 发布的库 `[dependencies]` 始终为空。近期改动仅新增*开发期*基准（`benches/`）、发布工作流与内部 `core` 原语 —— 未引入任何外部 crate。

### 非推荐 / 废弃特性 / Deprecated features
- **无。** 本版本未引入任何废弃特性，也未删减任何已发布能力（见 [ADR 0002](docs/adr/0002-release-scope-milestones.md)）。

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

采用里程碑式发布（见 [ADR 0002](docs/adr/0002-release-scope-milestones.md)）。**本版本已交付完整的 TA-Lib 0.7.1 公开函数面 —— 10 大类、共 161 个函数，且不删减任何已发布能力。**

- ✅ **0.1.3（当前）：161 / 161 函数，平均快于 C** —— 重叠研究（18）、动量（31）、波动率（3）、成交量（3）、价格变换（5）、统计（9）、周期 / 希尔伯特变换（7）、数学算子（11）、数学变换（15）、模式识别（61 个蜡烛形态）。每个函数均逐项比照 TA-Lib 0.7.1 黄金向量验证（`cargo test` → 326/326 全绿，`reconcile.py` → 161/161），基于 **222 个黄金向量 fixture**，并通过全量 161 基准 + 验证套件（[`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md)）确认完整覆盖；经 0.1.3 优化后 adaq-talib 平均已**约为 C 的 1.27× 快**（几何均值 Rust/C = 0.786×；85 更快 / 60 持平 / 16 更慢；启用可选 `parallel` 特性后进一步为 88/63/10，0.734×）—— 见[验证与基准](#验证与基准--verification--benchmarks)。
- 🔜 **后续工作（1.0 之后）**：可选的 candle-settings 变体（[ADR 0009](docs/adr/0009-candle-settings-default-only.md)）、为新优化指标（LINREG/CORREL/WILLR/STOCH）接 `bench-c` 对照、以及文档 / CI 润色。**针对 TA-Lib 0.7.1 已无任何功能性覆盖缺口。**

完成上述后，adaq-talib 即与 TA-Lib 0.7.1 等价全量覆盖。

---

## 变更日志 / Changelog

### 0.1.3
- **模式识别性能推广**：将 `cdl_hammer` 的内联运行和累加器模板推广到**全部 61 个蜡烛函数**（零偏差 transformer `tools/opt_pattern.py`）；把逐函数的 `CandleAvg::new`+`value`+`advance` 替换为内联 `sum_*`/`trail_*`/`cur_*`/`val_*` 累加器（跳过无 `CandleAvg` 的函数，如 `cdl_engulfing`/`cdl_3outside`/`cdl_hikkake`/`cdl_tristar`）。模式识别几何均值 **Rust/C 由 2.98× → 0.677×**（43 快 / 13 持平 / 5 慢，原为 1/3/57）—— 本次发布的最大单项收益。
- **P2 算法优化（零偏差，0 回退）**：以环形缓冲 `MonoQueue` 替换 `VecDeque` 滚动极值（`min`/`max`/`min_index`/`max_index`，每极值约快 32%）；为 `ht_dcperiod` 增加跳过未用 `compute_dc_phase` 正弦/余弦窗口的循环-IIR 快路径（3.59× → 1.19×，已持平）；在 `compute_dc_phase` 中改用正弦/余弦角度加法递推（`ht_dcphase`/`ht_sine`/`ht_trendmode`）；并将 `mfi` 改写为单遍滑动窗口融合（2.56× → 1.41×）。合计 **82 快 / 54 持平 / 25 慢，几何均值 Rust/C = 0.792×** —— adaq-talib 平均现为 C 的约 1.26× 快（此前为 1.50× 慢）。
- **P3-2b 并行重叠播种（零偏差，0 回退）**：新增默认关闭的 `parallel` 特性，对 5 个可重叠播种的 A 类窗口函数（`midpoint`/`minmax`/`minmax_index`/`willr`/`stoch_f`）采用 `std::thread::scope` + `available_parallelism` 的重叠播种并行分块（纯 `std`，零外部依赖）；每块以 `period-1`（或 `stoch_f` 的 `fk+fd-2`）个前导元素重叠，复用与串行逐字节一致的核，输出 1:1。合计由 **85 快 / 60 持平 / 16 慢（0.786×）** 变为 **88 快 / 63 持平 / 10 慢（0.734×，约 1.36× 快于 C）**；对其余 156 个函数该特性为 no-op。详见 [`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md) §3.5。
- **报告与工具**：更新 [`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md)（新分组 / 逐指标表，三次取中位方法学）与交互式 `docs/benchmarks/adaq-vs-talib-161.html`；新增 `benches/extreme_ab.rs`、`tools/opt_pattern.py` 与 `docs/research/perf-161-analysis.md`。
- **发布**：版本号提升至 `0.1.3`。无新增公开 API、无弃用、无依赖变更（[ADR 0002](docs/adr/0002-release-scope-milestones.md)）。

### 0.1.2
- **全量 161 基准与验证套件**：新增 `benches/all161_bench.rs`（由 `tools/bench/gen_all161.py` 自动生成），对**全部 161** 个指标与原生 TA-Lib C 0.7.1 逐项基准对照，并附带实时数值一致性校验和；配套 `benches/poc_bench.rs` 为概念验证脚手架。统一报告 [`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md)、交互式 `docs/benchmarks/adaq-vs-talib-161.html` 与 `all161_results.csv` 由 `tools/bench/gen_report.py` 生成（双轨方法论见 [ADR 0004](docs/adr/0004-benchmark-dual-track.md)）。
- **黄金向量覆盖扩大**：**222 个黄金向量 fixture 文件**（原 159 个）——补全了完整的模式识别 fixture 集与 `macd_ext` / `macd_fix` fixture。全量测试现为 **326 项测试，0 失败**（原 308），`tools/reconcile.py` 确认 **161/161**。
- **文档完整性**：逐函数表现已列出全部 161 个函数。`accbands`（重叠研究）、`dx` / `imi`（动量）与 `avgdev`（价格变换）此前已实现并计入 161 总数，但被遗漏在明细表之外 —— 现均已补入文档。
- **发布**：版本号提升至 `0.1.2`。除上述外无新增公开 API；无弃用、无依赖变更（[ADR 0002](docs/adr/0002-release-scope-milestones.md)）。

### 0.1.1
- **数学算子 —— O(n) 极值索引函数**：`max_index` / `min_index` / `minmax_index` 现采用单遍单调队列（`core::rolling_extreme_index`），替换原先 O(n·period) 的嵌套扫描 —— 提速约 1.9×，且与 TA-Lib 0.7.1 仍逐项 1:1（见 [ADR 0005](docs/adr/0005-error-tolerance.md)）。新增 `benches/index_bench.rs` 与 `benches/minmax_bench.rs`。
- **`minmax` 收敛**：`math_ops::minmax` 现复用单遍 `core::rolling_minmax` 核（与 `midpoint` 同源），消除重复的极值逻辑。性能中性，精度不变。
- **P2 全阶段性能优化（1:1 验证）**：`dema` / `tema` / `t3` 嵌套 EMA 融合（P2-1）；`midpoint` / `midprice` 单调队列（P2-2）；`wma` O(n) 滑动递推（P2-3）；`bbands` 中轨单遍融合（P2-4）；`linear_reg` 家族 / `correl` / `willr` / `stoch` 滑动 O(n)（P2-5）。详见 [`benches/BASELINE.md`](benches/BASELINE.md)。
- **发布工具与文档**：新增 `.github/workflows/release.yml`（发布自动化）与 CI；修复 doc-comment 与发布 `exclude`；版本号提升至 `0.1.1`。
- **模式识别与数学运算模块**：全部 61 个蜡烛形态与完整的 `math_ops` / `math_trans` 函数面均已实现，并补齐黄金向量 fixture（P4 里程碑 —— 161/161 函数）。

### 0.1.0
- 首个公开里程碑：完整的 TA-Lib 0.7.1 公开函数面 —— 10 大类共 161 个函数，并以零偏差黄金向量验证。
