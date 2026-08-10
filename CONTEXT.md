# CONTEXT.md — adaq-talib 领域术语表

> 单一上下文词汇表。仅记录术语定义，不含实现细节或算法描述。

## 术语

- **adaq-talib**：本项目名。目标为 TA-Lib 0.7.1 的纯 Rust、Zero-FFI、No-Dependencies 复刻库，计划发布至 crates.io。
- **TA-Lib 0.7.1**：数值对照基准，原版 C 实现，约 200 个技术指标函数，按类别分为重叠研究、动量、成交量、波动率、价格变换、周期、模式识别（蜡烛形态）、统计、数学运算、数学变换。
- **Zero-FFI**：发布的库不调用任何 C ABI，不包含对 TA-Lib C 库的链接。
- **No-Dependencies**：`Cargo.toml` 的 `[dependencies]` 为空，全部算法原生手写，不引入任何外部 crates。
- **零偏差 (zero-deviation)**：数值输出与 TA-Lib 0.7.1 逐项一致（在浮点误差容限 ε 内，见 ADR 0005）。
- **API 保真度模型 B**：公共 API 采用惯用 Rust 形态（切片 `&[f64]` 入参、`Result<Output, Error>` 出参、多输出以结构体/元组返回），内部算法与数值与原版 1:1 一致。
- **黄金向量 (golden vectors)**：由 TA-Lib 原版生成的参考输出，作为测试比对基准，以 fixture 形式入库（见 ADR 0003）。
- **里程碑式发布**：0.1.0 先覆盖高频核心指标，最终全量覆盖；期间不删减任何原版能力（见 ADR 0002）。
- **不稳定期 (unstable period)**：指标预热阶段，前导若干输出尚未稳定或未被计算；Rust 侧以 `f64::NAN` 填充前导，等长返回（见 ADR 0007）。
- **out_beg_idx**：TA-Lib 输出起始索引，即从该位置起输出有效；前导 `out_beg_idx` 个位置未计算。
- **TaError**：本库公开错误枚举，语义映射 TA-Lib `TA_RetCode`（`BadParam` / `OutOfRange` / `LibNotInitialized` / `OutOfMemory` 等），见 ADR 0006。
- **TA-Lib Python 绑定**：PyPI 包 `TA-Lib`（导入名 `talib`），为 TA-Lib C 库的 ctypes/numpy 封装，运行需系统已安装 TA-Lib C 库。
- **candle settings**：TA-Lib 模式识别函数依赖的蜡烛部件（实体/上影/下影）涨跌色与允许范围设置；默认有一组内建值（待 ADR 0009 决议）。
