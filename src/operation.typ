// Explicit references: unit("km") for a unit, operation("convolve") for an operation with Typst's call syntax
// (named arguments, step: …, only parse in math for functions).

#import "builtins.typ": _op-call

/// An operation for math, by its name: `#let conv = operation("convolve")`, then `$conv(x, h, step: Delta""t)$`.
///
/// Like `math.op("convolve")`, the result explicitly selects the underlying operation, independently of
/// `builtin-functions`. Unlike `math.op`, it is a Typst
/// function, so math can pass it named arguments: in Typst math, `name: value` in parentheses parses only for
/// a function. solveng's own operations with named arguments (`convolve`, `range`) are made this way.
///
/// - name (str): The underlying operation's name, such as `"convolve"` or `"tri"`.
/// -> function
#let operation(name) = {
  assert(type(name) == str, message: "operation: the operation's name as a string, operation(\"convolve\"), got " + str(type(name)))
  (..args) => _op-call(name, args)
}

/// An explicit unit reference: `#let ing = unit("iwef")` makes a unit shortcut for `$1ing$`.
///
/// Unlike an ordinary string binding, this explicitly marks its value as a unit and completes unit names
/// inside the string. Only the unit string is highlighted in the declaration; evaluated uses get unit styling.
/// Hover identifies the binding as a unit shortcut and shows the target unit's definition.
/// Inline references also work, with or without a space: `$y := 1unit("km")$`, `$y := 1 unit("km")$`.
/// Completion and unit styling apply inside the string; the constructor collapses to the unit in the editor.
/// The unit resolves through the units of the worksheet at each use, not where the binding is declared.
/// It also reaches a name hidden with `none` (`T: none`); with `builtin-units: false` only your own units exist.
/// Multiletter bindings need no hash; single letters do: `#let i = unit("iwef")`, then `$1#i$`.
/// `$1i$` stays multiplication by a worksheet variable. Complex numbers are unsupported; `i` is no imaginary constant.
///
/// ```typ
/// #show: worksheet.with(units: (iwef: "km"))
/// #let ing = unit("iwef")
/// #let i = unit("iwef")
/// $t := 1ing =$
/// $u := 1#i =$
/// ```
///
/// - name (str): The unit name or expression, such as `"km"`, `"iwef"`, or `"m/s^2"`.
/// -> content
#let unit(name) = {
  assert(type(name) == str, message: "unit: the unit as a string, unit(\"mA\"), got " + str(type(name)))
  [#name<solveng-unit>]
}
