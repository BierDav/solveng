// Units and variables: what is a unit, what is a variable, and what is refused as unclear.
#import "/tests/util.typ": *

// a unit right after a number, names in equations are variables
#def("A = 12")
#solve("x = A [A]") #expect("x", 12, u: "A")
#solve("y = A A", unit: "A") #fails("result has unit 1")
#fails("for the unit write [A]")
#solve("z = [A] A") #expect("z", 12)
#reset

// a value whose unit is also a variable is refused
#def("T = 2 s")
#def("f = 1/T") #fails("both a defined variable and a unit")
#def("y = 3 T") #fails("both a defined variable and a unit")
#def("f2 = 1/(T)") #expect("f2", 0.5)
#def("y2 = 3*T") #expect("y2", 6)
#def("B = [3 T]") #expect("B", 3, u: "T")
#reset

// math: quoted is a unit, bare letters are variables if defined, else units after a number
#solve-block[
  $h = 2$ $t = 3 h$ #expect("t", 6)
  $t_2 = 3 "h"$ #expect("t_2", 10800)
  $U = 5 V$ #expect("U", 5, u: "V")
  $R = 4.7 "k"Omega$ #expect("R", 4700)
  $R_2 = 1 "k" Omega$ #expect("R_2", 1000)
  $s = 2 "m" Omega$ #expect("s", 2)
  $rho = 0.0178 "Ω mm"^2/"m"$ #expect("ρ", 1.78e-8)
  $Q_0 = frac(3450, 2) "mAh"$ #expect("Q_0", 6210)
]
#def($I = 2 "A"$) #expect("I", 2)
#def("h = 2") #def($t = 3 h$) #expect("t", 6)
#reset

// A·A is A², not "A A"
#def("I = 2 A") #solve("P = I I") #expect("P", 4, u: "A^2")
#reset

// mol and cd
#def("n = 2 mmol", "c = 3 cd") #expect("n", 0.002, u: "mmol") #expect("c", 3)
#reset

// custom units, removing units, preferred result units
#solve-config(units: (bar: "100 kPa", rpm: "1/min"))
#def("p = 3 bar") #expect("p", 3e5)
#def("F = p [2 cm^2]") #expect("F", 60, u: "N")
#def("w = 3000 rpm") #expect("w", 50)
#solve("w2 = w", unit: "Hz") #expect("w2", 50, u: "Hz")
#def("p2 = 2 mbar") #expect("p2", 200)
#solve-config(preferred-units: ("bar",))[
  #def("p3 = 2 p") #expect("p3", 6e5, u: "bar")
]
#def("p4 = 2 p") #expect("p4", 6e5, u: "bar") // the unit of the inputs
#solve-config(units: (T: none))
#def("T = 2 s") #def("y = 3 T") #expect("y", 6) // T is no unit any more: 3 times T
#def("x = [2 T]") #fails("unknown unit 'T' (it was removed")
#reset
#solve-config(units: (T: "1 kg/(s^2 A)"))
#def("B = 2 T") #expect("B", 2)
#reset

// strict: unit names can't be variables, units in math must be quoted
#solve-config(strict: true)
#def("A = 3 m^2") #fails("is a unit and can't be a variable")
#def("f(V) = 2 V") #fails("is a unit and can't be a variable")
#def("Area = 3 m^2") #passes()
#def("P_1 = Area [2 Pa]") #expect("P_1", 6)
#solve("5 = A_1 + 2") #expect("A_1", 3)
#context assert("err" in _math-src("5 V", (:), _DEF + (strict: true)))
#context assert("err" not in _math-src("5 \"V\"", (:), _DEF + (strict: true)))
#context assert("err" not in _math-src("5 Ω", (:), _DEF + (strict: true)))
#solve-config(strict: false)

// options are checked
#def("x = 1", units: (bar: "1e5 Pa")) #context assert(_st.get().at("x", default: none) == none)

// auto: a base unit of its own, with prefixes and in products; numbers are plain factors
#solve-block[
  #solve-config(units: (ü: auto, sheep: auto, dozen: 12))
  #def("x = 3 mü", "y = 2 ü", "n = 2 dozen", "a = 4 [sheep m]")
  #solve("w = x * y")
  #solve("v = x + y", unit: "ü")
  #expect("x", 0.003) #expect("y", 2) #expect("n", 24) #expect("a", 4) #expect("w", 0.006) #expect("v", 2.003)
  #context assert.eq(_st.get().at("v").u, "ü")
  #solve("z = x + [1 m]")
  #fails("different units")
]

// Omega is Ω; a prefix goes with it glued, as in math, never written together
#def("R1 = 3 k\"\"Omega", "R2 = 3 kΩ", "R3 = 2 Omega")
#expect("R1", 3000) #expect("R2", 3000) #expect("R3", 2)
#def("R4 = 3 kOhm")
#fails("kOhm")
#def("R5 = 3 kOmega")
#fails("kOmega")
