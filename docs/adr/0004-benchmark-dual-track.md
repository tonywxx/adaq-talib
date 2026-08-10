# ADR 0004: 基准测速采用双轨（Rust FFI 对照原生 C + Python 便捷对照）

- 状态：已采纳（2026-08-09）
- 决策人：用户 + WorkBuddy（grilling 会话）

## 背景

要求"自研 Rust 实现与原生 TA-Lib 运行速度对照测试"；但主库要求 Zero-FFI，不能将 C 库编入发布产物。

## 决策

- **权威对照（可进 CI）**：Rust `benches/` 在可选 feature（如 `feature = "bench-c"`）下 FFI 链接 TA-Lib C 库，测量 Rust vs **原生 C** 的真实速度差。
- **便捷对照（用户本地）**：Python 脚本（`tools/bench/`）调用 TA-Lib Python 绑定计时，供用户本地对照；报告**须如实标注"vs TA-Lib Python binding"**，不得声称"vs native C"。
- Python 脚本可读取 `cargo bench` 输出与 C 侧计时，汇总生成对照报告。

## 权衡

- 优点：主库保持纯 Rust 零依赖；既有权威的原生 C 对照，又有用户友好的便捷对照。
- 缺点：维护两套基准；Python 绑定层计时不代表原生 C 性能，必须在报告中明确标注口径。
- 难以回退：基准协议与口径约定需跨版本保持一致，避免前后数字不可比。

## 影响

- `benches/` 与 `tools/bench/` 分离；`bench-c` feature 默认关闭，发布产物不含 C 链接。
- 报告模板需显式区分两种对照口径。
