# adaq-talib 验证 + 性能对照报告 · Verification & Performance Report

- **验证人 / QA**: 严过关 (Yan)
- **日期 / Date**: 2026-08-10
- **仓库 / Repo**: `/Users/tony/github/adaq-talib`
- **对照基准 / Baseline**: TA-Lib 0.7.1
- **方法论 / Method**: 沿用 `benches/BASELINE.md`。Rust 基准使用受管 `cargo`（**未**加 `-C target-cpu=native`；该标志仅用于临时 SIMD 对照）。

---

## 1. 正确性验证 · Correctness (1:1 golden-vector)

### 1.1 `cargo test` 全量
- 命令: `/Users/tony/.cargo/bin/cargo test`
- 退出码: **0**
- 总计: **308 项测试, 0 失败**, 跨 **22 个测试二进制**:
  - `src/lib.rs` 单元测试 (45，含核心原语 `*_matches_naive` 零偏差护栏)
  - `examples/demo.rs` (0)
  - 指标测试: cycle(7), math_ops(11), math_trans(15), momentum(29), overlap(9)+overlap_new(16), price_transform(5), sma(1), stat(9), volatility(3), volume(3)
  - **8 个 CDL 模式测试文件**: pattern_test(9) + pattern_batch2..8 (16+16+16+21+21+21+24) = **144 项**
  - 文档测试 Doc-tests (21)
- 编译警告: **0**；错误: **0**

### 1.2 `tools/reconcile.py` 函数对账
- 命令: `/opt/homebrew/bin/python3 tools/reconcile.py`
- 退出码: **0**
- 结论: **161/161** 对外函数 1:1 对应 TA-Lib 0.7.1（9 大组全覆盖；live 交叉校验 mismatches=0）

> ✅ **161 个对外函数全部通过黄金向量**（ADR 0005 容差 rel 1e-8 + abs 1e-10），零偏差、0 失败。

---

## 2. 性能对照表 · Rust vs Native C

**环境**: Apple Silicon aarch64, macOS · N=1_000_000, PERIOD=20, ITERS=20 · `ns/elem = elapsed / ITERS / N`。
**Rust 列** = 本次 **canonical 实测**（受管 cargo，无 native）。**C 列** = `cargo bench --features bench-c` FFI 实测。
**状态阈值**: ratio < 0.8 → faster · 0.8–1.2 → ≈ at parity · > 1.2 → slower (>20%)。

| Indicator | Rust ns/elem | Native C ns/elem | Rust/C ratio | Status |
|-----------|-------------:|-----------------:|-------------:|--------|
| SMA      | 1.18 | 1.92  | 0.61 | faster than C |
| BBANDS   | 3.02 | 5.20  | 0.58 | faster than C |
| DEMA     | 3.63 | 4.85  | 0.75 | faster than C |
| TEMA     | 3.46 | 7.44  | 0.47 | faster than C |
| T3       | 3.76 | 2.78  | 1.35 | slower than C (>20%) |
| MIDPRICE | 7.30 | 12.25 | 0.60 | faster than C |
| MIDPOINT | 6.88 | 3.05  | 2.26 | slower than C (>20%) |
| WMA      | 2.11 | 2.28  | 0.93 | ≈ at parity |
| LINEARREG| 2.33 | N/A   | N/A  | C comparison N/A (not wired) |
| CORREL   | 4.81 | N/A   | N/A  | C comparison N/A (not wired) |
| WILLR    | 7.90 | N/A   | N/A  | C comparison N/A (not wired) |
| STOCH    | 10.99| N/A   | N/A  | C comparison N/A (not wired) |

**注记**:
- LINREG/CORREL/WILLR/STOCH 的 C 对照**未接线**（`bench-c` 仅覆盖原始 8 个；新函数接原生需 `unsafe`/系统 TA-Lib C，超出零-FFI 精神，按 `perf-impl-summary` "Rust 侧即可并注明"）。
- STOCHF 复用 `stoch_fastk`（与 STOCH 同一热路径），ns/elem ≈ STOCH，未单列 bench。
- SMA 的 Rust/C bench 均不打印 `ns/elem`，由 elapsed 推算（Rust 23.62075ms → 1.18；C 38.482792ms → 1.92）。
- `_with_output` 原生变体额外实测（免中间 `Vec`，更快且与公开 API 位级一致）: CORREL 4.65、LINEARREG(_with_output) 2.21、STOCH 10.39、WILLR 7.60。

### 2.1 Python 绑定对照（量级参考，非原生 C 口径）
来自 `/opt/homebrew/bin/python3 tools/bench/compare.py`（同输入）:

| Indicator | Python ns/elem |
|-----------|---------------:|
| SMA | 2.05 |
| DEMA | 4.98 |
| TEMA | 7.33 |
| T3 | 2.84 |
| WMA | 2.31 |
| MIDPOINT | 2.16 |
| MIDPRICE | 12.59 |
| BBANDS | 5.67 |

---

## 3. 优化函数 before→after 加速

> ⚠️ **数字归属标注**: **[Yan]** = 本次 canonical 实测（受管 cargo，无 native）；**[Kou]** = 工程师 `perf-impl-summary.md` 实测（带 `-C target-cpu=native`，非 canonical 口径）。

| 任务 | 函数 | 技术 | before | after ([Yan] canonical) | after ([Kou] native) | 加速 |
|------|------|------|--------|------------------------|---------------------|------|
| T01 | BBANDS (SMA 中轨) | 单遍 `rolling_mean_var` 融合 | 4.56 [Kou] (stash 前) | **3.02** [Yan] | 2.81 [Kou] | ~1.5–1.6× |
| T02 | LINREG 家族 (linear_reg/_angle/_intercept/_slope/tsf) | O(n) 滑动 sy+sxy | O(n·period) 朴素（无严格 before，见注） | **2.33** [Yan] | 2.33 [Kou] | 渐近 ~20× |
| T03 | CORREL | O(n) 滑动 s0/s1+s00/s11/s01 | O(n·period) 朴素 | **4.81** [Yan] | 4.61 [Kou] | 渐近 ~20× |
| T04 | WILLR | 单调队列 rolling_max/min (O(n)) | O(n·period) 朴素 | **7.90** [Yan] | 7.83 [Kou] | 渐近 ~20× |
| T04 | STOCH/STOCHF | 复用 stoch_fastk 极值队列 (O(n)) | O(n·period) 朴素 | **10.99** [Yan] | 10.71 [Kou] | 渐近 ~20× |

**注**: LINREG/CORREL/WILLR/STOCH 的 "before" 为原 O(n·period) 朴素窗口扫描；bare HEAD 不可独立编译（依赖未提交前序改动），故无严格 before ns/elem，按 ADR 0010 以**渐近 O(n·period)→O(n)** 报告（理论降幅 ≈ period=20，实测因新核常数略低）。我的 canonical 实测 after 与工程师 native 实测高度一致（差异 <5%，符合点测 ±5% 波动）。

---

## 4. 残差缺口与诚实说明 · Residual Gaps

- **MIDPOINT 2.26× (≈ BASELINE 2.08×) — 慢于 C**: 单调双队列 (`rolling_extreme`/`rolling_minmax`) 非向量化友好；TA-Lib C 的 `TA_MIDPOINT` 同样用单遍双队列 `MINMAXINDEX`、无 SIMD。P3 SIMD = **NO-GO**（ADR 0010 闸门）。已知权衡，非缺陷。
- **T3 1.35× (≈ BASELINE 1.40×) — 慢于 C (>20%)**: 顺序 EMA 递推 (`nested_ema_with_output`) 非向量化友好；P3 SIMD = **NO-GO**。P3 评估候选，预计维持现状。
- **CCI / AVGDEV 不可分离**: `avgdev = mean(|x−mean|)` 为非可分离统计，无法用滑动求和原语，故不在 O(n) 滑动家族内；已通过黄金向量验证正确性，无 ns/elem 基准（亦非优化对象）。
- **61 个 CDL 模式函数**: 整数输出 O(n) 蜡烛比较，手写即快、几乎不依赖重原语，超出 ns/elem 范畴；正确性护栏为黄金向量 1:1（144 项模式测试全绿），非性能基准。
- **文档不一致（非源码缺陷）**: `benches/BASELINE.md` 的 BBANDS "Rust P2 = 5.61 (Δ 1.00×)" 与本次 canonical 实测 3.02 及 T01 融合（Kou: 2.81）矛盾，疑似 pre-T01 旧值，建议刷新该表行；WMA/MIDPRICE/MIDPOINT/T3 的 P2 值与本次实测在 ±5% 点测波动内吻合。
- **SMA C 对照 checksum=0（测试脚手架小瑕疵）**: `sma_bench` 的 C 侧取 `out[out.len()−1]` 而非 `out[nb−1]`（TA-Lib 紧凑输出），导致 checksum 恒为 0；**不影响计时**（计时循环仍跑满 20 次 TA_SMA）。建议改为 `out[nb−1]` 以与 BASELINE 方法论一致。

---

## 5. 结论 · Conclusion

- **正确性**: `cargo test` **308/308 通过，0 失败**；`reconcile.py` **161/161，exit 0**。161 函数 1:1 对齐 TA-Lib 0.7.1 黄金向量，零偏差。
- **性能**: 原始 8 项中 6 项快于/持平 C（SMA/BBANDS/DEMA/TEMA/MIDPRICE 快于 C，WMA ≈ 持平）；仅 MIDPOINT (2.26×)、T3 (1.35×) 慢于 C，均因 SIMD NO-GO 的结构性限制，属已知权衡。
- **新优化**（BBANDS 融合 + LINREG/CORREL/WILLR/STOCH O(n) 滑动）全部落地且零偏差，canonical 实测与工程师 native 实测一致。
- **无源码缺陷，无需路由工程师。** 仅 2 处测试/文档小瑕疵（SMA C checksum、BASELINE BBANDS 旧值）建议后续清理。
