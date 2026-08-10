#!/usr/bin/env python3
"""生成参考黄金向量 fixture（纯 Python 复刻 src 算法，非 TA-Lib C 绑定）。

Generate REFERENCE golden-vector fixtures: a pure-Python reimplementation of the
indicators in `src/overlap.rs`, `src/cycle.rs`, `src/volatility.rs`, `src/volume.rs`,
`src/price_transform.rs`, `src/stat.rs`. These mirror the Rust algorithms exactly and are
used to verify
cross-language consistency and protect against regressions.

They are NOT authoritative TA-Lib 0.7.1 vectors. To obtain the authoritative
fixtures, run `tools/gen_fixtures/generate.py` once the system TA-Lib C library
and the `TA-Lib` Python package are installed (see ADR 0003), then overwrite
these files. Every emitted file carries an `_note` flagging its REFERENCE status.

用法 / Usage:
  python tools/gen_fixtures/reference.py
"""
import json
import math
import os

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
FIXTURE_DIR = os.path.join(REPO_ROOT, "tests", "fixtures")

NOTE = (
    "REFERENCE values derived from the TA-Lib 0.7.1 documented algorithm "
    "(independent Python reimplementation, mirroring the src Rust modules). "
    "NOT yet bound to TA-Lib C output. Regenerate authoritatively via "
    "tools/gen_fixtures/generate.py."
)


def to_jsonable(arr):
    """将序列转为 JSON 友好列表；NaN 用 None 表示（与 Rust `f64::NAN` 对应）。"""
    out = []
    for v in arr:
        f = float(v)
        # NaN != NaN, so this is the portable check for NaN.
        out.append(None if f != f else f)
    return out


def write_fixture(payload):
    name = payload["name"]
    os.makedirs(FIXTURE_DIR, exist_ok=True)
    path = os.path.join(FIXTURE_DIR, f"{name}.json")
    payload["_note"] = NOTE
    with open(path, "w") as f:
        json.dump(payload, f, indent=2)
    print(f"wrote {path}")


# ---------------------------------------------------------------------------
# 样本数据 / Sample data (identical to generate.py's OHLC so inputs match)
# ---------------------------------------------------------------------------
def build_ohlc():
    close = [100.0 + 10.0 * math.sin(i * 0.3) + i * 0.05 for i in range(120)]
    o = [close[0]] + close[:119]
    high = [max(o[i], close[i]) + 1.0 + 0.5 * math.sin(i) for i in range(120)]
    low = [min(o[i], close[i]) - 1.0 - 0.5 * math.sin(i + 1) for i in range(120)]
    volume = [1000.0 + 100.0 * math.sin(i * 0.7) for i in range(120)]
    return {"open": o, "high": high, "low": low, "close": close, "volume": volume}


OHLC = build_ohlc()


# ---------------------------------------------------------------------------
# 内核 / Kernels (mirror src/core + src modules)
# ---------------------------------------------------------------------------
def ema(values, period):
    n = len(values)
    out = [float("nan")] * n
    # 跳过前导 NaN（与 Rust `core::ema` 一致，用于 DEMA/TEMA/T3 的嵌套 EMA）。
    # Skip a leading NaN prefix (matches Rust `core::ema`; used by nested EMAs).
    start = next((i for i, v in enumerate(values) if v == v), None)
    if start is None:
        return out
    if n - start < period:
        return out
    seed = sum(values[start : start + period]) / period
    out[start + period - 1] = seed
    k = 2.0 / (period + 1.0)
    prev = seed
    for i in range(start + period, n):
        prev = (values[i] - prev) * k + prev
        out[i] = prev
    return out


def ema_wilder(values, period):
    n = len(values)
    out = [float("nan")] * n
    if n < period:
        return out
    seed = sum(values[:period]) / period
    out[period - 1] = seed
    k = 1.0 / period
    prev = seed
    for i in range(period, n):
        prev = prev + (values[i] - prev) * k
        out[i] = prev
    return out


def true_range(high, low, close):
    n = min(len(high), len(low), len(close))
    out = [float("nan")] * n
    if n == 0:
        return out
    out[0] = high[0] - low[0]
    for i in range(1, n):
        hl = high[i] - low[i]
        hc = abs(high[i] - close[i - 1])
        lc = abs(low[i] - close[i - 1])
        out[i] = max(hl, hc, lc)
    return out


def rolling_var(values, period):
    n = len(values)
    out = [float("nan")] * n
    if n < period:
        return out
    p = float(period)
    sx = sum(values[:period])
    sxx = sum(v * v for v in values[:period])
    out[period - 1] = (sxx - sx * sx / p) / p
    for i in range(period, n):
        sx += values[i] - values[i - period]
        sxx += values[i] * values[i] - values[i - period] * values[i - period]
        out[i] = (sxx - sx * sx / p) / p
    return out


def linreg_core(values, period, mode):
    n = len(values)
    out = [float("nan")] * n
    if n < period:
        return out
    p = float(period)
    sx = (period * (period - 1)) / 2.0
    sxx = (period * (period - 1) * (2 * period - 1)) / 6.0
    denom = p * sxx - sx * sx
    for i in range(period - 1, n):
        sy = 0.0
        sxy = 0.0
        for k in range(period):
            x = values[i - (period - 1) + k]
            sy += x
            sxy += k * x
        slope = (p * sxy - sx * sy) / denom
        intercept = (sy - slope * sx) / p
        if mode == 1:
            out[i] = math.atan(slope) * 180.0 / math.pi
        elif mode == 2:
            out[i] = intercept
        elif mode == 3:
            out[i] = slope
        elif mode == 4:
            out[i] = intercept + slope * p
        else:
            out[i] = intercept + slope * (p - 1.0)
    return out


def beta_corr_core(real0, real1, period, mode):
    n = len(real0)
    out = [float("nan")] * n
    if n < period:
        return out
    p = float(period)
    for i in range(period - 1, n):
        s0 = s1 = s00 = s11 = s01 = 0.0
        for k in range(period):
            a = real0[i - k]
            b = real1[i - k]
            s0 += a
            s1 += b
            s00 += a * a
            s11 += b * b
            s01 += a * b
        cov = (s01 - s0 * s1 / p) / p
        v0 = (s00 - s0 * s0 / p) / p
        v1 = (s11 - s1 * s1 / p) / p
        if mode == 0:
            out[i] = 0.0 if v0 == 0.0 else cov / v0
        else:
            out[i] = 0.0 if (v0 == 0.0 or v1 == 0.0) else cov / math.sqrt(v0 * v1)
    return out


# ---------------------------------------------------------------------------
# 波动率 / Volatility
# ---------------------------------------------------------------------------
def gen_trange():
    tr = true_range(OHLC["high"], OHLC["low"], OHLC["close"])
    write_fixture(
        {
            "name": "trange_basic",
            "params": {},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(tr),
        }
    )


def gen_atr():
    tr = true_range(OHLC["high"], OHLC["low"], OHLC["close"])
    out = ema_wilder(tr, 14)
    write_fixture(
        {
            "name": "atr_basic",
            "params": {"period": 14},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(out),
        }
    )


def gen_natr():
    tr = true_range(OHLC["high"], OHLC["low"], OHLC["close"])
    atr_line = ema_wilder(tr, 14)
    out = [float("nan")] * len(atr_line)
    close = OHLC["close"]
    for i in range(len(atr_line)):
        if atr_line[i] != atr_line[i]:  # NaN
            continue
        out[i] = 0.0 if close[i] == 0.0 else 100.0 * atr_line[i] / close[i]
    write_fixture(
        {
            "name": "natr_basic",
            "params": {"period": 14},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(out),
        }
    )


# ---------------------------------------------------------------------------
# 成交量 / Volume
# ---------------------------------------------------------------------------
def cumulative_ad(high, low, close, volume):
    n = len(close)
    out = [0.0] * n
    prev = 0.0
    for i in range(n):
        clv = 0.0 if high[i] == low[i] else (2.0 * close[i] - high[i] - low[i]) / (high[i] - low[i])
        out[i] = prev + volume[i] * clv
        prev = out[i]
    return out


def gen_ad():
    out = cumulative_ad(OHLC["high"], OHLC["low"], OHLC["close"], OHLC["volume"])
    write_fixture(
        {
            "name": "ad_basic",
            "params": {},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "volume": to_jsonable(OHLC["volume"]),
            "expected": to_jsonable(out),
        }
    )


def gen_adosc():
    ad_line = cumulative_ad(OHLC["high"], OHLC["low"], OHLC["close"], OHLC["volume"])
    ef = ema(ad_line, 3)
    es = ema(ad_line, 10)
    out = [float("nan")] * len(ad_line)
    for i in range(len(ad_line)):
        if ef[i] == ef[i] and es[i] == es[i]:
            out[i] = ef[i] - es[i]
    write_fixture(
        {
            "name": "adosc_basic",
            "params": {"fast": 3, "slow": 10},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "volume": to_jsonable(OHLC["volume"]),
            "expected": to_jsonable(out),
        }
    )


def gen_obv():
    close = OHLC["close"]
    volume = OHLC["volume"]
    n = len(close)
    out = [0.0] * n
    out[0] = volume[0]
    for i in range(1, n):
        if close[i] > close[i - 1]:
            out[i] = out[i - 1] + volume[i]
        elif close[i] < close[i - 1]:
            out[i] = out[i - 1] - volume[i]
        else:
            out[i] = out[i - 1]
    write_fixture(
        {
            "name": "obv_basic",
            "params": {},
            "close": to_jsonable(close),
            "volume": to_jsonable(volume),
            "expected": to_jsonable(out),
        }
    )


# ---------------------------------------------------------------------------
# 价格变换 / Price Transform
# ---------------------------------------------------------------------------
def gen_avgprice():
    o, h, l, c = OHLC["open"], OHLC["high"], OHLC["low"], OHLC["close"]
    out = [(h[i] + l[i] + c[i] + o[i]) / 4.0 for i in range(len(c))]
    write_fixture(
        {
            "name": "avgprice_basic",
            "params": {},
            "open": to_jsonable(o),
            "high": to_jsonable(h),
            "low": to_jsonable(l),
            "close": to_jsonable(c),
            "expected": to_jsonable(out),
        }
    )


def gen_medprice():
    h, l = OHLC["high"], OHLC["low"]
    out = [(h[i] + l[i]) / 2.0 for i in range(len(h))]
    write_fixture(
        {
            "name": "medprice_basic",
            "params": {},
            "high": to_jsonable(h),
            "low": to_jsonable(l),
            "expected": to_jsonable(out),
        }
    )


def gen_typprice():
    h, l, c = OHLC["high"], OHLC["low"], OHLC["close"]
    out = [(h[i] + l[i] + c[i]) / 3.0 for i in range(len(c))]
    write_fixture(
        {
            "name": "typprice_basic",
            "params": {},
            "high": to_jsonable(h),
            "low": to_jsonable(l),
            "close": to_jsonable(c),
            "expected": to_jsonable(out),
        }
    )


def gen_wclprice():
    h, l, c = OHLC["high"], OHLC["low"], OHLC["close"]
    out = [(h[i] + l[i] + 2.0 * c[i]) / 4.0 for i in range(len(c))]
    write_fixture(
        {
            "name": "wclprice_basic",
            "params": {},
            "high": to_jsonable(h),
            "low": to_jsonable(l),
            "close": to_jsonable(c),
            "expected": to_jsonable(out),
        }
    )


# ---------------------------------------------------------------------------
# 统计 / Statistic
# ---------------------------------------------------------------------------
def gen_stddev():
    out = [float("nan")] * len(OHLC["close"])
    var = rolling_var(OHLC["close"], 5)
    for i in range(len(var)):
        if var[i] == var[i]:
            out[i] = 1.0 * math.sqrt(var[i])
    write_fixture(
        {
            "name": "stddev_basic",
            "params": {"period": 5, "nb_dev": 1.0},
            "input": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(out),
        }
    )


def gen_var():
    out = rolling_var(OHLC["close"], 5)
    write_fixture(
        {
            "name": "var_basic",
            "params": {"period": 5, "nb_dev": 1.0},
            "input": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(out),
        }
    )


def gen_linear_reg():
    out = linreg_core(OHLC["close"], 14, 0)
    write_fixture(
        {
            "name": "linear_reg_basic",
            "params": {"period": 14},
            "input": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(out),
        }
    )


def gen_linear_reg_angle():
    out = linreg_core(OHLC["close"], 14, 1)
    write_fixture(
        {
            "name": "linear_reg_angle_basic",
            "params": {"period": 14},
            "input": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(out),
        }
    )


def gen_linear_reg_intercept():
    out = linreg_core(OHLC["close"], 14, 2)
    write_fixture(
        {
            "name": "linear_reg_intercept_basic",
            "params": {"period": 14},
            "input": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(out),
        }
    )


def gen_linear_reg_slope():
    out = linreg_core(OHLC["close"], 14, 3)
    write_fixture(
        {
            "name": "linear_reg_slope_basic",
            "params": {"period": 14},
            "input": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(out),
        }
    )


def gen_tsf():
    out = linreg_core(OHLC["close"], 14, 4)
    write_fixture(
        {
            "name": "tsf_basic",
            "params": {"period": 14},
            "input": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(out),
        }
    )


def gen_beta():
    out = beta_corr_core(OHLC["close"], OHLC["high"], 5, 0)
    write_fixture(
        {
            "name": "beta_basic",
            "params": {"period": 5},
            "real0": to_jsonable(OHLC["close"]),
            "real1": to_jsonable(OHLC["high"]),
            "expected": to_jsonable(out),
        }
    )


def gen_correl():
    out = beta_corr_core(OHLC["close"], OHLC["high"], 5, 1)
    write_fixture(
        {
            "name": "correl_basic",
            "params": {"period": 5},
            "real0": to_jsonable(OHLC["close"]),
            "real1": to_jsonable(OHLC["high"]),
            "expected": to_jsonable(out),
        }
    )


# ---------------------------------------------------------------------------
# 重叠研究（第二批）/ Overlap Studies (batch 2)
# ---------------------------------------------------------------------------
def rolling_mean(values, period):
    n = len(values)
    out = [float("nan")] * n
    if n < period:
        return out
    s = sum(values[:period])
    out[period - 1] = s / period
    for i in range(period, n):
        s += values[i] - values[i - period]
        out[i] = s / period
    return out


def rolling_mean_skip(values, period):
    n = len(values)
    out = [float("nan")] * n
    start = next((i for i, v in enumerate(values) if v == v), None)
    if start is None:
        return out
    if n - start < period:
        return out
    s = sum(values[start : start + period])
    out[start + period - 1] = s / period
    for i in range(start + period, n):
        s += values[i] - values[i - period]
        out[i] = s / period
    return out


def wma(values, period):
    n = len(values)
    out = [float("nan")] * n
    if n < period:
        return out
    denom = period * (period + 1) / 2.0
    for i in range(period - 1, n):
        s = 0.0
        for j in range(period):
            s += values[i - j] * (period - j)
        out[i] = s / denom
    return out


def stddev(values, period, nb_dev):
    n = len(values)
    out = [float("nan")] * n
    var = rolling_var(values, period)
    for i in range(n):
        if var[i] == var[i]:
            out[i] = nb_dev * math.sqrt(var[i])
    return out


def trima(values, period):
    n = len(values)
    if n < period:
        return [float("nan")] * n
    if period % 2 == 1:
        h = (period + 1) // 2
        p1, p2 = h, h
    else:
        h = period // 2
        p1, p2 = h, h + 1
    inner = rolling_mean(values, p1)
    return rolling_mean_skip(inner, p2)


def t3(values, period, v_factor):
    n = len(values)
    out = [float("nan")] * n
    if n < 6 * (period - 1) + 1:
        return out
    e1 = ema(values, period)
    e2 = ema(e1, period)
    e3 = ema(e2, period)
    e4 = ema(e3, period)
    e5 = ema(e4, period)
    e6 = ema(e5, period)
    v = v_factor
    c1 = -v * v * v
    c2 = 3.0 * (v * v - c1)
    c3 = -6.0 * v * v - 3.0 * (v - c1)
    c4 = (3.0 * v * v + 3.0 * v + 1.0) - c1
    for i in range(n):
        if e6[i] == e6[i]:
            out[i] = c1 * e6[i] + c2 * e5[i] + c3 * e4[i] + c4 * e3[i]
    return out


def kama(values, time_period):
    n = len(values)
    out = [float("nan")] * n
    if time_period == 1:
        return list(values)
    p = time_period
    if n <= p:
        return out
    const_max = 2.0 / 31.0
    const_diff = 2.0 / 3.0 - const_max
    sum_roc1 = 0.0
    today = 0
    trailing_idx = 0
    i = p
    while i > 0:
        i -= 1
        diff = values[today] - values[today + 1]
        sum_roc1 += abs(diff)
        today += 1
    prev_kama = values[today - 1]
    temp = values[today]
    temp2 = values[trailing_idx]
    trailing_idx += 1
    period_roc = temp - temp2
    trailing_value = temp2
    er = 1.0 if (sum_roc1 <= period_roc or abs(sum_roc1) < 1e-14) else abs(period_roc / sum_roc1)
    sc = er * const_diff + const_max
    sc *= sc
    prev_kama = (values[today] - prev_kama) * sc + prev_kama
    today += 1
    out[p] = prev_kama
    while today <= n - 1:
        temp = values[today]
        temp2 = values[trailing_idx]
        trailing_idx += 1
        period_roc = temp - temp2
        sum_roc1 -= abs(trailing_value - temp2)
        sum_roc1 += abs(temp - values[today - 1])
        trailing_value = temp2
        er = 1.0 if (sum_roc1 <= period_roc or abs(sum_roc1) < 1e-14) else abs(period_roc / sum_roc1)
        sc = er * const_diff + const_max
        sc *= sc
        prev_kama = (values[today] - prev_kama) * sc + prev_kama
        out[today] = prev_kama
        today += 1
    return out


def sar(high, low, acceleration, maximum):
    n = len(high)
    out = [float("nan")] * n
    if n < 2:
        return out
    af = acceleration
    if af > maximum:
        af = maximum
    up_move = high[1] - high[0]
    down_move = low[0] - low[1]
    is_long = not (down_move > up_move)
    today = 1
    new_high = high[0]
    new_low = low[0]
    if is_long:
        ep = high[1]
        s = new_low
    else:
        ep = low[1]
        s = new_high
    new_low = low[1]
    new_high = high[1]
    while today <= n - 1:
        bar = today
        prev_low = new_low
        prev_high = new_high
        new_low = low[today]
        new_high = high[today]
        today += 1
        if is_long:
            if new_low <= s:
                is_long = False
                s = ep
                if s < prev_high:
                    s = prev_high
                if s < new_high:
                    s = new_high
                out[bar] = s
                af = acceleration
                ep = new_low
                s = af * (ep - s) + s
                if s < prev_high:
                    s = prev_high
                if s < new_high:
                    s = new_high
            else:
                out[bar] = s
                if new_high > ep:
                    ep = new_high
                    af += acceleration
                    if af > maximum:
                        af = maximum
                s = af * (ep - s) + s
                if s > prev_low:
                    s = prev_low
                if s > new_low:
                    s = new_low
        elif new_high >= s:
            is_long = True
            s = ep
            if s > prev_low:
                s = prev_low
            if s > new_low:
                s = new_low
            out[bar] = s
            af = acceleration
            ep = new_high
            s = af * (ep - s) + s
            if s > prev_low:
                s = prev_low
            if s > new_low:
                s = new_low
        else:
            out[bar] = s
            if new_low < ep:
                ep = new_low
                af += acceleration
                if af > maximum:
                    af = maximum
            s = af * (ep - s) + s
            if s < prev_high:
                s = prev_high
            if s < new_high:
                s = new_high
    return out


def sarext(
    high,
    low,
    start_value,
    offset_on_reverse,
    accel_init_long,
    accel_long,
    accel_max_long,
    accel_init_short,
    accel_short,
    accel_max_short,
):
    n = len(high)
    out = [float("nan")] * n
    if n < 2:
        return out
    af_long = accel_init_long
    af_short = accel_init_short
    if af_long > accel_max_long:
        af_long = accel_max_long
    if af_short > accel_max_short:
        af_short = accel_max_short
    if start_value == 0.0:
        up_move = high[1] - high[0]
        down_move = low[0] - low[1]
        is_long = not (down_move > up_move)
    elif start_value > 0.0:
        is_long = True
    else:
        is_long = False
    today = 1
    new_high = high[0]
    new_low = low[0]
    if start_value == 0.0:
        if is_long:
            ep = high[1]
            s = new_low
        else:
            ep = low[1]
            s = new_high
    elif start_value > 0.0:
        ep = high[1]
        s = start_value
    else:
        ep = low[1]
        s = abs(start_value)
    new_low = low[1]
    new_high = high[1]
    while today <= n - 1:
        bar = today
        prev_low = new_low
        prev_high = new_high
        new_low = low[today]
        new_high = high[today]
        today += 1
        if is_long:
            if new_low <= s:
                is_long = False
                s = ep
                if s < prev_high:
                    s = prev_high
                if s < new_high:
                    s = new_high
                if offset_on_reverse != 0.0:
                    s += s * offset_on_reverse
                out[bar] = 0.0 - s
                af_short = accel_init_short
                ep = new_low
                s = af_short * (ep - s) + s
                if s < prev_high:
                    s = prev_high
                if s < new_high:
                    s = new_high
            else:
                out[bar] = s
                if new_high > ep:
                    ep = new_high
                    af_long += accel_long
                    if af_long > accel_max_long:
                        af_long = accel_max_long
                s = af_long * (ep - s) + s
                if s > prev_low:
                    s = prev_low
                if s > new_low:
                    s = new_low
        elif new_high >= s:
            is_long = True
            s = ep
            if s > prev_low:
                s = prev_low
            if s > new_low:
                s = new_low
            if offset_on_reverse != 0.0:
                s -= s * offset_on_reverse
            out[bar] = s
            af_long = accel_init_long
            ep = new_high
            s = af_long * (ep - s) + s
            if s > prev_low:
                s = prev_low
            if s > new_low:
                s = new_low
        else:
            out[bar] = 0.0 - s
            if new_low < ep:
                ep = new_low
                af_short += accel_short
                if af_short > accel_max_short:
                    af_short = accel_max_short
            s = af_short * (ep - s) + s
            if s < prev_high:
                s = prev_high
            if s < new_high:
                s = new_high
    return out


def mavp(values, periods, min_period, max_period, ma_type_sma=True):
    n = len(values)
    out = [float("nan")] * n
    p = []
    for x in periods:
        v = int(x)
        if v < min_period:
            v = min_period
        elif v > max_period:
            v = max_period
        if v < 1:
            v = 1
        p.append(v)
    lookback = max_period - 1  # SMA lookback
    distinct = sorted(set(p))
    for pp in distinct:
        ma_series = rolling_mean(values, pp)
        for i in range(lookback, n):
            if p[i] == pp:
                out[i] = ma_series[i]
    return out


def gen_bbands():
    close = OHLC["close"]
    period = 20
    middle = rolling_mean(close, period)
    sd = stddev(close, period, 1.0)
    n = len(close)
    upper = [float("nan")] * n
    lower = [float("nan")] * n
    for i in range(n):
        if middle[i] == middle[i] and sd[i] == sd[i]:
            upper[i] = middle[i] + 2.0 * sd[i]
            lower[i] = middle[i] - 2.0 * sd[i]
    write_fixture(
        {
            "name": "bbands_basic",
            "params": {
                "time_period": period,
                "nb_dev_up": 2.0,
                "nb_dev_dn": 2.0,
                "ma_type": "Sma",
            },
            "input": to_jsonable(close),
            "upper": to_jsonable(upper),
            "middle": to_jsonable(middle),
            "lower": to_jsonable(lower),
        }
    )


def gen_trima_basic():
    out = trima(OHLC["close"], 30)
    write_fixture(
        {
            "name": "trima_basic",
            "params": {"time_period": 30},
            "input": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(out),
        }
    )


def gen_t3_basic():
    out = t3(OHLC["close"], 5, 0.7)
    write_fixture(
        {
            "name": "t3_basic",
            "params": {"time_period": 5, "v_factor": 0.7},
            "input": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(out),
        }
    )


def gen_ma_basic():
    # MA default with SMA (period 30) == rolling mean.
    out = rolling_mean(OHLC["close"], 30)
    write_fixture(
        {
            "name": "ma_basic",
            "params": {"time_period": 30, "ma_type": "Sma"},
            "input": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(out),
        }
    )


def gen_mavp_basic():
    # 变周期数组：在 [2,30] 区间内往复，覆盖多种周期。
    periods = [2 + (i % 29) for i in range(len(OHLC["close"]))]
    out = mavp(OHLC["close"], periods, 2, 30)
    write_fixture(
        {
            "name": "mavp_basic",
            "params": {"min_period": 2, "max_period": 30, "ma_type": "Sma"},
            "input": to_jsonable(OHLC["close"]),
            "periods": to_jsonable(periods),
            "expected": to_jsonable(out),
        }
    )


def gen_kama_basic():
    out = kama(OHLC["close"], 30)
    write_fixture(
        {
            "name": "kama_basic",
            "params": {"time_period": 30},
            "input": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(out),
        }
    )


def gen_sar_basic():
    out = sar(OHLC["high"], OHLC["low"], 0.02, 0.2)
    write_fixture(
        {
            "name": "sar_basic",
            "params": {"acceleration": 0.02, "maximum": 0.2},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "expected": to_jsonable(out),
        }
    )


def gen_sarext_basic():
    out = sarext(
        OHLC["high"], OHLC["low"], 0.0, 0.0,
        0.02, 0.02, 0.2, 0.02, 0.02, 0.2,
    )
    write_fixture(
        {
            "name": "sarext_basic",
            "params": {
                "start_value": 0.0,
                "offset_on_reverse": 0.0,
                "accel_init_long": 0.02,
                "accel_long": 0.02,
                "accel_max_long": 0.2,
                "accel_init_short": 0.02,
                "accel_short": 0.02,
                "accel_max_short": 0.2,
            },
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "expected": to_jsonable(out),
        }
    )


# ---------------------------------------------------------------------------
# 周期类（希尔伯特变换）/ Cycle (Hilbert transform) — translated op-by-op.
# ---------------------------------------------------------------------------
class Hilbert:
    def __init__(self):
        self.period_wma_sub = 0.0
        self.period_wma_sum = 0.0
        self.trailing_wma_value = 0.0
        self.smoothed_value = 0.0
        self.trailing_wma_idx = 0
        self.a = 0.0962
        self.b = 0.5769
        self.rad2deg = 180.0 / (4.0 * math.atan(1.0))
        self.hilbert_idx = 0
        self.detrender_odd = [0.0, 0.0, 0.0]
        self.detrender_even = [0.0, 0.0, 0.0]
        self.detrender = 0.0
        self.prev_detrender_odd = 0.0
        self.prev_detrender_even = 0.0
        self.prev_detrender_input_odd = 0.0
        self.prev_detrender_input_even = 0.0
        self.q1_odd = [0.0, 0.0, 0.0]
        self.q1_even = [0.0, 0.0, 0.0]
        self.q1 = 0.0
        self.prev_q1_odd = 0.0
        self.prev_q1_even = 0.0
        self.prev_q1_input_odd = 0.0
        self.prev_q1_input_even = 0.0
        self.ji_odd = [0.0, 0.0, 0.0]
        self.ji_even = [0.0, 0.0, 0.0]
        self.ji = 0.0
        self.prev_ji_odd = 0.0
        self.prev_ji_even = 0.0
        self.prev_ji_input_odd = 0.0
        self.prev_ji_input_even = 0.0
        self.jq_odd = [0.0, 0.0, 0.0]
        self.jq_even = [0.0, 0.0, 0.0]
        self.jq = 0.0
        self.prev_jq_odd = 0.0
        self.prev_jq_even = 0.0
        self.prev_jq_input_odd = 0.0
        self.prev_jq_input_even = 0.0
        self.period = 0.0
        self.q2 = 0.0
        self.i2 = 0.0
        self.prev_q2 = 0.0
        self.prev_i2 = 0.0
        self.re = 0.0
        self.im = 0.0
        self.i1_for_odd_prev2 = 0.0
        self.i1_for_odd_prev3 = 0.0
        self.i1_for_even_prev2 = 0.0
        self.i1_for_even_prev3 = 0.0
        self.prev_phase = 0.0
        self.mama = 0.0
        self.fama = 0.0
        self.today_value = 0.0

    def init(self, values, lookback_total, wma_init_iters):
        start_idx = lookback_total if 0 < lookback_total else 0
        self.trailing_wma_idx = start_idx - lookback_total
        today = self.trailing_wma_idx
        t = values[today]
        today += 1
        self.period_wma_sub = t
        self.period_wma_sum = t
        t = values[today]
        today += 1
        self.period_wma_sub += t
        self.period_wma_sum += t * 2.0
        t = values[today]
        today += 1
        self.period_wma_sub += t
        self.period_wma_sum += t * 3.0
        self.trailing_wma_value = 0.0
        i = wma_init_iters
        while True:
            t = values[today]
            today += 1
            self.period_wma_sub += t
            self.period_wma_sub -= self.trailing_wma_value
            self.period_wma_sum += t * 4.0
            self.trailing_wma_value = values[self.trailing_wma_idx]
            self.trailing_wma_idx += 1
            self.smoothed_value = self.period_wma_sum * 0.1
            self.period_wma_sum -= self.period_wma_sub
            i -= 1
            if i == 0:
                break
        return today

    def step(self, values, today, today_value):
        adjusted_prev_period = 0.075 * self.period + 0.54
        self.today_value = today_value
        self.period_wma_sub += today_value
        self.period_wma_sub -= self.trailing_wma_value
        self.period_wma_sum += today_value * 4.0
        self.trailing_wma_value = values[self.trailing_wma_idx]
        self.trailing_wma_idx += 1
        self.smoothed_value = self.period_wma_sum * 0.1
        self.period_wma_sum -= self.period_wma_sub
        if today % 2 == 0:
            h = self.a * self.smoothed_value
            self.detrender = 0.0 - self.detrender_even[self.hilbert_idx]
            self.detrender_even[self.hilbert_idx] = h
            self.detrender += h
            self.detrender -= self.prev_detrender_even
            self.prev_detrender_even = self.b * self.prev_detrender_input_even
            self.detrender += self.prev_detrender_even
            self.prev_detrender_input_even = self.smoothed_value
            self.detrender *= adjusted_prev_period

            h = self.a * self.detrender
            self.q1 = 0.0 - self.q1_even[self.hilbert_idx]
            self.q1_even[self.hilbert_idx] = h
            self.q1 += h
            self.q1 -= self.prev_q1_even
            self.prev_q1_even = self.b * self.prev_q1_input_even
            self.q1 += self.prev_q1_even
            self.prev_q1_input_even = self.detrender
            self.q1 *= adjusted_prev_period

            h = self.a * self.i1_for_even_prev3
            self.ji = 0.0 - self.ji_even[self.hilbert_idx]
            self.ji_even[self.hilbert_idx] = h
            self.ji += h
            self.ji -= self.prev_ji_even
            self.prev_ji_even = self.b * self.prev_ji_input_even
            self.ji += self.prev_ji_even
            self.prev_ji_input_even = self.i1_for_even_prev3
            self.ji *= adjusted_prev_period

            h = self.a * self.q1
            self.jq = 0.0 - self.jq_even[self.hilbert_idx]
            self.jq_even[self.hilbert_idx] = h
            self.jq += h
            self.jq -= self.prev_jq_even
            self.prev_jq_even = self.b * self.prev_jq_input_even
            self.jq += self.prev_jq_even
            self.prev_jq_input_even = self.q1
            self.jq *= adjusted_prev_period

            self.hilbert_idx += 1
            if self.hilbert_idx == 3:
                self.hilbert_idx = 0

            self.q2 = 0.2 * (self.q1 + self.ji) + 0.8 * self.prev_q2
            self.i2 = 0.2 * (self.i1_for_even_prev3 - self.jq) + 0.8 * self.prev_i2

            self.i1_for_odd_prev3 = self.i1_for_odd_prev2
            self.i1_for_odd_prev2 = self.detrender

            denom = self.i1_for_even_prev3
            phase = math.atan(self.q1 / denom) * self.rad2deg if denom != 0.0 else 0.0
        else:
            h = self.a * self.smoothed_value
            self.detrender = 0.0 - self.detrender_odd[self.hilbert_idx]
            self.detrender_odd[self.hilbert_idx] = h
            self.detrender += h
            self.detrender -= self.prev_detrender_odd
            self.prev_detrender_odd = self.b * self.prev_detrender_input_odd
            self.detrender += self.prev_detrender_odd
            self.prev_detrender_input_odd = self.smoothed_value
            self.detrender *= adjusted_prev_period

            h = self.a * self.detrender
            self.q1 = 0.0 - self.q1_odd[self.hilbert_idx]
            self.q1_odd[self.hilbert_idx] = h
            self.q1 += h
            self.q1 -= self.prev_q1_odd
            self.prev_q1_odd = self.b * self.prev_q1_input_odd
            self.q1 += self.prev_q1_odd
            self.prev_q1_input_odd = self.detrender
            self.q1 *= adjusted_prev_period

            h = self.a * self.i1_for_odd_prev3
            self.ji = 0.0 - self.ji_odd[self.hilbert_idx]
            self.ji_odd[self.hilbert_idx] = h
            self.ji += h
            self.ji -= self.prev_ji_odd
            self.prev_ji_odd = self.b * self.prev_ji_input_odd
            self.ji += self.prev_ji_odd
            self.prev_ji_input_odd = self.i1_for_odd_prev3
            self.ji *= adjusted_prev_period

            h = self.a * self.q1
            self.jq = 0.0 - self.jq_odd[self.hilbert_idx]
            self.jq_odd[self.hilbert_idx] = h
            self.jq += h
            self.jq -= self.prev_jq_odd
            self.prev_jq_odd = self.b * self.prev_jq_input_odd
            self.jq += self.prev_jq_odd
            self.prev_jq_input_odd = self.q1
            self.jq *= adjusted_prev_period

            self.hilbert_idx += 1
            if self.hilbert_idx == 3:
                self.hilbert_idx = 0

            self.q2 = 0.2 * (self.q1 + self.ji) + 0.8 * self.prev_q2
            self.i2 = 0.2 * (self.i1_for_odd_prev3 - self.jq) + 0.8 * self.prev_i2

            self.i1_for_even_prev3 = self.i1_for_even_prev2
            self.i1_for_even_prev2 = self.detrender

            denom = self.i1_for_odd_prev3
            phase = math.atan(self.q1 / denom) * self.rad2deg if denom != 0.0 else 0.0
        return phase

    def update_period(self):
        self.re = 0.8 * self.re + 0.2 * (self.i2 * self.prev_i2 + self.q2 * self.prev_q2)
        self.im = 0.8 * self.im + 0.2 * (self.i2 * self.prev_q2 - self.q2 * self.prev_i2)
        self.prev_q2 = self.q2
        self.prev_i2 = self.i2
        temp_real = self.period
        if self.im != 0.0 and self.re != 0.0:
            self.period = 360.0 / (math.atan(self.im / self.re) * self.rad2deg)
        hi = 1.5 * temp_real
        if self.period > hi:
            self.period = hi
        lo = 0.67 * temp_real
        if self.period < lo:
            self.period = lo
        if self.period < 6.0:
            self.period = 6.0
        elif self.period > 50.0:
            self.period = 50.0
        self.period = 0.2 * self.period + 0.8 * temp_real


def mama(values, fast_limit, slow_limit):
    n = len(values)
    lookback = 32
    out_mama = [float("nan")] * n
    out_fama = [float("nan")] * n
    if n <= lookback:
        return out_mama, out_fama
    h = Hilbert()
    first_main = h.init(values, lookback, 9)
    today = first_main
    while today <= n - 1:
        phase = h.step(values, today, values[today])
        delta_phase = h.prev_phase - phase
        h.prev_phase = phase
        alpha = delta_phase
        if alpha < 1.0:
            alpha = 1.0
        if alpha > 1.0:
            alpha = fast_limit / alpha
            if alpha < slow_limit:
                alpha = slow_limit
        else:
            alpha = fast_limit
        h.mama = (1.0 - alpha) * h.mama + alpha * h.today_value
        alpha2 = alpha * 0.5
        h.fama = (1.0 - alpha2) * h.fama + alpha2 * h.mama
        h.update_period()
        if today >= lookback:
            out_mama[today] = h.mama
            out_fama[today] = h.fama
        today += 1
    return out_mama, out_fama


def ht_trendline(values):
    n = len(values)
    lookback = 63
    out = [float("nan")] * n
    if n <= lookback:
        return out
    h = Hilbert()
    first_main = h.init(values, lookback, 34)
    today = first_main
    i_trend1 = 0.0
    i_trend2 = i_trend1
    i_trend3 = i_trend2
    smooth_period = 0.0
    while today <= n - 1:
        h.step(values, today, values[today])
        h.update_period()
        smooth_period = 0.67 * smooth_period + 0.33 * h.period
        dc_period = smooth_period + 0.5
        dc_period_int = int(dc_period)
        s = 0.0
        for i in range(50):
            if i < dc_period_int and today >= i:
                s += values[today - i]
        if dc_period_int > 0:
            s /= dc_period_int
        trend = (2.0 * i_trend2 + 4.0 * s + 3.0 * i_trend1 + i_trend3) / 10.0
        i_trend3 = i_trend2
        i_trend2 = i_trend1
        i_trend1 = s
        if today >= lookback:
            out[today] = trend
        today += 1
    return out


def gen_mama_basic():
    m, f = mama(OHLC["close"], 0.5, 0.05)
    write_fixture(
        {
            "name": "mama_basic",
            "params": {"fast_limit": 0.5, "slow_limit": 0.05},
            "input": to_jsonable(OHLC["close"]),
            "mama": to_jsonable(m),
            "fama": to_jsonable(f),
        }
    )


def gen_ht_trendline_basic():
    out = ht_trendline(OHLC["close"])
    write_fixture(
        {
            "name": "ht_trendline_basic",
            "params": {},
            "input": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(out),
        }
    )


GENERATORS = [
    gen_trange,
    gen_atr,
    gen_natr,
    gen_ad,
    gen_adosc,
    gen_obv,
    gen_avgprice,
    gen_medprice,
    gen_typprice,
    gen_wclprice,
    gen_stddev,
    gen_var,
    gen_linear_reg,
    gen_linear_reg_angle,
    gen_linear_reg_intercept,
    gen_linear_reg_slope,
    gen_tsf,
    gen_beta,
    gen_correl,
    # ---- 重叠研究（第二批）/ Overlap Studies (batch 2) ----
    gen_bbands,
    gen_trima_basic,
    gen_t3_basic,
    gen_ma_basic,
    gen_mavp_basic,
    gen_kama_basic,
    gen_sar_basic,
    gen_sarext_basic,
    # ---- 周期类 / Cycle ----
    gen_mama_basic,
    gen_ht_trendline_basic,
]


def main():
    for g in GENERATORS:
        g()


if __name__ == "__main__":
    main()
