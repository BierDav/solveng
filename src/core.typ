// worksheet-config and clear.
#import "engine.typ": *

// ---------- public API ----------

/// Sets options for number formatting, units and output.
///
/// Without a body the options apply from here on; with a body only inside it. Every option defaults to `auto`, which keeps the current setting, so only what you pass changes. Most options can also be passed directly to `worksheet`, `defined` and `graph`, for that worksheet or call only. Unknown options and values of the wrong type are shown as an error in the document.
///
/// ```typ
/// #show: worksheet-config.with(digits: 3, form: "fraction")   // your defaults: at the top of the document
/// #worksheet-config(layout: "column")                          // from here on, definitions are kept
/// #worksheet-config(steps: "result")[ … ]                      // only inside the brackets
/// #show: worksheet
/// $U := 5 V$, $I := 30 mA$
/// #worksheet($R := U/I =$, digits: 2)                     // only these equations
/// #let opts = (digits: 2, separator: ",")
/// #worksheet($P := U I =$, ..opts)                          // options kept in a dictionary
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
/// #let setup(body) = worksheet-config(digits: 3, multiplication: "times", layout: "column", worksheet(body))
///
/// // in a document
/// #import "my-defaults.typ": *
/// #show: setup
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
/// - integration (auto, str): Whether a `worksheet` calculates the math in it: `"math"` (default) or `"none"`, then only ```` ```solveng ```` code blocks and `#worksheet(…)` calls are calculated. Only here, not per call.
/// -> content
#let worksheet-config(
  ..body,
  digits: auto, trailing-zeros: auto, separator: auto, grouping: auto, scientific: auto, engineering: auto, multiplication: auto, form: auto, max-denominator: auto, fractions: auto,
  prefixes: auto, show-si: auto, units: auto, preferred-units: auto, strict: auto,
  steps: auto, inline-steps: auto, layout: auto, implies: auto, approximate: auto, block: auto, max-solutions: auto,
  defined-style: auto, errors: auto, integration: auto, show-casts: auto, show-types: auto,
  words: auto, chips: auto, unit-style: auto, unit-separator: auto,
) = {
  let b = body.pos().at(0, default: none)
  if body.named().len() > 0 { return _err-now("unknown option '" + body.named().keys().first() + "' (options: " + _DEF.keys().join(", ") + ")") + b }
  if body.pos().len() > 1 { return _err-now("worksheet-config takes at most one body") + b }
  if b != none and type(b) != content { return _err-now("worksheet-config: the body must be content, got " + _tn(b)) }
  let n = _check(_given((
    digits: digits, trailing-zeros: trailing-zeros, separator: separator, grouping: grouping, scientific: scientific, engineering: engineering, multiplication: multiplication, form: form, max-denominator: max-denominator, fractions: fractions,
    prefixes: prefixes, show-si: show-si, units: units, preferred-units: preferred-units, strict: strict,
    steps: steps, inline-steps: inline-steps, layout: layout, implies: implies, approximate: approximate, block: block,
    max-solutions: max-solutions, defined-style: defined-style, errors: errors, integration: integration, show-casts: show-casts, show-types: show-types,
    words: words, chips: chips, unit-style: unit-style, unit-separator: unit-separator,
  )), global: true)
  if _E(n) { return _err-now(n.err) + b }
  // units add to the units set before (none removes one); ones the engine can't read are left out, with a note
  let bad(c) = _call(_p.check_units, _req-cfg(c + (separator: ".", units: c.units + n.units)))
  let up(c) = {
    let c2 = c + n
    if "units" in n { c2.units = if bad(c) == none { c.units + n.units } else { c.units } }
    c2
  }
  if "units" in n { context { let m = bad(_cf.get()); if m != none { _errc(m, _cf.get(), wide: true) } } }
  if b == none { _cf.update(up) } else {
    _cf.update(c => up(c) + ("^": c))
    b
    _cf.update(c => c.at("^"))
  }
}
#let _restart(d) = if "^0" in d { let z = d.at("^0"); z.insert("^0", z); z } else { (:) }
/// Forgets definitions: the names given, or with `all: true` every one made so far.
///
/// A value and a function share their name – defining one replaces the other – so `clear("f")` forgets `f` either way (`"f()"` and `"f(x)"` are read the same). Names are written as in code (`"Q_0"`, `"tau"`) or in math (`$Q_0$`). A name that isn't defined is shown as a problem. With `all: true` inside a `worksheet` it's back to how the worksheet started (with `inherit: true` the outer definitions stay).
///
/// ```typ
/// #show: worksheet
/// $I := 30 mA$, $f(x) := 2 x$, $U := 5 V$
/// #clear("I", $f$)
/// $I := 1 A$
/// #clear(all: true)
/// $U := 12 V$
/// ```
///
/// - ..names (str, content): The names to forget, as strings or math.
/// - all (bool): Forget every definition instead.
/// -> content
#let clear(..names, all: false) = {
  if names.named().len() > 0 { return _err-now("unknown argument '" + names.named().keys().first() + "' for clear (arguments: the names, all)") }
  if type(all) != bool { return _err-now("clear: all: true or false, got " + _tn(all)) }
  let ns = names.pos()
  if all {
    if ns.len() > 0 { return _err-now("clear: either names or all: true, not both") }
    return _st.update(_restart)
  }
  if ns.len() == 0 { return _err-now("clear: which names – clear(\"x\", \"f\") – or every one, clear(all: true)") }
  let bad = ns.find(n => type(n) != str and not (type(n) == content and n.func() == math.equation))
  if bad != none { return _err-now("clear: names as strings or math, clear(\"x\", $f$), got " + _tn(bad)) }
  let req = ns.map(n => if type(n) == str { (n, "code") } else { let s = _src(n.body); (if _E(s) { repr(n.body) } else { s }, "math") })
  _st.update(s => {
    let r = _call(_p.clear, _env(s), (names: req))
    s + (env: r.env, last: (errors: r.errors, result: none))
  })
  context { let cf = _cf.get(); _st.get().last.errors.map(m => _errc(m, cf, wide: true)).join() }
}
