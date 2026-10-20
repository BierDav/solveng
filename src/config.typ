// Options, their checks, and the two states: definitions and options.

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
