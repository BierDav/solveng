// worksheet: math and solveng code calculated where they stand.
#import "core.typ": *

// what the engine defined so far, for defined(), editors and the tests
#let _rows(s, cf) = if _env(s).len() == 0 { () } else { _call(_p.defined, _env(s), (cfg: _req-cfg(cf))) }

// what went wrong, for editors to show at the source: the element it's in (none for equations and code passed to
// worksheet, which have no place of their own) and the statement solveng read there, for code its line
#let _err-notes(loc, errors, source) = errors.map(m => [#metadata((kind: "error", element: loc, message: m, source: source))<solveng>]).join()

// a math equation in a worksheet: := defines (≔ in math), a trailing = shows the value (in the unit and form its
// clause gives, $t = 1 h, digits(3)$), -> the symbolic result or, with solve(x), solves for x; any other = states
// something. Equations that aren't formulas are left alone. quiet: only problems are shown (prose stays). loc: the
// equation in the document, for the errors' notes; auto is its own.
#let _math-def(it, quiet: false, loc: auto) = context {
  if _busy.get() > 0 or _reshown(it) or _own(it) { return it }
  let loc = if loc == auto { it.location() } else { loc }
  let children = if it.body.has("children") { it.body.children } else { (it.body,) }
  if children.any(c => c.func() == linebreak) {
    let groups = ((),)
    for c in children {
      if c.func() == linebreak { groups.push(()) } else { groups.last().push(c) }
    }
    return groups.map(g => _math-def(math.equation(g.join(), block: it.block), quiet: quiet, loc: loc)).join()
  }
  let cf = _cf.get() + (block: it.block)
  let src = _src(it.body)
  if _E(src) or src.trim() == "" { return it }
  _calc(src, "math", cf, out => {
    let errs = out.errors.map(e => _errc(e, cf, wide: it.block)).join() + _err-notes(loc, out.errors, "")
    if out.prose { return _with-body(it, _restyle(it.body, (), cf)) + errs }
    if quiet { return errs }
    let shown = if out.mark == none { _restyle(it.body, out.letters, cf) } else {
      let (head, m) = _split-eq(it.body, out.mark)
      let res = if out.result == none { [] } else { _m(out.result, cf) }
      _restyle(head.join(), out.letters, cf) + (if m == "=" { _BEQ } else { $->$.body }) + res
    }
    _with-body(it, shown) + errs
  })
}

// one line of a code block, read like math: x := … defines, a trailing = shows a value, -> the formula or solves
// (-> solve(x)); units in brackets or after numbers, ';' separates a system, // starts a comment
// (loc: the raw element in the document, for the errors' notes)
#let _line(s, block, quiet: false, loc: none) = {
  let s = s.split("//").first().trim()
  if s == "" { return none }
  context {
    let cf = _cf.get() + (block: block)
    _calc(s, "code", cf, out => {
      let errs = out.errors.map(e => _errc(e + " – in '" + s + "'", cf, wide: block)).join() + _err-notes(loc, out.errors, s)
      if quiet or out.show == none { return errs }
      _eqn(_m(out.show, cf), cf) + errs
    })
  }
}

// ---------- what editors read ----------
// Typewright and other editors query <solveng> metadata after compiling: per element worksheet saw (by its
// location, which maps back to the source) whether it was calculated, per block what it defined, with values, and
// per problem (kind: "error") its message, element and statement, see _err-notes.
#let _block-note = context {
  let cf = _cf.get()
  let units = cf.units.pairs().filter(((k, v)) => v != none).map(((k, v)) => (name: k, value: if v == auto { "auto" } else if type(v) in (str, int, float) { str(v) } else { "" }))
  let defined = _rows(_st.get(), cf).map(r => (name: r.name, kind: r.kind, value: r.plain))
  [#metadata((kind: "block", defined: defined, units: units))<solveng>]
}
// whether the innermost worksheet around calculates its math: its evaluate, one per worksheet, innermost first
#let _evaluates(cf) = cf.at("#evaluate", default: ()).at(0, default: true)
// the element `it` shown as `body`, followed by its record; `cf` is the configuration around it
#let _noted(it, kind, active, cf, body) = {
  let loc = it.location()
  if loc == none { return body }
  body + [#metadata((kind: kind, element: loc, active: active, evaluate: _evaluates(cf)))<solveng>]
}

// how many worksheets are around, kept with the configuration that every element reads anyway
#let _blocks(cf) = cf.at("#blocks", default: 0)
// `b` with math (unless the innermost worksheet around an element says evaluate: false) and solveng code blocks calculated;
// hidden: what is calculated isn't shown (only its problems), the rest of `b` is
#let _rules(b, clear-at, hidden) = context {
  // raw text is set smaller, monospaced and in English; definitions use the surrounding text and language
  let (size, font, lang, region) = (text.size, text.font, text.lang, text.region)
  show raw.where(lang: "solveng", block: true): it => context {
    _noted(it, "code", true, _cf.get(), text(size: size, font: font, lang: lang, region: region, _call(_p.statements, it.text, "code").map(l => _line(l, true, quiet: hidden, loc: it.location())).filter(x => x != none).join()))
  }
  show std.math.equation: it => context {
    if _busy.get() > 0 or _reshown(it) or _own(it) { return it }
    let cf = _cf.get()
    let on = _evaluates(cf)
    _noted(it, "math", on, cf, if on { _math-def(it, quiet: hidden) } else { it })
  }
  // start over at each heading (or whatever clear-at selects): back to the state at the start of the worksheet
  show selector(if clear-at == none { <solveng-never> } else { clear-at }): it => {
    if clear-at != none { _block-note + _st.update(_restart) }
    it
  }
  b
}

/// Calculates the math you write, like a Mathcad worksheet.
///
/// `#show: worksheet` calculates the rest of the document, `#worksheet[…]` its content (whose definitions stay local unless `export: true`), and `#worksheet($…$)` single equations, also outside a worksheet.
///
/// ```typ
/// #show: worksheet
/// The battery holds $Q_0 := 3450 mAh$ and the load draws $I := 30 mA$,
/// so it lasts $t := Q_0 / I = 1 "h"$.
/// ```
///
/// **Math.** As in Mathcad:
///
/// - `:=` defines (shown as ≔), symbolically: `$I := 30 mA$`, `$P := U I$`, `$f(x) := x^2 + 1$`. Names not defined yet stay names until they are, see *Calculating like Mathcad*.
/// - `=` inside an expression compares, 1 or 0: `$M := 1 = 2 =$` is 0.
/// - `=` at the end shows the value, with a bold =: `$t =$` → t **=** 115 h, `$t := Q_0/I =$` defines and shows it. What follows it is the unit and keywords: `$t = 1 "h"$`, `$t = min, digits(2)$`.
/// - `->` at the end shows the symbolic result, the known values put in, exactly: `$f := (x^2 - 1)/(x - 1) -> simplify$`. With `solve(x)` it solves an equation for x (and defines it when the solution is a value): `$Q_0/2 = I t -> solve(t)$`, `$x^2 - 4 = 0 -> solve(x)$`, `$x + y = 3, x - y = 1 -> solve(x, y)$`, `$R <= U/I -> solve(R)$`.
///
/// Keywords: `solve(x, …)`, `substitute(x, …)` (these variables' formulas put in, other names kept), `laplace`, `invlaplace` (optional input and output names), `simplify`, `frac` (exact fractions), `digits(n)` (decimals with n significant digits), `sci` (scientific notation), `eng` (engineering notation). Any other equation with `=` states something: it's checked when everything in it is known (`$U = R I$` after defining them) and left alone as prose when not (`$E = m c^2$`).
///
/// **Units** are names: `$30 mA$`, `$4.7 kΩ$`, `$9.81 "m"/"s"^2$`, and `$3 "bar"$` for your own units (with `worksheet(units: …)`). Single-letter units require quotes: `$5 "V"$`, `$3 "h"$`, and `$t = 1 "h"$` shows t in hours. Anywhere else it's a variable, also alone after `=`: `$t = h$` compares t with a variable h, and `$3 dot h$` is 3 times h.
///
/// **Code.** Each line of a `solveng` code block is calculated the same way and shown with all its steps: `t := Q0/I = [h]`, `Q0/I =`, `x^2 = 4 -> solve(x)`; units in brackets, `;` separates the equations of a system, `//` starts a comment. Inline code and code with another language are left alone.
///
/// **Calls.** `#worksheet($U := 5 "V"$, $I := U/R =$)` calculates its equations (and strings, read like a code line) where it stands, with the definitions made before. A code block passed to it is calculated line by line, like a `solveng` code block but without the tag. Its options are for these only; with content, for the content.
///
/// **Evaluate.** `#worksheet(evaluate: false)[…]` leaves its math alone: only its `solveng` code blocks and `#worksheet(…)` calls are calculated. Nested worksheets don't inherit it, so a `#worksheet[…]` or `#worksheet($t =$)` inside calculates its math again.
///
/// ````typ
/// #worksheet(```
///   t := 2 s
///   v := 3 m / t =
/// ```)
/// ````
///
/// ```typ
/// #worksheet[
///   $t := 5 "s"$
///   #worksheet(evaluate: false, inherit: true)[
///     $E = m c^2$ stays as written, and so does $t = 3 "s"$;
///     #worksheet($t =$) is still calculated.
///   ]
/// ]
/// ```
///
/// **Hidden.** `hidden: true` calculates without showing what is calculated: the equations of a call, or in `#worksheet(hidden: true)[…]` the math and code blocks it calculates – its text, headings and prose equations (`$E = m c^2$`) stay. Problems are still shown. Nested worksheets don't inherit it, so a `#worksheet($t =$)` inside shows its equation again.
///
/// ````typ
/// #worksheet(hidden: true)[
///   The values for the example:
///   $U := 5 "V"$, $R := 100 "Ω"$
///   ```solveng
///   I := U/R
///   ```
///   The current is #worksheet($I = mA$).
/// ]
/// ````
///
/// **Scopes.** `inherit` brings the parent's definitions into the worksheet; `export` passes
/// the worksheet's final definitions back when it ends. Both default to `auto`: true for
/// expressions (including strings and code-block arguments), false for content blocks.
/// A block containing only one equation or raw code block is the same content value as a direct
/// expression argument in Typst; set `export: false` explicitly to keep that case local.
/// Neither option is inherited by nested worksheets. Exported variables, functions and solved
/// values replace existing names in the parent. Only definitions still present at the end
/// are exported: `clear`, `clear-at` and `clear(all: true)` do not delete parent definitions.
/// Configuration and units set inside the block stay scoped as before.
///
/// ```typ
/// #show: worksheet
/// $U := 12 "V"$
/// #worksheet(inherit: true, export: true)[
///   $R := 100 "Ω"$
///   $I := U/R = mA$
/// ]
/// $I = mA$ // 120 mA, exported from the block
/// #worksheet($U := 5 "V"$, export: false) // local to this call
/// $U =$ // still 12 V
/// ```
///
/// **Editors.** Each equation and code block the worksheet sees leaves a `metadata` labelled `<solveng>` with its `location()`, whether it was calculated and whether math is evaluated there; each problem one with `kind: "error"`, its `message`, the `element` it's in (`none` for equations and code passed to `worksheet`) and the `source` statement (a code line; empty for math); at the end (and before each match of `clear-at`) one lists everything defined, with values, and the units. Editors like Typewright read them after compiling: `query(<solveng>)`.
///
/// **Starting over.** `clear-at` clears every definition at each match; `#clear(all: true)` inside a worksheet does the same once: back to how it started (with `inherit: true` the outer definitions stay), and `#clear("x", "f")` forgets only those names. After a `#worksheet[…]` the outer definitions come back; with `export: true`, its final definitions are merged over them.
///
/// ```typ
/// #show: worksheet.with(clear-at: heading)                    // every heading
/// #show: worksheet.with(clear-at: heading.where(level: 1))    // only top-level ones
/// #show: worksheet.with(clear-at: heading, inherit: true)     // keep the definitions made before
/// ```
///
/// **Options.** Every option defaults to `auto`, which keeps the current setting, so only what you pass changes. For the whole document: `#show: worksheet.with(digits: 3)`; for a part: `#worksheet(evaluate: false, steps: "result")[…]`; for equations or content it calculates: `#worksheet($R := U/I =$, digits: 2)`; or keep them in a dictionary: `#let opts = (digits: 2, separator: ",")`, then `#worksheet($P := U I =$, ..opts)`. `defined` and `graph` take the same options except `units`. Unknown options and values of the wrong type are shown as an error in the document.
///
/// ```typ
/// #show: worksheet.with(digits: 3, form: "fraction")   // your defaults, the whole document
/// $U := 5 "V"$, $I := 30 mA$
/// #worksheet($I/3 =$, digits: 2)                        // only this equation
/// #worksheet(evaluate: false, multiplication: "times")[Prose and $E = m c^2$ stay as written.]
/// ```
///
/// `form`:
///
/// - `"auto"`: plain numbers as typed (`7.20 kΩ` keeps its zero), arithmetic evaluated (`3450/2 mAh` → 1725 mAh), results rounded to `digits`.
/// - `"decimal"`: everything rounded to `digits`, including what you typed.
/// - `"fraction"`: exact fractions, simplified (`$frac(2, 4) A$` → ½ A, `I/3` → ⅙ A); values that are no fraction with denominator ≤ `max-denominator` fall back to decimals.
/// - `"input"`: given values exactly as typed (`3450/2 mAh` stays a fraction).
///
/// An explicit unit (`$I = mA$`) always wins, then `preferred-units`, then the non-SI unit of the inputs (h, mAh, bar), then the SI unit with a prefix.
///
/// To reuse your defaults in many documents, keep a small file of your own:
///
/// ```typ
/// // my-defaults.typ
/// #import "@preview/solveng:1.0.0": *
/// #let setup = worksheet.with(digits: 3, multiplication: "times", layout: "column")
///
/// // in a document
/// #import "my-defaults.typ": *
/// #show: setup
/// ```
///
/// - ..body (content, str): The content, or equations to calculate: math, strings (code lines) and code blocks.
/// - inherit (auto, bool): See the definitions made before the worksheet (also after each match of `clear-at`). `auto` (default): yes for equations, no for content. `inherit: false` with equations calculates them on their own; what they define still counts afterwards unless `export: false`, as what a `graph` defines does.
/// - export (auto, bool): Pass the final definitions and solved values to the enclosing scope. `auto` (default): yes for expressions, strings and code-block arguments; no for content blocks. Independent of `inherit`. Exported names replace existing names; clearing a name inside does not remove it from the parent.
/// - clear-at (none, function, selector): Clear every definition at each match, e.g. `heading` (every heading) or `heading.where(level: 1)`: the definitions made since are forgotten, like `#clear(all: true)`.
/// - hidden (bool): Calculate without showing what is calculated (only problems are shown): the equations, or the math and code blocks of the content. Not inherited by nested worksheets.
/// - evaluate (bool): Calculate the math in the content; `false`: only its code blocks and `#worksheet(…)` calls are. Not inherited by nested worksheets, and not for equations, which are always calculated.
/// - digits (auto, int): Significant digits of computed values, 1…15. Default `4`: `57.5 h`, `123500`.
/// - trailing-zeros (auto, bool): Keep trailing zeros up to `digits` (`2.500 V`). Default `false`.
/// - separator (auto, str): Decimal separator, any string like `","`. `auto` (default): from the document language, `,` for de, fr, it, … and `.` otherwise.
/// - grouping (auto, none, str, symbol, content): Digit grouping for integer parts with 5+ digits, e.g. `sym.space.thin` → `12 345`. Default `none`.
/// - scientific (auto, none, array): `(low, high)`: plain notation while the exponent `e` is in `low ≤ e < high`, else `2.5 · 10⁻⁷`. Default `(-3, 6)`; `none` never switches. Only how results are written: `1.12e4` is 1.12 times the variable `e4`, not a number.
/// - engineering (auto, bool): Engineering notation, exponents in steps of 3. Default `false`.
/// - multiplication (auto, str): `"dot"` (default, `2 · 10⁶`) or `"times"` (`2 × 10⁶`).
/// - form (auto, str): How values are written. `"auto"` (default): as typed, results rounded · `"decimal"`: everything rounded to `digits` · `"fraction"`: exact fractions (`⅙ A`) · `"input"`: given values exactly as typed.
/// - max-denominator (auto, int): Largest denominator for `form: "fraction"`. Default `1000`.
/// - fractions (auto, str): How fractions are written: `"stacked"` (default) or `"slash"` (`1/2`).
/// - prefixes (auto, str, none, array): Prefixes picked for results: `"all"` (default: a f p n µ m k M G T; times from a minute on in min or h, not ks), `none` (base unit only) or an array like `("m", "k")` (`"u"` = µ).
/// - show-si (auto, bool): `defined` also shows the SI value (`Q₀ = 3450 mAh = 12420 C`). Default `false`.
/// - builtin-units (auto, bool): Enable automatic unit words and quoted built-in names, true by default. False disables their exposure; user overlays still work: `units: (s: "s")` restores `"s"`. Unit definitions can always refer to underlying built-ins.
/// - units (auto, dictionary): Add a user overlay of unit definitions: `(bar: "100 kPa", rpm: "1/min", dozen: 12)`. Each overlay falls back to lower overlays and underlying built-ins. Entries may refer to each other regardless of order; self references use lower overlays, so `(s: "2s")` doubles the previous second. Circular dependencies are rejected. `auto` creates a base unit; `none` hides a name. Use quoted names (`$1"ing"$`) or declare `#let ing = unit("ing")` for `$1ing$`; single-letter bindings need a hash (`#let i = unit("i")`, `$1#i$`). Explicit `unit` shortcuts complete inside their strings, show unit-shortcut hover documentation, and resolve through the overlays at each use. The editor action *Declare unit overlay bindings* generates these declarations; worksheet options cannot inject Typst lexical bindings. With content, overlays are scoped to the worksheet; a show rule applies them to remaining content.
/// - builtin-functions (auto, bool): Enable automatic built-in operation names, true by default. False disables automatic lookup in code and solveng's exported math names. Explicit `math.op("tri")` and `operation("convolve")` references always select the underlying operations. Quoted math names never select built-in operations.
/// - preferred-units (auto, str, array): Units for results and graph axes of their dimension, `("bar", "kWh")`, before input units and automatic SI prefixes.
/// - strict (auto, bool): Unit names can't be variable names, and units in math must be quoted (`$5 "V"$`), see *Units and variables*.
/// - steps (auto, str): Default for `steps` of `solve`: `"full"` (default), `"formula"`, `"result"` or `"value"`.
/// - layout (auto, str): Default for `layout` of `solve`: `"row"` (default) or `"column"`.
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
/// -> content
#let worksheet(
  ..body,
  inherit: auto, export: auto, clear-at: none, hidden: false, evaluate: true,
  digits: auto, trailing-zeros: auto, separator: auto, grouping: auto, scientific: auto, engineering: auto, multiplication: auto, form: auto, max-denominator: auto, fractions: auto,
  prefixes: auto, show-si: auto, builtin-units: auto, units: auto, preferred-units: auto, strict: auto,
  builtin-functions: auto,
  steps: auto, layout: auto, implies: auto, approximate: auto, block: auto, max-solutions: auto,
  defined-style: auto, errors: auto, show-casts: auto, show-types: auto,
  words: auto, chips: auto, unit-style: auto, unit-separator: auto,
) = {
  let pos = body.pos()
  if body.named().len() > 0 {
    let k = body.named().keys().first()
    return _err-now("unknown argument '" + k + "' for worksheet (arguments: inherit, export, clear-at, hidden, evaluate and the options " + _DEF.keys().join(", ") + ")")
  }
  let lc = _check(_given((
    digits: digits, trailing-zeros: trailing-zeros, separator: separator, grouping: grouping, scientific: scientific, engineering: engineering, multiplication: multiplication, form: form, max-denominator: max-denominator, fractions: fractions,
    prefixes: prefixes, show-si: show-si, builtin-units: builtin-units, units: units, preferred-units: preferred-units, strict: strict,
    builtin-functions: builtin-functions, steps: steps, layout: layout, implies: implies, approximate: approximate, block: block, max-solutions: max-solutions,
    defined-style: defined-style, errors: errors, show-casts: show-casts, show-types: show-types,
    words: words, chips: chips, unit-style: unit-style, unit-separator: unit-separator,
  )))
  if _E(lc) { return _err-now(lc.err) }
  if type(hidden) != bool { return _err-now("hidden: true or false, got " + _tn(hidden)) }
  if type(evaluate) != bool { return _err-now("evaluate: true or false, got " + _tn(evaluate)) }
  if inherit != auto and type(inherit) != bool { return _err-now("inherit: auto, true or false, got " + _tn(inherit)) }
  if export != auto and type(export) != bool { return _err-now("export: auto, true or false, got " + _tn(export)) }
  let eqs = pos.len() > 0 and pos.all(x => type(x) == str or (type(x) == content and x.func() in (math.equation, raw)))
  let exports = if export == auto { eqs } else { export }
  let finish(d) = {
    let outer = d.at("^")
    if exports { outer + (env: _p.merge(_env(outer), _env(d))) + if "last" in d { (last: d.last) } else { (:) } } else { outer }
  }
  let out = if eqs {
    if clear-at != none { return _err-now("clear-at is for a worksheet's content, #worksheet[…]") }
    if not evaluate { return _err-now("evaluate: false is for a worksheet's content, #worksheet(evaluate: false)[…]; equations passed to worksheet are always calculated") }
    let code(s, block) = _call(_p.statements, s, "code").map(l => _line(l, block, quiet: hidden)).filter(l => l != none).join()
    let shown = pos.map(x => if type(x) == str { code(x, true) } else if x.func() == raw { code(x.text, x.block) } else { _math-def(x, quiet: hidden) }).join()
    if inherit != false and exports { shown } else {
      _st.update(d => { let inner = if inherit != false { d } else { (:) }; inner.insert("^", d); inner })
      shown
      _st.update(finish)
    }
  } else {
    let b = pos.at(0, default: [])
    if pos.len() != 1 or type(b) != content { return _err-now("worksheet takes content, #worksheet[…], or equations, #worksheet($U := 5 V$)") }
    if type(clear-at) not in (type(none), function, selector, label) { return _err-now("clear-at: none or where to clear the definitions, like heading or heading.where(level: 1), got " + _tn(clear-at)) + b }
    // "^": the state outside, restored at the end; "^0": the state at the start, for clear-at and clear(all: true)
    _st.update(d => { let n = if inherit == true { d } else { (:) }; n.insert("^", d); n.insert("^0", n); n })
    _cf.update(c => c + ("#blocks": _blocks(c) + 1, "#evaluate": (evaluate,) + c.at("#evaluate", default: ())))
    _rules(b, clear-at, hidden)
    _block-note
    _cf.update(c => c + ("#blocks": _blocks(c) - 1, "#evaluate": c.at("#evaluate").slice(1)))
    _st.update(finish)
  }
  if lc.len() == 0 { out } else {
    // units add to the ones set before (none removes one); ones the engine can't read are left out, with a note
    let unit-config(c) = {
      let layers = c.at("#unit-layers", default: if c.units.len() == 0 { () } else { (c.units,) })
      c + (units: c.units + lc.units, "#unit-layers": layers + (lc.units,))
    }
    let up(c) = {
      let c2 = c + lc + ("^": c)
      if "units" in lc {
        let m = _call(_p.check_units, _req-cfg(unit-config(c) + (separator: ".",)))
        if m == none {
          c2.units = c.units + lc.units
          c2.insert("#unit-layers", unit-config(c).at("#unit-layers"))
        } else { c2.units = c.units }
      }
      c2
    }
    if "units" in lc {
      context {
        let c = _cf.get()
        let m = _call(_p.check_units, _req-cfg(unit-config(c) + (separator: ".",)))
        if m != none { _errc(m, c, wide: true) }
      }
    }
    _cf.update(up)
    out
    _cf.update(c => c.at("^"))
  }
}
