// Expressions: evaluation with units, parsing, simplifying, isolating.
#import "numbers.typ": *

// ---------- evaluate with units -> (value, dims) ----------
// whether x op y holds, as 1 or 0; = and ≠ allow for rounding
#let _holds(op, x, y) = {
  let same = calc.abs(x - y) <= 1e-9 * calc.max(calc.abs(x), calc.abs(y), 1e-300)
  let t = if op == "=" { same } else if op in ("≠", "!=") { not same } else if op == "<" { x < y and not same } else if op == ">" { x > y and not same } else if op in ("<=", "≤") { x < y or same } else { x > y or same }
  if t { 1.0 } else { 0.0 }
}
#let _ev(e, d) = {
  let (k, ..a) = e
  if k == "n" { return (a.first(), _Z) }
  if k == "q" { return (a.first().v, a.first().d) }
  if k == "c" { return (if a.first() == "e" { calc.e } else { calc.pi }, _Z) }
  if k == "v" { let q = d.at(a.first(), default: none); return if q == none { _err("unknown variable '" + a.first() + "'") } else { (q.v, q.d) } }
  // a comparison, ("cmp", op, l, r): 1 when it holds, else 0
  if k == "cmp" {
    let (l, r) = (_ev(a.at(1), d), _ev(a.at(2), d))
    if _E(l) { return l }
    if _E(r) { return r }
    // values of different units are never equal: = and orderings are false, ≠ true
    if l.at(1).map(z => calc.round(z, digits: 6)) != r.at(1).map(z => calc.round(z, digits: 6)) { return (if a.first() in ("≠", "!=") { 1.0 } else { 0.0 }, _Z) }
    return (_holds(a.first(), l.at(0), r.at(0)), _Z)
  }
  // (e: u): e's number (in SI) as a value in u, whatever unit e had
  if k == "as" { let r = _ev(a.first(), d); return if _E(r) { r } else { (r.at(0) * a.at(1).f, a.at(1).d) } }
  let xs = a.filter(t => type(t) == array).map(t => _ev(t, d))
  let b = _first-err(xs)
  if b != none { return b }
  if k == "-" { let (v, u) = xs.first(); (-v, u) }
  else if k == "+" {
    if xs.any(x => x.at(1) != xs.first().at(1)) { return _err("adding different units") }
    (xs.map(x => x.at(0)).sum(), xs.first().at(1))
  }
  else if k == "*" { xs.reduce((x, y) => (x.at(0) * y.at(0), _add(x.at(1), y.at(1)))) }
  else if k == "/" { let (x, y) = xs; if y.at(0) == 0 { return _err("division by zero") }; (x.at(0) / y.at(0), _add(x.at(1), y.at(1), k: -1)) }
  else if k == "^" {
    let (x, y) = xs
    if y.at(1) != _Z { return _err("an exponent must be unitless") }
    if x.at(0) == 0 and y.at(0) < 0 { return _err("division by zero (0 to a negative power)") }
    if x.at(0) < 0 and calc.round(y.at(0)) != y.at(0) { return _err("a negative number to a fractional power is not real") }
    (calc.pow(float(x.at(0)), y.at(0)), x.at(1).map(z => calc.round(z * y.at(0), digits: 6)))
  } else {
    let (v, u) = xs.first()
    let f = a.first()
    if f == "sqrt" { return if v < 0 { _err("square root of a negative number") } else { (calc.sqrt(v), u.map(z => z / 2)) } }
    if f == "abs" { return (calc.abs(v), u) }
    if u != _Z { return _err("the argument of " + f + " must be unitless") }
    if f in ("ln", "log") and v <= 0 { return _err(f + " of a number ≤ 0") }
    if f in ("asin", "acos") and calc.abs(v) > 1 { return _err(f + " of a number outside −1…1") }
    let r = (exp: calc.exp, ln: calc.ln, log: calc.log, sin: calc.sin, cos: calc.cos, tan: calc.tan,
      asin: x => calc.asin(x).rad(), acos: x => calc.acos(x).rad(), atan: x => calc.atan(x).rad()).at(f)(v)
    if not _ok(r) { return _err(f + " is out of range here") }
    (r, _Z)
  }
}

// ---------- expressions: ("n", v) ("c", name) ("v", name) ("+", ..) ("*", ..) ("-", a) ("/", a, b) ("^", a, b) ("f", name, a) ----------
#let _F = ("sqrt", "exp", "ln", "log", "sin", "cos", "tan", "asin", "acos", "atan", "abs")
// "as" ends a term too: U as A casts it
#let _op = ("+", "-", "−", "*", "·", "⋅", "×", "/", "^", ")", "=", ",", "as", none)
#let _subst(e, m) = if e.at(0) == "v" and e.at(1) in m { m.at(e.at(1)) } else { (e.at(0), ..e.slice(1).map(a => if type(a) == array { _subst(a, m) } else { a })) }
#let _GR = "alpha|beta|gamma|Gamma|delta|Delta|epsilon|zeta|eta|theta|Theta|iota|kappa|lambda|Lambda|mu|nu|xi|Xi|pi|Pi|rho|sigma|Sigma|tau|upsilon|phi|Phi|chi|psi|Psi|omega|Omega"
// a Greek letter spelled out, alone or before an index (Delta_R, alpha2), is the letter, as math reads it: Δ_R;
// DeltaT stays a name of its own
#let _greek(n) = {
  let m = n.match(regex("^(" + _GR + ")((?:_|\\d).*)?$"))
  if m == none { n } else { str(eval("sym." + m.captures.at(0))) + if m.captures.at(1) == none { "" } else { m.captures.at(1) } }
}
// math glues names with empty strings (Delta""r_R is Δr_R, Delta""Gamma is ΔΓ); strings and code may too
#let _glue(s) = s.replace(regex("\\p{L}[\\p{L}\\p{N}]*(?:\"\"\\p{L}[\\p{L}\\p{N}]*)+"), m => m.text.split("\"\"").map(_greek).join())
// a name given on its own (unknown: "tau", lambdify(…, "Delta\"\"r")), as an equation would read it
#let _resolve(n) = _greek(_glue(n))
#let _resolve-keys(x) = if type(x) == dictionary { x.pairs().map(((k, v)) => (_resolve(k), v)).to-dict() } else { x }
// the Greek letters, as characters
#let _GRC = _GR.split("|").map(n => str(eval("sym." + n)))
// an argument for a parameter with a unit u (f(I: mA)): a plain number gets the unit (f(2) is f(2 mA)), a value must
// have its dimension; one not known yet (being solved for, plotted against) is passed as it is
#let _typed(a, u, f, p, vars, U) = {
  let r = _ev(a, vars)
  if _E(r) or r.at(1) == u.d { return a }
  if r.at(1) != _Z { return _err(f + " takes " + p + " in " + u.u + ", got " + _auto(r.at(1), U: U)) }
  let one = (v: u.f, d: u.d, raw: "1", n: 1.0, u: u.u, ex: none, bare: true)
  if a.at(0) == "n" { ("q", one + (v: a.at(1) * u.f, raw: str(a.at(1)), n: a.at(1), bare: false)) } else { ("*", a, ("q", one)) }
}
#let _tk(s) = s.matches(regex("\\[[^\\]]*\\]|\\d+\\.?\\d*(?:[eE][-+]?\\d+)?|\\.\\d+|@?\\p{L}[\\p{L}\\p{N}_]*|\\S")).map(m => m.text)
// after a term, "as u" casts it to the unit u, one word, [a bracket], (parentheses) or "quoted": U as A, (R I) as [m/s],
// U as (A as), x as as; after a number as is the unit, attosecond (3 as), so numbers aren't cast -> (node, next index) or an error
#let _cast(t, j, e, d) = {
  if e.at(0) == "n" { return (e, j) }
  while t.at(j, default: none) == "as" {
    let u = t.at(j + 1, default: none)
    if u == none or (u in _op and u != "as") { return _err("a unit after 'as' to cast to, like U as A") }
    // a unit word or (group), then more joined by * or /, as long as they're units, not variables: (A as)/V
    let U = d.at("U", default: _U)
    let atom(k) = {
      let a = t.at(k, default: none)
      if a == "(" {
        let (dp, e) = (1, k + 1)
        while e < t.len() and dp > 0 { if t.at(e) == "(" { dp += 1 } else if t.at(e) == ")" { dp -= 1 }; e += 1 }
        e
      } else if a != none and (a.starts-with("[") or (a.match(regex("^\\p{L}")) != none and a not in d.v and _known(a, U))) { k + 1 }
    }
    let k = atom(j + 1)
    if k == none { k = j + 2 }
    while t.at(k, default: none) in ("*", "/", "·", "⋅") and atom(k + 1) != none { k = atom(k + 1) }
    let u = t.slice(j + 1, k).map(x => x.trim("[", at: start).trim("]", at: end).trim("\"")).join(" ")
    let r = _uexpr(u, U: U)
    if _E(r) { return _err("the unit to cast to: " + r.err) }
    e = ("as", e, (f: r.at(0), d: r.at(1), u: r.at(2)))
    j = k
  }
  (e, j)
}
#let _p(t, i, l, d) = {
  let at(j) = t.at(j, default: none)
  if l == 0 {
    let r = _p(t, i, 1, d)
    if _E(r) { return r }
    let (a, i) = r
    while at(i) in ("+", "-", "−") {
      let r = _p(t, i + 1, 1, d)
      if _E(r) { return r }
      let (b, j) = r
      let b = if at(i) == "+" { b } else { ("-", b) }
      a = if a.at(0) == "+" { a + (b,) } else { ("+", a, b) }; i = j // flat, so long sums don't nest deeply
    }
    (a, i)
  } else if l == 1 {
    let r = _p(t, i, 2, d)
    if _E(r) { return r }
    let (a, i) = r
    while at(i) not in _op or at(i) in ("*", "·", "⋅", "×", "/") {
      let o = at(i)
      let r = _p(t, if o in ("*", "·", "⋅", "×", "/") { i + 1 } else { i }, 2, d)
      if _E(r) { return r }
      let (b, j) = r
      a = if o == "/" { ("/", a, b) } else if a.at(0) == "*" { a + (b,) } else { ("*", a, b) }; i = j
    }
    (a, i)
  } else {
    // l 2: sign, atom, power in one frame (deep nesting stays within typst's call depth); l 4: atom only
    if l == 2 and at(i) in ("-", "−") { let r = _p(t, i + 1, 2, d); return if _E(r) { r } else { (("-", r.at(0)), r.at(1)) } }
    if l == 2 and at(i) == "+" { return _p(t, i + 1, 2, d) }
    let k = at(i)
    let r = if k == "(" {
      let r = _p(t, i + 1, 0, d)
      if _E(r) { r } else if at(r.at(1)) != ")" { _err("missing ')'") } else { (r.at(0), r.at(1) + 1) }
    } else if k == none { _err("unexpected end of the expression") }
    else if k.starts-with("[") {
      let num(s) = { let r = _p(_tk(s), 0, 0, (v: (:), f: (:), keep: false)); if _E(r) { r } else { let v = _ev(r.at(0), (:)); if _E(v) { v } else { v.at(0) } } }
      let U = d.at("U", default: _U)
      let inner = k.slice(1, -1).trim()
      // [A]: one ampere, shown as just the unit
      let bare = inner != "" and inner.match(regex("^[-+−\d.(]")) == none
      let q = _q0(if bare { "1 " + inner } else { inner }, num, U: U)
      let q = if bare and not _E(q) { q + (bare: true) } else { q }
      if _E(q) { q } else { (("q", q), i + 1) }
    }
    else if k.match(regex("^(?:\d|\.\d)")) != none { (("n", float(k)), i + 1) }
    else if k.starts-with("@") {
      let n = _greek(k.slice(1))
      if n not in d.v or d.v.at(n).at("ex", default: none) == none { _err("no stored expression for '" + n + "'") } else { (d.v.at(n).ex, i + 1) }
    } else if k in d.f and at(i + 1) == "(" {
      // user function f(a, b): expanded into its body, or kept as a call for display
      let fn = d.f.at(k)
      let args = ()
      let j = i + 2
      let bad = none
      if at(j) != ")" {
        let go = true
        while go {
          let r = _p(t, j, 0, d)
          if _E(r) { bad = r; go = false } else {
            args.push(r.at(0))
            j = r.at(1)
            if at(j) == "," { j += 1 } else { go = false }
          }
        }
      }
      if bad != none { bad }
      else if at(j) != ")" { _err("missing ')' after the arguments of " + k) }
      else if args.len() != fn.params.len() { _err(k + " takes " + str(fn.params.len()) + " argument(s), got " + str(args.len())) }
      else if d.keep { (("f", k, if args.len() == 1 { args.first() } else { ("tuple", ..args) }), j + 1) }
      else {
        let us = fn.at("units", default: (:))
        let args = fn.params.zip(args).map(((p, a)) => if p in us { _typed(a, us.at(p), k, p, d.v, d.at("U", default: _U)) } else { a })
        let b = _first-err(args)
        if b != none { b } else { (_subst(fn.body, fn.params.zip(args).to-dict()), j + 1) }
      }
    } else if k in _F and at(i + 1) == "(" { let r = _p(t, i + 1, 4, d); if _E(r) { r } else { (("f", k, r.at(0)), r.at(1)) } }
    else if (k in ("pi", "π") or k == "e") and _greek(k) not in d.v { (("c", k), i + 1) }
    else if k.match(regex("^\p{L}")) != none { (("v", _greek(k)), i + 1) }
    else { _err("unexpected '" + k + "'") }
    if _E(r) { return r }
    let (b, i) = r
    let r = if l == 2 and at(i) == "^" {
      let r = _p(t, i + 1, 2, d)
      // U^2 as V casts the power, not the exponent
      if _E(r) { r } else if r.at(0).at(0) == "as" { (("as", ("^", b, r.at(0).at(1)), r.at(0).at(2)), r.at(1)) } else { (("^", b, r.at(0)), r.at(1)) }
    } else { (b, i) }
    // a term with its power, cast: U^2 as V
    if l == 2 and not _E(r) { _cast(t, r.at(1), r.at(0), d) } else { r }
  }
}
#let _parse(s, d, f: (:), keep: false, U: _U) = {
  let t = _tk(_glue(s))
  if t.len() == 0 { return _err("empty expression") }
  let r = _p(t, 0, 0, (v: d, f: f, keep: keep, U: U))
  if _E(r) { return _err(r.err + " in '" + s.trim() + "'") }
  let (a, i) = r
  if i < t.len() { return _err("unexpected '" + t.at(i) + "' in '" + s.trim() + "'") }
  a
}
#let _isu(u, U: _U) = type(u) == str and u.trim() != "" and _tok(u, U: U).flatten().chunks(2).all(((n, _)) => n == "1" or _known(n, U))
#let _isq(x, U: _U) = {
  if type(x) in (int, float) { return true }
  let N = "[-+−]?[\\d.,()]+(?:[eE][-+−]?\\d+)?"
  let x = _str(x)
  if _E(x) { return false }
  let m = x.trim().match(regex("^(" + N + "(?:\\s*[-+−*/·⋅^]\\s*" + N + ")*)\\s*(.*)$"))
  if m == none { return false }
  let (n, u) = m.captures
  // 3*h and 1/(h) are a number times a variable, not 3 hours
  let bal = n.matches("(").len() == n.matches(")").len()
  // as after a unit casts: 2 A as mA is (2 A) as mA, while 3 as is attoseconds
  if u.trim().split(regex("\\s+")).slice(1).contains("as") { return false }
  n.match(regex("^[-+−(]*[\\d.]")) != none and bal and u.match(regex("^[*·⋅]")) == none and (u.trim() == "" or _isu(u, U: U))
}
#let _q(x, U: _U) = _q0(x, U: U, s => { let e = _parse(s, (:)); if _E(e) { return e }; let v = _ev(e, (:)); if _E(v) { v } else { v.at(0) } })
#let _vars(e) = if e.at(0) == "v" { (e.at(1),) } else { e.slice(1).filter(a => type(a) == array).map(_vars).flatten() }
#let _has(e, x) = x in _vars(e)
// a function's parameter list as written, "I: mA, R, ..": names, their units ([…] as math writes them) and whether
// .. (or as implicit-parameters says, see _DEF) lets the body's other undefined names be parameters too -> (names, units, rest), or an error
#let _plist(src, name, U: _U, implicit: "no") = {
  let items = src.split(",").map(str.trim).filter(t => t != "")
  let (names, units, anys, rest) = ((), (:), (), false)
  for (i, it) in items.enumerate() {
    // math writes . . and …
    if it.replace(regex("\\s"), "") in ("..", "…") {
      if i != items.len() - 1 { return _err("'..' comes last in " + name + "(…): it stands for the rest") }
      rest = true
      continue
    }
    let (n, ..u) = it.split(":")
    let n = n.trim()
    let u = u.join(":", default: "").trim().trim("[", at: start).trim("]", at: end).trim()
    if n.match(regex("^\p{L}[\p{L}\p{N}_]*$")) == none { return _err("the parameters of " + name + " are names, optionally with a unit (I: mA), and .. for the rest, got '" + it + "'") }
    let g = _greek(n)
    names.push(g)
    // any: every unit, the result's follows it
    if u == "any" { anys.push(g) } else if u != "" {
      let r = _uexpr(u, U: U)
      if _E(r) { return _err("the unit of " + n + " in " + name + ": " + r.err) }
      units.insert(g, (f: r.at(0), d: r.at(1), u: r.at(2)))
    }
  }
  // implicit-parameters: "empty" makes f() short for f(..), "allow" every list
  (names: names, units: units, anys: anys, rest: rest or implicit == "allow" or (implicit == "empty" and items.len() == 0))
}
// the types untyped parameters get from the functions they're passed to as they are, read from a body kept with its
// calls (keep: true): f(y) = q(y) with q(x: A) makes y an A, with q(x: any) an any -> ((name, unit or "any"), …)
#let _passed(e, fns) = {
  if type(e) != array { return () }
  let out = ()
  if e.at(0) == "f" and e.at(1) in fns {
    let fn = fns.at(e.at(1))
    let args = if e.at(2).at(0) == "tuple" { e.at(2).slice(1) } else { (e.at(2),) }
    for (p, a) in fn.params.zip(args) {
      if a.at(0) != "v" { continue }
      if p in fn.at("units", default: (:)) { out.push((a.at(1), fn.units.at(p))) }
      else if p in fn.at("anys", default: ()) { out.push((a.at(1), "any")) }
    }
  }
  for t in e.slice(1) { out += _passed(t, fns) }
  out
}
// the parameters with a unit as values of it, so a body is read knowing them: q(x) with q(x: A) checks x
#let _pvars(pl) = pl.units.pairs().map(((n, u)) => (n, (v: u.f, d: u.d, raw: none, n: 1.0, u: u.u, ex: none))).to-dict()
// the parameters of a function with this body: those written, then with .. its other undefined names in order;
// without .. those are an error
#let _pfree(body, pl, vars, name) = {
  let free = _vars(body).dedup().filter(v => v not in pl.names and v not in vars)
  if free.len() == 0 or pl.rest { return pl.names + free }
  let one = free.len() == 1
  _err(free.map(v => "'" + v + "'").join(", ", last: " and ") + " in " + name + " " + (if one { "isn't" } else { "aren't" }) + " defined – define " + (if one { "it" } else { "them" }) + " first, or let " + name + " take " + (if one { "it" } else { "them" }) + ": " + name + "(" + (pl.names + free).join(", ") + ") or " + name + "(" + (pl.names + ("..",)).join(", ") + ")" + " (or set solve-config(implicit-parameters: " + (if pl.names.len() == 0 { "\"empty\"" } else { "\"allow\"" }) + "))")
}

// ---------- simplify (just enough for nice output) ----------
#let _s(e) = {
  let (k, ..a) = e
  if k in ("n", "c", "v", "q") { return e }
  if k == "as" { return ("as", _s(a.at(0)), a.at(1)) }
  if k == "cmp" { return ("cmp", a.at(0), _s(a.at(1)), _s(a.at(2))) }
  if k == "f" {
    let x = _s(a.at(1))
    let inv = (exp: "ln", ln: "exp")
    return if x.at(0) == "f" and inv.at(a.at(0), default: none) == x.at(1) { x.at(2) } else { ("f", a.at(0), x) }
  }
  if k == "^" {
    let (b, x) = a.map(_s)
    if b.at(0) == "^" and b.at(2).at(0) == "n" and x.at(0) == "n" { (b, x) = (b.at(1), ("n", b.at(2).at(1) * x.at(1))) }
    if x.at(0) == "n" {
      let v = x.at(1)
      if b.at(0) == "n" and (b.at(1) > 0 or (b.at(1) == 0 and v >= 0) or (b.at(1) < 0 and calc.round(v) == v)) { return ("n", calc.pow(b.at(1), v)) }
      if v == 1 { return b }
      if v == 0.5 { return ("f", "sqrt", b) }
      if v < 0 { return _s(("/", ("n", 1), ("^", b, ("n", -v)))) }
    }
    if b.at(0) == "f" and b.at(1) == "sqrt" and x == ("n", 2) { return b.at(2) }
    return ("^", b, x)
  }
  if k == "-" { let x = _s(a.first()); if x.at(0) == "+" { return _s(("+", ..x.slice(1).map(t => ("-", t)))) } }
  if k == "+" {
    let ts = a.map(_s).map(t => if t.at(0) == "+" { t.slice(1) } else { (t,) }).sum(default: ())
    let ns = ts.filter(t => t.at(0) == "n")
    let ts = if ns.len() < 2 { ts } else { ts.filter(t => t.at(0) != "n") + (("n", ns.map(t => t.at(1)).sum()),) }
    let ts = ts.filter(t => t != ("n", 0))
    let ts = ts.filter(t => t.at(0) != "-") + ts.filter(t => t.at(0) == "-")
    return if ts.len() == 0 { ("n", 0) } else if ts.len() == 1 { ts.first() } else { ("+", ..ts) }
  }
  // "*", "/", "-": collect into cn/cd · Π num / Π den
  let md(e) = {
    let (k, ..a) = e
    if k == "n" { (e.at(1), 1, (), ()) }
    else if k == "-" { let (p, q, n, m) = md(a.first()); (-p, q, n, m) }
    else if k == "*" { a.map(md).fold((1, 1, (), ()), (r, x) => (r.at(0) * x.at(0), r.at(1) * x.at(1), r.at(2) + x.at(2), r.at(3) + x.at(3))) }
    else if k == "/" { let (x, y) = a.map(md); (x.at(0) * y.at(1), x.at(1) * y.at(0), x.at(2) + y.at(3), x.at(3) + y.at(2)) }
    else { let s = _s(e); if s.at(0) in ("*", "/", "-") and s != e { md(s) } else if s.at(0) == "n" { (s.at(1), 1, (), ()) } else { (1, 1, (s,), ()) } }
  }
  let (p, q, n, m) = md(e)
  if q == 0 { return e }
  if p == 0 { return ("n", 0) }
  if q < 0 { (p, q) = (-p, -q) }
  // cancel equal factors pairwise (c·c/c → c)
  let keep = ()
  for x in n { let i = m.position(y => y == x); if i != none { let _ = m.remove(i) } else { keep.push(x) } }
  n = keep
  let neg = p < 0
  p = calc.abs(p)
  if n == () and m == () { return ("n", if neg { -p / q } else { p / q }) }
  let rk = (n: 0, c: 1, v: 2, q: 2, "^": 3, f: 4)
  let prod(c, xs) = { let xs = xs.sorted(key: x => rk.at(x.at(0), default: 5)); let xs = (if c != 1 { (("n", c),) } else { () }) + xs; if xs.len() == 0 { ("n", 1) } else if xs.len() == 1 { xs.first() } else { ("*", ..xs) } }
  let r = if m == () and q == 1 { prod(p, n) } else { ("/", prod(p, n), prod(q, m)) }
  if neg { ("-", r) } else { r }
}

// ---------- solve: isolate x (must occur once) ----------
#let _iso(l, r, x) = {
  if _has(l, x) and _has(r, x) { return none }
  if not _has(l, x) { (l, r) = (r, l) }
  while l != ("v", x) {
    let (k, ..a) = l
    // through a cast the unit of x isn't known: left to the numeric solver
    if k == "as" { return none }
    let i = a.position(t => type(t) == array and _has(t, x))
    let rest = a.enumerate().filter(p => p.at(0) != i).map(p => p.at(1))
    if a.filter(t => type(t) == array and _has(t, x)).len() > 1 { return none }
    if k == "+" { r = ("+", r, ("-", ("+", ..rest))) }
    else if k == "*" { r = ("/", r, ("*", ..rest)) }
    else if k == "-" { r = ("-", r) }
    else if k == "/" { r = if i == 0 { ("*", r, a.at(1)) } else { ("/", a.at(0), r) } }
    else if k == "^" { r = if i == 0 { ("^", r, ("/", ("n", 1), a.at(1))) } else { ("/", ("f", "ln", r), ("f", "ln", a.at(0))) } }
    else if k == "f" {
      let f = a.at(0)
      r = if f == "sqrt" { ("^", r, ("n", 2)) } else if f == "log" { ("^", ("n", 10), r) } else {
        ("f", (exp: "ln", ln: "exp", sin: "asin", cos: "acos", tan: "atan", asin: "sin", acos: "cos", atan: "tan").at(f, default: none), r)
      }
      if r.at(1) == none { return none }
    }
    l = a.at(i)
  }
  r
}
// the option units applied to the built-in units: (bar: "100 kPa", rpm: "1/min", T: none, ü: auto) -> unit table, or an error
#let _mktab(units) = {
  let U = _U
  // base units of your own take the slots after the SI ones, in order
  let own = 0
  for (k, v) in units {
    if v == none {
      if k not in U { return _err("units: '" + k + "' is not a unit, so it can't be removed") }
      let _ = U.remove(k)
      continue
    }
    if k.match(regex("^[^\\s\\d*/^()\\[\\]+\\-−·⋅=,.;]+$")) == none { return _err("units: '" + k + "' can't be a unit name (no spaces, digits or operators)") }
    if v == auto {
      if own == _NX { return _err("units: at most " + str(_NX) + " base units of your own, '" + k + "' is one more") }
      let d = _Z
      d.at(7 + own) = 1
      U.insert(k, (1.0, d))
      own += 1
      continue
    }
    let s = _str(v)
    if _E(s) { return s }
    let q = _q(s, U: U)
    if _E(q) { return _err("units: " + k + ": " + q.err) }
    U.insert(k, (q.v, q.d))
  }
  U
}
