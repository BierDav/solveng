// Keywords for the clause after a trailing = or ->, and the type any: labelled metadata the worksheet reads,
// invisible elsewhere.

#let _key(name, ..args) = [#metadata((key: name, args: args.pos()))<solveng-key>]
/// Solves an equation for these names (and defines them): `$x^2 - 4 = 0 -> solve(x)$`.
///
/// - ..names (content): What to solve for.
/// -> content
#let solve(..names) = _key("solve", ..names.pos())
/// The result with these variables' formulas and constants' expressions put in, every other name kept:
/// `$P := U^2/R -> substitute(U)$`, `$E := m #c^2 -> substitute(#c)$`. A constant alone shows its expression
/// as written: `$#i -> substitute(#i)$` is √−1, where a simplified result would be i again.
///
/// - ..names (content): The variables and constants whose formulas to put in.
/// -> content
#let substitute(..names) = _key("substitute", ..names.pos())
/// The symbolic result, what `->` shows anyway: `$f := (x^2 - 1)/(x - 1) -> simplify$`.
#let simplify = _key("simplify")
/// Decimals with n significant digits: `$t = #h, digits(2)$`.
///
/// - n (int, content): The digits, 1…15.
/// -> content
#let digits(n) = _key("digits", n)
/// Scientific notation: `$C = sci$` → 2.2 · 10⁻⁶ F.
#let sci = _key("sci")
/// Engineering notation, exponents in steps of 3: `$f = eng$`.
#let eng = _key("eng")
/// Exact fractions: `$I / 3 = fraction$` → ⅙ A.
#let fraction = _key("frac")
/// The type of a parameter that takes every unit: `$f(x: any) := 2 x$`.
#let any = _key("any")
