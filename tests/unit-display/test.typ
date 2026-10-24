// Units shown as their expression are one factor, in the input as written and in results alike; displays are
// content, strings or functions.
#import "/tests/util.typ": *
#import "/src/engine.typ": _restyle
#import "/src/config.typ": _DEF
#import units: cm, m, s, N
#set page(width: 18cm, height: auto)

#let inch = unit($2.54 #cm$)
#let Nm = unit($#N #m$)
#let inC = unit($2.54 #cm$, display: $"in"$)
#let inS = unit($2.54 #cm$, display: "in")
#let inF = unit($2.54 #cm$, display: () => math.upright("in"))
#let bit = base-unit("information", display: "bit")
#let byte = unit($8 bit$, display: $"B"$)
#let triangle = operation("tri", display: "triangle")

// the input as written: a dot between a number (or factor) and a display that starts with one
#let restyled(eq) = repr(_restyle(if eq.body.has("children") { eq.body.children } else { eq.body }, _DEF + (block: false)))
#let dot = repr(sym.dot.op).trim("symbol(\"", at: start).trim("\")", at: end)
#assert(restyled($1#inch$).contains(dot))
#assert(restyled($1 #inch$).contains(dot))
#assert(restyled($#inch #inch$).contains(dot))
#assert(restyled($2 3$).contains(dot))
#assert(not restyled($3 Nm$).contains(dot))
#assert(not restyled($3 inC$).contains(dot))
#assert(not restyled($2 x$).contains(dot))
#assert(not restyled($x := #inch$).contains(dot))
// operators apply to what follows: no dot after them, also when they end in a digit or unit
#assert(not restyled($sum_(k=1)^10 2^k$).contains(dot))
#assert(not restyled($integral_0^(2 #s) 3 #m dif t$).contains(dot))
#assert(not restyled($sin 2$).contains(dot))
#assert(not restyled($dif / (dif x) 2 x$).contains(dot))
#assert(not restyled($(dif^2) / (dif x^2) 2 x$).contains(dot))
#assert(restyled($x^2 3$).contains(dot))
// and parentheses around one under a power
#assert(restyled($#inch^2$).contains("lr("))
#assert(not restyled($#m^2$).contains("lr("))

#show: worksheet.with(errors: "panic")
$a := 1 #inch = inch$ #expect("a", 0.0254, u: "2.54 cm")
$b := 3 #inch = cm$ #expect("b", 0.0762, u: "cm")
$A := #inch^2 = inch^2$ #expect("A", 0.0254 * 0.0254)
$k := 1 / #inch = 1/inch$ #expect("k", 1 / 0.0254)
$T := 3 Nm =$ #expect("T", 3)
$x := 3 inC = inC$ #expect("x", 0.0762, u: "in")
$y := 3 inS = cm$ #expect("y", 0.0762)
$z := 3 inF = cm$ #expect("z", 0.0762)
$d := 3 byte = bit$ #expect("d", 24)
$t := triangle(1/2) =$ #expect("t", 0.5)
#show: worksheet.with(separator: ",")
$w := 2 #inch = inch$ #expect("w", 0.0508)
