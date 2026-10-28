"""Writes core/tests/oracle/*.txt: cases with results from SymPy and mpmath, checked by core/tests/oracle.rs.

The cases are families of problems (random within each family, from a fixed seed), so a reported failure
stands for a class of inputs. Expected results are exact where SymPy's are, else numbers to 20 digits.

    uv run --no-project --with sympy --with mpmath python tests/oracle/generate.py

Fixture lines are tab-separated; the first field is the kind:

    value     id  expression                                  expected            tolerance
    identity  id  expression  names  points                   expected per point  degrees
    solve     id  equations   unknowns  values of parameters  solutions           how

expected: a number (p/q exact, else a float), or rows×cols:n,n,… for a vector or matrix. points: name=value
pairs joined by ',', points joined by '|'. degrees: the degree of the cancelled numerator and denominator in
each name (x:2/1,y:0/0), 'const' for a result without names, or '-'. solutions: values of the unknowns joined by ',', solutions by '|', 'none' for
none. tolerance: the relative error allowed, '-' for the default 1e-9. how: 'exact' (a formula of the parameters is expected) or 'numeric' (values only).
"""
import os
import random

import mpmath
import sympy as sp

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "core", "tests", "oracle")
mpmath.mp.dps = 40

x, y, z, p, q, u, w = sp.symbols("x y z p q u w")


# ---------- solveng's code syntax ----------

FUNCS = {sp.exp: "exp", sp.log: "ln", sp.sin: "sin", sp.cos: "cos", sp.tan: "tan", sp.atan: "atan", sp.sinh: "sinh",
         sp.cosh: "cosh", sp.tanh: "tanh", sp.Abs: "abs", sp.asin: "asin", sp.acos: "acos", sp.factorial: "factorial"}


def code(e):
    """An expression in solveng's code syntax, parenthesized so that nothing depends on precedence."""
    e = sp.sympify(e)
    if isinstance(e, sp.MatrixBase):
        if e.cols == 1:
            return "vec(" + ", ".join(code(v) for v in e) + ")"
        return "mat(" + "; ".join(", ".join(code(v) for v in e.row(i)) for i in range(e.rows)) + ")"
    if e.is_Integer:
        return str(e) if e >= 0 else f"({e})"
    if e.is_Rational:
        return f"({e.p}/{e.q})"
    if e.is_Float:
        return repr(float(e)) if e >= 0 else f"({float(e)!r})"
    if e.is_Symbol:
        return e.name
    if e == sp.pi:
        return "pi"
    if e == sp.E:
        return "e"
    if e.is_Add:
        return "(" + " + ".join(code(t) for t in e.args) + ")"
    if e.is_Mul:
        return "(" + "*".join(code(f) for f in e.args) + ")"
    if e.is_Pow:
        return f"({code(e.base)})^({code(e.exp)})"
    if e.func in FUNCS:
        return f"{FUNCS[e.func]}({code(e.args[0])})"
    raise ValueError(f"no code for {e!r}")


def num(v):
    """A number for a fixture: exact p/q, else a float to 20 digits."""
    v = sp.sympify(v)
    if v.is_Rational:
        return str(v.p) if v.q == 1 else f"{v.p}/{v.q}"
    f = sp.N(v, 25)
    if not f.is_real:
        raise ValueError(f"not real: {v}")
    return mpmath.nstr(mpmath.mpf(str(f)), 20, min_fixed=-30, max_fixed=30)


def expected(v):
    v = sp.sympify(v)
    if isinstance(v, sp.MatrixBase):
        return f"{v.rows}x{v.cols}:" + ",".join(num(e) for e in v)
    return num(v)


def point_text(pt):
    return ",".join(f"{k}={num(v)}" for k, v in pt.items())


# ---------- families ----------

def decimal(v):
    """A float as the decimal it was written as, exactly."""
    return sp.Rational(repr(float(v)))


def tolerance(A):
    """The relative error to allow for a float computation with A: rounding times its condition number."""
    m = mpmath.matrix([[mpmath.mpf(sp.Rational(v).p) / sp.Rational(v).q for v in A.row(i)] for i in range(A.rows)])
    cond = mpmath.norm(m, 1) * mpmath.norm(mpmath.inverse(m), 1)
    return mpmath.nstr(max(mpmath.mpf("1e-9"), 1e-13 * cond * A.rows), 3)


def rnd_rat(r, lo=-9, hi=9, den=(1, 1, 1, 2, 3, 4, 5, 7)):
    while True:
        v = sp.Rational(r.randint(lo, hi), r.choice(den))
        if v != 0:
            return v


def values(r, cases):
    """Matrices of exact numbers and of floats: products, powers, determinants, inverses, systems, ranks."""
    for i in range(40):
        n = r.randint(2, 5)
        A = sp.Matrix(n, n, lambda *_: r.randint(-6, 6))
        if A.det() == 0:
            A[0, 0] += 1
        if A.det() == 0:
            continue
        b = sp.Matrix(n, 1, lambda *_: rnd_rat(r))
        B = sp.Matrix(n, r.randint(1, 3), lambda *_: rnd_rat(r))
        cases.append(("value", f"det-int-{i}", f"det({code(A)})", expected(A.det()), "-"))
        cases.append(("value", f"inverse-int-{i}", f"{code(A)}^(-1)", expected(A.inv()), "-"))
        cases.append(("value", f"solve-int-{i}", f"{code(A)}^(-1) {code(b)}", expected(A.LUsolve(b)), "-"))
        cases.append(("value", f"product-{i}", f"{code(A)} {code(B)}", expected(A * B), "-"))
        k = r.choice([-2, 2, 3, 5])
        cases.append(("value", f"power-{i}", f"{code(A)}^({k})", expected(A ** k), "-"))
        cases.append(("value", f"transpose-product-{i}", f"{code(b)}^T {code(A)} {code(b)}", expected((b.T * A * b)[0]), "-"))
    for i in range(25):
        n = r.randint(2, 6)
        rank = r.randint(1, n)
        L = sp.Matrix(n, rank, lambda *_: r.randint(-4, 4))
        R = sp.Matrix(rank, n, lambda *_: r.randint(-4, 4))
        M = L * R
        cases.append(("value", f"rank-{i}", f"rank({code(M)})", expected(M.rank()), "-"))
        cases.append(("value", f"det-singular-{i}", f"det({code(M)})", expected(M.det()), "-"))
    # floats: random well-conditioned matrices, and Hilbert matrices (ill-conditioned) with a looser tolerance
    for i in range(20):
        n = r.randint(2, 6)
        A = sp.Matrix(n, n, lambda *_: sp.Float(round(r.uniform(-10, 10), 6)))
        b = sp.Matrix(n, 1, lambda *_: sp.Float(round(r.uniform(-10, 10), 6)))
        exact = A.applyfunc(decimal)
        if exact.det() == 0:
            continue
        tol = tolerance(exact)
        cases.append(("value", f"det-float-{i}", f"det({code(A)})", expected(exact.det()), tol))
        cases.append(("value", f"solve-float-{i}", f"{code(A)}^(-1) {code(b)}", expected(exact.LUsolve(b.applyfunc(decimal))), tol))
    for n in range(3, 8):
        H = sp.Matrix(n, n, lambda i, j: sp.Rational(1, i + j + 1))
        cases.append(("value", f"hilbert-inverse-{n}", f"{code(H)}^(-1)", expected(H.inv()), "-"))
        Hf = H.applyfunc(lambda v: sp.Float(v, 17))
        cases.append(("value", f"hilbert-float-det-{n}", f"det({code(Hf)})", expected(H.det()), tolerance(H)))
    # vectors
    for i in range(15):
        a = sp.Matrix(3, 1, lambda *_: rnd_rat(r))
        b = sp.Matrix(3, 1, lambda *_: rnd_rat(r))
        cases.append(("value", f"cross-{i}", f"{code(a)} × {code(b)}", expected(a.cross(b)), "-"))
        cases.append(("value", f"dot-{i}", f"{code(a)} {code(b)}", expected(a.dot(b)), "-"))
        cases.append(("value", f"norm-{i}", f"|{code(a)}|", expected(a.norm()), "-"))
    # block matrices and the Frobenius norm
    for i in range(10):
        n, m = r.randint(1, 3), r.randint(1, 3)
        A = sp.Matrix(n, m, lambda *_: rnd_rat(r))
        B = sp.Matrix(n, r.randint(1, 2), lambda *_: rnd_rat(r))
        C = sp.Matrix(r.randint(1, 2), m + B.cols, lambda *_: rnd_rat(r))
        cases.append(("value", f"blocks-{i}", f"mat({code(A)}, {code(B)}; {code(C)})", expected(A.row_join(B).col_join(C)), "-"))
        cases.append(("value", f"frobenius-{i}", f"norm({code(C)})", expected(C.norm()), "-"))


def calculus(r, cases):
    """Definite integrals and series from SymPy, with mpmath as a fallback: smooth, infinite ranges,
    endpoint singularities, slowly converging and alternating series."""
    t, k = sp.symbols("t k")
    integrals = [
        ("poly-exp", lambda: (sp.Poly([r.randint(-3, 3) for _ in range(4)], t).as_expr() * sp.exp(rnd_rat(r, -2, 2) * t), rnd_rat(r, -2, 0), rnd_rat(r, 1, 3))),
        ("rational", lambda: (1 / (t ** 2 + rnd_rat(r, 1, 5)), rnd_rat(r, -3, 0), rnd_rat(r, 1, 4))),
        ("sqrt", lambda: (sp.sqrt(1 - t ** 2), 0, rnd_rat(r, 1, 9) / 10)),
        ("gaussian-infinite", lambda: (sp.exp(-rnd_rat(r, 1, 4) * t ** 2), -sp.oo, sp.oo)),
        ("gamma", lambda: (t ** r.randint(1, 5) * sp.exp(-t), 0, sp.oo)),
        ("log-endpoint", lambda: (sp.log(t), 0, rnd_rat(r, 1, 3))),
        ("inverse-sqrt-endpoint", lambda: (1 / sp.sqrt(t), 0, rnd_rat(r, 1, 4))),
        ("oscillating", lambda: (sp.sin(r.randint(5, 40) * t) * sp.exp(-t), 0, sp.oo)),
        ("lorentzian-infinite", lambda: (1 / (1 + t ** 4), 0, sp.oo)),
    ]
    for i in range(3):
        for name, make in integrals:
            f, lo, hi = make()
            # Prefer an exact independent result. Unsplit quadrature on an
            # infinite interval can miss oscillatory tails even at high precision.
            exact = sp.integrate(f, (t, lo, hi))
            if not exact.has(sp.Integral):
                val = exact
            else:
                val = sp.Float(str(mpmath.quad(sp.lambdify(t, f, "mpmath"), [mpmath.mpf(sp.N(lo, 30)) if lo != -sp.oo else -mpmath.inf, mpmath.mpf(sp.N(hi, 30)) if hi != sp.oo else mpmath.inf])), 30)
            bound = lambda b: "infinity" if b == sp.oo else "(-infinity)" if b == -sp.oo else code(b)
            cases.append(("value", f"integral-{name}-{i}", f"integrate({code(f)}, t, {bound(lo)}, {bound(hi)})", num(val), "1e-7"))
    series = [
        ("zeta-2", 1 / k ** 2, 1, sp.oo), ("zeta-3", 1 / k ** 3, 1, sp.oo), ("geometric", sp.Rational(2, 3) ** k, 0, sp.oo),
        ("exponential", sp.Rational(3, 2) ** k / sp.factorial(k), 0, sp.oo), ("leibniz", (-1) ** k / (2 * k + 1), 0, sp.oo),
        ("alternating-harmonic", (-1) ** (k + 1) / k, 1, sp.oo), ("finite-power", k ** 3, 1, 25), ("finite-rational", 1 / (k * (k + 1)), 1, 40),
    ]
    for name, f, lo, hi in series:
        val = sp.summation(f, (k, lo, hi))
        bound = "infinity" if hi == sp.oo else code(hi)
        cases.append(("value", f"sum-{name}", f"sum({code(f)}, k, {code(lo)}, {bound})", num(val), "1e-7" if hi == sp.oo else "-"))


def identities(r, cases):
    """Expressions to simplify: rational functions with common factors, sums of fractions, expansions that
    cancel, symbolic determinants and inverses. Checked at points, and for how far fractions are cancelled."""
    names = [x, y, p]

    def lin(vs):
        return sum(r.randint(-3, 3) * v for v in vs) + r.randint(-4, 4)

    def points(vs, e, k=3):
        pts = []
        while len(pts) < k:
            pt = {v: rnd_rat(r, -7, 7, (1, 2, 3, 5)) for v in vs}
            val = e.subs(pt) if not isinstance(e, sp.MatrixBase) else e.subs(pt)
            if isinstance(val, sp.MatrixBase):
                if any(v.has(sp.zoo, sp.nan) or not v.is_finite for v in val):
                    continue
            elif val.has(sp.zoo, sp.nan) or not val.is_finite:
                continue
            pts.append(pt)
        return pts

    def add(id_, e, vs, degrees=True):
        pts = points(vs, e)
        exp = "|".join(expected(sp.simplify(e.subs(pt)) if not isinstance(e, sp.MatrixBase) else e.subs(pt)) for pt in pts)
        deg = "-"
        if degrees:
            n, d = sp.fraction(sp.cancel(sp.together(e)))
            deg = ",".join(f"{v}:{sp.degree(n, v)}/{sp.degree(d, v)}" for v in vs)
        cases.append(("identity", id_, code(e) if not isinstance(e, str) else e, ",".join(map(str, vs)), "|".join(point_text(pt) for pt in pts), exp, deg))

    for i in range(30):
        vs = names[: r.randint(1, 3)]
        common = [lin(vs) for _ in range(r.randint(1, 2))]
        common = [c for c in common if c.free_symbols] or [vs[0] + 1]
        num_ = sp.Mul(*common, *[lin(vs) for _ in range(r.randint(0, 2))])
        den = sp.Mul(*common, *[lin(vs) for _ in range(r.randint(0, 1))])
        e = sp.expand(num_) / sp.expand(den)
        if e.free_symbols:
            add(f"cancel-{i}", e, sorted(e.free_symbols, key=str))
    for i in range(20):
        vs = names[: r.randint(1, 2)]
        terms = [rnd_rat(r) / lin(vs) for _ in range(r.randint(2, 3))]
        e = sp.Add(*terms, evaluate=False)
        if all(t.free_symbols for t in terms):
            add(f"fractions-{i}", e, sorted(e.free_symbols, key=str))
    for i in range(15):
        vs = names[: r.randint(1, 3)]
        f, g = lin(vs), lin(vs)
        e = sp.Add((f + g) ** 2, -f ** 2, -2 * f * g, -g ** 2, evaluate=False)
        add(f"expands-to-zero-{i}", e, vs, degrees=False)
    # identities that need rules beyond polynomials: the result must be a constant
    for i, (e, vs) in enumerate([
        (sp.sin(x) ** 2 + sp.cos(x) ** 2, [x]), (sp.cosh(x) ** 2 - sp.sinh(x) ** 2, [x]),
        (sp.log(sp.exp(x + y)) - x - y, [x, y]), (sp.exp(2 * sp.log(x)) - x ** 2, [x]),
        (sp.sqrt(x ** 2) - sp.Abs(x), [x]), (sp.tan(x) * sp.cos(x) - sp.sin(x), [x]),
        (sp.exp(x) * sp.exp(-x), [x]), (sp.log(x * y) - sp.log(x) - sp.log(y), [x, y]),
    ]):
        pts = [{v: rnd_rat(r, 1, 7, (1, 2, 3)) for v in vs} for _ in range(3)]
        cases.append(("identity", f"constant-{i}", code(sp.Add(*e.args, evaluate=False) if e.is_Add else e), ",".join(map(str, vs)),
                      "|".join(point_text(pt) for pt in pts), "|".join(expected(sp.simplify(e.subs(pt))) for pt in pts), "const"))
    # nested fractions
    for i in range(6):
        a_, b_ = rnd_rat(r), rnd_rat(r)
        e = 1 / (a_ / x + b_ / y)
        add(f"nested-fraction-{i}", e, [x, y])
    # symbolic matrices: determinants and inverses as formulas
    for i in range(10):
        n = r.choice([2, 2, 3, 3, 4])
        vs = names[: r.randint(1, 3)]
        M = sp.Matrix(n, n, lambda *_: r.choice([lin(vs), rnd_rat(r), r.choice(vs)]))
        if sp.expand(M.det()) == 0:
            continue
        cases.append(("identity", f"det-symbolic-{i}", f"det({code(M)})", ",".join(map(str, vs)),
                      *_pts_exp(r, vs, M.det(), points), "-"))
        if n <= 3:
            inv = M.inv()
            cases.append(("identity", f"inverse-symbolic-{i}", f"{code(M)}^(-1)", ",".join(map(str, vs)),
                          *_pts_exp(r, vs, inv, points), "-"))


def _pts_exp(r, vs, e, points):
    pts = points(vs, e)
    return "|".join(point_text(pt) for pt in pts), "|".join(expected(e.subs(pt)) for pt in pts)


def solves(r, cases):
    """Equations and systems: linear and quadratic with parameters (formulas expected), polynomials with
    numbers (every real root), rational and isolatable equations, transcendental ones with known roots,
    linear and nonlinear systems."""
    def solve_case(id_, eqs, unknowns, params, sols, how):
        eq_text = "; ".join(f"{code(l)} = {code(rr)}" for l, rr in eqs)
        sol_text = "|".join(",".join(num(s[v]) for v in unknowns) for s in sols) or "none"
        cases.append(("solve", id_, eq_text, ",".join(map(str, unknowns)), point_text(params) or "-", sol_text, how))

    def real_solutions(eqs, unknowns, params):
        sub = [(sp.sympify(l).subs(params), sp.sympify(rr).subs(params)) for l, rr in eqs]
        # one polynomial: real root isolation finds every real root (solve can drop roots of factors it can't
        # solve in radicals)
        if len(unknowns) == 1 and len(sub) == 1:
            n, d = sp.fraction(sp.together(sub[0][0] - sub[0][1]))
            if n.is_polynomial(unknowns[0]) and d.is_polynomial(unknowns[0]):
                roots = set(sp.Poly(n, unknowns[0]).real_roots()) if sp.degree(n, unknowns[0]) > 0 else set()
                return [{unknowns[0]: t} for t in sorted(roots, key=lambda t: sp.N(t)) if d.subs(unknowns[0], t) != 0]
        out = []
        for s in sp.solve([sp.Eq(l, rr) for l, rr in sub], unknowns, dict=True):
            vals = {v: sp.nsimplify(s[v]) if s[v].is_Float else s[v] for v in unknowns}
            if all(sp.im(sp.N(v, 30)) == 0 for v in vals.values()):
                out.append({k: sp.re(v) for k, v in vals.items()})
        return out

    # linear with parameters: a formula of the parameters
    for i in range(20):
        a, b, c = [r.choice([p, q, w, rnd_rat(r)]) for _ in range(3)]
        if not ({a, b, c} & {p, q, w}) or a == 0:
            continue
        eqs = [(a * x + b, c)]
        params = {s: rnd_rat(r) for s in {a, b, c} & {p, q, w}}
        sols = real_solutions(eqs, [x], params)
        if sols:
            solve_case(f"linear-params-{i}", eqs, [x], params, sols, "exact")
    # quadratics: rational roots, irrational roots, no real roots; with parameters
    for i in range(20):
        r1, r2 = rnd_rat(r), rnd_rat(r)
        k = rnd_rat(r)
        eqs = [(sp.expand(k * (x - r1) * (x - r2)), 0)]
        solve_case(f"quadratic-rational-{i}", eqs, [x], {}, real_solutions(eqs, [x], {}), "exact")
        c1, c2 = r.randint(-9, 9), r.randint(-9, 9)
        eqs = [(x ** 2 + c1 * x, c2)]
        solve_case(f"quadratic-{i}", eqs, [x], {}, real_solutions(eqs, [x], {}), "exact")
        eqs = [(x ** 2 + p * x + q, 0)]
        pv, qv = rnd_rat(r), rnd_rat(r)
        if pv ** 2 - 4 * qv > 0:
            solve_case(f"quadratic-params-{i}", eqs, [x], {p: pv, q: qv}, real_solutions(eqs, [x], {p: pv, q: qv}), "exact")
    # polynomials of degree 3 to 6 with numbers: every real root
    for i in range(25):
        deg = r.randint(3, 6)
        roots = [rnd_rat(r) for _ in range(r.randint(1, deg))]
        poly = sp.expand(sp.Mul(*[(x - t) for t in roots]) * sp.Poly(sp.Add(*[r.randint(-3, 3) * x ** j for j in range(deg - len(roots) + 1)]) + x ** (deg - len(roots)), x).as_expr())
        if sp.degree(poly, x) < 1:
            continue
        eqs = [(poly, 0)]
        solve_case(f"polynomial-{i}", eqs, [x], {}, real_solutions(eqs, [x], {}), "numeric")
    # equations whose squaring or isolating brings in roots that aren't solutions
    for i in range(10):
        a, b = rnd_rat(r, -5, 5), rnd_rat(r, -5, 5)
        eq = (sp.sqrt(x + a), x + b)
        sols = [s_ for s_ in real_solutions([(x + a, (x + b) ** 2)], [x], {}) if sp.N(s_[x] + b) >= 0]
        solve_case(f"sqrt-extraneous-{i}", [eq], [x], {}, sols, "exact")
        eq = (sp.Abs(x - a), sp.Abs(b))
        solve_case(f"abs-{i}", [eq], [x], {}, [{x: a - abs(b)}, {x: a + abs(b)}] if b != 0 else [{x: a}], "exact")
        a2, b2 = rnd_rat(r, 1, 6), rnd_rat(r, 1, 6)
        if a2 != b2:
            eq = (sp.exp(2 * x) - (a2 + b2) * sp.exp(x) + a2 * b2, 0)
            solve_case(f"quadratic-in-exp-{i}", [eq], [x], {}, [{x: sp.log(a2)}, {x: sp.log(b2)}], "exact")
    # every root of sin(x) = x/c: all lie in [-c, c]
    for c in [3, 10, 25]:
        f = lambda v, c=c: mpmath.sin(v) - v / c
        grid = [mpmath.mpf(-c) + j * mpmath.mpf(2 * c) / 20000 for j in range(20001)]
        roots = sorted({mpmath.nstr(mpmath.findroot(f, (u, v), solver="anderson"), 25) for u, v in zip(grid, grid[1:]) if f(u) == 0 or f(u) * f(v) < 0})
        solve_case(f"sin-line-{c}", [(sp.sin(x), x / c)], [x], {}, [{x: sp.Float(v, 25)} for v in roots], "numeric")
    # cubics with parameters: a formula (Cardano) for one real root
    for i in range(5):
        pv, qv = rnd_rat(r, 1, 6), rnd_rat(r)
        eqs = [(x ** 3 + p * x + q, 0)]
        solve_case(f"cubic-params-{i}", eqs, [x], {p: pv, q: qv}, real_solutions(eqs, [x], {p: pv, q: qv}), "exact")
    # rational equations
    for i in range(15):
        a, b, c = rnd_rat(r), rnd_rat(r), rnd_rat(r)
        if c == 1:
            continue
        eqs = [((x + a) / (x - b), c)]
        solve_case(f"rational-{i}", eqs, [x], {}, real_solutions(eqs, [x], {}), "exact")
        eqs = [(1 / (x - a) + 1 / (x - b), c)]
        if a != b:
            solve_case(f"rational-sum-{i}", eqs, [x], {}, real_solutions(eqs, [x], {}), "numeric")
    # isolatable: x once
    for i in range(15):
        a, b = rnd_rat(r, 1, 9), rnd_rat(r, 1, 9)
        for name, eq in [("exp", (sp.exp(a * x), b)), ("ln", (sp.log(x + a), b)), ("sqrt", (sp.sqrt(x + a), b)),
                         ("cube", ((x + a) ** 3, -b)), ("power", (a * 2 ** x, b))]:
            solve_case(f"isolate-{name}-{i}", [eq], [x], {}, real_solutions([eq], [x], {}), "exact")
    # transcendental with roots known in closed form: x e^x = c (Lambert W, one or two real roots)
    for i in range(10):
        c = sp.Rational(r.randint(-36, -1), 100) if i % 2 == 0 else rnd_rat(r, 1, 9)
        roots = [sp.LambertW(c)] + ([sp.LambertW(c, -1)] if c < 0 else [])
        solve_case(f"lambert-{i}", [(x * sp.exp(x), c)], [x], {}, [{x: t} for t in roots], "numeric")
    solve_case("dottie", [(sp.cos(x), x)], [x], {}, [{x: sp.Float(mpmath.findroot(lambda t: mpmath.cos(t) - t, 0.7), 30)}], "numeric")
    # linear systems: numbers, and with parameters
    for i in range(15):
        n = r.randint(2, 3)
        vs = [x, y, z][:n]
        A = sp.Matrix(n, n, lambda *_: r.randint(-5, 5))
        if A.det() == 0:
            continue
        b = [rnd_rat(r) for _ in range(n)]
        eqs = [(sum(A[i_, j] * vs[j] for j in range(n)), b[i_]) for i_ in range(n)]
        solve_case(f"linear-system-{i}", eqs, vs, {}, real_solutions(eqs, vs, {}), "exact")
        eqs = [(p * x + y, 1), (x - q * y, 2)]
        pv, qv = rnd_rat(r), rnd_rat(r)
        if -pv * qv - 1 != 0:
            solve_case(f"linear-system-params-{i}", eqs, [x, y], {p: pv, q: qv}, real_solutions(eqs, [x, y], {p: pv, q: qv}), "exact")
    # matrix equations, element by element
    for i in range(8):
        A = sp.Matrix(2, 2, lambda *_: r.randint(-5, 5))
        if A.det() == 0:
            continue
        b = sp.Matrix(2, 1, lambda *_: rnd_rat(r))
        sol = A.LUsolve(b)
        cases.append(("solve", f"matrix-equation-{i}", f"{code(A)} vec(x, y) = {code(b)}", "x,y", "-", f"{num(sol[0])},{num(sol[1])}", "exact"))
    # a circle and a line: two, one or no intersections
    for i in range(10):
        rr, m, c = rnd_rat(r, 1, 5), rnd_rat(r), rnd_rat(r)
        eqs = [(x ** 2 + y ** 2, rr ** 2), (y, m * x + c)]
        solve_case(f"circle-line-{i}", eqs, [x, y], {}, real_solutions(eqs, [x, y], {}), "numeric")


def main():
    os.makedirs(OUT, exist_ok=True)
    for name, family in [("values", values), ("calculus", calculus), ("identities", identities), ("solving", solves)]:
        # a seed per file: cases added to one file leave the others as they are
        r = random.Random(f"20261023-{name}")
        cases = []
        family(r, cases)
        with open(os.path.join(OUT, name + ".txt"), "w") as f:
            f.write(f"# generated by tests/oracle/generate.py (SymPy {sp.__version__}, mpmath {mpmath.__version__})\n")
            for c in cases:
                f.write("\t".join(c) + "\n")
        print(f"{name}: {len(cases)} cases")


if __name__ == "__main__":
    main()
