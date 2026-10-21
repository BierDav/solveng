// lambdify: an expression as a plain Typst function.
#import "api.typ": *

/// Turns an expression into a plain Typst function of numbers, for value tables,
/// other plotting packages (lilaq, cetz-plot, simple-plot) or your own calculations.
/// Defined variables and functions are used with their current values. Needs `context`.
///
/// ```typ
/// #worksheet($R := 50 Ω$, hidden: true)
/// #context {
///   let P = lambdify("U^2/R", "U", unit: "W", units: (U: "V"))
///   table(columns: 2, ..range(1, 6).map(u => ([#u V], [#P(u) W])).flatten())
/// }
/// ```
///
/// The function takes one argument per parameter: a number (in the unit given in
/// `units`, else in the SI unit) or a value like `"5 V"`. It returns a float in
/// `unit` (else in the SI unit), or `none` where the expression isn't defined
/// (`sqrt(-1)`, division by zero).
///
/// Unlike the other functions, problems stop the compilation here, since the
/// result is a function and not content.
///
/// - expr (str, content): The expression, using defined variables and the parameters: `"U^2/R"` or `$U^2 / R$`.
/// - ..params (str): The parameters in order, e.g. `"U"` or `"U", "R"`. A defined variable as a parameter is replaced by the argument. Without any: the undefined variables of the expression, in the order they appear.
/// - unit (none, str, content): Unit of the result, e.g. `"W"`. Checked against the expression when all parameters have units.
/// - units (dictionary): Units of numeric arguments, e.g. `(U: "V", R: "kΩ")`.
/// -> function
#let lambdify(expr, ..params, unit: none, units: (:)) = {
  let fail(m) = panic("solveng: lambdify: " + m)
  if params.named().len() > 0 { fail("unknown argument '" + params.named().keys().first() + "' (arguments: the parameters, unit, units)") }
  if type(expr) not in (str, content) { fail("the expression must be a string or math, got " + _tn(expr)) }
  if type(units) != dictionary { fail("units: a dictionary like (U: \"V\"), got " + _tn(units)) }
  let ps = params.pos()
  if ps.any(p => type(p) != str) { fail("the parameters must be names like \"U\", got " + repr(ps.find(p => type(p) != str))) }
  let cf = _cf.get()
  let math = type(expr) == content and expr.func() == std.math.equation
  let src = if math { _src(expr.body) } else { _plain(expr) }
  if _E(src) { fail("can't read the expression") }
  let r = _call(_p.lambdify, _env(_st.get()), (
    cfg: _req-cfg(cf + (separator: ".")), src: src, mode: if math { "math" } else { "code" }, params: ps,
    unit: if unit == none { none } else { _plain(unit) }, units: units.pairs().map(((k, v)) => (k, _plain(v))).to-dict(),
  ))
  if "err" in r { fail(r.err) }
  let lam = r.lam
  (..args) => {
    if args.named().len() > 0 { panic("solveng: lambdify: the function takes no named arguments") }
    let a = args.pos().map(v => if type(v) in (int, float) { float(v) } else if type(v) in (str, content) { _plain(v) } else { panic("solveng: lambdify: a number or a value like \"5 V\", got " + _tn(v)) })
    let y = _call(_p.lam_eval, lam, a)
    if type(y) == dictionary { panic("solveng: lambdify: " + y.err) }
    y
  }
}
