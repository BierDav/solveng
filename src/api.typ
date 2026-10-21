// worksheet, and def and solve, the calls behind code lines (and the tests).
#import "core.typ": *

#let _STEPS = ("full", "formula", "result", "value")
// typed display arguments -> local options, or an error message
#let _disp(steps, layout, inline) = {
  if steps != auto and steps not in _STEPS { return _err("steps: use " + _STEPS.map(repr).join(", ")) }
  if layout != auto and layout not in ("row", "column") { return _err("layout: use \"row\" or \"column\"") }
  if type(inline) != bool { return _err("inline: true or false") }
  let o = (:)
  if inline { o.block = false }
  if steps != auto { o.steps = steps }
  if layout != auto { o.layout = layout }
  o
}

// defines from equations as strings, silently (the string API behind code lines, kept for the tests)
#let def(
  ..eqs,
  unit: none, output: false, steps: auto, layout: auto, inline: false,
  unknown: none, root: auto, guess: none, listed: true,
  digits: auto, trailing-zeros: auto, separator: auto, grouping: auto, scientific: auto, engineering: auto, multiplication: auto, form: auto, max-denominator: auto, fractions: auto,
  prefixes: auto, show-si: auto, preferred-units: auto, strict: auto, inline-steps: auto, implies: auto, approximate: auto, block: auto, max-solutions: auto,
  defined-style: auto, errors: auto,
) = {
  if eqs.named().len() > 0 {
    let k = eqs.named().keys().first()
    return _err-now("unknown argument '" + k + "' (arguments: unit, output, steps, layout, inline, unknown, root, guess, listed and the options " + _DEF.keys().filter(k => k not in ("steps", "layout", "units", "integration", "implicit-parameters")).join(", ") + ")")
  }
  let lc = _check(_given((
    digits: digits, trailing-zeros: trailing-zeros, separator: separator, grouping: grouping, scientific: scientific, engineering: engineering, multiplication: multiplication, form: form, max-denominator: max-denominator, fractions: fractions,
    prefixes: prefixes, show-si: show-si, preferred-units: preferred-units, strict: strict, inline-steps: inline-steps, implies: implies, approximate: approximate, block: block, max-solutions: max-solutions,
    defined-style: defined-style, errors: errors,
  )))
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
  // math keeps its quotes, so "V" (a unit) and V (a variable) stay apart; it is read once the variables are known
  let strs = pos.map(x => if type(x) == content and x.func() == math.equation { let m = _str(x, q: true); if _E(m) { m } else { (math: m) } } else { _str(x) })
  let b = _first-err(strs)
  if b != none { return _err-now(b.err) }
  // a unit as the 2nd positional argument: solve("Q0/I", "h")
  for s in strs.slice(1).filter(s => type(s) == str).map(str.trim) {
    if s != "" and not s.contains("=") and _isu(s) { return _err-now("'" + s + "' looks like a unit – pass it as unit: \"" + s + "\"") }
  }
  // inline without explicit steps: the shorter inline-steps
  let lc = lc + (if inline and steps == auto { (steps: auto) } else { (:) }) + dl
  // names given on their own read like those in the equations: unknown: "tau" is τ
  let unknown = if type(unknown) == str { _resolve(unknown) } else if type(unknown) == array { unknown.map(_resolve) } else { unknown }
  _def(strs, _resolve-keys(un), (unknown: unknown, root: root, guess: _resolve-keys(guess)), lc, output, listed)
}
// def with its output: equation ⇒ rearranged formula = numbers = result
#let solve(
  ..eqs,
  unit: none, output: true, steps: auto, layout: auto, inline: false,
  unknown: none, root: auto, guess: none, listed: true,
  digits: auto, trailing-zeros: auto, separator: auto, grouping: auto, scientific: auto, engineering: auto, multiplication: auto, form: auto, max-denominator: auto, fractions: auto,
  prefixes: auto, show-si: auto, preferred-units: auto, strict: auto, inline-steps: auto, implies: auto, approximate: auto, block: auto, max-solutions: auto,
  defined-style: auto, errors: auto,
) = if eqs.pos().any(x => { let t = if type(x) == content { _str(x) } else { x }; type(t) == str and (t.contains(":=") or t.contains("≔")) }) {
  // solve takes what's true; assigning is def's
  _err-now("solve takes equations, what's true, not assignments – define with def(\"… := …\") or in math, then solve")
} else { def(
  ..eqs,
  unit: unit, output: output, steps: steps, layout: layout, inline: inline,
  unknown: unknown, root: root, guess: guess, listed: listed,
  digits: digits, trailing-zeros: trailing-zeros, separator: separator, grouping: grouping, scientific: scientific, engineering: engineering, multiplication: multiplication, form: form, max-denominator: max-denominator, fractions: fractions,
  prefixes: prefixes, show-si: show-si, preferred-units: preferred-units, strict: strict, inline-steps: inline-steps, implies: implies, approximate: approximate, block: block, max-solutions: max-solutions,
  defined-style: defined-style, errors: errors,
) }

// expression tree -> string _parse reads back the same
#let _src(e) = {
  let (k, ..a) = e
  let w(x) = "(" + _src(x) + ")"
  if k == "n" { repr(float(a.first())) }
  else if k in ("c", "v") { a.first() }
  else if k == "as" { "(" + _src(a.first()) + ") as [" + a.at(1).u + "]" }
  else if k == "q" { let q = a.first(); if q.u == "" { repr(float(q.n)) } else { "[" + repr(float(q.n)) + " " + q.u + "]" } }
  else if k == "+" { a.map(w).join(" + ") }
  else if k == "*" { a.map(w).join(" * ") }
  else if k == "-" { "-" + w(a.first()) }
  else if k in ("/", "^") { w(a.first()) + " " + k + " " + w(a.last()) }
  else if k == "tuple" { a.map(_src).join(", ") }
  else if k == "cmp" { w(a.at(1)) + " " + a.first() + " " + w(a.at(2)) }
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
// a math equation in a worksheet: "I = 30 mA" is defined silently, the equation stays as written.
// Equations without '=', derivations (⇒, ≈, <, …) or content that isn't a formula (alignment, line breaks, calls) are left alone.
// math with its casts (U "as" A) and types (f(x: "mA")) as chips, plain or left out, as the options say, their
// units simplified; "as" after a number is the unit (3 "as"), a colon before something that isn't a unit stays
#let _restyle(c, cf, U) = {
  if type(c) != content { return c }
  let f = c.func()
  let space(x) = x == none or x.func() == [ ].func()
  // letters and : are symbols in math, quoted text is text
  let word(x) = if x.has("text") and type(x.text) == str { x.text } else { none }
  if c.has("children") {
    let (out, ch, i) = ((), c.children, 0)
    while i < ch.len() {
      let x = ch.at(i)
      let mark = word(x)
      if mark in ("as", ":") {
        let before = out.rev().find(y => not space(y))
        let number = before != none and word(before) != none and word(before).match(regex("\\d$")) != none
        // the unit: the longest run of children after the mark that reads as one ("A" "as", ("A" * "as")/"V")
        let (k, end, us) = (i + 1, none, none)
        while k < ch.len() and not (mark == "as" and number) {
          let w = word(ch.at(k))
          if not space(ch.at(k)) and w not in ("∗", "*", "·", "⋅", "/") {
            let src = ch.slice(i + 1, k + 1).map(y => _str(y, q: true)).fold("", (a, b) => if _E(a) or _E(b) { _err("") } else { a + b })
            // a type may also be any, every unit
            let r = if mark == ":" and type(src) == str and src.trim() == "\"any\"" { (1, _Z, "any") } else { _math-unit(src, U) }
            if _E(r) { break }
            (end, us) = (k + 1, r.at(2))
          }
          k += 1
        }
        if end != none {
          while out.len() > 0 and space(out.last()) { let _ = out.pop() }
          out.push(_mark(mark, us, cf))
          i = end
          continue
        }
      }
      // a letter after a number is a unit, set upright as units are: 5 V
      let before = out.rev().find(y => not space(y))
      let named = x.func() == text and x.has("label") and x.label == <solveng-unit>
      if (named or (x.func() != text and mark != none and mark.clusters().len() == 1 and _known(mark, U))) and before != none and word(before) != none and word(before).match(regex("\\d$")) != none {
        while out.len() > 0 and space(out.last()) { let _ = out.pop() }
        out.push(h(0.1667em) + math.upright(mark))
        i += 1
        continue
      }
      out.push(_restyle(x, cf, U))
      i += 1
    }
    return out.join()
  }
  if f == math.lr { return math.lr(_restyle(c.body, cf, U)) }
  if f == math.frac { return math.frac(_restyle(c.num, cf, U), _restyle(c.denom, cf, U)) }
  if f == math.root { return math.root(..(if c.has("index") { (c.index,) } else { () }), _restyle(c.radicand, cf, U)) }
  c
}
// an equation with its casts and types restyled, marked so solveng's rules leave it alone (it's already read); the
// equation itself when nothing changes
#let _RESHOWN = metadata("solveng-reshown")
#let _reshown(eq) = eq.body.has("children") and eq.body.children.any(c => c == _RESHOWN)
#let _reshow(eq, cf, U) = {
  let body = _restyle(eq.body, cf, U)
  if body == eq.body { return eq }
  let fs = eq.fields()
  let _ = fs.remove("body")
  math.equation(..fs, _RESHOWN + body)
}
// the equation itself, marked as read so solveng's rules leave it alone
#let _as-shown(eq) = if _reshown(eq) { eq } else {
  let fs = eq.fields()
  let _ = fs.remove("body")
  math.equation(..fs, _RESHOWN + eq.body)
}
// an equation without a clause: := defines, = states (checked when everything is known, else prose)
// quiet: only problems are shown, not the equation
#let _math-def0(it, quiet: false) = context {
  if _busy.get() > 0 or _reshown(it) or _own(it) { return it }
  let cf = _cf.get()
  let U = _UT(cf)
  // the equation as written, its casts and types as chips (or left out)
  let src = _str(it.body, q: true)
  let shown = if quiet { [] } else if not _E(src) { _reshow(it, cf, U) } else { it }
  let shown = if quiet { [] } else { _as-shown(shown) }
  let s = _str(it.body, q: true)
  if _E(s) or s.match(regex("[⇒⇔⟹⟺→←↔≈≠≡∼∝∈]")) != none { return shown }
  let fail(e) = shown + _errc(e.err, cf, wide: it.block)
  // f(…) := body (≔ in math): the body as written, which may compare, U(x, y) := x = y
  let fm = s.split("≔")
  // and a variable assigned a comparison, V := a < b: whether it holds
  let stmt = fm.len() == 2 and fm.first().trim().match(regex("^\\p{L}[\\p{L}\\p{N}_]*$")) != none and fm.last().match(regex("[<>≤≥≠]")) != none
  if stmt or (fm.len() == 2 and fm.first().trim().match(regex("^\\p{L}[\\p{L}\\p{N}_]*\\s*\\(")) != none) {
    let vars = _vars-of(_st.get())
    let (l, r) = (_math-src(fm.first(), vars, cf), _math-eq(fm.last(), vars, cf))
    if _E(l) { return fail(l) }
    if _E(r) { return fail(r) }
    return shown + _def((l.trim() + " := " + r.trim(),), none, (:), (block: it.block), false, true)
  }
  // inequalities are solved with -> solve(x), else they're prose
  if s.match(regex("[<>≤≥]")) != none { return shown }
  // := (≔ in math) assigns, = states
  let assign = s.contains("≔")
  let p = s.replace("≔", "=").split("=")
  if p.len() < 2 or p.any(x => x.trim() == "") { return shown }
  // a worked chain R = 7.2 kΩ = 7200 Ω: the first '=' defines, the rest are steps
  let vars = _vars-of(_st.get())
  let ms = p.map(x => _math-src(x, vars, cf))
  let b = _first-err(ms)
  if b != none { return fail(b) }
  let ms = ms.map(str.trim)
  let lhs = ms.at(0)
  let fns = _st.get().at("#fn", default: (:))
  if not assign {
    // = states: with everything known its sides must agree, else it's prose (:= defines, -> solve(x) solves)
    let vals = ms.map(x => { let e = _parse(x, vars, f: fns, U: U); if not _E(e) and _vars(e).all(v => v in vars) { _ev(e, vars) } })
    if vals.any(v => v == none or _E(v)) { return shown }
    let r0 = vals.first()
    let round(dm) = dm.map(z => calc.round(z, digits: 6))
    let i = vals.position(r => round(r.at(1)) != round(r0.at(1)) or calc.abs(r.at(0) - r0.at(0)) > 1e-3 * calc.max(calc.abs(r.at(0)), calc.abs(r0.at(0))))
    if i == none { return shown }
    let tidy(x) = x.replace(regex("[\\[\\]\"]"), "").trim()
    let why = if round(vals.at(i).at(1)) != round(r0.at(1)) { "has another unit" } else { "has another value" }
    return fail(_err("'" + tidy(p.at(i)) + "' " + why + " than '" + tidy(p.first()) + "' – = states what's true; := defines"))
  }
  // a variable has one value: the last step of a chain gives it (R = 7.2 kΩ = 7200 Ω), unless it needs what isn't known
  let last = if ms.len() > 2 { ms.slice(1).rev().position(x => _isq(x, U: U) or { let e = _parse(x, vars, f: fns, U: U); not _E(e) and _vars(e).all(v => v in vars) }) }
  // a chain goes whole: the steps are checked, the last gives the value, shown in the unit that fits
  let whole = ms.len() > 2 and last != none
  let eq = lhs + (if assign { " := " } else { " = " }) + if whole { ms.slice(1).join(" = ") } else { ms.at(1) }
  // steps that write an undefined variable out define it: R = ρ l/A = ρ l/(b h) gives A = b h
  let es = ms.slice(1).map(x => _parse(x, vars, f: fns, keep: true, U: U))
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
  let le = _parse(lhs, vars, f: fns, U: U)
  let k = ds.position(c => c == none)
  let k = if k == none { ds.len() } else { k }
  if imp.len() > 0 and not _E(le) {
    let n(i) = (_vars(le) + _vars(es.at(i))).dedup().filter(v => v not in known).len()
    let best = range(k + 1).sorted(key: i => (n(i), -i)).first()
    eq = lhs + (if assign { " := " } else { " = " }) + ms.at(best + 1)
    whole = false
  }
  let one(x, e) = _def((x + " = " + _src(e),), none, (:), (block: it.block), false, true)
  // the steps must agree: those with only known values have the first one's unit and value (rounded ones within 0.1 %)
  let lv = _tk(lhs.split(":").first()).filter(t => t.match(regex("^\\p{L}")) != none).map(_greek)
  // a step's value: a plain quantity (7300 Ω, as _math-src leaves it) or a formula of known values
  let val(i, e) = if _isq(ms.at(i + 1), U: U) { let q = _q(ms.at(i + 1), U: U); if _E(q) { q } else { (q.v, q.d) } } else if _E(e) or not _vars(e).all(v => v in vars and v not in lv) { _err("") } else { _ev(e, vars) }
  let vals = es.enumerate().map(((i, e)) => (i, val(i, e))).filter(((i, r)) => not _E(r))
  let bad = vals.slice(calc.min(1, vals.len())).find(((i, r)) => {
    let r0 = vals.first().at(1)
    r.at(1) != r0.at(1) or calc.abs(r.at(0) - r0.at(0)) > 1e-3 * calc.max(calc.abs(r.at(0)), calc.abs(r0.at(0)))
  })
  // a whole chain is checked where it's defined
  let bad = if whole { none } else { bad }
  if bad != none {
    let (i0, i) = (vals.first().at(0), bad.at(0))
    let why = if bad.at(1).at(1) != vals.first().at(1).at(1) { "has another unit" } else { "has another value" }
    let tidy(x) = x.replace(regex("[\\[\\]\"]"), "").trim()
    shown = shown + _errc("'" + tidy(ms.at(i + 1)) + "' " + why + " than '" + tidy(ms.at(i0 + 1)) + "' in '" + p.map(tidy).join(" = ") + "' – the steps of an equation must be equal (≈ for rounded ones)", cf, wide: it.block)
  }
  shown + now.map(((x, e)) => one(x, e)).join() + _def((eq,), none, (:), (block: it.block), false, true) + imp.filter(c => c not in now).map(((x, e)) => one(x, e)).join()
}
// ---------- the clause after a trailing = or ->: a unit and keywords ----------
#let _KEYS = ("solve", "simplify", "digits", "sci", "eng", "frac")
// items ((key: …, args: (str, …)) or (unit: …)) -> (unit, local options, what to solve for), or an error
#let _clause(items, U) = {
  let (un, lc, xs) = (none, (:), none)
  for it in items {
    if "unit" in it {
      if un != none { return _err("one unit per result, got " + un + " and " + it.unit) }
      un = it.unit
      continue
    }
    let (k, a) = (it.key, it.args)
    if k not in _KEYS { return _err("unknown keyword '" + k + "' (keywords: " + _KEYS.join(", ") + ")") }
    let n = (solve: a.len() > 0, digits: a.len() == 1).at(k, default: a.len() == 0)
    if not n { return _err(if k == "solve" { "solve: what to solve for, solve(x) or solve(x, y)" } else if k == "digits" { "digits: the number of digits, digits(3)" } else { k + " takes no arguments" }) }
    if k == "solve" { xs = a.map(x => _resolve(x.trim())) }
    else if k == "digits" {
      let d = a.first().trim()
      if d.match(regex("^\d+$")) == none or int(d) < 1 or int(d) > 15 { return _err("digits: 1…15, got " + d) }
      lc += (form: "decimal", digits: int(d))
    }
    else if k == "frac" { lc.form = "fraction" }
    // in the base unit, 2.2 · 10⁻⁶ F
    else if k == "sci" { lc += (scientific: (0, 1), prefixes: none) }
    else if k == "eng" { lc += (engineering: true, scientific: (0, 1), prefixes: none) }
  }
  (unit: un, lc: lc, solve: xs)
}
// a unit written in a clause has no number (but exponents): = 2 A is a step, = A a unit
#let _unitish(s) = s.replace(regex("\^\s*\(?\s*[-−]?\d+\s*\)?"), "").match(regex("\d")) == none
// the clause in math: children after the marker, split at commas -> items, or none if it isn't one
#let _citems(cs, U) = {
  let groups = ((),)
  for c in cs { if c.has("text") and c.text == "," { groups.push(()) } else { groups.last().push(c) } }
  let items = ()
  for g in groups {
    let g = g.filter(c => c.func() != [ ].func())
    if g.len() == 0 { if groups.len() == 1 { continue } else { return none } }
    let x = g.first()
    if g.len() == 1 and x.func() == metadata and x.has("label") and x.label == <solveng-key> {
      items.push((key: x.value.key, args: x.value.args.map(a => { let t = _str(a); if _E(t) { "?" } else { t } })))
      continue
    }
    // Typst's own frac, which math shows as its name
    if g.len() == 1 and x.func() == raw and x.text == "frac" { items.push((key: "frac", args: ())); continue }
    let s = _str(g.join(), q: true)
    if _E(s) or not _unitish(s) { return none }
    let r = _math-unit(s, U)
    if _E(r) { return none }
    items.push((unit: r.at(2)))
  }
  items
}
// an equation's children -> (head, marker, clause): at the last top-level ->, else at the last top-level =
#let _split-eq(body) = {
  let cs = if body.has("children") { body.children } else { (body,) }
  let at(t) = range(cs.len()).filter(i => cs.at(i).has("text") and cs.at(i).text == t)
  let ar = at("→")
  let eq = at("=")
  let (i, m) = if ar.len() > 0 { (ar.last(), "→") } else if eq.len() > 0 { (eq.last(), "=") } else { return (cs, none, ()) }
  (cs.slice(0, i), m, cs.slice(i + 1))
}
// the evaluating =, bold as in Mathcad: stroked, since math fonts have no bold =
#let _BEQ = math.class("relation", context text(stroke: (paint: text.fill, thickness: 0.045em), "="))
// a solved record's results, x = 2 or x₁ = −2, x₂ = 2
#let _result(r, cf) = {
  let st = _stages(r, cf).last()
  // an inequality's result: R ≤ 166.7 Ω
  let b = st.at("rel", default: none)
  st.segs.map(((lhs, ps)) => [#lhs#ps.slice(calc.max(ps.len() - 1, 0)).map(((rel, x, _)) => [#if b != none { _REL.at(b) } else { rel } #x]).join()]).join(st.join)
}
// the result shown after the equation as written, and the problems of the last calculation
// errs: the calculation was solveng's (its problems are in its records)
#let _shown-with(it, head, mark, res, quiet, errs: true) = context {
  let cf = _cf.get()
  let rs = if errs { _st.get().at("..", default: ()) } else { () }
  let errs = rs.filter(r => r.k == "error").map(r => _errc(r.err, cf, wide: it.block)).join()
  if quiet { return errs }
  let U = _UT(cf)
  let fs = it.fields()
  let _ = fs.remove("body")
  let m = if mark == "=" { _BEQ } else { $->$.body }
  math.equation(..fs, _RESHOWN + _restyle(head, cf, U) + $#m #res(rs.filter(r => r.k != "error"), cf)$.body) + errs
}
// a math equation in a worksheet: := defines (≔ in math), a trailing = shows the value (in the unit and form its
// clause gives, $t = h, digits(3)$), -> the symbolic result or, with solve(x), solves for x
#let _math-def(it, quiet: false) = context {
  if _busy.get() > 0 or _reshown(it) or _own(it) { return it }
  let cf = _cf.get()
  let U = _UT(cf)
  let (head, mark, tail) = _split-eq(it.body)
  let items = if mark == none { () } else { _citems(tail, U) }
  // = followed by more than a unit and keywords is a step, $V := 2 A = 2000 mA$; an arrow among other words prose
  if mark == none or items == none { return _math-def0(it, quiet: quiet) }
  let fail(e) = (if quiet { none } else { _as-shown(it) }) + _errc(e.err, cf, wide: it.block)
  let c = _clause(items, U)
  if _E(c) { return fail(c) }
  let lc = c.lc + (block: it.block)
  while head.len() > 0 and head.last().func() == [ ].func() { let _ = head.pop() }
  let hb = head.join()
  let hs = if hb == none { _err("") } else { _str(hb, q: true) }
  if _E(hs) or hs.trim() == "" { return fail(_err("nothing before the " + mark)) }
  let assign = hs.contains("≔")
  let (lhs, rhs) = if assign { let p = hs.split("≔"); (p.first().trim(), p.slice(1).join("≔")) } else { (none, hs) }
  let fn = assign and lhs.contains("(")
  // what it defines, silently
  let defs = if assign { _math-def0(math.equation(block: it.block, hb), quiet: true) }
  if c.solve != none {
    if assign { return fail(_err("solve(…) solves an equation, which states something: write it without :=")) }
    let xs = c.solve
    return {
      context {
        let e = _math-eq(rhs, _vars-of(_st.get()), cf)
        if _E(e) { _errc(e.err, cf, wide: it.block) } else { _def((e,), c.unit, (unknown: if xs.len() == 1 { xs.first() } else { xs }), lc, none, true) }
      }
      _shown-with(it, hb, mark, (rs, cf) => rs.map(r => _result(r, cf + lc)).join($, quad$), quiet)
    }
  }
  if mark == "=" {
    if fn { return fail(_err("a function has no value of its own – evaluate a call, " + lhs.replace(regex("\(.*$"), "") + "(2) =")) }
    // a statement before the = is checked, its last side shown
    let checks = if not assign and rhs.contains("=") { _math-def0(math.equation(block: it.block, hb), quiet: true) }
    let target = if assign { lhs } else { rhs.split("=").last() }
    return defs + checks + context {
      let src = _math-src(target, _vars-of(_st.get()), cf, bare: true)
      if _E(src) { _errc(src.err, cf, wide: it.block) } else { _def((src,), c.unit, (:), lc, none, false) }
    } + _shown-with(it, hb, mark, (rs, cf) => rs.filter(r => "q" in r).map(r => _qc(r.q, cf + lc)).join(), quiet)
  }
  // ->: what's written with the known values put in, exact; a value once nothing unknown is left
  let src = if assign { rhs.split("=").first() } else { rhs.split("=").last() }
  let params = if fn { _tk(lhs.slice(lhs.position("(") + 1)).filter(t => t.match(regex("^\p{L}")) != none).map(_greek) } else { () }
  defs + _shown-with(it, hb, mark, (rs, cf) => {
    let cf = cf + (form: "fraction") + lc
    let vars = _vars-of(_st.get())
    let ms = _math-src(src, vars, cf)
    if _E(ms) { return $?$ }
    let e = _parse(ms, vars, f: _st.get().at("#fn", default: (:)), U: U)
    if _E(e) { return $?$ }
    let e = _s(_subst(e, vars.pairs().filter(((k, _)) => k not in params).map(((k, q)) => (k, ("q", q))).to-dict()))
    if _vars(e).len() > 0 { return _dp(e, none, cf).at(0) }
    let v = _ev(e, (:))
    let q = if _E(v) { v } else { _mkq(v.at(0), v.at(1), c.unit, cf) }
    if _E(q) { $?$ } else { _qc(q, cf) }
  }, quiet, errs: false)
}

// one line of a code block, read like math: x := … defines, a trailing = shows a value, -> the formula or solves
// (-> solve(x)); units in a clause in brackets or bare ([mA], mA), ';' separates a system, // starts a comment
#let _cut(s) = {
  let ar = s.matches("->")
  if ar.len() > 0 { let m = ar.last(); return (s.slice(0, m.start), "→", s.slice(m.end)) }
  let eq = s.matches(regex("[:<>!=]?=")).filter(m => m.text == "=")
  if eq.len() > 0 { let m = eq.last(); return (s.slice(0, m.start), "=", s.slice(m.end)) }
  (s, none, "")
}
// a clause in code -> items, or none if it isn't one
#let _sitems(s, U) = {
  let (parts, d, cur) = ((), 0, "")
  for ch in s.clusters() {
    if ch in ("(", "[") { d += 1 } else if ch in (")", "]") { d -= 1 }
    if ch == "," and d == 0 { parts.push(cur); cur = "" } else { cur += ch }
  }
  let parts = (parts + (cur,)).map(str.trim)
  if parts == ("",) { return () }
  let items = ()
  for p in parts {
    let m = p.match(regex("^(\p{L}+)\s*(?:\((.*)\))?$"))
    if m != none and m.captures.first() in _KEYS {
      items.push((key: m.captures.first(), args: if m.captures.at(1) == none { () } else { m.captures.at(1).split(",").map(str.trim) }))
      continue
    }
    let u = p.trim("[", at: start).trim("]", at: end).trim()
    if u == "" or not _unitish(u) or not _isu(u, U: U) { return none }
    items.push((unit: u))
  }
  items
}
#let _line(s, block, quiet: false) = {
  let s = s.split("//").first().trim()
  if s == "" { return none }
  context {
    let cf = _cf.get()
    let U = _UT(cf)
    let (head, mark, tail) = _cut(s)
    let items = if mark == none { () } else { _sitems(tail, U) }
    let (head, mark, items) = if mark == "=" and items == none { (s, none, ()) } else { (head, mark, items) }
    if items == none { return _errc("-> takes keywords and a unit: -> solve(x), -> simplify, -> digits(3), [mA] – in '" + s + "'", cf, wide: block) }
    let c = _clause(items, U)
    if _E(c) { return _errc(c.err + " – in '" + s + "'", cf, wide: block) }
    let lc = c.lc + (block: block)
    let out = not quiet
    let eqs = head.split(";").map(str.trim).filter(x => x != "")
    if eqs.len() == 0 { return _errc("nothing before the " + mark + " in '" + s + "'", cf, wide: block) }
    let assign = head.contains(":=")
    if c.solve != none { return _def(eqs, c.unit, (unknown: if c.solve.len() == 1 { c.solve.first() } else { c.solve }), lc, out, true) }
    if assign { return _def(eqs, c.unit, (:), lc + if mark == "→" { (steps: "formula") } else { (:) }, out, true) }
    if mark != none { return _def(eqs, c.unit, (:), lc + (steps: if mark == "=" { "result" } else { "formula" }), out, false) }
    // = states: checked against what's known, never defines
    let vars = _vars-of(_st.get())
    let fns = _st.get().at("#fn", default: (:))
    let shown = ()
    for e in eqs {
      let sides = e.split("=")
      if sides.len() < 2 { return _errc("'" + e + "' does nothing – x := … defines, a trailing = shows a value", cf, wide: block) }
      let es = sides.map(x => _parse(x, vars, f: fns, U: U))
      let b = _first-err(es)
      if b != none { return _errc(b.err, cf, wide: block) }
      let free = es.map(_vars).flatten().dedup().filter(v => v not in vars)
      if free.len() > 0 { return _errc(free.map(v => "'" + v + "'").join(", ") + " in '" + e + "' isn't defined – define it with :=, or solve for it: " + e + " -> solve(" + free.first() + ")", cf, wide: block) }
      let vals = es.map(x => _ev(x, vars))
      let b = _first-err(vals)
      if b != none { return _errc(b.err, cf, wide: block) }
      let round(dm) = dm.map(z => calc.round(z, digits: 6))
      let r0 = vals.first()
      if vals.any(r => round(r.at(1)) != round(r0.at(1)) or calc.abs(r.at(0) - r0.at(0)) > 1e-3 * calc.max(calc.abs(r.at(0)), calc.abs(r0.at(0)))) {
        return _errc("'" + e + "' doesn't hold – = states what's true; := defines", cf, wide: block)
      }
      shown.push(es.map(x => _dp(x, none, cf).at(0)).join($=$))
    }
    if out { _eqn(shown.join($, quad$), cf + lc) }
  }
}
// ---------- what editors read ----------
// Typewright and other editors query <solveng> metadata after compiling: per element worksheet saw (by its
// location, which maps back to the source) whether it was calculated, per block what it defined, with values.
#let _vstr(q, cf) = {
  let (m, e) = _fmt(q.n, cf + (digits: calc.max(cf.digits, 4)))
  let rel = q.at("rel", default: none)
  (if rel != none { rel + " " } else { "" }) + m + (if e != none { "e" + str(e) } else { "" }) + (if q.u != "" { " " + q.u } else { "" })
}
#let _defs-since(before, st, cf) = {
  let (fb, fa) = (before.at("#fn", default: (:)), st.at("#fn", default: (:)))
  st.at("!", default: ()).filter(e => if e.kind == "function" { fa.at(e.name, default: none) != fb.at(e.name, default: none) } else { e.name in st and st.at(e.name) != before.at(e.name, default: none) }).map(e => {
    let value = if e.kind == "function" { let f = fa.at(e.name); let us = f.at("units", default: (:)); let anys = f.at("anys", default: ()); "(" + f.params.map(p => if p in us { p + ": " + us.at(p).u } else if p in anys { p + ": any" } else { p }).join(", ") + ")" + (if f.at("ret", default: none) != none { ": " + f.ret } else { "" }) + " = " + f.at("src", default: _src(f.body)) } else { _vstr(st.at(e.name), cf + e.cf) }
    (name: e.name, kind: e.kind, value: value)
  })
}
// everything defined so far in the block, also by calls, and the units
#let _block-note = context {
  let cf = _cf.get()
  let units = cf.units.pairs().filter(((k, v)) => v != none).map(((k, v)) => (name: k, value: if v == auto { "auto" } else if type(v) in (str, int, float) { str(v) } else { "" }))
  [#metadata((kind: "block", defined: _defs-since((:), _st.get(), cf), units: units)) <solveng>]
}
// the element `it` shown as `body`, followed by its record; `cf` is the configuration around it. What is defined
// is listed once per block (reading the state after every element would take Typst more layout passes).
#let _noted(it, kind, active, cf, body) = {
  let loc = it.location()
  if loc == none { return body }
  body + [#metadata((kind: kind, element: loc, active: active, integration: cf.integration)) <solveng>]
}

// how many worksheets are around, kept with the configuration that every element reads anyway
#let _blocks(cf) = cf.at("#blocks", default: 0)
// `b` with math (as worksheet-config(integration: …) says where each element is) and solveng code blocks calculated
#let _rules(b, reset) = context {
  // raw text is set smaller, monospaced and in English; definitions use the surrounding text and language
  let (size, font, lang, region) = (text.size, text.font, text.lang, text.region)
  show raw.where(lang: "solveng", block: true): it => context {
    _noted(it, "code", true, _cf.get(), text(size: size, font: font, lang: lang, region: region, it.text.split("\n").map(l => _line(l, true)).filter(x => x != none).join()))
  }
  show std.math.equation: it => context {
    if _busy.get() > 0 or _reshown(it) or _own(it) { return it }
    let cf = _cf.get()
    let on = cf.integration == "math"
    _noted(it, "math", on, cf, if on { _math-def(it) } else { it })
  }
  // start over at each heading (or whatever reset selects): back to the state at the start of the worksheet
  show selector(if reset == none { <solveng-never> } else { reset }): it => {
    if reset != none { _block-note + _st.update(_restart) }
    it
  }
  b
}

/// Calculates the math you write, like a Mathcad worksheet.
///
/// `#show: worksheet` calculates the rest of the document, `#worksheet[…]` its content (whose definitions are dropped at its end), and `#worksheet($…$)` single equations, also outside a worksheet.
///
/// ```typ
/// #show: worksheet
/// The battery holds $Q_0 := 3450 mAh$ and the load draws $I := 30 mA$,
/// so it lasts $t := Q_0 / I = h$.
/// ```
///
/// **Math.** Three operators, as in Mathcad:
///
/// - `:=` defines (shown as ≔): `$I := 30 mA$`, `$P := U I$`, `$f(x) := x^2 + 1$`. Steps after it are checked: `$R := 7.2 kΩ = 7200 Ω$`.
/// - `=` at the end shows the value, with a bold =: `$t =$` → t **=** 115 h, `$t := Q_0/I =$` defines and shows it. What follows it is the unit and keywords: `$t = h$`, `$t = min, digits(2)$`.
/// - `->` at the end shows the symbolic result, the known values put in, exactly: `$f := (x^2 - 1)/(x - 1) -> simplify$`. With `solve(x)` it solves an equation for x (and defines it): `$Q_0/2 = I t -> solve(t)$`, `$x^2 - 4 = 0 -> solve(x)$`, `$x + y = 3, x - y = 1 -> solve(x, y)$`, `$R <= U/I -> solve(R)$`.
///
/// Keywords: `solve(x, …)`, `simplify`, `frac` (exact fractions), `digits(n)` (decimals with n significant digits), `sci` (scientific notation), `eng` (engineering notation). Any other equation with `=` states something: it's checked when everything in it is known (`$U = R I$` after defining them) and left alone as prose when not (`$E = m c^2$`).
///
/// **Units** are names: `$30 mA$`, `$4.7 kΩ$`, `$9.81 m/s^2$`, and `unit("bar")` for units without one (your own, from `worksheet-config(units: …)`). A single letter is a unit right after a number (`$5 V$`, `$3 h$`) or alone after a trailing `=` (`$t = h$`); anywhere else it's a variable, and `$3 dot h$` is 3 times a variable h.
///
/// **Code.** Each line of a ```` ```solveng ```` code block is calculated the same way and shown with all its steps: `t := Q0/I = [h]`, `Q0/I =`, `x^2 = 4 -> solve(x)`; units in brackets, `;` separates the equations of a system, `//` starts a comment. Inline code and code with another language are left alone.
///
/// **Calls.** `#worksheet($U := 5 V$, $I := U/R =$)` calculates its equations (and strings, read like a code line) where it stands, `hidden: true` without showing them. Its options are for these only; with content, for the content.
///
/// **Editors.** Each equation and code block the worksheet sees leaves a `metadata` labelled `<solveng>` with its `location()`, whether it was calculated and the `integration` there; at the end (and before each `reset`) one lists everything defined, with values, and the units. Editors like Typewright read them after compiling: `query(<solveng>)`.
///
/// **Starting over.** `reset` starts over at every match; `#reset` inside a worksheet does the same once: back to how it started (with `inherit: true` the outer definitions stay). After a `#worksheet[…]` the outer definitions come back.
///
/// ```typ
/// #show: worksheet.with(reset: heading)                    // every heading
/// #show: worksheet.with(reset: heading.where(level: 1))    // only top-level ones
/// #show: worksheet.with(reset: heading, inherit: true)     // keep the definitions made before
/// ```
///
/// - ..body (content, str): The content, or equations (math or code lines) to calculate.
/// - inherit (bool): See the definitions made before the worksheet (also after each `reset`).
/// - reset (none, function, selector): Start over at every match, e.g. `heading` (every heading) or `heading.where(level: 1)`: the definitions made since are forgotten, like `#reset`.
/// - hidden (bool): For equations: calculate them without showing them (only problems are shown).
/// - digits, separator, form, … (auto): Every option of `worksheet-config` (except `units`, `integration` and `implicit-parameters`), for this worksheet or these equations only.
/// -> content
#let worksheet(
  ..body,
  inherit: false, reset: none, hidden: false,
  digits: auto, trailing-zeros: auto, separator: auto, grouping: auto, scientific: auto, engineering: auto, multiplication: auto, form: auto, max-denominator: auto, fractions: auto,
  prefixes: auto, show-si: auto, preferred-units: auto, strict: auto, steps: auto, layout: auto, implies: auto, approximate: auto, block: auto, max-solutions: auto,
  defined-style: auto, errors: auto, show-casts: auto, show-types: auto,
) = {
  let pos = body.pos()
  if body.named().len() > 0 {
    let k = body.named().keys().first()
    return _err-now("unknown argument '" + k + "' for worksheet (arguments: inherit, reset, hidden and the options " + _DEF.keys().filter(k => k not in ("units", "integration", "implicit-parameters", "inline-steps")).join(", ") + ")")
  }
  let lc = _check(_given((
    digits: digits, trailing-zeros: trailing-zeros, separator: separator, grouping: grouping, scientific: scientific, engineering: engineering, multiplication: multiplication, form: form, max-denominator: max-denominator, fractions: fractions,
    prefixes: prefixes, show-si: show-si, preferred-units: preferred-units, strict: strict, steps: steps, layout: layout, implies: implies, approximate: approximate, block: block, max-solutions: max-solutions,
    defined-style: defined-style, errors: errors, show-casts: show-casts, show-types: show-types,
  )))
  if _E(lc) { return _err-now(lc.err) }
  if type(hidden) != bool { return _err-now("hidden: true or false, got " + _tn(hidden)) }
  let eqs = pos.len() > 0 and pos.all(x => type(x) == str or (type(x) == content and x.func() == math.equation))
  let out = if eqs {
    if inherit != false or reset != none { return _err-now("inherit and reset are for a worksheet's content, #worksheet[…]") }
    pos.map(x => if type(x) == str { x.split("\n").map(l => _line(l, true, quiet: hidden)).filter(l => l != none).join() } else { _math-def(x, quiet: hidden) }).join()
  } else {
    let b = pos.at(0, default: [])
    if pos.len() != 1 or type(b) != content { return _err-now("worksheet takes content, #worksheet[…], or equations, #worksheet($U := 5 V$)") }
    if hidden { return _err-now("hidden is for equations, #worksheet($U := 5 V$, hidden: true)") + b }
    if type(inherit) != bool { return _err-now("inherit: true or false, got " + _tn(inherit)) + b }
    if type(reset) not in (type(none), function, selector, label) { return _err-now("reset: none or what to start over at, like heading or heading.where(level: 1), got " + _tn(reset)) + b }
    // "^": the state outside, restored at the end; "^0": the state at the start, for reset
    _st.update(d => { let n = if inherit { d } else { (:) }; n.insert("^", d); n.insert("^0", n); n })
    _cf.update(c => c + ("#blocks": _blocks(c) + 1))
    _rules(b, reset)
    _block-note
    _cf.update(c => c + ("#blocks": _blocks(c) - 1))
    _st.update(d => d.at("^"))
  }
  if lc.len() == 0 { out } else {
    _cf.update(c => c + lc + ("^": c))
    out
    _cf.update(c => c.at("^"))
  }
}
