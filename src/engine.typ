// The engine: solveng calculates in a WebAssembly plugin (Rust, in core/ and plugin/ of the repository). Here math
// is read as written for it, the worksheet's definitions are kept in a state as the bytes it gives back, and what
// it returns – Typst math markup – is shown.
/// ## Calculating like Mathcad
///
/// Three operators, and `=` keeps its meaning from maths:
///
/// - **`:=` defines**, symbolically. Names defined by then are put in right away, names that aren't stay names until they are: `$h := 5 z$` works before `z` is defined, and after `$z := 20 cm$`, `$h =$` shows 1 m. What's put in is the value at that point – defining a name again later changes what comes after, not what came before.
/// - **`=` inside an expression compares**, as Mathcad's bold =: `$M := 1 = 2 =$` gives 0, `$M := 2 A = 2000 mA$` gives 1. Values of different units are never equal (`1 A = 2 V` is 0, `1 A != 2 V` is 1); `<`, `<=`, `>`, `>=` and `!=` compare too, chains like `$0 < x <= 1$` hold when every step does.
/// - **A trailing `=` shows the value**, numerically, with a bold =. After it come a unit and keywords: `$t = 1 h, digits(3)$`. Every name in it must have a value by then.
/// - **A trailing `->` shows the symbolic result**: defined values put in, exactly (fractions, roots and π stay), undefined names kept: `$x + 3/3 y -> simplify$` is x + y. With `solve(x)` it solves – symbolically when it can (`$0 = 1 + 3/x + y^2 -> solve(x)$` is x = −3/(y² + 1)), numerically when every other name has a value – and defines x when the solution is a value.
/// - **Any other `=`** states something: it's checked when everything in it has a value (`$U = R I$`), and left alone as prose when not (`$E = m c^2$`).
///
/// **Functions** are defined the same way: `$f(x) := x^2 + 1$`. Their body is symbolic too, so a function needn't define everything it uses: names that aren't parameters and aren't defined stay names, looked up when the function is evaluated, and cancel where the algebra lets them (`$U(x: m) := h x a$`, then `$g(x: m) := U(x)/a ->$` is `1 m · x` without any `a`). Undefined function calls stay symbolic too: `$f(t) := integral_(-infinity)^t sigma(tau) dif tau$` is valid before `sigma` is defined; numerical evaluation requires its definition and convergence. A parameter can have a unit, `f(I: mA)`: plain numbers passed to it get the unit (`f(2)` is `f(2 mA)`), values must have its dimension; `any` takes every unit. `..` makes the body's other undefined names parameters, in the order they appear: `$P(..) := U^2/R$` is `P(U, R)`. A parameter passed on to one with a unit takes that unit: with `$q(x: A) := 2 x$`, `$f(y) := q(y)$` makes y an A.
///
/// ````typ
/// #show: worksheet
/// $h := 5 z$, $z := 20 cm$, $h =$             // h ≔ 5z, z ≔ 20 cm, h = 1 m
/// $U(x: m) := h x a ->$                      // U(x) ≔ h x a → 1 m · a x
/// $g(x: m) := U(x)/a ->$                     // g(x) ≔ U(x)/a → 1 m · x
/// $M := 1 A = 2 V ->$                        // M ≔ 1 A = 2 V → 0
/// ```solveng
/// k := 2 m = 200 cm =                        // k = 1: the comparison holds
/// ```
/// ````
///
/// ## Writing values
///
/// ````typ
/// #show: worksheet
/// $R := 4.7 kΩ$                        // math: a number and a unit
/// $I := frac(2, 4) A$, $Q_0 := 3450/2 mAh$, $U := (1+2) dot 1,5 V$, $A := 2^3 m^2$  // arithmetic in the number
/// ```solveng
/// R2 := 2 R; Z := [5 V] / I            // code: units in brackets inside an equation
/// U2 := R [1 mA]
/// Q := @U2 [2 s]                       // @U2 inlines its formula: Q = R [1 mA] [2 s]
/// ```
/// ````
///
/// - **Units:** SI base units (m kg s A K mol cd) and V A Ω W J C F H S Hz N Pa T Wb, min h Ah Wh As Ws eV l %, ‰, °, rad, °C (as a temperature *difference*), all SI prefixes; your own with the option `units`. Powers as `m^2`, `m²`, `s^-1`; quotients as `m/s^2`; products with spaces `kg m`.
/// - **Typeable spellings:** `Omega` → Ω, with a prefix glued as in math: `k""Omega` → kΩ (`kOmega` is not a unit, like `DeltaT` is not ΔT); `u` → µ, `degC` → °C, `deg` → °. They are printed with the proper symbols (`[20 uA]` → 20 µA). In math, units are names (`$4.7 kΩ$`, `$20 µA$`), see [Units and variables](#units-and-variables).
/// - **Decimal comma:** `7,2 V` works; the output separator follows the document language (option `separator`).
/// - **Names:** the acute accent `´` and the backtick (`` ` ``) are literal name characters (`$x´ := 3$`), with no escaping needed. Digits continue a name: `t2` differs from `t_2` and keeps its digits on the baseline. Complete Latin letter runs normalize to Greek: `Delta2tau` → Δ2τ, while `Deltatau` stays unchanged. `""` joins name fragments: `Delta""tau` → Δτ and `t""2` → t2. Quotes only carry a name through Typst: `$"Delta2tau"$` names Δ2τ, and `$"Delta\"\"tau"$` names Δτ. Explicit subscripts may be grouped: `$R_(a b)$` names R_ab. Operators and whitespace end a name; a leading number remains a coefficient (`2t2` is 2 times t2).
/// - **Sums:** `sum_(k=0)^10 k^2` or `sum(k^2, k, 0, 10)` sums over inclusive integer bounds. The index is local and case-sensitive. Parenthesize a summand containing addition. Small finite sums simplify exactly; infinite sums stay symbolic after `->` and are evaluated numerically for `=` and graphs. For example, `$s_2(t) := sum_(k=0)^infinity sigma(t-k+3/4)$` defines a function usable as `#graph("s_2(t)")` once `sigma` is defined. Piecewise linear signal sums are counted directly between their corners. Other numerical sums use at most 100000 terms and a shrinking-tail convergence estimate; slow or nonconvergent series report an error. Bounds and the index are unitless, while summands may carry units.
/// - **Signals:** `rect(x)` is 1 for |x| < ½, ½ at ±½, otherwise 0; `tri(x)` is max(1 − |x|, 0); `sgn(x)` is −1, 0 or 1. Arguments are unitless. The standard names are always available and exported by solveng. Define a function such as `$Lambda(t) := tri(t)$` to use another name. Linear combinations simplify and can be solved; definite integrals split affine arguments at their boundaries. Distributional derivatives stay symbolic.
/// - **Operators:** `+ - * / ^ · ×`, implicit multiplication (`2 pi f C`), `sqrt exp ln log sin cos tan asin acos atan abs`, constants `pi e` (unless you define a variable with that name), your own functions.
///
/// ## Units and variables
///
/// Many engineering symbols are also units: `A` (area, ampere), `V` (volume, volt), `T` (period, tesla), `h` (height, hour), `C`, `F`, `W`, `m`, `s`, `l`, `L`, … solveng keeps them apart by *how* they are written, never by guessing:
///
/// - **Math:** units are names, `$30 mA$`, `$4.7 kΩ$`, `$9.81 m/s^2$` (solveng brings every unit of several letters and every prefixed one, set upright); `unit("bar")` writes any other, also your own. A single letter is a unit only right after a number (`$5 V$`, `$3 h$`, whatever is defined), so `$t = 1 h$` shows t in hours; anywhere else it's a variable, also alone after `=` (`$t = h$` compares t with h), and `$3 dot h$` is 3 times h. Quoted text is a name of several letters (`$"tews" := 2$`), as Typst needs; `"as"` casts.
/// - **Code:** a unit is read only where a value is written – right after a number (`I := 30 mA`) or in brackets (`[5 V]`, `[A]` for one ampere). Every other name in an equation is a variable. A value whose unit is also a defined variable (`t := 3 h` with `h` defined, `f := 1/T` with `T` defined) is refused, with a hint how to write either meaning: `[3 h]` for hours, `3*h` or `1/(T)` for the variable.
///
/// ````typ
/// #show: worksheet
/// $A := 12$, $x := A dot 2 A =$        // x ≔ A · 2 A = 24 A
/// $T := 2 s$, $f := 1/T = Hz$          // f ≔ 1/T = 0.5 Hz
/// ```solveng
/// g := 1/T                             // error: 'T' in '1/T' is both a defined variable and a unit – …
/// ```
/// ````
///
/// For documents where nothing may be read two ways, `#worksheet-config(strict: true)` also refuses unit names as variable names (`A := 3 m^2` → pick `A_1`, or remove the unit).
///
/// The available units can be changed with the option `units`: new units are defined by a value, `none` removes one (which frees its name, e.g. `T` for a period that is then never read as tesla):
///
/// ```typ
/// #worksheet-config(units: (bar: "100 kPa", rpm: "1/min", t: "1000 kg", T: none))
/// #show: worksheet
/// $p := 3 unit("bar")$, $n := 3000 unit("rpm")$          // prefixes work too: unit("mbar")
/// #worksheet-config(preferred-units: ("bar", "kWh"))  // results with these dimensions are shown in these units
/// ```
///
/// ## How it solves
///
/// solveng calculates in a WebAssembly plugin written in Rust (`core/` in the repository). Equations are solved symbolically first, other names as parameters: an unknown that occurs once is isolated; an equation that is a polynomial in it, once both sides are over one denominator, is solved by formula up to degree 2 and for all real roots numerically above; a system is solved one equation at a time where an equation has a single unknown, and by elimination where it's linear. Results are simplified: like terms combined, fractions cancelled (`(x^2 - 1)/(x - 1)` is x + 1), exact numbers kept exact. What can't be done symbolically is solved numerically, once every other name has a value: sign changes over ±10⁻¹⁵…10¹⁵ (times the target unit) are bisected and Newton starts from every local minimum of the absolute value; systems by damped Newton from 41 starts. Units of unknowns are inferred from the equations; mismatched units stop with an error.
///
/// A solved definition keeps its solutions: `circle(x, r) := x^2 + b^2 = r^2 -> solve(b)` returns a vector of both branches. A relation function can supply the equation: `p(a, b, c) := a^2 + b^2 = c^2`, then `circle(x, r) := p(x, b, r) -> solve(b)`. The solve variable is local, not a parameter. One solution returns a scalar; several return a vector, with `at(vector, 0)` selecting its first component. Graphing a vector plots every branch. Solved definitions require a symbolic solution; equations in `vec(...)` with several solve variables return components in the requested order, nested by branch when needed.
///
/// ## Limits
///
/// - **Numbers** are exact fractions as long as they fit (128-bit numerator and denominator), else 64-bit floats: about 15 significant digits, magnitudes up to 10³⁰⁰.
/// - **Isolating** `sqrt(x) = 2` gives only the positive root, periodic functions only the principal value (`tan(x) = 1` → `arctan(1)`); polynomials like `x^2 = 4` give all their roots.
/// - **Symbolically**, polynomials are solved up to degree 2; higher degrees need values for their coefficients. Simplifying cancels common factors of polynomials (with exact coefficients) but doesn't factor or use trigonometric identities.
/// - **Numeric search** can miss roots squeezed between nearby poles, roots of very fast oscillations, and roots outside ±10¹⁵ (scaled by the target unit); the result then says "no real solution".
/// - **Systems** are solved numerically from a fixed set of starts; solutions far away from them can be missed. Underdetermined systems show one arbitrary solution.
/// - **Domains:** only real numbers; `(-8)^(1/3)` is −2, `sqrt(-1)` an error.
/// - **°C** is a temperature difference (no 273.15 offset); temperatures in formulas like `p V = n R T` need K. New base dimensions (bit, €) can be added with `units: (bit: auto)`, at most 8.
/// - **Prefixes:** a lone `m` before a unit is metre: `"m s"`, `$"m" Omega$` are m·s and Ω·m; write `"ms"`, `$"m"Omega$` for milli.
/// - **Functions** can't call themselves.
/// - **Math in a worksheet:** an equation with `=` and only known names is checked, so a formula meant as plain text that happens to use defined names (`$a^2 + b^2 = c^2$` after defining a, b and c) must hold. Put those outside the worksheet or use `#worksheet(evaluate: false)[…]` around them.

#import "display.typ": *

#let _p = plugin("solveng.wasm")
#let _call(f, ..args) = cbor(f(..args.pos().map(a => if type(a) == bytes { a } else { cbor.encode(a) })))

// ---------- options ----------
#let _LANGS = ("de", "fr", "it", "es", "pt", "nl", "ru", "pl", "cs", "sk", "sl", "hr", "hu", "sv", "da", "nb", "fi", "tr", "ro", "uk")
// needs context: the decimal separator follows the document language
#let _sep(cf) = if cf.separator != auto { cf.separator } else if text.lang in _LANGS { "," } else { "." }
// plain text of content (units: (bar: [100 kPa]))
#let _plain(c) = if type(c) == str { c } else if type(c) in (int, float, symbol) { str(c) } else if c.has("text") and type(c.text) == str { c.text } else if c.has("children") { c.children.map(_plain).join() } else if c.has("body") { _plain(c.body) } else { " " }
#let _uopt(v) = if v == none { "none" } else if v == auto { "auto" } else if type(v) in (int, float) { float(v) } else { _plain(v) }
// the options as the engine reads them
#let _req-cfg(cf) = {
  let g = cf.grouping
  (
    digits: cf.digits, trailing-zeros: cf.trailing-zeros, separator: _sep(cf),
    grouping: if g == none { none } else if type(g) == str { g } else if type(g) == symbol { str(g) } else { "#" },
    scientific: cf.scientific, engineering: cf.engineering, multiplication: cf.multiplication, form: cf.form,
    max-denominator: cf.max-denominator, fractions: cf.fractions,
    prefixes: if cf.prefixes == none { "None" } else if cf.prefixes == "all" { "All" } else { (Some: cf.prefixes) },
    show-si: cf.show-si, units: cf.units.pairs().map(((k, v)) => (k, _uopt(v))).to-dict(),
    preferred-units: if type(cf.preferred-units) == str { (cf.preferred-units,) } else { cf.preferred-units }, strict: cf.strict,
    steps: if cf.steps == auto { cf.inline-steps } else { cf.steps }, inline-steps: cf.inline-steps, layout: cf.layout,
    approximate: cf.approximate, block: cf.block, max-solutions: cf.max-solutions,
    show-casts: cf.show-casts, show-types: cf.show-types,
    words: (cast: cf.words.at("cast", default: "as"), any: cf.words.at("any", default: "any")),
    unit-style: type(cf.unit-style) == function,
  )
}
// what the engine's markup refers to
#let _UDOT = h(0.06em) + math.class("normal", sym.dot.op) + h(0.06em)
#let _scope(cf) = (
  at: math.op("at"),
  svsep: { let s = cf.at("unit-separator", default: auto); if s == auto { _UDOT } else if type(s) == str { math.upright(s) } else { s } },
  svimp: cf.implies,
  svgroup: cf.grouping,
  svu: { let f = cf.at("unit-style", default: auto); if type(f) == function { f } else { x => x } },
  svchip: (kind, mark, u) => _chip(kind, mark, u, cf),
)
// markup from the engine as math content
#let _m(s, cf) = eval(s, mode: "math", scope: _scope(cf))

// ---------- reading math as written ----------
#let _space = [ ].func()
#let _word-end = regex("[\\p{L}\\p{N}\"⦄⦆)]$")
#let _word-start = regex("^[\\p{L}\\p{N}\"⦃⦅]")
#let _prefix = regex("^\"[afpnµμumcdkMGT]\"$")
#let _align-point = $&$.body.func()
// math content -> the equation as text for the engine: letters as they are, quoted text "…" (names of several
// letters, units, as), solveng's unit words ⦃mA⦄, keywords ⦅solve(x)⦆; or an error for what isn't a formula
#let _src(c) = {
  if c == math.dif { return "dif" }
  if type(c) == str { return c }
  if type(c) in (int, float) { return str(c) }
  if type(c) == symbol { return if str(c) == "ⅆ" { "dif" } else { str(c) } }
  if type(c) != content { return _err("not a formula") }
  let f = c.func()
  if f == math.vec {
    let parts = c.children.map(x => _src(x))
    let error = parts.find(_E)
    return if error != none { error } else { "vec(" + parts.join(", ") + ")" }
  }
  if f == math.cases {
    let branches = ()
    for (i, row) in c.children.enumerate() {
      let parts = if row.has("children") { row.children } else { (row,) }
      let at = range(parts.len()).filter(i => parts.at(i).func() == _align-point)
      if at.len() > 1 { return _err("cases: one & separates a value from its condition") }
      let value = _src(if at.len() == 0 { row } else { parts.slice(0, at.first()).join() })
      if _E(value) { return value }
      let condition = if at.len() == 0 { () } else { parts.slice(at.first() + 1).filter(p => p.func() != _space and p.func() != h) }
      if condition.len() > 0 and condition.first().func() == text and condition.first().text.trim() in ("für", "if", "for", "wenn", "falls") { condition = condition.slice(1) }
      if condition.len() == 1 and condition.first().func() == text and condition.first().text.trim() in ("otherwise", "else", "sonst", "andernfalls") { condition = () }
      if condition.len() == 0 and i != c.children.len() - 1 { return _err("cases: only the last branch can omit its condition") }
      let test = if condition.len() == 0 { "1" } else { _src(condition.join()) }
      if _E(test) { return test }
      branches.push(value + ", " + test)
    }
    return "cases(" + branches.join(", ") + ")"
  }
  if c.has("label") and c.label == <solveng-unit> { return "⦃" + _plain(c) + "⦄" }
  if f == metadata {
    if c.has("label") and c.label == <solveng-key> {
      let args = c.value.args.map(a => { let s = _src(a); if _E(s) { "?" } else { s.replace("\"", "").trim() } })
      return "⦅" + c.value.key + (if args.len() > 0 { "(" + args.join(",") + ")" } else { "" }) + "⦆"
    }
    return ""
  }
  if f == text {
    let t = c.text
    if t == "ⅆ" { return "dif" }
    return if t == "" { "\"\"" } else if t.match(regex("\\p{L}|\\p{So}|[°%‰]")) != none { repr(t) } else { t }
  }
  if f == linebreak { return "\\" }
  if f == _space or f == h { return " " }
  if c.has("children") {
    let out = ""
    let prev = ""
    for x in c.children {
      let s = _src(x)
      if _E(s) { return s }
      if s == "" { continue }
      // m glued to Ω is milliohm, apart it's metre times ohm
      if prev.match(_prefix) != none and s.starts-with("Ω") {
        out = out.slice(0, out.len() - prev.len()) + "⦃" + prev.trim("\"") + "Ω⦄"
        prev = "⦄"
        let rest = s.slice("Ω".len())
        out += rest
        continue
      }
      // names must not run into each other (Δ""r glues on purpose)
      if prev != "\"\"" and s != "\"\"" and prev.match(_word-end) != none and s.match(_word-start) != none { out += " " }
      out += s
      prev = s
    }
    return out
  }
  if f == math.frac {
    let (a, b) = (_src(c.num), _src(c.denom))
    return if _E(a) { a } else if _E(b) { b } else if a.trim() == "dif" { "dif / (" + b.replace("ⅆ", "dif") + ")" } else { "frac(" + a + ", " + b + ")" }
  }
  if f == math.attach {
    let base = _src(c.base)
    if _E(base) { return base }
    if (base == "∫" or base == "∑") and c.has("b") and c.b != none and c.has("t") and c.t != none {
      return (if base == "∑" { "sum_(" } else { "integral_(" }) + _src(c.b) + ")^(" + _src(c.t) + ")"
    }
    let out = base
    // subscripts are part of a name (R_"load", "x"_("in" Delta))
    if c.has("b") and c.b != none {
      let sub = _src(c.b)
      if _E(sub) { return sub }
      out = base.trim("\"") + "_" + sub.replace("\"", "").replace(regex("\\s"), "")
    }
    if c.has("t") and c.t != none {
      let t = _src(c.t)
      if _E(t) { return t }
      out += "^(" + t + ")"
    }
    for k in ("tl", "tr", "bl", "br") { if c.has(k) and c.at(k) != none { return _err("not a formula") } }
    return out
  }
  if f == math.class and _plain(c.body) == "d" { return "dif" }
  if f == math.op {
    let name = _plain(c.text).trim()
    return if name == "d" { "dif" } else { name }
  }
  if f == math.root {
    let r = _src(c.radicand)
    if _E(r) { return r }
    if c.has("index") and c.index != none { let i = _src(c.index); return if _E(i) { i } else { "(" + r + ")^(1/(" + i + "))" } }
    return "sqrt(" + r + ")"
  }
  if f == math.lr { return _src(c.body) }
  if f == raw { return if c.text == "frac" { "⦅frac⦆" } else { _err("not a formula") } }
  // styling (upright, class, …) is read through
  if c.has("body") { return _src(c.body) }

  if c.has("text") and type(c.text) == str { return c.text }
  _err("not a formula")
}

// ---------- the equation as written ----------
// solveng's equations carry a mark, so its rules never read them again
#let _RESHOWN = metadata("solveng-reshown")
#let _reshown(eq) = eq.body.has("children") and eq.body.children.any(c => c == _RESHOWN)
#let _unit-mk(s, cf) = {
  let r = _call(_p.unit, (cfg: _req-cfg(cf), src: s.replace("⦃", " ").replace("⦄", " ")))
  if r == none { none } else { _m(r, cf) }
}
// casts (U "as" A) and types (f(x: "mA")) as chips, plain or left out, as the options say; letters the engine read
// as units after a number upright: 5 V
#let _restyle(c, letters, cf) = {
  if type(c) != content { return c }
  if c == math.dif { return c }
  let f = c.func()
  if f == text and not (c.has("label") and c.label == <solveng-unit>) {
    let normalized = _call(_p.name, c.text)
    if normalized != none { return _m(normalized, cf) }
  }
  if f == math.vec { return math.vec(..c.children.map(x => _restyle(x, letters, cf))) }
  if f == math.cases {
    let fields = c.fields()
    let children = fields.remove("children")
    return math.cases(..fields, ..children.map(x => _restyle(x, letters, cf)))
  }
  let space(x) = x == none or x.func() == _space
  let word(x) = if x.has("text") and type(x.text) == str { x.text } else { none }
  if c.has("children") {
    let (out, ch, i) = ((), c.children, 0)
    while i < ch.len() {
      let x = ch.at(i)
      // A differential is an operator, even though _src spells it as `dif`.
      // Preserve it before the name scanner can turn it into a literal name.
      if x == math.dif {
        out.push(x)
        i += 1
        continue
      }
      let mark = word(x)
      let before = out.rev().find(y => not space(y))
      let number = before != none and word(before) != none and word(before).match(regex("\\d$")) != none
      if mark in ("as", ":") and not (mark == "as" and number) {
        // the unit: the longest run of children after the mark that reads as one
        let (k, end, um) = (i + 1, none, none)
        while k < ch.len() {
          let w = word(ch.at(k))
          if not space(ch.at(k)) and w not in ("∗", "*", "·", "⋅", "/") {
            let s = ch.slice(i + 1, k + 1).map(_src)
            if s.any(_E) { break }
            let r = _unit-mk(s.join(" "), cf)
            if r == none { if end != none { break } } else { (end, um) = (k + 1, r) }
          }
          k += 1
        }
        if end != none {
          while out.len() > 0 and space(out.last()) { let _ = out.pop() }
          let how = cf.at(if mark == "as" { "show-casts" } else { "show-types" }, default: "chip")
          let m = if mark == "as" { math.upright(cf.words.at("cast", default: "as")) } else { math.class("normal", sym.colon) }
          if how == "chip" { out.push(h(0.15em) + _chip(if mark == "as" { "cast" } else { "type" }, m, um, cf)) }
          else if how == "plain" { out.push($med #m med #um$.body) }
          i = end
          continue
        }
      }
      // a letter after a number is a unit, set upright as units are: 5 V
      let named = x.func() == text and x.has("label") and x.label == <solveng-unit>
      if mark != none and (named or (mark in letters and x.func() != text)) and number {
        while out.len() > 0 and space(out.last()) { let _ = out.pop() }
        out.push(h(0.1667em) + math.upright(mark))
        i += 1
        continue
      }
      let (end, normalized, source) = (none, none, "")
      let k = i
      while k < ch.len() and not space(ch.at(k)) {
        let s = _src(ch.at(k))
        if _E(s) { break }
        source += s
        let n = _call(_p.name, source)
        if n != none { (end, normalized) = (k + 1, n) } else if s != "\"\"" { break }
        k += 1
      }
      if end != none {
        out.push(_m(normalized, cf))
        i = end
      } else {
        out.push(_restyle(x, letters, cf))
        i += 1
      }
    }
    return out.join()
  }
  if f == math.lr { return math.lr(_restyle(c.body, letters, cf)) }
  if f == math.frac { return math.frac(_restyle(c.num, letters, cf), _restyle(c.denom, letters, cf)) }
  if f == math.root { return math.root(c.at("index", default: none), _restyle(c.radicand, letters, cf)) }
  c
}
// the equation with a new body, marked as read
#let _with-body(eq, body) = {
  let fs = eq.fields()
  let _ = fs.remove("body")
  math.equation(..fs, _RESHOWN + body)
}
// an equation's children -> (head, marker, clause): at the last top-level →, else at the last top-level =
#let _split-eq(body, mark) = {
  let cs = if body.has("children") { body.children } else { (body,) }
  let t = if mark == "->" { "→" } else { "=" }
  let at = range(cs.len()).filter(i => cs.at(i).has("text") and cs.at(i).text == t)
  if at.len() == 0 { return (cs, none) }
  let head = cs.slice(0, at.last())
  while head.len() > 0 and head.last().func() == _space { let _ = head.pop() }
  (head, t)
}
// the evaluating =, bold as in Mathcad: stroked, since math fonts have no bold =
#let _BEQ = math.class("relation", context text(stroke: (paint: text.fill, thickness: 0.045em), "="))

// ---------- running ----------
// the state: env (the engine's bytes), last (what the last line gave), ^ (outside a worksheet), ^0 (its start)
#let _env(s) = s.at("env", default: bytes(()))
#let _run(s, req) = {
  let r = _call(_p.line, _env(s), req)
  s + (env: r.env, last: r.out)
}
// one line: calculated in the state, shown with view(out) (in context)
#let _calc(src, mode, cf, view) = {
  let req = (src: src, mode: mode, cfg: _req-cfg(cf))
  _st.update(s => _run(s, req))
  context view(_st.get().at("last"))
}
