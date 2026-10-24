// Options, their checks, and the two states: definitions and options.

#let _st = state("solveng", (:))
// errors are values (typst can't catch panics): (err: "message"), passed up and shown in the document
#let _err(m) = (err: m)
#let _E(x) = type(x) == dictionary and "err" in x
// the export target; Typst 0.14 only knows target() with --features html, else it is paged
#let _target() = if "target" in dictionary(std) { std.target() } else { "paged" }
// ---------- configuration: #worksheet(..) / #show: worksheet.with(..), or per call ----------
#let _DEF = (
  // numbers
  digits: 4, trailing-zeros: false, separator: auto, grouping: none, scientific: (-3, 6), engineering: false, multiplication: "dot",
  form: "auto", max-denominator: 1000, fractions: "stacked",
  // units
  prefixes: "all", show-si: false, preferred-units: (),
  // output
  approximate: true, block: true, max-solutions: 6,
  // output of defined
  defined-style: "grid",
  // problems: shown in the document, or stop the compilation
  errors: "document",
  // types (f(x: #A)) in what solveng shows, the equations as written included: left out, as plain math or as chips
  show-types: "chip",
  // how they look: the word for any-unit types (a German text might want beliebig), the chips' colours or a
  // function drawing them, and units: a function given each shown unit, the separator between units
  words: (any: "any"), chips: auto, unit-style: auto, unit-separator: auto,
  // how constants are shown, by identity, over the built-in displays: (imaginary: $j$)
  constants: (:),
)
#let _CH = (
  form: ("auto", "decimal", "fraction", "input"), multiplication: ("dot", "times"), fractions: ("stacked", "slash"),
  errors: ("document", "panic"),
  show-types: ("no", "plain", "chip"), defined-style: ("grid", "column", "inline", "list"),
  // worksheet's scope, not an option
  scope: ("isolated", "inherit", "shared"),
)
// allowed types per option
#let _TY = (
  digits: (int,), trailing-zeros: (bool,), separator: (type(auto), str), grouping: (type(none), str, symbol, content), scientific: (type(none), array), engineering: (bool,),
  multiplication: (str,), form: (str,), max-denominator: (int,), fractions: (str,), prefixes: (str, type(none), array), show-si: (bool,),
  preferred-units: (array, content),
  approximate: (bool,), block: (bool,),
  max-solutions: (int,), defined-style: (str,), errors: (str,), show-types: (str,),
  words: (dictionary,), chips: (type(auto), dictionary, function), unit-style: (type(auto), function), unit-separator: (type(auto), content, str, symbol),
  constants: (dictionary,),
)
#let _tyname(ts) = ts.map(t => if t == type(none) { "none" } else if t == type(auto) { "auto" } else { str(t) }).join(" or ")
// units are checked by the engine once a worksheet sets them (see #worksheet)
#let _check(o) = {
  if type(o) != dictionary { return (err: "config: expected a dictionary of options like (digits: 3), got " + str(type(o))) }
  let o = o
  for (k, v) in o {
    if k not in _DEF { return (err: "unknown option '" + k + "' (options: " + _DEF.keys().join(", ") + ")") }
    if type(v) not in _TY.at(k) { return (err: k + ": expected " + _tyname(_TY.at(k)) + ", got " + repr(v)) }
    if k in _CH and v not in _CH.at(k) { return (err: k + ": " + repr(v) + " – use one of " + _CH.at(k).map(repr).join(", ")) }
  }
  if "digits" in o and (o.digits < 1 or o.digits > 15) { return (err: "digits must be 1…15") }
  if "max-denominator" in o and o.max-denominator < 1 { return (err: "max-denominator must be ≥ 1") }
  if "max-solutions" in o and o.max-solutions < 1 { return (err: "max-solutions must be ≥ 1") }
  if "scientific" in o and o.scientific != none and (o.scientific.len() != 2 or o.scientific.any(x => type(x) != int)) { return (err: "scientific must be (low, high) with integers, or none") }
  if "prefixes" in o and type(o.prefixes) == str and o.prefixes != "all" { return (err: "prefixes: \"all\", none or an array like (\"m\", \"k\")") }
  if "prefixes" in o and type(o.prefixes) == array and o.prefixes.any(p => type(p) != str) { return (err: "prefixes: the array must hold strings like \"m\"") }
  if "words" in o and o.words.keys().any(k => k != "any") { return (err: "words: the word shown for any-unit types, (any: \"beliebig\")") }
  if "chips" in o and type(o.chips) == dictionary and o.chips.pairs().any(((k, v)) => k != "type" or type(v) != dictionary or v.keys().any(c => c not in ("fill", "mark", "ink"))) { return (err: "chips: the colours of type chips, (type: (fill: …, mark: …, ink: …)), or a function (kind, mark, unit) => content") }
  if "constants" in o and o.constants.values().any(v => type(v) not in (content, symbol)) { return (err: "constants: how constants are shown, by identity: (imaginary: $j$)") }
  if "preferred-units" in o {
    if type(o.preferred-units) == content { o.preferred-units = (o.preferred-units,) }
    if o.preferred-units.any(u => type(u) != content or u.func() != math.equation) { return (err: "preferred-units: math expressions containing unit values") }
  }
  o
}
// named option arguments -> the ones actually given (auto = keep the current setting)
#let _given(o) = o.pairs().filter(((k, v)) => v != auto).to-dict()
#let _cf = state("solveng-cf", _DEF)
