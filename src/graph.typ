// Plotting with simple-plot.
#import "defined.typ": *
#import "@preview/simple-plot:1.1.0" as _sp

// ---------- plotting (simple-plot) ----------
/// Plots an equation with `simple-plot`: ranges, nice ticks, axis labels and units are picked automatically.
///
/// One string or math holds the curves, separated by a backslash: an expression (`"cos(w)"`), or a function of
/// what it's plotted against, named in the legend and on the axis: `"U(x: A) = x * [2 Ω]"`, with
/// the parameter's unit for the x axis, or `"P(..) = U^2/R"`, where `..` takes the one variable
/// that isn't defined (with its unit from the range or the marks). Like every function, a curve
/// may use only its parameters and defined variables. With `:=` the curve also defines what it names, as the same equation in a worksheet does: `#graph($y := X(x)$)` plots y over x and defines y as X(x), with x left free; `#graph($y := 2 y$)` plots and defines twice that; `#graph($y(t) := X(t)$)` defines the function y. All curves share x. Where two cross is marked:
/// `#graph("cos(w) \\ w", mark: auto)` shows the solution of cos(w) = w. `xrange:` / `yrange:` list values that must be visible;
/// without them the x range is picked around the marks, else around the interesting part
/// (roots, extrema, poles, intersections). Marked values are always visible too.
///
/// ```typ
/// #worksheet("R := 50 Ω", "P0 := 2 W", "U0 := 10 V", hidden: true)
/// #graph("P(U: V) = U^2/R", ymark: "P0")                    // where P reaches P0
/// #graph("P(..) = U0^2/R_L", xrange: ("10 Ω", "1 kΩ"))       // R_L, in Ω from the range
/// #graph("cos(w) \\ w", mark: auto)                          // range picked, crossing marked
/// #graph("y(x) = x^2 \\ y(x) = 2x + 3", xrange: (-3, 5))     // several curves
/// #graph("P(R_L: Ω) = U0^2/R_L", xrange: (0, "2 R"), mark: ("R", "extrema"))
/// #worksheet("tau := vec(0, 1, 2, 3) * 1 s", "y := vec(0, 2, 1, 3) * 1 V", hidden: true)
/// #graph("y(tau) \\ 1 V/s * tau", mark: "crossings")         // data over tau and a line
/// ```
///
/// Uses `@preview/simple-plot:1.1.0`. Ticks use the document's decimal separator, axis labels read `U / V`; curves of the same unit share one graph, with a legend.
///
/// **Ranges:** values and expressions in `xrange` / `yrange` must have the axis unit (`$f(20) Omega$` and `"f(20) Ω"` are fine) – `xrange: ("10 Ω", "f(20)")` with a unitless `f(20)` is an error, since it isn't clear what was meant. Without a range: around the marks; else around what's interesting – roots, extrema, poles, edges of the domain (√, ln) and, above all, where curves cross – rounded to nice ticks, starting at 0 when everything interesting is positive. With `ymark`, the y range is around the marked values (steep curves leave the picture). **Units** follow from what's plotted: y's from the curves (shown in the SI unit with a fitting prefix; `preferred-units` picks others), x's from the curve's parameter (`U(I: mA) = …`) or a function x is passed to whose parameter has one (`$q(I: mA) := I R$`, then `#graph("U(I) = q(I)")` is over I in mA), else from the units in `xrange` and the marks, else from the curve and the units of the y values (`P(..) = U0^2/R2` with `ymark: "2 W"`: R₂ in Ω).
///
/// **Data:** a vector of values is plotted as the points through its elements, joined in index order (with a dot at each when there are at most 30): `#graph("y")` against the zero-based index k (as in `at(y, k)`), `#graph("y(tau)")` over the vector `tau` of the same length – or over any expression without free names that is one, `y(2 tau)`. Units come from the elements (each vector needs one unit for all its elements; every element must have a value). Function curves can go alongside: `#graph("y(tau) \\ c(tau)")` with a function `c`, or an expression in `tau` (`"2 tau"`), where `tau` stands for the x axis and the function is sampled over it; the x range is from the data's first to last x value unless `xrange` says otherwise, and the units of x and y must agree. Marks work on data too, with linear interpolation between the points: x values, `"roots"` and `"crossings"` (with data and with functions) where the values change sign, `"extrema"` as the local extreme points, and `ymark`. A vector with a free name is several curves instead, one per element (`"vec(t, t^2)"`).
///
/// **Marks** default to none. Requested marks are always visible – the ranges grow to show them. Combine them in an array, `(auto, "R1")`.
///
/// **Size:** `width` / `height` are the whole graph, labels included; `auto` makes the plot area 8 × 5 cm. Any other named argument goes straight to simple-plot's `plot` (e.g. `show-grid: false`), other positional ones (series from simple-plot) are added as they are. For full control, plot with simple-plot directly.
///
/// - equation (str, content): Curves separated by a backslash; strings also accept complete equations on separate lines.
/// - ..args (any): Named options passed on to simple-plot's `plot` (e.g. `show-grid: false`).
/// - xrange (auto, array, str, content, int, float): x values that must be visible: numbers in the x axis's unit (`(1, 2)`), values (`("10 Ω", "1 kΩ")`) or expressions of defined variables (`(0, "2 R")`, math works too). Two or more give the range from the smallest to the largest; a single one widens the picked range. `auto`: around the marked x values, else picked from the curves.
/// - yrange (auto, array, str, content, int, float): y values that must be visible, like `xrange` (numbers in the y axis's SI unit). Two or more give the exact y range. `auto`: fitted to the curves.
/// - mark (none, auto, array, str, content, int, float): What to mark, with both coordinates written on the axes: x values like `xrange` (`"R"`, `2`, `"5 V"`; a point with guide lines on every curve, the range grows to show it), `"crossings"`, `"roots"`, `"extrema"` (searched within the range) and `auto`: where two curves cross. An array combines them: `(auto, "R1")`. `()` or `none` (default): no marks.
/// - xmark (none, array, str, content, int, float): Like `mark`, but only the x value is written on the axis.
/// - ymark (none, array, str, content, int, float): y values to mark, like `yrange`: every point where a curve reaches one, with guide lines; the value is written on the y axis.
/// - samples (int): Number of sample points.
/// - width (auto, length, ratio, relative, fraction): Width of the whole graph, labels included: `12cm`, `300pt` or a share like `100%`. `1fr` is the container's width – in a grid, the column's. `auto`: a plot area of 8 cm.
/// - height (auto, length, ratio, relative, fraction): Like `width`; `auto`: a plot area of 5 cm. A share is of the container's height, or of the width when the page grows with its content.
/// - colors (array, color): Colors of the curves, e.g. `(gray, blue, rgb("#f98eff"))`.
/// - digits, separator, form, … (auto): Every option of `worksheet` (except `units`) works here too, for this call only.
/// -> content
#let graph(
  equation, ..args, xrange: auto, yrange: auto, mark: none, xmark: (), ymark: (), samples: 400,
  width: auto, height: auto, colors: (blue, red.darken(10%), green.darken(25%), orange.darken(10%)),
  digits: auto, trailing-zeros: auto, separator: auto, grouping: auto, scientific: auto, engineering: auto, multiplication: auto, form: auto,
  max-denominator: auto, fractions: auto, prefixes: auto, show-si: auto, builtin-units: auto, builtin-functions: auto, preferred-units: auto, strict: auto, steps: auto, inline-steps: auto,
  layout: auto, implies: auto, approximate: auto, block: auto, max-solutions: auto, defined-style: auto, errors: auto, show-casts: auto, show-types: auto,
  words: auto, chips: auto, unit-style: auto, unit-separator: auto,
) = _mine(context {
  let lc = _check(_given((digits: digits, trailing-zeros: trailing-zeros, separator: separator, grouping: grouping, scientific: scientific, engineering: engineering, multiplication: multiplication, form: form, max-denominator: max-denominator, fractions: fractions, prefixes: prefixes, show-si: show-si, builtin-units: builtin-units, builtin-functions: builtin-functions, preferred-units: preferred-units, strict: strict, steps: steps, inline-steps: inline-steps, layout: layout, implies: implies, approximate: approximate, block: block, max-solutions: max-solutions, defined-style: defined-style, errors: errors, show-casts: show-casts, show-types: show-types, words: words, chips: chips, unit-style: unit-style, unit-separator: unit-separator)))
  if _E(lc) { return _errc("graph: " + lc.err, _cf.get(), wide: true) }
  let cf = _cf.get() + lc
  let fail(m) = _errc("graph: " + m, cf, wide: true)
  // types
  if args.pos().len() > 0 { return fail("pass curves in one equation, separated by a backslash") }
  let eqs = (equation,)
  if args.pos().any(a => type(a) == array) { return fail("the x range is named: xrange: " + repr(args.pos().find(a => type(a) == array))) }
  // options graph had: what replaces them
  let gone = (x: "x is the one variable of the curves that isn't defined", xunit: "x's unit comes from a function parameter with a unit (q(I: mA)), xrange or the marks", yunit: "y's unit comes from the curves; preferred-units: (\"mW\",) picks another")
  let old = args.named().keys().find(k => k in gone)
  if old != none { return fail(old + ": is gone – " + gone.at(old)) }
  let args = arguments(..args.pos().filter(a => type(a) not in (str, content)), ..args.named())
  if eqs.len() == 0 { return fail("nothing to plot – pass an equation like \"P = U^2/R\"") }
  let XV = (str, content, int, float)
  let list(v) = if v == auto or v == none { () } else if type(v) == array { v } else { (v,) }
  let (rng, yrng) = (list(xrange), list(yrange))
  for (n, r) in (xrange: rng, yrange: yrng) { if r.any(v => type(v) not in XV) { return fail(n + ": values to show, like (1, 2), (\"10 Ω\", \"1 kΩ\") or (0, \"2 R\")") } }
  let ymk = list(ymark)
  if ymk.any(v => type(v) not in XV) { return fail("ymark: y values like \"P0\" or 2, or an array of these") }
  // marks: x values, keywords, auto; (item, y labelled): mark writes x and y on the axes, xmark only x
  let KW = ("crossings", "roots", "extrema")
  let ms(v) = if v == none or v == false { () } else if v == true { (auto,) } else if type(v) == array { v } else { (v,) }
  let ents = ms(mark).map(m => (m, true)) + ms(xmark).map(m => (m, false))
  if ents.any(((m, _)) => m != auto and type(m) not in XV) { return fail("mark / xmark: auto, x values like \"R\" or 2, \"crossings\", \"roots\", \"extrema\", or an array of these; () for none") }
  let iskw(m) = type(m) == str and m in KW
  let kws = ents.filter(((m, _)) => iskw(m))
  let autos = ents.filter(((m, _)) => m == auto).map(e => e.at(1))
  let mxs = ents.filter(((m, _)) => m != auto and not iskw(m))
  if type(samples) != int or samples < 2 { return fail("samples: an integer ≥ 2") }
  for (n, v) in (width: width, height: height) {
    let ok = if v == auto { true } else if type(v) == length { v.to-absolute() > 0pt } else if type(v) == ratio { v > 0% } else if type(v) == relative { v.ratio >= 0% and v.length.to-absolute() >= -1e9pt } else if type(v) == fraction { v > 0fr } else { false }
    if not ok { return fail(n + ": auto, a length like 8cm or 300pt, or a share like 100% or 1fr, got " + repr(v) + if type(v) in (int, float) { " – write " + repr(v) + "cm" } else { "" }) }
  }
  let colors = if type(colors) == color { (colors,) } else { colors }
  if type(colors) == content { return fail("colors: square brackets make content – write an array with parentheses: (gray, blue, rgb(\"#f98eff\"))") }
  if type(colors) != array or colors.len() == 0 or colors.any(c => type(c) not in (color, gradient)) { return fail("colors: a color or an array of colors like (gray, blue, rgb(\"#f98eff\"))") }

  // what the engine reads: math as written for it, strings as code
  let src(q) = if type(q) == content and q.func() == math.equation { let m = _src(q.body); (src: if _E(m) { "?" } else { m }, mode: "math", label: _src(q.body)) } else { (src: _plain(q), mode: "code", label: none) }
  let v(a) = if type(a) in (int, float) { float(a) } else { src(a) }
  let mk(m) = if m == auto { "auto" } else if type(m) == str and m in KW { m } else { v(m) }
  let req = (
    cfg: _req-cfg(cf), curves: eqs.map(src).map(q => {
      let labels = if q.label == none or _E(q.label) { () } else { _call(_p.statements, q.label, q.mode) }
      _call(_p.statements, q.src, q.mode).enumerate().map(((i, s)) => (src: s, mode: q.mode, label: labels.at(i, default: none)))
    }).flatten(), xrange: rng.map(v), yrange: yrng.map(v),
    marks: ms(mark).map(m => (mk(m), true)) + ms(xmark).map(m => (mk(m), false)), ymark: ymk.map(v), samples: samples,
  )
  // drawn with the definitions made before; the curves written with := define what they name afterwards, as the
  // same equation in a worksheet does – a pure update of the state, so graphs building on each other settle
  let draw = context {
    let g = _call(_p.graph, _env(_st.get()), req)
    if "err" in g { return fail(g.err) }
    let M(s) = _m(s, cf)
    let dash = (paint: luma(110), thickness: 0.6pt, dash: "dashed")
    let ser = g.series.map(s => { let c = colors.at(calc.rem(s.curve, colors.len())); _sp.line-plot(s.points, stroke: c + 1.2pt,
      // data with few points: a dot at each
      mark: if s.at("dots", default: false) { "*" } else { "none" }, mark-size: 0.07, mark-fill: c, mark-stroke: c + 0.6pt,
      label: if s.label != none { $#M(s.label)$ }, label-pos: s.label-pos) })
    for p in g.points {
      if p.guides != "" { ser.push(_sp.vline(p.x, ymin: calc.min(g.ax, p.y), ymax: calc.max(g.ax, p.y), stroke: dash)) }
      if p.guides == "xy" { ser.push(_sp.hline(p.y, xmin: calc.min(g.ay, p.x), xmax: calc.max(g.ay, p.x), stroke: dash)) }
      ser.push(_sp.scatter(((p.x, p.y),), mark-fill: black, mark-stroke: black, mark-size: 0.08))
    }
    let o = (
      xmin: g.x0, xmax: g.x1, ymin: g.y0, ymax: g.y1, width: width, height: height,
      xtick: g.xticks, xtick-labels: g.xlabels.map(l => $#M(l)$), ytick: g.yticks, ytick-labels: g.ylabels.map(l => $#M(l)$), show-grid: "major",
      axis-x-pos: if g.x-axis-zero { 0 } else { "bottom" }, axis-y-pos: if g.y-axis-zero { 0 } else { "left" }, show-origin: g.x-axis-zero and g.y-axis-zero,
      xlabel: $#M(g.xlabel)$, ylabel: $#M(g.ylabel)$,
    ) + args.named()
    let make(w, h) = _sp.plot(..o + (width: w, height: h), ..ser, ..args.pos())
    // auto: a plot area of 8 × 5 cm; lengths and shares (100%, 50pt) are the whole graph, labels included
    let sized = std.layout(size => {
      let (aw, ah) = (width == auto, height == auto)
      let abs(v, full) = if type(v) == ratio { v * full } else if type(v) == relative { v.ratio * full + v.length.to-absolute() } else if type(v) == fraction { v / 1fr * full } else { v.to-absolute() }
      if aw and ah { return make(8, 5) }
      // HTML has no width to take a share of: a nominal 16cm, the SVG scales with the page anyway
      let full-w = if calc.abs(size.width.pt()) < 1e6 { size.width } else { 16cm }
      let W = if aw { 8cm } else { abs(width, full-w) }
      // a share of the height: of the container, or of the width when the page grows with its content
      let full-h = if calc.abs(size.height.pt()) < 1e6 { size.height } else { W }
      let H = if ah { 5cm } else { abs(height, full-h) }
      let m = measure(make(W, H))
      let w1 = if aw { W } else { calc.max(W - (m.width - W), 1cm) }
      let h1 = if ah { H } else { calc.max(H - (m.height - H), 1cm) }
      make(w1, h1)
    })
    // HTML has no layout of its own for this: the graph becomes an inline SVG
    if _target() == "html" { _css(); html.elem("div", attrs: (class: "solveng-plot"), html.frame(sized)) } else { align(center, sized) }
  }
  draw + _st.update(s => { let r = _call(_p.graph_define, _env(s), req); s + (env: r.env, last: (errors: r.errors)) })
  context _st.get().last.errors.map(fail).join()
})
