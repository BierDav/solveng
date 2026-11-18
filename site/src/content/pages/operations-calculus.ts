import type { Page } from "../types";

export const pages: Page[] = [
  {
    slug: "operations/diff",
    title: "diff",
    group: "operations",
    section: "Calculus",
    summary: "The symbolic derivative of an expression with respect to a name, also written in Leibniz's notation d/dx.",
    signature: "diff(expression, variable)",
    import: "#import operations: diff",
    returns: "Expression.",
    keywords: ["derivative", "differentiate", "d/dx", "Leibniz", "partial", "∂", "slope"],
    blocks: [
      { md: "`diff(f, x)` differentiates f with respect to x, exactly. You rarely need to write the call: Leibniz's notation does the same, and a worksheet shows `diff` that way too.\n\n- `(dif f)/(dif x)` and `dif/(dif x) f` – the first derivative\n- `(dif^2 f)/(dif x^2)` – higher orders\n- `(partial^2 f(x, y))/(partial x partial y)` – partial derivatives\n\nWrite `dif` (d), not `diff` (which Typst shows as ∂). The variable is local: after `x := 3`, `diff(x^2, x)` still differentiates 2x – and then puts in x = 3. Derivatives of pulses and steps that would need impulses stay symbolic and can't be evaluated." },
      {
        example: {
          title: "Three ways to write it",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: diff
#show: math

$diff(x^3, x) ->$
$(dif x^3)/(dif x) ->$
$dif/(dif x) sin(x)^2 ->$`,
        },
      },
      {
        example: {
          title: "Position, velocity, acceleration",
          code: `#import "@preview/solveng:2.0.0": *
#import units: m, s
#show: math

$a_0 := 5 #m / #s^2$, $v_0 := 2 #m / #s$
$s(t) := a_0 t^2 + v_0 t$
$v(t) := (dif s(t))/(dif t) ->$
$a(t) := dif/(dif t) v(t) ->$
$v(3 #s) =>$`,
          caption: "Leibniz's notation needs no import – only `diff` written as a call does. Units come along: m/s, then m/s².",
        },
      },
      {
        example: {
          title: "A function and its slope",
          code: `#import "@preview/solveng:2.0.0": *
#show: math

$f(x) := sin({{k}} x)$
$g(x) := dif/(dif x) f(x) ->$
#graph($f(x) \\ g(x)$, x-range: (-4, 4))`,
          controls: [{ kind: "slider", name: "k", label: "Frequency k", min: 0.5, max: 3, step: 0.5, value: 2, suffix: "" }],
          caption: "The chain rule brings k out front: faster oscillation, steeper slopes.",
        },
      },
      {
        example: {
          title: "Higher and partial derivatives",
          code: `#import "@preview/solveng:2.0.0": *
#show: math

$f(x) := x^2 sin(x)$
$(dif^2 f(x))/(dif x^2) ->$
$g(x, y) := x^2 y^3$
$(partial^2 g(x, y))/(partial x partial y) ->$`,
        },
      },
      {
        quiz: {
          question: "What does `$dif/(dif x) x^2 + x ->$` show?",
          options: ["2x + 1", "3x", "2x"],
          answer: 1,
          explain: "`dif/(dif x)` applies to the term right after it, x², like a factor would. So it's 2x + x = 3x. For the whole sum write `dif/(dif x) (x^2 + x)` or `(dif (x^2 + x))/(dif x)`.",
        },
      },
      {
        params: [
          { name: "expression", type: "expression", md: "Expression to differentiate." },
          { name: "variable", type: "name", md: "Differentiation variable, written as a name rather than a value." },
        ],
      },
    ],
    related: ["operations/integrate", "operations/taylor", "operations/jacobian", "calculus"],
  },
  {
    slug: "operations/integrate",
    title: "integrate",
    group: "operations",
    section: "Calculus",
    summary: "Antiderivatives and definite integrals, exact where possible and numerical otherwise – also written with the integral sign.",
    signature: "integrate(expression, variable, lower, upper)",
    import: "#import operations: integrate",
    returns: "Expression (antiderivative) or value (definite integral).",
    keywords: ["integral", "∫", "antiderivative", "area", "quadrature", "dif"],
    blocks: [
      { md: "`integrate(f, x)` is an antiderivative, without the constant; `integrate(f, x, a, b)` the definite integral. In math you write the integral sign: `integral_a^b f dif x`, and a worksheet shows the call that way.\n\nThe differential `dif x` ends the integrand wherever it stands: `integral_0^1 x^2 + 1 dif x` integrates x² + 1, `integral_0^1 (dif x)/(1 + x^2)` works too.\n\n**Exact** antiderivatives are found for polynomials, exponentials, sin, cos, sinh, cosh, ln and tan of linear arguments, products that multiply out into these, and by parts for a polynomial times one of them. Pulses (`rect`, `tri`, `sgn`, `abs`, `cases`) are split at their corners, so Fourier integrals of pulses come out exact.\n\n**Numerical** integration takes over for `=>` and graphs when there is no closed form: adaptive Gauss–Kronrod to a relative error of 10⁻¹⁰, infinite bounds included. With `->` such integrals stay written as integrals." },
      {
        example: {
          title: "Exact",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: integrate
#import constants: pi, e
#show: math

$integrate(x^2, x, 0, 3) ->$
$integral_0^pi sin(x) dif x ->$
$integral x^2 sin(x) dif x ->$
$integral #e^(a x) dif x ->$`,
          caption: "The call and the integral sign are the same operation. x² sin x is integrated by parts; e^(ax) keeps the case a = 0, where dividing by a would fail, apart.",
        },
      },
      {
        example: {
          title: "Charge from a discharge current",
          code: `#import "@preview/solveng:2.0.0": *
#import units: A, ms, mC, s
#import constants: e
#show: math

$tau := {{tau}} ms$, $I_0 := 2 #A$
$i(t) := I_0 #e^(-t/tau)$
$Q := integral_(0 #s)^infinity i(t) dif t => mC$`,
          controls: [{ kind: "slider", name: "tau", label: "Time constant τ", min: 1, max: 20, step: 1, value: 4, suffix: " ms" }],
          caption: "Amperes times seconds are coulombs: Q = I₀ τ. When both bounds are 0 or ∞, one of them carries the unit of t.",
        },
      },
      {
        example: {
          title: "When there's no closed form",
          code: `#import "@preview/solveng:2.0.0": *
#import constants: e
#show: math

$integral_0^1 sin(x^2) dif x ->$
$integral_0^1 sin(x^2) dif x =>$
$integral_(-infinity)^infinity #e^(-x^2) dif x =>$`,
          caption: "sin(x²) has no elementary antiderivative: `->` keeps the integral, `=>` integrates numerically. The Gaussian over the whole line gives √π ≈ 1.772.",
        },
      },
      {
        quiz: {
          question: "What does `$integral_0^1 (x dif x) + 1 ->$` show?",
          options: ["3/2", "1/2", "2"],
          answer: 0,
          explain: "The parentheses end the integrand at `dif x`: it's ∫₀¹ x dx + 1 = ½ + 1 = 3/2. Without them, `integral_0^1 x + 1 dif x` would integrate x + 1, which is also 3/2 – but `integral_0^1 x dif x + 1` is clearer to read.",
        },
      },
      {
        params: [
          { name: "expression", type: "expression", md: "Integrand." },
          { name: "variable", type: "name", md: "Integration variable, written as a name rather than a value." },
          { name: "lower", type: "expression", md: "Lower bound, `-infinity` allowed; none for an antiderivative." },
          { name: "upper", type: "expression", md: "Upper bound, `infinity` allowed; none for an antiderivative." },
        ],
      },
      { md: "The bounds must have matching units (0 and ±∞ match any); the result has the integrand's unit times the variable's. Undefined samples, exhausted precision or 20000 intervals without convergence are errors – the estimate is numerical, not a proof." },
    ],
    related: ["operations/diff", "operations/convolve", "operations/rect", "calculus", "numerics"],
  },
  {
    slug: "operations/taylor",
    title: "taylor",
    group: "operations",
    section: "Calculus",
    summary: "The Taylor polynomial of an expression about a center, through a given degree.",
    signature: "taylor(expression, variable, center, degree)",
    import: "#import operations: taylor",
    returns: "Expression: a polynomial in the variable.",
    keywords: ["series", "expansion", "approximation", "linearize", "Maclaurin"],
    blocks: [
      { md: "`taylor(f, x, a, n)` is Σₖ f⁽ᵏ⁾(a)/k! · (x − a)ᵏ for k from 0 to n, with exact coefficients. The degree goes from 0 to 32. Every derivative must exist at the center – `ln(x)` about 0 is an error.\n\nIt's an approximation near a: the polynomial makes no claim about convergence further away. `-> expand` writes it term by term." },
      {
        example: {
          title: "Familiar series",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: taylor
#import modifiers: expand
#import constants: e
#show: math

$taylor(#e^x, x, 0, 4) -> expand$
$taylor(cos(x), x, 0, 6) -> expand$
$taylor(1/(1 - x), x, 0, 5) ->$`,
        },
      },
      {
        example: {
          title: "Watch it hug the sine",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: taylor
#show: math

$S(x) := taylor(sin(x), x, 0, {{n}})$
#graph($sin(x) \\ S(x)$, x-range: (-8, 8), y-range: (-2, 2))`,
          controls: [{ kind: "slider", name: "n", label: "Degree n", min: 1, max: 21, step: 2, value: 5, suffix: "" }],
          caption: "Each odd degree adds a term and a bit more of the wave. Degree 1 is the small-angle approximation sin x ≈ x.",
        },
      },
      {
        example: {
          title: "About another center",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: taylor
#import modifiers: expand
#show: math

$taylor(ln(x), x, 1, 3) -> expand$
$taylor(sqrt(x), x, 4, 1) -> expand$`,
          caption: "ln x near 1, and the tangent line of √x at 4: 1 + x/4.",
        },
      },
      {
        example: {
          title: "No derivative at the center",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: taylor
#show: math

$taylor(ln(x), x, 0, 3) ->$`,
          expectError: true,
        },
      },
      {
        params: [
          { name: "expression", type: "expression", md: "Differentiable expression." },
          { name: "variable", type: "name", md: "Local variable name." },
          { name: "center", type: "expression", md: "Expansion center, finite." },
          { name: "degree", type: "integer", md: "Highest retained power, from 0 to 32." },
        ],
      },
    ],
    related: ["operations/diff", "operations/factorial", "modifiers/expand", "calculus"],
  },
  {
    slug: "operations/jacobian",
    title: "jacobian",
    group: "operations",
    section: "Calculus",
    summary: "The Jacobian matrix of a vector of expressions: one row per component, one column per variable.",
    signature: "jacobian(expression, variables)",
    import: "#import operations: jacobian",
    returns: "Matrix.",
    keywords: ["Jacobian", "matrix", "linearize", "partial derivatives", "coordinates"],
    blocks: [
      { md: "`jacobian(vec(f_1, …, f_m), vec(x_1, …, x_n))` is the m × n matrix of partial derivatives ∂fᵢ/∂xⱼ. The variables must be distinct names; their order is the column order.\n\nIt's the derivative of a vector function: it linearizes a nonlinear system at a point, and its determinant scales areas when coordinates change." },
      {
        example: {
          title: "Polar coordinates",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: jacobian
#show: math

$J := jacobian(vec(r cos(phi), r sin(phi)), vec(r, phi)) ->$
$det(J) ->$`,
          caption: "The determinant r is why the area element in polar coordinates is r dr dφ.",
        },
      },
      {
        example: {
          title: "Linearize at a point",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: jacobian
#show: math

$J(x, y) := jacobian(vec(x^2 - y, x y), vec(x, y)) ->$
$J({{x}}, 2) =>$`,
          controls: [{ kind: "slider", name: "x", label: "x", min: -3, max: 3, step: 1, value: 1, suffix: "" }],
          caption: "Define it as a function of the variables, then evaluate it where the system operates.",
        },
      },
      {
        params: [
          { name: "expression", type: "expression", md: "Column vector of differentiable expressions." },
          { name: "variables", type: "expression", md: "Nonempty vector of distinct variable names." },
        ],
      },
    ],
    related: ["operations/hessian", "operations/diff", "matrices", "vectors"],
  },
  {
    slug: "operations/hessian",
    title: "hessian",
    group: "operations",
    section: "Calculus",
    summary: "The Hessian matrix of a scalar expression: all second partial derivatives, in the given variable order.",
    signature: "hessian(expression, variables)",
    import: "#import operations: hessian",
    returns: "Matrix.",
    keywords: ["Hessian", "second derivative", "curvature", "optimization", "saddle"],
    blocks: [
      { md: "`hessian(f, vec(x_1, …, x_n))` is the symmetric n × n matrix of ∂²f/∂xᵢ∂xⱼ. At a critical point it tells what kind it is: all eigenvalues positive – a minimum; all negative – a maximum; mixed signs – a saddle." },
      {
        example: {
          title: "A saddle",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: hessian, eigenvals
#show: math

$H := hessian(x^2 + {{c}} x y + y^2, vec(x, y)) ->$
$eigenvals(H) =>$`,
          controls: [{ kind: "slider", name: "c", label: "Coupling c", min: -4, max: 4, step: 1, value: 3, suffix: "" }],
          caption: "x² + c·xy + y² has a critical point at the origin. For |c| < 2 both eigenvalues are positive – a bowl; beyond, one turns negative – a saddle.",
        },
      },
      {
        example: {
          title: "Symbolic",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: hessian
#show: math

$H := hessian(x^3 + x y^2, vec(x, y)) ->$
$det(H) ->$`,
        },
      },
      {
        params: [
          { name: "expression", type: "expression", md: "Scalar differentiable expression." },
          { name: "variables", type: "expression", md: "Nonempty vector of distinct variable names." },
        ],
      },
    ],
    related: ["operations/jacobian", "operations/diff", "operations/eigenvals", "operations/minimize"],
  },
  {
    slug: "operations/prod",
    title: "prod",
    group: "operations",
    section: "Calculus",
    summary: "The finite product of an expression over an integer index, both bounds included.",
    signature: "prod(expression, index, lower, upper)",
    import: "#import operations: prod",
    returns: "Value or expression, in the factor's unit to the number of factors.",
    keywords: ["product", "∏", "Π", "pi product"],
    blocks: [
      { md: "`prod(f, k, a, b)` multiplies f for k = a, a + 1, …, b. The index is local. The bounds must be known unitless integers, at most 10000 factors; an empty product (b < a) is 1. Factors may carry units: three factors of 2 m give 8 m³.\n\nThe product sign `product_(k=1)^n` isn't read as a calculation; write `prod`." },
      {
        example: {
          title: "Basics",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: prod
#import units: m
#show: math

$prod(k, k, 1, 5) =>$
$prod(k, k, 1, 0) =>$
$prod(2 #m, k, 1, 3) =>$`,
          caption: "5! = 120, the empty product, and a cube.",
        },
      },
      {
        example: {
          title: "The birthday paradox",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: prod
#show: math

$n := {{n}}$
$p := 1 - prod((365 - k)/365, k, 0, n - 1) =>$`,
          controls: [{ kind: "slider", name: "n", label: "People n", min: 2, max: 70, step: 1, value: 23, suffix: "" }],
          caption: "The chance that two of n people share a birthday. With 23 it's already over a half.",
        },
      },
      {
        example: {
          title: "Telescoping, exactly",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: prod
#show: math

$prod(1 - 1/k^2, k, 2, {{n}}) ->$`,
          controls: [{ kind: "slider", name: "n", label: "Upper bound", min: 2, max: 200, step: 1, value: 50, suffix: "" }],
          caption: "The exact fraction is (n + 1)/(2n): it tends to ½.",
        },
      },
      {
        params: [
          { name: "expression", type: "expression", md: "Factor, optionally with units." },
          { name: "index", type: "name", md: "Local product index." },
          { name: "lower", type: "int", md: "Inclusive lower bound." },
          { name: "upper", type: "int", md: "Inclusive upper bound." },
        ],
      },
    ],
    related: ["operations/factorial", "sums", "operations/integrate"],
  },
];
