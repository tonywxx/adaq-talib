#!/usr/bin/env python3
"""adaq-talib 函数计数与全量对标对账（见 ADR 0010 P0-4 / `0.1.0-scope.md`）。

Counts `pub fn` (剔除 `_default` 便捷变体) across the 7 public indicator modules and
reconciles them against the authoritative TA-Lib 0.7.1 function set (161 public functions,
group classification captured live from `talib.abstract.Function(name).info['group']`).

This script is dependency-free: the authoritative 161-name group map is baked in, so it
runs in CI / on machines WITHOUT TA-Lib installed. If `talib` IS importable it prefers the
live classification as a self-check.

Usage:
    python3 tools/reconcile.py          # print table + exit 0/1
    python3 tools/reconcile.py --json    # machine-readable summary
"""
from __future__ import annotations

import glob
import json
import os
import re
import sys

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# Public indicator modules (do NOT include `core` / `utils` internal primitives).
MODULES = [
    "overlap", "momentum", "volatility", "volume",
    "price_transform", "stat", "cycle",
]

# Authoritative TA-Lib 0.7.1 function → group, captured live from the installed
# `talib` 0.7.1 library (`abstract.Function(name).info['group']`).
TALIB_GROUPS = {
    "Cycle Indicators": ["HT_DCPERIOD", "HT_DCPHASE", "HT_PHASOR", "HT_SINE", "HT_TRENDMODE"],
    "Math Operators": ["ADD", "DIV", "MAX", "MAXINDEX", "MIN", "MININDEX", "MINMAX",
                       "MINMAXINDEX", "MULT", "SUB", "SUM"],
    "Math Transform": ["ACOS", "ASIN", "ATAN", "CEIL", "COS", "COSH", "EXP", "FLOOR",
                       "LN", "LOG10", "SIN", "SINH", "SQRT", "TAN", "TANH"],
    "Momentum Indicators": ["ADX", "ADXR", "APO", "AROON", "AROONOSC", "BOP", "CCI", "CMO",
                            "DX", "IMI", "MACD", "MACDEXT", "MACDFIX", "MFI", "MINUS_DI",
                            "MINUS_DM", "MOM", "PLUS_DI", "PLUS_DM", "PPO", "ROC", "ROCP",
                            "ROCR", "ROCR100", "RSI", "STOCH", "STOCHF", "STOCHRSI", "TRIX",
                            "ULTOSC", "WILLR"],
    "Overlap Studies": ["ACCBANDS", "BBANDS", "DEMA", "EMA", "HT_TRENDLINE", "KAMA", "MA",
                        "MAMA", "MAVP", "MIDPOINT", "MIDPRICE", "SAR", "SAREXT", "SMA", "T3",
                        "TEMA", "TRIMA", "WMA"],
    "Pattern Recognition": ["CDL2CROWS", "CDL3BLACKCROWS", "CDL3INSIDE", "CDL3LINESTRIKE",
                            "CDL3OUTSIDE", "CDL3STARSINSOUTH", "CDL3WHITESOLDIERS",
                            "CDLABANDONEDBABY", "CDLADVANCEBLOCK", "CDLBELTHOLD", "CDLBREAKAWAY",
                            "CDLCLOSINGMARUBOZU", "CDLCONCEALBABYSWALL", "CDLCOUNTERATTACK",
                            "CDLDARKCLOUDCOVER", "CDLDOJI", "CDLDOJISTAR", "CDLDRAGONFLYDOJI",
                            "CDLENGULFING", "CDLEVENINGDOJISTAR", "CDLEVENINGSTAR",
                            "CDLGAPSIDESIDEWHITE", "CDLGRAVESTONEDOJI", "CDLHAMMER",
                            "CDLHANGINGMAN", "CDLHARAMI", "CDLHARAMICROSS", "CDLHIGHWAVE",
                            "CDLHIKKAKE", "CDLHIKKAKEMOD", "CDLHOMINGPIGEON", "CDLIDENTICAL3CROWS",
                            "CDLINNECK", "CDLINVERTEDHAMMER", "CDLKICKING", "CDLKICKINGBYLENGTH",
                            "CDLLADDERBOTTOM", "CDLLONGLEGGEDDOJI", "CDLLONGLINE", "CDLMARUBOZU",
                            "CDLMATCHINGLOW", "CDLMATHOLD", "CDLMORNINGDOJISTAR", "CDLMORNINGSTAR",
                            "CDLONNECK", "CDLPIERCING", "CDLRICKSHAWMAN", "CDLRISEFALL3METHODS",
                            "CDLSEPARATINGLINES", "CDLSHOOTINGSTAR", "CDLSHORTLINE",
                            "CDLSPINNINGTOP", "CDLSTALLEDPATTERN", "CDLSTICKSANDWICH", "CDLTAKURI",
                            "CDLTASUKIGAP", "CDLTHRUSTING", "CDLTRISTAR", "CDLUNIQUE3RIVER",
                            "CDLUPSIDEGAP2CROWS", "CDLXSIDEGAP3METHODS"],
    "Price Transform": ["AVGDEV", "AVGPRICE", "MEDPRICE", "TYPPRICE", "WCLPRICE"],
    "Statistic Functions": ["BETA", "CORREL", "LINEARREG", "LINEARREG_ANGLE",
                            "LINEARREG_INTERCEPT", "LINEARREG_SLOPE", "STDDEV", "TSF", "VAR"],
    "Volatility Indicators": ["ATR", "NATR", "TRANGE"],
    "Volume Indicators": ["AD", "ADOSC", "OBV"],
}

# Project snake_case names whose separator differs from TA-Lib's (e.g. `aroon_osc` → AROONOSC).
NAME_ALIASES = {
    "aroon_osc": "AROONOSC",
    "linear_reg": "LINEARREG",
    "linear_reg_angle": "LINEARREG_ANGLE",
    "linear_reg_intercept": "LINEARREG_INTERCEPT",
    "linear_reg_slope": "LINEARREG_SLOPE",
    "macd_ext": "MACDEXT",
    "macd_fix": "MACDFIX",
    "stoch_f": "STOCHF",
    "stoch_rsi": "STOCHRSI",
}

# Current published scope target (see `0.1.0-scope.md`). Soft-checked; bump as P4 lands.
TARGET_PUB_FN = 65


def collect_project_functions():
    """Return {module: [pub fn names (non-_default)]} and the flat unique set."""
    per_module: dict[str, list[str]] = {}
    for mod in MODULES:
        names: list[str] = []
        for path in glob.glob(os.path.join(REPO_ROOT, "src", f"{mod}.rs")) + \
                glob.glob(os.path.join(REPO_ROOT, "src", mod, "*.rs")):
            with open(path, encoding="utf-8") as fh:
                src = fh.read()
            names += re.findall(r"^\s*pub\s+fn\s+(\w+)", src, re.M)
        per_module[mod] = [n for n in names if not n.endswith("_default")]
    return per_module


def to_talib_name(project_fn: str) -> str:
    return NAME_ALIASES.get(project_fn, project_fn.upper())


def main() -> int:
    as_json = "--json" in sys.argv
    per_module = collect_project_functions()
    flat = sorted({n for ns in per_module.values() for n in ns})

    all_talib = sorted(n for g in TALIB_GROUPS.values() for n in g)
    mapped = {to_talib_name(n) for n in flat}
    unmatched = sorted(mapped - set(all_talib))
    covered = mapped & set(all_talib)
    gap = sorted(set(all_talib) - mapped)

    non_default_total = sum(len(v) for v in per_module.values())

    # Coverage per group
    group_rows = []
    for g in TALIB_GROUPS:
        members = set(TALIB_GROUPS[g])
        done = sorted(members & mapped)
        miss = sorted(members - mapped)
        group_rows.append((g, len(members), len(done), miss))

    # Self-check: if talib is importable, prefer live classification as a cross-check.
    live_note = "baked authoritative map (no live talib)"
    try:
        import talib  # type: ignore
        from talib import abstract  # type: ignore
        live = {}
        for nm in talib.get_functions():
            live[nm] = abstract.Function(nm).info["group"]
        # Compare group membership for covered names
        mismatches = []
        proj_to_talib = {n: to_talib_name(n) for n in flat}
        for n, tl in proj_to_talib.items():
            if tl in live and live[tl] != _group_of(tl):
                mismatches.append((tl, live[tl], _group_of(tl)))
        live_note = f"live talib 0.7.1 (cross-check mismatches={len(mismatches)})"
        if mismatches:
            print("WARN: live group classification differs from baked map:", mismatches,
                  file=sys.stderr)
    except Exception:
        pass

    if as_json:
        print(json.dumps({
            "project_pub_fn_non_default": non_default_total,
            "target_pub_fn": TARGET_PUB_FN,
            "talib_total": len(all_talib),
            "covered": len(covered),
            "gap": len(gap),
            "unmatched_project_names": unmatched,
            "gap_list": gap,
            "per_module": per_module,
            "per_group": [{"group": g, "total": t, "done": d, "missing": m}
                          for (g, t, d, m) in group_rows],
            "source": live_note,
        }, indent=2, ensure_ascii=False))
        return 0 if not unmatched else 1

    # Human-readable table
    print("=" * 64)
    print("adaq-talib · 函数计数与 TA-Lib 0.7.1 全量对标对账")
    print("=" * 64)
    print(f"\n[1] 项目对外函数（7 模块 pub fn，剔除 _default）")
    for mod in MODULES:
        print(f"    {mod:15s} {len(per_module[mod]):3d}")
    print(f"    {'TOTAL':15s} {non_default_total:3d}   (target {TARGET_PUB_FN})")
    if non_default_total != TARGET_PUB_FN:
        print(f"    ⚠ 与 TARGET_PUB_FN={TARGET_PUB_FN} 不一致（随 P4 推进会变化，请同步更新）")

    print(f"\n[2] TA-Lib 0.7.1 覆盖（source: {live_note}）")
    print(f"    {'Group':20s} {'Done':>5s}/{'Total':<5s}  Missing")
    for (g, t, d, m) in group_rows:
        miss = ", ".join(m) if m else "—"
        print(f"    {g:20s} {d:5d}/{t:<5d}  {miss}")
    print(f"    {'TOTAL':20s} {len(covered):5d}/{len(all_talib)}")

    print(f"\n[3] 结论")
    print(f"    已实现对外函数 : {non_default_total}")
    print(f"    TA-Lib 0.7.1   : {len(all_talib)}")
    print(f"    剩余缺口       : {len(gap)}")
    if unmatched:
        print(f"    ❌ 项目存在无法映射到 TA-Lib 的函数: {unmatched}")
        return 1
    if len(covered) != non_default_total:
        print(f"    ❌ 覆盖数({len(covered)}) ≠ 项目函数数({non_default_total})，"
              f"可能存在重复映射或漏计")
        return 1
    print("    ✅ 每个项目函数均 1:1 对应一个 TA-Lib 0.7.1 函数；口径自洽。")
    return 0


def _group_of(talib_name: str) -> str:
    for g, members in TALIB_GROUPS.items():
        if talib_name in members:
            return g
    return "?"


if __name__ == "__main__":
    raise SystemExit(main())
