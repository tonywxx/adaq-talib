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
- [Documentation](#documentation)
- [License](#license)
- [Roadmap](#roadmap)

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

As of this release, **7 categories and 65 TA-Lib 0.7.1 indicator functions** are implemented.
The tables below list every public function, its TA-Lib counterpart, default parameters, and
return shape.

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

### Add the dependency

If published on crates.io:

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

`src/utils.rs` is a `doc(hidden)` internal implementation detail (alignment, range checks)
and is **not** part of the public API — do not depend on it externally.

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
Overlap:            sma ema wma dema tema midpoint midprice bbands trima t3 ma mavp kama sar sarext
Momentum:           rsi cmo trix mom macd cci willr bop ultosc adx aroon stoch mfi
Volatility:         trange atr natr
Volume:             ad adosc obv
Price Transform:    avgprice medprice typprice wclprice
Statistic:          stddev var linear_reg linear_reg_angle linear_reg_intercept linear_reg_slope tsf beta correl
Cycle:              mama ht_trendline
```

> Note: the interactive demo covers the indicators above. The remaining implemented functions
> (e.g. `roc`, `rocp`, `apo`, `ppo`, `stoch_f`, `stoch_rsi`, `aroon_osc`, the directional
> components, `adxr`, `macd_fix`, `macd_ext`, …) are callable directly in code — see the
> [function overview](#implemented-functions) above.

> An unknown name prints the supported list and exits with code 2.

---

## Verification & Benchmarks

### Correctness

- `cargo test` compares against in-repo **golden vectors** (real TA-Lib C 0.7.1 output, in
  `tests/fixtures/`, 63 files). Running the tests needs **no** Python or TA-Lib C library.
- Tolerance: relative `1e-8` + absolute `1e-10` ([ADR 0005](docs/adr/0005-error-tolerance.md)).
- The golden-vector generator lives in [`tools/gen_fixtures`](tools/) (requires TA-Lib C +
  the `TA-Lib` Python binding; for maintainers only). Current fixture status and known gaps
  are in [`tools/README.md`](tools/README.md).

```bash
cargo test                 # unit tests + golden-vector comparison (no Python / C needed)
cargo test --doc           # doc examples only
```

### Benchmarks

Dual-track benchmarks ([ADR 0004](docs/adr/0004-benchmark-dual-track.md)):

```bash
# 1) Rust side (default, dependency-free): std::time timing, harness = false
cargo bench --bench sma_bench

# 2) Native C comparison (optional feature): FFI-links the system TA-Lib C library
cargo bench --bench sma_bench --features bench-c
```

> The second form needs the TA-Lib C library installed (`brew install ta-lib` / build from
> source); `build.rs` links it only under `bench-c`, so the build is unaffected otherwise.
> Reports must clearly distinguish the two tracks.

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

Milestone-based release ([ADR 0002](docs/adr/0002-release-scope-milestones.md)) — the final
release covers **all** TA-Lib 0.7.1 built-ins with **no deletion** of published capabilities:

- ✅ **0.1.0 (current)**: Overlap + Momentum + Volatility + Volume + Price Transform + Statistic
  + Cycle (MAMA/HT_TRENDLINE) — 65 functions.
- ⏳ **Later milestones**: Pattern Recognition (candlestick, ~61, default candle settings only,
  [ADR 0009](docs/adr/0009-candle-settings-default-only.md)), Math Operators (7: ADD/DIV/MAX/MIN/
  MULT/SUB/SUM), Math Transform (15: ACOS/ASIN/ATAN/CEIL/COS/COSH/EXP/FLOOR/LN/LOG10/SIN/SINH/
  SQRT/TAN/TANH).

Once those land, adaq-talib reaches full coverage equivalent to TA-Lib 0.7.1.
