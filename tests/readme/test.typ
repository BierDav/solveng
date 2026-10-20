// The examples of the README, compared with reference images.
#import "/lib.typ": *
#set page(width: 16cm, height: auto, margin: 1cm)
#set text(lang: "de")

// Math and code in a solve-block (plain code calculated too)
#solve-config(integration: "full")[#solve-block[
  The battery holds `Q0 = 3450 mAh` and the load draws $I = 30 "mA"$, so it lasts `= Q0/I -> h`.
  ```
  t = Q0 / (2 I) -> h     // half the charge
  P(U) = U I
  P1 = P([5 V])
  ```
  #defined(included: "given")
]]

// Calls: def and solve
#solve-block[
  #def("Q0 = 3450 mAh", "I = 30 mA", "U = 5 V")
  #defined()
  #solve("Q0/2 = I t", unit: "h")
  #solve("R <= U/I")
  #solve("x^2 + 2x - 3 = 0")
  Runtime: #solve("Q0/I", unit: "h", inline: true, steps: "value")
]

// Units and variables
#solve-block[
  #def("A = 12")
  #solve("x = A [A]")
  #def("T = 2 s")
  #solve("f = 1/(T)")
  #def("g = 1/T")
]

// Plots
#solve-block[
  #def("R = 50 Ω", "P0 = 2 W")
  #solve("P0 = U^2/R")
  #graph("P = U^2/R", x: "U")
  #graph("P = U^2/R", x: "R", xrange: ("10 Ω", "1 kΩ"), width: 10cm)
]

// Options
#solve-block[
  #show: solve-config.with(digits: 3, layout: "column")
  #def("U = 5 V", "I = 30 mA")
  #solve-config(form: "fraction")[#solve("I3 = I/3")]
  #solve("R = U/I", digits: 2, separator: ",")
  #defined(show-si: true, style: "inline")
]
