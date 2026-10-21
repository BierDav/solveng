// solve-config, reset and the solver behind def and solve.
#import "read.typ": *

// ---------- public API ----------

/// Sets options for number formatting, units and output.
///
/// Without a body the options apply from here on; with a body only inside it. Every option defaults to `auto`, which keeps the current setting, so only what you pass changes. Every option except `units` can also be passed directly to `def`, `solve`, `defined` and `graph`, for that call only. Unknown options and values of the wrong type are shown as an error in the document.
///
/// ```typ
/// #show: solve-config.with(digits: 3, form: "fraction")   // your defaults: at the top of the document
/// #solve-config(layout: "column")                          // from here on, definitions are kept
/// #solve-config(steps: "result")[ … ]                      // only inside the brackets
/// #def("U = 5 V", "I = 30 mA")
/// #solve("R = U/I", digits: 2)                             // only this call
/// #let opts = (digits: 2, separator: ",")
/// #solve("P = U I", ..opts)                                // options kept in a dictionary
/// ```
///
/// `form`:
///
/// - `"auto"`: plain numbers as typed (`7.20 kΩ` keeps its zero), arithmetic evaluated (`3450/2 mAh` → 1725 mAh), results rounded to `digits`.
/// - `"decimal"`: everything rounded to `digits`, including what you typed.
/// - `"fraction"`: exact fractions, simplified (`$frac(2, 4)"A"$` → ½ A, `I/3` → ⅙ A); values that are no fraction with denominator ≤ `max-denominator` fall back to decimals.
/// - `"input"`: given values exactly as typed (`3450/2 mAh` stays a fraction).
///
/// An explicit unit (`#solve(eq, unit: "mA")`) always wins, then `preferred-units`, then the non-SI unit of the inputs (h, mAh, bar), then the SI unit with a prefix.
///
/// Typst only allows `#set` for its built-in elements, not for functions of packages, so `#set solve(steps: "result")` isn't possible (yet). Instead, `#let solve = solve.with(steps: "result")` changes your `solve` calls, `#show: solve-config.with(steps: "result")` everything, including code in a solve-block. To reuse your defaults in many documents, keep a small file of your own:
///
/// ```typ
/// // my-defaults.typ
/// #import "@preview/solveng:0.2.0": *
/// #let setup = solve-config.with(digits: 3, multiplication: "times", layout: "column")
/// #let solve = solve.with(steps: "formula")
///
/// // in a document
/// #import "my-defaults.typ": *
/// #show: setup
/// ```
///
/// **Calculating without a solve-block.** Outside a `solve-block`, `integration` other than `"none"` makes the content a calculating scope of its own, whose definitions stay: `#solve-config(integration: "full")[…]` calculates its brackets, `#show: solve-config.with(integration: "math")` the rest of the document. Without content (`#solve-config(integration: "full")`) it only sets the option for the solve-blocks that follow.
///
/// ```typ
/// #solve-config(integration: "full")[
///   $Delta_R = 5 "cm"$
///   #solve($Delta_R$)
/// ]
/// ```
///
/// - ..body (content): Optional: the content the options apply to.
/// - digits (auto, int): Significant digits of computed values, 1…15. Default `4`: `57.5 h`, `123500`.
/// - trailing-zeros (auto, bool): Keep trailing zeros up to `digits` (`2.500 V`). Default `false`.
/// - separator (auto, str): Decimal separator, any string like `","`. `auto` (default): from the document language, `,` for de, fr, it, … and `.` otherwise.
/// - grouping (auto, none, str, symbol, content): Digit grouping for integer parts with 5+ digits, e.g. `sym.space.thin` → `12 345`. Default `none`.
/// - scientific (auto, none, array): `(low, high)`: plain notation while the exponent `e` is in `low ≤ e < high`, else `2.5 · 10⁻⁷`. Default `(-3, 6)`; `none` never switches.
/// - engineering (auto, bool): Engineering notation, exponents in steps of 3. Default `false`.
/// - multiplication (auto, str): `"dot"` (default, `2 · 10⁶`) or `"times"` (`2 × 10⁶`).
/// - form (auto, str): How values are written. `"auto"` (default): as typed, results rounded · `"decimal"`: everything rounded to `digits` · `"fraction"`: exact fractions (`⅙ A`) · `"input"`: given values exactly as typed.
/// - max-denominator (auto, int): Largest denominator for `form: "fraction"`. Default `1000`.
/// - fractions (auto, str): How fractions are written: `"stacked"` (default) or `"slash"` (`1/2`).
/// - prefixes (auto, str, none, array): Prefixes picked for results: `"all"` (default: a f p n µ m k M G T; times from a minute on in min or h, not ks), `none` (base unit only) or an array like `("m", "k")` (`"u"` = µ).
/// - show-si (auto, bool): `defined` also shows the SI value (`Q₀ = 3450 mAh = 12420 C`). Default `false`.
/// - units (auto, dictionary): Your own units, defined by a value in known units: `(bar: "100 kPa", rpm: "1/min", dozen: 12)`; `auto` makes a base unit of its own, a dimension no other unit has (`(ü: auto)`, then `mü`, `ü/s`); `none` removes one, `(T: none)`. Adds to the units set before. Only here, not per call.
/// - preferred-units (auto, str, array): Units for results of their dimension, `("bar", "kWh")`, before the unit of the inputs and the SI unit.
/// - strict (auto, bool): Unit names can't be variable names, and units in math must be quoted (`$5 "V"$`), see [Units and variables](#units-and-variables).
/// - steps (auto, str): Default for `steps` of `solve`: `"full"` (default), `"formula"`, `"result"` or `"value"`.
/// - layout (auto, str): Default for `layout` of `solve`: `"row"` (default) or `"column"`.
/// - inline-steps (auto, str): Steps for `inline: true` and inline code: `"full"`, `"formula"`, `"result"` (default) or `"value"`.
/// - implies (auto, content, symbol, str): What is written between steps, e.g. `sym.arrow.r.long` or `[and]`; ⇒ by default.
/// - approximate (auto, bool): Write `≈` when the shown result is rounded. Default `true`.
/// - block (auto, bool): Display equations (centred, default) or inline.
/// - max-solutions (auto, int): How many solutions are listed; all are found, `root` picks among all.
/// - defined-style (auto, str): How `defined` looks: `"grid"` (default: columns side by side, aligned at `=`), `"column"`, `"inline"` or `"list"`.
/// - errors (auto, str): `"document"` (default): problems are shown as red notes · `"panic"`: stop the compilation.
/// - show-casts (auto, str): How casts (`U as A`) are shown in the steps and results and in the equations as written: `"chip"` (default): as an amber chip, since a cast isn't maths · `"plain"`: as written, U as A · `"no"`: left out (`$I = U "as" A$` shows as I = U).
/// - show-types (auto, str): How types (`f(I: mA)`) are shown in the steps and in the equations as written: `"chip"` (default): as a blue chip · `"plain"`: as written, f(I: mA) · `"no"`: left out (`$f(I: "mA") = 2 I$` shows as f(I) = 2 I).
/// - words (auto, dictionary): The words shown for casts and any-unit types, for documents in other languages: `(cast: "als", any: "beliebig")`. What you type stays `as` and `any`.
/// - chips (auto, dictionary, function): The colours of the chips per kind, `(cast: (fill: …, mark: …, ink: …), type: (…))` (`mark` is the colour of `as` and `:`, `ink` the unit's), or a function `(kind, mark, unit) => content` drawing them, with `kind` `"cast"` or `"type"` and `mark` and `unit` as content.
/// - unit-style (auto, function): A function given every shown unit as content, `u => text(fill: blue, u)`.
/// - unit-separator (auto, content, str, symbol): What goes between units, a thin dot by default (A·as): `sym.times`, `h(0.2em)`.
/// - implicit-parameters (auto, str): What a function takes of what its body leaves undefined, besides `f(x, ..)`, which always takes it: `"no"` (default): nothing, `f()` takes nothing and `f(x) = x + k` with an undefined `k` is an error · `"empty"`: empty parentheses take it all, `f() = x^2 + k` is `f(x, k)` · `"allow"`: every function takes it after its listed parameters, `f(x) = x + k` is `f(x, k)`. Only here, not per call.
/// - integration (auto, str): What a `solve-block` calculates besides code tagged `solveng`: `"math"` (default: math equations) · `"code"`: code without a language · `"full"`: both · `"none"`: only tagged code. Only here, not per call. Given with content outside a solve-block, the content is calculated as in one.
/// -> content
#let solve-config(
  ..body,
  digits: auto, trailing-zeros: auto, separator: auto, grouping: auto, scientific: auto, engineering: auto, multiplication: auto, form: auto, max-denominator: auto, fractions: auto,
  prefixes: auto, show-si: auto, units: auto, preferred-units: auto, strict: auto,
  steps: auto, inline-steps: auto, layout: auto, implies: auto, approximate: auto, block: auto, max-solutions: auto,
  defined-style: auto, errors: auto, integration: auto, implicit-parameters: auto, show-casts: auto, show-types: auto,
  words: auto, chips: auto, unit-style: auto, unit-separator: auto,
) = {
  let b = body.pos().at(0, default: none)
  if body.named().len() > 0 { return _err-now("unknown option '" + body.named().keys().first() + "' (options: " + _DEF.keys().join(", ") + ")") + b }
  if body.pos().len() > 1 { return _err-now("solve-config takes at most one body") + b }
  if b != none and type(b) != content { return _err-now("solve-config: the body must be content, got " + _tn(b)) }
  let n = _check(_given((
    digits: digits, trailing-zeros: trailing-zeros, separator: separator, grouping: grouping, scientific: scientific, engineering: engineering, multiplication: multiplication, form: form, max-denominator: max-denominator, fractions: fractions,
    prefixes: prefixes, show-si: show-si, units: units, preferred-units: preferred-units, strict: strict,
    steps: steps, inline-steps: inline-steps, layout: layout, implies: implies, approximate: approximate, block: block,
    max-solutions: max-solutions, defined-style: defined-style, errors: errors, integration: integration, implicit-parameters: implicit-parameters, show-casts: show-casts, show-types: show-types,
    words: words, chips: chips, unit-style: unit-style, unit-separator: unit-separator,
  )), global: true)
  if _E(n) { return _err-now(n.err) + b }
  // units add to the units set before (none removes one); the table is built from them once, here
  let tab(c) = _mktab(c.units + n.units)
  let up(c) = {
    let c2 = c + n
    if "units" in n {
      let t = tab(c)
      if _E(t) { c2.units = c.units } else { c2.units = c.units + n.units; c2.insert("#U", t) }
    }
    c2
  }
  let bad = if "units" in n { context { let t = tab(_cf.get()); if _E(t) { _errc(t.err, _cf.get(), wide: true) } } }
  bad
  if b == none { _cf.update(up) } else {
    _cf.update(c => up(c) + ("^": c))
    b
    _cf.update(c => c.at("^"))
  }
}
#let _restart(d) = if "^0" in d { let z = d.at("^0"); z.insert("^0", z); z } else { (:) }
/// Forgets every definition (values and functions) made so far. Inside a `solve-block`:
/// back to how the block started (with `inherit: true` the outer definitions stay).
///
/// ```typ
/// #def("I = 30 mA")
/// #reset
/// #def("I = 1 A")
/// ```
#let reset = _st.update(_restart)
#let _inabs(e, x) = if e.at(0) == "f" and e.at(1) == "abs" and _has(e, x) { true } else { e.slice(1).filter(t => type(t) == array).any(t => _inabs(t, x)) }
#let _gv(g, U: _U) = if type(g) in (str, content) { let q = _q(g, U: U); if _E(q) { q } else { q.v } } else { float(g) }
// unit of the inputs to reuse for a result: only non-SI ones (h, mAh, eV); SI units get automatic prefixes anyway
#let _hint(ex, d, dm, U: _U) = {
  if ex == none { return none }
  let us = _vars(ex).dedup().filter(n => n in d and d.at(n).d == dm).map(n => d.at(n).u).dedup()
  if us.len() != 1 or us.first() == "" { return none }
  let u = us.first()
  let c = u.clusters()
  let base = if u in U { u } else if c.len() > 1 and c.first() in _P and c.slice(1).join() in U { c.slice(1).join() } else { u }
  if base in _AU or base == "g" or base in ("Omega", "\u{2126}") { none } else { u }
}
// the unit of a result: the one asked for, else a preferred one (option prefer), the non-SI unit of the inputs, or SI with a prefix
#let _mkq(v, dm, u, cf, ex: none, hint: none) = {
  let U = _UT(cf)
  let pf = none
  for p in cf.preferred-units {
    let r = _unit(p, U: U)
    if _E(r) { return _err("prefer: " + r.err) }
    if pf == none and r.at(1) == dm { pf = p }
  }
  let u = if u == none and pf != none { pf } else if u == none and hint != none { hint } else if u == none {
    let a = _auto(dm, U: U)
    // fractions read better in the base unit (1/6 A, not 500/3 mA)
    let r = if cf.form == "fraction" { _ratio(v / _factor(a, U: U), cf.max-denominator) }
    if r != none and r.at(1) > 1 { a } else { _pre(v / _factor(a, U: U), a, cf) }
  } else {
    let un = _unit(u, U: U)
    if _E(un) { return un }
    if un.at(1) != dm { return _err("the result has unit " + (if dm == _Z { "1 (no unit)" } else { _auto(dm, U: U) }) + ", not " + u) }
    u
  }
  (v: v, d: dm, raw: none, u: u, n: v / _factor(u, U: U), ex: ex)
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
// a function's body: an expression, or a comparison that's 1 where it holds, else 0 (U(x, y) := x = y)
#let _pbody(r, vars, f: (:), keep: false, U: _U) = {
  let r2 = r.replace("<=", "≤").replace(">=", "≥").replace("!=", "≠")
  let ms = r2.matches(regex("[=≤≥<>≠]"))
  if ms.len() == 0 { return _parse(r, vars, f: f, keep: keep, U: U) }
  if ms.len() > 1 { return _err("a function compares once, '" + r.trim() + "' has " + str(ms.len()) + " relations") }
  let m = ms.first()
  let (L, R) = (r2.slice(0, m.start), r2.slice(m.end)).map(x => _parse(x, vars, f: f, keep: keep, U: U))
  if _E(L) { return L }
  if _E(R) { return R }
  ("cmp", m.text, L, R)
}
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

// names in an equation are variables, also when they are written like a unit (3 h, A A): a hint for the error message,
// only for names that follow a number, a bracket or another name, as units do
#let _uvar-hint(ns, U, src) = {
  let n = ns.dedup().find(n => _known(n, U) and src.match(regex("[\\d\\])\\p{L}]\\s+" + n + "(?:[^\\p{L}\\p{N}_]|$)|\\d" + n + "(?:[^\\p{L}\\p{N}_]|$)")) != none)
  if n == none { "" } else { " – '" + n + "' is read as a variable here; for the unit write [" + n + "] (e.g. x [" + n + "]) or [2 " + n + "]" }
}
// strict: a name that is a unit can't be a variable
#let _strict-name(n, cf) = if cf.strict and _known(n, _UT(cf)) {
  _err("'" + n + "' is a unit and can't be a variable name (option strict) – pick another name like " + n + "_1, or remove the unit with solve-config(units: (" + n + ": none))")
}

// solve equations (strings) for their unknowns -> (new state, record for display) or an error
// un: target unit, none, a string (one unknown) or a dictionary name -> unit
#let _solve-into(d, eqs, un, opt, cf) = {
  let U = _UT(cf)
  let root = opt.at("root", default: auto)
  let d0 = _vars-of(d)
  let fns = d.at("#fn", default: (:))
  let eq = eqs.first()
  let env = _env(d0)
  let ps = ()
  for s in eqs {
    let p = s.split("=")
    if p.len() > 2 { return _err("more than one '=' in '" + s + "'") }
    let l = _parse(p.at(0), d0, f: fns, U: U)
    if _E(l) { return l }
    let r = if p.len() > 1 { _parse(p.at(1), d0, f: fns, U: U) }
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
    if _E(q) { return _err(q.err + _uvar-hint(_vars(e), U, eq)) }
    return (d, (k: "expr", e: e, e0: _parse(eqs.first(), d0, f: fns, keep: true, U: U), q: q, d: d0))
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
    return _err(str(ps.len()) + " equation" + (if ps.len() > 1 { "s" }) + " but " + str(xs.len()) + " unknown" + (if xs.len() != 1 { "s" }) + " (" + (if xs == () { "none" } else { xs.join(", ") }) + ") in " + eqs.map(e => "'" + e.trim() + "'").join(", ") + if xs.len() > ps.len() { " – define the missing values first, or pass unknown: …" + _uvar-hint(xs, U, eqs.join(" ")) } else { " – pass unknown: …" })
  }
  for x in xs { let b = _strict-name(x, cf); if b != none { return b } }
  if type(un) == str and xs.len() > 1 { return _err("several unknowns – give units as unit: (name: \"…\")") }
  let units = if type(un) == dictionary { un } else if type(un) == str { ((xs.first()): un) } else { (:) }
  for u in units.values() { let r = _unit(u, U: U); if _E(r) { return r } }
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
      let q = _mkq(v, dm, us.first(), cf, ex: ex, hint: _hint(ex, d0, dm, U: U))
      if _E(q) { return _err(q.err + _uvar-hint(_vars(ex), U, eq)) }
      d.insert(x, q)
      let p = eq.split("=")
      return (d, (k: "iso", L: L, R: R, L0: _parse(p.at(0), d0, f: fns, keep: true, U: U), R0: _parse(p.at(1), d0, f: fns, keep: true, U: U), x: x, ex: ex, q: q, d: d0))
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
        let q = _mkq(v, dm, us.at(j), cf, ex: ex, hint: _hint(ex, d0, dm, U: U))
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
      return (d, (k: "roots", eqs: eqs.map(s => { let p = s.split("="); (_parse(p.at(0), d0, f: fns, keep: true, U: U), _parse(p.at(1), d0, f: fns, keep: true, U: U)) }), xs: pre.keys(), poly: none, cn: none, sols: ((pre.values().map(q => q.v)),), dm: pre.values().map(q => q.d), u: pre.values().map(q => q.u), d: d0))
    }
  }
  // numeric
  let dd = _dims(ps, d0, env, xs)
  if _E(dd) { return dd }
  let (D, free) = dd
  let dms = range(xs.len()).map(i => if i in free and us.at(i) != none { _unit(us.at(i), U: U).at(1) } else { D.at(i) })
  let sc = us.map(u => if u == none { 1.0 } else { _factor(u, U: U) })
  let sols = if g != none {
    let g = if type(g) == dictionary { xs.enumerate().map(((i, x)) => if x in g { _gv(g.at(x), U: U) } else { sc.at(i) }) } else { (_gv(g, U: U),) }
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
  let ps0 = eqs.map(s => { let p = s.split("="); (_parse(p.at(0), d0, f: fns, keep: true, U: U), _parse(p.at(1), d0, f: fns, keep: true, U: U)) })
  if sel != none { for (x, q) in pre { d.insert(x, q) } }
  (d, (k: "roots", eqs: ps0, xs: px + xs, poly: poly, cn: cn, sols: sols.map(s => pre.values().map(q => q.v) + s),
    dm: pre.values().map(q => q.d) + dms, u: pre.values().map(q => q.u) + us, d: d0))
}
// one record -> stages
// a variable's declared type after its name, as show-types says
#let _typed(r, cf) = if "type" in r { _mark(":", r.type, cf) }
#let _stages(r, cf) = {
  let d = r.d
  let d = r.d
  let sub = d.pairs().map(((k, q)) => (k, (_qc(q, cf), if q.u != "" { 3 } else if q.v < 0 { 2 } else { 9 }))).to-dict()
  let D(e, s: none) = _dp(e, s, cf).at(0)
  let rel(q) = if not cf.approximate or _nshow(q, cf).at(1) { $=$ } else { $approx$ }
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
    (if L == ("v", x) { () } else { ((segs: (seg(D(L), (r0, D(R), "eq")),), join: none),) }) + ((segs: (seg($#_name(x) #_typed(r, cf)$, ..ps),), join: none, rel: r.at("rel", default: none)),)
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
    let n = cf.max-solutions
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
    if cf.steps == "value" { _eqn(_qc(r.q, cf), cf) } else { _eqn($#_name(r.name) #_typed(r, cf) #_relc(r.q, $=$) #_qc(r.q, cf)$.body, cf) }
  } else if r.k == "fn" {
    let us = r.at("units", default: (:))
    let anys = r.at("anys", default: ())
    let lhs = $#_name(r.name) (#r.params.map(p => if p in us { $#_name(p) #_mark(":", us.at(p).u, cf)$ } else if p in anys { $#_name(p) #_mark(":", "any", cf)$ } else { _name(p) }).join($, $))$
    _eqn($#lhs #if r.at("assign", default: false) { sym.colon.eq } else { $=$ } #_dp(r.body, none, cf).at(0)$.body, cf)
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
    let U = _UT(cf)
    // L := statement -> R: the statement (which tells R its value) and then L := R; R is what's solved for when the
    // statement has it, else -> gives a unit as everywhere
    let name = "^\\p{L}[\\p{L}\\p{N}_]*$"
    let (eqs, un) = {
      let (out, un2) = ((), un)
      for e in eqs {
        if type(e) != str or not e.contains(":=") { out.push(e); continue }
        let (l, r) = (e.split(":=").first(), e.split(":=").slice(1).join(":="))
        let (r, t) = if r.contains("->") { let q = r.split("->"); (q.first(), q.last().trim()) } else { (r, if type(un) == str { un.trim() }) }
        let names = _tk(_glue(r)).map(_greek)
        if t != none and t.match(regex(name)) != none and r.contains("=") and _greek(t) in names {
          out += (r.trim(), l.trim() + " := " + t)
          if t == un { un2 = none }
        } else { out.push(e) }
      }
      (out, un2)
    }
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
        let (L, R) = p.map(e => _parse(e, d0, f: fns, U: U))
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
      // units declared with names (V: A = …), for what's solved as a system
      let declared = (:)
      for s0 in eqs {
        // := (in math ≔) assigns: defines or gives a known name a new value; = states something, which determines
        // what isn't known and is checked against what is
        let raw = if type(s0) == dictionary { s0.math } else { s0 }
        let assign = type(raw) == str and (raw.contains(":=") or raw.contains("≔"))
        // V := 2 A = 2 V: a statement assigned is whether it holds, 1 or 0 (values of different units are never equal)
        let vm = if assign and type(s0) == str { s0.match(regex("^\\s*(\\p{L}[\\p{L}\\p{N}_]*)\\s*:=(.*)$")) }
        let rels = if vm != none { vm.captures.at(1).replace("<=", "≤").replace(">=", "≥").replace("!=", "≠").matches(regex("[=≤≥<>≠]")) } else { () }
        if rels.len() > 0 {
          let (n, rhs) = (_greek(vm.captures.at(0)), vm.captures.at(1).replace("<=", "≤").replace(">=", "≥").replace("!=", "≠"))
          let known = _vars-of(d)
          let sides = rhs.split(regex("[=≤≥<>≠]")).map(str.trim)
          let val(x) = { let xb = x.trim("[", at: start).trim("]", at: end); if _isq(xb, U: U) { let q = _q(xb, U: U); if _E(q) { q } else { (q.v, q.d) } } else { let e = _parse(x, known, f: d.at("#fn", default: (:)), U: U); if _E(e) { e } else { _ev(e, known) } } }
          let vs = sides.map(val)
          let b = _first-err(vs)
          if b != none { recs.push(fail(s0, b)); continue }
          let t = range(rels.len()).all(i => _ev(("cmp", rels.at(i).text, ("q", (v: vs.at(i).at(0), d: vs.at(i).at(1))), ("q", (v: vs.at(i + 1).at(0), d: vs.at(i + 1).at(1)))), (:)).at(0) == 1.0)
          let v = if t { 1.0 } else { 0.0 }
          let q = (v: v, d: _Z, raw: none, n: v, u: "", ex: none)
          d.insert(n, q)
          defs = add(defs, n, "given")
          recs.push((k: "given", name: n, q: q))
          continue
        }
        let s0 = if not assign { s0 } else if type(s0) == dictionary { (math: s0.math.replace("≔", "=").replace(":=", "=")) } else { s0.replace(":=", "=") }
        // math: read now that the variables are known ("V" is a unit, a bare V a variable if there is one)
        let s0 = if type(s0) == dictionary { _math-eq(s0.math, _vars-of(d), cf) } else { _glue(s0) }
        if _E(s0) { recs.push(fail("", s0)); continue }
        // .. stands only in the parameters of a function being defined, f(x, ..) = …
        if s0.match(regex("\\.\\s*\\.|…")) != none and s0.match(regex("^\\s*\\p{L}[\\p{L}\\p{N}_]*\\s*\\([^()]*\\)\\s*(?::[^=]*)?=[^=]*$")) == none {
          let tidy = s0.replace(regex("\\s*([(),])\\s*"), m => m.captures.at(0) + if m.captures.at(0) == "," { " " } else { "" }).replace(regex("\\.\\s+\\."), "..")
          recs.push(fail(s0, _err("'" + tidy.trim() + "' defines nothing: '..' stands only in the parameters of a function being defined, on its own on the left, f(x, ..) = …")))
          continue
        }
        // f(…) := body: the body may compare, U(x, y) := x = y
        let fbody = assign and s0.match(regex("^\\s*\\p{L}[\\p{L}\\p{N}_]*\\s*\\([^()]*\\)\\s*=")) != none
        let s = if fbody { (s0, none) } else { _ineq(s0) }
        if _E(s) { recs.push(fail(s0, s)); continue }
        let (s, rel) = s
        let (d0, fns0) = (_vars-of(d), d.at("#fn", default: (:)))
        let p = s.split("=")
        let p = if fbody { (p.first(), p.slice(1).join("=")) } else { p }
        // a chain R = 7.2 kΩ = 7200 Ω: its steps must be equal, the last gives the value, shown in the unit that fits
        let chain = p.len() > 2 and rel == none and not fbody
        if chain {
          let known = _vars-of(d)
          let val(x) = { let x = x.trim(); let xb = x.trim("[", at: start).trim("]", at: end); if _isq(xb, U: U) { let q = _q(xb, U: U); if not _E(q) { (q.v, q.d) } } else { let e = _parse(x, known, f: d.at("#fn", default: (:)), U: U); if not _E(e) and _vars(e).all(v => v in known) { let r = _ev(e, known); if not _E(r) { r } } } }
          let vals = p.slice(1).map(x => (x.trim(), val(x))).filter(x => x.at(1) != none)
          let round(dm) = dm.map(z => calc.round(z, digits: 6))
          let bad = vals.slice(calc.min(1, vals.len())).find(((x, r)) => round(r.at(1)) != round(vals.first().at(1).at(1)) or calc.abs(r.at(0) - vals.first().at(1).at(0)) > 1e-3 * calc.max(calc.abs(r.at(0)), calc.abs(vals.first().at(1).at(0))))
          if bad != none {
            let tidy(x) = x.replace(regex("[\\[\\]\"]"), "").trim()
            let why = if round(bad.at(1).at(1)) != round(vals.first().at(1).at(1)) { "has another unit" } else { "has another value" }
            recs.push(fail(s, _err("'" + tidy(bad.at(0)) + "' " + why + " than '" + tidy(vals.first().at(0)) + "' in '" + tidy(s) + "' – the steps of an equation must be equal (≈ for rounded ones)")))
            continue
          }
        }
        let p = if chain { (p.first(), p.last()) } else { p }
        if p.len() > 2 { recs.push(fail(s, _err("more than one '=' in '" + s.trim() + "'"))); continue }
        // Delta_R is Δ_R, as in math
        let (l, r) = (_greek(p.at(0).trim()), p.at(1, default: "").trim())
        // V: A = …: variables have no written type, their unit is their value's (functions' parameters have types)
        let tm = if p.len() == 2 and rel == none { l.match(regex("^(\\p{L}[\\p{L}\\p{N}_]*)\\s*:\\s*([^-+−=]+)$")) }
        let lu = if tm != none {
          let (n, u) = tm.captures.map(x => x.trim().trim("[", at: start).trim("]", at: end).trim().trim("\""))
          _err("'" + n + ": " + u + "' – a variable has the unit of its value, so it takes no type: write " + n + " = …, with unit: \"" + u + "\" (or -> " + u + " in code) to show it in " + u + " (types are for a function's parameters, f(x: " + u + "))")
        }
        if _E(lu) { recs.push(fail(s, lu)); continue }
        let (l, s) = if lu != none { (_greek(tm.captures.at(0)), _greek(tm.captures.at(0)) + " = " + r) } else { (l, s) }
        if lu != none { declared.insert(l, lu.u) }
        let un = if lu == none { un } else if type(un) == dictionary { un + ((l): lu.u) } else { lu.u }
        // f(x, y) or f(x: A, y: A): A, with the type of what it returns
        let fm = l.match(regex("^(\\p{L}[\\p{L}\\p{N}_]*)\\s*\\((.*)\\)\\s*(?::\\s*([^=]+))?$"))
        // a known function on the left (U(o) = 20) is an equation to solve; := defines it again
        if rel == none and p.len() == 2 and fm != none and fm.captures.at(0) not in _F and (fm.captures.at(0) not in d.at("#fn", default: (:)) or assign) {
          // function: f(x, y) = …
          let name = fm.captures.at(0)
          // f(I: mA, ..): parameters, optionally with a unit (which plain numbers passed to them get); .. lets the body's
          // other undefined names be parameters too, else they're an error
          let pl = _plist(fm.captures.at(1), name, U: U, implicit: cf.implicit-parameters)
          if _E(pl) { recs.push(fail(s, pl)); continue }
          let b = (name, ..pl.names).map(n => _strict-name(n, cf)).find(x => x != none)
          if b != none { recs.push(fail(s, b)); continue }
          // what a function returns follows from its body; it takes no written type
          let rt = fm.captures.at(2)
          if rt != none { let u = rt.trim().trim("[", at: start).trim("]", at: end).trim().trim("\""); recs.push(fail(s, _err(name + "(…): " + u + " – a function returns what its body gives, so it takes no type: cast the body to give it that unit, " + name + "(…) = (…) as " + u))); continue }
          let fns = d.at("#fn", default: (:))
          let body = _pbody(r, _vars-of(d) + _pvars(pl), f: fns, U: U)
          if _E(body) { recs.push(fail(s, body)); continue }
          let ps = _pfree(body, pl, _vars-of(d), name)
          if _E(ps) { recs.push(fail(s, ps)); continue }
          // untyped parameters take the types of what they're passed to (f(y) = q(y) with q(x: A): y is an A), and
          // are checked like written ones
          let (units, anys) = (pl.units, pl.anys)
          let kept = _pbody(r, _vars-of(d) + _pvars(pl), f: fns, keep: true, U: U)
          if not _E(kept) {
            for (n, t) in _passed(kept, fns) {
              if n not in ps or n in units or n in anys { continue }
              if t == "any" { anys.push(n) } else { units.insert(n, t) }
            }
          }
          let body = if units == pl.units { body } else { _pbody(r, _vars-of(d) + _pvars((units: units)), f: fns, U: U) }
          if _E(body) { recs.push(fail(s, body)); continue }
          // with every parameter typed, so is the result, as it comes out
          let typed = ps.all(n => n in units)
          let out = if typed { _ev(body, _vars-of(d) + _pvars((units: units))) }
          let ret = if out == none or _E(out) { none } else if out.at(1) == _Z { "1" } else { _auto(out.at(1), U: U) }
          fns.insert(name, (params: ps, body: body, src: r, units: units, anys: anys, ret: ret))
          d.insert("#fn", fns)
          defs = add(defs, name, "function")
          recs.push((k: "fn", name: name, params: ps, units: units, anys: anys, ret: ret, assign: assign, body: _pbody(r, _vars-of(d), f: fns, keep: true, U: U)))
          continue
        }
        let id = p.len() == 2 and l.match(regex(_ID)) != none
        if id { let b = _strict-name(l, cf); if b != none { recs.push(fail(s, b)); continue } }
        // a statement about a known variable can't change it: one that disagrees is an error, := assigns
        let known = _vars-of(d)
        if id and not assign and rel == none and l in known {
          let rb = r.trim("[", at: start).trim("]", at: end)
          let rv = if _isq(rb, U: U) { let q = _q(rb, U: U); if not _E(q) { (q.v, q.d) } } else {
            let e = _parse(r, known, f: d.at("#fn", default: (:)), U: U)
            if not _E(e) and _vars(e).all(v => v in known and v != l) { let x = _ev(e, known); if not _E(x) { x } }
          }
          let q = known.at(l)
          let round(dm) = dm.map(z => calc.round(z, digits: 6))
          if rv != none and (round(rv.at(1)) != round(q.d) or calc.abs(rv.at(0) - q.v) > 1e-3 * calc.max(calc.abs(rv.at(0)), calc.abs(q.v))) {
            let was = str(calc.round(q.at("n", default: q.v), digits: 4)) + if q.u != "" { " " + q.u } else { "" }
            recs.push(fail(s, _err(l + " is " + was + " already, but '" + s.replace(regex("[\\[\\]\"]"), "").trim() + "' says otherwise – = states what's true and can't change a variable; " + l + " := … gives it a new value")))
            continue
          }
        }
        // [3 h]: a value in brackets, the unit is meant even if there is a variable h
        let br = r.match(regex("^\\[([^\\]]*)\\]$"))
        let br = if br != none and _isq(br.captures.at(0), U: U) { br.captures.at(0) }
        if id and (br != none or _isq(r, U: U)) {
          // plain value: R = 7.2 kΩ
          let q = _q(if br != none { br } else { r }, U: U)
          if _E(q) { recs.push(fail(s, q)); continue }
          // "3 h" with a variable h: hours or 3 times h? Refused, so it is always clear what is meant
          let amb = if br == none { _unames(q.u, U: U).find(n => n in _vars-of(d)) }
          if amb != none {
            recs.push(fail(s, _err("'" + amb + "' in '" + r + "' is both a defined variable and a unit – write [" + r + "] for the unit, or use * or parentheses for the variable (3*" + amb + ", 1/(" + amb + "))")))
            continue
          }
          if rel != none { q.rel = rel }
          if lu != none and q.d != lu.d { recs.push(fail(s, _err(l + " is in " + lu.u + ", but its value " + r.trim("[", at: start).trim("]", at: end) + " isn't"))); continue }
          // shown in the unit it's declared with
          if lu != none and q.u != lu.u { q = q + (n: q.v / lu.f, u: lu.u, raw: none) }
          // a chain's value in the unit that fits it, not the last step's
          if chain and q.d != _Z { let a = _auto(q.d, U: U); let u = _pre(q.v / _factor(a, U: U), a, cf); q = q + (n: q.v / _factor(u, U: U), u: u, raw: none) }
          d.insert(l, q)
          defs = add(defs, l, "given")
          recs.push((k: "given", name: l, q: q) + if lu != none { (type: lu.u) } else { (:) })
          continue
        }
        if id and pending.len() == 0 {
          // expression of known variables: U = R I
          let e = _parse(r, _vars-of(d), f: d.at("#fn", default: (:)), U: U)
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
            recs.push(res.at(1) + if lu != none { (type: lu.u) } else { (:) })
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
        let un = if declared.len() == 0 { un } else if type(un) == dictionary { declared + un } else if un == none { declared } else { un }
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
