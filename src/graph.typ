// Plotting: lilaq on paper, our own renderer (src/plot.js) in HTML.
#import "defined.typ": *
#import "@preview/lilaq:0.6.0" as _lq

// HTML: the renderer, once per document
#let _plot-js-done = state("solveng-plot-js", false)
#let _plot-js() = context if not _plot-js-done.get() { html.elem("script", read("plot.js")); _plot-js-done.update(true) }

// ---------- plotting ----------
/// Plots an equation: ranges, nice ticks, axis labels and units are picked automatically.
///
/// Math holds the curves, separated by a backslash: `$cos(w) \ w$`, or a function such as
/// `$P(U: #V) = U^2/R$`. A parameter annotation supplies the axis unit; `..` takes the one undefined variable.
/// With `:=`, curves also define their names for later calculations. Curves share the same x axis.
///
/// ```typ
/// #import units: Ω, W, V
/// #worksheet($R := 50 #Ω$, $P_0 := 2 #W$, hidden: true)
/// #graph($P(U: #V) = U^2/R$, ymark: $P_0$)
/// #graph($cos(w) \ w$, mark: auto)
/// #graph($y(x) = x^2 \ y(x) = 2x + 3$, xrange: (-3, 5))
/// ```
///
/// Drawn with `@preview/lilaq:0.6.0` in the schoolbook style: axes through the origin with arrows, a faint grid at the regular ticks. Ticks use the document's decimal separator, axis labels read `U / V`; curves of the same unit share one graph, with a legend in the corner the curves leave emptiest. In HTML export the graph is drawn in the browser instead, in the same style: the curves draw themselves in when the graph comes into view, and the pointer shows the values of every curve with their units. The static picture stays in the page for readers without scripts.
///
/// **Ranges:** values and expressions in `xrange` / `yrange` must have the axis unit (`$f(20) #Ω$` is fine) – `xrange: ($10 #Ω$, $f(20)$)` with a unitless `f(20)` is an error, since it isn't clear what was meant. Without a range: around the marks; else around what's interesting – roots, extrema, poles, edges of the domain (√, ln) and, above all, where curves cross – rounded to nice ticks, starting at 0 when everything interesting is positive. With `ymark`, the y range is around the marked values (steep curves leave the picture). **Units** follow from what's plotted: y's from the curves (shown in the SI unit with a fitting prefix; `preferred-units` picks others), x's from the curve's parameter (`U(I: mA) = …`) or a function x is passed to whose parameter has one (`$q(I: mA) := I R$`, then `#graph($U(I) = q(I)$)` is over I in mA), else from the units in `xrange` and the marks, else from the curve and the units of the y values (`P(..) = U_0^2/R_2` with `ymark: $2 #W$`: R₂ in Ω).
///
/// **Data:** a vector of values is plotted as the points through its elements, joined in index order (with a dot at each when there are at most 30): `#graph($y$)` against the zero-based index k (as in `at(y, k)`), `#graph($y(τ)$)` over the vector `tau` of the same length – or over any expression without free names that is one, `y(2 tau)`. Units come from the elements (each vector needs one unit for all its elements; every element must have a value). Function curves can go alongside: `#graph($y(τ) \ c(τ)$)` with a function `c`, or an expression in `tau` (`$2 tau$`), where `tau` stands for the x axis and the function is sampled over it; the x range is from the data's first to last x value unless `xrange` says otherwise, and the units of x and y must agree. Marks work on data too, with linear interpolation between the points: x values, `"roots"` and `"crossings"` (with data and with functions) where the values change sign, `"extrema"` as the local extreme points, and `ymark`. A vector with a free name is several curves instead, one per element (`$vec(t, t^2)$`). For measurements, `lines: false` draws the data as points alone, without the line: `#graph($y(τ) \ c(τ)$, lines: (false, true))` shows measured points next to the curve `c`.
///
/// **Marks** default to none. Requested marks are always visible – the ranges grow to show them. Combine them in an array, `(auto, $R_1$)`.
///
/// **Size:** `width` / `height` are the whole graph, labels included; `auto` makes the plot area 8 × 5 cm. Any other named argument goes straight to lilaq's `diagram` (e.g. `title: [Load]`, `grid: none`); it shapes the printed graph, not the one drawn in HTML. For full control, plot with lilaq directly.
///
/// - equation (content): Math curves separated by a backslash.
/// - ..args (any): Named options passed on to lilaq's `diagram` (e.g. `title: [Load]`).
/// - xrange (auto, array, content, int, float): x values that must be visible: numbers in the x axis's unit (`(1, 2)`), values (`($10 #Ω$, $1 kΩ$)`) or expressions of defined variables (`(0, $2 R$)`). Two or more give the range from the smallest to the largest; a single one widens the picked range. `auto`: around the marked x values, else picked from the curves.
/// - yrange (auto, array, content, int, float): y values that must be visible, like `xrange` (numbers in the y axis's SI unit). Two or more give the exact y range. `auto`: fitted to the curves.
/// - mark (none, auto, array, content, int, float): What to mark, with both coordinates written on the axes: x values like `xrange` (`$R$`, `2`, `$5 #V$`; a point with guide lines on every curve, the range grows to show it), `"crossings"`, `"roots"`, `"extrema"` (searched within the range) and `auto`: where two curves cross. An array combines them: `(auto, $R_1$)`. `()` or `none` (default): no marks.
/// - xmark (none, array, str, content, int, float): Like `mark`, but only the x value is written on the axis.
/// - ymark (none, array, content, int, float): y values to mark, like `yrange`: every point where a curve reaches one, with guide lines; the value is written on the y axis.
/// - samples (int): Number of sample points.
/// - width (auto, length, ratio, relative, fraction): Width of the whole graph, labels included: `12cm`, `300pt` or a share like `100%`. `1fr` is the container's width – in a grid, the column's. `auto`: a plot area of 8 cm.
/// - height (auto, length, ratio, relative, fraction): Like `width`; `auto`: a plot area of 5 cm. A share is of the container's height, or of the width when the page grows with its content.
/// - colors (array, color): Colors of the curves, e.g. `(gray, blue, rgb("#f98eff"))`. The default is Petroff's ten colors, chosen to stay apart for color-blind readers (lilaq's default too).
/// - dashes (array, str, dictionary, none): Line styles of the curves, so they can be told apart without color (color-blind readers, grayscale prints): Typst dash patterns such as `"solid"`, `"dashed"`, `"dotted"`, `"dash-dotted"`, `"densely-dashed"`, `"loosely-dotted"`, or an array of lengths like `(4pt, 2pt)`. The default is the order of scientific plotting tools (MATLAB, matplotlib, gnuplot): solid, dashed, dotted, dash-dotted. `"solid"` or `none`: every curve solid. Curve n takes color n and dash n, each list repeating on its own.
/// - lines (bool, array): Whether data curves are joined by lines, per curve and repeating like `colors`: `false` draws the data as points alone (a scatter plot), `(false, true)` only the first curve. Points are circles, squares, triangles and diamonds, curve after curve, so they can be told apart without color too. Functions are always lines.
/// - digits, separator, form, … (auto): Every option of `worksheet` works here too, for this call only.
/// -> content
#let graph(
  equation, ..args, xrange: auto, yrange: auto, mark: none, xmark: (), ymark: (), samples: 400,
  width: auto, height: auto, colors: (rgb("#3f90da"), rgb("#ffa90e"), rgb("#bd1f01"), rgb("#94a4a2"), rgb("#832db6"), rgb("#a96b59"), rgb("#e76300"), rgb("#b9ac70"), rgb("#717581"), rgb("#92dadd")),
  dashes: ("solid", "dashed", "dotted", "dash-dotted"), lines: true,
  digits: auto, trailing-zeros: auto, separator: auto, grouping: auto, scientific: auto, engineering: auto, multiplication: auto, form: auto,
  max-denominator: auto, fractions: auto, prefixes: auto, show-si: auto, preferred-units: auto,
  approximate: auto, block: auto, max-solutions: auto, defined-style: auto, errors: auto, show-types: auto,
  words: auto, chips: auto, unit-style: auto, unit-separator: auto,
) = _mine(context {
  let lc = _check(_given((digits: digits, trailing-zeros: trailing-zeros, separator: separator, grouping: grouping, scientific: scientific, engineering: engineering, multiplication: multiplication, form: form, max-denominator: max-denominator, fractions: fractions, prefixes: prefixes, show-si: show-si, preferred-units: preferred-units, approximate: approximate, block: block, max-solutions: max-solutions, defined-style: defined-style, errors: errors, show-types: show-types, words: words, chips: chips, unit-style: unit-style, unit-separator: unit-separator)))
  if _E(lc) { return _errc("graph: " + lc.err, _cf.get(), wide: true) }
  let cf = _cf.get() + lc
  let fail(m) = _errc("graph: " + m, cf, wide: true)
  // types
  if args.pos().len() > 0 { return fail("pass curves in one equation, separated by a backslash") }
  if type(equation) != content or equation.func() != math.equation { return fail("pass curves as math equations") }
  let eqs = (equation,)
  if args.pos().any(a => type(a) == array) { return fail("the x range is named: xrange: " + repr(args.pos().find(a => type(a) == array))) }
  // options graph had: what replaces them
  let gone = (x: "x is the one variable of the curves that isn't defined", xunit: "x's unit comes from a function parameter with a unit (q(I: mA)), xrange or the marks", yunit: "y's unit comes from the curves; preferred-units: (\"mW\",) picks another", builtin-units: "units are Typst bindings: #import units: …", units: "units are Typst bindings: #import units: …, or unit($…$)")
  let old = args.named().keys().find(k => k in gone)
  if old != none { return fail(old + ": is gone – " + gone.at(old)) }
  let XV = (content, int, float)
  let list(v) = if v == auto or v == none { () } else if type(v) == array { v } else { (v,) }
  let (rng, yrng) = (list(xrange), list(yrange))
  for (n, r) in (xrange: rng, yrange: yrng) { if r.any(v => type(v) not in XV) { return fail(n + ": values to show, like (1, 2), (\"10 Ω\", \"1 kΩ\") or (0, \"2 R\")") } }
  let ymk = list(ymark)
  if ymk.any(v => type(v) not in XV) { return fail("ymark: y values like \"P0\" or 2, or an array of these") }
  // marks: x values, keywords, auto; (item, y labelled): mark writes x and y on the axes, xmark only x
  let KW = ("crossings", "roots", "extrema")
  let ms(v) = if v == none or v == false { () } else if v == true { (auto,) } else if type(v) == array { v } else { (v,) }
  let ents = ms(mark).map(m => (m, true)) + ms(xmark).map(m => (m, false))
  if ents.any(((m, _)) => m != auto and type(m) not in XV and m not in ("crossings", "roots", "extrema")) { return fail("mark / xmark: auto, x values like \"R\" or 2, \"crossings\", \"roots\", \"extrema\", or an array of these; () for none") }
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
  // a single pattern can be an array of lengths too: (4pt, 2pt) is one dash, ((4pt, 2pt), "dotted") two
  let DASH = ("solid", "dotted", "densely-dotted", "loosely-dotted", "dashed", "densely-dashed", "loosely-dashed", "dash-dotted", "densely-dash-dotted", "loosely-dash-dotted")
  let lens(a) = type(a) == array and a.len() > 0 and a.all(l => type(l) == length or l == "dot")
  let is-dash(d) = d == none or (type(d) == str and d in DASH) or lens(d) or (type(d) == dictionary and lens(d.at("array", default: none)) and d.keys().all(k => k in ("array", "phase")) and type(d.at("phase", default: 0pt)) == length)
  let dashes = if dashes == none or type(dashes) in (str, dictionary) or lens(dashes) { (dashes,) } else { dashes }
  if type(dashes) == content { return fail("dashes: square brackets make content – write an array with parentheses: (\"solid\", \"dashed\")") }
  if type(dashes) != array or dashes.len() == 0 or not dashes.all(is-dash) { return fail("dashes: a dash pattern or an array of them, like (\"solid\", \"dashed\", \"dotted\", \"dash-dotted\") or (4pt, 2pt); patterns: " + DASH.join(", ")) }
  let lines = if type(lines) == bool { (lines,) } else { lines }
  if type(lines) != array or lines.len() == 0 or lines.any(l => type(l) != bool) { return fail("lines: true, false (data as points alone) or an array of these per curve, like (false, true)") }

  // Serialize math with its explicit unit and operation substitutions.
  let src(q) = if type(q) == content and q.func() == math.equation { let m = _src(q.body); (src: if _E(m) { "?" } else { m }, label: _src(q.body)) } else { panic("solveng: graph: values must be numbers or math equations") }
  let v(a) = if type(a) in (int, float) { float(a) } else { src(a) }
  let mk(m) = if m == auto { "auto" } else if type(m) == str and m in KW { m } else { v(m) }
  let req = (
    cfg: _req-cfg(cf), curves: eqs.map(src).map(q => {
      let labels = if q.label == none or _E(q.label) { () } else { _call(_p.statements, q.label) }
      _call(_p.statements, q.src).enumerate().map(((i, s)) => (src: s, label: labels.at(i, default: none)))
    }).flatten(), xrange: rng.map(v), yrange: yrng.map(v),
    marks: ms(mark).map(m => (mk(m), true)) + ms(xmark).map(m => (mk(m), false)), ymark: ymk.map(v), samples: samples,
  )
  // drawn with the definitions made before; the curves written with := define what they name afterwards, as the
  // same equation in a worksheet does – a pure update of the state, so graphs building on each other settle
  let draw = context {
    let g = _call(_p.graph, _env(_st.get()), req)
    if "err" in g { return fail(g.err) }
    let M(s) = _m(s, cf)
    // a shape per curve as well as a color and a dash: curves stay apart without color
    let shapes = ("o", "s", "^", "d")
    let style(i) = { let at(a) = a.at(calc.rem(i, a.len())); (color: at(colors), dash: at(dashes), shape: at(shapes), line: at(lines)) }
    // the legend goes to the corner the curves leave emptiest
    let (gw, gh) = (g.x1 - g.x0, g.y1 - g.y0)
    let uv = (g.series.map(s => s.points).join() + g.points.map(p => (p.x, p.y))).map(((x, y)) => ((x - g.x0) / gw, (y - g.y0) / gh))
    let corners = ((top + left, (u, v) => u < 0.45 and v > 0.6), (top + right, (u, v) => u > 0.55 and v > 0.6), (bottom + right, (u, v) => u > 0.55 and v < 0.4), (bottom + left, (u, v) => u < 0.45 and v < 0.4))
    let (corner, crowd) = corners.map(((a, inside)) => (a, uv.filter(((u, v)) => inside(u, v)).len())).sorted(key: c => c.at(1)).first()
    // no corner is free: beside the plot, at the top
    let outside = crowd > 0.02 * uv.len()
    let legend = if outside { (position: top + left, dx: 100% + 1.2em) } else { (position: corner) }
    let plots = ()
    let named = ()
    for s in g.series {
      let st = style(s.curve)
      let alone = s.at("data", default: false) and not st.line
      // the legend names each curve once
      let label = if s.label != none and s.curve not in named { named.push(s.curve); $#M(s.label)$ }
      // round caps: the dots of a dotted pattern are zero-length dashes
      plots.push(_lq.plot(s.points.map(p => p.at(0)), s.points.map(p => p.at(1)), color: st.color, label: label,
        stroke: if alone { none } else { (paint: st.color, thickness: 1.2pt, dash: if st.dash == "solid" { none } else { st.dash }, cap: "round", join: "round") },
        // data with few points: a mark at each
        mark: if alone or s.at("dots", default: false) { st.shape } else { none }, mark-size: if alone { 5pt } else { 4pt }))
    }
    let guide = (paint: luma(110), thickness: 0.6pt, dash: "dashed")
    for p in g.points {
      if p.guides != "" { plots.push(_lq.plot((p.x, p.x), (g.ax, p.y), stroke: guide, mark: none)) }
      if p.guides == "xy" { plots.push(_lq.plot((g.ay, p.x), (p.y, p.y), stroke: guide, mark: none)) }
      plots.push(_lq.plot((p.x,), (p.y,), color: black, stroke: none, mark: "o", mark-size: 4pt))
    }
    // the origin's 0, unless an axis writes it already
    let origin = g.x-axis-zero and g.y-axis-zero and 0 not in g.xticks and 0 not in g.yticks
    if origin { plots.push(_lq.place(0, 0, align: top + right, box(inset: (right: 0.35em, top: 0.3em), $0$))) }
    // regular ticks get a grid line, marked values are only written on the axis. The grid stays open at the
    // edges, no line closes the plot area on any side; the arrow's end has its axis label, not a tick.
    let axis(kind, ticks, labels, marked, position, lim) = {
      let at(v, e) = calc.abs(v - e) <= 1e-9 * (lim.at(1) - lim.at(0))
      let all = ticks.zip(labels, marked).filter(t => not at(t.at(0), lim.at(1))).map(((v, l, m)) => (v, l, m or at(v, lim.at(0))))
      let regular = all.filter(t => not t.at(2))
      (position: position, mirror: false, subticks: none,
        ticks: regular.map(t => (t.at(0), $#M(t.at(1))$)),
        extra-ticks: all.filter(t => t.at(2)).map(((v, l, _)) => _lq.tick(v, label: $#M(l)$, kind: kind)))
    }
    let xm = g.at("xmarked", default: g.xticks.map(_ => false))
    let ym = g.at("ymarked", default: g.yticks.map(_ => false))
    let make(w, h) = {
      // schoolbook: axes through the origin with arrows, a faint grid at the regular ticks only
      show: _lq.theme.schoolbook
      show: _lq.set-grid(stroke: luma(225) + 0.5pt, stroke-sub: none)
      _lq.diagram(width: w, height: h, xlim: (g.x0, g.x1), ylim: (g.y0, g.y1), legend: legend,
        xlabel: $#M(g.xlabel)$, ylabel: $#M(g.ylabel)$,
        xaxis: axis("x", g.xticks, g.xlabels, xm, if g.x-axis-zero { 0 } else { bottom }, (g.x0, g.x1)),
        yaxis: axis("y", g.yticks, g.ylabels, ym, if g.y-axis-zero { 0 } else { left }, (g.y0, g.y1)),
        ..args.named(), ..plots)
    }
    // auto: a plot area of 8 × 5 cm; lengths and shares (100%, 50pt) are the whole graph, labels included
    let sized = std.layout(size => {
      let (aw, ah) = (width == auto, height == auto)
      let abs(v, full) = if type(v) == ratio { v * full } else if type(v) == relative { v.ratio * full + v.length.to-absolute() } else if type(v) == fraction { v / 1fr * full } else { v.to-absolute() }
      if aw and ah { return make(8cm, 5cm) }
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
    if _target() != "html" { return align(center, sized) }
    // HTML: drawn in the browser, animated and with the values under the pointer; the static picture stays for
    // readers without scripts. Labels are the document's math, placed over the drawing.
    let hex(c) = (if type(c) == gradient { c.sample(50%) } else { c }).to-hex()
    let svg-dash(d) = if d == none or d == "solid" { "" } else if type(d) == str {
      (dotted: "0 3.5", densely-dotted: "0 2.5", loosely-dotted: "0 6", dashed: "6 4", densely-dashed: "6 2.5", loosely-dashed: "6 7",
        dash-dotted: "6 3 0 3", densely-dash-dotted: "6 2 0 2", loosely-dash-dotted: "6 5 0 5").at(d)
    } else { (if type(d) == dictionary { d.array } else { d }).map(l => if l == "dot" { "0" } else { str(calc.round(l.to-absolute().pt() * 4 / 3, digits: 2)) }).join(" ") }
    let ncurves = calc.max(0, ..g.series.map(s => s.curve + 1))
    let rc = _req-cfg(cf)
    let spec = (
      x0: g.x0, x1: g.x1, y0: g.y0, y1: g.y1, ax: g.ax, ay: g.ay, xz: g.x-axis-zero, yz: g.y-axis-zero, origin: origin,
      xt: g.xticks.zip(xm), yt: g.yticks.zip(ym),
      series: g.series.map(s => (p: s.points, c: s.curve, data: s.at("data", default: false), dots: s.at("dots", default: false))),
      curves: range(ncurves).map(i => { let st = style(i); (color: hex(st.color), dash: svg-dash(st.dash), shape: st.shape, line: st.line, named: i in named) }),
      points: g.points, legend: if outside { "outside" } else { (if corner.y == top { "top-" } else { "bottom-" }) + (if corner.x == left { "left" } else { "right" }) }, sep: rc.separator, digits: rc.digits, size: { let m = measure(sized); (w: m.width.pt(), h: m.height.pt()) },
    )
    let lab(k, body) = html.elem("span", attrs: ("data-k": k), body)
    let labels = (
      lab("x", $#M(g.xlabel)$), lab("y", $#M(g.ylabel)$),
      ..("xu", "yu", "xn", "yn").map(k => { let v = g.at((xu: "xunit", yu: "yunit", xn: "xname", yn: "yname").at(k), default: ""); lab(k, if v != "" { $#M(v)$ }) }),
      ..g.xlabels.enumerate().map(((i, l)) => lab("xt" + str(i), $#M(l)$)),
      ..g.ylabels.enumerate().map(((i, l)) => lab("yt" + str(i), $#M(l)$)),
      ..g.series.filter(s => s.label != none).map(s => lab("c" + str(s.curve), $#M(s.label)$)),
    )
    _css(); _plot-js()
    html.elem("div", attrs: (class: "solveng-plot"), {
      html.elem("div", attrs: (class: "solveng-plot-static"), html.frame(sized))
      html.elem("div", attrs: (class: "solveng-plot-labels", hidden: "hidden"), labels.join())
      html.elem("script", attrs: (type: "application/json", class: "solveng-plot-spec"), json.encode(spec, pretty: false))
    })
  }
  draw + _st.update(s => { let r = _call(_p.graph_define, _env(s), req); s + (env: r.env, last: (errors: r.errors)) })
  context _st.get().last.errors.map(fail).join()
})
