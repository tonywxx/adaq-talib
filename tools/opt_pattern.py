#!/usr/bin/env python3
"""Parity-preserving inline-expansion of CandleAvg in adaq-talib pattern functions.

Replaces each `CandleAvg::new(SET, open, high, low, close, lookback, OFF)` +
`avg.value(i, ...)` / `avg.advance(i, ...)` with inline running-sum accumulators
that replicate CandleAvg's EXACT recurrence (warm-up sum, value formula, advance
delta). The condition, leading let-statements, and output expressions are copied
verbatim (with `avg.value(...)` -> `val_*` substituted), so golden-vector parity
is preserved by construction. Strict: skips any function it cannot parse
confidently, reporting it for manual handling.
"""
import re
import sys

SETS = {
    'BODY_LONG': ('RealBody', 10, 1.0),
    'BODY_VERY_LONG': ('RealBody', 10, 3.0),
    'BODY_SHORT': ('RealBody', 10, 1.0),
    'BODY_DOJI': ('HighLow', 10, 0.1),
    'SHADOW_LONG': ('RealBody', 0, 1.0),
    'SHADOW_VERY_LONG': ('RealBody', 0, 2.0),
    'SHADOW_SHORT': ('Shadows', 10, 1.0),
    'SHADOW_VERY_SHORT': ('HighLow', 10, 0.1),
    'NEAR': ('HighLow', 5, 0.2),
    'FAR': ('HighLow', 5, 0.6),
    'EQUAL': ('HighLow', 5, 0.05),
}


def range_of_expr(rt, o, h, l, c):
    if rt == 'RealBody':
        return f"real_body({o}, {c})"
    if rt == 'HighLow':
        return f"high_low_range({h}, {l})"
    if rt == 'Shadows':
        return f"(upper_shadow({o}, {h}, {c}) + lower_shadow({o}, {l}, {c}))"
    raise ValueError(rt)


def find_matching_brace(s, start):
    depth = 0
    for i in range(start, len(s)):
        if s[i] == '{':
            depth += 1
        elif s[i] == '}':
            depth -= 1
            if depth == 0:
                return i
    return -1


NEW_RE = re.compile(
    r'let mut (\w+) = CandleAvg::new\((\w+),\s*open,\s*high,\s*low,\s*close,\s*lookback,\s*(\d+)\);'
)


def transform_function(text):
    news = list(NEW_RE.finditer(text))
    if not news:
        return None, "no CandleAvg::new"
    avgs = []
    for m in news:
        var, setname, off = m.group(1), m.group(2), int(m.group(3))
        if setname not in SETS:
            return None, f"unknown setting {setname}"
        rt, p, f = SETS[setname]
        avgs.append((var, setname, off, rt, p, f))

    first_new_pos = news[0].start()
    nl = text.rfind('\n', 0, first_new_pos)
    prelude = text[:nl + 1] if nl != -1 else text[:first_new_pos]

    wm = re.search(r'while i < n \{', text)
    if not wm:
        return None, "no while loop"
    wopen = wm.end() - 1
    wclose = find_matching_brace(text, wopen)
    if wclose == -1:
        return None, "no while close"
    while_body = text[wm.end(): wclose]

    # Globally substitute avg.value(i, ...) -> val_*, including leading let-statements.
    body2 = while_body
    for (var, *_) in avgs:
        body2 = re.sub(r'\b' + re.escape(var) + r'\.value\(i, open, high, low, close\)',
                       f'val_{var}', body2)

    stripped = body2.lstrip()
    is_b = stripped.startswith('out[i] = if') or stripped.startswith('out[i]=if')
    then_pre = ""
    if is_b:
        aim = re.search(r'out\[i\]\s*=\s*if\s', body2)
        if not aim:
            return None, "pattern B parse fail"
        cond_start = aim.end()
        depth = 0
        i = cond_start
        while i < len(body2):
            c = body2[i]
            if c == '(':
                depth += 1
            elif c == ')':
                depth -= 1
            elif c == '{':
                if depth == 0:
                    break
            i += 1
        then_open = i
        then_close = find_matching_brace(body2, then_open)
        then_body = body2[then_open + 1:then_close].strip()
        out_expr = then_body
        cond = body2[cond_start:i].strip()
        pre_if = ""
        # else block
        rest = body2[then_close + 1:]
        eo = rest.find('{')
        if eo == -1:
            return None, "no else (B)"
        ec = find_matching_brace(rest, eo)
        else_body = rest[eo + 1:ec].strip()
        if '0.0' not in else_body.replace(' ', ''):
            return None, f"else not 0.0: {else_body}"
    else:
        me = re.search(r'\}\s*else\s*\{\s*out\[i\]\s*=\s*0\.0;?\s*\}', body2)
        if not me:
            return None, "no main else (out[i]=0.0)"
        then_close = me.start()
        depth = 1
        j = then_close - 1
        while j >= 0:
            c = body2[j]
            if c == '}':
                depth += 1
            elif c == '{':
                depth -= 1
                if depth == 0:
                    break
            j -= 1
        then_open = j
        then_body = body2[then_open + 1:then_close].strip()
        # Only the simple "single out[i] = X" then-block is transformable.
        # Nested ifs / multiple assignments (e.g. tristar's default+override) must be skipped.
        out_count = len(re.findall(r'out\[i\]\s*=', then_body))
        if out_count != 1:
            return None, f"then-block has {out_count} out assignments (nested logic)"
        outm = re.search(r'out\[i\]\s*=\s*([^;]+);', then_body)
        if not outm:
            return None, "no out assignment in then"
        out_expr = outm.group(1).strip()
        then_pre = then_body[:outm.start()].strip()
        # main `if` keyword position
        p = then_open
        found = False
        while p >= 0:
            if body2[p:p + 2] == 'if' and (p == 0 or not body2[p - 1].isalnum()):
                cond_start = p + 2
                found = True
                break
            p -= 1
        if not found:
            return None, "no if before then"
        cond = body2[cond_start:then_open].strip()
        pre_if = body2[:p]

    warm = ""
    loop_pre = ""
    advances = ""
    for (var, setname, off, rt, p, f) in avgs:
        begin = f"(lookback - {off} - {p})"
        end = f"(lookback - {off})"
        idx = "i" if off == 0 else f"(i - {off})"
        cur_expr = range_of_expr(rt, f"open[{idx}]", f"high[{idx}]", f"low[{idx}]", f"close[{idx}]")
        trail_expr = range_of_expr(rt, f"open[trail_{var}]", f"high[trail_{var}]",
                                   f"low[trail_{var}]", f"close[trail_{var}]")
        shadows = (rt == 'Shadows')
        warm += (
            f"    let mut sum_{var} = {{\n"
            f"        let mut s = {begin};\n"
            f"        let mut acc = 0.0_f64;\n"
            f"        while s < {end} {{\n"
            f"            acc += {range_of_expr(rt, 'open[s]', 'high[s]', 'low[s]', 'close[s]')};\n"
            f"            s += 1;\n"
            f"        }}\n"
            f"        acc\n"
            f"    }};\n"
            f"    let mut trail_{var} = {begin};\n"
        )
        if p == 0:
            val_line = f"        let val_{var} = cur_{var} * {f}{' / 2.0' if shadows else ''};\n"
        else:
            val_line = (f"        let val_{var} = sum_{var} / {p} as f64"
                        f" * {f}{' / 2.0' if shadows else ''};\n")
        loop_pre += f"        let cur_{var} = {cur_expr};\n" + val_line
        advances += (
            f"        sum_{var} += cur_{var} - {trail_expr};\n"
            f"        trail_{var} += 1;\n"
        )

    pre_if_out = (pre_if.rstrip() + '\n') if pre_if.strip() else ''
    then_pre_out = (then_pre.rstrip() + '\n') if then_pre.strip() else ''
    cond_t = cond.rstrip()
    new_while = (
        f"    let mut i = lookback;\n"
        f"    while i < n {{\n"
        f"{loop_pre}"
        f"{pre_if_out}"
        f"{then_pre_out}"
        f"        out[i] = if {cond_t}\n        {{ {out_expr} }} else {{ 0.0 }};\n"
        f"{advances}"
        f"        i += 1;\n"
        f"    }}\n"
    )

    new_func = prelude + warm + new_while + text[wclose + 1:]
    return new_func, None


FUNC_RE = re.compile(r'pub fn (cdl_\w+_with_output)\(')


def transform_file(path, apply=False, dry=False):
    src = open(path).read()
    out = []
    last = 0
    skipped = []
    transformed = []
    for m in FUNC_RE.finditer(src):
        fn_start = m.start()
        ob = src.find('{', fn_start)
        if ob == -1:
            continue
        cb = find_matching_brace(src, ob)
        if cb == -1:
            continue
        fn_text = src[fn_start:cb + 1]
        new_fn, reason = transform_function(fn_text)
        out.append(src[last:fn_start])
        if new_fn is None:
            skipped.append((m.group(1), reason))
            out.append(fn_text)
        else:
            transformed.append(m.group(1))
            out.append(new_fn)
        last = cb + 1
    out.append(src[last:])
    result = ''.join(out)
    if dry:
        print(f"=== {path} ===")
        print(f"transformed: {transformed}")
        print(f"skipped: {skipped}")
    if apply:
        open(path, 'w').write(result)
    return transformed, skipped


if __name__ == '__main__':
    mode = sys.argv[1] if len(sys.argv) > 1 else 'dry'
    files = sys.argv[2:] or [f"/Users/tony/github/adaq-talib/src/pattern/batch_{n}.rs" for n in range(1, 9)]
    for f in files:
        transform_file(f, apply=(mode == 'apply'), dry=(mode == 'dry'))
