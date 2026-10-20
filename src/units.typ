// Units: names, prefixes, parsing and picking a readable unit.
#import "config.typ": *

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
