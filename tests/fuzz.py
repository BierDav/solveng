#!/usr/bin/env python3
"""Fuzzer for solveng: generates random documents, compiles them with `typst`, checks the stored results.

  python3 tests/fuzz.py                 # 300 cases per category
  python3 tests/fuzz.py -n 1000 -c solve1,units --seed 7 -j 12 --show 5

Outcomes per case:
  ok        result checked and correct
  graceful  the library refused with a "solveng: …" message (fine if the input really is bad)
  miss      no solution stored although one exists
  wrong     a stored/displayed value is wrong            <- bug
  crash     a typst error that is not a solveng panic   <- bug
"""
import argparse, json, math, os, random, subprocess, sys, tempfile, collections
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
LIB = os.path.join(os.path.dirname(HERE), "src", "graph.typ")  # internals (_st, _fmt, …) live in src/
HEAD = '#import "{lib}": *\n#set page(height: auto)\n#show: solve-config.with(errors: "panic")\n'

def run(doc, typst, head=None):
    with tempfile.NamedTemporaryFile("w", suffix=".typ", dir=TMP, delete=False) as f:
        f.write((head or HEAD).format(lib=LIB) + doc)
        path = f.name
    try:
        p = subprocess.run([typst, "eval", "query(<r>).map(it => it.value)", "--in", path, "--root", "/"],
                           capture_output=True, text=True, timeout=120)
    except subprocess.TimeoutExpired:
        return ("crash", "timeout")
    finally:
        os.unlink(path)
    if p.returncode != 0:
        err = p.stderr.strip().split("\n")[0]
        return ("graceful" if "solveng:" in err else "crash", err)
    return ("out", json.loads(p.stdout))

def num(x):
    return repr(float(x)).replace("inf", "1e999")

def close(a, b, rel=1e-6, abs_=1e-12):
    return abs(a - b) <= rel * max(abs(a), abs(b)) + abs_

# ---------- units: name -> (factor, dims m kg s A K; the library adds mol cd) ----------
UNITS = {
    "m": (1, (1, 0, 0, 0, 0)), "g": (1e-3, (0, 1, 0, 0, 0)), "s": (1, (0, 0, 1, 0, 0)), "A": (1, (0, 0, 0, 1, 0)),
    "V": (1, (2, 1, -3, -1, 0)), "Ω": (1, (2, 1, -3, -2, 0)), "Ohm": (1, (2, 1, -3, -2, 0)), "W": (1, (2, 1, -3, 0, 0)),
    "J": (1, (2, 1, -2, 0, 0)), "C": (1, (0, 0, 1, 1, 0)), "F": (1, (-2, -1, 4, 2, 0)), "H": (1, (2, 1, -2, -2, 0)),
    "Hz": (1, (0, 0, -1, 0, 0)), "N": (1, (1, 1, -2, 0, 0)), "Pa": (1, (-1, 1, -2, 0, 0)), "h": (3600, (0, 0, 1, 0, 0)),
    "min": (60, (0, 0, 1, 0, 0)), "Ah": (3600, (0, 0, 1, 1, 0)), "Wh": (3600, (2, 1, -2, 0, 0)), "eV": (1.602176634e-19, (2, 1, -2, 0, 0)),
}
PREF = {"p": 1e-12, "n": 1e-9, "u": 1e-6, "µ": 1e-6, "m": 1e-3, "": 1, "k": 1e3, "M": 1e6, "G": 1e9}

def rand_unit(r, dims=None):
    names = [u for u, (f, d) in UNITS.items() if dims is None or d == dims]
    u = r.choice(names)
    p = r.choice(list(PREF)) if u not in ("h", "min", "eV") else ""
    if p and p + u in UNITS: p = ""  # avoid "mm" style clashes being misread
    return p + u, PREF[p] * UNITS[u][0], UNITS[u][1]

def rand_mag(r, lo=-6, hi=6):
    return r.choice([-1, 1]) * 10 ** r.uniform(lo, hi) if r.random() < 0.15 else 10 ** r.uniform(lo, hi)

# ---------- parse: quantity strings and math content ----------
def case_parse(r):
    unit, f, d = rand_unit(r)
    kind = r.randrange(7)
    if kind == 0: s, v = str(n := r.randint(-999, 99999)), n
    elif kind == 1: v = round(r.uniform(-1000, 1000), r.randint(0, 4)); s = repr(v).replace(".", r.choice([".", ","]))
    elif kind == 2: v = rand_mag(r, -12, 12); s = "%.6e" % v; v = float(s)
    elif kind == 3: a, b = r.randint(1, 999), r.randint(1, 99); s, v = f"{a}/{b}", a / b
    elif kind == 4: a, b, c = r.randint(1, 9), r.randint(1, 9), r.randint(1, 9); s, v = f"({a}+{b})*{c}", (a + b) * c
    elif kind == 5: a, b = r.randint(1, 9), r.randint(0, 4); s, v = f"{a}^{b}", a ** b
    else:
        a, b = r.randint(1, 99), r.randint(1, 99)
        mu = unit.replace("Ω", "Omega").replace("Ohm", "Omega")
        pre, base = (mu[0], mu[1:]) if len(mu) > 1 and mu[0] in PREF and mu[1:] in UNITS | {"Omega": 0} else ("", mu)
        base_m = "Omega" if base == "Omega" else f'"{base}"'
        if pre in ("m", "") or base_m[0] == '"': pre, base_m = "", f'"{pre}{base}"' if base != "Omega" else (f'"{pre}"Omega' if pre else "Omega")  # "m" Omega (spaced) is metre·ohm
        q = f'$frac({a}, {b}) "{pre}" {base_m}$' if r.random() < .5 else f'${a}/{b} "{pre}"{base_m}$'
        return f"#def({q.replace('$', '$x = ', 1)})\n#context [#metadata(_st.get().at(\"x\").v) <r>]", a / b * f, None
    sp = r.choice(["", " ", "  "])
    return f'#def("x = {s}{sp}{unit}")\n#context [#metadata(_st.get().at("x").v) <r>]', v * f, None

# ---------- expressions in x and known unitless a, b, c ----------
FUNS = ["sqrt", "exp", "ln", "sin", "cos", "abs"]
def gen(r, depth, vars_):
    if depth == 0 or r.random() < 0.25:
        k = r.random()
        if k < 0.45: return ("x",)
        if k < 0.75: return ("v", r.choice(vars_))
        return ("n", r.choice([r.randint(1, 9), round(r.uniform(0.1, 9.9), 2)]))
    op = r.choices(["+", "-", "*", "/", "^", "f", "neg"], [3, 2, 3, 2, 1, 1.2, 0.5])[0]
    if op == "^": return ("^", gen(r, depth - 1, vars_), ("n", r.choice([2, 3, 0.5, -1])))
    if op == "f": return ("f", r.choice(FUNS), gen(r, depth - 1, vars_))
    if op == "neg": return ("neg", gen(r, depth - 1, vars_))
    return (op, gen(r, depth - 1, vars_), gen(r, depth - 1, vars_))

def has_x(e): return e[0] == "x" or any(has_x(a) for a in e[1:] if isinstance(a, tuple))
def count_x(e): return (e[0] == "x") + sum(count_x(a) for a in e[1:] if isinstance(a, tuple))

def show(e, r):
    k = e[0]
    if k == "x": return "x"
    if k == "v": return e[1]
    if k == "n": return str(e[1])
    if k == "neg": return "-(" + show(e[1], r) + ")"
    if k == "f": return f"{e[1]}({show(e[2], r)})"
    if k == "^": return f"({show(e[1], r)})^{('(' + str(e[2][1]) + ')') if e[2][1] < 0 else e[2][1]}"
    a, b = show(e[1], r), show(e[2], r)
    if k == "*" and r.random() < 0.4: return f"({a}) ({b})"  # implicit multiplication
    return f"({a}){k}({b})"

def ev(e, env):
    k = e[0]
    if k == "x": return env["x"]
    if k == "v": return env[e[1]]
    if k == "n": return float(e[1])
    if k == "neg": return -ev(e[1], env)
    if k == "f":
        y = ev(e[2], env)
        return {"sqrt": math.sqrt, "exp": math.exp, "ln": math.log, "sin": math.sin, "cos": math.cos, "abs": abs}[e[1]](y)
    a, b = ev(e[1], env), ev(e[2], env)
    if k == "+": return a + b
    if k == "-": return a - b
    if k == "*": return a * b
    if k == "/": return a / b
    p = e[2][1]
    if a < 0 and p != int(p): raise ValueError
    return a ** p

def case_solve1(r, depth=None):
    vars_ = ["a", "b", "c"]
    for _ in range(200):
        e = gen(r, depth or r.randint(1, 4), vars_)
        if not has_x(e): e = ("+", e, ("x",)) if r.random() < .5 else ("*", e, ("x",))
        env = {v: round(rand_mag(r, -1, 2), 3) or 1.0 for v in vars_}
        env["x"] = round(rand_mag(r, -2, 3), 4) or 1.0
        try:
            C = ev(e, env)
            if not (1e-6 < abs(C) < 1e9): continue
        except (ValueError, ZeroDivisionError, OverflowError):
            continue
        break
    else:
        return None
    used = sorted({a for a in vars_ if f"('v', '{a}')" in repr(e)})
    giv = "".join(f'"{v} = {num(env[v])}", ' for v in used)
    doc = f'#def({giv}"C = {num(C)}")\n'
    doc += f'#solve("{show(e, r)} = C", unknown: "x")\n'
    doc += '#context [#metadata((x: _st.get().at("x", default: (v: none)).v, sols: _st.get().at("..").last().at("sols", default: none))) <r>]'
    def check(out):
        x = out["x"]
        if x is None: return "miss", f"true x = {env['x']}"
        try: y = ev(e, {**env, "x": x})
        except Exception as ex: return "wrong", f"x = {x} outside the domain ({ex})"
        if not close(y, C, 1e-6, 1e-9 * max(1, abs(C))): return "wrong", f"x = {x}: lhs = {y}, C = {C} (true x = {env['x']})"
        return "ok", ""
    return doc, None, check

def case_poly(r):
    deg = r.randint(2, 5)
    roots = sorted(round(r.uniform(-20, 20), r.randint(0, 2)) for _ in range(deg))
    lead = r.choice([1, 2, -3, 0.5])
    # expand lead * Π (x - root)
    co = [lead]
    for z in roots: co = [a - z * b for a, b in zip(co + [0], [0] + co)]
    terms = " + ".join(f"({num(c)}) x^{deg - i}" for i, c in enumerate(co) if c != 0)
    doc = f'#solve("{terms} = 0", unknown: "x", solutions: 10)\n#context [#metadata(_st.get().at("..").last().sols) <r>]'
    want = sorted(set(roots))
    def check(out):
        got = sorted(s[0] for s in out)
        miss = [z for z in want if not any(close(z, g, 1e-5, 1e-6) for g in got)]
        extra = [g for g in got if not any(close(z, g, 1e-5, 1e-6) for z in want)]
        if extra: return "wrong", f"roots {want}, got {got}"
        if miss: return "miss", f"roots {want}, got {got}"
        return "ok", ""
    return doc, None, check

def case_system(r):
    a, b, c, d = (round(r.uniform(-9, 9), 1) or 1 for _ in range(4))
    x, y = round(rand_mag(r, -1, 2), 3), round(rand_mag(r, -1, 2), 3)
    kind = r.randrange(3)
    if kind == 0: e1, e2, f1, f2 = f"({a}) x + ({b}) y", f"({c}) x - ({d}) y", a * x + b * y, c * x - d * y
    elif kind == 1: e1, e2, f1, f2 = "x y", f"x + ({b}) y", x * y, x + b * y
    else: e1, e2, f1, f2 = "x^2 + y^2", f"x - ({c}) y", x * x + y * y, x - c * y
    doc = (f'#def("p = {num(f1)}", "q = {num(f2)}")\n#solve("{e1} = p", "{e2} = q", unknown: ("x", "y"))\n'
           '#context [#metadata((x: _st.get().at("x", default: (v: none)).v, y: _st.get().at("y", default: (v: none)).v)) <r>]')
    def check(out):
        X, Y = out["x"], out["y"]
        if X is None: return "miss", f"true ({x}, {y})"
        g = {"x": X, "y": Y}
        r1 = eval(e1.replace("^", "**").replace(") y", ")*y").replace(") x", ")*x").replace("x y", "x*y"), {}, g)
        r2 = eval(e2.replace("^", "**").replace(") y", ")*y").replace(") x", ")*x"), {}, g)
        if not (close(r1, f1, 1e-6, 1e-9) and close(r2, f2, 1e-6, 1e-9)): return "wrong", f"({X}, {Y}) → {r1}, {r2} want {f1}, {f2}"
        return "ok", ""
    return doc, None, check

# ---------- physics formulas with prefixed units ----------
FORMULAS = [  # (equation, {var: dims}, python check lhs-rhs)
    ("U = R I", {"U": "V", "R": "Ω", "I": "A"}, lambda v: v["U"] - v["R"] * v["I"]),
    ("P = U^2/R", {"P": "W", "U": "V", "R": "Ω"}, lambda v: v["P"] - v["U"] ** 2 / v["R"]),
    ("Q = C U", {"Q": "C", "C": "F", "U": "V"}, lambda v: v["Q"] - v["C"] * v["U"]),
    ("tau = R C", {"tau": "s", "R": "Ω", "C": "F"}, lambda v: v["tau"] - v["R"] * v["C"]),
    ("f = 1/(2 pi sqrt(L C))", {"f": "Hz", "L": "H", "C": "F"}, lambda v: v["f"] - 1 / (2 * math.pi * math.sqrt(v["L"] * v["C"]))),
    ("E = P t", {"E": "J", "P": "W", "t": "s"}, lambda v: v["E"] - v["P"] * v["t"]),
    ("Q = I t", {"Q": "C", "I": "A", "t": "s"}, lambda v: v["Q"] - v["I"] * v["t"]),
    ("U = U0 exp(-t/tau)", {"U": "V", "U0": "V", "t": "s", "tau": "s"}, lambda v: v["U"] - v["U0"] * math.exp(-v["t"] / v["tau"])),
]
def case_units(r):
    eq, dims, F = r.choice(FORMULAS)
    names = list(dims)
    for _ in range(100):
        vals = {n: rand_mag(r, -4, 4) for n in names}
        if "U0" in vals: vals["U0"] = abs(vals["U0"]); vals["t"] = abs(vals["t"]); vals["tau"] = abs(vals["tau"])
        for n in ("L", "C", "R", "tau"):
            if n in vals: vals[n] = abs(vals[n])
        x = r.choice(names)
        # make the equation hold by solving for x numerically in python (bisection on a log scale)
        def g(t):
            try: return F({**vals, x: t})
            except (OverflowError, ValueError, ZeroDivisionError): return math.nan
        lo, hi = 1e-15, 1e15
        sgn = 1
        if not g(lo) * g(hi) <= 0:
            lo, hi, sgn = -1e15, -1e-15, -1
            if not g(lo) * g(hi) <= 0: continue
        for _ in range(300):
            m = math.copysign(math.sqrt(abs(lo * hi)), lo) if lo * hi > 0 else (lo + hi) / 2
            if g(lo) * g(m) <= 0: hi = m
            else: lo = m
        vals[x] = (lo + hi) / 2
        if 1e-12 < abs(vals[x]) < 1e12 and abs(g(vals[x])) < 1e-9 * max(abs(F({**vals, x: 0.0}) if False else 1), 1e-300) or (1e-12 < abs(vals[x]) < 1e12 and close(g(vals[x]) + 1, 1, 1e-9, 0)): break
    else:
        return None
    giv = []
    for n in names:
        if n == x: continue
        u, f, _ = rand_unit(r, UNITS[dims[n]][1])
        giv.append(f'"{n} = {vals[n] / f:.12g} {u}"')
    tu, tf, _ = rand_unit(r, UNITS[dims[x]][1])
    target = f', unit: "{tu}"' if r.random() < 0.5 else ""
    doc = (f'#def({", ".join(giv)})\n#solve("{eq}"{target}, unknown: "{x}")\n'
           f'#context [#metadata((v: _st.get().at("{x}", default: (v: none)).v, d: _st.get().at("{x}", default: (d: none)).d)) <r>]')
    def check(out):
        if out["v"] is None: return "miss", f"{x} = {vals[x]}"
        if list(out["d"]) != list(UNITS[dims[x]][1]) + [0, 0]: return "wrong", f"dims {out['d']} for {x}"
        if not close(out["v"], vals[x], 1e-6, 1e-300): return "wrong", f"{x} = {out['v']}, want {vals[x]}"
        return "ok", ""
    return doc, None, check

# ---------- number formatting and fractions ----------
def case_format(r):
    v = rand_mag(r, -20, 20)
    dg = r.randint(1, 10)
    sci = r.choice(["(-3, 6)", "none", "(-1, 2)", "(0, 0)"])
    eng = r.choice(["true", "false"])
    doc = f'#context [#metadata(_fmt({num(v)}, _DEF + (digits: {dg}, sci: {sci}, eng: {eng}))) <r>]'
    def check(out):
        m, e = out
        shown = float(m) * 10 ** (e or 0)
        if not close(shown, v, 0.5 * 10 ** (1 - dg) * 1.0000001, 0): return "wrong", f"{v} digits {dg} sci {sci} eng {eng} → {m}e{e}"
        mant = m.lstrip("-").replace(".", "").lstrip("0")
        if len(mant.rstrip("0")) > dg: return "wrong", f"{v} digits {dg} → {m}e{e} (too many digits)"
        if e is not None and eng == "true" and e % 3: return "wrong", f"eng exponent {e}"
        if e is not None and eng == "false" and not (1 <= abs(float(m)) < 10): return "wrong", f"mantissa {m} not normalised"
        return "ok", ""
    return doc, None, check

def case_ratio(r):
    q = r.randint(1, 1500); p = r.randint(-5000, 5000)
    mx = r.choice([10, 100, 1000])
    doc = f'#context [#metadata(_ratio({num(p / q)}, {mx})) <r>]'
    g = math.gcd(p, q); P, Q = p // g, q // g
    def check(out):
        if Q > mx:
            if out is not None and out[1] > mx: return "wrong", f"{p}/{q} → {out} over max {mx}"
            return "ok", ""
        if out != [P, Q]: return "wrong" if out is not None else "miss", f"{p}/{q} → {out}, want {P}/{Q}"
        return "ok", ""
    return doc, None, check

# ---------- display: random options on random solves must never crash ----------
OPTS = {
    "digits": ["1", "3", "4", "8"], "zeros": ["true", "false"], "sep": ['"lang"', '","', '"."'], "group": ["none", "sym.space.thin", '" "'],
    "sci": ["(-3, 6)", "none", "(0, 3)"], "eng": ["true", "false"], "times": ['"dot"', '"times"'],
    "form": ['"auto"', '"decimal"', '"fraction"', '"input"'], "frac-max": ["10", "1000"], "slash": ["true", "false"],
    "prefixes": ['"all"', "none", '("m", "k")', '("u",)'], "si": ["true", "false"], "steps": ['"full"', '"formula"', '"result"', '"value"', "false"], "inline-steps": ['"value"', '"full"'],
    "layout": ['"row"', '"column"'], "approx": ["true", "false"], "block": ["true", "false"], "solutions": ["1", "6"],
"definitions": ['"grid"', '"column"', '"inline"', '"list"'],
}
def case_display(r):
    opts = ", ".join(f"{k}: {r.choice(v)}" for k, v in OPTS.items() if r.random() < 0.5)
    sub = case_solve1(r, depth=2) if r.random() < .5 else (case_poly(r) if r.random() < .5 else case_system(r))
    if sub is None: return None
    body = sub[0].rsplit("\n", 1)[0]
    g = r.choice(['"3450/2 mAh"', '$frac(2, 4)"A"$', '"0.000123 kOhm"', '"-12345678 V"', '"7.20 uF"'])
    doc = f'#solve-config({opts})\n#def({g.replace(g[0], g[0] + 'z = ', 1)})\n{body}\n#definitions()\n#val("z")\n#context [#metadata(1) <r>]'
    return doc, None, lambda out: ("ok", "")


# ---------- fixed hostile inputs: (document, expected outcome, check of the stored value or None) ----------
V = lambda n: f'#context [#metadata(_st.get().at("{n}", default: (v: none)).v) <r>]'
EDGE = [
    ('#def("a = ")', "graceful", None), ('#def("a =  V")', "graceful", None), ('#def("a = abc")', "graceful", None),
    ('#def("a = 1..2 V")', "graceful", None), ('#def("a = 1e999 V")', "graceful", None), ('#def("a = --5 V")', "graceful", None),
    ('#def("a = −5 V")' + V("a"), "ok", -5), ('#def("a = 5 %") #solve("b = a * 200")' + V("b"), "ok", 10),
    ('#def("Q_0 = 1 C", "R_min = 2 Ohm") #solve("Q_0 R_min = y")' + V("y"), "ok", 2),
    ('#def("e = 2") #solve("y = e^2")' + V("y"), "ok", 4), ('#def("pi = 3") #solve("y = pi")' + V("y"), "ok", 3),
    ('#solve("y == 2")', "graceful", None), ('#solve("")', "graceful", None), ('#solve("= 5")', "graceful", None),
    ('#solve("x^1000 = 2")' + V("x"), "ok", 2 ** (1 / 1000)), ('#solve("exp(x) = -1")', "ok", None), ('#solve("sin(x) = 2")', "ok", None),
    ('#solve("' + "(" * 18 + "x+1" + ")" * 18 + ' = 2")' + V("x"), "ok", 1),
    ('#solve("' + " + ".join(f"{i} x^{i % 5}" for i in range(1, 80)) + ' = 7", root: 1)', "ok", None),
    ('#solve("' + " ".join(["x"] * 40) + ' = 2")' + V("x"), "ok", 2 ** (1 / 40)),
    ('#solve("x + y + z = 6", "x - y = 0", "y + 2 z = 5")' + V("z"), "ok", 4 / 3),
    ('#solve("x + y = 2", "x + y = 3", unknown: ("x", "y"))', "ok", None),
    ('#solve-config(digit: 3)', "graceful", None), ('#solve-config(form: "frac")', "graceful", None), ('#solve-config(digits: 0)', "graceful", None),
    ('#val("nope")', "graceful", None), ('#def("a = 1 V") #val("a", unit: "A")', "graceful", None),
    ('#def("a = 1 V", "b = 1 A") #solve("c = a + b")', "graceful", None),
    ('#def("a = 3 V") #solve-block[#def("a = 5 V")] #solve("b = a")' + V("b"), "ok", 3),
    ('#def("a = 3 V") #solve-block(inherit: true)[#solve("b = 2 a")]' + V("b"), "ok", None),
    ('#solve("x^2 = 4", root: 3)', "graceful", None), ('#solve("x^2 = 4", root: x => x > 100)', "graceful", None),
    ('#solve("x^2 = 4", guess: -1)' + V("x"), "ok", -2), ('#solve("abs(x - 3) = 2", root: 1)' + V("x"), "ok", 1),
    ('#solve($y = 2$)' + V("y"), "ok", 2), ('#def("R = 2 kOhm", "I = 3 mA") #solve($U = R dot I$)' + V("U"), "ok", 6),
    ('#solve($x + y = 3$, $x - y = 1$)' + V("x"), "ok", 2), ('#solve($w = root(3, 8) times 2$)' + V("w"), "ok", 4),
    ('#def("a = 2 m^2", "b = 3 m²") #solve("c = a + b")' + V("c"), "ok", 5), ('#def("v = 3 m/s/s") #val("v")', "ok", None),
    ('#def("m = 2 kg") #solve("F = m * [9.81 m/s^2]")' + V("F"), "ok", 19.62),
    ('#def("a = 3 µF", "b = 3 μF", "c = 3 uF") #solve("d = a + b + c")' + V("d"), "ok", 9e-6),
    ('#def("a = 2 V") #def("b = a^2", "c = 2 a", "d = (1+2) V", $f = frac(1, 3) "V"$)' + V("d"), "ok", 3),
    ('#def(config: (foo: 1), "a = 1 V")', "graceful", None), ('#solve("q = 1", digits: 99)', "graceful", None),
    ('#def("a = 0 V") #solve("b = 1/a")', "graceful", None), ('#solve("x = 1/0")', "graceful", None), ('#solve("x = 0^-1")', "graceful", None), ('#solve("x = (-8)^(1/3)")', "graceful", None),
    ('#def("a = 3450/2 mAh", $b = frac(2, 4) "A"$) #solve-config(form: "fraction")[#definitions(included: "given")] #solve-config(form: "input")[#val("a")]', "ok", None),
    ('#solve-config(layout: "column", steps: "formula")[#solve("x^3 - x = 0")] #solve("p q = 6", "p + q = 5", steps: "result")', "ok", None),
    ('#show: solve-config.with(digits: 2, sep: ",")\n#solve("y = 1/3")', "ok", None),
    ('#def("f(x) = x^2 + 1") #def("y = f(3)")' + V("y"), "ok", 10),
    ('#def("g(a, b) = a b + b", "R = 2 kOhm") #def("z = g(2, R) / R")' + V("z"), "ok", 3),
    ('#def("f(x) = 2 x") #def("f(y) = 3 = 9")', "graceful", None), ('#def("f(x) = x") #def("y = f(1, 2)")', "graceful", None),
    ('#def("f(2 x) = x")', "graceful", None), ('#def("sin(x) = 2")' + V("x"), "ok", None), ('#def("f(x) = x") #def("y = f(")', "graceful", None),
    ('#def("f(x) = x^2") #solve("f(x) = 9", root: 2)' + V("x"), "ok", 3),
    ('#def("h(s) = s/2") #def("t = 40 h") #def("w = h(t)")' + V("w"), "ok", 72000),
    ('#def("R = 1 kOhm", "I = 2 mA", "U = R I")' + V("U"), "ok", 2), ('#def("U = R I")', "graceful", None),
    ('#def("x + y = 3", "x - y = 1")' + V("y"), "ok", 1), ('#def()', "graceful", None), ('#def("a = 1", output: "yes")', "graceful", None),
    ('#def("a = 1 V", output: true, steps: "value") #def("b = 2 V", output: true, inline: true) #solve("c = 2 a", steps: "value", inline: true) #solve("d = 2 a", layout: "column", steps: "formula") #def("c = 3 V", listed: false)', "ok", None),
    ('#solve-block[`a = 2 V` and\n```\nb = 2 a -> mV // c\nf(x) = x + 1\nc = f(1)\n\np + q = 3; p - q = 1\nd -> V\n```\n]', "graceful", None),
    ('#solve-block[`a = 2 V` and\n```\nb = 2 a -> mV // c\nf(x) = x + 1\nc = f(1)\n\np + q = 3; p - q = 1\nb -> V\n```\n```py\nprint(1)\n```\n#definitions()]', "ok", None),
    ('#def("a = 2 V") #solve-block[`a = 5 V`] #def("b = a")' + V("b"), "ok", 2),
    ('#def("a = 2 V") #solve-block(inherit: true)[#solve("b = a / 3", digits: 2)]', "ok", None), ('#solve-block(digits: 2)[]', "graceful", None),
    ('#def("a = 1 V", "b = 2 V", "f(x) = x", "c = a + b") #definitions(included: "given", excluded: "a") #definitions(included: regex("^[ab]$"), style: "column") #definitions(excluded: ("given", "function"), style: "inline") #definitions(included: n => n == "c", si: true)', "ok", None),
    ('#definitions(included: 5)', "graceful", None), ('#def("a = 1") #definitions(included: 5)', "graceful", None),
    # the typed API: the old forms are refused with a hint
    ('#def(I: "30 mA")', "graceful", None), ('#def("I = 30 mA") #solve("I", "mA")', "graceful", None), ('#solve("x = 1", unitt: "A")', "graceful", None),
    ('#solve("x = 1", unit: 5)', "graceful", None), ('#solve("x = 1", root: "1")', "graceful", None), ('#def(5)', "graceful", None),
    ('#solve("x = 1", config: 3)', "graceful", None), ('#solve("x = 1", sci: (1, 2, 3))', "graceful", None), ('#solve-config(prefixes: "k")', "graceful", None),
    ('#def("I = 30mAh")' + V("I"), "ok", 108), ('#def("x = 1 V", "y = 1 A") #solve("z = x y", unit: "mW")' + V("z"), "ok", 1),
    ('#solve("p + q = 3", "p - q = 1", unit: (p: "m"))', "graceful", None), ('#solve("p + q = [3 m]", "p - q = [1 m]", unit: (p: "mm", q: "cm"))' + V("q"), "ok", 1),
    # solve finds the unknown on either side, units must fit
    ('#def("f(y) = y * [2 V]") #solve("f(1) = x")' + V("x"), "ok", 2), ('#def("f(y) = y * [2 V]") #solve("x = f(1)", unit: "A")', "graceful", None),
    ('#def("x = 1 V", "f(y) = 2 y") #solve("f([3 V]) = x")' + V("x"), "ok", 6),
    # math equations in a solve-block are definitions
    ('#show: solve-block\n$I = "30 mA"$ (#solve("I", steps: "value", inline: true)) $t = 2 I$ $x^2$ $ U &= 1 \\ &= 2 $' + V("t"), "ok", 0.06),
    ('#solve-block[$a = 2 "V"$ #solve("b = 2 a")] #solve-block(equations: false)[$c = 2$ #solve("c")]', "graceful", None),
    ('#show: solve-block\n$q = 7 "furlongs"$', "graceful", None),
    ('#show: solve-block.with(reset: heading)\n= A\n$a = 2$\n= B\n#solve("b = a")', "graceful", None),
    ('#def("a = 3")\n#show: solve-block.with(reset: heading, inherit: true)\n= A\n$a = 2$\n= B\n#reset\n#solve("b = a")\n#solve-block[#reset]' + V("b"), "ok", 3),
    ('#solve-block(reset: 5)[]', "graceful", None),
    ('#show: solve-block\n$ t = frac(3450 / 2 "mAh", 30 "mA") = 57","5 "h" $' + V("t"), "ok", 207000),
    ('#show: solve-block\n$R = 7.2 "k"Omega = 7200 Omega$ $x = 0,5 dot R$' + V("x"), "ok", 3600),
    ('#show: solve-block\n$a = 2 "m"/"s"^2$ $h = 2$ $y = 3 h$ $Q = I t => t = Q/I$' + V("a") + V("y"), "ok", None),
    ('#graph("y = 1/x", xrange: (-1, 1), ymark: 2)', "ok", None), ('#graph("y = x^2", "y = 2x + 3")', "ok", None), ('#graph("sin(t)")', "ok", None), ('#graph("exp(a) - 3", xrange: (0, 2), yrange: (-3, 5), mark: ("roots", 1.5), xmark: "crossings")', "ok", None), ('#graph("sqrt(u) - 1", colors: red)', "ok", None), ('#graph("x", colors: [red])', "ok", None), ('#graph("x", width: 100%, height: 50%)', "ok", None), ('#graph("x", width: 6cm, height: 120pt)', "ok", None), ('#graph("x", width: -1)', "graceful", None), ('#graph("y = 2", x: "x", xrange: (0, 1), mark: ())', "ok", None),
]
def case_edge(r, _i=[0]):
    i = _i[0]; _i[0] += 1
    if i >= len(EDGE): return None
    doc, want, val = EDGE[i]
    if "<r>" not in doc: doc += "\n#context [#metadata(1) <r>]"
    def check(out):
        if want == "graceful": return "wrong", f"accepted bad input: {doc[:80]}"
        if val is not None and (out is None or not close(out, val, 1e-6, 1e-12)): return "wrong", f"got {out}, want {val}"
        return "ok", ""
    return doc, None, check, want

# the same hostile inputs with the default errors: "document": the document must always compile
DOC_HEAD = '#import "{lib}": *\n#set page(height: auto)\n'
def case_edge_doc(r, _i=[0]):
    i = _i[0]; _i[0] += 1
    if i >= len(EDGE): return None
    doc = EDGE[i][0]
    if "<r>" not in doc: doc += "\n#context [#metadata(1) <r>]"
    return doc, None, lambda out: ("ok", ""), "compiles"

CATS = {"parse": case_parse, "solve1": case_solve1, "poly": case_poly, "system": case_system,
        "units": case_units, "edge": case_edge, "edge-doc": case_edge_doc, "format": case_format, "ratio": case_ratio, "display": case_display}

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("-n", type=int, default=300)
    ap.add_argument("-c", "--cats", default=",".join(CATS))
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("-j", type=int, default=os.cpu_count())
    ap.add_argument("--show", type=int, default=3, help="examples printed per outcome")
    ap.add_argument("--typst", default="typst")
    ap.add_argument("--save", help="write failing documents into this folder")
    a = ap.parse_args()
    global TMP
    TMP = tempfile.mkdtemp(prefix="solveng-fuzz-")
    jobs = []
    for cat in a.cats.split(","):
        r = random.Random(f"{a.seed}-{cat}")
        for i in range(a.n):
            c = CATS[cat](r)
            if c: jobs.append((cat, i, c))
    def work(j):
        cat, i, c = j
        doc, want, check = c[:3]
        kind, out = run(doc, a.typst, DOC_HEAD if len(c) > 3 and c[3] == "compiles" else None)
        if len(c) > 3 and kind == "graceful" and c[3] == "graceful": return cat, "ok", "", doc
        if len(c) > 3 and kind == "graceful": return cat, "wrong", f"refused: {out}", doc
        if kind != "out": return cat, kind, out, doc
        if want is not None:
            v = out[0]
            return (cat, "ok", "", doc) if close(v, want, 1e-9, 1e-300) else (cat, "wrong", f"got {v}, want {want}", doc)
        k, msg = check(out[0])
        return cat, k, msg, doc
    res = collections.defaultdict(lambda: collections.defaultdict(list))
    with ThreadPoolExecutor(a.j) as ex:
        for n, (cat, kind, msg, doc) in enumerate(ex.map(work, jobs)):
            res[cat][kind].append((msg, doc))
            if sys.stderr.isatty(): print(f"\r{n + 1}/{len(jobs)}", end="", file=sys.stderr)
    print(file=sys.stderr)
    bad = 0
    for cat in res:
        tot = sum(len(v) for v in res[cat].values())
        print(f"\n== {cat}: " + ", ".join(f"{k} {len(v)}/{tot}" for k, v in sorted(res[cat].items())))
        for kind in ("crash", "wrong", "miss", "graceful"):
            seen = collections.Counter(m.replace("error: panicked with: ", "")[:110] for m, _ in res[cat].get(kind, []))
            for m, c in seen.most_common(a.show):
                ex = next(d for mm, d in res[cat][kind] if mm.replace("error: panicked with: ", "")[:110] == m)
                print(f"  [{kind}] ×{c} {m}")
                if kind in ("crash", "wrong", "miss"): print("      " + ex.replace("\n", "\n      ")[:600])
            if kind in ("crash", "wrong"): bad += len(res[cat].get(kind, []))
            if a.save:
                os.makedirs(a.save, exist_ok=True)
                for k, (m, d) in enumerate(res[cat].get(kind, [])):
                    open(os.path.join(a.save, f"{cat}-{kind}-{k}.typ"), "w").write(HEAD.format(lib="@local/solveng:0.1.0").replace('"@local', '"@local') + d)
    sys.exit(1 if bad else 0)

if __name__ == "__main__":
    main()
