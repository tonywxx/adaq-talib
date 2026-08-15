//! 指标脚手架接缝（Indicator Scaffold Seam，架构评审候选①深化）。
//!
//! Indicator-scaffold seam (candidate-① deepening).
//!
//! 把每个指标函数三件套里重复的「等长 `f64::NAN` 缓冲分配 + 转发到 `*_with_output` 内核」
//! 集中到一个 **零成本** `macro_rules!` 宏。每个指标只声明其 `func` 签名与对应的
//! `*_with_output` 内核名称；逐元素热路径（`*_with_output` 体）保持手写、字节级不变。
//!
//! 设计取舍（对照项目双重基准）：
//! - **局部性**：不稳定期填充 / 长度校验逻辑只存在一处；
//! - **杠杆**：修一次 unstable-period 规则即覆盖全部接入指标（见 ADR 0007）；
//! - **零性能退化**：宏在编译期展开为与手写为完全一致的代码——无 `dyn Fn`、无间接调用、
//!   无每轮分配。满足 ADR 0010 与「度量前置协议」（A/B Δ≤±5% + 黄金向量 1:1，见 ADR 0003 / 0005）。
//!
//! 形态约定（被宏假定，不得违反）：
//! - 每个指标函数的**第一个参数必为 `&[f64]` 主序列**，其长度即输出长度；
//! - `*_with_output` 的签名为 `(主序列, …其他参数…, out: &mut [f64])`；
//! - 若带默认参数，该参数必须位于 `fn` 参数表**末尾**（宏在 `default` 臂将其替换为给定常量）。
//!
//! 路线图（本文件仅实现 Phase 1a/1b 单输出臂；多输入 `check_eq_len` 臂与多输出 struct 臂
//! 属阶段二，沿用同一接缝理念，不在候选①首轮范围）：
//! - **Phase 1a**（已落地试点）：单输入、单输出、`*_with_output` 仅含主序列——见 `math_trans`。
//! - **Phase 1b**：单输入、单输出、带默认参数——`default` 臂，见 `stat` 单输入子集。
//! - **Phase 1c**：其余单输出函数（overlap / momentum / math_ops / price_transform / volume / …）。
//! - **Phase 2**：多输入臂（`check_eq_len` + 按首输入定长）、多输出 struct 臂。

/// 指标函数脚手架宏。
///
/// **不带默认参数（Phase 1a 试点形态）：**
/// ```text
/// indicator! {
///     /// 文档注释
///     fn ln(values: &[f64]) -> Vec<f64> with ln_with_output;
/// }
/// ```
/// 展开为：
/// ```text
/// pub fn ln(values: &[f64]) -> Result<Vec<f64>, crate::error::TaError> {
///     let mut out = vec![f64::NAN; values.len()];
///     ln_with_output(values, &mut out)?;
///     Ok(out)
/// }
/// ```
///
/// **带默认参数（Phase 1b `stat` 单输入子集形态）：** 在 `with` 子句后追加
/// `default <默认函数名>(<透传参数>) => (<默认常量 1>, <默认常量 2>, …)`，
/// 末尾默认参数支持 1 个或多个：
/// ```text
/// indicator! {
///     /// 文档注释
///     fn stddev(values: &[f64], time_period: usize, nb_dev: f64) -> Vec<f64> with stddev_with_output
///     default stddev_default(values: &[f64]) => (STDDEV_PERIOD, STDDEV_NB_DEV)
///     /// 默认函数文档（可选，写在 `=>` 之后、`;` 之前）
///     ;
/// }
/// ```
/// 默认参数必须位于 `fn` 参数表**末尾**（宏将其逐个替换为给定常量）；支持 1 个或多个末尾默认参数。
macro_rules! indicator {
    // —— 带默认参数臂（Phase 1b）：支持末尾 1 个或多个默认参数 ——
    (
        $(#[$meta:meta])*
        fn $fname:ident ( $len:ident : &[f64] $(, $arg:ident : $argty:ty)* $(,)? )
            -> Vec<f64>
        with $with_output:ident
        default $dname:ident ( $($darg:ident : $dargty:ty),* $(,)? )
            => ( $($def:expr),+ $(,)? )
        $(#[$dmeta:meta])*
        $(;)?
    ) => {
        $(#[$meta])*
        pub fn $fname ( $len : &[f64] $(, $arg : $argty)* )
            -> Result<Vec<f64>, $crate::error::TaError>
        {
            let mut out = vec![f64::NAN; $len.len()];
            $with_output ( $len $(, $arg)* , &mut out )?;
            Ok(out)
        }
        $(#[$dmeta])*
        pub fn $dname ( $($darg : $dargty),* )
            -> Result<Vec<f64>, $crate::error::TaError>
        {
            $fname ( $($darg),* , $($def),* )
        }
    };

    // —— 不带默认参数臂（Phase 1a 试点）——
    (
        $(#[$meta:meta])*
        fn $fname:ident ( $len:ident : &[f64] $(, $arg:ident : $argty:ty)* $(,)? )
            -> Vec<f64>
        with $with_output:ident
        $(;)?
    ) => {
        $(#[$meta])*
        pub fn $fname ( $len : &[f64] $(, $arg : $argty)* )
            -> Result<Vec<f64>, $crate::error::TaError>
        {
            let mut out = vec![f64::NAN; $len.len()];
            $with_output ( $len $(, $arg)* , &mut out )?;
            Ok(out)
        }
    };

    // —— 多输入臂 · 0-init（Phase 2 启动，候选①）—— 用于无不稳定期、前导无 NaN 的多输入指标，
    //    e.g. 蜡烛形态 cdl_*（open/high/low/close，首片定长）。首片 `&[f64]` 即输出长度源；
    //    其余参数按 `ident : ty` 透传（可为更多 `&[f64]` 切片或末尾标量）；内核内部已做长度/OHLC
    //    校验时此臂不重复校验，保证与手写字节级一致（见 Q3）。
    (
        $(#[$meta:meta])*
        fn $fname:ident (
            $first:ident : &[f64]
            $(, $rest:ident : $restty:ty)* $(,)?
        ) -> Vec<f64>
        with $with_output:ident
        init zero
        $(;)?
    ) => {
        $(#[$meta])*
        pub fn $fname (
            $first : &[f64]
            $(, $rest : $restty)*
        ) -> Result<Vec<f64>, $crate::error::TaError> {
            let mut out = vec![0.0_f64; $first.len()];
            $with_output ( $first $(, $rest)* , &mut out )?;
            Ok(out)
        }
    };

    // —— 多输入臂 · NAN 默认（Phase 2 启动，候选①）—— 不含 `init` 修饰时以 f64::NAN 初始化
    //    （含前导不稳定期）。供后续 OHLC 数值指标（如 cci/willr/±di 等）接入；本论 cdl_* 用上一条。
    (
        $(#[$meta:meta])*
        fn $fname:ident (
            $first:ident : &[f64]
            $(, $rest:ident : $restty:ty)* $(,)?
        ) -> Vec<f64>
        with $with_output:ident
        $(;)?
    ) => {
        $(#[$meta])*
        pub fn $fname (
            $first : &[f64]
            $(, $rest : $restty)*
        ) -> Result<Vec<f64>, $crate::error::TaError> {
            let mut out = vec![f64::NAN; $first.len()];
            $with_output ( $first $(, $rest)* , &mut out )?;
            Ok(out)
        }
    };
}

// `pub(crate)` 重导出，使各指标模块可经 `use crate::indicator::indicator;` 引入本宏。
pub(crate) use indicator;
