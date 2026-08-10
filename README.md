# adaq-talib

**AdaQ-TAlib** — 纯 Rust、Zero-FFI、零依赖的 [TA-Lib](https://ta-lib.org) 0.7.1 技术指标复刻库。

Pure-Rust, zero-FFI, dependency-free reimplementation of the
[TA-Lib](https://ta-lib.org) 0.7.1 technical-analysis indicators.

## 特性 / Features

- **零偏差 (zero-deviation)**：每个函数的数值输出与原版 TA-Lib 0.7.1 逐项一致（浮点容限见下）。
- **Zero-FFI / No-Dependencies**：发布的库不调用任何 C ABI，`Cargo.toml` 的 `[dependencies]` 为空，全部算法原生手写。
- **惯用 Rust API (模型 B)**：切片入参、`Result<_, TaError>` 出参、多输出用结构体返回；前导不稳定期以 `NaN` 填充、等长返回。
- **性能优先**：在内存布局、循环分支、数组运算层面做优化。

## 覆盖范围 / Coverage

里程碑式发布（见 [`docs/adr/0002-release-scope-milestones.md`](docs/adr/0002-release-scope-milestones.md)）：

- `0.1.0`：重叠研究 + 动量 + 波动率 + 成交量（约 70 个函数）。
- 后续版本：模式识别（蜡烛形态）、数学类、周期等补齐；**最终全量且不删减任何原版能力**。

当前已实现 / Currently implemented：`overlap::sma`。

## 快速开始 / Quick start

```rust
use adaq_talib::overlap::sma;

let prices = [1.0, 2.0, 3.0, 4.0, 5.0];
let out = sma(&prices, 3).unwrap();
// out 与 prices 等长；前导 2 个位置为 NaN，其余为窗口均值。
assert!(out[0].is_nan());
assert!((out[2] - 2.0).abs() < 1e-9);
```

运行交互式示例 / Run the interactive demo:

```text
cargo run -- sma
```

## 验证与基准 / Verification & benchmarks

- **正确性**：与入库黄金向量（由 TA-Lib 原版生成，`tools/gen_fixtures`）比对，普通
  `cargo test` 无需 Python（见 [`docs/adr/0003-verification-golden-fixtures.md`](docs/adr/0003-verification-golden-fixtures.md)）。
  容限策略：相对 `1e-8` + 绝对 `1e-10`（见 [`docs/adr/0005-error-tolerance.md`](docs/adr/0005-error-tolerance.md)）。
- **性能**：双轨基准（见 [`docs/adr/0004-benchmark-dual-track.md`](docs/adr/0004-benchmark-dual-track.md)）。
  - 权威对照（原生 C）：`cargo bench --bench sma_bench --features bench-c`（需系统安装 TA-Lib C 库）。
  - 便捷对照（Python 绑定层）：`python tools/bench/run.py`（如实标注为 TA-Lib Python binding）。

## 文档 / Documentation

- 设计决策见 [`docs/adr/`](docs/adr/)（ADR 0001–0009）。
- 统一 API 写法见 [`docs/api-conventions.md`](docs/api-conventions.md)。
- 术语表见 [`CONTEXT.md`](CONTEXT.md)。

## 许可证 / License

Apache-2.0（见 [`LICENSE`](LICENSE)）。
