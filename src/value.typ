// value: a result alone, in the text.
#import "api.typ": *

/// The result of an expression alone, where the text needs it: what `$U =$` shows
/// after its `=`. Calculated with the worksheet's definitions at this point, which
/// it leaves as they are.
///
/// ```typ
/// #show: worksheet
/// #import units: kΩ, mA, mV
/// $R := 4.7 kΩ$, $I := 2 mA$, $U := R I$, $f(x: Ω) := 2 x$
/// The voltage is #value("U") (#value($U = mV$)), and #value("f(3 Ω)") for 3 Ω.
/// ```
///
/// A clause picks the unit or form as after `=`: `#value("U = mV, digits(2)")`; a
/// trailing `->` shows the symbolic result. A function's name alone shows its body.
/// It shows one expression and defines nothing: definitions go in the worksheet.
///
/// - expr (str, content): The expression: `"U"`, `"f(2)"`, `"U / 2 = mV"` or `$U$`.
/// -> content
#let value(expr) = context {
  let cf = _cf.get() + (block: false)
  if type(expr) not in (str, content) { return _errc("value: an expression as a string or math, got " + _tn(expr), cf) }
  let math = type(expr) == content and expr.func() == std.math.equation
  let parts = if math and expr.body.has("children") { expr.body.children } else { () }
  if parts.any(c => c.func() == linebreak) { return _errc("value: one expression per value – a #value for each line", cf) }
  let src = if math { _src(expr.body) } else { _plain(expr) }
  if not math and src.contains("\n") { return _errc("value: one expression per value – a #value for each line", cf) }
  if _E(src) { return _errc("value: can't read the expression", cf) }
  let r = _call(_p.value, _env(_st.get()), (src: src, mode: if math { "math" } else { "code" }, cfg: _req-cfg(cf), listed: false))
  if "err" in r { return _errc("value: " + r.err, cf) + _err-notes(none, (r.err,), src) }
  _eqn(_m(r.value, cf), cf)
}
