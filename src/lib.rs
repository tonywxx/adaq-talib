//! # adaq-talib
//!
//! 纯 Rust、Zero-FFI、零依赖的 [TA-Lib](https://ta-lib.org) 0.7.1 技术指标复刻库。
//!
//! Pure-Rust, zero-FFI, dependency-free reimplementation of the
//! [TA-Lib](https://ta-lib.org) 0.7.1 technical-analysis indicators.
//!
//! ## 设计目标 / Design goals
//!
//! - **零偏差 (zero-deviation)**：每个函数的数值输出与原版 TA-Lib 0.7.1 逐项一致
//!   （在浮点误差容限内，见 [`crate::utils`] 与 ADR 0005）。
//! - **Zero-FFI / No-Dependencies**：发布的库不调用任何 C ABI，`[dependencies]` 为空，
//!   全部算法原生手写。
//! - **惯用 Rust API (模型 B)**：切片入参、`Result<_, TaError>` 出参、多输出用结构体返回
//!   （见 ADR 0001）。前导不稳定期以 [`f64::NAN`] 填充、等长返回（见 ADR 0007）。
//! - **性能优先**：在内存布局、循环分支、数组运算层面做优化。
//!
//! ## 覆盖范围 / Coverage
//!
//! 采用里程碑式发布（见 ADR 0002）：`0.1.0` 覆盖重叠研究、动量、波动率、成交量、价格变换
//! 与统计类；数学变换（Math Transform）、数学运算符（Math Operators）与剩余周期/模式识别
//! 指标在后续里程碑补齐，最终全量且不删减任何原版能力（见 [`docs/0.1.0-scope.md`](docs/0.1.0-scope.md)
//! 与 `docs/NEXT-ACTIONS-perf.md` 的 P4 进度）。
//!
//! ## 快速开始 / Quick start
//!
//! ```rust
//! use adaq_talib::overlap::sma;
//!
//! let prices = [1.0, 2.0, 3.0, 4.0, 5.0];
//! let out = sma(&prices, 3).unwrap();
//! // out 与 prices 等长；前导 2 个位置为 NaN，其余为窗口均值。
//! // `out` has the same length as `prices`; the first 2 positions are NaN, the rest are window means.
//! assert!(out[0].is_nan());
//! assert!((out[2] - 2.0).abs() < 1e-9);
//! ```
//!
//! ## 验证与基准 / Verification & benchmarks
//!
//! - 正确性：与入库黄金向量（由 TA-Lib 原版生成，`tools/gen_fixtures`）比对，普通
//!   `cargo test` 无需 Python（见 ADR 0003）。
//! - 性能：双轨基准 —— `benches/` 可选 `bench-c` feature FFI 对照原生 C，
//!   `tools/bench` 提供 Python 便捷对照（如实标注为 TA-Lib Python 绑定层，见 ADR 0004）。

pub mod error;
pub mod momentum;
pub mod overlap;
pub mod price_transform;
pub mod stat;
pub mod volume;
pub mod volatility;
pub mod cycle;
pub mod math_ops;
pub mod math_trans;
pub mod pattern;

pub(crate) mod core;
// 暴露为 doc(hidden) 公共模块，便于集成测试与 benches 复用 approx 工具（内部实现细节）。
// Exposed as a doc(hidden) public module so integration tests and benches can reuse the
// approx helpers; it is an internal implementation detail, not part of the public API.
#[doc(hidden)]
pub mod utils;

pub use error::TaError;
