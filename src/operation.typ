// Operations for math with Typst's call syntax: named arguments (step: …) only parse in math for functions.

// math.op(name) followed by its arguments, as Typst writes a call: what the worksheet reads as an explicit
// reference ⦇name⦈(…), resolved through the function overlays where it's used
#let _op-call(name, args) = {
  // the colon of a named argument as a class of its own: no parameter type to show as a chip
  let parts = args.pos() + args.named().pairs().map(((k, v)) => [#k] + math.class("normal", [:]) + h(0.25em) + v)
  let o = math.op(name)
  // a comma and a space between arguments: 1, 10 (without the space, 1,10 reads as a decimal comma)
  let body = parts.join(math.class("punctuation", [,]) + h(0.17em))
  $#o (#body)$.body
}

/// An operation for math, by its name: `#let conv = operation("convolve")`, then `$conv(x, h, step: Delta""t)$`.
///
/// Like `math.op("convolve")`, the result is an explicit reference to an operation, resolved through the
/// function overlays active where it is used (see the option `functions`). Unlike `math.op`, it is a Typst
/// function, so math can pass it named arguments: in Typst math, `name: value` in parentheses parses only for
/// a function. solveng's own operations with named arguments (`convolve`, `range`) are made this way.
///
/// - name (str): The operation's name, or a name a function overlay maps to one: `"convolve"`, `"conv"`.
/// -> function
#let operation(name) = {
  assert(type(name) == str, message: "operation: the operation's name as a string, operation(\"convolve\"), got " + str(type(name)))
  (..args) => _op-call(name, args)
}
