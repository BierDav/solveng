// Units are Typst bindings: imported from `units`, or made with base-unit and unit.
#import "/tests/util.typ": *
#import units: m, s, km, h, A, V, W, Ω, kΩ, mΩ, mm, mAh, mmol, cd, kPa, cm, N, Hz, min
#set page(width: 18cm, height: auto)
#include "math-inputs.typ"
#include "math-definitions.typ"
#show: math.with(errors: "panic")
#let second = base-unit("time", display: () => std.math.upright("s"))
#let minute = unit($60 second$, display: () => std.math.upright("min"))
#let hour = unit($60 minute$, display: () => std.math.upright("h"))
$T := 2 minute =>$ #expect("T", 120, u: "min")
$t := T = second$ #expect("t", 120, u: "s")
$d := 1 km =>$ #expect("d", 1000, u: "km")
$v := d / T => #m / #s$ #expect("v", 1000 / 120)
#let travel = constant("speed-of-light", $299792458 #m / #s$)
$u := travel =>$ #expect("u", 299792458)
#let length = base-unit("length", display: () => std.math.upright("metre"))
$x := 2 #m + 3 length =>$ #expect("x", 5)
#let inch = unit($0.0254 #m$, display: () => std.math.upright("in"))
$L := 10 inch => #m$ #expect("L", 0.254)
#mathscope(scope: "isolated")[$"time" := 2 minute =>$ #expect("time", 120)]
#let tau = base-unit("oscillation-count", display: () => std.math.upright("count"))
$N_c := 4 tau =>$ #expect("N_c", 4, u: "count")

// a single letter is a variable; its unit binding needs #
#mathscope[
  $h := 2$ $t := 3 h$ #expect("t", 6)
  $t_h := 3 #units.h$ #expect("t_h", 10800, u: "h")
  $t_1 := 3 dot h$ #expect("t_1", 6)
  $U := 5 #V$ #expect("U", 5, u: "V")
  $R := 4.7 kΩ$ #expect("R", 4700, u: "kΩ")
  $s := 2 mΩ$ #expect("s", 0.002, u: "mΩ")
  $rho := 0.0178 #Ω mm^2/#m$ #expect("ρ", 1.78e-8)
  $Q_0 := frac(3450, 2) mAh$ #expect("Q_0", 6210)
  $I := 2 #A$ $P := I^2 dot 3 #Ω => #W$ #expect("P", 12, u: "W")
  $U_2 := 1.5 #V$ #expect("U_2", 1.5)
  $A_x := 12$ $x := A_x dot 2 #A =>$ #expect("x", 24)
  // A·A is A², not "A A"; mol and cd
  $P_2 := I I =>$ #expect("P_2", 4, u: "A^2")
  $n := 2 mmol$ #expect("n", 0.002, u: "mmol") $c := 3 cd$ #expect("c", 3, u: "cd")
  // a variable T and the unit T (tesla) are different things
  $T := 2 #s$ $y := 3 T$ #expect("y", 6, u: "s")
  $B := 3 #units.T =>$ #expect("B", 3, u: "T")
]
// quoted text is a worksheet variable, never a unit
#mathscope[#math(errors: "document")[
  $p := 3 "bar" =>$ #fails("'bar' in 'p' isn't defined")
]]

// own units, conversions, preferred result units
#mathscope[
  #let bar = unit($100 kPa$, display: () => std.math.upright("bar"))
  #let rpm = unit($1 / min$, display: () => std.math.upright("rpm"))
  #let tesla = unit($1 #units.kg / (#s^2 #A)$, display: () => std.math.upright("tesla"))
  $p := 3 bar$ #expect("p", 3e5, u: "bar")
  $F := p dot 2 cm^2 => #N$ #expect("F", 60, u: "N")
  $w := 3000 rpm$ #expect("w", 50, u: "rpm")
  $w_2 := w => Hz$ #expect("w_2", 50, u: "Hz")
  #mathscope(scope: "inherit")[#math(preferred-units: ($bar$,))[
    $p_3 := 2 p =>$ #expect("p_3", 6e5, u: "bar")
  ]]
  $p_4 := 2 p =>$ #expect("p_4", 6e5, u: "bar") // the unit of the inputs
  $B := 2 tesla =>$ #expect("B", 2, u: "tesla")
]

// base units of their own: in products, and numbers as plain factors
#mathscope[
  #let ue = base-unit("ü", display: () => std.math.upright("ü"))
  #let sheep = base-unit("sheep")
  #let dozen = unit($12$, display: () => std.math.upright("dozen"))
  $x := 3 ue$ $y := 2 ue$ $n := 2 dozen$ $a := 4 sheep #m$
  $w := x y$ $v := x + y = ue$
  #expect("x", 3) #expect("y", 2) #expect("n", 24) #expect("a", 4, u: "sheep·m") #expect("w", 6, u: "ü^2") #expect("v", 5, u: "ü")
  #mathscope(scope: "inherit")[#math(errors: "document")[$z := x + 1 #m =>$ #fails("different units")]]
]

// Code identifiers with underscores need # to avoid Typst's math subscripts.
#let fluid_ounce = unit($28.413 #units.ml$, display: $"fl oz"$)
$1 #units.l -> #fluid_ounce$ #passes()
$"volume" := 1 #units.l = #fluid_ounce$ #expect("volume", 0.001)
