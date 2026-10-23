#import "/tests/util.typ": *
#import units: s as test, cm, l, ml, kg, g
#show: worksheet.with(
  errors: "panic",
  builtin-units: true,
  units: (
    inch: $2.54cm$,
    foot: $30.48cm$,
    yard: $91.44cm$,
    mile: $1609.344m$,
    ounce: $28.3495g$,
    pound: $453.592g$,
    stone: $6.3503kg$,
    fluid_ounce: $28.413ml$,
    pint: $568.261ml$,
    quart: $1.13657l$,
    gallon: $4.54609l$,
    speed: $frac(2 unit("m"), 4 unit("s"))$,
  ),
)
#let inch = unit("inch")
#let tri = math.op("tri")
$x := 1inch$ #expect("x", 0.0254)
$t := 1#test$ #expect("t", 1)
$y := tri(0)$ #expect("y", 1)
$v := 2unit("speed")$ #expect("v", 1)
#value($x = unit("cm")$)
#context assert.eq(lambdify($x + a$, "a", unit: "m", units: (a: "m"))(1), 1.0254)
