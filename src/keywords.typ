// Keywords for the clause after a trailing = or ->: labelled metadata the worksheet reads, invisible elsewhere.
// Typst's own frac is a keyword too (exact fractions); math shows a function as its name, which is read.

#let _key(name, ..args) = [#metadata((key: name, args: args.pos()))<solveng-key>]
/// Solves an equation for these names (and defines them): `$x^2 - 4 = 0 -> solve(x)$`.
///
/// - ..names (content): What to solve for.
/// -> content
#let solve(..names) = _key("solve", ..names.pos())
/// The result with these variables' formulas put in, every other name kept: `$P := U^2/R -> substitute(U)$`.
///
/// - ..names (content): The variables whose formulas to put in.
/// -> content
#let substitute(..names) = _key("substitute", ..names.pos())
/// The symbolic result, what `->` shows anyway: `$f := (x^2 - 1)/(x - 1) -> simplify$`.
#let simplify = _key("simplify")
/// Decimals with n significant digits: `$t = 1 unit("h"), digits(2)$`.
///
/// - n (int, content): The digits, 1…15.
/// -> content
#let digits(n) = _key("digits", n)
/// Scientific notation: `$C = sci$` → 2.2 · 10⁻⁶ F.
#let sci = _key("sci")
/// Engineering notation, exponents in steps of 3: `$f = eng$`.
#let eng = _key("eng")
