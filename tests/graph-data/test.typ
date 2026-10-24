#import "/lib.typ": *
#import "/src/engine.typ": _p, _call, _env, _req-cfg
#import "/src/api.typ": _st, _cf
#import "/tests/util.typ": reported
#import units: ms, V
#set page(width: 18cm, height: auto)
#show: worksheet

// sampled data: a signal over a time axis, and a vector against its index
#worksheet($tau := vec(0, 1, 2, 3, 4, 5, 6) * 1 ms$, $y := vec(0, 1.5, 2.5, 2, 0.5, -1, -0.5) * 1 #V$, $c(t: #units.s) := 0.4 #V/ms dot t$, hidden: true)

// what the engine gives the graph: the data's points, in its units
#context {
  let g = _call(_p.graph, _env(_st.get()), (
    cfg: _req-cfg(_cf.get()), curves: ((src: "y(tau)", mode: "code", label: none),),
    xrange: (), yrange: (), marks: (("roots", true), ("extrema", true)), ymark: (), samples: 100,
  ))
  assert("err" not in g, message: repr(g))
  assert.eq(g.series.at(0).points.len(), 7)
  assert.eq(g.series.at(0).points.at(2), (2.0, 2.5))
  assert(g.series.at(0).dots)
  assert((g.x0, g.x1) == (0.0, 6.0))
  assert(g.xlabel.contains("ms"), message: g.xlabel)
  // the root between 4 and 5 ms, interpolated; the extrema at 2 and 5 ms
  assert(g.points.any(p => calc.abs(p.x - 4 - 1 / 3) < 1e-9 and p.y == 0))
  assert(g.points.any(p => p.x == 2 and p.y == 2.5) and g.points.any(p => p.x == 5 and p.y == -1))
}

#graph($y(τ)$, mark: ("roots", "extrema"), errors: "panic")
#graph($y(tau) \ c(tau)$, mark: "crossings", errors: "panic")
#graph($y \ 1 #V$, errors: "panic")
#graph($y(τ) \ 2 #V dot sin(τ / (1 ms))$, xrange: (-1, 8), mark: (3.5, "crossings"), errors: "panic")

// a long vector: a line without dots
#worksheet(eval("$n := vec(" + range(200).map(k => str(calc.round(calc.sin(k / 10) * calc.exp(-k / 80), digits: 4))).join(", ") + ")$"), hidden: true)
#graph($n$, ymark: 0.5, errors: "panic")

// data as points alone, next to joined data and a function; custom dashes
#graph($y(τ) \ c(τ)$, lines: (none, auto), mark: "crossings", errors: "panic")
#worksheet($z := vec(1, 2, 1.8, 0.9, 0, -0.4, 0.2) * 1 #V$, hidden: true)
#graph($y(τ) \ z(τ) \ c(τ)$, lines: ("dotted", none, (4pt, 2pt)), errors: "panic")
#graph($y$, lines: none, errors: "panic")

// dot shapes per graph and as worksheet defaults; lines: none on a function leaves its single points
#graph($y(τ) \ z(τ)$, lines: none, dots: ("star", "cross"), colors: (black, red), errors: "panic")
#worksheet(scope: "inherit", graph-dots: "hexagon", graph-colors: blue, graph-lines: "dashed", graph-joined: true)[
  #graph($y(τ) \ z(τ)$, errors: "panic")
]
#worksheet($p(x) := cases(2 & "for" x = 1, x & "for" x < 0 or x > 2)$, hidden: true)
#graph($p(x)$, xrange: (-2, 4), lines: none, errors: "panic")
// joined: a function's pieces joined through its single points, every mode, and joins to points beyond the graph
#for l in (auto, none) { for d in (auto, none) { for j in (false, true) { if l != none or d != none { graph($p(x)$, xrange: (-2, 4), lines: l, dots: d, joined: j, errors: "panic") } } } }
#graph($y(τ) \ p(τ)$, joined: (auto, true), errors: "panic")
#graph($p(x)$, xrange: (0.5, 1.8), yrange: (0, 3), dots: none, joined: true, errors: "panic")
// auto keeps the worksheet's, curve by curve, through nested worksheets
#worksheet(scope: "inherit", graph-dots: (auto, "star"), graph-lines: ("dotted", auto))[
  #context assert.eq(_cf.get().graph-dots, ("circle", "star", "triangle", "star"))
  #context assert.eq(_cf.get().graph-lines, ("dotted", "dashed", "dotted", "dash-dotted"))
  #worksheet(graph-dots: ("cross", auto, auto), graph-joined: (auto, true))[
    #context assert.eq(_cf.get().graph-dots, ("cross", "star", "triangle", "cross"))
    #context assert.eq(_cf.get().graph-joined, (false, true))
  ]
  #graph($y(τ) \ z(τ) \ p(τ)$, dots: (auto, auto, none), joined: (false, auto, true), errors: "panic")
]
// a dot right on a dot of another curve: both drawn, the later smaller
#graph($y(τ) \ y(τ)$, lines: none, errors: "panic")
#worksheet(scope: "inherit", errors: "document")[
  #graph($x^2$, lines: none)
  #reported("draws nothing")
  #graph($x^2$, draw: "dots")
  #reported("draw: is gone")
  #graph($x^2$, dashes: "dotted")
  #reported("dashes: is gone")
  #graph($x^2$, dots: "auto")
  #reported("dots: a dot shape")
  #graph($x^2$, joined: "yes")
  #reported("joined: true")
  #reported("draws nothing")
  #graph($y(τ)$, dots: "blob")
  #reported("dots: a dot shape")
  #worksheet(graph-colors: [red])[]
  #reported("graph-colors: square brackets")
]
