# ADR 0003: 数值验证采用入库黄金向量（Python/TA-Lib 生成，开发工具化）

- 状态：已采纳（2026-08-09）
- 决策人：用户 + WorkBuddy（grilling 会话）

## 背景

测试要求输出值与原版 TA-Lib 0.7.1 结果做误差比对校验；但主库要求 Zero-FFI、No-Dependencies，不能链接 C 库。

## 决策

- 用 Python 脚本（依赖系统已装的 TA-Lib C 库 + PyPI 包 `TA-Lib`）一次性生成**黄金向量**（标准化输入数据集 + 期望输出），以 fixture（`.json`/`.bin`）入库于 `tests/fixtures/`。
- 生成器置于 `tools/gen_fixtures/`，属开发工具，**非 crate 依赖**。
- 普通用户执行 `cargo test` 直接比对 fixture，**零 Python 依赖**。
- 生成端所用 TA-Lib C 库版本**固定并登记于 `tools/README`**，确保"0.7.1"口径一致。
- 误差容限策略见 ADR 0005。

## 权衡

- 优点：守住 Zero-FFI / No-Dependencies；验证可复现，CI 无需 C 环境。
- 缺点：fixture 需随算法修正 / 版本对齐维护；生成环节依赖 Python + TA-Lib C（仅开发者侧）。
- 难以回退：fixture 一旦入库即成为比对基准，口径变更需重生成并评审。

## 影响

- `tests/` 下建立 fixture 加载与比对基础设施。
- `tools/` 目录纳入仓库，与 `src/` 分离，不进入发布产物。
