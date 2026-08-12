//! 统计函数（Statistic Functions）。
//!
//! Statistic Functions.
//!
//! 本模块全部函数的数值输出与 [TA-Lib](https://ta-lib.org) 0.7.1 逐项一致（浮点误差容限内，
//! 见 [`crate::utils`] 与 ADR 0005）。前导不稳定期以 [`f64::NAN`] 填充、等长返回（见 ADR 0007）。
//!
//! Every function in this module reproduces the numerical output of TA-Lib 0.7.1 (within the
//! float tolerance in ADR 0005). The leading unstable period is filled with [`f64::NAN`] and
//! returned at equal length (ADR 0007).

use crate::core::defaults::{BETA_PERIOD, CORREL_PERIOD, LINEARREG_PERIOD, STDDEV_NB_DEV, STDDEV_PERIOD};
use crate::core::{check_eq_len, rolling_var};
use crate::error::{check_period, TaError};
use crate::indicator::indicator;

indicator! {
    /// 标准差（Standard Deviation，TA-Lib `TA_STDDEV`）。
    ///
    /// `STDDEV = nb_dev * sqrt(population_variance)`，`population_variance` 为窗口内总体方差
    /// （除以 `period`，见 [`crate::core::rolling_var`]）。前导 `period-1` 个为 [`f64::NAN`]。
    ///
    /// `STDDEV = nb_dev * sqrt(population variance)`. The leading `period - 1` positions are
    /// [`f64::NAN`].
    ///
    /// # 示例 / Example
    /// ```
    /// use adaq_talib::stat::stddev;
    /// let x = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    /// let out = stddev(&x, 8, 1.0).unwrap();
    /// // 总体方差 = 32/8 = 4 -> 标准差 = 2
    /// assert!((out[7] - 2.0).abs() < 1e-9);
    /// ```
    fn stddev(values: &[f64], time_period: usize, nb_dev: f64) -> Vec<f64> with stddev_with_output
    default stddev_default(values: &[f64]) => (STDDEV_PERIOD, STDDEV_NB_DEV)
    /// `stddev` 便捷版本，默认周期 5、偏离倍数 1.0。/ `stddev` with defaults (5, 1.0).
    ;
}

/// 标准差，零拷贝写入 `out`（与 `values` 等长）。见 [`stddev`]。
/// Standard Deviation, written zero-copy into `out`. See [`stddev`].
pub fn stddev_with_output(
    values: &[f64],
    time_period: usize,
    nb_dev: f64,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "stddev_with_output: out length must equal values length".into(),
        ));
    }
    let var = rolling_var(values, time_period);
    let n = values.len();
    for i in 0..n {
        if !var[i].is_nan() {
            out[i] = nb_dev * var[i].sqrt();
        }
    }
    Ok(())
}

indicator! {
    /// 方差（Variance，TA-Lib `TA_VAR`）。
    ///
    /// 窗口内总体方差（除以 `period`，见 [`crate::core::rolling_var`）；`nb_dev` 参数与 TA-Lib
    /// 一致被忽略（TA-Lib `TA_VAR` 不对方差应用偏离倍数）。前导 `period-1` 个为 [`f64::NAN`]。
    ///
    /// Population variance (divide by `period`). Like TA-Lib `TA_VAR`, the `nb_dev` argument is
    /// accepted but **not applied** to the variance (TA-Lib ignores it for `TA_VAR`). The leading
    /// `period - 1` positions are [`f64::NAN`].
    fn var(values: &[f64], time_period: usize, _nb_dev: f64) -> Vec<f64> with var_with_output
    default var_default(values: &[f64]) => (STDDEV_PERIOD, STDDEV_NB_DEV)
    /// `var` 便捷版本，默认周期 5。/ `var` with default period (5).
    ;
}

/// 方差，零拷贝写入 `out`（与 `values` 等长）。见 [`var`]。
/// Variance, written zero-copy into `out`. See [`var`].
pub fn var_with_output(
    values: &[f64],
    time_period: usize,
    _nb_dev: f64,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "var_with_output: out length must equal values length".into(),
        ));
    }
    let temp = rolling_var(values, time_period);
    out.copy_from_slice(&temp);
    Ok(())
}

// ---------------------------------------------------------------------------
// 线性回归族 / Linear Regression: LINEARREG, _ANGLE, _INTERCEPT, _SLOPE, TSF
// ---------------------------------------------------------------------------

/// 线性回归族的内核（最小二乘）。
/// Shared core for the linear-regression family (least squares).
///
/// - `mode = 0`：LINEARREG —— 回归线在窗口右端（位置 `period-1`）的预测值。
/// - `mode = 1`：LINEARREG_ANGLE —— 斜率的反正切（角度，度）。
/// - `mode = 2`：LINEARREG_INTERCEPT —— 截距。
/// - `mode = 3`：LINEARREG_SLOPE —— 斜率。
/// - `mode = 4`：TSF —— 回归线在窗口右端再外推一步（位置 `period`）的预测值。
// 仅被 `mod tests`（滑动 vs 朴素对照）引用；非测试构建中 `linear_reg` 等改走
// `*_with_output`，故标 `#[cfg(test)]` 以免 dead_code 警告。
#[cfg(test)]
fn linreg_core(values: &[f64], period: usize, mode: u8) -> Vec<f64> {
    let mut out = vec![f64::NAN; values.len()];
    linreg_core_with_output(values, period, mode, &mut out);
    out
}

/// 线性回归族内核（最小二乘，O(n) 滑动递推，P2-6，ADR 0010）。
///
/// Shared linear-regression core (least squares, O(n) sliding recurrence, P2-6, ADR 0010).
///
/// 维护滑动窗口和 `sy`（= 朴素窗口求和，`sy[i]=sy[i-1]+x[i]-x[i-period]`）与加权累加
/// `sxy`，以闭式递推 `sxy[i]=sxy[i-1]+period·x[i]-sy[i]`（见 `docs/perf-final-plan.md` 附录 A）
/// 将每窗口 O(period) 降为 O(1)。首个窗口沿用朴素求和作种子，之后 `slope`/`intercept` 的
/// 计算与历史逐项对齐，数值同黄金向量一致（ADR 0005）。
///
/// Maintains the sliding window sum `sy` (`sy[i]=sy[i-1]+x[i]-x[i-period]`) and weighted
/// accumulator `sxy` with the closed-form recurrence `sxy[i]=sxy[i-1]+period·x[i]-sy[i]`
/// (see Appendix A of `docs/perf-final-plan.md`), reducing each window from O(period) to O(1).
/// The first window uses the naïve sum as a seed; the `slope`/`intercept` math stays aligned
/// with the historical impl (1:1 with the golden vector, ADR 0005).
///
/// `mode` 取值同 [`linreg_core`]。/ `mode` is the same as in [`linreg_core`].
fn linreg_core_with_output(values: &[f64], period: usize, mode: u8, out: &mut [f64]) {
    let n = values.len();
    if n < period {
        return;
    }
    let p = period as f64;
    let sx = (period * (period - 1)) as f64 / 2.0; // Σ k, k=0..period-1
    let sxx = (period * (period - 1) * (2 * period - 1)) as f64 / 6.0; // Σ k^2
    let denom = p * sxx - sx * sx;
    // 种子：首个窗口（i = period-1）的朴素窗口和 `sy` 与朴素加权累加 `sxy`。
    // Seed: naïve window sum `sy` and naïve weighted accumulator `sxy` of the first window.
    let mut sy = 0.0_f64;
    let mut sxy = 0.0_f64;
    for k in 0..period {
        let x = values[k];
        sy += x;
        sxy += (k as f64) * x;
    }
    let slope = (p * sxy - sx * sy) / denom;
    let intercept = (sy - slope * sx) / p;
    out[period - 1] = match mode {
        1 => slope.atan() * 180.0 / std::f64::consts::PI, // ANGLE (degrees)
        2 => intercept,                                   // INTERCEPT
        3 => slope,                                       // SLOPE
        4 => intercept + slope * p,                       // TSF (project one step beyond)
        _ => intercept + slope * (p - 1.0),               // LINEARREG (right edge)
    };
    for i in period..n {
        // 滑动递推（与 WMA 同理）：sxy[i]=sxy[i-1]+period·x[i]-sy[i]，sy[i]=sy[i-1]+x[i]-x[i-period]。
        // Sliding recurrence: sxy[i]=sxy[i-1]+period·x[i]-sy[i], sy[i]=sy[i-1]+x[i]-x[i-period].
        sy = sy + values[i] - values[i - period];
        sxy = sxy + (period as f64) * values[i] - sy;
        let slope = (p * sxy - sx * sy) / denom;
        let intercept = (sy - slope * sx) / p;
        out[i] = match mode {
            1 => slope.atan() * 180.0 / std::f64::consts::PI,
            2 => intercept,
            3 => slope,
            4 => intercept + slope * p,
            _ => intercept + slope * (p - 1.0),
        };
    }
}

indicator! {
    /// 线性回归（Linear Regression，TA-Lib `TA_LINEARREG`）。
    ///
    /// 在每个窗口（长度 `period`）上做最小二乘拟合，返回回归线在窗口右端（当前 bar）的预测值。
    /// 前导 `period-1` 个为 [`f64::NAN`]。
    fn linear_reg(values: &[f64], time_period: usize) -> Vec<f64> with linear_reg_with_output
    default linear_reg_default(values: &[f64]) => (LINEARREG_PERIOD)
    /// `linear_reg` 便捷版本，默认周期 14。/ `linear_reg` with default period (14).
    ;
}

indicator! {
    /// 线性回归角度（TA-Lib `TA_LINEARREG_ANGLE`）。
    ///
    /// 返回回归线斜率的反正切，单位为**度**（TA-Lib 约定）。前导 `period-1` 个为 [`f64::NAN`]。
    fn linear_reg_angle(values: &[f64], time_period: usize) -> Vec<f64> with linear_reg_angle_with_output
    default linear_reg_angle_default(values: &[f64]) => (LINEARREG_PERIOD)
    /// `linear_reg_angle` 便捷版本，默认周期 14。/ `linear_reg_angle` with default period (14).
    ;
}

indicator! {
    /// 线性回归截距（TA-Lib `TA_LINEARREG_INTERCEPT`）。前导 `period-1` 个为 [`f64::NAN`]。
    fn linear_reg_intercept(values: &[f64], time_period: usize) -> Vec<f64> with linear_reg_intercept_with_output
    default linear_reg_intercept_default(values: &[f64]) => (LINEARREG_PERIOD)
    /// `linear_reg_intercept` 便捷版本，默认周期 14。
    ;
}

indicator! {
    /// 线性回归斜率（TA-Lib `TA_LINEARREG_SLOPE`）。前导 `period-1` 个为 [`f64::NAN`]。
    fn linear_reg_slope(values: &[f64], time_period: usize) -> Vec<f64> with linear_reg_slope_with_output
    default linear_reg_slope_default(values: &[f64]) => (LINEARREG_PERIOD)
    /// `linear_reg_slope` 便捷版本，默认周期 14。
    ;
}

indicator! {
    /// 时间序列预测（Time Series Forecast，TA-Lib `TA_TSF`）。
    ///
    /// 在窗口上拟合回归线后，外推一步（窗口右端再 +1）得到预测值。前导 `period-1` 个为 [`f64::NAN`]。
    fn tsf(values: &[f64], time_period: usize) -> Vec<f64> with tsf_with_output
    default tsf_default(values: &[f64]) => (LINEARREG_PERIOD)
    /// `tsf` 便捷版本，默认周期 14。
    ;
}

/// 线性回归，零拷贝写入 `out`（与 `values` 等长）。见 [`linear_reg`]。
/// Linear Regression, written zero-copy into `out`. See [`linear_reg`].
pub fn linear_reg_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "linear_reg_with_output: out length must equal values length".into(),
        ));
    }
    linreg_core_with_output(values, time_period, 0, out);
    Ok(())
}

/// 线性回归角度，零拷贝写入 `out`。见 [`linear_reg_angle`]。
/// Linear Regression Angle, written zero-copy into `out`. See [`linear_reg_angle`].
pub fn linear_reg_angle_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "linear_reg_angle_with_output: out length must equal values length".into(),
        ));
    }
    linreg_core_with_output(values, time_period, 1, out);
    Ok(())
}

/// 线性回归截距，零拷贝写入 `out`。见 [`linear_reg_intercept`]。
/// Linear Regression Intercept, written zero-copy into `out`. See [`linear_reg_intercept`].
pub fn linear_reg_intercept_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "linear_reg_intercept_with_output: out length must equal values length".into(),
        ));
    }
    linreg_core_with_output(values, time_period, 2, out);
    Ok(())
}

/// 线性回归斜率，零拷贝写入 `out`。见 [`linear_reg_slope`]。
/// Linear Regression Slope, written zero-copy into `out`. See [`linear_reg_slope`].
pub fn linear_reg_slope_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "linear_reg_slope_with_output: out length must equal values length".into(),
        ));
    }
    linreg_core_with_output(values, time_period, 3, out);
    Ok(())
}

/// 时间序列预测，零拷贝写入 `out`。见 [`tsf`]。
/// Time Series Forecast, written zero-copy into `out`. See [`tsf`].
pub fn tsf_with_output(
    values: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    if out.len() != values.len() {
        return Err(TaError::BadParam(
            "tsf_with_output: out length must equal values length".into(),
        ));
    }
    linreg_core_with_output(values, time_period, 4, out);
    Ok(())
}

// ---------------------------------------------------------------------------
// BETA / CORREL
// ---------------------------------------------------------------------------

/// TA-Lib `TA_IS_ZERO` 的等价判定：`(-1e-8 < v) && (v < 1e-8)`。
/// 与原生 C 实现的收益率分母保护保持一致。
#[inline]
fn ta_is_zero(v: f64) -> bool {
    v > -1e-8 && v < 1e-8
}

/// 计算第 `idx` 个相邻价格（索引 `idx-1 -> idx`）的收益率对 `(x, y)`，
/// 其中 `x` 来自 `real0`、`y` 来自 `real1`，并更新各自的"上一价格"游标。
/// Computes the (x, y) return pair between prices `idx-1` and `idx`,
/// 贝塔系数内核（TA-Lib `TA_BETA`，逐字移植 C 流式算法）。
///
/// TA-Lib 的 BETA **不是** 对原始价格的协方差/方差，而是：对相邻价格取相对变化
/// （收益率）得到 `(x, y)` 样本点，再对窗口内 `period` 个收益点对做线性回归，
/// 返回斜率 `β = (n·Sxy − Sx·Sy) / (n·Sxx − Sx²)`。
/// 由于每个收益点对需要 2 个价格，首个有效输出落在索引 `period`（lookback = period）。
///
/// Beta kernel (verbatim numeric port of TA-Lib's streaming C algorithm).
///
/// 与 C 的逐字双游标移植不同，此处先把收益率序列 `(rx, ry)` 预计算一次（含 `TA_IS_ZERO`
/// 守卫，逐位等价于 C 的 `return_pair`），再以 O(1) 滑动窗口累加 `s_xx/s_xy/s_x/s_y`
/// （与 [`correl_core_with_output`] 同构）。窗口集合恒为 `[i-period+1 .. i]`，故 C 的求和
/// 顺序与数值得以保留（1e-8 容差内，ADR 0005）；每元素除法由 2 次降为 1 次，消除对尾部
/// 收益对的重复重算（P3-2，ADR 0010）。
fn beta_core(real0: &[f64], real1: &[f64], period: usize) -> Vec<f64> {
    let n = real0.len();
    let mut out = vec![f64::NAN; n];
    // 需要 period+1 个价格才能构成 period 个收益点对。
    if n <= period {
        return out;
    }
    let p = period as f64;
    // 预计算收益率序列（与 C `return_pair` 逐位一致：含 TA_IS_ZERO 守卫，游标顺序推进）。
    let mut rx = vec![0.0_f64; n];
    let mut ry = vec![0.0_f64; n];
    let mut last_x = real0[0];
    let mut last_y = real1[0];
    for i in 1..n {
        let cur_x = real0[i];
        rx[i] = if ta_is_zero(last_x) { 0.0 } else { (cur_x - last_x) / last_x };
        last_x = cur_x;
        let cur_y = real1[i];
        ry[i] = if ta_is_zero(last_y) { 0.0 } else { (cur_y - last_y) / last_y };
        last_y = cur_y;
    }
    // 种子：首个窗口（输出索引 `period`）的朴素前向求和（窗口 = [1 .. period]）。
    let mut s_xx = 0.0_f64;
    let mut s_xy = 0.0_f64;
    let mut s_x = 0.0_f64;
    let mut s_y = 0.0_f64;
    for k in 1..=period {
        let a = rx[k];
        let b = ry[k];
        s_xx += a * a;
        s_xy += a * b;
        s_x += a;
        s_y += b;
    }
    let denom = p * s_xx - s_x * s_x;
    out[period] = if ta_is_zero(denom) { 0.0 } else { (p * s_xy - s_x * s_y) / denom };
    // 滑动：窗口右移一格，加入右端新收益率对、剔除左端出窗对（O(1)）。
    for i in (period + 1)..n {
        let a_new = rx[i];
        let b_new = ry[i];
        let a_old = rx[i - period];
        let b_old = ry[i - period];
        s_xx = s_xx + a_new * a_new - a_old * a_old;
        s_xy = s_xy + a_new * b_new - a_old * b_old;
        s_x = s_x + a_new - a_old;
        s_y = s_y + b_new - b_old;
        let denom = p * s_xx - s_x * s_x;
        out[i] = if ta_is_zero(denom) { 0.0 } else { (p * s_xy - s_x * s_y) / denom };
    }
    out
}

/// 皮尔逊相关系数内核（TA-Lib `TA_CORREL`，总体口径，原始价格）。
/// Pearson correlation kernel (population basis, raw prices).
fn correl_core(real0: &[f64], real1: &[f64], period: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; real0.len()];
    correl_core_with_output(real0, real1, period, &mut out);
    out
}

/// 皮尔逊相关系数内核（O(n) 滑动，P2-6，ADR 0010）。
///
/// Pearson correlation kernel (O(n) sliding, P2-6, ADR 0010).
///
/// 维护滚动 `s0`/`s1`（窗口和，= `rolling_sum`）、`s00`/`s11`（滚动平方和）、`s01`（滚动
/// 叉积），均按 O(1) 滑动递推（与 `rolling_var` 同构），消除每窗口 `period` 次重算。
/// 首个窗口沿用朴素求和作种子，`cov`/`v0`/`v1` 计算与历史逐项对齐，数值同黄金向量一致
/// （ADR 0005）。
///
/// Maintains rolling `s0`/`s1` (window sums = `rolling_sum`), `s00`/`s11` (rolling
/// sum-of-squares), `s01` (rolling cross-product), all slid in O(1) (isomorphic to
/// `rolling_var`), eliminating the per-window `period` recompute. The first window uses the
/// naïve sum as a seed; the `cov`/`v0`/`v1` math stays aligned with the historical impl
/// (1:1 with the golden vector, ADR 0005).
fn correl_core_with_output(real0: &[f64], real1: &[f64], period: usize, out: &mut [f64]) {
    let n = real0.len();
    if n < period {
        return;
    }
    let p = period as f64;
    // 种子：首个窗口（i = period-1）的朴素求和（窗口 = [i-period+1 .. i]）。
    // Seed: naïve sums of the first window (i = period-1, window = [i-period+1 .. i]).
    let mut s0 = 0.0_f64;
    let mut s1 = 0.0_f64;
    let mut s00 = 0.0_f64;
    let mut s11 = 0.0_f64;
    let mut s01 = 0.0_f64;
    for k in 0..period {
        let a = real0[period - 1 - k];
        let b = real1[period - 1 - k];
        s0 += a;
        s1 += b;
        s00 += a * a;
        s11 += b * b;
        s01 += a * b;
    }
    let cov = (s01 - s0 * s1 / p) / p;
    let v0 = (s00 - s0 * s0 / p) / p;
    let v1 = (s11 - s1 * s1 / p) / p;
    out[period - 1] = if ta_is_zero(v0) || ta_is_zero(v1) {
        0.0
    } else {
        cov / (v0 * v1).sqrt()
    };
    for i in period..n {
        // 滚动滑动：窗口右移一格，加入右端新元素、剔除左端出窗元素。
        // Slide: the window shifts right by one; add the new right element, drop the left one.
        s0 = s0 + real0[i] - real0[i - period];
        s1 = s1 + real1[i] - real1[i - period];
        s00 = s00 + real0[i] * real0[i] - real0[i - period] * real0[i - period];
        s11 = s11 + real1[i] * real1[i] - real1[i - period] * real1[i - period];
        s01 = s01 + real0[i] * real1[i] - real0[i - period] * real1[i - period];
        let cov = (s01 - s0 * s1 / p) / p;
        let v0 = (s00 - s0 * s0 / p) / p;
        let v1 = (s11 - s1 * s1 / p) / p;
        out[i] = if ta_is_zero(v0) || ta_is_zero(v1) {
            0.0
        } else {
            cov / (v0 * v1).sqrt()
        };
    }
}

/// 贝塔系数（Beta，TA-Lib `TA_BETA`）。
///
/// 以 `real0` 为"市场"、`real1` 为"标的"，取相邻价格的相对变化（收益率）构成样本点，
/// 对窗口内 `period` 个收益点对做线性回归，返回该回归线的斜率。
/// 数值与 TA-Lib 0.7.1 逐项一致。前导 `period` 个为 [`f64::NAN`]（lookback = period）。
///
/// Beta is the slope of the linear regression through the (return_x, return_y)
/// sample points over a trailing window of `period` return pairs.
pub fn beta(
    real0: &[f64],
    real1: &[f64],
    time_period: usize,
) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[real0, real1], "beta")?;
    let mut out = vec![f64::NAN; real0.len()];
    beta_with_output(real0, real1, time_period, &mut out)?;
    Ok(out)
}

/// 贝塔系数，零拷贝写入 `out`（与 `real0` 等长）。见 [`beta`]。
/// Beta, written zero-copy into `out`. See [`beta`].
pub fn beta_with_output(
    real0: &[f64],
    real1: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    check_eq_len(&[real0, real1], "beta")?;
    if out.len() != real0.len() {
        return Err(TaError::BadParam(
            "beta_with_output: out length must equal real0 length".into(),
        ));
    }
    let temp = beta_core(real0, real1, time_period);
    out.copy_from_slice(&temp);
    Ok(())
}

/// `beta` 便捷版本，默认周期 5。/ `beta` with default period (5).
pub fn beta_default(real0: &[f64], real1: &[f64]) -> Result<Vec<f64>, TaError> {
    beta(real0, real1, BETA_PERIOD)
}

/// 皮尔逊相关系数（Pearson Correlation Coefficient，TA-Lib `TA_CORREL`）。
///
/// `CORREL = cov(real0, real1) / sqrt(var(real0) * var(real1))`，总体口径（原始价格）。
/// 数值与 TA-Lib 0.7.1 逐项一致。前导 `period-1` 个为 [`f64::NAN`]。
pub fn correl(
    real0: &[f64],
    real1: &[f64],
    time_period: usize,
) -> Result<Vec<f64>, TaError> {
    check_period(time_period)?;
    check_eq_len(&[real0, real1], "correl")?;
    Ok(correl_core(real0, real1, time_period))
}

/// 皮尔逊相关系数，零拷贝写入 `out`（与 `real0` 等长）。见 [`correl`]。
/// Pearson Correlation Coefficient, written zero-copy into `out`. See [`correl`].
pub fn correl_with_output(
    real0: &[f64],
    real1: &[f64],
    time_period: usize,
    out: &mut [f64],
) -> Result<(), TaError> {
    check_period(time_period)?;
    check_eq_len(&[real0, real1], "correl")?;
    if out.len() != real0.len() {
        return Err(TaError::BadParam(
            "correl_with_output: out length must equal real0 length".into(),
        ));
    }
    correl_core_with_output(real0, real1, time_period, out);
    Ok(())
}

/// `correl` 便捷版本，默认周期 5。/ `correl` with default period (5).
pub fn correl_default(real0: &[f64], real1: &[f64]) -> Result<Vec<f64>, TaError> {
    correl(real0, real1, CORREL_PERIOD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stddev_population() {
        let x = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        // 均值 = 40/8 = 5；Σ(x-5)^2 = 9+1+1+1+0+0+4+16 = 32；总体方差 = 4 -> 标准差 2
        let out = stddev(&x, 8, 1.0).unwrap();
        assert!((out[7] - 2.0).abs() < 1e-9);
        // nb_dev = 2 -> 4
        let out2 = stddev(&x, 8, 2.0).unwrap();
        assert!((out2[7] - 4.0).abs() < 1e-9);
    }

    #[test]
    fn var_matches_stddev_squared() {
        let x: Vec<f64> = (0..30).map(|i| (i as f64).sin() + 2.0).collect();
        let v = var(&x, 10, 1.0).unwrap();
        let s = stddev(&x, 10, 1.0).unwrap();
        let i = 29;
        assert!((v[i] - s[i] * s[i]).abs() < 1e-9);
    }

    #[test]
    fn linear_reg_through_points() {
        // 完美线性序列 y = 2x+1，period=3 -> 斜率 2，截距 1，LINEARREG 在位置 2 = 5
        let y = [1.0, 3.0, 5.0, 7.0, 9.0];
        let lr = linear_reg(&y, 3).unwrap();
        assert!((lr[2] - 5.0).abs() < 1e-9);
        let slope = linear_reg_slope(&y, 3).unwrap();
        assert!((slope[2] - 2.0).abs() < 1e-9);
        let intercept = linear_reg_intercept(&y, 3).unwrap();
        assert!((intercept[2] - 1.0).abs() < 1e-9);
        let tsf_out = tsf(&y, 3).unwrap();
        // TSF 外推一步：位置 3 = 1 + 2*3 = 7
        assert!((tsf_out[2] - 7.0).abs() < 1e-9);
        let angle = linear_reg_angle(&y, 3).unwrap();
        // atan(2)*180/π
        assert!((angle[2] - (2.0_f64).atan() * 180.0 / std::f64::consts::PI).abs() < 1e-9);
    }

    #[test]
    fn correl_perfect_positive() {
        let a: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let b: Vec<f64> = (0..20).map(|i| 2.0 * i as f64 + 1.0).collect();
        let c = correl(&a, &b, 10).unwrap();
        assert!((c[19] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn beta_of_proportional_series_is_one() {
        // TA-Lib 的 BETA 基于"收益率"而非原始价格：若 b 与 a 成严格比例（b = 3·a），
        // 二者的相对变化完全相同，回归斜率为 1.0（比例缩放不会放大 beta）。
        // TA-Lib BETA is return-based: proportional series share identical returns,
        // so the regression slope is 1.0 regardless of the scaling factor.
        let a: Vec<f64> = (0..20).map(|i| 10.0 + i as f64).collect();
        let b: Vec<f64> = a.iter().map(|&v| 3.0 * v).collect();
        let bt = beta(&a, &b, 10).unwrap();
        assert!((bt[19] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn beta_length_mismatch_is_error() {
        assert!(matches!(
            beta(&[1.0, 2.0], &[1.0], 1),
            Err(TaError::BadParam(_))
        ));
    }

    /// 朴素 O(n·period) 线性回归（仅对照滑动实现，非热路径）。
    /// Naïve O(n·period) linear regression — reference for the sliding impl.
    fn linreg_core_naive(values: &[f64], period: usize, mode: u8) -> Vec<f64> {
        let n = values.len();
        let mut out = vec![f64::NAN; n];
        if n < period {
            return out;
        }
        let p = period as f64;
        let sx = (period * (period - 1)) as f64 / 2.0;
        let sxx = (period * (period - 1) * (2 * period - 1)) as f64 / 6.0;
        let denom = p * sxx - sx * sx;
        for i in (period - 1)..n {
            let mut sy = 0.0_f64;
            let mut sxy = 0.0_f64;
            for k in 0..period {
                let x = values[i - (period - 1) + k];
                sy += x;
                sxy += (k as f64) * x;
            }
            let slope = (p * sxy - sx * sy) / denom;
            let intercept = (sy - slope * sx) / p;
            out[i] = match mode {
                1 => slope.atan() * 180.0 / std::f64::consts::PI,
                2 => intercept,
                3 => slope,
                4 => intercept + slope * p,
                _ => intercept + slope * (p - 1.0),
            };
        }
        out
    }

    /// 朴素 O(n·period) 相关系数（仅对照滑动实现，非热路径）。
    /// Naïve O(n·period) correlation — reference for the sliding impl.
    fn correl_core_naive(real0: &[f64], real1: &[f64], period: usize) -> Vec<f64> {
        let n = real0.len();
        let mut out = vec![f64::NAN; n];
        if n < period {
            return out;
        }
        let p = period as f64;
        for i in (period - 1)..n {
            let mut s0 = 0.0_f64;
            let mut s1 = 0.0_f64;
            let mut s00 = 0.0_f64;
            let mut s11 = 0.0_f64;
            let mut s01 = 0.0_f64;
            for k in 0..period {
                let a = real0[i - k];
                let b = real1[i - k];
                s0 += a;
                s1 += b;
                s00 += a * a;
                s11 += b * b;
                s01 += a * b;
            }
            let cov = (s01 - s0 * s1 / p) / p;
            let v0 = (s00 - s0 * s0 / p) / p;
            let v1 = (s11 - s1 * s1 / p) / p;
            out[i] = if ta_is_zero(v0) || ta_is_zero(v1) {
                0.0
            } else {
                cov / (v0 * v1).sqrt()
            };
        }
        out
    }

    /// 滑动递推线性回归必须与朴素扫描逐项相等（含不同窗口、序列形态、5 个 mode）。
    /// The sliding-recurrence linear regression must equal the naïve scan element-wise
    /// (across windows, series shapes, and all 5 modes).
    #[test]
    fn linreg_core_matches_naive() {
        let close = |a: f64, b: f64| -> bool {
            if a.is_nan() && b.is_nan() {
                return true;
            }
            (a - b).abs() <= 1e-9
                || (a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-6 * a.abs().max(b.abs()))
        };
        let mut x: u64 = 0x1234_5678_9abc_def0;
        let mut lcg = || {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((x >> 11) as f64 / (1u64 << 53) as f64) * 100.0 - 50.0
        };
        for &n in &[0usize, 1, 2, 5, 20, 64, 137, 500] {
            for &p in &[1usize, 2, 3, 7, 20, 30, 64] {
                if n < p || p < 2 {
                    // period == 1 是退化情形（denom=0，数学未定义，TA-Lib 要求 period>=2，
                    // 黄金向量从不使用）；朴素与滑动实现在此产生不可比的 NaN/±inf，跳过。
                    // period == 1 is degenerate (denom = 0, undefined; TA-Lib requires period>=2
                    // and golden vectors never use it); skip the comparison.
                    continue;
                }
                let v: Vec<f64> = (0..n).map(|_| lcg()).collect();
                for mode in 0..5u8 {
                    let fast = linreg_core(&v, p, mode);
                    let naive = linreg_core_naive(&v, p, mode);
                    for i in 0..n {
                        assert!(
                            close(fast[i], naive[i]),
                            "linreg mismatch n={n} p={p} mode={mode} i={i}: {} vs {}",
                            fast[i],
                            naive[i]
                        );
                    }
                }
            }
        }
    }

    /// 滑动递推相关系数必须与朴素扫描逐项相等（含不同窗口、序列形态）。
    /// The sliding-recurrence correlation must equal the naïve scan element-wise
    /// (across windows and series shapes).
    #[test]
    fn correl_core_matches_naive() {
        let close = |a: f64, b: f64| -> bool {
            if a.is_nan() && b.is_nan() {
                return true;
            }
            (a - b).abs() <= 1e-9
                || (a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-6 * a.abs().max(b.abs()))
        };
        let mut x: u64 = 0x1357_9bdf_2468_ace0;
        let mut lcg = || {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((x >> 11) as f64 / (1u64 << 53) as f64) * 100.0 - 50.0
        };
        for &n in &[0usize, 1, 2, 5, 20, 64, 137, 500] {
            for &p in &[1usize, 2, 3, 7, 20, 30, 64] {
                if n < p {
                    continue;
                }
                let a: Vec<f64> = (0..n).map(|_| lcg()).collect();
                let b: Vec<f64> = (0..n).map(|_| lcg()).collect();
                let fast = correl_core(&a, &b, p);
                let naive = correl_core_naive(&a, &b, p);
                for i in 0..n {
                    assert!(
                        close(fast[i], naive[i]),
                        "correl mismatch n={n} p={p} i={i}: {} vs {}",
                        fast[i],
                        naive[i]
                    );
                }
            }
        }
    }
}
