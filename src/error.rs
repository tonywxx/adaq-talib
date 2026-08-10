//! 错误类型定义 / Error type definitions.
//!
//! `TaError` 语义映射 TA-Lib 的 `TA_RetCode`（见 TA-Lib `ta_defs.h`）。
//! `TaError` is the semantic mapping of TA-Lib's `TA_RetCode` (see TA-Lib `ta_defs.h`).

use std::fmt;

/// 本库公开错误类型，对应 TA-Lib `TA_RetCode` 的语义。
///
/// Public error type, corresponding to the semantics of TA-Lib `TA_RetCode`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaError {
    /// 非法参数（对应 `TA_BAD_PARAM`）。例如时间周期 <= 0。
    /// Invalid parameter (maps to `TA_BAD_PARAM`), e.g. time period <= 0.
    BadParam(String),

    /// 参数越界（对应 `TA_OUT_OF_RANGE`）。
    /// Parameter out of range (maps to `TA_OUT_OF_RANGE`).
    OutOfRange(String),

    /// 库未初始化（对应 `TA_LIB_NOT_INITIALIZED`）。
    /// Library not initialized (maps to `TA_LIB_NOT_INITIALIZED`).
    LibNotInitialized,

    /// 内存分配失败（对应 `TA_ALLOC_ERR`）。
    /// Memory allocation failure (maps to `TA_ALLOC_ERR`).
    OutOfMemory,

    /// 内部错误（对应 `TA_INTERNAL_ERROR`）。
    /// Internal error (maps to `TA_INTERNAL_ERROR`).
    InternalError(String),
}

impl fmt::Display for TaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaError::BadParam(s) => write!(f, "Bad parameter: {s}"),
            TaError::OutOfRange(s) => write!(f, "Out of range: {s}"),
            TaError::LibNotInitialized => write!(f, "Library not initialized"),
            TaError::OutOfMemory => write!(f, "Out of memory"),
            TaError::InternalError(s) => write!(f, "Internal error: {s}"),
        }
    }
}

impl std::error::Error for TaError {}

/// 将周期参数转换为 `Result`，非法则返回 [`TaError::BadParam`]。
///
/// Convert a period parameter into a `Result`; returns [`TaError::BadParam`] when invalid.
#[inline]
pub(crate) fn check_period(period: usize) -> Result<usize, TaError> {
    if period == 0 {
        return Err(TaError::BadParam("time period must be >= 1".into()));
    }
    Ok(period)
}
