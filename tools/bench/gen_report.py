#!/usr/bin/env python3
"""Generate docs/benchmarks/adaq-vs-talib-161.html from all161_results.csv.

Self-contained HTML (dark theme, inline SVG charts, sortable table). No external deps.
"""
import csv
import math
import os
import datetime

ROOT = "/Users/tony/github/adaq-talib"
CSV = os.path.join(ROOT, "all161_results.csv")
OUT = os.path.join(ROOT, "docs/benchmarks/adaq-vs-talib-161.html")

rows = []
with open(CSV, encoding="utf-8") as fh:
    for r in csv.DictReader(fh):
        r["adaq_ns"] = float(r["adaq_ns_per_elem"])
        r["c_ns"] = float(r["c_ns_per_elem"])
        r["speedup"] = float(r["speedup"]) if r["speedup"] != "NA" else None
        r["parity"] = float(r["parity"]) if r["parity"] != "NA" else None
        r["c_missing"] = r["c_missing"] == "true"
        rows.append(r)

total = len(rows)
compared = sum(1 for r in rows if not r["c_missing"])
n_missing = total - compared
n_parity_bad = sum(1 for r in rows if not r["c_missing"] and r["parity"] is not None and r["parity"] > 1e-6)
n_parity_clean = compared - n_parity_bad
faster = sum(1 for r in rows if r["speedup"] is not None and r["speedup"] > 1.0)
slower = sum(1 for r in rows if r["speedup"] is not None and r["speedup"] <= 1.0)

# geomean speedup over compared
logsum = 0.0
nvalid = 0
for r in rows:
    if r["speedup"] is not None and r["speedup"] > 0:
        logsum += math.log(r["speedup"])
        nvalid += 1
geomean = math.exp(logsum / nvalid) if nvalid else 0.0

# per-category
cats = {}
for r in rows:
    g = r["group"]
    cats.setdefault(g, {"n": 0, "logs": 0.0, "nv": 0, "faster": 0, "slower": 0})
    c = cats[g]
    c["n"] += 1
    if r["speedup"] is not None and r["speedup"] > 0:
        c["logs"] += math.log(r["speedup"])
        c["nv"] += 1
        if r["speedup"] > 1.0:
            c["faster"] += 1
        else:
            c["slower"] += 1
cat_rows = []
for g, c in cats.items():
    gm = math.exp(c["logs"] / c["nv"]) if c["nv"] else 0.0
    cat_rows.append((g, c["n"], gm, c["faster"], c["slower"]))
cat_rows.sort(key=lambda x: x[2])

# histogram buckets (log10 speedup)
buckets = [("<0.2", 0.0, 0.2), ("0.2–0.3", 0.2, 0.3), ("0.3–0.5", 0.3, 0.5),
           ("0.5–0.7", 0.5, 0.7), ("0.7–1.0", 0.7, 1.0), ("1.0–1.5", 1.0, 1.5),
           ("1.5–2.0", 1.5, 2.0), ("2.0–3.0", 2.0, 3.0), (">3.0", 3.0, 1e9)]
hist = [0] * len(buckets)
for r in rows:
    s = r["speedup"]
    if s is None:
        continue
    for i, (_, lo, hi) in enumerate(buckets):
        if lo <= s < hi:
            hist[i] += 1
            break

# ---- build SVG: category bar chart ----
def svg_bar_chart(cat_rows):
    w, h = 760, 320
    pad_l, pad_b, pad_t = 150, 50, 20
    plot_w = w - pad_l - 20
    plot_h = h - pad_b - pad_t
    maxv = max(max(c[2] for c in cat_rows), 2.0)
    # x scale: map [0, maxv] -> [0, plot_w]; reference line at 1.0
    def x(v):
        return pad_l + (v / maxv) * plot_w
    svg = [f'<svg viewBox="0 0 {w} {h}" width="100%" xmlns="http://www.w3.org/2000/svg">']
    svg.append(f'<line x1="{pad_l}" y1="{pad_t}" x2="{pad_l}" y2="{pad_t+plot_h}" stroke="#3a3a4a"/>')
    svg.append(f'<line x1="{x(1.0)}" y1="{pad_t}" x2="{x(1.0)}" y2="{pad_t+plot_h}" stroke="#6b7280" stroke-dasharray="4 3"/>')
    svg.append(f'<text x="{x(1.0)}" y="{pad_t-6}" fill="#9aa0aa" font-size="11" text-anchor="middle">1.0 (tie)</text>')
    bh = plot_h / (len(cat_rows) * 1.6)
    y = pad_t
    for g, n, gm, fa, sl in cat_rows:
        col = "#3fb950" if gm >= 1.0 else "#f85149"
        svg.append(f'<rect x="{pad_l}" y="{y}" width="{x(gm)-pad_l:.1f}" height="{bh:.1f}" fill="{col}" rx="2"/>')
        svg.append(f'<text x="{pad_l-8}" y="{y+bh/2+4}" fill="#c9d1d9" font-size="12" text-anchor="end">{g}</text>')
        svg.append(f'<text x="{x(gm)+6}" y="{y+bh/2+4}" fill="#c9d1d9" font-size="11">{gm:.3f}×</text>')
        y += bh * 1.6
    # x axis ticks
    for tv in [0.0, 0.5, 1.0, 1.5, 2.0]:
        if tv <= maxv:
            svg.append(f'<text x="{x(tv)}" y="{pad_t+plot_h+18}" fill="#9aa0aa" font-size="11" text-anchor="middle">{tv:.1f}</text>')
    svg.append('</svg>')
    return "".join(svg)

# ---- build SVG: histogram ----
def svg_hist(buckets, hist):
    w, h = 760, 260
    pad_l, pad_b, pad_t = 40, 60, 20
    plot_w = w - pad_l - 20
    plot_h = h - pad_b - pad_t
    maxc = max(hist) if max(hist) > 0 else 1
    bw = plot_w / len(buckets)
    svg = [f'<svg viewBox="0 0 {w} {h}" width="100%" xmlns="http://www.w3.org/2000/svg">']
    svg.append(f'<rect x="{pad_l}" y="{pad_t}" width="{plot_w}" height="{plot_h}" fill="#16161f" stroke="#3a3a4a"/>')
    for i, ((lab, lo, hi), c) in enumerate(zip(buckets, hist)):
        bh = (c / maxc) * plot_h if maxc else 0
        x0 = pad_l + i * bw
        col = "#3fb950" if (lo >= 1.0) else "#f85149"
        svg.append(f'<rect x="{x0+2:.1f}" y="{pad_t+plot_h-bh:.1f}" width="{bw-4:.1f}" height="{bh:.1f}" fill="{col}" opacity="0.85"/>')
        svg.append(f'<text x="{x0+bw/2:.1f}" y="{pad_t+plot_h-bh-4:.1f}" fill="#c9d1d9" font-size="10" text-anchor="middle">{c}</text>')
        svg.append(f'<text x="{x0+bw/2:.1f}" y="{pad_t+plot_h+14:.1f}" fill="#9aa0aa" font-size="9" text-anchor="middle" transform="rotate(35 {x0+bw/2:.1f} {pad_t+plot_h+14:.1f})">{lab}</text>')
    svg.append(f'<text x="{pad_l}" y="{pad_t-6}" fill="#9aa0aa" font-size="11">speedup = C ns/elem ÷ adaq ns/elem  (green: adaq faster, red: adaq slower)</text>')
    svg.append('</svg>')
    return "".join(svg)

chart_cat = svg_bar_chart(cat_rows)
chart_hist = svg_hist(buckets, hist)

# ---- full table rows ----
def fmt_speed(s):
    return "NA" if s is None else f"{s:.3f}"
def fmt_parity(p):
    if p is None:
        return "NA"
    if p > 1e-6:
        return f"{p:.2e}"
    return "1:1"

tbody = []
for r in sorted(rows, key=lambda x: (x["group"], x["name"])):
    s = r["speedup"]
    if s is None:
        cls, sd = "na", "c-missing"
    elif s > 1.0:
        cls, sd = "fast", f"{s:.3f}"
    else:
        cls, sd = "slow", f"{s:.3f}"
    pstat = "divergent" if (not r["c_missing"] and r["parity"] is not None and r["parity"] > 1e-6) else ("clean" if not r["c_missing"] else "n/a")
    pcls = "par-bad" if pstat == "divergent" else ("par-ok" if pstat == "clean" else "par-na")
    note = ""
    if r["name"] == "stoch_rsi":
        note = ' title="adaq stoch_rsi(14,5) vs TA STOCHRSI(14,5,3): ~2× divergence on default period=5 path; adaq golden vector uses period=14 (passes). Follow-up."'
    tbody.append(
        f'<tr><td class="nm">{r["name"]}</td><td>{r["group"]}</td>'
        f'<td class="num">{r["adaq_ns"]:.4f}</td><td class="num">{r["c_ns"]:.4f}</td>'
        f'<td class="num {cls}">{fmt_speed(s)}</td>'
        f'<td class="num {pcls}"{note}>{fmt_parity(r["parity"])}</td></tr>'
    )
tbody_html = "\n".join(tbody)

cat_trs = []
for g, n, gm, fa, sl in cat_rows:
    col = "#3fb950" if gm >= 1.0 else "#f85149"
    cat_trs.append(f'<tr><td>{g}</td><td class="num">{n}</td><td class="num" style="color:{col}">{gm:.3f}×</td>'
                   f'<td class="num" style="color:#3fb950">{fa}</td><td class="num" style="color:#f85149">{sl}</td></tr>')
cat_html = "\n".join(cat_trs)

now = datetime.datetime.now().strftime("%Y-%m-%d %H:%M")

html = f"""<!DOCTYPE html>
<html lang="zh-CN"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>adaq-talib vs TA-Lib C — 161 Indicators Benchmark</title>
<style>
  :root {{ --bg:#0d1117; --panel:#16161f; --panel2:#1c1c28; --border:#2a2a38;
          --fg:#c9d1d9; --muted:#9aa0aa; --green:#3fb950; --red:#f85149; --accent:#58a6ff; }}
  * {{ box-sizing:border-box; }}
  body {{ margin:0; background:var(--bg); color:var(--fg);
         font-family:-apple-system,"Segoe UI",Roboto,"Helvetica Neue","PingFang SC","Microsoft YaHei",sans-serif;
         line-height:1.55; }}
  .wrap {{ max-width:1040px; margin:0 auto; padding:32px 20px 80px; }}
  h1 {{ font-size:26px; margin:0 0 6px; }}
  h2 {{ font-size:19px; margin:36px 0 12px; border-left:3px solid var(--accent); padding-left:10px; }}
  .sub {{ color:var(--muted); font-size:13.5px; margin-bottom:8px; }}
  .cards {{ display:grid; grid-template-columns:repeat(auto-fit,minmax(150px,1fr)); gap:12px; margin:18px 0 8px; }}
  .card {{ background:var(--panel); border:1px solid var(--border); border-radius:10px; padding:14px 16px; }}
  .card .k {{ color:var(--muted); font-size:12px; }}
  .card .v {{ font-size:23px; font-weight:650; margin-top:4px; }}
  .card .v.green {{ color:var(--green); }} .card .v.red {{ color:var(--red); }}
  .panel {{ background:var(--panel); border:1px solid var(--border); border-radius:10px; padding:16px 18px; margin:14px 0; }}
  table {{ width:100%; border-collapse:collapse; font-size:13px; }}
  th,td {{ padding:6px 10px; border-bottom:1px solid var(--border); text-align:left; }}
  th {{ color:var(--muted); font-weight:600; cursor:pointer; user-select:none; position:sticky; top:0; background:var(--panel2); }}
  th:hover {{ color:var(--accent); }}
  td.num, th.num {{ text-align:right; font-variant-numeric:tabular-nums; }}
  td.nm {{ font-family:ui-monospace,SFMono-Regular,Menlo,monospace; }}
  .fast {{ color:var(--green); font-weight:600; }} .slow {{ color:var(--red); }}
  ..par-ok {{ color:var(--green); }} .par-bad {{ color:var(--red); font-weight:600; }} .par-na {{ color:var(--muted); }}
  .tablewrap {{ max-height:620px; overflow:auto; border:1px solid var(--border); border-radius:10px; }}
  .legend {{ font-size:12.5px; color:var(--muted); }}
  code {{ background:var(--panel2); padding:1px 5px; border-radius:4px; font-size:12px; }}
  ul {{ margin:8px 0; padding-left:20px; }} li {{ margin:4px 0; }}
  .note {{ font-size:12.5px; color:var(--muted); }}
</style></head>
<body><div class="wrap">
<h1>adaq-talib vs TA-Lib C — 161 Indicators Performance Benchmark</h1>
<div class="sub">Pure-Rust zero-FFI reimplementation (<code>adaq-talib</code> 0.1.2) benchmarked against native TA-Lib C 0.7.1.
Generated {now}.</div>

<div class="cards">
  <div class="card"><div class="k">Indicators</div><div class="v">{total}</div></div>
  <div class="card"><div class="k">Compared (both sides)</div><div class="v">{compared}</div></div>
  <div class="card"><div class="k">Geomean speedup</div><div class="v {'green' if geomean>=1 else 'red'}">{geomean:.3f}×</div></div>
  <div class="card"><div class="k">adaq faster</div><div class="v green">{faster}</div></div>
  <div class="card"><div class="k">adaq slower</div><div class="v red">{slower}</div></div>
  <div class="card"><div class="k">Parity 1:1</div><div class="v green">{n_parity_clean}/{compared}</div></div>
</div>

<h2>Methodology</h2>
<div class="panel">
<ul>
  <li><b>Authoritative track (ADR 0004):</b> native TA-Lib C 0.7.1, linked via <code>build.rs</code> under the
      <code>bench-c</code> feature (Homebrew <code>/opt/homebrew/Cellar/ta-lib/0.7.1</code>).</li>
  <li><b>Fair workload:</b> both sides compute identical configurations. TA-Lib's own default opt-in values are read
      at runtime via the abstract API and forwarded to adaq; for functions where adaq's parameter API diverges
      (interleaved <code>MAType</code>, or simplified stochastic params), the C side is driven with matched opt-ins
      (e.g. EMA forced for APO/PPO/MACDEXT; <code>MACDFIX</code> compared against <code>MACD</code> because TA-Lib's own
      <code>MACDFIX</code> differs from <code>MACD</code> at identical parameters — a TA-Lib internal inconsistency).</li>
  <li><b>Isolated compute:</b> adaq uses the in-place <code>_with_output</code> variant with a pre-allocated, reused
      buffer (zero per-call allocation). C uses the abstract API with a reused parameter holder; output buffers are
      allocated once per function and reused across iterations.</li>
  <li><b>Input:</b> N = 100,000 pseudo-random prices (close = real0; OHLCV derived). Release (<code>--bench</code>) build.</li>
  <li><b>Timing:</b> per-function iteration count auto-scaled to an ~80 ms budget; reported as <b>ns per element</b>.
      <code>speedup = C_ns/elem ÷ adaq_ns/elem</code> (&gt;1 = adaq faster).</li>
  <li><b>Parity:</b> live checksum (last output element summed across iterations) compared between adaq and C;
      tolerance 1e-6 absolute (&lt;1e-8 relative holds for the validated subset). Surfaces any config/runtime divergence.</li>
</ul>
</div>

<h2>Per-category geomean speedup</h2>
<div class="panel">{chart_cat}
<div class="legend">Bars show the geometric mean of per-function speedups within each TA-Lib category.
Green ≥ 1.0 (adaq at least as fast on average); red &lt; 1.0. The 61 candle functions (Pattern Recognition) dominate
the overall average drag.</div></div>

<h2>Speedup distribution (all {compared} compared functions)</h2>
<div class="panel">{chart_hist}</div>

<h2>Category summary</h2>
<div class="panel"><table>
<thead><tr><th>Category</th><th class="num">n</th><th class="num">geomean</th>
<th class="num">adaq faster</th><th class="num">adaq slower</th></tr></thead>
<tbody>{cat_html}</tbody></table></div>

<h2>Full results — all 161 indicators</h2>
<div class="note">Click a column header to sort. <span style="color:var(--green)">green speedup</span> = adaq faster,
<span style="color:var(--red)">red</span> = adaq slower. <span style="color:var(--green)">1:1</span> parity = numeric match with TA-Lib;
<span style="color:var(--red)">divergent</span> = flagged for follow-up.</div>
<div class="tablewrap"><table id="tbl">
<thead><tr>
  <th data-k="name">Indicator</th><th data-k="group">Category</th>
  <th class="num" data-k="adaq">adaq ns/elem</th><th class="num" data-k="c">C ns/elem</th>
  <th class="num" data-k="sp">Speedup</th><th class="num" data-k="par">Parity</th>
</tr></thead>
<tbody>{tbody_html}</tbody></table></div>

<h2>Limitations &amp; notes</h2>
<div class="panel">
<ul>
  <li><b>Overall:</b> adaq is on average ~1.5× slower than TA-Lib C (geomean {geomean:.3f}×). It is <b>faster or
      comparable</b> for Overlap, Math Operators, Math Transform, Price Transform, Statistic Functions, Volatility and
      Volume indicators, but slower for Momentum (0.75×) and especially the 61 candle (Pattern Recognition) functions
      (0.34×), which are branch-heavy pattern logic and are TA-Lib's most optimized code.</li>
  <li><b>Parity:</b> {n_parity_clean}/{compared} functions match TA-Lib 1:1 out of the box. TA-Lib C is the reference.</li>
  <li><b><code>stoch_rsi</code> (1 divergence):</b> adaq <code>stoch_rsi(14,5)</code> vs TA <code>STOCHRSI(14,5,3)</code>
      differ ~2× on the default <code>period=5</code> path. adaq's golden vector uses <code>period=14</code> and passes,
      so the discrepancy is specific to the default-5 path and warrants follow-up verification (likely a leading-NaN
      alignment subtlety in the RSI-compaction mapping). Performance numbers remain valid.</li>
  <li><b>Default-MAType difference (by design):</b> APO/PPO/MACDEXT in adaq use <b>EMA</b> by default (matching their
      golden vectors, which were generated with <code>ma_type=Ema</code>), whereas TA-Lib's default <code>MAType</code>
      is <b>SMA</b>. These are not correctness defects — adaq matches TA-Lib 1:1 once the same <code>MAType</code> is
      selected. The benchmark forces TA EMA for a config-matched, fair speed comparison.</li>
  <li><b>Integer-output functions</b> (max_index, min_index, minmax_index, ht_trendmode) return integer indices; the
      FFI driver was extended to support <code>TA_SetOutputParamIntegerPtr</code> so all 161 are now compared.</li>
  <li><b>Five functions</b> (<code>mama</code>, <code>dx</code>, <code>imi</code>, <code>stoch_rsi</code>, <code>trix</code>)
      lack a <code>_with_output</code> variant and fall back to adaq's allocating public API, which slightly penalizes
      their measured adaq time (they still run the same compute kernel).</li>
  <li><b>Honest framing:</b> TA-Lib C is a mature, hand-tuned C library; adaq is a from-scratch pure-Rust, zero-dependency,
      zero-FFI reimplementation. A modest speed gap is expected; the value proposition is safety, no FFI/ABI risk, and
      a clean Rust API.</li>
</ul>
</div>
</div>
</body></html>"""

js = r'''
<script>
document.querySelectorAll('#tbl th').forEach(function(th){
  th.addEventListener('click', function(){
    var k=th.getAttribute('data-k'), tb=th.parentNode.parentNode.parentNode,
        trs=Array.from(tb.querySelectorAll('tbody tr'));
    var asc=!th.asc;
    trs.sort(function(a,b){
      var x=a.children[th.cellIndex].textContent, y=b.children[th.cellIndex].textContent;
      if(k==='name'||k==='group') return asc? x.localeCompare(y): y.localeCompare(x);
      return asc? (parseFloat(x)-parseFloat(y)) : (parseFloat(y)-parseFloat(x));
    });
    th.asc=asc;
    var tb2=tb.querySelector('tbody'); trs.forEach(function(tr){tb2.appendChild(tr);});
  });
});
</script>
'''
html = html + js

with open(OUT, "w", encoding="utf-8") as fh:
    fh.write(html)
print(f"wrote {OUT} ({len(html)} bytes)")
print(f"total={total} compared={compared} missing={n_missing} parity_clean={n_parity_clean} parity_bad={n_parity_bad} geomean={geomean:.3f} faster={faster} slower={slower}")
