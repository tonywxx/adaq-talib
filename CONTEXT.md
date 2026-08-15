# CONTEXT.md — adaq-talib 领域术语表

> 单一上下文词汇表。仅记录术语定义，不含实现细节或算法描述。

## 术语

- **adaq-talib**：本项目名。目标为 TA-Lib 0.7.1 的纯 Rust、Zero-FFI、No-Dependencies 复刻库，已发布至 crates.io。<https://crates.io/crates/adaq-talib>
- **TA-Lib 0.7.1**：数值对照基准，原版 C 实现，约 200 个技术指标函数，按类别分为重叠研究、动量、成交量、波动率、价格变换、周期、模式识别（蜡烛形态）、统计、数学运算、数学变换。
- **Zero-FFI**：发布的库不调用任何 C ABI，不包含对 TA-Lib C 库的链接。
- **No-Dependencies**：`Cargo.toml` 的 `[dependencies]` 为空，全部算法原生手写，不引入任何外部 crates。
- **零偏差 (zero-deviation)**：数值输出与 TA-Lib 0.7.1 逐项一致（在浮点误差容限 ε 内，见 ADR 0005）。
- **API 保真度模型 B**：公共 API 采用惯用 Rust 形态（切片 `&[f64]` 入参、`Result<Output, Error>` 出参、多输出以结构体/元组返回），内部算法与数值与原版 1:1 一致。
- **黄金向量 (golden vectors)**：由 TA-Lib 原版生成的参考输出，作为测试比对基准，以 fixture 形式入库（见 ADR 0003）。
- **里程碑式发布**：规划上 0.1.0 先覆盖高频核心指标、后续补齐，最终全量覆盖且不删减任何原版能力（见 ADR 0002）；实际执行中 0.1.0 即完成全量 **161 个对外函数**（TA-Lib 0.7.1 完整公开面）的 1:1 验证，等同「全量覆盖」，不再分 65→96 阶段发布（见 ADR 0002 补充记录，2026-08-10）。
- **不稳定期 (unstable period)**：指标预热阶段，前导若干输出尚未稳定或未被计算；Rust 侧以 `f64::NAN` 填充前导，等长返回（见 ADR 0007）。
- **out_beg_idx**：TA-Lib 输出起始索引，即从该位置起输出有效；前导 `out_beg_idx` 个位置未计算。
- **TaError**：本库公开错误枚举，语义映射 TA-Lib `TA_RetCode`（`BadParam` / `OutOfRange` / `LibNotInitialized` / `OutOfMemory` 等），见 ADR 0006。
- **TA-Lib Python 绑定**：PyPI 包 `TA-Lib`（导入名 `talib`），为 TA-Lib C 库的 ctypes/numpy 封装，运行需系统已安装 TA-Lib C 库。
- **candle settings**：TA-Lib 模式识别函数依赖的蜡烛部件（实体/上影/下影）涨跌色与允许范围设置；本项目仅采用默认内建值、不暴露配置 API（ADR 0009 已采纳）。
- **单调队列 (monotonic queue)**：O(n) 滚动极值算法，替代朴素 O(n·period) 窗口扫描，用于 MIDPOINT / MIDPRICE 等热路径（见 ADR 0010 D1）。
- **融合单遍核 (fused single-pass kernel)**：将多次独立扫描合并为单次遍历，减少分配与缓存 miss，如 DEMA/TEMA 复用 EMA 状态、BBANDS 合并 middle+sd（见 ADR 0010 D1）。
- **原地写入 (in-place / write-to-buffer)**：指标提供 `*_with_output(&mut [f64])` 变体，由调用方提供输出缓冲，避免每调用分配（见 ADR 0010 D2）。
- **指标脚手架 (indicator scaffold)**：把「分配等长 `f64::NAN` 缓冲 + 调用 `*_with_output` + 包成 `Result<Vec<f64>, TaError>`」这类重复胶水代码，统一抽为编译期 `indicator!` 宏生成的公共 `func` 入口（见 ADR-0011）。热路径 `*_with_output` 体保持手写、字节级不变；宏为零成本文本展开（无 `dyn Fn`、无间接调用、无每轮分配），故数值与性能与手写为 1:1。多输入接缝已定型为两条臂：**NAN 默认臂**（首片 `&[f64]` 即输出长度源 + 其余参数按 `ident : ty` 透传，`vec![f64::NAN]` 填充前导不稳定期，用于 `momentum` 的 `cci`/`mfi`/`willr`/`adx`/`dx`/`imi`、`volume` 的 `adosc`、`overlap` 的 `midprice`/`sar`/`sarext`）+ **0-init 臂**（`with $wo init zero` → `vec![0.0_f64]`，用于无不稳定期的蜡烛形态 `cdl_*`、price_transform 四函数、及 `bop`/`ad`/`obv`，见 ADR-0011 D7）；结构体多输出 arm 仍待做（仅在确有模块需要时扩展）。
- **自动向量化 (autovectorization)**：依赖编译器将标量循环生成为 SIMD 指令，靠 `#[inline]`、消除热循环 bounds-check、数据对齐达成；本项目性能首层手段（见 ADR 0010 D1）。
- **内存对齐 (alignment)**：保证 `f64` 数组按 32/64 字节对齐以提升 SIMD / 缓存效率（见 ADR 0010 D1）。
- **基准双轨 (dual-track benchmark)**：Rust FFI 对照原生 C（`bench-c`，可进 CI）+ Python 便捷对照（`tools/bench`，标注 vs TA-Lib Python binding），见 ADR 0004 / ADR 0010 D4。
- **权威黄金向量 (authoritative golden vectors)**：由真实 TA-Lib 0.7.1 C 库生成的参考输出（区别于当前"对照文档算法的参考值"），作为性能重构的数值 Oracle（见 ADR 0010 D3）。
