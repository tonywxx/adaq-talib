# adaq-talib

**AdaQ-TAlib** — a pure-Rust, zero-FFI, dependency-free reimplementation of the
[TA-Lib](https://ta-lib.org) 0.7.1 technical-analysis indicators.

> 纯 Rust、Zero-FFI、零依赖的 [TA-Lib](https://ta-lib.org) 0.7.1 技术指标复刻库。
>
> 中文文档: [README.zh-CN.md](./README.zh-CN.md)

---

## Contents

- [Features](#features)
- [Implemented Functions](#implemented-functions)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
- [Interactive Demo](#interactive-demo)
- [Verification & Benchmarks](#verification--benchmarks)
- [Known Issues & Deprecations](#known-issues--deprecations)
- [Documentation](#documentation)
- [License](#license)
- [Roadmap](#roadmap)
- [Changelog](#changelog)

---

## Features

- **Zero-deviation**: every function reproduces the numerical output of TA-Lib 0.7.1
  (within the float tolerance in [ADR 0005](docs/adr/0005-error-tolerance.md)). The
  authoritative reference is the golden vectors generated from TA-Lib C 0.7.1
  ([ADR 0003](docs/adr/0003-verification-golden-fixtures.md)).
- **Zero-FFI / No-Dependencies**: the published crate calls no C ABI and its
  `Cargo.toml` `[dependencies]` is empty — every algorithm is hand-written in Rust.
- **Idiomatic Rust API (Model B)**: slice inputs (`&[f64]`), `Result<_, TaError>` outputs,
  multi-output functions return structs; the leading unstable period is filled with
  `f64::NAN` and the result is equal-length ([ADR 0001](docs/adr/0001-api-fidelity-model.md)
  / [ADR 0007](docs/adr/0007-unstable-period.md)).
- **Performance-oriented**: optimizations at the level of memory layout, loop branching,
  and array arithmetic. The cycle indicators (MAMA / HT_TRENDLINE) are ported line-for-line
  from the official C sources for bit-close fidelity.
- **Verify without system deps**: `cargo test` compares against in-repo golden vectors, so
  running the tests needs **no** Python or TA-Lib C library.

---

## Implemented Functions

**adaq-talib now implements the full TA-Lib 0.7.1 public surface — all 161 functions across
10 categories — with zero deviation** (verified 1:1 against golden vectors, see
[Verification & Benchmarks](#verification--benchmarks)). The tables below list every public
function, its TA-Lib counterpart, default parameters, and return shape.

| Category | Module | Count | TA-Lib group |
| --- | --- | ---: | --- |
| Overlap Studies | `overlap` | 18 | Overlap Studies |
| Momentum Indicators | `momentum` | 31 | Momentum Indicators |
| Volatility Indicators | `volatility` | 3 | Volatility Indicators |
| Volume Indicators | `volume` | 3 | Volume Indicators |
| Price Transform | `price_transform` | 5 | Price Transform |
| Statistic Functions | `stat` | 9 | Statistic Functions |
| Cycle (Hilbert Transform) | `cycle` | 7\* | Cycle Indicators (5) + Overlap (2)† |
| Math Operators | `math_ops` | 11 | Math Operators |
| Math Transform | `math_trans` | 15 | Math Transform |
| Pattern Recognition | `pattern` | 61 | Pattern Recognition |
| **Total** | | **161** | **161** |

\* The `cycle` module holds 7 functions; TA-Lib classifies 5 as *Cycle Indicators*
(`HT_DCPERIOD`/`HT_DCPHASE`/`HT_PHASOR`/`HT_SINE`/`HT_TRENDMODE`) and 2 as *Overlap Studies*
(`MAMA`/`HT_TRENDLINE`). † Grouping follows TA-Lib's authoritative `info['group']`.

> Convention: every function ships both an explicit-parameter version and a `_default`
> convenience version that uses TA-Lib's default parameters (see
> [`src/core/defaults.rs`](src/core/defaults.rs)).

### Overlap Studies — `adaq_talib::overlap`

| Function | TA-Lib | Defaults | Returns |
| --- | --- | --- | --- |
| `sma` / `sma_default` | `TA_SMA` | period = 30 | `Vec<f64>` |
| `ema` / `ema_default` | `TA_EMA` | period = 30 | `Vec<f64>` |
| `wma` / `wma_default` | `TA_WMA` | period = 30 | `Vec<f64>` |
| `dema` / `dema_default` | `TA_DEMA` | period = 30 | `Vec<f64>` |
| `tema` / `tema_default` | `TA_TEMA` | period = 30 | `Vec<f64>` |
| `midpoint` / `midpoint_default` | `TA_MIDPOINT` | period = 30 | `Vec<f64>` |
| `midprice` / `midprice_default` | `TA_MIDPRICE` | period = 30 | `Vec<f64>` |
| `bbands` / `bbands_default` | `TA_BBANDS` | period = 20, nb_dev = 2.0/2.0, SMA middle | `Bbands { upper, middle, lower }` |
| `trima` / `trima_default` | `TA_TRIMA` | period = 30 | `Vec<f64>` |
| `t3` / `t3_default` | `TA_T3` | period = 5, vfactor = 0.7 | `Vec<f64>` |
| `ma` / `ma_default` | `TA_MA` | period = 30, `MaType::Sma` | `Vec<f64>` (dispatched by `MaType`) |
| `mavp` / `mavp_default` | `TA_MAVP` | min = 2 / max = 30, SMA | `Vec<f64>` (variable period) |
| `kama` / `kama_default` | `TA_KAMA` | period = 30 | `Vec<f64>` |
| `sar` / `sar_default` | `TA_SAR` | accel = 0.02, max = 0.2 | `Vec<f64>` |
| `sarext` / `sarext_default` | `TA_SAREXT` | long/short accel 0.02/0.02/0.2 | `Vec<f64>` (short side negative) |
| `accbands` / `accbands_default` | `TA_ACCBANDS` | period = 20 | `AccBands { upper, middle, lower }` |
| `MaType` | `TA_MAType` | `Sma/Ema/Wma/Dema/Tema/Trima/Kama/Mama` | enum (for `ma`/`bbands`/`mavp`) |

### Momentum Indicators — `adaq_talib::momentum`

| Function | TA-Lib | Defaults | Returns |
| --- | --- | --- | --- |
| `mom` / `mom_default` | `TA_MOM` | period = 10 | `Vec<f64>` |
| `roc` / `roc_default` | `TA_ROC` | period = 10 | `Vec<f64>` |
| `rocp` / `rocp_default` | `TA_ROCP` | period = 10 | `Vec<f64>` |
| `rocr` / `rocr_default` | `TA_ROCR` | period = 10 | `Vec<f64>` |
| `rocr100` / `rocr100_default` | `TA_ROCR100` | period = 10 | `Vec<f64>` |
| `rsi` / `rsi_default` | `TA_RSI` | period = 14 | `Vec<f64>` |
| `macd` / `macd_default` | `TA_MACD` | fast = 12, slow = 26, signal = 9 | `Macd { macd, signal, hist }` |
| `macd_fix` / `macd_fix_default` | `TA_MACDFIX` | fast = 12, slow = 26, signal = 9 | `Macd` |
| `macd_ext` / `macd_ext_default` | `TA_MACDEXT` | fast = 12, slow = 26, signal = 9 | `Macd` (all-EMA default) |
| `apo` / `apo_default` | `TA_APO` | fast = 12, slow = 26 | `Vec<f64>` |
| `ppo` / `ppo_default` | `TA_PPO` | fast = 12, slow = 26 | `Vec<f64>` |
| `cmo` / `cmo_default` | `TA_CMO` | period = 14 | `Vec<f64>` |
| `imi` / `imi_default` | `TA_IMI` | period = 14 | `Vec<f64>` (open/close) |
| `cci` / `cci_default` | `TA_CCI` | period = 20 | `Vec<f64>` |
| `mfi` / `mfi_default` | `TA_MFI` | period = 14 | `Vec<f64>` (needs volume) |
| `willr` / `willr_default` | `TA_WILLR` | period = 14 | `Vec<f64>` |
| `bop` | `TA_BOP` | — | `Vec<f64>` (lookback 0) |
| `ultosc` / `ultosc_default` | `TA_ULTOSC` | 7 / 14 / 28 | `Vec<f64>` |
| `plus_dm` / `plus_dm_default` | `TA_PLUS_DM` | period = 14 | `Vec<f64>` |
| `minus_dm` / `minus_dm_default` | `TA_MINUS_DM` | period = 14 | `Vec<f64>` |
| `plus_di` / `plus_di_default` | `TA_PLUS_DI` | period = 14 | `Vec<f64>` |
| `minus_di` / `minus_di_default` | `TA_MINUS_DI` | period = 14 | `Vec<f64>` |
| `adx` / `adx_default` | `TA_ADX` | period = 14 | `Vec<f64>` |
| `adxr` / `adxr_default` | `TA_ADXR` | period = 14 | `Vec<f64>` |
| `dx` / `dx_default` | `TA_DX` | period = 14 | `Vec<f64>` (OHLC) |
| `aroon` / `aroon_default` | `TA_AROON` | period = 14 | `Aroon { up, down }` |
| `aroon_osc` / `aroon_osc_default` | `TA_AROONOSC` | period = 14 | `Vec<f64>` |
| `stoch` / `stoch_default` | `TA_STOCH` | fastK = 5, slowK = 3, slowD = 3 | `Stoch { slow_k, slow_d }` |
| `stoch_f` / `stoch_f_default` | `TA_STOCHF` | fastK = 5, fastD = 3 | `StochF { fast_k, fast_d }` |
| `stoch_rsi` / `stoch_rsi_default` | `TA_STOCHRSI` | rsi = 14, period = 14 | `Vec<f64>` |
| `trix` / `trix_default` | `TA_TRIX` | period = 30 | `Vec<f64>` |

### Volatility Indicators — `adaq_talib::volatility`

| Function | TA-Lib | Defaults | Returns |
| --- | --- | --- | --- |
| `trange` | `TA_TRANGE` | — | `Vec<f64>` (lookback 0) |
| `atr` / `atr_default` | `TA_ATR` | period = 14 | `Vec<f64>` |
| `natr` / `natr_default` | `TA_NATR` | period = 14 | `Vec<f64>` |

### Volume Indicators — `adaq_talib::volume`

| Function | TA-Lib | Defaults | Returns |
| --- | --- | --- | --- |
| `ad` | `TA_AD` | — | `Vec<f64>` (cumulative, lookback 0) |
| `adosc` / `adosc_default` | `TA_ADOSC` | fast = 3, slow = 10 | `Vec<f64>` |
| `obv` | `TA_OBV` | — | `Vec<f64>` (cumulative, lookback 0) |

### Price Transform — `adaq_talib::price_transform`

| Function | TA-Lib | Defaults | Returns |
| --- | --- | --- | --- |
| `avgdev` / `avgdev_default` | `TA_AVGDEV` | period = 14 | `Vec<f64>` |
| `avgprice` | `TA_AVGPRICE` | — | `Vec<f64>` ((H+L+C+O)/4) |
| `medprice` | `TA_MEDPRICE` | — | `Vec<f64>` ((H+L)/2) |
| `typprice` | `TA_TYPPRICE` | — | `Vec<f64>` ((H+L+C)/3) |
| `wclprice` | `TA_WCLPRICE` | — | `Vec<f64>` ((H+L+2C)/4) |

### Statistic Functions — `adaq_talib::stat`

| Function | TA-Lib | Defaults | Returns |
| --- | --- | --- | --- |
| `stddev` / `stddev_default` | `TA_STDDEV` | period = 5, nb_dev = 1.0 | `Vec<f64>` |
| `var` / `var_default` | `TA_VAR` | period = 5, nb_dev (ignored) | `Vec<f64>` |
| `linear_reg` / `linear_reg_default` | `TA_LINEARREG` | period = 14 | `Vec<f64>` |
| `linear_reg_angle` / `linear_reg_angle_default` | `TA_LINEARREG_ANGLE` | period = 14 | `Vec<f64>` (angle, degrees) |
| `linear_reg_intercept` / `linear_reg_intercept_default` | `TA_LINEARREG_INTERCEPT` | period = 14 | `Vec<f64>` |
| `linear_reg_slope` / `linear_reg_slope_default` | `TA_LINEARREG_SLOPE` | period = 14 | `Vec<f64>` |
| `tsf` / `tsf_default` | `TA_TSF` | period = 14 | `Vec<f64>` |
| `beta` / `beta_default` | `TA_BETA` | period = 5 | `Vec<f64>` |
| `correl` / `correl_default` | `TA_CORREL` | period = 5 | `Vec<f64>` |

### Cycle (Hilbert Transform) — `adaq_talib::cycle`

| Function | TA-Lib | Defaults | Returns |
| --- | --- | --- | --- |
| `mama` / `mama_default` | `TA_MAMA` | fast = 0.5, slow = 0.05 | `Mama { mama, fama }` |
| `ht_trendline` / `ht_trendline_default` | `TA_HT_TRENDLINE` | — | `Vec<f64>` (lookback 63) |
| `ht_dcperiod` / `ht_dcperiod_default` | `TA_HT_DCPERIOD` | — | `Vec<f64>` (dominant cycle period) |
| `ht_dcphase` / `ht_dcphase_default` | `TA_HT_DCPHASE` | — | `Vec<f64>` (dominant cycle phase) |
| `ht_phasor` / `ht_phasor_default` | `TA_HT_PHASOR` | — | `Phasor { in_phase, quadrature }` |
| `ht_sine` / `ht_sine_default` | `TA_HT_SINE` | — | `HtSine { sine, lead_sine }` |
| `ht_trendmode` / `ht_trendmode_default` | `TA_HT_TRENDMODE` | — | `Vec<f64>` (0/1 trend mode) |

### Math Operators — `adaq_talib::math_ops`

Element-wise / array operators over one or two equal-length series. All return `Vec<f64>`
(equal-length; lookback 0). Binary operators take `(&[f64], &[f64])`; `maxindex`/`minindex`/
`minmax`/`minmaxindex` reduce a windowed series.

| Function | TA-Lib | Signature | Returns |
| --- | --- | --- | --- |
| `add` / `add_default` | `TA_ADD` | `(a, b)` | `Vec<f64>` |
| `sub` / `sub_default` | `TA_SUB` | `(a, b)` | `Vec<f64>` |
| `mult` / `mult_default` | `TA_MULT` | `(a, b)` | `Vec<f64>` |
| `div` / `div_default` | `TA_DIV` | `(a, b)` | `Vec<f64>` |
| `sum` / `sum_default` | `TA_SUM` | `(a, period)` | `Vec<f64>` |
| `min` / `min_default` | `TA_MIN` | `(a, period)` | `Vec<f64>` |
| `max` / `max_default` | `TA_MAX` | `(a, period)` | `Vec<f64>` |
| `min_index` / `min_index_default` | `TA_MININDEX` | `(a, period)` | `Vec<f64>` (index of min) |
| `max_index` / `max_index_default` | `TA_MAXINDEX` | `(a, period)` | `Vec<f64>` (index of max) |
| `minmax` / `minmax_default` | `TA_MINMAX` | `(a, period)` | `MinMax { min, max }` |
| `minmax_index` / `minmax_index_default` | `TA_MINMAXINDEX` | `(a, period)` | `MinMaxIndex { min_idx, max_idx }` |

### Math Transform — `adaq_talib::math_trans`

Element-wise transcendental / rounding transforms over a single series. All return `Vec<f64>`
(equal-length; lookback 0).

| Function | TA-Lib | Returns |
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

### Pattern Recognition — `adaq_talib::pattern`

All **61 candlestick patterns** (TA-Lib *Pattern Recognition* group). Each takes
`(&[f64] open, &[f64] high, &[f64] low, &[f64] close)` and returns an equal-length
`Vec<f64>` of integer signals: `+100` bullish / `0` neutral / `−100` bearish; the leading
`lookback` positions are `0.0` (consistent with TA-Lib's integer-output convention, ADR 0007).
Only the default candle settings are implemented (ADR 0009).

| Function | TA-Lib | Function | TA-Lib |
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

### Error Type

- `TaError` (`adaq_talib::TaError`) — the public error enum, a semantic mapping of TA-Lib's
  `TA_RetCode`: `BadParam` / `OutOfRange` / `LibNotInitialized` / `OutOfMemory` /
  `InternalError` (see [ADR 0006](docs/adr/0006-type-error-model.md)).

---

## Quick Start

```rust
use adaq_talib::overlap::sma;

let prices = [1.0, 2.0, 3.0, 4.0, 5.0];
let out = sma(&prices, 3).unwrap();
// `out` has the same length as `prices`; the first 2 positions are NaN, the rest are window means.
assert!(out[0].is_nan());
assert!((out[2] - 2.0).abs() < 1e-9);
```

Multi-output examples (Bollinger Bands, MACD):

```rust
use adaq_talib::overlap::bbands_default;
use adaq_talib::momentum::macd_default;

let close = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0,
                 20.0, 19.0, 18.0, 17.0, 16.0, 15.0, 14.0, 13.0, 12.0, 11.0];

// Bollinger Bands: three equal-length bands, leading unstable period is NaN.
let b = bbands_default(&close).unwrap();
let _ = (b.upper, b.middle, b.lower);

// MACD: macd / signal / hist, aligned to the same leading NaN.
let m = macd_default(&close).unwrap();
let _ = (m.macd, m.signal, m.hist);
```

OHLC input example (ATR, needs high/low/close):

```rust
use adaq_talib::volatility::atr_default;

// A 40-bar sample so the first valid value is observable (leading period-1 = 13 are NaN).
let high:  Vec<f64> = (0..40).map(|i| 10.0 + i as f64 * 0.1 + 0.5).collect();
let low:   Vec<f64> = (0..40).map(|i| 9.0  + i as f64 * 0.1 - 0.5).collect();
let close: Vec<f64> = (0..40).map(|i| 9.5  + i as f64 * 0.1).collect();

let out = atr_default(&high, &low, &close).unwrap();
// The first `period - 1` = 13 positions are NaN; the first valid value is the
// mean of the first 14 True Ranges (Wilder-smoothing seed).
assert!(out[0].is_nan() && out[12].is_nan());
assert!(!out[13].is_nan()); // first valid at index period-1 = 13
```

---

## Installation

### Requirements

- **Rust toolchain ≥ 1.85** (this crate uses `edition = "2024"`, see `rust-version` in
  `Cargo.toml`). Install via [rustup.rs](https://rustup.rs).
- **No external crates**: the published crate's `[dependencies]` is empty.
- **No TA-Lib C library or Python** required to build or test (golden vectors are in-repo).

### From crates.io

```bash
cargo add adaq-talib
```

Or add manually to `Cargo.toml`:

```toml
[dependencies]
adaq-talib = "0.1"
```

Build from source (after cloning):

```bash
git clone https://github.com/tonywxx/adaq-talib
cd adaq-talib
cargo build --release
```

---

## Usage

### 1. Calling conventions

- **Single input series**: e.g. `sma(&prices, period)`, `rsi(&close, period)`.
- **OHLC multi-input**: e.g. `atr(&high, &low, &close, period)`,
  `cci(&high, &low, &close, period)`, `bop(&open, &high, &low, &close)`.
- **With volume**: e.g. `mfi(&high, &low, &close, &volume, period)`, `obv(&close, &volume)`.
- **Multi-output**: returned as a dedicated struct, e.g. `Macd { macd, signal, hist }`,
  `Bbands { upper, middle, lower }`, `Aroon { up, down }`, `Mama { mama, fama }`.

### 2. Default-parameter helpers

Every function has a `_default` variant that applies TA-Lib's original default parameters:

```rust
use adaq_talib::momentum::rsi_default;
use adaq_talib::volatility::atr_default;

let rsi = rsi_default(&close).unwrap();   // default period = 14
let atr = atr_default(&high, &low, &close).unwrap(); // default period = 14
```

### 3. Error handling

All functions return `Result<_, TaError>`. The common error is `TaError::BadParam`
(e.g. `period == 0`, mismatched input lengths, or `mama`'s `fast/slow_limit` outside
[0.01, 0.99]).

```rust
use adaq_talib::overlap::sma;
use adaq_talib::TaError;

match sma(&[1.0, 2.0], 0) {
    Ok(out) => println!("{out:?}"),
    Err(TaError::BadParam(msg)) => eprintln!("bad param: {msg}"),
    Err(e) => eprintln!("other error: {e}"),
}
```

### 4. Unstable period & NaN

Consistent with TA-Lib, the warm-up (unstable) period is filled with `f64::NAN`, and the
**return is equal-length with the input**. Common lookbacks:

- SMA/EMA/WMA/TRIMA etc.: `period - 1`
- DEMA: `2*(period-1)`; TEMA: `3*(period-1)`; T3: `6*(period-1)`; TRIX: `3*period-2`
- RSI/CMO/CCI etc.: `period` (Wilder-style)
- ADX: `2*period-1`; ADXR: `3*period-2`
- MAMA: `32`; HT_TRENDLINE: `63`
- Lookback 0 (no leading NaN): `trange`, `ad`, `obv`, price transforms, `bop`

Skip the unstable period with `f64::is_nan()` before consuming the valid segment.

### 5. Reuse outside the library

`src/utils.rs` and `src/core/` are `doc(hidden)` internal implementation details (alignment,
range checks, and shared rolling primitives) and are **not** part of the public API — do not
depend on them externally.

---

## Interactive Demo

`src/main.rs` is a command-line demo entry point that runs a built-in example by indicator
name — **no code required**:

```bash
cargo run -- sma
cargo run -- rsi
cargo run -- macd
cargo run -- bbands
cargo run -- atr
cargo run -- adx
cargo run -- mama
```

All supported indicators (run directly with `cargo run -- <name>`, grouped by category):

```
Overlap:            sma ema wma dema tema midpoint midprice bbands trima t3 ma mavp kama sar sarext accbands
Momentum:           rsi cmo trix mom macd cci willr bop ultosc adx aroon stoch mfi dx imi
Volatility:         trange atr natr
Volume:             ad adosc obv
Price Transform:    avgprice medprice typprice wclprice avgdev
Statistic:          stddev var linear_reg linear_reg_angle linear_reg_intercept linear_reg_slope tsf beta correl
Cycle:              mama ht_trendline
```

> Note: the interactive demo covers the indicators above. The remaining implemented functions
> (e.g. `roc`, `rocp`, `apo`, `ppo`, `stoch_f`, `stoch_rsi`, `aroon_osc`, the directional
> components, `adxr`, `macd_fix`, `macd_ext`, the math operators in `math_ops`, the math
> transforms in `math_trans`, and all 61 candlestick patterns in `pattern`, …) are callable
> directly in code — see the [function overview](#implemented-functions) above.

> An unknown name prints the supported list and exits with code 2.

---

## Verification & Benchmarks

### Correctness (1:1 golden vectors)

- `cargo test` compares against in-repo **golden vectors** (real TA-Lib C 0.7.1 output, in
  `tests/fixtures/` — **222 golden-vector fixture files** covering the full 161-function surface).
  Running the tests needs **no** Python or TA-Lib C library.
- Tolerance: relative `1e-8` + absolute `1e-10` ([ADR 0005](docs/adr/0005-error-tolerance.md)).
- Full suite: **326 tests, 0 failures**; `tools/reconcile.py` confirms **161/161** public functions
  map 1:1 to TA-Lib 0.7.1 (exit 0). The complete 1:1 validation and performance report for all
  161 indicators is in [`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md).
- The golden-vector generator lives in [`tools/gen_fixtures`](tools/) (requires TA-Lib C +
  the `TA-Lib` Python binding; for maintainers only). Current fixture status and known gaps
  are in [`tools/README.md`](tools/README.md).

```bash
cargo test                 # unit tests + golden-vector comparison (no Python / C needed)
cargo test --doc           # doc examples only
```

### Performance (Rust vs TA-Lib C)

All **161 / 161** indicators were benchmarked head-to-head against native TA-Lib C 0.7.1
(dual-track, [ADR 0004](docs/adr/0004-benchmark-dual-track.md); the C track FFI-links system
TA-Lib C under `--features bench-c`). Environment: Apple Silicon aarch64, **N = 100,000** elements
per indicator; `ns/elem = elapsed / ITERS / N`; `Rust/C = Rust_ns/elem ÷ C_ns/elem`.
Status: ratio < 0.8 → Faster, 0.8–1.2 → At parity, > 1.2 → Slower.

**Headline:** **36 faster**, **33 at parity**, **92 slower** than native C; geomean
**Rust/C = 1.50×** (adaq-talib is ~1.5× slower than C on average). **54 indicators are strictly
faster than C** (Rust/C < 1).

| TA-Lib Group | Indicators | Faster (<0.8) | At parity (0.8–1.2) | Slower (>1.2) | Geomean Rust/C |
|---|---:|---:|---:|---:|---:|
| Cycle Indicators | 5 | 0 | 1 | 4 | 1.57× |
| Math Operators | 11 | 5 | 1 | 5 | 1.01× |
| Math Transform | 15 | 4 | 11 | 0 | 0.85× |
| Momentum Indicators | 31 | 7 | 5 | 19 | 1.31× |
| Overlap Studies | 18 | 5 | 8 | 5 | 0.98× |
| Pattern Recognition | 61 | 1 | 3 | 57 | 2.98× |
| Price Transform | 5 | 5 | 0 | 0 | 0.58× |
| Statistic Functions | 9 | 7 | 1 | 1 | 0.54× |
| Volatility Indicators | 3 | 1 | 2 | 0 | 0.83× |
| Volume Indicators | 3 | 1 | 1 | 1 | 0.97× |
| **Total** | **161** | **36** | **33** | **92** | **1.50×** |

adaq-talib is faster than C on Statistic, Price Transform and Math Transform; at parity on
Overlap / Volume / Volatility / Math Operators; and slower on Momentum, Cycle and most markedly
Pattern Recognition (57/61 slower). The full per-indicator table — all 161, with Rust/C ratio,
status and a live TA-Lib parity checksum — is in
[`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md).
Caveats (e.g. `stoch_rsi` exposes only the `fastk` line, so its bench checksum differs — a bench
artifact, not a correctness gap) are detailed there.

### Performance optimizations applied

All optimizations below are **zero-deviation** — verified 1:1 against TA-Lib 0.7.1 golden
vectors (see [`benches/BASELINE.md`](benches/BASELINE.md) for the full per-indicator write-up).
`ns/elem` measured on Apple Silicon aarch64, `N = 1_000_000`, `PERIOD = 20`, `ITERS = 20`
(spot measurement, ±5% jitter).

| Phase | Function(s) | Technique | After (Rust ns/elem) | Speed-up vs naive |
|-------|-------------|-----------|---------------------:|------------------:|
| P2-1 | `dema` / `tema` / `t3` | single-pass nested-EMA fusion core (`core::nested_ema_with_output`) | 3.63 / 3.46 / 3.76 | ~2× / ~3× / ~6× |
| P2-2 | `midpoint` / `midprice` | monotonic-queue `core::rolling_extreme` O(n) | 6.88 / 7.30 | ~3× / ~3× |
| P2-3 | `wma` | O(n) sliding recurrence (`W[i] = W[i-1] + period·x[i] − sw[i-1]`) | 2.11 | ~4.7× |
| P2-4 | `bbands` (SMA middle) | single-pass `rolling_mean_var` fusion | 3.02 | ~1.5–1.6× |
| P2-5 | `linear_reg` family / `correl` | O(n) sliding sum / cross-product | 2.33 / 4.81 | ~20× asymptotic |
| P2-5 | `willr` / `stoch` / `stoch_f` | shared monotonic extreme queue O(n) | 7.90 / 10.99 | ~20× asymptotic |
| P1② | `minmax` | reuse single-pass `core::rolling_minmax` (consolidation; perf-neutral) | 6.76 | ≈ (accuracy-only) |
| P1③ | `max_index` / `min_index` / `minmax_index` | single-pass `core::rolling_extreme_index` O(n) | 3.43 / 3.31 / 6.79 | ~1.9× (index) |

Every optimization keeps the full `cargo test` suite green (326/326) and each refactored function
still reproduces its TA-Lib 0.7.1 golden vector within tolerance
([ADR 0005](docs/adr/0005-error-tolerance.md)). The full QA write-up (methodology, residual gaps,
Python-binding reference numbers) lives in [`docs/perf-verify-report.md`](docs/perf-verify-report.md).

### Benchmarks (how to run)

```bash
# 1) Rust side (default, dependency-free): std::time timing, harness = false
cargo bench --bench sma_bench

# 2) Native C comparison (optional feature): FFI-links the system TA-Lib C library
cargo bench --bench sma_bench --features bench-c

# 3) All 161 indicators vs native C (auto-generated suite):
cargo bench --bench all161_bench
cargo bench --bench all161_bench --features bench-c   # with the C reference track
```

> The second form needs the TA-Lib C library installed (`brew install ta-lib` / build from
> source); `build.rs` links it only under `bench-c`, so the build is unaffected otherwise.
> Reports must clearly distinguish the two tracks.

---

## Known Issues & Deprecations

### Known issues
- **Two indicators are still slower than native TA-Lib C** — `MIDPOINT` (~2.26×) and `T3` (~1.35×). Both are structurally non-vectorizable (a data-dependent monotonic deque and a sequential EMA IIR respectively), so the planned P3 SIMD pass is a documented **NO-GO** ([ADR 0010](docs/adr/0010-performance-strategy.md)). This is a known, accepted trade-off, **not a defect**.
- **No native-C wiring for `linear_reg` / `correl` / `willr` / `stoch`** — their Rust-side numbers are the canonical reference. A C comparison would require `unsafe` plus the system TA-Lib C library, which goes against the zero-FFI design; their Rust results are authoritative.
- **Pattern recognition uses TA-Lib's default candle settings only** ([ADR 0009](docs/adr/0009-candle-settings-default-only.md)); no configuration API is exposed. There is **no functional coverage gap** against TA-Lib 0.7.1 — all 61 candlestick patterns are implemented.
- **`aroon` / `aroon_osc` output order** — adaq-talib follows the canonical TA-Lib C 0.7.1 `outAroonUp` / `outAroonDown` order (the authoritative golden vectors). If you cross-check against the `talib` Python wheel (0.7.1), note that build historically swaps these two outputs; see [ADR 0003](docs/adr/0003-verification-golden-fixtures.md).

### Dependencies
- **No runtime dependencies.** The published crate's `[dependencies]` is and remains empty. All recent work adds only *dev* benchmarks (`benches/`), a release workflow, and internal `core` primitives — no new external crates are introduced.

### Deprecated features
- **None.** This release introduces no deprecations and removes no published capability ([ADR 0002](docs/adr/0002-release-scope-milestones.md)).

---

## Documentation

- Design decisions (ADR 0001–0009): [`docs/adr/`](docs/adr/)
- Unified API conventions: [`docs/api-conventions.md`](docs/api-conventions.md)
- 0.1.0 function scope baseline: [`docs/0.1.0-scope.md`](docs/0.1.0-scope.md)
- Glossary: [`CONTEXT.md`](CONTEXT.md)
- Every public function carries a bilingual (Chinese/English) doc-comment (formula source,
  parameters, return value, leading `NaN`, runnable example). Browse with `cargo doc --open`.

---

## License

Apache-2.0 (see [`LICENSE`](LICENSE)).

---

## Roadmap

Milestone-based release ([ADR 0002](docs/adr/0002-release-scope-milestones.md)). **This release
ships the complete TA-Lib 0.7.1 public surface — all 161 functions across 10 categories — with no
deletion of published capabilities.**

- ✅ **0.1.2 (current): 161 / 161 functions** — Overlap Studies (18), Momentum (31), Volatility
  (3), Volume (3), Price Transform (5), Statistic (9), Cycle / Hilbert Transform (7), Math
  Operators (11), Math Transform (15), and Pattern Recognition (61 candlestick patterns). Every
  function is verified 1:1 against TA-Lib 0.7.1 golden vectors (`cargo test` → 326/326 green,
  `reconcile.py` → 161/161) across **222 golden-vector fixtures**, and a comprehensive all-161
  benchmark + validation suite ([`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md))
  confirms full coverage and performance parity (see
  [Verification & Benchmarks](#verification--benchmarks)).
- 🔜 **Future work (post-1.0)**: per [ADR 0009](docs/adr/0009-candle-settings-default-only.md) only
  the **default** candle settings are implemented and no configuration API is exposed; optional
  `bench-c` wiring for the newly optimized indicators (LINREG/CORREL/WILLR/STOCH), and
  documentation/CI polish. **No functional coverage gap remains against TA-Lib 0.7.1.**

Once those land, adaq-talib reaches full coverage equivalent to TA-Lib 0.7.1.

---

## Changelog

### 0.1.2 (current)
- **Comprehensive all-161 benchmark & validation suite**: new `benches/all161_bench.rs`
  (auto-generated by `tools/bench/gen_all161.py`) benchmarks **all 161** indicators
  head-to-head against native TA-Lib C 0.7.1 with a live numeric parity checksum; companion
  `benches/poc_bench.rs` is a proof-of-concept harness. The unified
  [`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md),
  the interactive `docs/benchmarks/adaq-vs-talib-161.html`, and `all161_results.csv` are produced
  by `tools/bench/gen_report.py` — all under the dual-track methodology ([ADR 0004](docs/adr/0004-benchmark-dual-track.md)).
- **Expanded golden-vector coverage**: **222 golden-vector fixture files** (up from 159) — added
  the full Pattern Recognition fixture set and the `macd_ext` / `macd_fix` fixtures. The full test
  suite is now **326 tests, 0 failures** (was 308), and `tools/reconcile.py` confirms **161/161**.
- **Documentation completeness**: the per-function tables now list every one of the 161 functions.
  `accbands` (Overlap), `dx` / `imi` (Momentum) and `avgdev` (Price Transform) were already
  implemented and counted in the 161 total, but had been omitted from the detailed tables — they
  are now documented.
- **Release**: version bumped to `0.1.2`. No new public API beyond the above; no deprecations,
  no dependency changes ([ADR 0002](docs/adr/0002-release-scope-milestones.md)).

### 0.1.1
- **Math operators — O(n) extreme-index functions**: `max_index` / `min_index` / `minmax_index` now use a single-pass monotonic-queue (`core::rolling_extreme_index`), replacing the former O(n·period) nested scan — ~1.9× faster while remaining 1:1 with TA-Lib 0.7.1 ([ADR 0005](docs/adr/0005-error-tolerance.md)). Added `benches/index_bench.rs` and `benches/minmax_bench.rs`.
- **`minmax` consolidation**: `math_ops::minmax` now reuses the single-pass `core::rolling_minmax` core (the same one used by `midpoint`), eliminating duplicated extreme logic. Performance-neutral; accuracy unchanged.
- **Full P2 performance sweep (verified 1:1)**: nested-EMA fusion for `dema` / `tema` / `t3` (P2-1); monotonic-queue `midpoint` / `midprice` (P2-2); O(n) sliding `wma` (P2-3); single-pass `bbands` middle (P2-4); sliding O(n) `linear_reg` family / `correl` / `willr` / `stoch` (P2-5). See [`benches/BASELINE.md`](benches/BASELINE.md).
- **Release tooling & docs**: added `.github/workflows/release.yml` (release automation) and CI; doc-comment and publish-`exclude` fixes; version bumped to `0.1.1`.
- **Pattern Recognition + Math Operations modules**: all 61 candlestick patterns and the full `math_ops` / `math_trans` surface are implemented, with comprehensive golden-vector fixtures (P4 milestone — 161/161 functions).

### 0.1.0
- Initial public milestone: the complete TA-Lib 0.7.1 public surface — 161 functions across 10 categories — with zero-deviation golden-vector verification.
