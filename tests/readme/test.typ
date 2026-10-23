// The examples of the README, compared with reference images.
#import "/lib.typ": *
#set page(width: 16cm, height: auto, margin: 1cm)
#set text(lang: "de")

// A worksheet: math and a code block
#worksheet[
  The battery holds $Q_0 := 3450 mAh$ and the load draws $I := 30 mA$, so it lasts $t := Q_0 / I = 1 h$.
  ```solveng
  t_half := Q_0 / (2 I) = [h]     // half the charge
  P(U) := U I
  P_1 := P([5 V]) =
  ```
  #defined(included: "given")
]

// Solving
#worksheet[
  $Q_0 := 3450 mAh$, $I := 30 mA$, $U := 5 V$
  #defined()
  $Q_0/2 = I t -> solve(t)$ and $t = 1 h$
  $R <= U/I -> solve(R)$
  $x^2 + 2x - 3 = 0 -> solve(x)$
  Runtime: $Q_0/I = 1 h$
]

// Units and variables
#worksheet[
  $A := 12$, $x := A dot 2 A =$
  $T := 2 s$, $f := 1/T = Hz$
]

// Plots
#worksheet[
  #worksheet("R := 50 Ω", "P0 := 2 W", "U0 := 10 V", hidden: true)
  #graph("P(U: V) = U^2/R", ymark: "P0")
  #graph("P(..) = U0^2/R_L", xrange: ("10 Ω", "1 kΩ"), width: 10cm)
]

// Options
#worksheet(digits: 3, layout: "column")[
  $U := 5 V$, $I := 30 mA$
  #worksheet($I/3 =$, form: "fraction")
  #worksheet(evaluate: false)[Equations like $E = m c^2$ stay as written.]
  #worksheet($R := U/I =$, digits: 2, separator: ",")
  #defined(show-si: true, style: "inline")
]
