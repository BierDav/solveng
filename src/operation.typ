#import "engine.typ": _p, _call, _src, _plain, _E, _binding-displays, _constant-display
#import "builtins.typ": _op-call

/// An operation under a name of your choice: `#let conv = operation("convolve")`, then `$conv(x, h)$`.
///
/// The name selects one of solveng's operations (see *Built-in operations*), whatever the binding is called;
/// quoted names in math never do. Unlike `math.op("tri")`, the result takes named arguments, as Typst only
/// parses them for functions: `$conv(x, h, step: 2)$`.
///
/// ```typ
/// #let triangle = operation("tri", display: () => math.op("triangle"))
/// #show: worksheet
/// $triangle(1/2) =$
/// ```
///
/// - name (str): The operation, e.g. `"tri"` or `"convolve"`.
/// - display (auto, function): A function returning how the operation is shown; `auto`: its name.
/// -> function
#let operation(name, display: auto) = {
  assert(type(name) == str, message: "operation: expected an operation identity")
  assert(display == auto or type(display) == function, message: "operation: display must be a function")
  (..args) => _op-call(name, args, display: display)
}

#let _references(c) = {
  if type(c) != content { return (:) }
  if c.has("label") and c.label == <solveng-binding> {
    return c.children.find(x => x.func() == metadata).value.displays
  }
  if c.has("children") { return c.children.map(_references).fold((:), (a, b) => a + b) }
  if c.has("body") { return _references(c.body) }
  if c.func() == math.frac { return _references(c.num) + _references(c.denom) }
  if c.func() == math.attach { return _references(c.base) + _references(c.fields().at("t", default: [])) + _references(c.fields().at("b", default: [])) }
  (:)
}

#let _binding(kind, expression, identity, display) = {
  assert(identity == none or (type(identity) == str and identity.trim() != ""), message: kind + ": identity must be a nonempty string")
  assert(display == auto or type(display) == function, message: kind + ": display must be a function")
  let is-base = kind == "base-unit"
  let body = if is-base { [] } else {
    assert(type(expression) == content and expression.func() == math.equation, message: kind + ": expected a math expression")
    expression.body
  }
  let src = if is-base { "" } else { _src(body) }
  assert(not _E(src), message: kind + ": cannot read the expression")
  // a constant is shown as the worksheet shows its identity, a unit as its display says
  let shown = if kind == "constant" { context _constant-display(identity, body) } else if display != auto { display() } else if is-base { math.upright(identity) } else { body }
  let presentation = if kind == "constant" { "" } else { repr(shown) }
  let binding = _call(_p.binding, (kind: kind, src: src, identity: identity, presentation: presentation, description: if kind == "constant" { "" } else { _plain(shown) }))
  let displays = _references(body)
  if kind != "constant" { displays.insert(binding.id, shown) }
  let body = [#_binding-displays.update(d => d + displays)#metadata((token: binding.token, displays: displays))#shown]
  [#body<solveng-binding>]
}

/// A unit of a new, independent dimension: `#let bit = base-unit("information")`.
///
/// The identity names the dimension: base units with the same identity are the same dimension, also when
/// they're declared in other files or worksheets. Results of the dimension are shown in the unit.
///
/// ```typ
/// #let bit = base-unit("information", display: () => math.upright("bit"))
/// #let byte = unit($8 bit$, display: () => math.upright("B"))
/// #show: worksheet
/// $d := 3 byte = bit$
/// ```
///
/// - identity (str): The dimension, the same in every file and worksheet.
/// - display (auto, function): A function returning how the unit is shown; `auto`: the identity, upright.
/// -> content
#let base-unit(identity, display: auto) = {
  assert(type(identity) == str and identity.trim() != "", message: "base-unit: expected a nonempty dimension identity")
  _binding("base-unit", none, identity, display)
}

/// A unit defined by an expression of unit values: `#let minute = unit($60 #s$)`.
///
/// The expression is the unit's value: a positive number times units, from `units`, `base-unit` or other
/// `unit`s (`$100 kPa$`, `$1/min$`, `$#m / #s$`). `display` changes how the unit is shown, not what it means;
/// binding names keep their spelling (a unit named `tau` isn't shown as τ). Like every binding, the unit holds
/// where the binding is visible, in every worksheet.
///
/// ```typ
/// #import units: kPa, min
/// #let bar = unit($100 kPa$, display: () => math.upright("bar"))
/// #let rpm = unit($1/min$, display: () => math.upright("rpm"))
/// #show: worksheet
/// $p := 3 bar$, $n := 3000 rpm$, $p = kPa$
/// ```
///
/// - expression (content): The unit's value as math.
/// - display (auto, function): A function returning how the unit is shown; `auto`: the expression.
/// -> content
#let unit(expression, display: auto) = _binding("unit", expression, none, display)

/// Defines a constant: an identity and the math expression it stands for, which can carry units.
///
/// The identity says what the constant is and how it is shown: worksheets show it as their option `constants`
/// says, `#show: worksheet.with(constants: (imaginary: $j$))`, with built-in displays for `"pi"`, `"euler"`,
/// `"imaginary"` and `"speed-of-light"`; an identity without a display is shown as its expression. The engine
/// calculates exactly with the identities it knows – `"pi"` (π), `"euler"` (e) and `"imaginary"` (i) – once the
/// expression has that value: `$#calc.pi$`, `$sqrt(-1)$`. Any other identity is kept by it in exact results
/// (`$E := m #c^2 ->$` is m c²) and calculated with its expression, which must have a value. `substitute` puts
/// the expression in: `$E := m #c^2 -> substitute(#c)$`, and a constant alone shows it as written,
/// `$#i -> substitute(#i)$` is √−1.
///
/// ```typ
/// #import units: m, s, kg
/// #let c0 = constant("speed-of-light", $299792458 #m / #s$)
/// #let g0 = constant("standard-gravity", $9.80665 #m / #s^2$)
/// #show: worksheet.with(constants: ("standard-gravity": $g_0$))
/// $F := 2 kg #g0 =$
/// ```
///
/// - identity (str): What the constant is, the same in every file and worksheet.
/// - expression (content): Its value as math: numbers, units and other constants.
/// -> content
#let constant(identity, expression) = {
  assert(type(identity) == str and identity.trim() != "", message: "constant: expected an identity, constant(\"imaginary\", $sqrt(-1)$)")
  _binding("constant", expression, identity, auto)
}
