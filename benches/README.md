# benches/ — 性能基准（见 ADR 0004 双轨）

## Rust 侧（默认，零依赖）
```
cargo bench --bench sma_bench
```
依赖-free，使用 `std::time` 计时，直接运行 `fn main()`（`harness = false`）。

## 原生 C 对照（可选 feature）
```
cargo bench --bench sma_bench --features bench-c
```
仅在启用 `bench-c` 时，通过 FFI 链接系统 TA-Lib C 库（`libta_lib`）做原生对照。
**需要系统已安装 TA-Lib C 库**；未启用该 feature 时构建不受影响（`build.rs` 仅在
`bench-c` 下链接）。报告须明确区分两种口径，不得将 Python/绑定层计时谎称为原生 C。
