//! TA-Lib 默认可选参数（optIn*）集中定义。
//!
//! Centralized definitions of TA-Lib default optional inputs (optIn*).
//!
//! 数值取自 TA-Lib 0.7.1 原函数默认值（见 `ta_func.h`）。
//! Values are taken from TA-Lib 0.7.1 per-function defaults (see `ta_func.h`).
//!
//! 每个常量均被对应指标的 `*_default` 入口引用（如 `MACD_FAST/SLOW/SIGNAL` 用于
//! `macd_default`、`BBANDS_PERIOD` 用于 `bbands_default`）。集中放置便于与 TA-Lib
//! `ta_func.h` 的 `optIn*` 默认值逐项对齐，也避免各指标模块内散落魔法数字。
//!
//! Every constant is referenced by its indicator's `*_default` entry point (e.g.
//! `MACD_FAST/SLOW/SIGNAL` by `macd_default`, `BBANDS_PERIOD` by `bbands_default`),
//! or by the test/bench suite. Centralizing them keeps the TA-Lib `ta_func.h` `optIn*`
//! defaults aligned 1:1 and avoids scattered magic numbers across indicator modules.
//!
//! `dead_code` is allowed at module level: a few constants are consumed only by the
//! integration tests / benchmarks (separate crates), so the lib build would otherwise
//! flag them as unused.
#![allow(dead_code)]

/// 默认时间周期（通用重叠/动量指标，对应 `TA_SMA`/`TA_EMA`/`TA_WMA`/`TA_DEMA`/`TA_TEMA` 的 30）。
/// Default time period for generic overlap/momentum indicators (TA-Lib default 30).
pub const DEFAULT_TIME_PERIOD: usize = 30;

/// 默认 RSI 时间周期（TA-Lib 默认 14）。
pub const RSI_PERIOD: usize = 14;
/// 默认 MOM / ROC 时间周期（TA-Lib 默认 10）。
pub const MOM_PERIOD: usize = 10;
/// 默认 ATR / ADX / CCI / WILLR 时间周期（TA-Lib 默认 14）。
pub const ATR_PERIOD: usize = 14;
pub const ADX_PERIOD: usize = 14;
pub const CCI_PERIOD: usize = 14;
pub const WILLR_PERIOD: usize = 14;
/// 默认 DX / IMI 时间周期（TA-Lib 默认 14）。
pub const DX_PERIOD: usize = 14;
pub const IMI_PERIOD: usize = 14;
/// 默认 ACCBANDS 时间周期（TA-Lib 默认 20）。
pub const ACCBANDS_PERIOD: usize = 20;

/// 默认 MACD 快/慢/信号周期（TA-Lib 默认 12 / 26 / 9）。
pub const MACD_FAST: usize = 12;
pub const MACD_SLOW: usize = 26;
pub const MACD_SIGNAL: usize = 9;

/// 默认 BBANDS 时间周期（TA-Lib 默认 20）。
pub const BBANDS_PERIOD: usize = 20;
/// 默认 BBANDS 上下偏离倍数（TA-Lib 默认 2.0 / 2.0）。
pub const BBANDS_NB_DEV_UP: f64 = 2.0;
pub const BBANDS_NB_DEV_DN: f64 = 2.0;

/// 默认 STDDEV 时间周期（TA-Lib 默认 5）与偏离倍数（TA-Lib 默认 1.0）。
pub const STDDEV_PERIOD: usize = 5;
pub const STDDEV_NB_DEV: f64 = 1.0;

/// 默认 STOCH 参数（TA-Lib 默认 fastK=5, slowK=3, slowD=3）。
pub const STOCH_FAST_K: usize = 5;
pub const STOCH_SLOW_K: usize = 3;
pub const STOCH_SLOW_D: usize = 3;

/// 默认 CMO 时间周期（TA-Lib 默认 14）。
pub const CMO_PERIOD: usize = 14;
/// 默认 MFI 时间周期（TA-Lib 默认 14）。
pub const MFI_PERIOD: usize = 14;
/// 默认 ULTOSC 三个时间周期（TA-Lib 默认 7 / 14 / 28）。
pub const ULTOSC_PERIOD1: usize = 7;
pub const ULTOSC_PERIOD2: usize = 14;
pub const ULTOSC_PERIOD3: usize = 28;
/// 默认 AROON 时间周期（TA-Lib 默认 14）。
pub const AROON_PERIOD: usize = 14;
/// 默认 STOCHRSI 的 RSI 周期与窗口周期（TA-Lib 默认 14 / 14）。
pub const STOCHRSI_RSI_PERIOD: usize = 14;
pub const STOCHRSI_PERIOD: usize = 14;
/// 默认 TRIX 时间周期（TA-Lib 默认 30）。
pub const TRIX_PERIOD: usize = 30;
/// 默认 APO / PPO 快/慢周期（复用 MACD 默认 12 / 26）。
pub const APO_FAST: usize = 12;
pub const APO_SLOW: usize = 26;

/// 默认 ADOSC 快/慢周期（TA-Lib 默认 3 / 10）。
pub const ADOSC_FAST: usize = 3;
pub const ADOSC_SLOW: usize = 10;

/// 默认 SAR 加速因子与最大值（TA-Lib 默认 0.02 / 0.2）。
pub const SAR_ACCELERATION: f64 = 0.02;
pub const SAR_MAX: f64 = 0.2;

/// 默认 SAREXT 参数（TA-Lib 默认 startValue=0, offsetOnReverse=0,
/// accelInit=0.02, accel=0.02, accelMax=0.2，多空两侧一致）。
pub const SAREXT_START_VALUE: f64 = 0.0;
pub const SAREXT_OFFSET_ON_REVERSE: f64 = 0.0;
pub const SAREXT_ACCEL_INIT_LONG: f64 = 0.02;
pub const SAREXT_ACCEL_LONG: f64 = 0.02;
pub const SAREXT_ACCEL_MAX_LONG: f64 = 0.2;
pub const SAREXT_ACCEL_INIT_SHORT: f64 = 0.02;
pub const SAREXT_ACCEL_SHORT: f64 = 0.02;
pub const SAREXT_ACCEL_MAX_SHORT: f64 = 0.2;

/// 默认 T3 平滑因子（TA-Lib 默认 0.7）。
pub const T3_VFACTOR: f64 = 0.7;

/// 默认 MAMA 快/慢限制（TA-Lib 默认 0.5 / 0.05）。
pub const MAMA_FAST_LIMIT: f64 = 0.5;
pub const MAMA_SLOW_LIMIT: f64 = 0.05;

/// 默认 MAVP 最小/最大周期（TA-Lib 默认 2 / 30）。
pub const MAVP_MIN_PERIOD: usize = 2;
pub const MAVP_MAX_PERIOD: usize = 30;

/// 默认 LINEARREG 族时间周期（TA-Lib 默认 14）。
pub const LINEARREG_PERIOD: usize = 14;
/// 默认 BETA 时间周期（TA-Lib 默认 5）。
pub const BETA_PERIOD: usize = 5;
/// 默认 CORREL 时间周期（TA-Lib 默认 5）。
pub const CORREL_PERIOD: usize = 5;
