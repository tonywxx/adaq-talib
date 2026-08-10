#!/usr/bin/env python3
"""adaq-talib 基准 · Python 便捷对照（见 ADR 0004 双轨基准）。

以 TA-Lib **Python 绑定**（`talib`，底层仍是 TA-Lib C 0.7.1）为对照，
计时热路径指标并打印 ns/elem。

⚠️ 口径说明 / Disclaimer：
    此处计时的 Python 绑定经过 CPython ↔ C 的 FFI 与 ndarray 拷贝开销，
    **不等同于原生 C**（`bench-c` feature 下的 FFI 才是原生 C 对照）。
    本脚本仅用于"量级参考"，不应作为 P3 SIMD 评估闸门的比较基准。

运行 / Run:
    python3 tools/bench/compare.py

需要：系统安装 TA-Lib C + `pip install TA-Lib`；无 talib 时本脚本跳过并提示。
"""
from __future__ import annotations

import time

N = 1_000_000
PERIOD = 20
ITERS = 20


def sample_prices(n: int) -> list[float]:
    prices = []
    x = 12345.0
    for _ in range(n):
        x = (x * 1103515245.0 + 12345.0) % 1e9
        prices.append(50.0 + (x / 1e9) * 10.0)
    return prices


def bench(name: str, func, *args):
    start = time.perf_counter()
    last = None
    for _ in range(ITERS):
        out = func(*args)
        last = out[-1] if hasattr(out, "__len__") and len(out) else out
    elapsed = time.perf_counter() - start
    ns_per_elem = (elapsed / ITERS / N) * 1e9
    print(f"  {name:10s} ns/elem = {ns_per_elem:7.2f}   (last={last})")


def main() -> int:
    try:
        import talib  # type: ignore
        import numpy as np  # type: ignore
    except Exception as e:
        print(f"SKIP: TA-Lib Python binding unavailable ({e}); install via `pip install TA-Lib`.")
        return 0

    print(f"TA-Lib Python binding {talib.__version__} — 便捷对照（非原生 C 口径）")
    print(f"N={N}, PERIOD={PERIOD}, ITERS={ITERS}\n")

    prices = sample_prices(N)
    high = [p * 1.01 for p in prices]
    low = [p * 0.99 for p in prices]
    pa = np.asarray(prices, dtype=float)
    ha = np.asarray(high, dtype=float)
    la = np.asarray(low, dtype=float)

    bench("SMA", talib.SMA, pa, PERIOD)
    bench("DEMA", talib.DEMA, pa, PERIOD)
    bench("TEMA", talib.TEMA, pa, PERIOD)
    bench("T3", talib.T3, pa, PERIOD, 0.7)
    bench("WMA", talib.WMA, pa, PERIOD)
    bench("MIDPOINT", talib.MIDPOINT, ha, PERIOD)
    bench("MIDPRICE", talib.MIDPRICE, ha, la, PERIOD)
    bench("BBANDS", lambda p: talib.BBANDS(p, PERIOD, 2.0, 2.0, 0), pa)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
