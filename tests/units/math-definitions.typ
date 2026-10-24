// Imperial units made from imported ones, used in math, value and lambdify.
#import "/tests/util.typ": *
#import units: m, s, cm, kg, g, ml, l
#import operations: tri
#worksheet(errors: "panic")[
  #let inch = unit($2.54 cm$, display: () => math.upright("in"))
  #let foot = unit($12 inch$, display: () => math.upright("ft"))
  #let yard = unit($3 foot$, display: () => math.upright("yd"))
  #let pound = unit($453.592 #g$, display: () => math.upright("lb"))
  #let gallon = unit($4.54609 #l$, display: () => math.upright("gal"))
  #let speed = unit($frac(2 #m, 4 #s)$, display: () => math.upright("speed"))
  #let test = s
  $x := 1 yard = #m$ #expect("x", 0.9144, u: "m")
  $x_2 := 1 inch$ #expect("x_2", 0.0254, u: "in")
  $t := 1 test$ #expect("t", 1, u: "s")
  $y := tri(0)$ #expect("y", 1)
  $v := 2 speed =$ #expect("v", 1)
  $M := 2 pound = kg$ #expect("M", 0.907184)
  $V := 1 gallon = #l$ #expect("V", 0.00454609)
  #value($x_2 = cm$)
  #context assert.eq(lambdify($x_2 + a$, "a", unit: $#m$, units: (a: $#m$))(1), 1.0254)
]
