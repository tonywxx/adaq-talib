#!/usr/bin/env python3
"""生成黄金向量 fixture（来自 TA-Lib 原版，经 Python 绑定调用 C 库）。

Generate golden-vector fixtures from the original TA-Lib (via the Python binding over the C lib).

前置要求 / Requirements:
  - 系统已安装 TA-Lib C 库（如 `brew install ta-lib` 或从源码编译）。
  - Python 3 + `pip install TA-Lib`（注意 PyPI 包名是 `TA-Lib`，导入名 `talib`，
    且必须先装 C 库）。

输出 / Output:
  向仓库 `tests/fixtures/` 写入 JSON fixture。单输入指标结构：
  { "name": str, "params": {...}, "input": [f64...], "expected": [f64|null...] }
  双输入指标（如 midprice）额外含 "high"/"low" 字段。
  （`null` 表示 NaN / 不稳定期，对应 Rust 侧 `f64::NAN`）

用法 / Usage:
  python tools/gen_fixtures/generate.py

注意 / Note:
  黄金向量的"零偏差"基准是 TA-Lib 0.7.1。请先确认所用 C 库版本并登记于
  `tools/README.md`（见 ADR 0003）。
"""
import json
import math
import os

import numpy as np
import talib  # PyPI "TA-Lib"; requires system TA-Lib C lib

# The TA-Lib Python binding only accepts numpy arrays, never bare Python lists.
npa = lambda x: np.asarray(x, dtype=np.float64)

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
FIXTURE_DIR = os.path.join(REPO_ROOT, "tests", "fixtures")


def to_jsonable(arr):
    """将 numpy/talib 输出转为 JSON 友好列表；NaN 用 None 表示。"""
    out = []
    for v in arr:
        if v is None:
            out.append(None)
        else:
            f = float(v)
            out.append(None if f != f else f)  # NaN != NaN
    return out


def write_payload(payload):
    name = payload["name"]
    os.makedirs(FIXTURE_DIR, exist_ok=True)
    path = os.path.join(FIXTURE_DIR, f"{name}.json")
    with open(path, "w") as f:
        json.dump(payload, f, indent=2)
    print(f"wrote {path}")


def write_fixture(name, params, inputs, expected):
    write_payload(
        {
            "name": name,
            "params": params,
            "input": to_jsonable(inputs),
            "expected": to_jsonable(expected),
        }
    )


# 单输入样本数据（周期 3）。/ Single-input sample data (period 3).
SAMPLE = npa([float(2**i) for i in range(12)])  # 1,2,4,...,2048
PERIOD = 3


def gen_sma():
    prices = npa([float(p) for p in range(1, 11)])  # [1..10]
    period = 3
    out = talib.SMA(prices, timeperiod=period)
    write_fixture("sma_basic", {"time_period": period}, prices, list(out))


def gen_ema():
    out = talib.EMA(SAMPLE, timeperiod=PERIOD)
    write_fixture("ema_basic", {"time_period": PERIOD}, SAMPLE, list(out))


def gen_wma():
    out = talib.WMA(SAMPLE, timeperiod=PERIOD)
    write_fixture("wma_basic", {"time_period": PERIOD}, SAMPLE, list(out))


def gen_dema():
    out = talib.DEMA(SAMPLE, timeperiod=PERIOD)
    write_fixture("dema_basic", {"time_period": PERIOD}, SAMPLE, list(out))


def gen_tema():
    out = talib.TEMA(SAMPLE, timeperiod=PERIOD)
    write_fixture("tema_basic", {"time_period": PERIOD}, SAMPLE, list(out))


def gen_midpoint():
    out = talib.MIDPOINT(SAMPLE, timeperiod=PERIOD)
    write_fixture("midpoint_basic", {"time_period": PERIOD}, SAMPLE, list(out))


def gen_midprice():
    high = npa([2.0, 3.0, 5.0, 9.0, 17.0, 33.0, 65.0, 129.0, 257.0, 513.0, 1025.0, 2049.0])
    low = npa([0.0, 1.0, 3.0, 7.0, 15.0, 31.0, 63.0, 127.0, 255.0, 511.0, 1023.0, 2047.0])
    out = talib.MIDPRICE(high, low, timeperiod=PERIOD)
    write_payload(
        {
            "name": "midprice_basic",
            "params": {"time_period": PERIOD},
            "high": to_jsonable(high),
            "low": to_jsonable(low),
            "expected": to_jsonable(list(out)),
        }
    )


# NOTE: The `GENERATORS` list is assembled at the END of this file (after all
# generator functions are defined and the OHLC sample data is built), because
# several generators depend on `OHLC`, which is constructed later. See the list
# just above `main()`.

# ---------------------------------------------------------------------------
# 动量指标样本数据 / Momentum OHLC sample data
# ---------------------------------------------------------------------------
OHLC = {
    "close": [100.0 + 10.0 * math.sin(i * 0.3) + i * 0.05 for i in range(120)],
}
OHLC["open"] = [OHLC["close"][0]] + OHLC["close"][:119]
OHLC["high"] = [
    max(OHLC["open"][i], OHLC["close"][i]) + 1.0 + 0.5 * math.sin(i)
    for i in range(120)
]
OHLC["low"] = [
    min(OHLC["open"][i], OHLC["close"][i]) - 1.0 - 0.5 * math.sin(i + 1)
    for i in range(120)
]
OHLC["volume"] = [1000.0 + 100.0 * math.sin(i * 0.7) for i in range(120)]

# TA-Lib Python binding needs numpy arrays, so convert every OHLC series once here.
for _k in list(OHLC.keys()):
    OHLC[_k] = npa(OHLC[_k])


def gen_mom():
    out = talib.MOM(OHLC["close"], timeperiod=10)
    write_fixture("mom_basic", {"period": 10}, OHLC["close"], list(out))


def gen_roc():
    out = talib.ROC(OHLC["close"], timeperiod=10)
    write_fixture("roc_basic", {"period": 10}, OHLC["close"], list(out))


def gen_rocp():
    out = talib.ROCP(OHLC["close"], timeperiod=10)
    write_fixture("rocp_basic", {"period": 10}, OHLC["close"], list(out))


def gen_rocr():
    out = talib.ROCR(OHLC["close"], timeperiod=10)
    write_fixture("rocr_basic", {"period": 10}, OHLC["close"], list(out))


def gen_rocr100():
    out = talib.ROCR100(OHLC["close"], timeperiod=10)
    write_fixture("rocr100_basic", {"period": 10}, OHLC["close"], list(out))


def gen_rsi():
    out = talib.RSI(OHLC["close"], timeperiod=14)
    write_fixture("rsi_basic", {"period": 14}, OHLC["close"], list(out))


def gen_cmo():
    out = talib.CMO(OHLC["close"], timeperiod=14)
    write_fixture("cmo_basic", {"period": 14}, OHLC["close"], list(out))


def gen_trix():
    out = talib.TRIX(OHLC["close"], timeperiod=30)
    write_fixture("trix_basic", {"period": 30}, OHLC["close"], list(out))


def gen_stoch_rsi():
    # STOCHRSI returns (fastk, fastd); Rust's `stoch_rsi` exposes only the
    # fastk (raw stochastic of RSI) line, so keep just that to match the
    # single-output fixture format and the integration test.
    fastk, _fastd = talib.STOCHRSI(OHLC["close"], timeperiod=14, fastk_period=14)
    write_fixture("stoch_rsi_basic", {"rsi_period": 14, "period": 14}, OHLC["close"], list(fastk))


def gen_apo():
    # NOTE: TA-Lib's Python binding defaults APO/PPO/MACD `matype` to SMA (0), but the
    # canonical/standard APO is EMA-based, and Rust's `apo` implements the EMA version.
    # Request matype=EMA (1) explicitly so the authoritative fixture matches Rust.
    out = talib.APO(OHLC["close"], fastperiod=12, slowperiod=26, matype=1)
    write_fixture("apo_basic", {"fast": 12, "slow": 26, "ma_type": "Ema"}, OHLC["close"], list(out))


def gen_ppo():
    out = talib.PPO(OHLC["close"], fastperiod=12, slowperiod=26, matype=1)
    write_fixture("ppo_basic", {"fast": 12, "slow": 26, "ma_type": "Ema"}, OHLC["close"], list(out))


def gen_macd():
    macd_line, signal, hist = talib.MACD(
        OHLC["close"], fastperiod=12, slowperiod=26, signalperiod=9
    )
    write_payload(
        {
            "name": "macd_basic",
            "params": {"fast": 12, "slow": 26, "signal_period": 9, "ma_type": "Ema"},
            "input": to_jsonable(OHLC["close"]),
            "macd": to_jsonable(list(macd_line)),
            "signal": to_jsonable(list(signal)),
            "hist": to_jsonable(list(hist)),
        }
    )


def gen_cci():
    out = talib.CCI(OHLC["high"], OHLC["low"], OHLC["close"], timeperiod=20)
    write_payload(
        {
            "name": "cci_basic",
            "params": {"period": 20},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_mfi():
    out = talib.MFI(
        OHLC["high"], OHLC["low"], OHLC["close"], OHLC["volume"], timeperiod=14
    )
    write_payload(
        {
            "name": "mfi_basic",
            "params": {"period": 14},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "volume": to_jsonable(OHLC["volume"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_willr():
    out = talib.WILLR(OHLC["high"], OHLC["low"], OHLC["close"], timeperiod=14)
    write_payload(
        {
            "name": "willr_basic",
            "params": {"period": 14},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_bop():
    out = talib.BOP(OHLC["open"], OHLC["high"], OHLC["low"], OHLC["close"])
    write_payload(
        {
            "name": "bop_basic",
            "params": {},
            "open": to_jsonable(OHLC["open"]),
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_ultosc():
    out = talib.ULTOSC(
        OHLC["high"], OHLC["low"], OHLC["close"],
        timeperiod1=7, timeperiod2=14, timeperiod3=28,
    )
    write_payload(
        {
            "name": "ultosc_basic",
            "params": {"p1": 7, "p2": 14, "p3": 28},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_plus_dm():
    # TA-Lib's PLUS_DM/MINUS_DM take (high, low, timeperiod) only — they do not
    # use close. (Rust's `plus_dm` accepts close for API symmetry but ignores it.)
    out = talib.PLUS_DM(OHLC["high"], OHLC["low"], timeperiod=14)
    write_payload(
        {
            "name": "plus_dm_basic",
            "params": {"period": 14},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_minus_dm():
    out = talib.MINUS_DM(OHLC["high"], OHLC["low"], timeperiod=14)
    write_payload(
        {
            "name": "minus_dm_basic",
            "params": {"period": 14},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_plus_di():
    out = talib.PLUS_DI(OHLC["high"], OHLC["low"], OHLC["close"], timeperiod=14)
    write_payload(
        {
            "name": "plus_di_basic",
            "params": {"period": 14},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_minus_di():
    out = talib.MINUS_DI(OHLC["high"], OHLC["low"], OHLC["close"], timeperiod=14)
    write_payload(
        {
            "name": "minus_di_basic",
            "params": {"period": 14},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_adx():
    out = talib.ADX(OHLC["high"], OHLC["low"], OHLC["close"], timeperiod=14)
    write_payload(
        {
            "name": "adx_basic",
            "params": {"period": 14},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_adxr():
    out = talib.ADXR(OHLC["high"], OHLC["low"], OHLC["close"], timeperiod=14)
    write_payload(
        {
            "name": "adxr_basic",
            "params": {"period": 14},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_aroon():
    up, down = talib.AROON(OHLC["high"], OHLC["low"], timeperiod=14)
    write_payload(
        {
            "name": "aroon_basic",
            "params": {"period": 14},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "up": to_jsonable(list(up)),
            "down": to_jsonable(list(down)),
        }
    )


def gen_aroon_osc():
    out = talib.AROONOSC(OHLC["high"], OHLC["low"], timeperiod=14)
    write_payload(
        {
            "name": "aroon_osc_basic",
            "params": {"period": 14},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_stoch():
    slowk, slowd = talib.STOCH(
        OHLC["high"], OHLC["low"], OHLC["close"],
        fastk_period=5, slowk_period=3, slowd_period=3,
    )
    write_payload(
        {
            "name": "stoch_basic",
            "params": {"fast_k": 5, "slow_k_p": 3, "slow_d_p": 3},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "slow_k": to_jsonable(list(slowk)),
            "slow_d": to_jsonable(list(slowd)),
        }
    )


def gen_stoch_f():
    fastk, fastd = talib.STOCHF(
        OHLC["high"], OHLC["low"], OHLC["close"],
        fastk_period=5, fastd_period=3,
    )
    write_payload(
        {
            "name": "stoch_f_basic",
            "params": {"fast_k_p": 5, "fast_d_p": 3},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "fast_k": to_jsonable(list(fastk)),
            "fast_d": to_jsonable(list(fastd)),
        }
    )


# ---------------------------------------------------------------------------
# 波动率 / Volatility (authoritative TA-Lib C generators)
# ---------------------------------------------------------------------------
def gen_trange():
    out = talib.TRANGE(OHLC["high"], OHLC["low"], OHLC["close"])
    write_payload(
        {
            "name": "trange_basic", "params": {},
            "high": to_jsonable(OHLC["high"]), "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]), "expected": to_jsonable(list(out)),
        }
    )


def gen_atr():
    out = talib.ATR(OHLC["high"], OHLC["low"], OHLC["close"], timeperiod=14)
    write_payload(
        {
            "name": "atr_basic", "params": {"period": 14},
            "high": to_jsonable(OHLC["high"]), "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]), "expected": to_jsonable(list(out)),
        }
    )


def gen_natr():
    out = talib.NATR(OHLC["high"], OHLC["low"], OHLC["close"], timeperiod=14)
    write_payload(
        {
            "name": "natr_basic", "params": {"period": 14},
            "high": to_jsonable(OHLC["high"]), "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]), "expected": to_jsonable(list(out)),
        }
    )


# ---------------------------------------------------------------------------
# 成交量 / Volume
# ---------------------------------------------------------------------------
def gen_ad():
    out = talib.AD(OHLC["high"], OHLC["low"], OHLC["close"], OHLC["volume"])
    write_payload(
        {
            "name": "ad_basic", "params": {},
            "high": to_jsonable(OHLC["high"]), "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]), "volume": to_jsonable(OHLC["volume"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_adosc():
    out = talib.ADOSC(
        OHLC["high"], OHLC["low"], OHLC["close"], OHLC["volume"],
        fastperiod=3, slowperiod=10,
    )
    write_payload(
        {
            "name": "adosc_basic", "params": {"fast": 3, "slow": 10},
            "high": to_jsonable(OHLC["high"]), "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]), "volume": to_jsonable(OHLC["volume"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_obv():
    out = talib.OBV(OHLC["close"], OHLC["volume"])
    write_payload(
        {
            "name": "obv_basic", "params": {},
            "close": to_jsonable(OHLC["close"]), "volume": to_jsonable(OHLC["volume"]),
            "expected": to_jsonable(list(out)),
        }
    )


# ---------------------------------------------------------------------------
# 价格变换 / Price Transform
# ---------------------------------------------------------------------------
def gen_avgprice():
    out = talib.AVGPRICE(OHLC["open"], OHLC["high"], OHLC["low"], OHLC["close"])
    write_payload(
        {
            "name": "avgprice_basic", "params": {},
            "open": to_jsonable(OHLC["open"]), "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]), "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_medprice():
    out = talib.MEDPRICE(OHLC["high"], OHLC["low"])
    write_payload(
        {
            "name": "medprice_basic", "params": {},
            "high": to_jsonable(OHLC["high"]), "low": to_jsonable(OHLC["low"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_typprice():
    out = talib.TYPPRICE(OHLC["high"], OHLC["low"], OHLC["close"])
    write_payload(
        {
            "name": "typprice_basic", "params": {},
            "high": to_jsonable(OHLC["high"]), "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_wclprice():
    out = talib.WCLPRICE(OHLC["high"], OHLC["low"], OHLC["close"])
    write_payload(
        {
            "name": "wclprice_basic", "params": {},
            "high": to_jsonable(OHLC["high"]), "low": to_jsonable(OHLC["low"]),
            "close": to_jsonable(OHLC["close"]),
            "expected": to_jsonable(list(out)),
        }
    )


# ---------------------------------------------------------------------------
# 统计 / Statistic
# ---------------------------------------------------------------------------
def gen_stddev():
    out = talib.STDDEV(OHLC["close"], timeperiod=5, nbdev=1.0)
    write_fixture("stddev_basic", {"period": 5, "nb_dev": 1.0}, OHLC["close"], list(out))


def gen_var():
    out = talib.VAR(OHLC["close"], timeperiod=5, nbdev=1.0)
    write_fixture("var_basic", {"period": 5, "nb_dev": 1.0}, OHLC["close"], list(out))


def gen_linear_reg():
    out = talib.LINEARREG(OHLC["close"], timeperiod=14)
    write_fixture("linear_reg_basic", {"period": 14}, OHLC["close"], list(out))


def gen_linear_reg_angle():
    out = talib.LINEARREG_ANGLE(OHLC["close"], timeperiod=14)
    write_fixture("linear_reg_angle_basic", {"period": 14}, OHLC["close"], list(out))


def gen_linear_reg_intercept():
    out = talib.LINEARREG_INTERCEPT(OHLC["close"], timeperiod=14)
    write_fixture("linear_reg_intercept_basic", {"period": 14}, OHLC["close"], list(out))


def gen_linear_reg_slope():
    out = talib.LINEARREG_SLOPE(OHLC["close"], timeperiod=14)
    write_fixture("linear_reg_slope_basic", {"period": 14}, OHLC["close"], list(out))


def gen_tsf():
    out = talib.TSF(OHLC["close"], timeperiod=14)
    write_fixture("tsf_basic", {"period": 14}, OHLC["close"], list(out))


def gen_beta():
    out = talib.BETA(OHLC["close"], OHLC["high"], timeperiod=5)
    write_payload(
        {
            "name": "beta_basic", "params": {"period": 5},
            "real0": to_jsonable(OHLC["close"]), "real1": to_jsonable(OHLC["high"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_correl():
    out = talib.CORREL(OHLC["close"], OHLC["high"], timeperiod=5)
    write_payload(
        {
            "name": "correl_basic", "params": {"period": 5},
            "real0": to_jsonable(OHLC["close"]), "real1": to_jsonable(OHLC["high"]),
            "expected": to_jsonable(list(out)),
        }
    )


# ---------------------------------------------------------------------------
# 重叠研究（第二批）/ Overlap Studies (batch 2) — authoritative TA-Lib C
# ---------------------------------------------------------------------------
def gen_bbands():
    upper, middle, lower = talib.BBANDS(
        OHLC["close"], timeperiod=20, nbdevup=2.0, nbdevdn=2.0, matype=0
    )
    write_payload(
        {
            "name": "bbands_basic",
            "params": {"time_period": 20, "nb_dev_up": 2.0, "nb_dev_dn": 2.0, "ma_type": "Sma"},
            "input": to_jsonable(OHLC["close"]),
            "upper": to_jsonable(list(upper)),
            "middle": to_jsonable(list(middle)),
            "lower": to_jsonable(list(lower)),
        }
    )


def gen_trima():
    out = talib.TRIMA(OHLC["close"], timeperiod=30)
    write_fixture("trima_basic", {"time_period": 30}, OHLC["close"], list(out))


def gen_t3():
    out = talib.T3(OHLC["close"], timeperiod=5, vfactor=0.7)
    write_fixture("t3_basic", {"time_period": 5, "v_factor": 0.7}, OHLC["close"], list(out))


def gen_ma():
    out = talib.MA(OHLC["close"], timeperiod=30, matype=0)
    write_fixture("ma_basic", {"time_period": 30, "ma_type": "Sma"}, OHLC["close"], list(out))


def gen_mavp():
    import numpy as np

    periods = np.array([2 + (i % 29) for i in range(len(OHLC["close"]))], dtype=float)
    out = talib.MAVP(OHLC["close"], periods, minperiod=2, maxperiod=30, matype=0)
    write_payload(
        {
            "name": "mavp_basic",
            "params": {"min_period": 2, "max_period": 30, "ma_type": "Sma"},
            "input": to_jsonable(OHLC["close"]),
            "periods": to_jsonable(list(periods)),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_kama():
    out = talib.KAMA(OHLC["close"], timeperiod=30)
    write_fixture("kama_basic", {"time_period": 30}, OHLC["close"], list(out))


def gen_sar():
    out = talib.SAR(OHLC["high"], OHLC["low"], acceleration=0.02, maximum=0.2)
    write_payload(
        {
            "name": "sar_basic",
            "params": {"acceleration": 0.02, "maximum": 0.2},
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_sarext():
    out = talib.SAREXT(
        OHLC["high"], OHLC["low"],
        startvalue=0.0, offsetonreverse=0.0,
        accelerationinitlong=0.02, accelerationlong=0.02, accelerationmaxlong=0.2,
        accelerationinitshort=0.02, accelerationshort=0.02, accelerationmaxshort=0.2,
    )
    write_payload(
        {
            "name": "sarext_basic",
            "params": {
                "start_value": 0.0, "offset_on_reverse": 0.0,
                "accel_init_long": 0.02, "accel_long": 0.02, "accel_max_long": 0.2,
                "accel_init_short": 0.02, "accel_short": 0.02, "accel_max_short": 0.2,
            },
            "high": to_jsonable(OHLC["high"]),
            "low": to_jsonable(OHLC["low"]),
            "expected": to_jsonable(list(out)),
        }
    )


def gen_mama():
    mama_line, fama_line = talib.MAMA(OHLC["close"], fastlimit=0.5, slowlimit=0.05)
    write_payload(
        {
            "name": "mama_basic",
            "params": {"fast_limit": 0.5, "slow_limit": 0.05},
            "input": to_jsonable(OHLC["close"]),
            "mama": to_jsonable(list(mama_line)),
            "fama": to_jsonable(list(fama_line)),
        }
    )


def gen_ht_trendline():
    out = talib.HT_TRENDLINE(OHLC["close"])
    write_fixture("ht_trendline_basic", {}, OHLC["close"], list(out))


GENERATORS = [
    gen_sma, gen_ema, gen_wma, gen_dema, gen_tema, gen_midpoint, gen_midprice,
    # ---- 动量指标 / Momentum ----
    gen_mom, gen_roc, gen_rocp, gen_rocr, gen_rocr100, gen_rsi, gen_cmo, gen_trix,
    gen_stoch_rsi, gen_apo, gen_ppo, gen_macd, gen_cci, gen_mfi, gen_willr, gen_bop,
    gen_ultosc, gen_plus_dm, gen_minus_dm, gen_plus_di, gen_minus_di, gen_adx,
    gen_adxr, gen_aroon, gen_aroon_osc, gen_stoch, gen_stoch_f,
    # ---- 波动率 / Volatility ----
    gen_trange, gen_atr, gen_natr,
    # ---- 成交量 / Volume ----
    gen_ad, gen_adosc, gen_obv,
    # ---- 价格变换 / Price Transform ----
    gen_avgprice, gen_medprice, gen_typprice, gen_wclprice,
    # ---- 统计 / Statistic ----
    gen_stddev, gen_var, gen_linear_reg, gen_linear_reg_angle,
    gen_linear_reg_intercept, gen_linear_reg_slope, gen_tsf, gen_beta, gen_correl,
    # ---- 重叠研究（第二批）/ Overlap Studies (batch 2) ----
    gen_bbands, gen_trima, gen_t3, gen_ma, gen_mavp, gen_kama, gen_sar, gen_sarext,
    # ---- 周期类 / Cycle ----
    gen_mama, gen_ht_trendline,
]


def main():
    print("TA-Lib version:", talib.__version__)  # 登记此版本于 tools/README.md
    for g in GENERATORS:
        g()


if __name__ == "__main__":
    main()
