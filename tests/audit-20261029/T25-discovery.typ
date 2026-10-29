#import "/src/config.typ": _merge, _DEF
#let c = _DEF + (graph-colors: (red, green, blue))
#let m = _merge(c, (graph-colors: (auto, black)))
#assert.eq(m.graph-colors.at(calc.rem(3, m.graph-colors.len())), black, message: "second/fourth curves must inherit override cycle")
