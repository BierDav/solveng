// Reading math as typed: which letters are units and which are variables.
/// ## Units and variables
///
/// Many engineering symbols are also units: `A` (area, ampere), `V` (volume, volt), `T` (period, tesla), `h` (height, hour), `C`, `F`, `W`, `m`, `s`, `l`, `L`, … solveng keeps them apart by *how* they are written, never by guessing:
///
/// - **Math:** units are names, `$30 mA$`, `$4.7 kΩ$`, `$9.81 m/s^2$` (solveng brings every unit of several letters and every prefixed one, set upright); `unit("bar")` writes any other, also your own. A single letter is a unit right after a number (`$5 V$`, `$3 h$`, whatever is defined) or alone after a trailing `=` (`$t = h$`); anywhere else it's a variable, so `$3 dot h$` is 3 times h. Quoted text is a name of several letters (`$"tews" := 2$`), as Typst needs; `"as"` casts.
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

#import "display.typ": *

// tokens like _tk, plus quoted text ("mA") as one token
#let _tkq(s) = s.matches(regex("\"[^\"]*\"|\\[[^\\]]*\\]|\\d+\\.?\\d*(?:[eE][-+]?\\d+)?|\\.\\d+|@?\\p{L}[\\p{L}\\p{N}_]*|\\S")).map(m => m.text)
#let _quoted(k) = k != none and k.len() >= 2 and k.starts-with("\"") and k.ends-with("\"")
#let _unq(k) = if _quoted(k) { k.slice(1, -1) } else { k }

// "k"Omega, "k" "A": a quoted prefix belongs to the unit after it ("m" Omega is metre times ohm, "m"Omega milliohm);
// other quoted units glued stay apart ("A""as" is A·as)
#let _uglue(s, U) = {
  let join(m) = {
    let (p, sp, u) = m.captures
    if sp == "" or not _known(p, U) { "\"" + p + u.trim("\"") + "\"" } else { m.text }
  }
  s.replace(regex("\"(" + _P.keys().join("|") + ")\"(\\s*)(\"[^\"]*\"|Ω|\u{2126})"), join)
}
// a unit as math content (from _str(…, q: true)) -> (factor, dims, unit) of _uexpr, or an error
#let _math-unit(s, U) = if _E(s) { s } else { _uexpr(_uglue(s, U).replace("\"", " ").replace("∗", "*"), U: U) }
// one side of a math equation as typed: decimal commas (0,95) and quantities written without brackets
// (frac(3450/2 mAh, 30 mA) → [(3450)/(2) mAh]/[30 mA]). Unit names (mA, unit("bar"), read here in quotes) are units;
// a bare letter is a unit right after a number, whatever is defined ($3 h$ is 3 hours, $3 dot h$ 3 times h).
// bare: units may also follow a call or a name (f(20) Ω is f(20) times 1 Ω); only for values, not equations, where an undefined A is a variable
// -> a string _parse reads, or an error
#let _math-src(s, vars, cf, bare: false) = {
  let U = _UT(cf)
  // math sets * as ∗
  let s = s.replace(regex("(\d),(\d)"), m => m.captures.at(0) + "." + m.captures.at(1)).replace("∗", "*")
  let s = _uglue(s, U)
  let t = _tkq(s)
  let num(k) = k.match(regex("^[\d.]")) != none
  let name(k) = k != none and k.match(regex("^\p{L}")) != none
  let unit(k) = if _quoted(k) { _isu(_unq(k), U: U) } else { name(k) and _isu(k, U: U) }
  // a plain value: R = 7.2 kΩ
  let plain = t.map(_unq).join(" ")
  // in brackets: the unit is meant even if there is a variable with its name
  if _isq(plain, U: U) { return "[" + plain + "]" }
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
    let u = inner.map(_unq).join(" ").replace(regex("\^\s*\(\s*([-−]?)\s*(\d+)\s*\)"), m => "^" + m.captures.at(0).replace("−", "-") + m.captures.at(1))
    (u.replace(regex("[()]"), "").replace(regex("\s*\^\s*"), "^").trim(), e + 1)
  }
  // the unit of a cast or type at j, as math writes it: "A" "as", ("A" * "as")/"V", "m"^2 -> (unit, index after it), or none
  let urun(j) = {
    let atom(j) = {
      let k = t.at(j, default: none)
      let e = if unit(k) { j + 1 } else if k == "(" {
        let (dp, e) = (0, j)
        while e < t.len() {
          if t.at(e) == "(" { dp += 1 } else if t.at(e) == ")" { dp -= 1; if dp == 0 { break } }
          e += 1
        }
        let inner = t.slice(j + 1, calc.min(e, t.len()))
        if e < t.len() and inner.any(unit) and inner.all(k => unit(k) or k in ("^", "(", ")", "-", "−", "*", "·", "⋅", "/") or k.match(regex("^\d+$")) != none) { e + 1 }
      }
      if e == none { return none }
      // a power: ^2, ^(2), ^(-1), ^-1
      let p = t.slice(e, calc.min(e + 5, t.len()))
      let n = if p.at(0, default: none) != "^" { 0 } else if p.at(1, default: none) == "(" {
        let o = int(p.at(2, default: none) in ("-", "−"))
        if num(p.at(2 + o, default: "")) and p.at(3 + o, default: none) == ")" { 4 + o } else { 0 }
      } else {
        let o = int(p.at(1, default: none) in ("-", "−"))
        if num(p.at(1 + o, default: "")) { 2 + o } else { 0 }
      }
      e + n
    }
    let e = atom(j)
    if e == none { return none }
    while true {
      let op = t.at(e, default: none) in ("*", "·", "⋅", "/")
      let n = atom(e + int(op))
      if n == none { break }
      e = n
    }
    (t.slice(j, e).map(_unq).join(" "), e)
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
    // the exponent of a name before it, I^2 · 3 Ω: the factor starts after it
    if best != none and best > 0 and out.at(best - 1) == "^" {
      let (e, dd) = (best, 0)
      if out.at(e) == "(" { while e < out.len() { if out.at(e) == "(" { dd += 1 } else if out.at(e) == ")" { dd -= 1; if dd == 0 { break } }; e += 1 } }
      e += 1
      while e < out.len() and out.at(e) in ("*", "·", "⋅", "×") { e += 1 }
      best = if e < out.len() { e }
    }
    best
  }
  let out = ()
  let i = 0
  while i < t.len() {
    let k = t.at(i)
    let b = if (unit(k) or group(i) != none) and out.len() > 0 and (num(out.last()) or out.last() == ")") { start(out) }
    if b == none and bare and (unit(k) or group(i) != none) and out.len() > 0 and (out.last() == ")" or name(out.last())) { b = out.len() }
    // "as" after anything but a value casts (x "as" "mA"); after a number it's the unit, attosecond
    // with its unit in brackets, which casts read whole
    if b == none and _unq(k) == "as" {
      let r = urun(i + 1)
      out.push("as")
      if r != none { out.push("[" + r.at(0) + "]"); i = r.at(1) } else { i += 1 }
      continue
    }
    // a parameter's type, f(I: "mA"), and what isn't allowed but is read to say so: V: A, f(…): A
    if b == none and k == ":" and out.len() > 0 and (name(out.last()) or out.last() == ")") {
      let r = urun(i + 1)
      if r != none { out += (":", "[" + r.at(0) + "]"); i = r.at(1); continue }
    }
    // a quoted unit on its own: "A" alone is one ampere
    if b == none and _quoted(k) and unit(k) { b = out.len() }
    if b == none {
      out.push(_unq(k))
      i += 1
      continue
    }
    let (u, j) = ((), i)
    let go = true
    while go and j < t.len() {
      let a = t.at(j)
      // as after a unit casts (2 "A" "as" "mA"); right after the number it's attoseconds
      if unit(a) and not (_unq(a) == "as" and u.len() > 0) {
        u.push(_unq(a)); j += 1
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
      } else if group(j) != none {
        let (g, e) = group(j)
        u.push(g); j = e
      }
      else if a == "/" and (unit(t.at(j + 1, default: none)) or group(j + 1) != none) { u.push("/"); j += 1 }
      else { go = false }
    }
    let f = out.slice(b).join()
    // [A]: a unit without a number is shown as just the unit
    out = out.slice(0, b) + ("[" + (if f in (none, "") { "" } else { f + " " }) + u.join(" ") + "]",)
    i = j
  }
  out.join(" ")
}
// a whole math equation or inequality (from _str(…, q: true)): each side read with _math-src -> string, or an error
#let _math-eq(s, vars, cf) = {
  let rs = s.matches(regex("<=|>=|[=<>≤≥]"))
  let (out, at) = ("", 0)
  for r in rs + ((start: s.len(), end: s.len(), text: ""),) {
    let side = _math-src(s.slice(at, r.start), vars, cf)
    if _E(side) { return side }
    out += side.trim() + if r.text != "" { " " + r.text + " " }
    at = r.end
  }
  out
}
