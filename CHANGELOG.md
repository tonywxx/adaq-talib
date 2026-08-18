# Changelog

This file was extracted from `README.zh-CN.md` (and `README.md`) into a standalone changelog
(2026-08-17). Every entry is verified under the
[measure-first double gate](docs/adr/0010-performance-strategy.md) (golden vectors 1:1 + A/B
`cargo bench` median). The zero-deviation promise is unchanged
([ADR 0005](docs/adr/0005-error-tolerance.md)). No new public API, no deprecations, no dependency
changes ([ADR 0002](docs/adr/0002-release-scope-milestones.md)), unless an entry says otherwise.

---

### V0.1.9

- **TA-Lib 0.7.1 accuracy hardening (correctness fixes, no public API change)**: closed several
  real divergences from C TA-Lib that the 1:1 golden vectors previously masked:
  - `macd_fix` / `macd_fix_default` (`TA_MACDFIX`): the fixed historic smoothing factors `0.15` /
    `0.075` are now used for the fast/slow EMAs (the famous MACDFIX quirk) instead of the standard
    `2/(period+1)` factor, and `signal_period` is now configurable. Before this, `macd_fix` was just
    `macd(12, 26, 9)` with fixed signal and did **not** match C's `TA_MACDFIX`.
  - `stoch_rsi` / `stoch_rsi_with_output` (`TA_STOCHRSI`): now also emits the `fastD` line, and `fastD`
    is aligned to the same leading unstable period as `fastK` (C aligns them by skipping `fastD−1`
    internal K values in `TA_STOCHF`), so `fastD`'s first valid index equals `fastK`'s — not later.
  - `max_index` / `min_index` / `minmax_index` (`TA_MAXINDEX` / `TA_MININDEX` / `TA_MINMAXINDEX`):
    the leading `period−1` positions are now `NaN` (C never writes them — "no value") instead of `0.0`.
  - `cdl_shootingstar` (`TA_CDLSHOOTINGSTAR`): lookback padded by one extra leading candle
    (the evaluated bar sits at `lookback`), matching C's `outBegIndex`.
  - Regenerated the affected golden-vector fixtures (`macd_fix_basic` / `max_index_basic` /
    `min_index_basic` / `minmax_index_basic`) from the C `talib` 0.7.1 binding.
  - Verified under the **measure-first double gate**: all 21 `cargo test` binaries green (0 failures,
    incl. 61 candle-pattern + 31 momentum golden vectors); the 4 regenerated fixtures reproduced
    bit-for-bit from C (max err 1.4e-14 for MACDFIX, exact 0.0 for the index family), and Rust
    `stoch_rsi` fastK+fastD matched C to ~2e-13. These are accuracy fixes, **not** perf optimizations:
    the Wilder-family hot paths are unchanged (median Δ within ±1.4%, i.e. 0 regression); the
    `macd_fix` / `stoch_rsi` edits add no structural regression (the new `fastD` copy costs ~0.18%).
- **Release**: version bumped to `0.1.9`. No new public API, no deprecations, no dependency changes
  ([ADR 0002](docs/adr/0002-release-scope-milestones.md)). User-facing behavior, calling conventions,
  and the `cargo test` / `cargo bench` workflows are unchanged.

### V0.1.8

- **Architecture-deepening wrap-up (measure-first double gate)**: candidate② (Wilder recurrence
  consolidation, see 0.1.7) is **adopted** under the measure-first gate (golden vectors 1:1 + A/B
  `cargo bench` median-of-9) — the 7 Wilder-family indicators (`rsi` / `cmo` / `plus_di` / `minus_di` /
  `dx` / `adx` / `adxr`) are **−12% ~ −46%** faster. Candidate③ (remove/dedup the default-off `parallel`
  feature) and candidate④ (pattern runtime / doc navigation index) are **rejected** by the same gate:
  both are structural refactors that don't touch the hot compute kernels, so they can't demonstrate a
  default-build perf gain (candidate④'s in-kernel shared-offset variant would regress 170–291%, like
  candidate①'s CandleAvg seam), so they are not adopted. Zero-deviation is unchanged (ADR 0005).
- **Docs & release hygiene**: Changelog extracted into this standalone file; the performance docs
  ([`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md) §3.6,
  [`docs/perf-verify-report.md`](docs/perf-verify-report.md),
  [`benches/BASELINE.md`](benches/BASELINE.md)) record the Wilder speedup and the median-of-9 bench
  guard (`benches/momentum_wilder_bench.rs` / `benches/cdl_bench.rs`); the README optimization table
  gains a Wilder row.
- **Release**: version bumped to `0.1.8`. No new public API, no deprecations, no dependency changes
  ([ADR 0002](docs/adr/0002-release-scope-milestones.md)).

### V0.1.7

- **Wilder recurrence seam consolidation (architecture-deepening candidate②)**: routed the 5 inline
  Wilder recurrences in `momentum.rs` onto `core::ema` primitives — `rsi` / `cmo` / `adx` use the mean
  form `wilder_step(prev, x, k)`, `dm_tr` / `adx_adxr_fused` / `dx_from_candles` use the sum form
  `wilder_step_sum(prev, x, k)` for ±DM/TR (the two forms are kept distinct, because the `period` factor
  only cancels in the ±DI ratio — blindly unifying them would corrupt +DI/−DI); `ema_wilder` now delegates
  to the new zero-copy `wilder_with_output`. Passed under the **measure-first double gate** (golden vectors
  1:1 + A/B `cargo bench` median-of-9): 31/31 momentum golden vectors 1:1, full `cargo test` green (incl.
  ATR/NATR via the refactored `ema_wilder`); `rsi` / `cmo` / `plus_di` / `minus_di` / `dx` / `adx` / `adxr`
  are **−12% ~ −46%** faster (`momentum_wilder_bench`, N=100k, 9-round median) — the speedup comes from
  replacing the per-step `/p` float division in the hot loop with a precomputed `k = 1/period` multiply.
- **Benchmark suite**: added `benches/momentum_wilder_bench.rs` (Wilder-family micro-bench, median-of-9);
  `benches/cdl_bench.rs` hardened from single-shot `Instant` to **median-of-9** (single-shot noise can read
  ±10%, which we proved mis-reports).
- **Release**: version bumped to `0.1.7`. No new public API, no deprecations, no dependency changes
  ([ADR 0002](docs/adr/0002-release-scope-milestones.md)). User-facing behavior, calling conventions, and
  the `cargo test` / `cargo bench` workflows are unchanged.

### V0.1.6

- **Candle-pattern kernels — `real_body` recompute dedup (perf(pattern))**: 20 candlestick kernels now
  reuse the already-computed `cur_avg_*` sliding-window value inside each condition instead of
  recomputing `real_body(open[i], close[i])`. Pure reordering — no arithmetic change — so the TA-Lib
  0.7.1 golden vectors stay bit-identical (all 144 candle integration tests pass). Control-corrected
  A/B vs the original baseline (median of 3 runs, env-drift corrected via untouched controls): 12
  clean wins (e.g. `cdl_closingmarubozu` −57%, `cdl_marubozu` −36%, `cdl_stalledpattern` −27%,
  `cdl_counterattack` −24%), 3 flat (`cdl_belthold` / `cdl_longleggeddoji` / `cdl_eveningstar`), and 5
  apparent "regressions" (`cdl_3starsinsouth` / `cdl_3whitesoldiers` / `cdl_abandonedbaby` /
  `cdl_eveningdojistar` / `cdl_morningstar`) identified as environment noise — removing a recompute
  cannot slow a function and the golden vectors are identical, so all were kept. Also folds in
  `cdl_harami` CandleAvg consolidation (validated win) and `cdl_homingpigeon` / `longline` /
  `shortline` shadow+body dedups.
- **Indicator scaffold rollout (`indicator!` macro) — consistency**: migrated `midprice`, `sar`,
  `sarext`, `avgprice`, `medprice`, `typprice`, `wclprice`, `ad`, `adosc`, and `obv` to the zero-cost
  `indicator!` macro (introduced in 0.1.5), removing redundant error-handling / output-init
  boilerplate; each function keeps its detailed bilingual doc-comment. The pattern module was migrated
  as well. Output remains golden-vector 1:1.
- **Candle-pattern modules — readability refactor**: removed unnecessary parentheses in arithmetic
  across the batch files, consolidated average-calculation variable initialization, and added explicit
  `#[allow(...)]` for unused assignments / variables in `pattern/mod.rs` to keep strict builds
  warning-free.
- **CI**: upgraded `actions/checkout` to **v5** in `.github/workflows/ci.yml` and `release.yml`.
- **Benchmark suite**: added `benches/cdl_bench.rs` and extended `benches/phase1c_bench.rs` /
  `benches/poc_bench.rs`; regenerated `all161_results.csv`.
- **Release**: version bumped to `0.1.6`. No new public API surface, no deprecations, no dependency
  changes ([ADR 0002](docs/adr/0002-release-scope-milestones.md)). User-facing behavior, calling
  conventions, and the `cargo test` / `cargo bench` workflows are unchanged.

### V0.1.5

- **Indicator scaffold (`indicator!` macro) — architecture-deepening candidate① (Phase 1a/1b/1c)**: added `src/indicator.rs` with a **zero-cost `macro_rules! indicator`** that unifies the repetitive "allocate an equal-length `f64::NAN` buffer → forward to the `*_with_output` kernel" glue shared by ~146 single-output public functions. Rolled out under a **measure-first double gate** (golden-vector 1:1 + A/B `cargo bench` median |Δ| ≤ ±5%):
  - **Phase 1a**: `math_trans` 15 single-input / single-output / element-wise functions.
  - **Phase 1b**: `stat` 7 single-input functions (`stddev`/`var`/`linear_reg`/`linear_reg_angle`/`linear_reg_intercept`/`linear_reg_slope`/`tsf`) via the new N-trailing-default arm; `beta`/`correl` (multi-input) stay hand-written (Phase 2).
  - **Phase 1c**: `math_ops` 9 (`add`/`sub`/`mult`/`div`/`sum`/`min`/`max`/`max_index`/`min_index`) + `volatility` 3 (`trange`/`atr`/`natr`, two with default arms) + `price_transform::avgdev`; `avgprice`/`medprice`/`typprice`/`wclprice` were **intentionally reverted to hand-written** — the macro's uniform `vec![f64::NAN; n]` init regresses them (isolated micro-bench: `avgprice` +34.7%, `add` +22.2%; A/B median |Δ| = 16–17% ≫ 5%), while they need no leading NaN and carry no default args (zero macro benefit).
- **Zero-cost guarantee verified**: the macro expands to byte-identical code (no `dyn Fn`, no indirection, no per-iteration allocation); the `*_with_output` hot paths are untouched. A/B results — Phase 1a max median |Δ| = **2.97%**, Phase 1b = **0.11%**, Phase 1c = **0.21%** (all ≤ 5% → PASS). Golden-vector gate: all **161/161** functions still reproduce TA-Lib 0.7.1 within tolerance; the full `cargo test` suite stays green (incl. new macro-emitted `doctest`s).
- **New A/B benchmark harness (methodology)**: added `benches/math_trans_bench.rs`, `benches/stat_bench.rs`, `benches/phase1c_bench.rs` (all registered in `Cargo.toml`) — a dependency-free `Instant` harness using **warmup + interleaved rounds + median** to suppress single-shot noise (which can read ±10%). Documented in [`benches/BASELINE.md`](benches/BASELINE.md) and [ADR 0011](docs/adr/0011-indicator-scaffold-seam.md).
- **Release**: version bumped to `0.1.5`. No new public API surface, no deprecations, no dependency changes ([ADR 0002](docs/adr/0002-release-scope-milestones.md)). User-facing behavior, calling conventions, and the `cargo test` / `cargo bench` workflows are unchanged.

### V0.1.4

- **Core modularization (architecture deepening)**: split the monolithic `src/core/mod.rs` into focused, single-responsibility modules — `ema.rs` (nested-EMA fusion), `extreme.rs` (monotonic-queue rolling extremes / indices), `window.rs` (windowed sums / variances), and `kernel.rs` (shared kernel helpers). Removed the redundant `check_eq_len` length-guard helper (length checks now live next to each kernel). Pure refactor — output remains bit-for-bit / golden-vector 1:1 with TA-Lib 0.7.1, zero performance impact.
- **`parallel` feature promoted to a first-class module**: the overlap-seed parallel chunking (formerly a proof-of-concept) is now `src/parallel.rs`, guarded by a dedicated `tests/parallel_equality.rs` 1:1 equality test and exercised by `benches/parallel_poc.rs`. The 5 A-class window functions (`midpoint`/`minmax`/`minmax_index`/`willr`/`stoch_f`) gain multi-core speedups under the default-off `parallel` feature — totals move **85 Faster / 60 Parity / 16 Slower (geomean 0.786×) → 88 / 63 / 10 (0.734×)**. For the other 156 functions it is a no-op.
- **Perf report & 161-indicator suite refresh**: refreshed [`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md) and the `all161_results*.csv` benchmark data; folded in the finalized 0.1.3 optimization-pass numbers (EMA-family FMA-contraction closing the EMA gap — see [Verification & Benchmarks](#verification--benchmarks)).
- **Release**: version bumped to `0.1.4`. No new public API surface, no deprecations, no dependency changes ([ADR 0002](docs/adr/0002-release-scope-milestones.md)).

### V0.1.3

- **Pattern Recognition performance rollout**: the `cdl_hammer` inline running-sum accumulator
  template was applied to **all 61 candlestick functions** (parity-preserving transformer
  `tools/opt_pattern.py`); per-function `CandleAvg::new`+`value`+`advance` replaced by inline
  `sum_*`/`trail_*`/`cur_*`/`val_*` accumulators (skipping functions with no `CandleAvg`, e.g.
  `cdl_engulfing`/`cdl_3outside`/`cdl_hikkake`/`cdl_tristar`). Pattern Recognition geomean
  **Rust/C dropped from 2.98× → 0.677×** (43 faster / 13 at parity / 5 slower, was 1/3/57) — the
  single biggest driver of the release.
- **P2 algorithm-optimization pass (zero-deviation, 0 regressions)**: a ring-buffer `MonoQueue`
  replacing the `VecDeque` rolling extremes (`min`/`max`/`min_index`/`max_index`, ~32% faster per
  extreme); a cycle-IIR fast path that skips the unused `compute_dc_phase` sin/cos window in
  `ht_dcperiod` (3.59× → 1.19×, now at parity); a sin/cos angle-addition recurrence in
  `compute_dc_phase` (`ht_dcphase`/`ht_sine`/`ht_trendmode`); and a single-pass sliding-window
  fusion of `mfi` (2.56× → 1.41×). Net: **82 faster / 54 at parity / 25 slower, geomean
  Rust/C = 0.792×** — adaq-talib is now ~1.26× faster than C on average (was 1.50× slower).
- **Reports & tooling**: updated [`docs/validation-and-performance-report.md`](docs/validation-and-performance-report.md)
  (new group/per-indicator tables, median-of-3-run methodology) and the interactive
  `docs/benchmarks/adaq-vs-talib-161.html`; added `benches/extreme_ab.rs`, `tools/opt_pattern.py`,
  and `docs/research/perf-161-analysis.md`.
- **Release**: version bumped to `0.1.3`. No new public API, no deprecations, no dependency
  changes ([ADR 0002](docs/adr/0002-release-scope-milestones.md)).

### V0.1.2

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

### V0.1.1

- **Math operators — O(n) extreme-index functions**: `max_index` / `min_index` / `minmax_index` now use a single-pass monotonic-queue (`core::rolling_extreme_index`), replacing the former O(n·period) nested scan — ~1.9× faster while remaining 1:1 with TA-Lib 0.7.1 ([ADR 0005](docs/adr/0005-error-tolerance.md)). Added `benches/index_bench.rs` and `benches/minmax_bench.rs`.
- **`minmax` consolidation**: `math_ops::minmax` now reuses the single-pass `core::rolling_minmax` core (the same one used by `midpoint`), eliminating duplicated extreme logic. Performance-neutral; accuracy unchanged.
- **Full P2 performance sweep (verified 1:1)**: nested-EMA fusion for `dema` / `tema` / `t3` (P2-1); monotonic-queue `midpoint` / `midprice` (P2-2); O(n) sliding `wma` (P2-3); single-pass `bbands` middle (P2-4); sliding O(n) `linear_reg` family / `correl` / `willr` / `stoch` (P2-5). See [`benches/BASELINE.md`](benches/BASELINE.md).
- **Release tooling & docs**: added `.github/workflows/release.yml` (release automation) and CI; doc-comment and publish-`exclude` fixes; version bumped to `0.1.1`.
- **Pattern Recognition + Math Operations modules**: all 61 candlestick patterns and the full `math_ops` / `math_trans` surface are implemented, with comprehensive golden-vector fixtures (P4 milestone — 161/161 functions).

### V0.1.0

- Initial public milestone: the complete TA-Lib 0.7.1 public surface — 161 functions across 10 categories — with zero-deviation golden-vector verification.
