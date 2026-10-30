// What isn't math is refused, and the removed unit options are reported.
#import "/tests/util.typ": *
#import units: s
#math("string_definition := 1")
#reported("math takes content")
#value("1 + 2")
#graph("cos(t)")
#mathscope[#math(units: (:))[]]
#reported("unknown argument 'units'")
#mathscope[#math(builtin-units: false)[]]
#reported("unknown argument 'builtin-units'")
#mathscope[#math(builtin-functions: false)[]]
#reported("unknown argument 'builtin-functions'")
#mathscope[
  $x := 2 "s"$ $x =>$ #fails("'s' in 'x' isn't defined")
  $y := "tri"(0) =>$ #fails("tri isn't defined")
  $t := 2 #s$ #expect("t", 2)
  $z := op("tri")(0)$ #expect("z", 1)
]
