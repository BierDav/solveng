// What isn't math is refused, and the removed unit options are reported.
#import "/tests/util.typ": *
#import units: s
#worksheet("string_definition := 1")
#reported("worksheet takes content")
#value("1 + 2")
#graph("cos(t)")
#worksheet(units: (:))[]
#reported("unknown argument 'units'")
#worksheet(builtin-units: false)[]
#reported("unknown argument 'builtin-units'")
#worksheet(builtin-functions: false)[]
#reported("unknown argument 'builtin-functions'")
#worksheet[
  $x := 2 "s"$ $x =>$ #fails("'s' in 'x' isn't defined")
  $y := "tri"(0) =>$ #fails("tri isn't defined")
  $t := 2 #s$ #expect("t", 2)
  $z := op("tri")(0)$ #expect("z", 1)
]
