#import "/tests/util.typ": *
#import units: *
#let unresolved(name) = context {
  let row = _rows(_st.get(), _cf.get()).find(r => r.name == name)
  assert(row != none and row.si == none, message: name + " should depend on an unresolved variable")
}

#worksheet(units: (to: $10unit("km")$, s: none, mys: $s$))[
  $z := 34unit("to")$ #expect("z", 340000)
  #let to = unit("m")
  $"km" := 23to$ #expect("km", 23, u: "m")
  $x := 2"km" ->$ #expect("x", 46, u: "m")
  $y := 2s$ #unresolved("y")
  $y := 2"s"$ #unresolved("y")
  #let second = unit("s")
  $y := 2second$ #expect("y", 2, u: "s")
  $w := 2"second"$ #unresolved("w")
  $f("xaef": second) := "xaef"$ #passes()
  $f("xaef": unit("second")) := "xaef"$ #fails("unknown unit 'second'")
  $t := 2ms$ #expect("t", 0.002)
  $u := 2unit("mys")$ #expect("u", 2)
]
#worksheet(units: (km: $m$, tui: $unit("km")$))[
  $t := 2unit("tui")$ #expect("t", 2, u: "tui")
]
#worksheet(units: (s: $2unit("s")$, mys: $s$))[
  #let second = unit("s")
  $t := 2second$ #expect("t", 4)
  $u := 2unit("mys")$ #expect("u", 4)
]
// without built-in units only the units of the option units exist: own base units (auto), plain numbers and
// expressions of these; prefixes still apply
#worksheet(builtin-units: false, units: (s: auto))[
  $t := 2s$ #unresolved("t")
  $t := 2unit("s")$ #expect("t", 2)
  #let s = unit("s")
  $t := 2s$ #unresolved("t")
  $t := 2#s$ #expect("t", 2)
  $x := 2m$ #unresolved("x")
  $x := 2"m"$ #unresolved("x")
  $x := 2ms$ #expect("x", 0.002)
  #let metre = unit("m")
  $x := 2metre$ #fails("unknown unit 'm' (built-in units are off")
  $x := 2km$ #fails("unknown unit 'km'")
  $x := 1unit("km") -> unit("s")$ #fails("unknown unit 'km'")
  $x := 3unit("s") -> min$ #fails("unknown unit 'min' (built-in units are off")
]
#worksheet(builtin-units: false, units: (m: auto, km: $1000 unit("m")$, s: auto, min: $60 unit("s")$, dz: 12, v: $unit("km")/unit("min")$))[
  $d := 3 km$ #expect("d", 3000, u: "km")
  $d_2 := 2500 mm =$ #expect("d_2", 2.5, u: "mm")
  $t := 90 unit("s") -> min$ #expect("t", 90) #result("\"1.5\" thin upright(\"min\")")
  $n := 2unit("dz")$ #expect("n", 24)
  $w := 6 unit("v") =$ #expect("w", 100)
  $p := 3 km -> unit("m")$ #result("\"3000\" thin upright(\"m\")")
]
// definitions see only what exists
#worksheet(builtin-units: false, units: (km: $1000 unit("m")$))[]
#reported("units: km: unknown unit 'm'")
#worksheet(builtin-units: false, units: (s: $2unit("s")$))[]
#reported("units: s: unknown unit 's'")
#worksheet(units: (second: $2unit("s")$))[
  #let second = unit("s")
  $f("xaef": second) := "xaef"$ #passes()
  $f("xaef": unit("second")) := "xaef"$ #passes()
  $t := f(2unit("second"))$ #expect("t", 4)
]
// an isolated worksheet starts a unit table of its own: the outer worksheet's units aren't there
#worksheet(units: (span: $3unit("m")$))[
  #worksheet(units: (span: none))[
    #let length = unit("span")
    $x := 2length$ #fails("unknown unit 'span'")
    $y := 2"span"$ #unresolved("y")
  ]
  #worksheet[$y := 2"span"$ #unresolved("y")]
  #worksheet(scope: "inherit")[$y := 2unit("span")$ #expect("y", 6)]
  #worksheet($z := 2unit("span")$) #expect("z", 6)
]

#import "/src/engine.typ": _p, _call, _req-cfg
#import "/src/config.typ": _DEF
#let cycle = _call(_p.check_units, _req-cfg(_DEF + (separator: ".", units: (zu: $unit("uz")$, uz: $unit("zu")$))))
#assert(cycle.contains("circular dependency") and cycle.contains("zu") and cycle.contains("uz"))

// a self reference is the built-in unit, also in nested worksheets: each table starts anew
#let second = unit("s")
#worksheet(units: (s: $2unit("s")$))[
  $x := 2second$ #expect("x", 4)
  #worksheet(units: (s: $3unit("s")$))[
    $y := 2second$ #expect("y", 6)
  ]
  #worksheet[$y := 2second$ #expect("y", 2)]
]
#worksheet(units: (s: $3unit("s")$))[
  $x := 2second$ #expect("x", 6)
]
#worksheet(units: (span: $2unit("m")$))[
  #worksheet(units: (span: $3unit("span")$))[]
]
#reported("units: span: unknown unit 'span'")
// builtin-units carries over like every option, units don't
#worksheet(builtin-units: false, units: (s: auto))[
  #worksheet[$x := 1"s"$ #unresolved("x") $y := 1"m"$ #unresolved("y")]
  #worksheet(builtin-units: true)[$y := 1unit("m")$ #expect("y", 1)]
]

#let α = unit("alpha")
#let alpha = unit("alpha")
#worksheet(builtin-functions: false, units: (bar: $100unit("kPa")$, Alpha: 1), preferred-units: "bar")[
  #let cusom = unit("Alpha")
  $t := 5cusom =$ #expect("t", 5, u: "Alpha") #result("\"5\" thin upright(\"Α\")")
]
#import "/src/engine.typ": _restyle, _unit-mk
#let render-cfg = _DEF + (separator: ".", builtin-functions: false, units: (bar: $100kPa$, alpha: 1), preferred-units: "bar")
#for name in ("alpha", "α", "m/s^2", "km") {
  assert.eq(_restyle(unit(name), (), render-cfg), _unit-mk(name, render-cfg))
  assert.eq(_restyle($5#unit(name)$.body, (), render-cfg), $5$.body + h(0.1667em) + _unit-mk(name, render-cfg))
}
#worksheet(builtin-units: false, units: (alpha: auto))[
  $a := 34unit("alpha")$ #expect("a", 34)
  $b := 34unit("α")$ #expect("b", 34)
  $c := 34#α$ #expect("c", 34)
  $d := 34alpha$ #expect("d", 34)
  #worksheet(scope: "inherit")[
    $e := 34#α$ #expect("e", 34)
  ]
  #worksheet[
    $e := 34#α$ #fails("unknown unit 'alpha'")
  ]
]

#worksheet[
  $f(x: A) := x$
  $f(1) =$ #fails("takes x in A")
  $f(1A) =$ #fails("A")
  $f(1unit("A")) =$ #result("\"1\" thin upright(\"A\")")
  $"answer" := f(1unit("A")) =$ #expect("answer", 1, u: "A")
  $A := 2unit("A")$
  $"answer" := f(1A) =$ #expect("answer", 2, u: "A")
]

#worksheet(units: (iwef: $unit("km")$))[
  #let ing = unit("iwef")
  #let i = unit("iwef")
  $a := 1ing$ #expect("a", 1000)
  $b := 1#i$ #expect("b", 1000)
  $c := 1i$ #unresolved("c")
]

// the reported documents: own units over the built-in ones, and none of them without built-in units
#worksheet(units: (km: $s$, km_old: $10^3unit("m")$))[
  $a := 1#km -> min$ #expect("a", 1) #result("\"0.01667\" thin upright(\"min\")")
  $b := 1unit("km") -> min$ #expect("b", 1)
  $c := 1unit("km_old") -> unit("m")$ #expect("c", 1000) #result("\"1000\" thin upright(\"m\")")
  $y := 10^3unit("m")$ #expect("y", 1000, u: "m")
  $y_2 := 2 dot 10^6unit("m") =$ #expect("y_2", 2e6, u: "Mm")
  #worksheet(builtin-units: false)[
    $d := 1km -> min$ #fails("unknown unit")
    $e := 1unit("km") -> min$ #fails("unknown unit 'min' (built-in units are off")
    $e := 1unit("km") =$ #fails("unknown unit 'km' (built-in units are off")
    $1km -> min$ #fails("unknown unit")
    $1unit("km") -> min$ #fails("unknown unit")
  ]
]
// numeric powers in definitions
#for (source, factor) in (($10^3unit("m")$, 1000), ($10^3 unit("m")$, 1000), ($10^(3) unit("m")$, 1000), ($10^(-2) unit("m")$, 0.01), ($2^3 unit("m")$, 8), ($1000unit("m")$, 1000)) {
  worksheet(units: (q: source))[$x := 1unit("q")$ #expect("x", factor)]
}

// unit words: imported, also single letters (which need a hash in math)
#worksheet[
  $a := 1#s + 2#s$ #expect("a", 3, u: "s")
  $b := 3#m dot 2#A$ #expect("b", 6)
  $c := 5#V$ #expect("c", 5, u: "V")
  $d := 2#Ω =$ #expect("d", 2, u: "Ω")
  $e := 1#g$ #expect("e", 0.001)
  $f := 4mA$ #expect("f", 0.004, u: "mA")
]
#worksheet(units: (T: none))[$T := 2$ $x := 3#T$ #expect("x", 6)]
#import "/lib.typ": units as unit-module
#let words = dictionary(unit-module).keys()
#for w in ("s", "m", "g", "A", "V", "C", "J", "W", "N", "K", "T", "Ω", "l", "L", "km", "mA", "kΩ", "µA", "uA", "Ah") { assert(w in words, message: w + " isn't a unit word") }
// Typst's own names stay Typst's
// Typst's global names stay Typst's, its math names are imported on purpose
#for w in ("h", "v", "as", "Omega") { assert(w not in words, message: w + " shadows Typst") }
#for w in ("min", "deg", "kmin", "mdeg") { assert(w in words, message: w + " isn't a unit word") }
#assert(words.all(w => w not in dictionary(std)))
#worksheet[
  #import units: min
  $t := 2 min$ #expect("t", 120, u: "min")
  $u := 1 unit("h") -> min$ #result("\"60\" thin upright(\"min\")")
  $w := 90 unit("s") -> min$ #expect("w", 90)
]
#assert(words.all(w => not w.starts-with("_")))

// a single letter is a name, also after ->: quote the unit or import it
#worksheet[
  $x := 5unit("km")$
  $x -> m$ #fails("'m' after -> is a name, not a unit – write unit(\"m\"), or #m with #import units: m")
  $1unit("min") -> s$ #fails("'s' after -> is a name, not a unit – write unit(\"s\"), or #s with #import units: s")
  $x -> unit("m")$ #result("\"5000\" thin upright(\"m\")")
  $x -> #m$ #result("\"5000\" thin upright(\"m\")")
  $v := 72unit("km")/unit("h")$
  $v -> unit("m")/unit("s")$ #result("\"20\" thin upright(\"m\") \\/ upright(\"s\")")
  $v -> #m/#s$ #result("\"20\" thin upright(\"m\") \\/ upright(\"s\")")
  $v = unit("m")/unit("s")$ #result("\"20\" thin upright(\"m\") \\/ upright(\"s\")")
  $v -> km/unit("h"), digits(2)$ #result("\"72\" thin upright(\"km\") \\/ upright(\"h\")")
  $t := x/v -> unit("s")$ #expect("t", 250)
  $f: X -> Y$ #passes()
  $x = m$ #passes()
]
