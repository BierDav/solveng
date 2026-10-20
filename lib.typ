// solveng – unit-aware "given → solve" calculations with minimal boilerplate.
//
//   #def("R = 7.2 kΩ", "I = 415 µA")  // stores the variables
//   #solve("U = R I")                  // finds the unknown, rearranges, substitutes, evaluates
//   #solve("Q0/2 = I t", unit: "h")    // optional target unit
//   #solve("Q = @I t")                 // @I inlines the stored expression of I
//   #val("U")  #reset  #solve-config(digits: 3, si: true)
//   #solve-block[ $I = "30 mA"$ … ]     // math equations and code blocks inside are definitions

#import "@preview/simple-plot:1.1.0" as _sp
#let _st = state("solveng", (:))
// ---------- configuration: #solve-config(..) globally, #solve-config(..)[..] / #show: solve-config.with(..) scoped, or per call ----------
#let _DEF = (
  // numbers
  digits: 4, zeros: false, sep: "lang", group: none, sci: (-3, 6), eng: false, times: "dot",
  form: "auto", frac-max: 1000, slash: false,
  // units
  prefixes: "all", si: false,
  // solve output
  steps: "full", inline-steps: "result", layout: "row", implies: sym.arrow.r.double, approx: true, block: true, solutions: 6,
  // definitions output
  definitions: "grid",
  // problems: shown in the document, or stop the compilation
  errors: "document",
)
#let _CH = (
  form: ("auto", "decimal", "fraction", "input"), times: ("dot", "times"), steps: ("full", "formula", "result", "value", auto), inline-steps: ("full", "formula", "result", "value"),
  layout: ("row", "column"), errors: ("document", "panic"), definitions: ("grid", "column", "inline", "list"),
)
// allowed types per option
#let _TY = (
  digits: (int,), zeros: (bool,), sep: (str,), group: (type(none), str, symbol, content), sci: (type(none), array), eng: (bool,),
  times: (str,), form: (str,), frac-max: (int,), slash: (bool,), prefixes: (str, type(none), array), si: (bool,),
  steps: (str, bool, type(auto)), inline-steps: (str,), layout: (str,), implies: (content, symbol, str), approx: (bool,), block: (bool,),
  solutions: (int,), definitions: (str,), errors: (str,),
)
#let _tyname(ts) = ts.map(t => if t == type(none) { "none" } else if t == type(auto) { "auto" } else { str(t) }).join(" or ")
#let _check(o) = {
  if type(o) != dictionary { return (err: "config: expected a dictionary of options like (digits: 3), got " + str(type(o))) }
  let o = o
  if type(o.at("steps", default: none)) == bool { o.steps = if o.steps { "full" } else { "result" } }
  for (k, v) in o {
    if k == "list" { return (err: "the option list is now called definitions: (definitions: " + repr(v) + ")") }
    if k not in _DEF { return (err: "unknown option '" + k + "' (options: " + _DEF.keys().join(", ") + ")") }
    if type(v) not in _TY.at(k) { return (err: k + ": expected " + _tyname(_TY.at(k)) + ", got " + repr(v)) }
    if k in _CH and v not in _CH.at(k) { return (err: k + ": " + repr(v) + " – use one of " + _CH.at(k).map(repr).join(", ")) }
  }
  if "digits" in o and (o.digits < 1 or o.digits > 15) { return (err: "digits must be 1…15") }
  if "frac-max" in o and o.frac-max < 1 { return (err: "frac-max must be ≥ 1") }
  if "solutions" in o and o.solutions < 1 { return (err: "solutions must be ≥ 1") }
  if "sci" in o and o.sci != none and (o.sci.len() != 2 or o.sci.any(x => type(x) != int)) { return (err: "sci must be (low, high) with integers, or none") }
  if "prefixes" in o and type(o.prefixes) == str and o.prefixes != "all" { return (err: "prefixes: \"all\", none or an array like (\"m\", \"k\")") }
  if "prefixes" in o and type(o.prefixes) == array and o.prefixes.any(p => type(p) != str) { return (err: "prefixes: the array must hold strings like \"m\"") }
  o
}
// named option arguments -> the ones actually given (auto = keep the current setting)
#let _given(o) = o.pairs().filter(((k, v)) => v != auto).to-dict()
#let _cf = state("solveng-cf", _DEF)

// ---------- units: dims = (m, kg, s, A, K) ----------
#let _Z = (0, 0, 0, 0, 0)
// errors are values (typst can't catch panics): (err: "message"), passed up and shown in the document
#let _err(m) = (err: m)
#let _E(x) = type(x) == dictionary and "err" in x
#let _first-err(xs) = xs.find(_E)
// state keys that are not variables: last outputs, outer scope, definition order, functions
#let _SP = ("..", "^", "^0", "!", "#fn")
#let _vars-of(d) = d.pairs().filter(p => p.at(0) not in _SP).to-dict()
#let _ok(v) = not float.is-nan(float(v)) and calc.abs(v) < 1e300
#let _add(a, b, k: 1) = a.zip(b).map(((x, y)) => calc.round(x + k * y, digits: 6))
#let _U = {
  let (V, C, J) = ((2, 1, -3, -1, 0), (0, 0, 1, 1, 0), (2, 1, -2, 0, 0))
  let O = (2, 1, -3, -2, 0)
  (
    m: (1, (1, 0, 0, 0, 0)), g: (1e-3, (0, 1, 0, 0, 0)), s: (1, (0, 0, 1, 0, 0)),
    A: (1, (0, 0, 0, 1, 0)), K: (1, (0, 0, 0, 0, 1)), V: (1, V), C: (1, C), J: (1, J),
    "Ω": (1, O), "\u{2126}": (1, O), Ohm: (1, O), ohm: (1, O), Omega: (1, O), W: (1, (2, 1, -3, 0, 0)),
    S: (1, O.map(x => -x)), F: (1, (-2, -1, 4, 2, 0)), H: (1, (2, 1, -2, -2, 0)),
    Hz: (1, (0, 0, -1, 0, 0)), N: (1, (1, 1, -2, 0, 0)), Pa: (1, (-1, 1, -2, 0, 0)),
    T: (1, (0, 1, -2, -1, 0)), Wb: (1, (2, 1, -2, -1, 0)), min: (60, (0, 0, 1, 0, 0)),
    h: (3600, (0, 0, 1, 0, 0)), Ah: (3600, C), As: (1, C), Wh: (3600, J), Ws: (1, J),
    eV: (1.602176634e-19, J), l: (1e-3, (3, 0, 0, 0, 0)), L: (1e-3, (3, 0, 0, 0, 0)),
    "%": (0.01, _Z), "‰": (1e-3, _Z), "°C": (1, (0, 0, 0, 0, 1)), degC: (1, (0, 0, 0, 0, 1)), "°": (calc.pi / 180, _Z), deg: (calc.pi / 180, _Z), rad: (1, _Z),
  )
}
#let _P = (a: 1e-18, f: 1e-15, p: 1e-12, n: 1e-9, "µ": 1e-6, "μ": 1e-6, u: 1e-6, m: 1e-3, c: 1e-2, d: 0.1, k: 1e3, M: 1e6, G: 1e9, T: 1e12)
// math content -> plain string: $"1k"Omega$ → "1kΩ", $frac(1, 2)"A"$ → "(1)/(2)A"
#let _str(c) = {
  if type(c) == str { return c }
  if type(c) in (int, float, symbol) { return str(c) }
  if type(c) != content { return _err("expected a string or math content, got " + repr(c)) }
  let f = c.func()
  let parts(..xs) = { let xs = xs.pos(); let b = _first-err(xs); if b != none { b } else { xs.join() } }
  if f == text or (c.has("text") and type(c.text) == str) { c.text }
  else if f == [ ].func() or f == h { " " }
  else if c.has("children") { parts(..c.children.map(_str)) }
  else if f == math.frac { parts("(", _str(c.num), ")/(", _str(c.denom), ")") }
  else if f == math.attach {
    parts(_str(c.base), if c.has("b") { parts("_", _str(c.b)) } else { "" }, if c.has("t") { parts("^(", _str(c.t), ")") } else { "" })
  }
  else if f == math.op { _str(c.text) }
  else if f == math.root {
    if c.has("index") and c.index != none { parts("(", _str(c.radicand), ")^(1/(", _str(c.index), "))") } else { parts("sqrt(", _str(c.radicand), ")") }
  }
  else if c.has("body") { _str(c.body) }
  else { _err("can't read " + repr(c)) }
}
// "k Ω" (prefix written apart, as in $1 "k" Omega$) -> "kΩ"
#let _join-prefix(ts) = {
  let out = ()
  for t in ts {
    if out.len() > 0 and out.last() in _P and out.last() not in _U and (t in _U or t.clusters().len() > 1) { out.last() += t } else { out.push(t) }
  }
  out
}
#let _tok(u) = u.replace(regex("[()]"), "").split("/").map(p => _join-prefix(p.split(regex("[\s*·⋅]+")).filter(t => t != "")).map(t => {
  let m = t.match(regex("^(.+?)(?:\^?([-−]?\d+)|([²³]))?$")).captures
  (m.at(0), if m.at(2) != none { int(m.at(2).replace("²", "2").replace("³", "3")) } else if m.at(1) != none { int(m.at(1).replace("−", "-")) } else { 1 })
}))
// unit string -> (factor, dims)
#let _unit(u) = {
  let (f, d) = (1.0, _Z)
  for (i, p) in _tok(u).enumerate() {
    for (n, e) in p.filter(t => t.at(0) != "1") {
      let c = n.clusters()
      let (uf, ud) = if n in _U { _U.at(n) } else if c.len() > 1 and c.first() in _P and c.slice(1).join() in _U {
        let (g, dd) = _U.at(c.slice(1).join()); (g * _P.at(c.first()), dd)
      } else { return _err("unknown unit '" + n + "'") }
      if i > 0 { e = -e }
      f *= calc.pow(float(uf), e); d = _add(d, ud, k: e)
    }
  }
  (f, d)
}
// typeable spellings shown with their symbols: kOhm → kΩ, uA → µA, degC → °C
#let _ASCII = (Ohm: "Ω", ohm: "Ω", Omega: "Ω", degC: "°C", deg: "°")
#let _uname(n) = if n in _U { _ASCII.at(n, default: n) } else {
  let c = n.clusters()
  let r = c.slice(1).join()
  if c.len() > 1 and c.first() in _P and r in _U { (if c.first() == "u" { "µ" } else { c.first() }) + _ASCII.at(r, default: r) } else { n }
}
#let _ushow(u) = _tok(u).map(p => p.map(((n, e)) => { let n = math.upright(_uname(n)); if e == 1 { n } else { math.attach(n, tr: str(e)) } }).join(h(0.17em))).join("/")
// pick a readable unit for dims d
#let _AU = ("V", "A", "Ω", "W", "J", "C", "F", "H", "S", "N", "Pa", "T", "Wb", "s", "m", "g", "K", "Hz")
#let _auto(d) = {
  if d == _Z { return "" }
  for u in _AU { if _U.at(u).at(1) == d { return u } }
  // m^2 before "m m"
  for b in ("m", "s") { for k in (2, 3, -1, -2, -3) { if _U.at(b).at(1).map(x => x * k) == d { return b + "^" + str(k) } } }
  for u in _AU { for b in ("m", "s", "A", "K") { for (k, j) in ((1, " "), (-1, "/"), (-2, "/")) {
    if _add(_U.at(u).at(1), _U.at(b).at(1), k: k) == d { return u + j + b + if k == -2 { "^2" } else { "" } }
  } } }
  // V², Ω⁻¹ before base units
  for u in _AU { for k in (2, 3, -1, -2) { if _U.at(u).at(1).map(x => x * k) == d { return u + "^" + str(k) } } }
  ("m", "kg", "s", "A", "K").zip(d).filter(p => p.at(1) != 0).map(((n, e)) => if e == 1 { n } else { n + "^" + str(e) }).join(" ")
}
#let _PX = (a: -18, f: -15, p: -12, n: -9, "µ": -6, m: -3, c: -2, d: -1, "": 0, k: 3, M: 6, G: 9, T: 12)
// unit u with the largest allowed prefix that keeps the number ≥ 1
#let _pre(v, u, cf) = {
  let f = u.split(regex("[ /]")).first()
  // areas and volumes: mm², m², km²
  let pw = f.match(regex("^(\p{L}+)\^(\d+)$"))
  if pw != none and v != 0 and cf.prefixes != none and _ok(v) {
    let n = int(pw.captures.at(1))
    let ps = ("m", "", "k").filter(p => p == "" or cf.prefixes == "all" or p in cf.prefixes)
    return ps.filter(p => _PX.at(p) * n <= calc.log(calc.abs(v)) + 1).at(-1, default: ps.first()) + u
  }
  if u == "" or f.contains("^") or v == 0 or cf.prefixes == none or not _ok(v) { return u }
  // times of a minute and more in min / h, not ks
  if u == "s" and cf.prefixes == "all" and calc.abs(v) >= 60 { return if calc.abs(v) < 3600 { "min" } else { "h" } }
  let ps = if cf.prefixes == "all" { ("a", "f", "p", "n", "µ", "m", "k", "M", "G", "T") } else { cf.prefixes }
  let ps = (ps.map(p => if p == "u" { "µ" } else { p }) + ("",)).filter(p => p in _PX).dedup().sorted(key: p => _PX.at(p))
  let e = calc.log(calc.abs(v))
  ps.filter(p => _PX.at(p) <= e + 1e-9).at(-1, default: ps.first()) + u
}

// ---------- numbers ----------
#let _sep(cf) = if cf.sep != "lang" { cf.sep } else if text.lang in ("de", "fr", "it", "es", "pt", "nl", "ru", "pl", "cs", "sk", "sl", "hr", "hu", "sv", "da", "nb", "fi", "tr", "ro", "uk") { "," } else { "." }
// |v| with d decimals as a plain digit string (no float→str surprises like 1e-05)
#let _fixed(v, d, zeros) = {
  let i = calc.round(calc.abs(v) * calc.pow(10.0, d))
  let s = str(int(i))
  if d > 0 {
    s = "0" * calc.max(0, d + 1 - s.len()) + s
    s = s.slice(0, s.len() - d) + "." + s.slice(s.len() - d)
    if not zeros { s = s.trim("0", at: end).trim(".", at: end) }
  }
  if v < 0 and i != 0 { "-" + s } else { s }
}
// -> (mantissa str, exponent | none), rounded to cf.digits significant digits
#let _fmt(x, cf) = {
  if x == 0 or not _ok(x) { return (if x == 0 { "0" } else { repr(x) }, none) }
  let n = cf.digits
  let e = calc.floor(calc.log(calc.abs(x)))
  if calc.abs(calc.round(x / calc.pow(10.0, e), digits: n - 1)) >= 10 { e += 1 } // 9.9996 → 10.00
  let (lo, hi) = if cf.sci == none { (-calc.inf, calc.inf) } else { cf.sci }
  let sci = e < lo or e >= hi or calc.abs(x) >= 1e15 or calc.abs(x) < 1e-15
  let p = if not sci { 0 } else if cf.eng { calc.floor(e / 3) * 3 } else { e }
  let d = n - 1 - e + p
  let y = x / calc.pow(10.0, p)
  if d < 0 { y = calc.round(y / calc.pow(10.0, -d)) * calc.pow(10.0, -d) } // 123456 → 123500 at 4 digits
  (_fixed(y, calc.max(d, 0), cf.zeros), if sci { p })
}
#let _grp(s, g) = if g == none or s.len() < 5 { s } else {
  let c = s.clusters().rev()
  range(c.len()).map(i => if i > 0 and calc.rem(i, 3) == 0 { c.at(i) + g } else { c.at(i) }).rev().join()
}
// mantissa string + exponent -> content
#let _nc(m, e, cf) = {
  let neg = m.starts-with("-") or m.starts-with("−")
  let m = m.trim(regex("[-−+]"), at: start)
  let (ip, fp) = if m.contains(".") { m.split(".") } else { (m, none) }
  let m = (if neg { "−" } else { "" }) + _grp(ip, if type(cf.group) == symbol { str(cf.group) } else { cf.group }) + if fp != none { _sep(cf) + fp } else { "" }
  let t = if cf.times == "times" { sym.times } else { sym.dot.op }
  if e == none or e == 0 { m } else if m == "1" { $10^#str(e).replace("-", "−")$ } else { $#m #t 10^#str(e).replace("-", "−")$ }
}
// best fraction p/q (q ≤ cf.frac-max) that equals x, or none
#let _ratio(x, mx) = {
  if not _ok(x) or calc.abs(x) >= 1e15 { return none }
  if calc.round(x) == x { return (int(x), 1) }
  let (h1, h2, k1, k2) = (1.0, 0.0, 0.0, 1.0)
  let y = calc.abs(x)
  for _ in range(40) {
    let a = calc.floor(y)
    (h1, h2) = (a * h1 + h2, h1)
    (k1, k2) = (a * k1 + k2, k1)
    if k1 > mx or h1 > 1e15 { return none }
    if calc.abs(h1 / k1 - calc.abs(x)) <= 1e-12 * calc.abs(x) { return (int(if x < 0 { -h1 } else { h1 }), int(k1)) }
    let f = y - a
    if f == 0 { break }
    y = 1 / f
  }
  none
}
#let _fc((p, q), cf) = {
  if q == 1 { return _nc(str(p), none, cf) }
  let (a, b) = (_nc(str(calc.abs(p)), none, cf), _nc(str(q), none, cf))
  let f = if cf.slash { $#a\/#b$ } else { math.frac(a, b) }
  if p < 0 { $−#f$ } else { f }
}
// how a number is shown -> (content, exact)
#let _nshow(q, cf) = {
  let src = q.at("src", default: none)
  if cf.form == "input" and src != none {
    let s = src.replace(regex("[*·⋅]"), " dot ").replace(".", _sep(cf))
    return (eval(s, mode: "math"), true)
  }
  if cf.form == "fraction" { let r = _ratio(q.n, cf.frac-max); if r != none { return (_fc(r, cf), true) } }
  if q.raw != none and cf.form != "decimal" {
    let m = q.raw.match(regex("^([^eE]+)(?:[eE]([-+−]?\d+))?$")).captures
    return (_nc(m.at(0), if m.at(1) != none { int(m.at(1).replace("−", "-").replace("+", "")) }, cf), true)
  }
  let (m, e) = _fmt(q.n, cf)
  (_nc(m, e, cf), not _ok(q.n) or calc.abs(float(m) * calc.pow(10.0, if e == none { 0 } else { e }) - q.n) <= 1e-9 * calc.abs(q.n))
}
// quantity -> content
#let _qc(q, cf) = {
  let n = _nshow(q, cf).at(0)
  if q.u == "" { n } else { $#n thin #_ushow(q.u)$ }
}
#let _q0(x, num) = {
  if type(x) in (int, float) { return (v: float(x), d: _Z, raw: str(x), n: float(x), u: "", ex: none) }
  let x = _str(x)
  if _E(x) { return x }
  let x = x.trim()
  let N = "[-+−]?[\\d.,()]+(?:[eE][-+−]?\\d+)?"
  let m = x.match(regex("^(" + N + "(?:\\s*[-+−*/·⋅^]\\s*" + N + ")*)\\s*(.*)$"))
  if m == none { return _err("cannot read the value '" + x + "' (a number with a unit, e.g. 7.2 kOhm)") }
  let (s, u) = m.captures
  let s = s.trim().replace(",", ".")
  if s.match(regex("\\d*\\.\\d*\\.")) != none { return _err("cannot read the number '" + s + "'") }
  let un = _unit(u)
  if _E(un) { return un }
  let (f, d) = un
  if s.match(regex("^[-+−]?[\\d.]+(?:[eE][-+−]?\\d+)?$")) != none {
    let n = float(s.replace("−", "-"))
    if not _ok(n) { return _err("'" + x + "' is out of range (|x| < 1e300)") }
    return (v: n * f, d: d, raw: s, n: n, u: u, ex: none)
  }
  // arithmetic like "3450/2 mAh": evaluate, show the result
  let n = num(s.replace("−", "-").replace(regex("[·⋅]"), "*"))
  if _E(n) { return n }
  (v: n * f, d: d, raw: none, n: n, src: s, u: u, ex: none)
}

// ---------- evaluate with units -> (value, dims) ----------
#let _ev(e, d) = {
  let (k, ..a) = e
  if k == "n" { return (a.first(), _Z) }
  if k == "q" { return (a.first().v, a.first().d) }
  if k == "c" { return (if a.first() == "e" { calc.e } else { calc.pi }, _Z) }
  if k == "v" { let q = d.at(a.first(), default: none); return if q == none { _err("unknown variable '" + a.first() + "'") } else { (q.v, q.d) } }
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
#let _op = ("+", "-", "−", "*", "·", "⋅", "×", "/", "^", ")", "=", ",", none)
#let _subst(e, m) = if e.at(0) == "v" and e.at(1) in m { m.at(e.at(1)) } else { (e.at(0), ..e.slice(1).map(a => if type(a) == array { _subst(a, m) } else { a })) }
#let _tk(s) = s.matches(regex("\\[[^\\]]*\\]|\\d+\\.?\\d*(?:[eE][-+]?\\d+)?|\\.\\d+|@?\\p{L}[\\p{L}\\p{N}_]*|\\S")).map(m => m.text)
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
      let q = _q0(k.slice(1, -1), num)
      if _E(q) { q } else { (("q", q), i + 1) }
    }
    else if k.match(regex("^[\d.]")) != none { (("n", float(k)), i + 1) }
    else if k.starts-with("@") {
      let n = k.slice(1)
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
      else { (if d.keep { ("f", k, if args.len() == 1 { args.first() } else { ("tuple", ..args) }) } else { _subst(fn.body, fn.params.zip(args).to-dict()) }, j + 1) }
    } else if k in _F and at(i + 1) == "(" { let r = _p(t, i + 1, 4, d); if _E(r) { r } else { (("f", k, r.at(0)), r.at(1)) } }
    else if (k in ("pi", "π") or k == "e") and k not in d.v { (("c", k), i + 1) }
    else if k.match(regex("^\p{L}")) != none { (("v", k), i + 1) }
    else { _err("unexpected '" + k + "'") }
    if _E(r) { return r }
    let (b, i) = r
    if l == 2 and at(i) == "^" { let r = _p(t, i + 1, 2, d); if _E(r) { r } else { (("^", b, r.at(0)), r.at(1)) } } else { (b, i) }
  }
}
#let _parse(s, d, f: (:), keep: false) = {
  let t = _tk(s)
  if t.len() == 0 { return _err("empty expression") }
  let r = _p(t, 0, 0, (v: d, f: f, keep: keep))
  if _E(r) { return _err(r.err + " in '" + s.trim() + "'") }
  let (a, i) = r
  if i < t.len() { return _err("unexpected '" + t.at(i) + "' in '" + s.trim() + "'") }
  a
}
#let _isu(u) = type(u) == str and _tok(u).flatten().chunks(2).all(((n, _)) => n == "1" or n in _U or { let c = n.clusters(); c.len() > 1 and c.first() in _P and c.slice(1).join() in _U })
#let _isq(x) = {
  if type(x) in (int, float) { return true }
  let N = "[-+−]?[\\d.,()]+(?:[eE][-+−]?\\d+)?"
  let x = _str(x)
  if _E(x) { return false }
  let m = x.trim().match(regex("^(" + N + "(?:\\s*[-+−*/·⋅^]\\s*" + N + ")*)\\s*(.*)$"))
  m != none and m.captures.at(0).match(regex("^[-+−(]*[\\d.]")) != none and _isu(m.captures.at(1))
}
#let _q(x) = _q0(x, s => { let e = _parse(s, (:)); if _E(e) { return e }; let v = _ev(e, (:)); if _E(v) { v } else { v.at(0) } })
#let _vars(e) = if e.at(0) == "v" { (e.at(1),) } else { e.slice(1).filter(a => type(a) == array).map(_vars).flatten() }
#let _has(e, x) = x in _vars(e)

// ---------- simplify (just enough for nice output) ----------
#let _s(e) = {
  let (k, ..a) = e
  if k in ("n", "c", "v", "q") { return e }
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

// ---------- numeric evaluation with derivatives (dual numbers) ----------
// env: name -> (value, grad | none); grad none means zero. Returns (value, grad) or none on a domain error.
#let _lc(p, g, q: 0, h: none) = if g == none and h == none { none } else if h == none { g.map(x => p * x) } else if g == none { h.map(y => q * y) } else { g.zip(h).map(((x, y)) => p * x + q * y) }
#let _nd(e, env) = {
  let (k, ..a) = e
  let r = if k == "n" { (float(a.first()), none) }
  else if k == "q" { (float(a.first().v), none) }
  else if k == "c" { (if a.first() == "e" { calc.e } else { calc.pi }, none) }
  else if k == "v" {
    let x = env.at(a.first(), default: none)
    if x == none { return none }
    x
  } else {
    let xs = ()
    for t in a.filter(t => type(t) == array) { let y = _nd(t, env); if y == none { return none }; xs.push(y) }
    if k == "-" { let (v, g) = xs.first(); (-v, _lc(-1, g)) }
    else if k == "+" { xs.reduce(((v, g), (w, h)) => (v + w, _lc(1, g, q: 1, h: h))) }
    else if k == "*" { xs.reduce(((v, g), (w, h)) => (v * w, _lc(w, g, q: v, h: h))) }
    else if k == "/" {
      let ((v, g), (w, h)) = xs
      if w == 0 or w * w == 0 { return none }
      (v / w, _lc(1 / w, g, q: -v / (w * w), h: h))
    } else if k == "^" {
      let ((b, g), (x, h)) = xs
      if h == none {
        let xi = calc.abs(x) < 1e9 and calc.round(x) == x
        if b == 0 {
          if x < 0 or (x < 1 and g != none) { return none }
          (if x == 0 { 1.0 } else { 0.0 }, _lc(if x == 1 { 1 } else { 0 }, g))
        } else {
          if b < 0 and not xi { return none }
          if x * calc.ln(calc.abs(b)) > 690 { return none }
          let v = if xi { calc.pow(b, int(x)) } else { calc.pow(b, x) }
          (v, _lc(x * v / b, g))
        }
      } else {
        if b <= 0 or x * calc.ln(b) > 690 { return none }
        let v = calc.exp(x * calc.ln(b))
        (v, _lc(v * x / b, g, q: v * calc.ln(b), h: h))
      }
    } else {
      let f = a.first()
      let (v, g) = xs.first()
      if f == "sqrt" {
        if v < 0 or (v == 0 and g != none) { return none }
        let s = calc.sqrt(v); (s, if v == 0 { none } else { _lc(0.5 / s, g) })
      } else if f == "exp" { if v > 690 { return none }; let y = calc.exp(v); (y, _lc(y, g)) }
      else if f == "ln" { if v <= 0 { return none }; (calc.ln(v), _lc(1 / v, g)) }
      else if f == "log" { if v <= 0 { return none }; (calc.log(v), _lc(1 / (v * calc.ln(10)), g)) }
      else if f == "sin" { (calc.sin(v), _lc(calc.cos(v), g)) }
      else if f == "cos" { (calc.cos(v), _lc(-calc.sin(v), g)) }
      else if f == "tan" { let c = calc.cos(v); if c == 0 { return none }; (calc.tan(v), _lc(1 / (c * c), g)) }
      else if f in ("asin", "acos") {
        if calc.abs(v) > 1 or (calc.abs(v) == 1 and g != none) { return none }
        let y = if f == "asin" { calc.asin(v).rad() } else { calc.acos(v).rad() }
        (y, if calc.abs(v) == 1 { none } else { _lc((if f == "asin" { 1 } else { -1 }) / calc.sqrt(1 - v * v), g) })
      } else if f == "atan" { (calc.atan(v).rad(), _lc(1 / (1 + v * v), g)) }
      else { (calc.abs(v), _lc(if v < 0 { -1 } else { 1 }, g)) }
    }
  }
  if not _ok(r.at(0)) or (r.at(1) != none and r.at(1).any(z => not _ok(z))) { none } else { r }
}
#let _env(d) = d.pairs().filter(p => p.at(0) not in _SP).map(((k, q)) => (k, (float(q.v), none))).to-dict()

// ---------- infer units of the unknowns ----------
// a dimension is (c, a): c + Σ a_j·dim(x_j); none = literal 0 (fits any unit)
#let _dx(e, d, env, xs) = {
  let n = xs.len()
  let K(c) = (c, range(n).map(_ => 0.0))
  let (k, ..a) = e
  if k == "n" { return (if a.first() == 0 { none } else { K(_Z) }, ()) }
  if k == "c" { return (K(_Z), ()) }
  if k == "q" { return (K(a.first().d), ()) }
  if k == "v" {
    let i = xs.position(y => y == a.first())
    if i != none { return ((_Z, range(n).map(j => if j == i { 1.0 } else { 0.0 })), ()) }
    let q = d.at(a.first(), default: none)
    if q == none { return _err("unknown variable '" + a.first() + "'") }
    return (K(q.d), ())
  }
  let sub = a.filter(t => type(t) == array).map(t => _dx(t, d, env, xs))
  let b = _first-err(sub)
  if b != none { return b }
  let cons = sub.map(s => s.at(1)).sum(default: ())
  let ds = sub.map(s => s.at(0))
  let A(x, y, s: 1) = (x.at(0).zip(y.at(0)).map(((p, q)) => p + s * q), x.at(1).zip(y.at(1)).map(((p, q)) => p + s * q))
  let S(x, f) = (x.at(0).map(z => z * f), x.at(1).map(z => z * f))
  let eq(x, y) = if x == none or y == none { () } else { (A(x, y, s: -1),) }
  if k == "-" { (ds.first(), cons) }
  else if k == "+" {
    let nn = ds.filter(x => x != none)
    if nn.len() == 0 { (none, cons) } else { (nn.first(), cons + nn.slice(1).map(x => A(x, nn.first(), s: -1))) }
  }
  else if k == "*" { (if none in ds { none } else { ds.reduce(A) }, cons) }
  else if k == "/" { let (x, y) = ds; (if x == none { none } else if y == none { K(_Z) } else { A(x, y, s: -1) }, cons) }
  else if k == "^" {
    let (b, x) = ds
    if xs.any(v => _has(a.at(1), v)) { (K(_Z), cons + eq(b, K(_Z)) + eq(x, K(_Z))) }
    else {
      let p = _nd(a.at(1), env)
      if p == none { return _err("cannot evaluate an exponent") }
      (if b == none { none } else { S(b, p.at(0)) }, cons + eq(x, K(_Z)))
    }
  } else {
    let (f, _) = a
    let b = ds.first()
    if f == "sqrt" { (if b == none { none } else { S(b, 0.5) }, cons) }
    else if f == "abs" { (b, cons) }
    else { (K(_Z), cons + eq(b, K(_Z))) }
  }
}
// -> (dims per unknown, indices whose unit is not determined)
#let _dims(eqs, d, env, xs) = {
  let n = xs.len()
  let cons = ()
  for (L, R) in eqs {
    let l1 = _dx(L, d, env, xs)
    let r1 = _dx(R, d, env, xs)
    if _E(l1) { return l1 }
    if _E(r1) { return r1 }
    let (l, c1) = l1
    let (r, c2) = r1
    cons += c1 + c2
    if l != none and r != none { cons.push((l.at(0).zip(r.at(0)).map(((p, q)) => p - q), l.at(1).zip(r.at(1)).map(((p, q)) => p - q))) }
  }
  let rows = cons.map(((c, a)) => a + c.map(z => -z))
  let piv = ()
  let r = 0
  for col in range(n) {
    let p = none
    for i in range(r, rows.len()) {
      if calc.abs(rows.at(i).at(col)) > 1e-9 and (p == none or calc.abs(rows.at(i).at(col)) > calc.abs(rows.at(p).at(col))) { p = i }
    }
    if p == none { continue }
    let t = rows.at(p); rows.at(p) = rows.at(r); rows.at(r) = t.map(z => z / t.at(col))
    for i in range(rows.len()) {
      if i != r and rows.at(i).at(col) != 0 {
        let f = rows.at(i).at(col)
        rows.at(i) = rows.at(i).zip(rows.at(r)).map(((x, y)) => x - f * y)
      }
    }
    piv.push((col, r)); r += 1
  }
  for i in range(r, rows.len()) {
    if rows.at(i).slice(n).any(z => calc.abs(z) > 1e-6) { return _err("the units in these equations don't match") }
  }
  let D = range(n).map(_ => _Z)
  for (col, i) in piv { D.at(col) = rows.at(i).slice(n).map(z => calc.round(z, digits: 6)) }
  (D, range(n).filter(j => j not in piv.map(p => p.at(0))))
}

// ---------- polynomials ----------
#let _pa(..ts) = { let ts = ts.pos().filter(t => t != ("n", 0)); if ts.len() == 0 { ("n", 0) } else if ts.len() == 1 { ts.first() } else { ("+", ..ts) } }
#let _pm(a, b) = if a == ("n", 0) or b == ("n", 0) { ("n", 0) } else if a == ("n", 1) { b } else if b == ("n", 1) { a } else { ("*", a, b) }
#let _pmul(p, q) = range(p.len() + q.len() - 1).map(i => _pa(..range(calc.max(0, i - q.len() + 1), calc.min(i, p.len() - 1) + 1).map(j => _pm(p.at(j), q.at(i - j)))))
// expression -> coefficient expressions (ascending powers of x), or none if not a polynomial in x
#let _poly(e, x) = {
  if not _has(e, x) { return (e,) }
  let (k, ..a) = e
  if k == "v" { return (("n", 0), ("n", 1)) }
  if k == "-" { let p = _poly(a.first(), x); return if p == none { none } else { p.map(c => if c == ("n", 0) { c } else { ("-", c) }) } }
  if k == "+" or k == "*" {
    let ps = a.map(t => _poly(t, x))
    if none in ps { return none }
    if k == "*" { return ps.reduce(_pmul) }
    let m = calc.max(..ps.map(p => p.len()))
    return range(m).map(i => _pa(..ps.map(p => p.at(i, default: ("n", 0)))))
  }
  if k == "/" {
    if _has(a.at(1), x) { return none }
    let p = _poly(a.at(0), x)
    return if p == none { none } else { p.map(c => if c == ("n", 0) { c } else { ("/", c, a.at(1)) }) }
  }
  if k == "^" {
    let y = a.at(1)
    if y.at(0) != "n" or y.at(1) < 0 or y.at(1) > 20 or calc.round(y.at(1)) != y.at(1) { return none }
    let p = _poly(a.at(0), x)
    if p == none { return none }
    return range(int(y.at(1))).fold((("n", 1),), (r, _) => _pmul(r, p))
  }
  none
}
// complex helpers
#let _cm(a, b) = (a.at(0) * b.at(0) - a.at(1) * b.at(1), a.at(0) * b.at(1) + a.at(1) * b.at(0))
#let _cs(a, b) = (a.at(0) - b.at(0), a.at(1) - b.at(1))
#let _cabs(a) = calc.sqrt(a.at(0) * a.at(0) + a.at(1) * a.at(1))
#let _cd(a, b) = { let n = b.at(0) * b.at(0) + b.at(1) * b.at(1); ((a.at(0) * b.at(0) + a.at(1) * b.at(1)) / n, (a.at(1) * b.at(0) - a.at(0) * b.at(1)) / n) }
#let _uniq(rs) = {
  let out = ()
  for r in rs.sorted() { if out.len() == 0 or calc.abs(r - out.last()) > 1e-7 * calc.max(calc.abs(r), calc.abs(out.last())) { out.push(r) } }
  out
}
// real roots of Σ c_i x^i (float coefficients, highest ≠ 0), ascending
#let _roots(c) = {
  let z = 0
  while c.at(z) == 0 { z += 1 }
  let c = c.slice(z)
  let n = c.len() - 1
  let rs = if z > 0 { (0.0,) } else { () }
  if n == 1 { rs.push(-c.at(0) / c.at(1)) }
  else if n == 2 {
    let (cc, b, a) = c
    let D = b * b - 4 * a * cc
    if D < 0 and -D <= 1e-12 * b * b { D = 0 }
    if D >= 0 {
      let q = -0.5 * (b + (if b >= 0 { 1 } else { -1 }) * calc.sqrt(D))
      rs += (q / a, cc / q)
    }
  } else if n > 2 {
    // Durand–Kerner on the rescaled monic polynomial (roots of size ~1)
    let R = calc.pow(calc.abs(c.at(0) / c.at(n)), 1 / n)
    let m = range(n + 1).map(i => c.at(i) / c.at(n) * calc.pow(R, i - n))
    let zs = range(n).map(k => { let t = 2 * calc.pi * k / n + 0.4; (calc.cos(t), calc.sin(t)) })
    for it in range(800) {
      let mx = 0
      for k in range(n) {
        let zk = zs.at(k)
        let p = (1.0, 0.0)
        for i in range(n - 1, -1, step: -1) { p = _cm(p, zk); p.at(0) += m.at(i) }
        let den = (1.0, 0.0)
        for j in range(n) { if j != k { den = _cm(den, _cs(zk, zs.at(j))) } }
        if _cabs(den) == 0 { den = (1e-12, 0.0) }
        let dz = _cd(p, den)
        zs.at(k) = _cs(zk, dz)
        mx = calc.max(mx, _cabs(dz))
      }
      if mx < 1e-15 { break }
    }
    let P(y) = range(n + 1).rev().fold((0.0, 0.0), ((v, g), i) => (v * y + m.at(i), g * y + v))
    for (re, im) in zs {
      if calc.abs(im) > 1e-6 * calc.max(_cabs((re, im)), 1e-9) { continue }
      let y = re
      for _ in range(5) { let (v, g) = P(y); if g == 0 { break }; let y2 = y - v / g; if calc.abs(P(y2).at(0)) < calc.abs(v) { y = y2 } else { break } }
      rs.push(y * R)
    }
  }
  _uniq(rs)
}

// ---------- numeric root finding ----------
#let _resid(eqs, env, xs, xv) = { // -> (F, J, S) or none
  let n = xs.len()
  let env = env
  for (j, x) in xs.enumerate() { env.insert(x, (float(xv.at(j)), range(n).map(i => if i == j { 1.0 } else { 0.0 }))) }
  let (F, J, S) = ((), (), ())
  for (L, R) in eqs {
    let l = _nd(L, env)
    let r = _nd(R, env)
    if l == none or r == none { return none }
    let z = range(n).map(_ => 0.0)
    F.push(l.at(0) - r.at(0))
    J.push(_lc(1, if l.at(1) == none { z } else { l.at(1) }, q: -1, h: if r.at(1) == none { z } else { r.at(1) }))
    S.push(calc.abs(l.at(0)) + calc.abs(r.at(0)))
  }
  (F, J, S)
}
#let _conv(F, S) = F.zip(S).all(((f, s)) => calc.abs(f) <= 1e-8 * s)
#let _gauss(A, b) = {
  let n = b.len()
  let M = A.zip(b).map(((r, y)) => r + (y,))
  for c in range(n) {
    let p = range(c, n).sorted(key: r => -calc.abs(M.at(r).at(c))).first()
    if M.at(p).at(c) == 0 { return none }
    let t = M.at(p); M.at(p) = M.at(c); M.at(c) = t
    for r in range(n) {
      if r != c and M.at(r).at(c) != 0 {
        let f = M.at(r).at(c) / M.at(c).at(c)
        M.at(r) = M.at(r).zip(M.at(c)).map(((x, y)) => x - f * y)
      }
    }
  }
  let x = range(n).map(i => M.at(i).at(n) / M.at(i).at(i))
  if x.all(_ok) { x } else { none }
}
// damped Newton from x0 -> solution or none
#let _newton(eqs, env, xs, x0) = {
  let x = x0
  let r = _resid(eqs, env, xs, x)
  if r == none { return none }
  let w = r.at(2).map(s => if s > 0 { 1 / s } else { 1 })
  let N(F) = F.zip(w).map(((f, w)) => calc.pow(f * w, 2)).sum()
  for it in range(40) {
    let (F, J, S) = r
    if N(F) == 0 { break }
    let dx = _gauss(J, F.map(f => -f))
    if dx == none { break }
    let (t, acc) = (1.0, none)
    for _ in range(12) {
      let xn = x.zip(dx).map(((a, b)) => a + t * b)
      let rn = _resid(eqs, env, xs, xn)
      if rn != none and N(rn.at(0)) < N(F) { acc = (xn, rn); break }
      t /= 2
    }
    if acc == none { break }
    let done = x.zip(acc.at(0)).all(((a, b)) => calc.abs(a - b) <= 1e-14 * calc.max(calc.abs(a), calc.abs(b)))
    (x, r) = acc
    if done { break }
  }
  if _conv(r.at(0), r.at(2)) { x } else { none }
}
// all real solutions of one equation in x: scan ±10^-15…10^15, then bisect sign changes
#let _scan(L, R, env, x, s) = {
  let f(t) = { let r = _resid(((L, R),), env, (x,), (t,)); if r == none { none } else { (r.at(0).first(), r.at(2).first()) } }
  let ks = range(-60, 61).map(k => s * calc.pow(10.0, k / 4))
  let pts = ks.rev().map(t => -t) + (0.0,) + ks
  let vs = pts.map(f)
  let rs = ()
  for i in range(pts.len()) {
    let v = vs.at(i)
    if v == none { continue }
    if v.at(0) == 0 { rs.push(pts.at(i)); continue }
    let w = vs.at(i + 1, default: none)
    if w == none or w.at(0) == 0 or (v.at(0) > 0) == (w.at(0) > 0) { continue }
    let (a, b, fa) = (pts.at(i), pts.at(i + 1), v.at(0))
    let ok = true
    for _ in range(200) {
      let m = (a + b) / 2
      if m == a or m == b { break }
      let fm = f(m)
      if fm == none { ok = false; break }
      if fm.at(0) == 0 { (a, b) = (m, m); break }
      if (fm.at(0) > 0) == (fa > 0) { (a, fa) = (m, fm.at(0)) } else { b = m }
    }
    let m = (a + b) / 2
    let fm = f(m)
    if ok and fm != none and calc.abs(fm.at(0)) <= 1e-7 * fm.at(1) { rs.push(m) }
  }
  // roots without a sign change (touching zero, between poles): Newton from local minima of |f|
  for i in range(1, pts.len() - 1) {
    let (u, v, w) = (vs.at(i - 1), vs.at(i), vs.at(i + 1))
    if u == none or v == none or w == none or calc.abs(v.at(0)) > calc.abs(u.at(0)) or calc.abs(v.at(0)) > calc.abs(w.at(0)) { continue }
    let r = _newton(((L, R),), env, (x,), (pts.at(i),))
    if r != none { rs.push(r.first()) }
  }
  _uniq(rs)
}
// several solutions of a system: Newton from deterministic pseudo-random starts
#let _msolve(eqs, env, xs, sc, guess) = {
  let n = xs.len()
  let starts = if guess != none { (guess,) } else {
    let st = (sc,)
    let seed = 20261019
    for i in range(32) {
      let p = ()
      for j in range(n) {
        seed = calc.rem(seed * 1103515245 + 12345, 2147483648)
        let u = seed / 2147483648
        seed = calc.rem(seed * 1103515245 + 12345, 2147483648)
        let sg = if seed / 2147483648 < 0.3 { -1 } else { 1 }
        p.push(sg * sc.at(j) * calc.pow(10.0, 8 * u - 4))
      }
      st.push(p)
    }
    st
  }
  let (sols, idle) = ((), 0)
  for s0 in starts {
    idle += 1
    if sols.len() > 0 and idle > 10 { break }
    let x = _newton(eqs, env, xs, s0)
    if x == none { continue }
    if not sols.any(y => y.zip(x).all(((a, b)) => calc.abs(a - b) <= 1e-6 * calc.max(calc.abs(a), calc.abs(b)) + 1e-300)) { sols.push(x); idle = 0 }
  }
  sols.sorted()
}

// ---------- display ----------
#let _GR = "alpha|beta|gamma|Gamma|delta|Delta|epsilon|zeta|eta|theta|Theta|iota|kappa|lambda|Lambda|mu|nu|xi|Xi|pi|Pi|rho|sigma|Sigma|tau|upsilon|phi|Phi|chi|psi|Psi|omega|Omega"
#let _name(n) = {
  let g(b) = if b.match(regex("^(" + _GR + ")$")) != none or b.match(regex("^\\p{L}$")) != none { eval(b, mode: "math") } else if b.clusters().len() == 1 { math.italic(b) } else { math.upright(b) }
  if n.starts-with("Delta") and n.len() > 5 { return $Delta #_name(n.slice(5).trim("_", at: start))$ }
  let m = n.match(regex("^(" + _GR + "|\\p{L})(\\d+)$"))
  let (b, s) = if n.contains("_") { let p = n.split("_"); (p.first(), p.slice(1).join("_")) } else if m != none { m.captures } else { (n, "") }
  if s == "" { g(b) } else { math.attach(g(b), br: math.upright(s)) }
}
#let _dp(e, s, cf) = { // -> (content, precedence)
  let P(x, p) = { let (c, q) = _dp(x, s, cf); if q < p { $(#c)$ } else { c } }
  let (k, ..a) = e
  if k == "tuple" { return (a.map(t => _dp(t, s, cf).at(0)).join($, $), 0) }
  if k == "n" { (_nc(.._fmt(a.first(), cf + (digits: 6, zeros: false)), cf), if a.first() < 0 { 2 } else { 9 }) }
  else if k == "q" { (_qc(a.first(), cf), if a.first().u != "" { 3 } else { 9 }) }
  else if k == "c" { (if a.first() == "e" { $e$ } else { $pi$ }, 9) }
  else if k == "v" { if s != none and a.first() in s { s.at(a.first()) } else { (_name(a.first()), 9) } }
  else if k == "+" { (a.enumerate().map(((i, t)) => if t.at(0) == "-" { $- #P(t.at(1), 2)$ } else if i == 0 { P(t, 1) } else { $+ #P(t, 1)$ }).join(), 1) }
  else if k == "-" { ($-#P(a.first(), 3)$, 2) }
  else if k == "*" { (a.enumerate().map(((i, t)) => if i > 0 and (s != none or t.at(0) in ("n", "q") or a.at(i - 1).at(0) == "q") { $dot #P(t, 3)$ } else { P(t, 3) }).join(), 3) }
  else if k == "/" { (math.frac(_dp(a.at(0), s, cf).at(0), _dp(a.at(1), s, cf).at(0)), 4) }
  else if k == "^" { (math.attach(P(a.at(0), 9), tr: _dp(a.at(1), s, cf).at(0)), 5) }
  else {
    let (f, x) = a
    let c = _dp(x, s, cf).at(0)
    if f == "sqrt" { (math.sqrt(c), 9) } else if f == "abs" { (math.abs(c), 9) } else if f == "exp" { (math.attach($e$, tr: c), 5) }
    else if f in _F { ($#math.op(f) (#c)$, 9) } else { ($#_name(f) (#c)$, 9) }
  }
}

// HTML: one stylesheet per document (browsers set math fonts on <math> itself, so inline styles can't reach it)
#let _css-done = state("solveng-css", false)
#let _CSS = ".solveng, .solveng-list { font-size: 1.12em; }
.solveng { margin: 0.9em 0; overflow-x: auto; overflow-y: hidden; }
.solveng math, .solveng-list math { font-family: 'STIX Two Math', 'Cambria Math', 'Latin Modern Math', math; }
.solveng-list { border-collapse: collapse; margin: 0.7em 0; }
.solveng-defs { columns: var(--w) auto; column-gap: 2.5em; margin: 0.7em 0; font-size: 1.12em; }
.solveng-defs > div { break-inside: avoid; display: flex; gap: 0.3em; padding: 0.15em 0; white-space: nowrap; }
.solveng-defs > div > span:first-child { flex: 0 0 var(--n); text-align: right; }
.solveng-plot { margin: 0.9em 0; text-align: center; }
.solveng-plot svg { max-width: 100%; height: auto !important; margin: 0 auto; }
.solveng-defs math { font-family: 'STIX Two Math', 'Cambria Math', 'Latin Modern Math', math; }
.solveng-list td { padding: 0.25em 0.4em; white-space: nowrap; }
.solveng-list tr:nth-child(even) td { background: rgba(127, 127, 127, 0.07); }
.solveng-error { color: #b3261e; background: #fdecea; border-radius: 3px; padding: 0.1em 0.35em; font-size: 0.9em; }
div.solveng-error { border-left: 3px solid #b3261e; padding: 0.4em 0.7em; margin: 0.6em 0; }"
#let _css() = context if not _css-done.get() { html.elem("style", _CSS); _css-done.update(true) }
// math block or inline; in HTML a spaced, scrollable block
#let _eqn(body, cf) = {
  let e = math.equation(block: cf.block, body)
  if cf.block and target() == "html" { _css(); html.elem("div", attrs: (class: "solveng"), e) } else { e }
}
// problems are shown where they happen (solve-config(errors: "panic") stops instead)
#let _errc(msg, cf, wide: false) = {
  if cf.at("errors", default: "document") == "panic" { panic("solveng: " + msg) }
  if target() == "html" {
    _css()
    html.elem(if wide { "div" } else { "span" }, attrs: (class: "solveng-error"), "⚠ solveng: " + msg)
  } else {
    let t = text(fill: rgb("#b3261e"), size: 0.9em)[⚠ solveng: #msg]
    if wide { block(width: 100%, fill: rgb("#fdecea"), stroke: (left: 2pt + rgb("#b3261e")), inset: (x: 8pt, y: 6pt), radius: 2pt, t) }
    else { box(fill: rgb("#fdecea"), inset: (x: 3pt, y: 1pt), outset: (y: 2pt), radius: 2pt, t) }
  }
}
#let _err-now(msg) = context _errc(msg, _cf.get(), wide: true)
// output of the package: its equations are not read as definitions by solve-block's math rule
#let _busy = state("solveng-busy", 0)
#let _mine(body) = _busy.update(n => n + 1) + body + _busy.update(n => n - 1)
#let _tn(x) = if x == none { "none" } else if x == auto { "auto" } else { str(type(x)) }

// ---------- public API ----------

/// Sets options for number formatting, units and output.
///
/// Without a body the options apply from here on; with a body only inside it.
/// Every option defaults to `auto`, which keeps the current setting, so only
/// what you pass changes.
///
/// ```typ
/// #show: solve-config.with(digits: 3, form: "fraction")   // defaults for the whole document
/// #solve-config(layout: "column")                          // from here on
/// #solve-config(steps: "result")[ … ]                      // only inside
/// ```
///
/// Unknown options and values of the wrong type are shown as an error in the document.
///
/// - ..body (content): Optional: the content the options apply to.
/// - digits (auto, int): Significant digits of computed values, 1…15. Default `4`: `57,5 h`, `123500`.
/// - zeros (auto, bool): Keep trailing zeros up to `digits` (`2,500 V`). Default `false`.
/// - sep (auto, str): Decimal separator. `"lang"` (default): `,` for de, fr, it, … and `.` otherwise; or any string like `","`.
/// - group (auto, none, str, symbol, content): Digit grouping for integer parts with 5+ digits, e.g. `sym.space.thin` → `12 345`. Default `none`.
/// - sci (auto, none, array): `(low, high)`: plain notation while the exponent `e` is in `low ≤ e < high`, else `2,5 · 10⁻⁷`. Default `(-3, 6)`; `none` never switches.
/// - eng (auto, bool): Engineering notation, exponents in steps of 3. Default `false`.
/// - times (auto, str): `"dot"` (default, `2 · 10⁶`) or `"times"` (`2 × 10⁶`).
/// - form (auto, str): How values are written. `"auto"` (default): as typed, results rounded · `"decimal"`: everything rounded to `digits` · `"fraction"`: exact fractions (`⅙ A`) · `"input"`: given values exactly as typed.
/// - frac-max (auto, int): Largest denominator for `form: "fraction"`. Default `1000`.
/// - slash (auto, bool): Fractions as `1/2` instead of stacked. Default `false`.
/// - prefixes (auto, str, none, array): Prefixes picked for results: `"all"` (default: a f p n µ m k M G T; times from a minute on in min or h, not ks), `none` (base unit only) or an array like `("m", "k")` (`"u"` = µ).
/// - si (auto, bool): `definitions` also shows the SI value (`Q₀ = 3450 mAh = 12420 C`). Default `false`.
/// - steps (auto, str): Default for `steps` of `solve`: `"full"` (default), `"formula"`, `"result"` or `"value"`.
/// - layout (auto, str): Default for `layout` of `solve`: `"row"` (default) or `"column"`.
/// - inline-steps (auto, str): Steps for `inline: true` and inline code: `"full"`, `"formula"`, `"result"` (default) or `"value"`.
/// - implies (auto, content, symbol, str): What is written between steps. Default `sym.arrow.r.double` (⇒).
/// - approx (auto, bool): Write `≈` when the shown result is rounded. Default `true`.
/// - block (auto, bool): Display equations (centred, default) or inline.
/// - solutions (auto, int): How many solutions are listed. Default `6`; all are found, `root` picks among all.
/// - definitions (auto, str): How `definitions` looks: `"grid"` (default: columns side by side, aligned at `=`), `"column"`, `"inline"` or `"list"`.
/// - errors (auto, str): `"document"` (default): problems are shown as red notes · `"panic"`: stop the compilation.
/// -> content
#let solve-config(
  ..body,
  digits: auto, zeros: auto, sep: auto, group: auto, sci: auto, eng: auto, times: auto, form: auto, frac-max: auto, slash: auto,
  prefixes: auto, si: auto,
  steps: auto, inline-steps: auto, layout: auto, implies: auto, approx: auto, block: auto, solutions: auto,
  definitions: auto, errors: auto,
) = {
  let b = body.pos().at(0, default: none)
  if "list" in body.named() { return _err-now("the option list is now called definitions: solve-config(definitions: " + repr(body.named().list) + ")") + b }
  if body.named().len() > 0 { return _err-now("unknown option '" + body.named().keys().first() + "' (options: " + _DEF.keys().join(", ") + ")") + b }
  if body.pos().len() > 1 { return _err-now("solve-config takes at most one body") + b }
  if b != none and type(b) != content { return _err-now("solve-config: the body must be content, got " + _tn(b)) }
  let n = _check(_given((
    digits: digits, zeros: zeros, sep: sep, group: group, sci: sci, eng: eng, times: times, form: form, frac-max: frac-max, slash: slash,
    prefixes: prefixes, si: si, steps: steps, inline-steps: inline-steps, layout: layout, implies: implies, approx: approx, block: block,
    solutions: solutions, definitions: definitions, errors: errors,
  )))
  if _E(n) { return _err-now(n.err) + b }
  if b == none { _cf.update(c => c + n) } else {
    _cf.update(c => c + n + ("^": c))
    b
    _cf.update(c => c.at("^"))
  }
}
/// Forgets every definition (values and functions) made so far.
/// Inside a `solve-block`: back to how the block started (with `inherit: true` the outer definitions stay).
///
/// ```typ
/// #reset
/// ```
#let _restart(d) = if "^0" in d { let z = d.at("^0"); z.insert("^0", z); z } else { (:) }
#let reset = _st.update(_restart)
/// Shows the value of a defined variable inline, optionally converted to another unit.
///
/// ```typ
/// #def("I = 30 mA")
/// The current is #val("I"), or #val("I", unit: "A").   // 30 mA, 0,03 A
/// ```
///
/// - name (str): The variable, e.g. `"I"` or `"Q0"`.
/// - unit (none, str, content): Unit to convert to, e.g. `"A"`. Must have the same dimension as the value.
/// - config (dictionary): Options for this value, e.g. `(digits: 2)`; see `solve-config`.
/// -> content
#let val(name, unit: none, config: (:)) = _mine(context {
  let lc = _check(config)
  if _E(lc) { return _errc(lc.err, _cf.get()) }
  let cf = _cf.get() + lc
  if type(name) != str { return _errc("val: the name must be a string, got " + _tn(name), cf) }
  if type(unit) not in (type(none), str, content) { return _errc("val: unit must be a string like \"mA\", got " + _tn(unit), cf) }
  let n = name
  let q = _vars-of(_st.get()).at(n, default: none)
  if q == none { return _errc("unknown variable '" + n + "'", cf) }
  let u = unit
  if u != none {
    let u = _str(u)
    if _E(u) { return _errc(u.err, cf) }
    let un = _unit(u)
    if _E(un) { return _errc(un.err, cf) }
    let (f, dm) = un
    if dm != q.d { return _errc(n + " has unit " + (if q.d == _Z { "1 (no unit)" } else { _auto(q.d) }) + ", not " + u, cf) }
    q = (v: q.v, d: dm, raw: none, u: u, n: q.v / f)
  }
  _qc(q, cf)
})
#let _inabs(e, x) = if e.at(0) == "f" and e.at(1) == "abs" and _has(e, x) { true } else { e.slice(1).filter(t => type(t) == array).any(t => _inabs(t, x)) }
#let _gv(g) = if type(g) in (str, content) { let q = _q(g); if _E(q) { q } else { q.v } } else { float(g) }
// unit of the inputs to reuse for a result: only non-SI ones (h, mAh, eV); SI units get automatic prefixes anyway
#let _hint(ex, d, dm) = {
  if ex == none { return none }
  let us = _vars(ex).dedup().filter(n => n in d and d.at(n).d == dm).map(n => d.at(n).u).dedup()
  if us.len() != 1 or us.first() == "" { return none }
  let u = us.first()
  let c = u.clusters()
  let base = if u in _U { u } else if c.len() > 1 and c.first() in _P and c.slice(1).join() in _U { c.slice(1).join() } else { u }
  if base in _AU or base == "g" or base in ("Ohm", "ohm", "Omega", "\u{2126}") { none } else { u }
}
#let _mkq(v, dm, u, cf, ex: none, hint: none) = {
  let u = if u == none and hint != none { hint } else if u == none {
    let a = _auto(dm)
    // fractions read better in the base unit (1/6 A, not 500/3 mA)
    let r = if cf.form == "fraction" { _ratio(v / _unit(a).at(0), cf.frac-max) }
    if r != none and r.at(1) > 1 { a } else { _pre(v / _unit(a).at(0), a, cf) }
  } else {
    let un = _unit(u)
    if _E(un) { return un }
    if un.at(1) != dm { return _err("the result has unit " + (if dm == _Z { "1 (no unit)" } else { _auto(dm) }) + ", not " + u) }
    u
  }
  (v: v, d: dm, raw: none, u: u, n: v / _unit(u).at(0), ex: ex)
}
// which solution gets stored: root: n (1-based), a predicate, or auto (the only all-positive one, else the first)
#let _pick(sols, root, xs) = {
  if sols.len() == 0 { return none }
  if type(root) == int {
    if root < 1 or root > sols.len() { return _err("root: " + str(root) + ", but there are " + str(sols.len()) + " solution(s)") }
    return sols.at(root - 1)
  }
  if type(root) == function {
    let m = sols.filter(s => if s.len() == 1 { root(s.first()) } else { root(xs.zip(s).to-dict()) })
    if m.len() == 0 { return _err("no solution matches root") }
    return m.first()
  }
  let pos = sols.filter(s => s.all(v => v > 0))
  if pos.len() == 1 { pos.first() } else { sols.first() }
}

// inequalities: "R <= U/I" is solved like "R = U/I"; the unknown is stored at its boundary with the relation it has to it
#let _REL = ("≤": sym.lt.eq, "≥": sym.gt.eq, "<": sym.lt, ">": sym.gt)
#let _FLIP = ("≤": "≥", "≥": "≤", "<": ">", ">": "<")
#let _ineq(s) = {
  let s = s.replace("<=", "≤").replace(">=", "≥")
  let rs = s.matches(regex("[≤≥<>]"))
  if rs.len() == 0 { return (s, none) }
  if rs.len() > 1 or s.contains("=") { return _err("only one relation per inequality, in '" + s.trim() + "'") }
  (s.replace(rs.first().text, "="), rs.first().text)
}
#let _relc(q, rel) = { let r = q.at("rel", default: none); if r == none { rel } else { _REL.at(r) } }
// L rel R holds on the side of the boundary v where L − R moves the right way: rel as written if L − R grows with x
#let _bound(L, R, env, x, v, rel) = {
  let r = _nd(("+", L, ("-", R)), env + ((x): (float(v), (1.0,))))
  if r == none or r.at(1) == none or r.at(1).first() == 0 { return _err("can't tell on which side of its boundary " + x + " satisfies the inequality") }
  if r.at(1).first() > 0 { rel } else { _FLIP.at(rel) }
}

// stages of a solution, each a list of segments (lhs, ((rel, rhs, kind), …)); kind "sub" = numbers substituted
#let _render(stages, cf) = {
  let A = $&$.body
  let st = if cf.steps == "result" {
    let l = stages.last()
    ((segs: l.segs.map(((lhs, ps)) => (lhs, ps.slice(calc.max(ps.len() - 1, 0)))), join: l.join, rel: l.at("rel", default: none)),)
  } else if cf.steps == "formula" {
    stages.map(s => s + (segs: s.segs.map(((lhs, ps)) => (lhs, ps.filter(p => p.at(2) != "sub")))))
  } else { stages }
  let st = st.filter(s => s.segs.len() > 0).map(s => if s.at("rel", default: none) == none { s } else {
    s.segs = s.segs.map(((lhs, ps)) => (lhs, if ps.len() == 0 { ps } else { ((_REL.at(s.rel), ..ps.first().slice(1)),) + ps.slice(1) }))
    s
  })
  let imp = cf.implies
  let body = if cf.layout == "row" {
    st.map(s => s.segs.map(((lhs, ps)) => [#lhs#ps.map(((r, x, _)) => [#r #x]).join()]).join(s.join)).join($quad #imp quad$)
  } else {
    let lines = ()
    for (i, s) in st.enumerate() {
      for (j, (lhs, ps)) in s.segs.enumerate() {
        let pre = if i > 0 and j == 0 { $#imp$ + h(0.3em) }
        if ps.len() == 0 { lines.push([#pre#lhs#A]) }
        for (k, (r, x, _)) in ps.enumerate() { lines.push(if k == 0 { [#pre#lhs#A#r #x] } else { [#A#r #x] }) }
      }
    }
    lines.join(linebreak())
  }
  _eqn(body, cf)
}

// solve equations (strings) for their unknowns -> (new state, record for display) or an error
// un: target unit, none, a string (one unknown) or a dictionary name -> unit
#let _solve-into(d, eqs, un, opt, cf) = {
  let root = opt.at("root", default: auto)
  let d0 = _vars-of(d)
  let fns = d.at("#fn", default: (:))
  let eq = eqs.first()
  let env = _env(d0)
  let ps = ()
  for s in eqs {
    let p = s.split("=")
    if p.len() > 2 { return _err("more than one '=' in '" + s + "'") }
    let l = _parse(p.at(0), d0, f: fns)
    if _E(l) { return l }
    let r = if p.len() > 1 { _parse(p.at(1), d0, f: fns) }
    if _E(r) { return r }
    ps.push((l, r))
  }
  // no '=': just evaluate / convert, e.g. solve("I", "A")
  if ps.len() == 1 and ps.first().at(1) == none {
    let e = ps.first().at(0)
    let ev = _ev(e, d0)
    if _E(ev) { return ev }
    let (v, dm) = ev
    let q = _mkq(v, dm, if type(un) == str { un }, cf)
    if _E(q) { return q }
    return (d, (k: "expr", e: e, e0: _parse(eqs.first(), d0, f: fns, keep: true), q: q, d: d0))
  }
  if ps.any(p => p.at(1) == none) { return _err("every equation of a system needs an '='") }
  let all = ps.map(((L, R)) => _vars(L) + _vars(R)).flatten()
  let xs = if opt.at("unknown", default: none) != none { if type(opt.unknown) == str { (opt.unknown,) } else { opt.unknown } } else {
    let u = all.dedup().filter(n => n not in d0)
    // everything known already: a lone variable on either side is recomputed (U = R I, f(1) = x)
    let (L, R) = ps.first()
    if u.len() == 0 and ps.len() == 1 and L.at(0) == "v" { (L.at(1),) }
    else if u.len() == 0 and ps.len() == 1 and R.at(0) == "v" { (R.at(1),) } else { u }
  }
  if xs.len() != ps.len() {
    return _err(str(ps.len()) + " equation" + (if ps.len() > 1 { "s" }) + " but " + str(xs.len()) + " unknown" + (if xs.len() != 1 { "s" }) + " (" + (if xs == () { "none" } else { xs.join(", ") }) + ") in " + eqs.map(e => "'" + e.trim() + "'").join(", ") + if xs.len() > ps.len() { " – define the missing values first, or pass unknown: …" } else { " – pass unknown: …" })
  }
  if type(un) == str and xs.len() > 1 { return _err("several unknowns – give units as unit: (name: \"…\")") }
  let units = if type(un) == dictionary { un } else if type(un) == str { ((xs.first()): un) } else { (:) }
  for u in units.values() { let r = _unit(u); if _E(r) { return r } }
  let bad = units.keys().filter(k => k not in xs)
  if bad.len() > 0 { return _err("unit: " + bad.map(k => "'" + k + "'").join(", ") + " is not an unknown here (unknowns: " + xs.join(", ") + ")") }
  let us = xs.map(x => units.at(x, default: none))
  let g = opt.at("guess", default: none)
  for x in xs { let _ = d.remove(x, default: none) }

  let (poly, cn) = (none, none)
  if ps.len() == 1 {
    let x = xs.first()
    let (L, R) = ps.first()
    let cnt = all.filter(v => v == x).len()
    if cnt == 0 { return _err("'" + x + "' does not appear in '" + eq.trim() + "'") }
    let simple = cnt == 1 and not _inabs(L, x) and not _inabs(R, x) and g == none and root == auto
    // isolating can fail numerically (e.g. sqrt of a negative) – then solve numerically instead
    let ex = if simple { let i = _iso(L, R, x); if i != none { _s(i) } }
    if ex != none and _nd(ex, env) == none { ex = none }
    if ex == none {
      let a = _poly(L, x)
      let b = _poly(R, x)
      if a != none and b != none {
        poly = range(calc.max(a.len(), b.len())).map(i => _s(_pa(a.at(i, default: ("n", 0)), ("-", b.at(i, default: ("n", 0))))))
        let rs = poly.map(c => _nd(c, env))
        if none in rs {
          let e = poly.map(c => _ev(c, d0)).find(_E)
          return if e != none { e } else { _err("cannot evaluate '" + eq.trim() + "'") }
        }
        cn = rs.map(r => r.at(0))
        while cn.len() > 1 and cn.last() == 0 { let _ = cn.pop(); let _ = poly.pop() }
        if cn.len() == 1 { return _err("'" + x + "' cancels out of '" + eq.trim() + "'") }
      }
    }
    // symbolic: x occurs once, or the equation is linear in x
    if ex == none and poly != none and poly.len() == 2 { ex = _s(("/", ("-", poly.at(0)), poly.at(1))) }
    if ex != none {
      let ev = _ev(ex, d0)
      if _E(ev) { return ev }
      let (v, dm) = ev
      let q = _mkq(v, dm, us.first(), cf, ex: ex, hint: _hint(ex, d0, dm))
      if _E(q) { return q }
      d.insert(x, q)
      let p = eq.split("=")
      return (d, (k: "iso", L: L, R: R, L0: _parse(p.at(0), d0, f: fns, keep: true), R0: _parse(p.at(1), d0, f: fns, keep: true), x: x, ex: ex, q: q, d: d0))
    }
  }
  // systems: solve equations that contain only one unknown (once) symbolically and substitute
  let pre = (:)
  if ps.len() > 1 and g == none and root == auto {
    let go = true
    while go {
      go = false
      for (i, (L, R)) in ps.enumerate() {
        let vs = (_vars(L) + _vars(R)).filter(v => v in xs)
        if vs.len() != 1 or _inabs(L, vs.first()) or _inabs(R, vs.first()) { continue }
        let x = vs.first()
        let ex = _iso(L, R, x)
        if ex == none { continue }
        let ex = _s(ex)
        if _nd(ex, env) == none { continue }
        let ev = _ev(ex, d0)
        if _E(ev) { return ev }
        let (v, dm) = ev
        let j = xs.position(y => y == x)
        let q = _mkq(v, dm, us.at(j), cf, ex: ex, hint: _hint(ex, d0, dm))
        if _E(q) { return q }
        pre.insert(x, q)
        d0.insert(x, pre.at(x)); env.insert(x, (float(v), none))
        let _ = ps.remove(i); let _ = xs.remove(j); let _ = us.remove(j)
        go = ps.len() > 0
        break
      }
    }
    if ps.len() == 0 {
      for (x, q) in pre { d.insert(x, q) }
      return (d, (k: "roots", eqs: eqs.map(s => { let p = s.split("="); (_parse(p.at(0), d0, f: fns, keep: true), _parse(p.at(1), d0, f: fns, keep: true)) }), xs: pre.keys(), poly: none, cn: none, sols: ((pre.values().map(q => q.v)),), dm: pre.values().map(q => q.d), u: pre.values().map(q => q.u), d: d0))
    }
  }
  // numeric
  let dd = _dims(ps, d0, env, xs)
  if _E(dd) { return dd }
  let (D, free) = dd
  let dms = range(xs.len()).map(i => if i in free and us.at(i) != none { _unit(us.at(i)).at(1) } else { D.at(i) })
  let sc = us.map(u => if u == none { 1.0 } else { _unit(u).at(0) })
  let sols = if g != none {
    let g = if type(g) == dictionary { xs.enumerate().map(((i, x)) => if x in g { _gv(g.at(x)) } else { sc.at(i) }) } else { (_gv(g),) }
    let b = _first-err(g)
    if b != none { return b }
    let r = _newton(ps, env, xs, g)
    if r == none { () } else { (r,) }
  } else if poly != none { _roots(cn).map(r => (r,)) }
  else if xs.len() == 1 { _scan(..ps.first(), env, xs.first(), sc.first()).map(r => (r,)) }
  else { _msolve(ps, env, xs, sc, none) }
  let sel = _pick(sols, root, xs)
  if _E(sel) { return sel }
  if sel != none {
    for (i, x) in xs.enumerate() {
      let q = _mkq(sel.at(i), dms.at(i), us.at(i), cf)
      if _E(q) { return q }
      d.insert(x, q)
    }
  }
  // show peeled-off unknowns together with the numeric ones
  let px = pre.keys()
  let ps0 = eqs.map(s => { let p = s.split("="); (_parse(p.at(0), d0, f: fns, keep: true), _parse(p.at(1), d0, f: fns, keep: true)) })
  if sel != none { for (x, q) in pre { d.insert(x, q) } }
  (d, (k: "roots", eqs: ps0, xs: px + xs, poly: poly, cn: cn, sols: sols.map(s => pre.values().map(q => q.v) + s),
    dm: pre.values().map(q => q.d) + dms, u: pre.values().map(q => q.u) + us, d: d0))
}
// one record -> stages
#let _stages(r, cf) = {
  let d = r.d
  let d = r.d
  let sub = d.pairs().map(((k, q)) => (k, (_qc(q, cf), if q.u != "" { 3 } else if q.v < 0 { 2 } else { 9 }))).to-dict()
  let D(e, s: none) = _dp(e, s, cf).at(0)
  let rel(q) = if not cf.approx or _nshow(q, cf).at(1) { $=$ } else { $approx$ }
  let seg(lhs, ..ps) = (lhs, ps.pos())
  let stages = if r.k == "expr" {
    let q = r.q
    let ps = ()
    let mid = if r.e.at(0) == "v" { _qc(d.at(r.e.at(1)), cf) } else { D(r.e, s: sub) }
    // a computed variable: its formula, if its inputs still give its value
    let ex = if r.e.at(0) == "v" { d.at(r.e.at(1)).at("ex", default: none) }
    let ev = if ex != none and ex != r.e { _ev(ex, d) }
    if ev != none and not _E(ev) and calc.abs(ev.at(0) - d.at(r.e.at(1)).v) <= 1e-9 * calc.abs(ev.at(0)) {
      ps.push(($=$, D(ex), "formula"))
      if _vars(ex).len() > 0 { ps.push(($=$, D(ex, s: sub), "sub")) }
      ps.push((rel(q), _qc(q, cf), "res"))
    } else {
    if r.e0 != r.e { ps.push(($=$, D(r.e), "formula")) }
    if mid != D(r.e) { ps.push(($=$, mid, "sub")) }
    if _qc(q, cf) != mid { ps.push((rel(q), _qc(q, cf), "res")) }
    }
    if ps.len() == 0 { ps.push(($=$, _qc(q, cf), "res")) }
    let br = if r.e.at(0) == "v" { d.at(r.e.at(1)).at("rel", default: none) }
    ((segs: (seg(D(r.e0), ..ps),), join: none, rel: br),)
  } else if r.k == "iso" {
    let (L, R, x, ex, q) = (r.L0, r.R0, r.x, r.ex, r.q)
    let ps = ()
    if L == ("v", x) and R != r.R { ps.push(($=$, D(R), "formula")) } // f(2) before its expansion
    if not (_vars(ex).len() == 0 and ex.at(0) in ("n", "q")) { ps.push(($=$, D(ex), "formula")) }
    if _vars(ex).len() >= (if ex.at(0) == "v" { 2 } else { 1 }) { ps.push(($=$, D(ex, s: sub), "sub")) }
    ps.push((rel(q), _qc(q, cf), "res"))
    let r0 = if "rel0" in r { _REL.at(r.rel0) } else { $=$ }
    (if L == ("v", x) { () } else { ((segs: (seg(D(L), (r0, D(R), "eq")),), join: none),) }) + ((segs: (seg(_name(x), ..ps),), join: none, rel: r.at("rel", default: none)),)
  } else {
    let r0 = if "rel0" in r { _REL.at(r.rel0) } else { $=$ }
    let stages = (if r.eqs.len() == 1 { let (L, R) = r.eqs.first(); (segs: (seg(D(L), (r0, D(R), "eq")),), join: none) }
      else { (segs: (seg(math.cases(..r.eqs.map(((L, R)) => $#D(L) = #D(R)$))),), join: none) },)
    if r.poly != none and r.at("rel", default: none) == none {
      let x = r.xs.first()
      let cs = if r.cn.last() < 0 { r.poly.map(c => _s(("-", c))) } else { r.poly }
      let ts = ()
      for i in range(cs.len() - 1, -1, step: -1) {
        let c = cs.at(i)
        if c == ("n", 0) { continue }
        let neg = c.at(0) == "-" or (c.at(0) == "n" and c.at(1) < 0)
        let c = if not neg { c } else if c.at(0) == "-" { c.at(1) } else { ("n", -c.at(1)) }
        let xp = if i == 0 { none } else if i == 1 { ("v", x) } else { ("^", ("v", x), ("n", i)) }
        let t = if xp == none { c } else if c == ("n", 1) { xp } else { ("*", c, xp) }
        ts.push(if neg { ("-", t) } else { t })
      }
      let pe = if ts.len() == 1 { ts.first() } else { ("+", ..ts) }
      let (L, R) = r.eqs.first()
      if $#D(pe) = 0$ != $#D(L) = #D(R)$ and _vars(L).len() + _vars(R).len() > 0 {
        stages.push((segs: (seg(D(pe), ($=$, $0$, "formula")),), join: none))
      }
    }
    let qs(s) = s.enumerate().map(((i, v)) => _mkq(v, r.dm.at(i), r.u.at(i), cf))
    let n = cf.solutions
    let res = if r.sols.len() == 0 { (segs: (seg($"no real solution"$),), join: none) } else if r.xs.len() == 1 {
      let x = r.xs.first()
      let segs = r.sols.slice(0, calc.min(n, r.sols.len())).enumerate().map(((i, s)) => {
        let q = qs(s).first()
        seg(if r.sols.len() == 1 { _name(x) } else { math.attach(_name(x), br: str(i + 1)) }, (rel(q), _qc(q, cf), "res"))
      })
      if r.sols.len() > n { segs.push(seg($dots.h$)) }
      (segs: segs, join: $, quad$, rel: r.at("rel", default: none))
    } else {
      let segs = r.sols.slice(0, calc.min(n, r.sols.len())).map(s => seg(math.cases(..r.xs.zip(qs(s)).map(((x, q)) => $#_name(x) #rel(q) #_qc(q, cf)$))))
      if r.sols.len() > n { segs.push(seg($dots.h$)) }
      (segs: segs, join: $quad or quad$)
    }
    stages + (res,)
  }
  stages
}

// definition record -> content, following cf.steps
#let _show(r, st, cf) = {
  if r.k == "error" { return _errc(r.err, cf, wide: cf.block) }
  let value(x) = _qc(st.at(x), cf)
  if r.k == "given" {
    if cf.steps == "value" { _eqn(_qc(r.q, cf), cf) } else { _eqn($#_name(r.name) #_relc(r.q, $=$) #_qc(r.q, cf)$.body, cf) }
  } else if r.k == "fn" {
    let lhs = $#_name(r.name) (#r.params.map(_name).join($, $))$
    _eqn($#lhs = #_dp(r.body, none, cf).at(0)$.body, cf)
  } else if cf.steps == "value" {
    let xs = if r.k == "iso" { (r.x,) } else if r.k == "roots" { r.xs } else { () }
    let vs = if r.k == "expr" { (_qc(r.q, cf),) } else if r.k == "iso" { (_qc(r.q, cf),) } else { xs.filter(x => x in st).map(value) }
    _eqn(vs.join($, $), cf)
  } else { _render(_stages(r, cf), cf) }
}

#let _ID = "^\\p{L}[\\p{L}\\p{N}_]*$"
// the core of def: equations (strings), target unit, solve options, local cfg, output?, listed
#let _def(eqs, un, opt, lc, out, listed) = _mine({
  context {
    let cf = _cf.get() + lc
    _st.update(d => {
      let recs = ()
      let defs = d.at("!", default: ())
      let add(defs, name, kind) = defs.filter(e => e.name != name) + ((name: name, kind: kind, listed: listed, cf: lc),)
      let fail(s, e) = (k: "error", err: e.err, src: s)
      let pending = ()
      // an inequality's record and value get the relation of the unknown to its boundary
      let bound(d, rec, s, rel, d0, fns, src) = {
        let one = rec.k == "iso" or (rec.k == "roots" and rec.xs.len() == 1 and rec.sols.len() == 1)
        if not one { return _err(if rec.k == "roots" and rec.sols.len() == 0 { "no boundary" } else { "several boundaries" } + " for the inequality '" + src.trim() + "' – only inequalities with one boundary are supported") }
        let (x, v) = if rec.k == "iso" { (rec.x, rec.q.v) } else { (rec.xs.first(), rec.sols.first().first()) }
        let p = s.split("=")
        let (L, R) = p.map(e => _parse(e, d0, f: fns))
        // isolating finds one boundary (sqrt gives +2 for y^2 < 4): look for others
        let more = _scan(L, R, _env(d0), x, if v == 0 { 1.0 } else { calc.abs(v) }).filter(t => calc.abs(t - v) > 1e-6 * calc.max(calc.abs(v), 1e-300))
        if more.len() > 0 { return _err("several boundaries for the inequality '" + s.trim() + "' – only inequalities with one boundary are supported") }
        let rx = _bound(L, R, _env(d0), x, v, rel)
        if _E(rx) { return rx }
        let q = d.at(x)
        q.rel = rx
        d.insert(x, q)
        rec.rel = rx
        rec.rel0 = rel
        (d, rec)
      }
      for s0 in eqs {
        let s = _ineq(s0)
        if _E(s) { recs.push(fail(s0, s)); continue }
        let (s, rel) = s
        let (d0, fns0) = (_vars-of(d), d.at("#fn", default: (:)))
        let p = s.split("=")
        if p.len() > 2 { recs.push(fail(s, _err("more than one '=' in '" + s.trim() + "'"))); continue }
        let (l, r) = (p.at(0).trim(), p.at(1, default: "").trim())
        let fm = l.match(regex("^(\\p{L}[\\p{L}\\p{N}_]*)\\s*\\((.*)\\)$"))
        // a known function on the left (f(x) = 9) is an equation to solve, not a new definition
        if rel == none and p.len() == 2 and fm != none and fm.captures.at(0) not in _F and fm.captures.at(0) not in d.at("#fn", default: (:)) {
          // function: f(x, y) = …
          let name = fm.captures.at(0)
          let ps = fm.captures.at(1).split(",").map(str.trim)
          if ps.any(x => x.match(regex(_ID)) == none) { recs.push(fail(s, _err("the parameters of " + name + " must be names, got '" + fm.captures.at(1) + "'"))); continue }
          let fns = d.at("#fn", default: (:))
          let body = _parse(r, _vars-of(d), f: fns)
          if _E(body) { recs.push(fail(s, body)); continue }
          fns.insert(name, (params: ps, body: body))
          d.insert("#fn", fns)
          defs = add(defs, name, "function")
          recs.push((k: "fn", name: name, params: ps, body: _parse(r, _vars-of(d), f: fns, keep: true)))
          continue
        }
        let id = p.len() == 2 and l.match(regex(_ID)) != none
        if id and _isq(r) {
          // plain value: R = 7.2 kOhm
          let q = _q(r)
          if _E(q) { recs.push(fail(s, q)); continue }
          if rel != none { q.rel = rel }
          d.insert(l, q)
          defs = add(defs, l, "given")
          recs.push((k: "given", name: l, q: q))
          continue
        }
        if id and pending.len() == 0 {
          // expression of known variables: U = R I
          let e = _parse(r, _vars-of(d), f: d.at("#fn", default: (:)))
          if _E(e) { recs.push(fail(s, e)); continue }
          // "z = 7 furlongs": a number with something that is neither a unit nor a variable
          let miss = _vars(e).dedup().filter(v => v not in _vars-of(d) and v != l)
          if l not in d and miss.len() > 0 and r.match(regex("^[-+−]?[\\d.,(]")) != none {
            recs.push(fail(s, _err(miss.map(v => "'" + v + "'").join(", ") + " is neither a unit nor a defined variable")))
            continue
          }
          if _vars(e).all(v => v in _vars-of(d) and v != l) {
            let res = _solve-into(d, (s,), if type(un) == dictionary { un.at(l, default: none) } else { un }, opt + (unknown: l), cf)
            let res = if rel == none or _E(res) { res } else { bound(..res, s, rel, d0, fns0, s0) }
            if _E(res) { recs.push(fail(s0, res)); continue }
            d = res.at(0)
            defs = add(defs, l, "computed")
            recs.push(res.at(1))
            continue
          }
        }
        if rel == none { pending.push(s); continue }
        // inequalities are solved on their own, never as part of a system
        let res = _solve-into(d, (s,), un, opt, cf)
        let res = if _E(res) { res } else { bound(..res, s, rel, d0, fns0, s0) }
        if _E(res) { recs.push(fail(s0, res)); continue }
        d = res.at(0)
        if res.at(1).k == "iso" { defs = add(defs, res.at(1).x, "computed") } else { defs = add(defs, res.at(1).xs.first(), "computed") }
        recs.push(res.at(1))
      }
      if pending.len() > 0 {
        let res = _solve-into(d, pending, un, opt, cf)
        if _E(res) { recs.push(fail(pending.join("; "), res)) } else {
          let (d2, rec) = res
          d = d2
          for x in (if rec.k == "iso" { (rec.x,) } else if rec.k == "roots" { rec.xs } else { () }) {
            if x in d { defs = add(defs, x, "computed") }
          }
          recs.push(rec)
        }
      }
      d.insert("!", defs)
      d.insert("..", recs)
      d
    })
  }
  // always read the state right away: silent definitions still report their problems (updates run lazily)
  context {
    let cf = _cf.get() + lc
    if cf.steps == auto { cf.steps = cf.inline-steps } // inline code / inline: true
    let st = _st.get()
    let recs = st.at("..").filter(r => out or r.k == "error")
    let outs = recs.map(r => _show(r, _vars-of(st), cf))
    if not cf.block { outs.join(", ") } else { outs.join() }
  }
})

#let _STEPS = ("full", "formula", "result", "value")
// typed display arguments -> local options, or an error message
#let _disp(steps, layout, inline) = {
  if steps != auto and steps not in _STEPS and type(steps) != bool { return _err("steps: use " + _STEPS.map(repr).join(", ")) }
  if layout != auto and layout not in ("row", "column") { return _err("layout: use \"row\" or \"column\"") }
  if type(inline) != bool { return _err("inline: true or false") }
  let o = (:)
  if inline { o.block = false }
  if steps != auto { o.steps = if steps == true { "full" } else if steps == false { "result" } else { steps } }
  if layout != auto { o.layout = layout }
  o
}

/// Defines values, functions or unknowns from equations, silently.
///
/// Every positional argument is an equation, as a string or as math. Several are
/// defined in order; equations that can't be defined one by one are solved together
/// as a system. `def` shows nothing (except problems) unless `output: true`; `solve`
/// is the same with the output on.
///
/// ```typ
/// #def("R = 7.2 kOhm", "I = 415 uA")     // values: a number with a unit
/// #def($Q_0 = 3450 "mAh"$)                // math works too
/// #def("U = R I")                          // computed from defined values
/// #def("f(x) = x^2 + 1")                   // a function, usable as f(3)
/// #def("Q0/2 = I t", unit: "h")            // an equation: t is solved for
/// #def("x + y = 3", "x - y = 1")           // a system
/// ```
///
/// Inside an equation `[5 V]` is a value and `@R` inlines the formula of `R`.
///
/// - ..eqs (str, content): The equations. A single one without `=` (`"Q0/I"`) is evaluated (only useful with `output: true`).
/// - unit (none, str, content, dictionary): Unit of the result: `"mA"` (or math `$"mA"$`). For a system a dictionary per unknown: `(I: "mA", U: "V")`. Without it: the SI unit with an automatic prefix, or the non-SI unit of the inputs (h, mAh). If the result has another unit (`"x = f(1)"` gives volts, `unit: "A"`), that is an error.
/// - output (bool): Show the definition (like `solve` does).
/// - steps (auto, str, bool): What is shown: `"full"`: equation ⇒ formula = numbers = result · `"formula"`: without the numbers · `"result"`: `t = 57,5 h` · `"value"`: just `57,5 h`. `auto`: the option `steps` (default `"full"`), or `inline-steps` with `inline: true`.
/// - layout (auto, str): `"row"`: all steps in one line · `"column"`: one step per line, aligned at `=`. `auto`: the option `layout` (default `"row"`).
/// - inline (bool): In the running text instead of a display block; `steps` default to `inline-steps` (`"result"`).
/// - unknown (none, str, array): What to solve for, `"t"` or `("x", "y")`. Only needed when it isn't clear; by default all variables that aren't defined yet are the unknowns (one per equation). If everything is defined, a lone variable on either side is recomputed.
/// - root (auto, int, function): Which solution is stored when there are several: `2` (1-based), or a test like `x => x > 0` (for systems it gets a dictionary). `auto`: the only all-positive one, else the first.
/// - guess (none, str, content, int, float, dictionary): Newton start value instead of the search: `"-1 A"`, or `(U: "5 V")` for systems.
/// - listed (bool): `false` hides it from `definitions`.
/// - config (dictionary): Options for this call as a dictionary, `(digits: 2)`. Same as passing them directly.
/// - digits (auto, int): Significant digits of computed values, 1…15. Default `4`: `57,5 h`, `123500`.
/// - zeros (auto, bool): Keep trailing zeros up to `digits` (`2,500 V`). Default `false`.
/// - sep (auto, str): Decimal separator. `"lang"` (default): `,` for de, fr, it, … and `.` otherwise; or any string like `","`.
/// - group (auto, none, str, symbol, content): Digit grouping for integer parts with 5+ digits, e.g. `sym.space.thin` → `12 345`. Default `none`.
/// - sci (auto, none, array): `(low, high)`: plain notation while the exponent `e` is in `low ≤ e < high`, else `2,5 · 10⁻⁷`. Default `(-3, 6)`; `none` never switches.
/// - eng (auto, bool): Engineering notation, exponents in steps of 3. Default `false`.
/// - times (auto, str): `"dot"` (default, `2 · 10⁶`) or `"times"` (`2 × 10⁶`).
/// - form (auto, str): How values are written. `"auto"` (default): as typed, results rounded · `"decimal"`: everything rounded to `digits` · `"fraction"`: exact fractions (`⅙ A`) · `"input"`: given values exactly as typed.
/// - frac-max (auto, int): Largest denominator for `form: "fraction"`. Default `1000`.
/// - slash (auto, bool): Fractions as `1/2` instead of stacked. Default `false`.
/// - prefixes (auto, str, none, array): Prefixes picked for results: `"all"` (default: a f p n µ m k M G T; times from a minute on in min or h, not ks), `none` (base unit only) or an array like `("m", "k")` (`"u"` = µ).
/// - si (auto, bool): `definitions` also shows the SI value (`Q₀ = 3450 mAh = 12420 C`). Default `false`.
/// - inline-steps (auto, str): Steps for `inline: true` and inline code: `"full"`, `"formula"`, `"result"` (default) or `"value"`.
/// - implies (auto, content, symbol, str): What is written between steps. Default `sym.arrow.r.double` (⇒).
/// - approx (auto, bool): Write `≈` when the shown result is rounded. Default `true`.
/// - block (auto, bool): Display equations (centred, default) or inline.
/// - solutions (auto, int): How many solutions are listed. Default `6`; all are found, `root` picks among all.
/// - definitions (auto, str): How `definitions` looks: `"grid"` (default: columns side by side, aligned at `=`), `"column"`, `"inline"` or `"list"`.
/// - errors (auto, str): `"document"` (default): problems are shown as red notes · `"panic"`: stop the compilation.
/// -> content
#let def(
  ..eqs,
  unit: none, output: false, steps: auto, layout: auto, inline: false,
  unknown: none, root: auto, guess: none, listed: true, config: (:),
  digits: auto, zeros: auto, sep: auto, group: auto, sci: auto, eng: auto, times: auto, form: auto, frac-max: auto, slash: auto,
  prefixes: auto, si: auto, inline-steps: auto, implies: auto, approx: auto, block: auto, solutions: auto,
  definitions: auto, errors: auto,
) = {
  if eqs.named().len() > 0 {
    let (k, v) = eqs.named().pairs().first()
    // the old form def(I: "30 mA")
    if type(v) in (str, content, int, float) and _isq(v) {
      return _err-now("unknown argument '" + k + "' – definitions are strings: def(\"" + k + " = " + _str(v).trim() + "\")")
    }
    return _err-now("unknown argument '" + k + "' (arguments: unit, output, steps, layout, inline, unknown, root, guess, listed, config and the options " + _DEF.keys().filter(k => k not in ("steps", "layout")).join(", ") + ")")
  }
  if type(config) != dictionary { return _err-now("config: expected a dictionary of options like (digits: 3), got " + _tn(config)) }
  let lc = _check(_given((
    digits: digits, zeros: zeros, sep: sep, group: group, sci: sci, eng: eng, times: times, form: form, frac-max: frac-max, slash: slash,
    prefixes: prefixes, si: si, inline-steps: inline-steps, implies: implies, approx: approx, block: block, solutions: solutions,
    definitions: definitions, errors: errors,
  )) + config)
  if _E(lc) { return _err-now(lc.err) }
  let dl = _disp(steps, layout, inline)
  if _E(dl) { return _err-now(dl.err) }
  for (n, v) in (output: output, listed: listed) { if type(v) != bool { return _err-now(n + ": true or false, got " + _tn(v)) } }
  if not (unknown == none or type(unknown) == str or (type(unknown) == array and unknown.len() > 0 and unknown.all(x => type(x) == str))) {
    return _err-now("unknown: a name like \"t\" or an array of names like (\"x\", \"y\")")
  }
  if type(root) not in (type(auto), int, function) { return _err-now("root: auto, a number (1, 2, …) or a function like x => x > 0, got " + _tn(root)) }
  let gok(g) = type(g) in (str, content, int, float)
  if not (guess == none or gok(guess) or (type(guess) == dictionary and guess.values().all(gok))) {
    return _err-now("guess: a value like \"-1 A\", or a dictionary like (U: \"5 V\")")
  }
  // unit: "mA", $"mA"$ or (I: "mA")
  let uok(u) = type(u) in (str, content)
  if not (unit == none or uok(unit) or (type(unit) == dictionary and unit.values().all(uok))) {
    return _err-now("unit: a unit like \"mA\", or for several unknowns a dictionary like (I: \"mA\"), got " + _tn(unit))
  }
  let un = if type(unit) == dictionary { unit.pairs().map(((k, u)) => (k, _str(u))).to-dict() } else if unit != none { _str(unit) }
  let b = if type(un) == dictionary { un.values().find(_E) } else if _E(un) { un }
  if b != none { return _err-now(b.err) }
  // the equations
  let pos = eqs.pos()
  if pos.len() == 0 { return _err-now("def needs something to define, e.g. def(\"I = 30 mA\")") }
  let bad = pos.find(x => type(x) not in (str, content))
  if bad != none { return _err-now("expected an equation as a string or math, got " + repr(bad) + " – e.g. def(\"I = 30 mA\")") }
  let strs = pos.map(_str)
  let b = _first-err(strs)
  if b != none { return _err-now(b.err) }
  // the target unit used to be the 2nd positional argument: solve("Q0/I", "h")
  for s in strs.slice(1).map(str.trim) {
    if s != "" and not s.contains("=") and _isu(s) { return _err-now("'" + s + "' looks like a unit – pass it as unit: \"" + s + "\"") }
  }
  // inline without explicit steps: the shorter inline-steps
  let lc = lc + (if inline and steps == auto { (steps: auto) } else { (:) }) + dl
  _def(strs, un, (unknown: unknown, root: root, guess: guess), lc, output, listed)
}
/// Solves equations and shows the way: equation ⇒ rearranged formula = numbers = result.
///
/// Solves for all unknowns, i.e. all variables that aren't defined yet (one unknown per
/// equation), and stores the results like `def`. The unknown can be on either side,
/// also inside a function call. If the unknown occurs once it is isolated symbolically;
/// polynomials give all real roots; anything else is solved numerically.
///
/// ```typ
/// #def("Q0 = 3450 mAh", "I = 30 mA")
/// #solve("Q0/2 = I t", unit: "h")          // Q₀/2 = I t ⇒ t = Q₀/(2I) = … = 57,5 h
/// #solve("I", unit: "A")                    // no '=': convert, I = 30 mA = 0,03 A
/// #solve("x^2 + 2x - 3 = 0")                // all real roots
/// #solve("P = U I", "U = R I")              // a system
/// Runtime: #solve("Q0/I", unit: "h", inline: true, steps: "value")   // 115 h
/// #solve("Q0/2 = I t", unit: "h", layout: "column", steps: "formula")
/// ```
///
/// - ..eqs (str, content): The equations (or a single expression without `=` to evaluate).
/// - unit (none, str, content, dictionary): Unit of the result: `"mA"` (or math `$"mA"$`). For a system a dictionary per unknown: `(I: "mA", U: "V")`. Without it: the SI unit with an automatic prefix, or the non-SI unit of the inputs (h, mAh). If the result has another unit (`"x = f(1)"` gives volts, `unit: "A"`), that is an error.
/// - output (bool): `false` defines silently, like `def`.
/// - steps (auto, str, bool): What is shown: `"full"`: equation ⇒ formula = numbers = result · `"formula"`: without the numbers · `"result"`: `t = 57,5 h` · `"value"`: just `57,5 h`. `auto`: the option `steps` (default `"full"`), or `inline-steps` with `inline: true`.
/// - layout (auto, str): `"row"`: all steps in one line · `"column"`: one step per line, aligned at `=`. `auto`: the option `layout` (default `"row"`).
/// - inline (bool): In the running text instead of a display block; `steps` default to `inline-steps` (`"result"`).
/// - unknown (none, str, array): What to solve for, `"t"` or `("x", "y")`. Only needed when it isn't clear; by default all variables that aren't defined yet are the unknowns (one per equation). If everything is defined, a lone variable on either side is recomputed.
/// - root (auto, int, function): Which solution is stored when there are several: `2` (1-based), or a test like `x => x > 0` (for systems it gets a dictionary). `auto`: the only all-positive one, else the first.
/// - guess (none, str, content, int, float, dictionary): Newton start value instead of the search: `"-1 A"`, or `(U: "5 V")` for systems.
/// - listed (bool): `false` hides it from `definitions`.
/// - config (dictionary): Options for this call as a dictionary, `(digits: 2)`. Same as passing them directly.
/// - digits (auto, int): Significant digits of computed values, 1…15. Default `4`: `57,5 h`, `123500`.
/// - zeros (auto, bool): Keep trailing zeros up to `digits` (`2,500 V`). Default `false`.
/// - sep (auto, str): Decimal separator. `"lang"` (default): `,` for de, fr, it, … and `.` otherwise; or any string like `","`.
/// - group (auto, none, str, symbol, content): Digit grouping for integer parts with 5+ digits, e.g. `sym.space.thin` → `12 345`. Default `none`.
/// - sci (auto, none, array): `(low, high)`: plain notation while the exponent `e` is in `low ≤ e < high`, else `2,5 · 10⁻⁷`. Default `(-3, 6)`; `none` never switches.
/// - eng (auto, bool): Engineering notation, exponents in steps of 3. Default `false`.
/// - times (auto, str): `"dot"` (default, `2 · 10⁶`) or `"times"` (`2 × 10⁶`).
/// - form (auto, str): How values are written. `"auto"` (default): as typed, results rounded · `"decimal"`: everything rounded to `digits` · `"fraction"`: exact fractions (`⅙ A`) · `"input"`: given values exactly as typed.
/// - frac-max (auto, int): Largest denominator for `form: "fraction"`. Default `1000`.
/// - slash (auto, bool): Fractions as `1/2` instead of stacked. Default `false`.
/// - prefixes (auto, str, none, array): Prefixes picked for results: `"all"` (default: a f p n µ m k M G T; times from a minute on in min or h, not ks), `none` (base unit only) or an array like `("m", "k")` (`"u"` = µ).
/// - si (auto, bool): `definitions` also shows the SI value (`Q₀ = 3450 mAh = 12420 C`). Default `false`.
/// - inline-steps (auto, str): Steps for `inline: true` and inline code: `"full"`, `"formula"`, `"result"` (default) or `"value"`.
/// - implies (auto, content, symbol, str): What is written between steps. Default `sym.arrow.r.double` (⇒).
/// - approx (auto, bool): Write `≈` when the shown result is rounded. Default `true`.
/// - block (auto, bool): Display equations (centred, default) or inline.
/// - solutions (auto, int): How many solutions are listed. Default `6`; all are found, `root` picks among all.
/// - definitions (auto, str): How `definitions` looks: `"grid"` (default: columns side by side, aligned at `=`), `"column"`, `"inline"` or `"list"`.
/// - errors (auto, str): `"document"` (default): problems are shown as red notes · `"panic"`: stop the compilation.
/// -> content
#let solve(
  ..eqs,
  unit: none, output: true, steps: auto, layout: auto, inline: false,
  unknown: none, root: auto, guess: none, listed: true, config: (:),
  digits: auto, zeros: auto, sep: auto, group: auto, sci: auto, eng: auto, times: auto, form: auto, frac-max: auto, slash: auto,
  prefixes: auto, si: auto, inline-steps: auto, implies: auto, approx: auto, block: auto, solutions: auto,
  definitions: auto, errors: auto,
) = def(
  ..eqs,
  unit: unit, output: output, steps: steps, layout: layout, inline: inline,
  unknown: unknown, root: root, guess: guess, listed: listed, config: config,
  digits: digits, zeros: zeros, sep: sep, group: group, sci: sci, eng: eng, times: times, form: form, frac-max: frac-max, slash: slash,
  prefixes: prefixes, si: si, inline-steps: inline-steps, implies: implies, approx: approx, block: block, solutions: solutions,
  definitions: definitions, errors: errors,
)

// one line of a code block: "U = R I -> mV" (a system: equations separated by ';'; comments with //; "= expr" prints a value)
#let _line(s, block) = {
  let s = s.split("//").first().trim()
  if s == "" { return none }
  let (e, u) = if s.contains("->") { let p = s.split("->"); (p.at(0), p.at(1).trim()) } else { (s, none) }
  let lc = (block: block) + if not block { (steps: auto) } else { (:) }
  if e.trim().starts-with("=") { return _def((e.trim().slice(1),), u, (:), lc + (steps: "value"), true, false) }
  _def(e.split(";").map(str.trim).filter(x => x != ""), u, (:), lc, true, true)
}
// one side of a math equation as typed: decimal commas (0,95) and quantities written without brackets
// (frac(3450/2 "mAh", 30 "mA") → [(3450)/(2) mAh]/[30 mA]): numbers followed by units that aren't defined variables
// bare: units may also follow a call or a name (f(20) Ω is f(20) times 1 Ω); only for values, not equations, where an undefined A is a variable
#let _math-src(s, vars, bare: false) = {
  let s = s.replace(regex("(\d),(\d)"), m => m.captures.at(0) + "." + m.captures.at(1))
  let t = _tk(s)
  if _isq(s) and t.all(k => k not in vars) { return s } // a plain value: R = 7.2 kΩ
  let num(k) = k.match(regex("^[\d.]")) != none
  let name(k) = k != none and k.match(regex("^\p{L}")) != none
  let unit(k) = name(k) and k not in vars and _isu(k)
  // a parenthesized unit as math writes it: ( mm ^ ( 2 ) ) -> ("mm^2", index after it), or none
  let group(j) = {
    if t.at(j, default: none) != "(" { return none }
    let (dp, e) = (0, j)
    while e < t.len() {
      if t.at(e) == "(" { dp += 1 } else if t.at(e) == ")" { dp -= 1; if dp == 0 { break } }
      e += 1
    }
    let inner = t.slice(j + 1, calc.min(e, t.len()))
    if e >= t.len() or not inner.any(unit) or not inner.all(k => unit(k) or k in ("^", "(", ")", "-", "−", "*", "·", "⋅") or k.match(regex("^\d+$")) != none) { return none }
    let u = inner.join(" ").replace(regex("\^\s*\(\s*([-−]?)\s*(\d+)\s*\)"), m => "^" + m.captures.at(0).replace("−", "-") + m.captures.at(1))
    (u.replace(regex("[()]"), "").replace(regex("\s*\^\s*"), "^").trim(), e + 1)
  }
  // start of the numeric factor at the end of out ((3450)/(2), 2^3), or none
  let start(out) = {
    let (d, best) = (0, none)
    for i in range(out.len() - 1, -1, step: -1) {
      let k = out.at(i)
      if k == ")" { d += 1 }
      else if k == "(" {
        d -= 1
        if d < 0 or (d == 0 and i > 0 and name(out.at(i - 1))) { break } // f(2) is a call
      }
      else if not (num(k) or k in ("*", "/", "·", "⋅", "×", "^") or (d > 0 and k in ("+", "-", "−"))) { break }
      if d == 0 and (num(k) or k == "(") { best = i }
    }
    best
  }
  let out = ()
  let i = 0
  while i < t.len() {
    let k = t.at(i)
    let b = if (unit(k) or group(i) != none) and out.len() > 0 and (num(out.last()) or out.last() == ")") { start(out) }
    if b == none and bare and (unit(k) or group(i) != none) and out.len() > 0 and (out.last() == ")" or name(out.last())) { b = out.len() }
    if b == none {
      // 3 h with a variable h: a number times the variable, not 3 hours
      out.push(if num(k) and name(t.at(i + 1, default: none)) and t.at(i + 1) in vars { "[" + k + "]" } else { k })
      i += 1
      continue
    }
    let (u, j) = ((), i)
    let go = true
    while go and j < t.len() {
      let a = t.at(j)
      if unit(a) {
        u.push(a); j += 1
        // m^2, s^-1, m^(2) (math writes the exponent in parentheses)
        if t.at(j, default: none) == "^" {
          let ex = t.slice(j + 1, calc.min(j + 5, t.len()))
          let par = ex.at(0, default: none) == "("
          let ex = if par { ex.slice(1) } else { ex }
          let neg = ex.at(0, default: none) in ("-", "−")
          let ex = if neg { ex.slice(1) } else { ex }
          let n = ex.at(0, default: none)
          if n != none and n.match(regex("^\d+$")) != none and (not par or ex.at(1, default: none) == ")") {
            u.last() += "^" + (if neg { "-" } else { "" }) + n
            j += 2 + int(par) * 2 + int(neg)
          }
        }
      } else if group(j) != none { let (g, e) = group(j); u.push(g); j = e }
      else if a == "/" and (unit(t.at(j + 1, default: none)) or group(j + 1) != none) { u.push("/"); j += 1 }
      else { go = false }
    }
    let f = out.slice(b).join()
    out = out.slice(0, b) + ("[" + (if f in (none, "") { "1" } else { f }) + " " + u.join(" ") + "]",)
    i = j
  }
  out.join(" ")
}
// expression tree -> string _parse reads back the same
#let _src(e) = {
  let (k, ..a) = e
  let w(x) = "(" + _src(x) + ")"
  if k == "n" { repr(float(a.first())) }
  else if k in ("c", "v") { a.first() }
  else if k == "q" { let q = a.first(); if q.u == "" { repr(float(q.n)) } else { "[" + repr(float(q.n)) + " " + q.u + "]" } }
  else if k == "+" { a.map(w).join(" + ") }
  else if k == "*" { a.map(w).join(" * ") }
  else if k == "-" { "-" + w(a.first()) }
  else if k in ("/", "^") { w(a.first()) + " " + k + " " + w(a.last()) }
  else if k == "tuple" { a.map(_src).join(", ") }
  else { a.first() + w(a.last()) }
}
// where two steps of a chain differ only by a variable written out (l/A vs l/(b h)): (("A", tree of b h), …), or none
#let _diff(a, b) = {
  if a == b { return () }
  if a.at(0) == "v" and b.at(0) == "v" { return ((a.at(1), b), (b.at(1), a)) }
  if a.at(0) == "v" { return ((a.at(1), b),) }
  if b.at(0) == "v" { return ((b.at(1), a),) }
  if a.at(0) != b.at(0) or a.len() != b.len() { return none }
  let r = ()
  for (x, y) in a.slice(1).zip(b.slice(1)) {
    if type(x) != array or type(y) != array { if x != y { return none }; continue }
    let d = _diff(x, y)
    if d == none { return none }
    r += d
  }
  r
}
// a math equation in a solve-block: "I = 30 mA" is defined silently, the equation stays as written.
// Equations without '=', derivations (⇒, ≈, <, …) or content that isn't a formula (alignment, line breaks, #val) are left alone.
#let _math-def(it) = context {
  if _busy.get() > 0 { return it }
  let s = _str(it.body)
  if _E(s) or s.match(regex("[⇒⇔⟹⟺→←↔≈≠≡∼∝∈]")) != none { return it }
  let rs = s.matches(regex("[<>≤≥]"))
  if rs.len() > 0 {
    // a bound R ≤ U/I: a definition only with exactly one unknown (0 < x < 1, a < b with known a, b stay prose)
    if rs.len() > 1 or s.contains("=") { return it }
    let rel = rs.first().text
    let vars = _vars-of(_st.get())
    let fns = _st.get().at("#fn", default: (:))
    let p = s.split(rel).map(x => _math-src(x, vars).trim())
    if p.any(x => x == "") { return it }
    let es = p.map(x => _parse(x, vars, f: fns))
    if es.any(_E) or es.map(_vars).flatten().dedup().filter(v => v not in vars).len() != 1 { return it }
    return it + _def((p.join(" " + rel + " "),), none, (:), (block: it.block), false, true)
  }
  let p = s.split("=")
  if p.len() < 2 or p.any(x => x.trim() == "") { return it }
  // a worked chain R = 7.2 kΩ = 7200 Ω: the first '=' defines, the rest are steps
  let vars = _vars-of(_st.get())
  let lhs = _math-src(p.at(0), vars).trim()
  let eq = lhs + " = " + _math-src(p.at(1), vars).trim()
  // steps that write an undefined variable out define it: R = ρ l/A = ρ l/(b h) gives A = b h
  let fns = _st.get().at("#fn", default: (:))
  let es = p.slice(1).map(x => _parse(_math-src(x, vars), vars, f: fns, keep: true))
  let imp = ()
  let ds = range(es.len() - 1).map(i => if es.any(_E) { none } else { _diff(es.at(i), es.at(i + 1)) })
  if not es.any(_E) {
    let cs = ds.filter(c => c != none).fold((), (acc, c) => acc + c)
    // only variables that aren't defined, with one consistent expression of other variables
    let cs = cs.filter(((x, e)) => x not in vars and x not in _vars(e) and _vars(e).len() > 0)
    for (x, e) in cs {
      if cs.all(((y, f)) => y != x or f == e) and not cs.any(((y, f)) => f == ("v", x)) and x not in imp.map(c => c.at(0)) { imp.push((x, e)) }
    }
  }
  // implied definitions computable now go first; the main equation is the most written-out step with the fewest unknowns
  // (R = ρ l/(b h) for b), only among steps reached by substitutions (not = 0.2 Ω); the other implied ones follow
  let now = imp.filter(((x, e)) => _vars(e).all(v => v in vars))
  let known = vars.keys() + now.map(c => c.at(0))
  let le = _parse(lhs, vars, f: fns)
  let k = ds.position(c => c == none)
  let k = if k == none { ds.len() } else { k }
  if imp.len() > 0 and not _E(le) {
    let n(i) = (_vars(le) + _vars(es.at(i))).dedup().filter(v => v not in known).len()
    let best = range(k + 1).sorted(key: i => (n(i), -i)).first()
    eq = lhs + " = " + _math-src(p.at(best + 1), vars).trim()
  }
  let one(x, e) = _def((x + " = " + _src(e),), none, (:), (block: it.block), false, true)
  it + now.map(((x, e)) => one(x, e)).join() + _def((eq,), none, (:), (block: it.block), false, true) + imp.filter(c => c not in now).map(((x, e)) => one(x, e)).join()
}
/// A scope in which math and code are calculations.
///
/// Inside:
/// - every math equation with `=` is a definition, like `def`: `$I = "30 mA"$`,
///   `$P = U I$`, `$Q_0/2 = I t$`. The equation stays as written. Numbers followed by
///   units (`frac(3450 "mAh", 30 "mA")`) are values, `0,95` is a decimal number, and in a
///   worked chain `$R = 7.2 "k"Omega = 7200 Omega$` the first `=` defines. Equations without
///   `=`, derivations (`⇒`, `≈`, `<`, …) and equations with alignment or line breaks are
///   left alone.
/// - code without a language (or `solve`) is evaluated: in a block every line shows itself
///   like `solve` (`t = Q0/I -> h`, systems with `;`, comments with `//`); inline
///   `` `t = Q0/I -> h` `` shows `t = 115 h`, `` `= Q0/I -> h` `` just the value.
///
/// Definitions made inside are dropped at the end. How results look is set with
/// `solve-config`, not here.
///
/// ```typ
/// #solve-block[
///   The load draws $I = "30 mA"$ (#solve("I", unit: "A", steps: "value", inline: true)),
///   the battery holds $Q_0 = 3450 "mAh"$, so it lasts $t = Q_0 / I$ = #val("t", unit: "h").
/// ]
/// #show: solve-block                       // the whole rest of the document
/// #show: solve-block.with(reset: heading)   // each section on its own
/// ```
///
/// - ..body (content): The content.
/// - inherit (bool): See the definitions made before the block (also after each `reset`).
/// - equations (bool): Read math equations as definitions. `false` leaves math alone.
/// - reset (none, function, selector): Start over at every match, e.g. `heading` (every heading) or `heading.where(level: 1)`: the definitions made since are forgotten, like `#reset`.
/// -> content
#let solve-block(
  ..body,
  inherit: false, equations: true, reset: none,
) = {
  let b = body.pos().at(0, default: [])
  if body.named().len() > 0 {
    let k = body.named().keys().first()
    return _err-now("unknown argument '" + k + "' for solve-block (arguments: inherit, equations, reset)" + if k in _DEF { " – options go in solve-config(" + k + ": …)" } else { "" }) + b
  }
  if body.pos().len() > 1 { return _err-now("solve-block takes one body") + b }
  if type(b) != content { return _err-now("solve-block: the body must be content, got " + _tn(b)) }
  for (n, v) in (inherit: inherit, equations: equations) { if type(v) != bool { return _err-now(n + ": true or false, got " + _tn(v)) + b } }
  if type(reset) not in (type(none), function, selector, label) { return _err-now("reset: none or what to start over at, like heading or heading.where(level: 1), got " + _tn(reset)) + b }
  // "^": the state outside, restored at the end; "^0": the state at the start, for reset
  _st.update(d => { let n = if inherit { d } else { (:) }; n.insert("^", d); n.insert("^0", n); n })
  context {
    // raw text is set smaller and monospaced; definitions use the surrounding text
    let (size, font) = (text.size, text.font)
    let out(it, sep) = text(size: size, font: font, it.text.split("\n").map(l => _line(l, it.block)).filter(x => x != none).join(sep))
    show raw.where(lang: none): it => out(it, none)
    show raw.where(lang: "solve"): it => out(it, if it.block { none } else { [, ] })
    show std.math.equation: it => if equations { _math-def(it) } else { it }
    // start over at each heading (or whatever reset selects): back to the state at the start of the block
    show selector(if reset == none { <solveng-never> } else { reset }): it => {
      if reset != none { _st.update(_restart) }
      it
    }
    b
  }
  _st.update(d => d.at("^"))
}

/// Lists the definitions made so far, in order.
///
/// ```typ
/// #definitions()                                          // everything, in columns side by side
/// #definitions(included: "given")                         // only plain values
/// #definitions(excluded: ("U", "function"))               // names and kinds
/// #definitions(included: regex("^R"), style: "column")    // one column aligned at '='
/// ```
///
/// `included` / `excluded` take `"all"`, a kind (`"given"`: plain values, `"computed"`,
/// `"function"`), a name, a regex, a function `name => bool` or an array of these;
/// `excluded` wins. (`include` is a Typst keyword, hence the names.)
///
/// - included (str, regex, function, array): What to list. Default `"all"`.
/// - excluded (none, str, regex, function, array): What to leave out.
/// - style (auto, str): `"grid"` (columns side by side, as many as fit, aligned at `=`; in HTML they reflow with the width), `"column"` (one aligned column, in HTML a table), `"inline"` (comma separated) or `"list"` (bullets). `auto`: the option `definitions` (default `"grid"`).
/// - si (auto, bool): Also show the SI value: `Q₀ = 3450 mAh = 12420 C`.
/// - config (dictionary): Other options, e.g. `(digits: 3)`; see `solve-config`.
/// -> content
#let definitions(included: "all", excluded: none, style: auto, si: auto, config: (:)) = _mine(context {
  if type(config) != dictionary { return _errc("config: expected a dictionary of options like (digits: 3), got " + _tn(config), _cf.get(), wide: true) }
  let lc = _check(_given((definitions: style, si: si)) + config)
  if _E(lc) { return _errc(lc.err, _cf.get(), wide: true) }
  let cf0 = _cf.get() + lc
  let st = _st.get()
  let fns = st.at("#fn", default: (:))
  let ok(p) = p == none or type(p) in (str, regex, function) or (type(p) == array and p.all(ok))
  if not ok(included) or not ok(excluded) { return _errc("included/excluded: use \"all\", a kind, names, a regex or a function", cf0, wide: true) }
  let hit(e, p) = if p == none { false }
    else if type(p) == str { if p == "all" { true } else if p in ("given", "computed", "function") { e.kind == p } else { e.name == p } }
    else if type(p) == array { p.any(x => hit(e, x)) }
    else if type(p) == regex { e.name.match(p) != none }
    else { p(e.name) }
  let es = st.at("!", default: ()).filter(e => e.listed and hit(e, included) and not hit(e, excluded) and (e.name in st or e.name in fns))
  if es.len() == 0 { return }
  let rows = es.map(e => {
    let cf = cf0 + e.cf
    if e.kind == "function" {
      let fn = fns.at(e.name)
      ($#_name(e.name) (#fn.params.map(_name).join($, $))$, (_dp(fn.body, none, cf).at(0),), $=$)
    } else {
      let q = st.at(e.name)
      let si = _auto(q.d)
      let sv = (raw: none, u: si, n: q.v / _unit(si).at(0))
      (_name(e.name), if cf.si and q.u != si and _ushow(q.u) != _ushow(si) { (_qc(q, cf), _qc(sv, cf)) } else { (_qc(q, cf),) }, _relc(q, $=$))
    }
  })
  let style = cf0.definitions
  let eqs = rows.map(((nm, vs, r0)) => (nm, vs.enumerate().map(((i, v)) => if i == 0 { $#r0 #v$ } else { $= #v$ }).join()))
  if style == "list" { std.list(..eqs.map(((nm, v)) => $#nm #v$)) }
  else if style == "inline" { eqs.map(((nm, v)) => $#nm #v$).join(", ") }
  else if target() == "html" and style == "column" {
    // a table aligned at '=': name | = | value | = SI value
    _css()
    let td(c, al) = html.elem("td", attrs: (style: "text-align: " + al), c)
    let n = calc.max(..rows.map(r => r.at(1).len()))
    html.elem("table", attrs: (class: "solveng-list"),
      rows.map(((nm, vs, r0)) => html.elem("tr", td($#nm$, "right") + range(n).map(i => if i < vs.len() { td(if i == 0 { $#r0$ } else { $=$ }, "center") + td($#vs.at(i)$, "left") } else { td([], "left") + td([], "left") }).join())).join())
  } else if target() == "html" {
    // CSS columns flow top to bottom and reflow with the width; sizes measured with Typst's fonts, plus slack for the browser's
    _css()
    let em(c) = measure($#c$).width / text.size
    let nw = calc.max(..eqs.map(((nm, _)) => em(nm))) * 1.15 + 0.2
    let vw = calc.max(..eqs.map(((_, v)) => em(v))) * 1.15 + 1
    html.elem("div", attrs: (class: "solveng-defs", style: "--n: " + str(calc.round(nw, digits: 2)) + "em; --w: " + str(calc.round(nw + vw, digits: 2)) + "em"),
      eqs.map(((nm, v)) => html.elem("div", html.elem("span", $#nm$) + html.elem("span", $#v$))).join())
  } else if style == "column" { _eqn(eqs.map(((nm, v)) => $#nm & #v$.body).join(linebreak()), cf0 + (block: true)) }
  else {
    // as many columns as fit, filled top to bottom, each aligned at '='
    block(width: 100%, layout(size => {
      let cells = eqs.map(((nm, v)) => ($#nm$, $#v$))
      let ws = cells.map(((a, b)) => (measure(a).width, measure(b).width))
      let (g, sp) = (2.5em.to-absolute(), 0.3em.to-absolute())
      // widths per column for n columns: (rows, ((name, value), …))
      let fit(n) = {
        let rows = calc.ceil(cells.len() / n)
        let cs = range(calc.ceil(cells.len() / rows)).map(c => ws.slice(c * rows, calc.min((c + 1) * rows, ws.len())))
        (rows, cs.map(c => (calc.max(..c.map(w => w.at(0))), calc.max(..c.map(w => w.at(1))))))
      }
      // the most columns that fit; one column at least
      let width(cw) = cw.map(((a, b)) => a + sp + b).sum() + g * (cw.len() - 1)
      let (rows, cw) = range(cells.len(), 0, step: -1).map(fit).find(((_, cw)) => cw.len() == 1 or width(cw) <= size.width)
      let n = cw.len()
      let out = ()
      for r in range(rows) {
        for c in range(n) {
          let i = c * rows + r
          let (a, b) = cells.at(i, default: ([], []))
          out += (a, b)
          if c < n - 1 { out.push([]) }
        }
      }
      align(center, grid(columns: cw.enumerate().map(((c, w)) => w + if c < n - 1 { (g - 2 * sp,) } else { () }).flatten(), column-gutter: sp, row-gutter: 0.75em, align: (x, _) => (if calc.rem(x, 3) == 0 { right } else { left }) + horizon, ..out))
    }))
  }
})
// the old name
#let definition-list(..args) = _err-now("definition-list is now called definitions(…)" + if "list" in args.named() { ", and its list: argument style: …" } else { "" })

// ---------- plotting (simple-plot) ----------
#let _nice(r) = { let b = calc.pow(10.0, calc.floor(calc.log(r))); for m in (1, 2, 5, 10) { if m * b >= r * (1 - 1e-9) { return m * b } } }
#let _axlabel(n, u) = if u == "" { $#n$ } else { $#n thin slash thin #_ushow(u)$ }
// tick positions + labels with the document's decimal separator
#let _ticks(a, b, st, skip0, cf) = {
  let ps = ()
  let v = calc.ceil(a / st - 1e-9) * st
  while v <= b + st * 1e-9 { if not (skip0 and calc.abs(v) < st * 1e-9) { ps.push(calc.round(v, digits: 12)) }; v += st }
  (ps, ps.map(p => _nc(_fixed(p, 10, false), none, cf)))
}

/// An expression as a plain function for `simple-plot`, in display units. Needs `context`.
///
/// Unlike the other functions, problems stop the compilation here.
///
/// ```typ
/// #import "@preview/simple-plot:1.1.0": plot
/// #context plot(xmin: 0, xmax: 20, ymin: 0, ymax: 10, (fn: curve("U^2/R", "U", xunit: "V", yunit: "W")))
/// ```
///
/// - expr (str, content): The expression, using defined variables and `x`.
/// - x (str): Name of the free variable.
/// - xunit (str, content): Unit of the function's argument, e.g. `"V"`.
/// - yunit (str, content): Unit of the function's value, e.g. `"W"`.
/// -> function
#let curve(expr, x, xunit: "", yunit: "") = {
  // returns a plain function for simple-plot, so problems can only stop the compilation here
  assert(type(expr) in (str, content), message: "solveng: curve: the expression must be a string or math, got " + _tn(expr))
  assert(type(x) == str, message: "solveng: curve: x must be a variable name, got " + _tn(x))
  assert(type(xunit) in (str, content) and type(yunit) in (str, content), message: "solveng: curve: xunit / yunit must be units like \"V\"")
  let (xunit, yunit) = (_str(xunit), _str(yunit))
  let d = _vars-of(_st.get())
  let ex = _str(expr)
  let e = if _E(ex) { ex } else { _parse(ex, d, f: _st.get().at("#fn", default: (:))) }
  let (xu, yu) = (_unit(xunit), _unit(yunit))
  let b = _first-err((e, xu, yu))
  if b != none { panic("solveng: curve: " + b.err) }
  let env = _env(d)
  let (xf, yf) = (xu.at(0), yu.at(0))
  t => { let env = env; env.insert(x, (float(t) * xf, none)); let r = _nd(e, env); if r == none { none } else { r.at(0) / yf } }
}

// bisection on [a, b] where h changes sign (or, with edge: true, where h goes from defined to undefined)
#let _bis(h, a, b, edge: false) = {
  let ha = h(a)
  for _ in range(50) {
    let m = (a + b) / 2
    if m == a or m == b { break }
    let hm = h(m)
    let left = if edge { (hm == none) == (ha == none) } else { hm != none and ha != none and (hm > 0) == (ha > 0) }
    if left { (a, ha) = (m, hm) } else { b = m }
  }
  (a + b) / 2
}
// x positions worth showing: roots, extrema, poles, edges of the domain, intersections -> (points, domain edges)
#let _interesting(cs, env, x, s) = {
  let F(e, t) = { let env = env; env.insert(x, (float(t), (1.0,))); _nd(e, env) }
  let ks = range(-48, 49).map(k => s * calc.pow(10.0, k / 8))
  let ts = ks.rev().map(t => -t) + (0.0,) + ks
  let (pts, edges, xs) = ((), (), ())
  // where h crosses zero: sign changes, refined, and isolated exact zeros (exp underflowing to 0 isn't one)
  let cross(h) = {
    let vs = ts.map(h)
    let r = ()
    for i in range(ts.len()) {
      let u = vs.at(i)
      if u == none { continue }
      if u == 0 { if i > 0 and i < ts.len() - 1 and vs.at(i - 1) not in (none, 0) and vs.at(i + 1) not in (none, 0) { r.push(ts.at(i)) }; continue }
      let w = vs.at(i + 1, default: none)
      if w != none and w != 0 and (u > 0) != (w > 0) { r.push(_bis(h, ts.at(i), ts.at(i + 1))) }
    }
    r
  }
  for e in cs {
    let f(t) = { let r = F(e, t); if r == none { none } else { r.at(0) } }
    let g(t) = { let r = F(e, t); if r == none or r.at(1) == none { none } else { r.at(1).first() } }
    pts += cross(f) + cross(g) // roots and poles, extrema
    for i in range(ts.len() - 1) {
      let (u, w) = (f(ts.at(i)), f(ts.at(i + 1)))
      // overflow (exp of a big number) isn't an edge of the domain
      if (u == none) != (w == none) and calc.abs(if u == none { w } else { u }) < 1e30 { edges.push(_bis(f, ts.at(i), ts.at(i + 1), edge: true)) }
    }
  }
  for (i, a) in cs.enumerate() {
    for b in cs.slice(i + 1) {
      let h(t) = { let (u, w) = (F(a, t), F(b, t)); if u == none or w == none { none } else { u.at(0) - w.at(0) } }
      xs += cross(h)
    }
  }
  (pts + xs + edges, edges, xs)
}
// automatic x range: the cluster of interesting points nearest 0 (periodic curves have them everywhere), with some room
#let _autorange(cs, env, x, s) = {
  let (pts, edges, xs) = _interesting(cs, env, x, s)
  // where curves cross is what the graph is about: only what lies near the crossings
  let pts = if xs.len() == 0 { pts } else { let w = 2 * calc.max(..xs.map(calc.abs)); pts.filter(t => calc.abs(t) <= w) }
  let pts = pts.map(t => if calc.abs(t) < 1e-9 * s { 0.0 } else { t })
  // distances from 0, outward while each is at most 10× the previous, 4 at most (sin has them everywhere)
  let ds = ()
  for a in pts.filter(t => t != 0).map(calc.abs).sorted() {
    if ds.len() > 0 and a <= ds.last() * (1 + 1e-6) { continue }
    if ds.len() >= 4 or (ds.len() > 0 and a > 10 * ds.last()) { break }
    ds.push(a)
  }
  let m = if ds.len() == 0 { 0 } else { ds.last() }
  let core = pts.filter(t => calc.abs(t) <= m * (1 + 1e-6))
  if core.len() == 0 { return (-s, s) }
  let (lo, hi) = (calc.min(..core), calc.max(..core))
  let (a, b) = if lo == hi { let w = if lo == 0 { s } else { calc.abs(lo) }; (lo - w, lo + w) } else { let p = (hi - lo) * 0.2; (lo - p, hi + p) }
  // physical ranges start at 0 when everything interesting is positive and not far from it
  if lo > 0 and lo < 0.5 * (hi - lo) { a = 0.0 }
  if hi < 0 and -hi < 0.5 * (hi - lo) { b = 0.0 }
  // don't run past the edge of the domain (sqrt, ln)
  let ok(t) = cs.any(e => { let env = env; env.insert(x, (float(t), none)); _nd(e, env) != none })
  for t in edges.sorted() { if t <= lo and t > a and not ok(t - (t - a) / 2) { a = t } }
  for t in edges.sorted().rev() { if t >= hi and t < b and not ok(t + (b - t) / 2) { b = t } }
  // round to a tick step, but not past a domain edge
  let st = _nice((b - a) / 5)
  let (a2, b2) = (calc.floor(a / st + 1e-9) * st, calc.ceil(b / st - 1e-9) * st)
  (if ok(a2) or a2 == a { a2 } else { a }, if ok(b2) or b2 == b { b2 } else { b })
}

/// Plots an equation with `simple-plot`: ranges, nice ticks, axis labels and units are picked automatically.
///
/// Every positional string or math is a curve: `y = …` gives one curve (named `y` in the
/// legend) and marks the stored point with guide lines; other equations draw both sides
/// and mark the intersections. `xrange:` / `yrange:` list values that must be visible;
/// without them the x range is picked around the marks, else around the interesting part
/// (roots, extrema, poles, intersections). Marked values are always visible too.
///
/// ```typ
/// #def("R = 50 Ω", "P0 = 2 W")
/// #solve("P0 = U^2/R")
/// #graph("P = U^2/R", x: "U")                                // 0…2·U, the solved point marked
/// #graph("P = U^2/R", x: "R", xrange: ("10 Ω", "1 kΩ"))
/// #graph("cos(w) = w")                                       // range picked, intersection marked
/// #graph("y = x^2", "y = 2x + 3", xrange: (-3, 5))           // several curves
/// #graph("P = U^2/R", x: "R", xrange: (0, "2 R"), mark: ("R", "extrema"))
/// #graph("P = U^2/R", x: "R", ymark: "P0")                  // where P reaches P0
/// ```
///
/// - ..args (str, content, any): The curves (equations or expressions) and options passed on to simple-plot's `plot` (e.g. `show-grid: false`); other positional ones are added as series.
/// - x (auto, str): The x variable. `auto`: the only variable that isn't defined.
/// - xrange (auto, array, str, content, int, float): x values that must be visible: numbers in `xunit` (`(1, 2)`), values (`("10 Ω", "1 kΩ")`) or expressions of defined variables (`(0, "2 R")`, math works too). Two or more give the range from the smallest to the largest; a single one widens the picked range. `auto`: 0…2·x if x is defined, else around the marked x values, else picked from the curves.
/// - yrange (auto, array, str, content, int, float): y values that must be visible, like `xrange` (numbers in `yunit`). Two or more give the exact y range. `auto`: fitted to the curves.
/// - xunit (auto, str, content): Unit of the x axis. `auto`: from the range or the variable.
/// - yunit (auto, str, content): Unit of the y axis. `auto`: SI unit with a fitting prefix.
/// - mark (auto, none, array, str, content, int, float): What to mark, with both coordinates written on the axes: x values like `xrange` (`"R"`, `2`, `"5 V"`; a point with guide lines on every curve, the range grows to show it), `"crossings"`, `"roots"`, `"extrema"` (searched within the range) and `auto`: the solved point or where the two sides cross. An array combines them: `(auto, "R1")`. `()` or `none`: no marks.
/// - xmark (none, array, str, content, int, float): Like `mark`, but only the x value is written on the axis.
/// - ymark (none, array, str, content, int, float): y values to mark, like `yrange`: every point where a curve reaches one, with guide lines; the value is written on the y axis.
/// - samples (int): Number of sample points.
/// - width (int, float, length, ratio, relative): A number: the plot area in cm (default 8). A length (`12cm`, `300pt`) or share (`100%`): the whole graph, labels included.
/// - height (int, float, length, ratio, relative): Like `width` (default 5). A share is of the container's height, or of the width when the page grows with its content.
/// - colors (array, color): Colors of the curves, e.g. `(gray, blue, rgb("#f98eff"))`.
/// - config (dictionary): Options, e.g. `(sep: ",")`; see `solve-config`.
/// -> content
#let graph(..args, x: auto, xrange: auto, yrange: auto, xunit: auto, yunit: auto, mark: auto, xmark: (), ymark: (), samples: 400, width: 8, height: 5, colors: (blue, red.darken(10%), green.darken(25%), orange.darken(10%)), config: (:)) = _mine(context {
  let lc = _check(config)
  if _E(lc) { return _errc("graph: " + lc.err, _cf.get(), wide: true) }
  let cf = _cf.get() + lc
  let fail(m) = _errc("graph: " + m, cf, wide: true)
  // types
  let eqs = args.pos().filter(a => type(a) in (str, content))
  if args.pos().any(a => type(a) == array) { return fail("the x range is named: xrange: " + repr(args.pos().find(a => type(a) == array))) }
  let args = arguments(..args.pos().filter(a => type(a) not in (str, content)), ..args.named())
  if eqs.len() == 0 { return fail("nothing to plot – pass an equation like \"P = U^2/R\"") }
  for (k, n) in (from: "xrange: (1, 2)", to: "xrange: (1, 2)", range: "xrange") { if k in args.named() { return fail(k + ": is now " + n) } }
  let XV = (str, content, int, float)
  let list(v) = if v == auto or v == none { () } else if type(v) == array { v } else { (v,) }
  let (rng, yrng) = (list(xrange), list(yrange))
  for (n, r) in (xrange: rng, yrange: yrng) { if r.any(v => type(v) not in XV) { return fail(n + ": values to show, like (1, 2), (\"10 Ω\", \"1 kΩ\") or (0, \"2 R\")") } }
  let ymk = list(ymark)
  if ymk.any(v => type(v) not in XV) { return fail("ymark: y values like \"P0\" or 2, or an array of these") }
  // marks: x values, keywords, auto; (item, y labelled): mark writes x and y on the axes, xmark only x
  let KW = ("crossings", "roots", "extrema")
  let ms(v) = if v == none or v == false { () } else if v == true { (auto,) } else if type(v) == array { v } else { (v,) }
  let ents = ms(mark).map(m => (m, true)) + ms(xmark).map(m => (m, false))
  if ents.any(((m, _)) => m != auto and type(m) not in XV) { return fail("mark / xmark: auto, x values like \"R\" or 2, \"crossings\", \"roots\", \"extrema\", or an array of these; () for none") }
  let iskw(m) = type(m) == str and m in KW
  let kws = ents.filter(((m, _)) => iskw(m))
  let autos = ents.filter(((m, _)) => m == auto).map(e => e.at(1))
  let mxs = ents.filter(((m, _)) => m != auto and not iskw(m))
  if x != auto and type(x) != str { return fail("x: a variable name like \"U\", got " + _tn(x)) }
  for (n, v) in (xunit: xunit, yunit: yunit) { if v != auto and type(v) not in (str, content) { return fail(n + ": a unit like \"V\", got " + _tn(v)) } }
  if type(samples) != int or samples < 2 { return fail("samples: an integer ≥ 2") }
  for (n, v) in (width: width, height: height) {
    let ok = if type(v) in (int, float) { v > 0 } else if type(v) == length { v.to-absolute() > 0pt } else if type(v) == ratio { v > 0% } else if type(v) == relative { v.ratio >= 0% and v.length.to-absolute() >= -1e9pt } else { false }
    if not ok { return fail(n + ": a number in cm, a length like 50pt or 8cm, or a share like 100%, got " + repr(v)) }
  }
  let colors = if type(colors) == color { (colors,) } else { colors }
  if type(colors) == content { return fail("colors: square brackets make content – write an array with parentheses: (gray, blue, rgb(\"#f98eff\"))") }
  if type(colors) != array or colors.len() == 0 or colors.any(c => type(c) not in (color, gradient)) { return fail("colors: a color or an array of colors like (gray, blue, rgb(\"#f98eff\"))") }
  let (xunit, yunit) = (xunit, yunit).map(v => if type(v) == content { _str(v) } else { v })
  let b = _first-err((xunit, yunit))
  if b != none { return fail(b.err) }
  for v in (xunit, yunit) { if type(v) == str { let u = _unit(v); if _E(u) { return fail(u.err) } } }
  let d = _vars-of(_st.get())
  let fns = _st.get().at("#fn", default: (:))
  let env = _env(d)
  let D(e) = _dp(e, none, cf).at(0)
  // each equation: (source, left, right or none)
  let ps = ()
  for q in eqs {
    let s = _str(q)
    if _E(s) { return fail(s.err) }
    let sides = s.split("=")
    if sides.len() > 2 { return fail("more than one '=' in '" + s + "'") }
    let L = _parse(sides.at(0), d, f: fns)
    let R = if sides.len() > 1 { _parse(sides.at(1), d, f: fns) }
    let b = _first-err((L, R))
    if b != none { return fail(b.err) }
    ps.push((s, L, R))
  }
  let eq = ps.map(p => p.at(0)).join("', '")
  let x = if x != auto { x } else {
    let ys = ps.filter(((_, L, R)) => R != none and L.at(0) == "v").map(p => p.at(1).at(1))
    let u = ps.map(((_, L, R)) => _vars(L) + if R != none { _vars(R) } else { () }).flatten().dedup().filter(n => n not in d and n not in ys)
    if u.len() != 1 { return fail("can't tell the x variable in '" + eq + "', pass x: \"…\"") }
    u.first()
  }
  // curves with their legend labels; pairs whose crossings are marked
  let (curves, labels, pairs) = ((), (), ())
  let ns = ps.filter(((_, L, R)) => R != none and L.at(0) == "v" and L.at(1) != x).map(p => p.at(1).at(1))
  for (_, L, R) in ps {
    if R == none { curves.push(L); labels.push(D(L)) }
    else if L.at(0) == "v" and L.at(1) != x {
      // y = x², y = 2x + 3: the same name twice, so the legend shows the equations
      curves.push(R); labels.push(if ns.filter(n => n == L.at(1)).len() > 1 { $#_name(L.at(1)) = #D(R)$ } else { _name(L.at(1)) })
    }
    else { pairs.push((curves.len(), curves.len() + 1)); curves += (L, R); labels += (D(L), D(R)) }
  }
  if curves.len() == 2 and pairs.len() == 0 { pairs.push((0, 1)) }
  let named = ps.filter(((_, L, R)) => R != none and L.at(0) == "v" and L.at(1) != x).map(p => p.at(1).at(1))
  let yname = if named.len() == ps.len() { named.dedup().join(", ") }
  let (L, R) = (ps.first().at(1), ps.first().at(2))
  // x range and unit
  // x values of range and mark: plain numbers (in xunit, known later) or (v, dims, unit) of a value or expression
  let xv(a) = {
    if type(a) in (int, float) { return float(a) }
    let s = _str(a)
    if _E(s) { return s }
    // "10" is a plain number like 10: in the axis unit
    if s.trim().match(regex("^[-+−]?\d+(?:[.,]\d+)?(?:[eE][-+]?\d+)?$")) != none { return float(s.trim().replace("−", "-").replace(",", ".")) }
    if _isq(s) { let q = _q(s); return if _E(q) { q } else { (q.v, q.d, q.u, s.trim()) } }
    let e = _parse(_math-src(s, d, bare: true), d, f: fns)
    if _E(e) { return e }
    let r = _ev(e, d)
    if _E(r) { return _err(r.err + " in '" + s.trim() + "'") }
    let a = _auto(r.at(1))
    (r.at(0), r.at(1), if r.at(1) == _Z { "" } else { _pre(r.at(0) / _unit(a).at(0), a, cf) }, s.trim())
  }
  // a value whose unit doesn't fit its axis: "'f(20)' is 40 (no unit), but x is in Ω"
  let misfit(v, axis, u) = {
    let (val, dm, vu, src) = v
    let shown = if vu == "" { str(calc.round(val, digits: 4)) + " (no unit)" } else { str(calc.round(val / _unit(vu).at(0), digits: 4)) + " " + vu }
    "'" + src + "' is " + shown + ", but " + axis + (if u == "" { " has no unit" } else { " is in " + u }) + " – plain numbers count in the axis unit"
  }
  let rvs = rng.map(xv)
  let mvs = mxs.map(e => xv(e.at(0)))
  let mly = mxs.map(e => e.at(1))
  let (yrv, ymv) = (yrng.map(xv), ymk.map(xv))
  let b = _first-err(rvs + mvs + yrv + ymv)
  if b != none { return fail(b.err) }
  let withu = (rvs + mvs).filter(v => type(v) == array and v.at(1) != _Z)
  let xunit = if xunit != auto { xunit } else if x in d { d.at(x).u } else if withu.len() > 0 { withu.first().at(2) } else { "" }
  let (xf, xd) = _unit(xunit)
  if x in d { xd = d.at(x).d } else if xunit == "" and withu.len() > 0 { xd = withu.first().at(1) }
  // no unit for x yet, but y values have one: x's unit follows from the curve (P = U0²/R2 and P0 in W: R2 in Ω)
  let yu = (yrv + ymv).find(v => type(v) == array and v.at(1) != _Z)
  if xunit == "" and x not in d and withu.len() == 0 and yu != none {
    let r = _dx(curves.first(), d, env, (x,))
    if not _E(r) and r.at(0) != none and r.at(0).at(1).first() != 0 {
      let (c, (k,)) = r.at(0)
      let dm = yu.at(1).zip(c).map(((y, z)) => calc.round((y - z) / k, digits: 6))
      if dm != _Z { xd = dm; xunit = _auto(dm); xf = _unit(xunit).at(0) }
    }
  }
  let bad = (rvs + mvs).find(v => type(v) == array and v.at(1) != xd)
  if bad != none { return fail(misfit(bad, x, xunit)) }
  let si(v) = if type(v) == float { v * xf } else { v.at(0) }
  let (rvs, mvs) = (rvs.map(si), mvs.map(si))
  // y unit from the units of the expression
  let dq = d + ((x): (v: xf, d: xd, raw: none, u: xunit, ex: none))
  let yr = _dx(curves.first(), dq, env, ())
  if _E(yr) { return fail(yr.err) }
  let yd = { let r = yr.at(0); if r == none { _Z } else { r.at(0).map(z => calc.round(z, digits: 6)) } }
  for (i, e) in curves.enumerate().slice(1) {
    let r = _dx(e, dq, env, ())
    let di = if _E(r) or r.at(0) == none { none } else { r.at(0).at(0).map(z => calc.round(z, digits: 6)) }
    if di != none and di != yd and yd != _Z { return fail("the curves have different units (" + _auto(yd) + " and " + (if di == _Z { "1" } else { _auto(di) }) + ") – plot them separately") }
  }
  let bad = (yrv + ymv).find(v => type(v) == array and v.at(1) != yd)
  if bad != none { return fail(misfit(bad, if yname != none { yname } else { "y" }, if yd == _Z { "" } else { _auto(yd) })) }
  // plain numbers for y are in yunit, or in the SI unit
  let yb = _unit(if yunit != auto { yunit } else { _auto(yd) }).at(0)
  let ysi(v) = if type(v) == float { v * yb } else { v.at(0) }
  let (yrv, ymv) = (yrv.map(ysi), ymv.map(ysi))
  // where the curves reach the y marks: the nearest ones must be visible
  let yneed = ()
  for Y in ymv {
    for e in curves {
      let rs = _scan(e, ("n", Y), env, x, xf).sorted(key: calc.abs)
      if rs.len() > 0 { yneed += rs.filter(t => calc.abs(t) <= 2 * calc.abs(rs.first()) + 1e-9 * xf).slice(0, calc.min(4, rs.len())) }
    }
  }
  let need = mvs + yneed
  let (xa, xb) = if rvs.dedup().len() >= 2 { (calc.min(..rvs), calc.max(..rvs)) } else if x in d {
    let v = d.at(x).v
    if v > 0 { (0.0, 2 * v) } else if v < 0 { (2 * v, 0.0) } else { (-xf, xf) }
  } else if need.len() > 0 {
    // the marks are what's interesting: around them, not back to 0
    let (lo, hi) = (calc.min(..need), calc.max(..need))
    let (a, b) = if lo == hi { let w = if lo == 0 { xf } else { calc.abs(lo) / 2 }; (lo - w, lo + w) } else { let p = (hi - lo) * 0.25; (lo - p, hi + p) }
    let st = _nice((b - a) / 5)
    (calc.floor(a / st + 1e-9) * st, calc.ceil(b / st - 1e-9) * st)
  } else { _autorange(curves, env, x, xf) }
  // a single range value and marked values widen the range; marks get a little room
  for v in rvs { (xa, xb) = (calc.min(xa, v), calc.max(xb, v)) }
  let w = xb - xa
  for v in need { if v < xa { xa = v - 0.05 * w } else if v > xb { xb = v + 0.05 * w } }
  if not (xa < xb) { return fail("the range is empty – give two different x values") }
  let F(e, t) = { let env = env; env.insert(x, (t, none)); let r = _nd(e, env); if r == none { none } else { r.at(0) } }
  let ts = range(samples + 1).map(i => xa + i * (xb - xa) / samples)
  let ys = curves.map(e => ts.map(t => F(e, t)))
  let all = ys.flatten().filter(y => y != none).sorted()
  if all.len() == 0 { return fail("'" + eq + "' can't be evaluated in this range") }
  let (lo, hi) = (all.first(), all.last())
  let (q1, q2) = (all.at(int(all.len() * 0.05)), all.at(int(all.len() * 0.95)))
  if hi - lo > 20 * (q2 - q1) and q2 > q1 { (lo, hi) = (q1 - (q2 - q1) / 2, q2 + (q2 - q1) / 2) } // poles
  // y marks are what's interesting: the y range is around them, steep curves leave the picture
  if ymv.len() > 0 and yrv.dedup().len() < 2 {
    let vs = ymv + yrv
    (lo, hi) = (calc.min(..vs), calc.max(..vs))
    if lo == hi { let w = if lo == 0 { yb } else { calc.abs(lo) }; (lo, hi) = (lo - w, lo + w) } else { let p = (hi - lo) * 0.25; (lo, hi) = (lo - p, hi + p) }
  }
  let yunit = if yunit != auto { yunit } else { let a = _auto(yd); _pre(calc.max(calc.abs(lo), calc.abs(hi)) / _unit(a).at(0), a, cf) }
  let yf = _unit(yunit).at(0)
  // display ranges and ticks
  let (x0, x1) = (xa / xf, xb / xf)
  let (y0, y1) = if yrv.dedup().len() >= 2 { (calc.min(..yrv) / yf, calc.max(..yrv) / yf) } else {
    for v in yrv + ymv { (lo, hi) = (calc.min(lo, v), calc.max(hi, v)) }
    let (y0, y1) = (lo / yf, hi / yf)
    if y0 > 0 and y0 < 0.5 * y1 { y0 = 0 }
    if y1 < 0 and y1 > 0.5 * y0 { y1 = 0 }
    if y0 == y1 { (y0, y1) = (y0 - 1, y1 + 1) }
    let pad = (y1 - y0) * 0.08
    (if y0 == 0 { 0 } else { y0 - pad }, if y1 == 0 { 0 } else { y1 + pad })
  }
  // marked y values beyond an exact y range still show
  for v in ymv { let w = y1 - y0; if v / yf < y0 { y0 = v / yf - 0.05 * w } else if v / yf > y1 { y1 = v / yf + 0.05 * w } }
  let (xs, ys2) = (_nice((x1 - x0) / 5), _nice((y1 - y0) / 5))
  let xc = x0 <= 0 and 0 <= x1
  let yc = y0 <= 0 and 0 <= y1
  let ax = if yc { 0 } else { y0 }
  let xt = _ticks(x0, x1, xs, xc and yc, cf)
  let yt = _ticks(y0, y1, ys2, xc and yc, cf)
  let ay = if xc { 0 } else { x0 }
  // own sampling (finer, split at gaps and poles) drawn as point lines
  let ser = ()
  let used = () // label spots taken, (x, y) in display units
  for (i, e) in curves.enumerate() {
    let runs = ((),)
    for (j, t) in ts.enumerate() {
      let v = ys.at(i).at(j)
      if v == none { if runs.last() != () { runs.push(()) }; continue }
      let p = (t / xf, v / yf, j)
      let q = runs.last().at(-1, default: none)
      if q != none and (q.at(1) > 0) != (p.at(1) > 0) and calc.abs(q.at(1) - p.at(1)) > 2 * (y1 - y0) { runs.push(()) }
      runs.last().push(p)
    }
    let runs = runs.filter(r => r.len() > 1)
    let longest = runs.map(r => r.len()).fold(0, calc.max)
    for r in runs {
      let lab = curves.len() > 1 and r.len() == longest
      // the label goes where this curve is farthest from the others (not at a crossing), away from other labels
      let lp = 0.8
      if lab {
        let (best, bd) = (none, -1)
        for (k, (px, py, j)) in r.enumerate().rev() {
          if k < r.len() * 0.15 or k > r.len() * 0.95 or py < y0 or py > y1 { continue }
          if used.any(((ux, uy)) => calc.abs(ux - px) < 0.2 * (x1 - x0) and calc.abs(uy - py) < 0.15 * (y1 - y0)) { continue }
          let ds = range(curves.len()).filter(c => c != i).map(c => { let w = ys.at(c).at(j); if w == none { 1.0 } else { calc.abs(w / yf - py) / (y1 - y0) } })
          let dd = calc.min(1.0, ..ds)
          if dd > bd + 0.02 { (best, bd) = (k, dd) }
        }
        if best != none { lp = best / (r.len() - 1); used.push((r.at(best).at(0), r.at(best).at(1))) }
      }
      ser.push(_sp.line-plot(r.map(p => p.slice(0, 2)), mark: "none", stroke: colors.at(calc.rem(i, colors.len())) + 1.2pt,
        label: if lab { $#labels.at(i)$ }, label-pos: lp))
    }
  }
  // marks
  let dash = (paint: luma(110), thickness: 0.6pt, dash: "dashed")
  // points (x, y, guides, x labelled, y labelled) in SI; guides: "xy" both guide lines, "x" only the vertical one, "" none
  let pts = ()
  // where h changes sign within the range
  let zeros(h) = {
    let vs = ts.map(h)
    let r = ()
    for i in range(samples) {
      let (u, w) = (vs.at(i), vs.at(i + 1))
      if u == none or w == none { continue }
      if u == 0 { r.push(ts.at(i)) } else if w != 0 and (u > 0) != (w > 0) { r.push(_bis(h, ts.at(i), ts.at(i + 1))) }
    }
    if vs.last() == 0 { r.push(ts.last()) }
    r
  }
  let crossings(prs, ly) = {
    let r = ()
    for (ci, cj) in prs {
      let (A, B) = (curves.at(ci), curves.at(cj))
      r += zeros(t => { let (a, b) = (F(A, t), F(B, t)); if a == none or b == none { none } else { a - b } }).map(t => (t, F(A, t), if ly { "xy" } else { "x" }, true, ly))
    }
    r
  }
  for ly in autos {
    if curves.len() == 1 and yname != none and x in d { mvs.push(d.at(x).v); mly.push(ly) }
    pts += crossings(pairs, ly)
  }
  for (t, ly) in mvs.zip(mly) { for e in curves { pts.push((t, F(e, t), if ly or curves.len() == 1 { "xy" } else { "x" }, true, ly)) } }
  for Y in ymv { for e in curves { pts += zeros(t => { let v = F(e, t); if v == none { none } else { v - Y } }).map(t => (t, Y, "xy", false, true)) } }
  let kw(k) = kws.filter(((m, _)) => m == k).map(e => e.at(1))
  for ly in kw("crossings") { pts += crossings(range(curves.len()).map(i => range(i + 1, curves.len()).map(j => (i, j))).flatten().chunks(2), ly) }
  if kw("roots").len() > 0 { for e in curves { pts += zeros(t => F(e, t)).map(t => (t, 0.0, "", false, false)) } }
  if kw("extrema").len() > 0 {
    for e in curves {
      let g(t) = { let env = env; env.insert(x, (t, (1.0,))); let r = _nd(e, env); if r == none or r.at(1) == none { none } else { r.at(1).first() } }
      pts += zeros(g).map(t => (t, F(e, t), "x", false, false))
    }
  }
  let seen = ()
  let (labx, laby) = ((), ())
  for (t, v, gl, lx, ly) in pts {
    if lx and t / xf >= x0 and t / xf <= x1 and not labx.any(a => calc.abs(a - t / xf) <= 1e-9 * (x1 - x0)) { labx.push(t / xf) }
    if ly and v != none and v / yf >= y0 and v / yf <= y1 and t / xf >= x0 and t / xf <= x1 and not laby.any(a => calc.abs(a - v / yf) <= 1e-9 * (y1 - y0)) { laby.push(v / yf) }
    if v == none { continue }
    let (px, py) = (t / xf, v / yf)
    if px < x0 or px > x1 or py < y0 or py > y1 or seen.any(((a, b)) => calc.abs(a - px) <= 1e-9 * (x1 - x0) and calc.abs(b - py) <= 1e-9 * (y1 - y0)) { continue }
    seen.push((px, py))
    if gl != "" { ser.push(_sp.vline(px, ymin: calc.min(ax, py), ymax: calc.max(ax, py), stroke: dash)) }
    if gl == "xy" { ser.push(_sp.hline(py, xmin: calc.min(ay, px), xmax: calc.max(ay, px), stroke: dash)) }
    ser.push(_sp.scatter(((px, py),), mark-fill: black, mark-stroke: black, mark-size: 0.08))
  }
  // marked values are written on the axes; regular ticks too close to them make room
  let relabel(tk, ls, w) = if ls.len() == 0 { tk } else {
    // mark labels too close to an earlier one are left out
    let ls = ls.fold((), (acc, a) => if acc.any(b => calc.abs(a - b) < 0.06 * w) { acc } else { acc + (a,) })
    let keep = tk.at(0).zip(tk.at(1)).filter(((p, _)) => ls.all(a => calc.abs(a - p) > 0.08 * w))
    let all = (keep + ls.map(a => (a, _nshow((n: a, raw: none), cf).at(0)))).sorted(key: p => p.at(0))
    (all.map(p => p.at(0)), all.map(p => p.at(1)))
  }
  let xt = relabel(xt, labx, x1 - x0)
  let yt = relabel(yt, laby, y1 - y0)
  let o = (
    xmin: x0, xmax: x1, ymin: y0, ymax: y1, width: width, height: height,
    xtick: xt.at(0), xtick-labels: xt.at(1), ytick: yt.at(0), ytick-labels: yt.at(1), show-grid: "major",
    axis-x-pos: if yc { 0 } else { "bottom" }, axis-y-pos: if xc { 0 } else { "left" }, show-origin: xc and yc,
    xlabel: _axlabel(_name(x), xunit),
    ylabel: _axlabel(if yname != none { named.dedup().map(_name).join($, $) } else if curves.len() == 1 { D(curves.first()) } else { $$ }, yunit),
  ) + args.named()
  let make(w, h) = _sp.plot(..o + (width: w, height: h), ..ser, ..args.pos())
  // numbers are the plot area in cm; lengths and shares (100%, 50pt) the whole graph, labels included
  let sized = layout(size => {
    let num(v) = type(v) in (int, float)
    let abs(v, full) = if type(v) == ratio { v * full } else if type(v) == relative { v.ratio * full + v.length.to-absolute() } else { v.to-absolute() }
    if num(width) and num(height) { return make(width, height) }
    // HTML has no width to take a share of: a nominal 16cm, the SVG scales with the page anyway
    let full-w = if calc.abs(size.width.pt()) < 1e6 { size.width } else { 16cm }
    let W = if num(width) { width * 1cm } else { abs(width, full-w) }
    // a share of the height: of the container, or of the width when the page grows with its content
    let full-h = if calc.abs(size.height.pt()) < 1e6 { size.height } else { W }
    let H = if num(height) { height * 1cm } else { abs(height, full-h) }
    let m = measure(make(W, H))
    let w1 = if num(width) { W } else { calc.max(W - (m.width - W), 1cm) }
    let h1 = if num(height) { H } else { calc.max(H - (m.height - H), 1cm) }
    make(w1, h1)
  })
  // HTML has no layout of its own for this: the graph becomes an inline SVG
  if target() == "html" { _css(); html.elem("div", attrs: (class: "solveng-plot"), html.frame(sized)) } else { align(center, sized) }
})
