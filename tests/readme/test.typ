// The examples of the README, compared with reference images.
#import "/lib.typ": *
#set page(width: 16cm, height: auto, margin: 1cm)
#show: solve-config.with(errors: "document")
#set text(lang: "de")
#def("Q0 = 3450 mAh", "I = 30 mA")
#definitions()
#solve("Q0/2 = I t", unit: "h")
#solve-block[
  The load draws $I = "30 mA"$ (#solve("I", unit: "A", steps: "value", inline: true)),
  the battery holds $Q_0 = 3450 "mAh"$, so it lasts $t = Q_0 / I$ = #val("t", unit: "h").
]
#solve-block[
  The battery holds `Q0 = 3450 mAh` and the load draws `I = 30 mA`,
  so it lasts `= Q0/I -> h`.
  ```
  t = Q0 / (2 I) -> h     // half the charge
  P(U) = U I
  P1 = P([5 V])
  ```
  #definitions(included: "given")
]
#def("R = 7.2 kOhm") #def("U = R I") #solve("U")
#def("f(x) = x^2 + 1", "g(a, b) = a b + b")
#solve("f(1) = x") #solve("z^2 + 2z - 3 = 0") #reset
#def("P = 2 W", "R = 50 Ω") #solve("P = U I", "U = R I") #reset
#def("U = 5 V", "I = 30 mA") #solve("R <= U/I") #solve("R")
#reset
#def($I = frac(2, 4) "A"$, "Q0 = 3450/2 mAh", "U = (1+2)*1,5 V", "A = 2^3 m^2")
#def("R2 = 2 kOhm", "Z = [5 V] / I") #definitions(si: true)
#solve-config(form: "fraction")[#solve("I3 = I/3")]
#solve-block[
  $rho = 0.0178 "Ω mm"^2/"m"$, $l = 10 "m"$, $h_t = 1 "mm"$, $R = 0.2 Omega$
  $R = rho dot frac(l, A_q) = rho dot frac(l, b dot h_t)$
  #definitions()
]
#def("R5 = 50 Ω", "P0 = 2 W")
#solve("P0 = U5^2/R5")
#graph("P = U5^2/R5", x: "U5")
#graph("cos(w) = w")
#graph("y = x^2", "y = 2x + 3", colors: (gray, blue))
#def("v0 = 2 m/s", "a = 1 m/s^2")
#graph("s = v0 t + a/2 t^2", xrange: ("-6 s", "3 s"))
#graph("P = U5^2/R5", x: "R5", xrange: (0, "2 R5"), mark: ("R5", "extrema"))
