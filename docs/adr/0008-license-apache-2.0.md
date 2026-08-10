# ADR 0008: 开源许可证采用 Apache-2.0

- 状态：已采纳（2026-08-09）
- 决策人：用户 + WorkBuddy（grilling 会话）

## 背景

crates.io 发布需明确 `license` 字段；需选择单许可方案。

## 决策

采用 **`Apache-2.0`**（单一许可，非双许可 MIT OR Apache-2.0）。

## 权衡

- 优点：显式专利授权，对企业友好；与 Rust 基金会生态兼容。
- 缺点：相比 MIT 文本更长；未采用双许可，少数偏好 MIT 的贡献者需接受 Apache-2.0。
- 难以回退：license 字段变更对使用者有法律含义，应在发布前定稿。

## 影响

- `Cargo.toml`：`license = "Apache-2.0"`。
- 仓库 `LICENSE` 文件已为 Apache-2.0 全文（已核对，无需替换）。
