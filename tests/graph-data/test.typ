#import "/lib.typ": *
#import "/src/engine.typ": _p, _call, _env, _req-cfg
#import "/src/api.typ": _st, _cf
#import functions: *
#set page(width: 18cm, height: auto)
#show: worksheet

// sampled data: a signal over a time axis, and a vector against its index
#worksheet(raw("tau := vec(0, 1, 2, 3, 4, 5, 6) * 1 ms"), raw("y := vec(0, 1.5, 2.5, 2, 0.5, -1, -0.5) * 1 V"), raw("c(t: s) := 0.4 V/ms * t"), hidden: true)

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
#graph($y \ 1 unit("V")$, errors: "panic")
#graph($y(τ) \ 2 unit("V") dot sin(τ / (1 unit("ms")))$, xrange: (-1, 8), mark: (3.5, "crossings"), errors: "panic")

// a long vector: a line without dots
#worksheet(raw("n := vec(" + range(200).map(k => str(calc.round(calc.sin(k / 10) * calc.exp(-k / 80), digits: 4))).join(", ") + ")"), hidden: true)
#graph($n$, ymark: 0.5, errors: "panic")
