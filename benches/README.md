# benches/ — 性能基准（见 ADR 0004 双轨）

## Rust 侧（默认，零依赖）
```
cargo bench
```
依赖-free，使用 `std::time` 计时（`harness = false`，各 bench 自行 `fn main()`）。
当前覆盖（热路径，P2 优化重点）：

- `sma_bench` — 基线（单遍滚动均值）
- `bbands_bench` — 布林带（SMA + 总体标准差）
- `dema_bench` / `tema_bench` / `t3_bench` — 嵌套 EMA 族
- `wma_bench` — 加权移动平均
- `midprice_bench` — MIDPRICE + MIDPOINT（滚动极值）

基线数值见 `benches/BASELINE.md`（Rust vs 原生 C 的 ns/elem 对照，作为回归护栏）。

## 原生 C 对照（可选 feature `bench-c`）
```
cargo bench --features bench-c
```
启用 `bench-c` 时，bench 二进制通过 FFI 链接系统 **TA-Lib C 0.7.1** 库做原生对照，
报告同时打印 `Rust …` 与 `C … (native)` 两套 ns/elem。

### 链接说明（重要）
- 链接由 `build.rs` 在 `bench-c` 下自动完成：探测 `libta-lib` / `libta_lib`
  （macOS Homebrew 命名为 `ta-lib` → `libta-lib.dylib`；其它平台多为 `ta_lib`），
  并把所在目录加入链接搜索路径。**普通构建/测试不启用 `bench-c`，完全零 C 依赖（Zero-FFI 不变）。**
- 若库不在默认搜索路径，设环境变量指定其目录：
  ```
  TA_LIB_LIB_DIR=/path/to/lib cargo bench --features bench-c
  ```
- `bench-c` 仅影响 bench 二进制；发布构建（`cargo build` / `cargo test`，不带该 feature）
  不会链接任何 C 库。

## Python 绑定对照（口径参考，非原生 C）
```
python3 tools/bench/compare.py
```
经 `talib` Python 绑定计时（底层同为 TA-Lib C 0.7.1，但含 CPython↔C FFI 与 ndarray 拷贝开销），
**不等同于原生 C**，仅作量级参考（见 `tools/bench/compare.py` 顶部说明）。
