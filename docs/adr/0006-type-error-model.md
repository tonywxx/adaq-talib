# ADR 0006: 数值类型与错误模型

- 状态：已采纳（2026-08-09）
- 决策人：用户 + WorkBuddy（grilling 会话）

## 背景

需确定公共 API 的数值类型与错误返回方式，以匹配 TA-Lib 0.7.1（`double` + `TA_RetCode`），同时满足 Rust 规约（见 ADR 0001 模型 B）。

## 决策

- 数值类型全程使用 `f64`（对应 TA-Lib `double`）。不考虑泛型浮点。
- 错误模型采用 `Result<T, TaError>`，其中 `TaError` 为错误枚举，语义映射 `TA_RetCode`（`BadParam`、`OutOfRange`、`LibNotInitialized`、`OutOfMemory` 等）。不采用返回裸 `RetCode` 的 C 风格。

## 权衡

- 优点：类型安全、符合 Rust 习惯、可组合；错误语义清晰。
- 缺点：与 C 调用方不直接兼容（已由 ADR 0001 模型 B 决定，可由 `sys` 层补足）。
- 难以回退：公开错误类型是 API 契约的一部分，变更破坏 SemVer。

## 影响

- 定义 `pub enum TaError` 及显示实现；文档中给出各变体触发条件。
- 可选入参的"默认便捷函数"约定见 `docs/api-conventions.md`。
