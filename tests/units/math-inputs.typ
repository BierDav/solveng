#import "/tests/util.typ": *
#worksheet("string_definition := 1")
#reported("worksheet takes content")
#value("1 + 2")
#graph("cos(t)")
#worksheet(units: (inch: "2.54cm"))[]
#reported("units: a dictionary of definitions")
#worksheet[
  $x := 2 "s"$ $x =$ #fails("'s' in 'x' isn't defined")
  $y := "tri"(0) =$ #fails("tri isn't defined")
  $t := 2unit("s")$ #expect("t", 2)
  $z := op("tri")(0)$ #expect("z", 1)
]
