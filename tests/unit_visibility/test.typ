#import "/tests/util.typ": *
#let unresolved(name) = context {
  let row = _rows(_st.get(), _cf.get()).find(r => r.name == name)
  assert(row != none and row.si == none, message: name + " should depend on an unresolved variable")
}

#worksheet(units: (to: "10km", s: none, mys: "s"))[
  $z := 34"to"$ #expect("z", 340000)
  #let to = unit("m")
  $"km" := 23to$ #expect("km", 23, u: "m")
  $x := 2"km" ->$ #expect("x", 46, u: "m")
  $y := 2s$ #unresolved("y")
  $y := 2"s"$ #unresolved("y")
  #let second = unit("s")
  $y := 2second$ #expect("y", 2, u: "s")
  $w := 2"second"$ #unresolved("w")
  $f("xaef": second) := "xaef"$ #passes()
  $f("xaef": "second") := "xaef"$ #fails("unknown unit 'second'")
  $t := 2ms$ #expect("t", 0.002)
  $u := 2"mys"$ #expect("u", 2)
]
#worksheet(units: (km: "m", tui: "km"))[
  $t := 2"tui"$ #expect("t", 2, u: "tui")
]
#worksheet(units: (s: "2s", mys: "s"))[
  #let second = unit("s")
  $t := 2second$ #expect("t", 4)
  $u := 2"mys"$ #expect("u", 4)
]
#worksheet(builtin-units: false, units: (s: "s"))[
  $t := 2s$ #unresolved("t")
  $t := 2"s"$ #expect("t", 2)
  #let s = unit("s")
  $t := 2s$ #unresolved("t")
  $t := 2#s$ #expect("t", 2)
  $x := 2m$ #unresolved("x")
  $x := 2"m"$ #unresolved("x")
  $x := 2ms$ #unresolved("x")
  #let metre = unit("m")
  $x := 2metre$ #expect("x", 2)
]
#worksheet(units: (second: "2s"))[
  #let second = unit("s")
  $f("xaef": second) := "xaef"$ #passes()
  $f("xaef": "second") := "xaef"$ #passes()
  $t := f(2"second")$ #expect("t", 4)
]
#worksheet(units: (span: "3m"))[
  #worksheet(units: (span: none))[
    #let length = unit("span")
    $x := 2length$ #expect("x", 6)
    $y := 2"span"$ #unresolved("y")
  ]
]

#import "/src/engine.typ": _p, _call, _req-cfg
#import "/src/config.typ": _DEF
#let cycle = _call(_p.check_units, _req-cfg(_DEF + (separator: ".", units: (zu: "uz", uz: "zu"))))
#assert(cycle.contains("circular dependency") and cycle.contains("zu") and cycle.contains("uz"))

#let second = unit("s")
#worksheet(units: (s: "2s"))[
  $x := 2second$ #expect("x", 4)
  #worksheet(units: (s: "3s"))[
    $y := 2second$ #expect("y", 12)
  ]
]
#worksheet(units: (s: "3s"))[
  $x := 2second$ #expect("x", 6)
]
#worksheet(units: (span: "2m"))[
  #worksheet(units: (span: "3span"))[
    $x := 2"span"$ #expect("x", 12)
  ]
]

#let α = unit("alpha")
#let alpha = unit("alpha")
#worksheet(builtin-functions: false, units: (bar: "100kPa", Alpha: 1), preferred-units: "bar")[
  #let cusom = unit("Alpha")
  $t := 5cusom =$ #expect("t", 5, u: "Alpha") #result("\"5\" thin upright(\"Α\")")
]
#import "/src/engine.typ": _restyle, _unit-mk
#let render-cfg = _DEF + (separator: ".", builtin-functions: false, units: (bar: "100kPa", alpha: 1), preferred-units: "bar")
#for name in ("alpha", "α", "m/s^2", "km") {
  assert.eq(_restyle(unit(name), (), render-cfg), _unit-mk(name, render-cfg))
  assert.eq(_restyle($5#unit(name)$.body, (), render-cfg), $5$.body + h(0.1667em) + _unit-mk(name, render-cfg))
}
#worksheet(builtin-units: false, units: (alpha: "m"))[
  $a := 34"alpha"$ #expect("a", 34)
  $b := 34"α"$ #expect("b", 34)
  $c := 34#α$ #expect("c", 34)
  $d := 34alpha$ #expect("d", 34)
  #worksheet(units: (α: "2alpha"))[
    $e := 34#α$ #expect("e", 68)
  ]
]

#worksheet[
  $f(x: A) := x$
  $f(1) =$ #fails("takes x in A")
  $f(1A) =$ #fails("A")
  $f(1"A") =$ #result("\"1\" thin upright(\"A\")")
  $"answer" := f(1"A") =$ #expect("answer", 1, u: "A")
  $A := 2"A"$
  $"answer" := f(1A) =$ #expect("answer", 2, u: "A")
]

#worksheet(units: (iwef: "km"))[
  #let ing = unit("iwef")
  #let i = unit("iwef")
  $a := 1ing$ #expect("a", 1000)
  $b := 1#i$ #expect("b", 1000)
  $c := 1i$ #unresolved("c")
]
