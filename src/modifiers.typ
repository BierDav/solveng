/// Modifiers for the clause after a trailing `=` or `->`: `solve`, `substitute`, `simplify`, `expand`, `factor`, `steps`, `digits`, `sci`, `eng`, `fraction`.
///
/// Import the ones you use, where you use them: `#import modifiers: solve, digits`. Typst imports are lexical, so
/// they apply to that file or block and the ones nested in it; nothing comes along with
/// `#import "@preview/solveng:1.0.0": *`. A renamed import works the same: `#import modifiers: solve as löse`,
/// then `$x^2 = 4 -> löse(x)$`.
///
/// ```typ
/// #import modifiers: solve
/// #show: worksheet
/// $x^2 = 4 -> solve(x)$
/// ```

/// ## Modifiers
///
/// The clause after a trailing `=` or `->` takes a unit and modifiers, separated by commas: `$t = #h, digits(2)$`,
/// `$x^2 = 4 -> solve(x)$`. Modifiers are Typst bindings of solveng's `modifiers` module, and importing them is the
/// only way to use them: `#import modifiers: solve, simplify` (or `#import modifiers: *`). Like any import, it holds
/// only where it's written; `#import "@preview/solveng:1.0.0": *` brings none. Renamed imports
/// (`#import modifiers: solve as löse`) work as the original. `solve`, `substitute` and `digits` are functions,
/// called with what they take; `simplify`, `expand`, `factor`, `steps`, `sci`, `eng` and `fraction` are written on their own.
///
/// ```typ
/// #import "@preview/solveng:1.0.0": *
/// #import modifiers: solve, digits, sci
/// #import units: h, mA, mAh
/// #show: worksheet
/// $I := 30 mA$, $Q := 3450 mAh$
/// $Q/2 = I t -> solve(t)$
/// $t = #h, digits(3)$
/// $I = sci$
/// ```

#import "keywords.typ": _key

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
/// The symbolic result, what `->` shows anyway: `$f := (x^2 - 1)/(x - 1) -> simplify$`. Of an equation, where it
/// holds: one side brought to zero and split into factors, each factor zero – `$x^2 = x -> simplify$` is
/// x = 0 ∨ x = 1; true when it always holds, false when never.
#let simplify = _key("simplify")
/// The symbolic result multiplied out: `$(x + 1)^2 -> expand$` is x² + 2x + 1. An equation side by side.
#let expand = _key("expand")
/// The symbolic result in factors: `$x^3 - x -> factor$` is x (x − 1)(x + 1), a fraction's numerator and
/// denominator each. An equation side by side. Splits off common factors, repeated factors and factors of degree one
/// in a name; numbers aren't split into primes, and decimals can't be factored.
#let factor = _key("factor")
/// The way to what `->` shows, a line each: `$F(a p) = a F(p) -> steps$` shows the equation with the definitions
/// put in, one side zero, in factors, and where it holds; an expression goes from the definitions put in to its
/// simplified form. Not with `solve`.
#let steps = _key("steps")
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
