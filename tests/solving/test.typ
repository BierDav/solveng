// The solver and the worksheet's semantics: := defines (symbolically), = compares, a trailing = shows the
// value, -> the symbolic result or a solution.
#import "/tests/util.typ": *

#worksheet[
  // Mathcad's =: a comparison, 1 or 0; different units are never equal
  $M := 1 = 2 =$ #expect("M", 0) #result("\"0\"")
  $M := 1 A = 2 V ->$ #expect("M", 0) #result("\"0\"")
  $M =$ #result("\"0\"")
  $N := 2 A = 2000 mA$ #expect("N", 1)
  $Z := 3 A < 4 A$ #expect("Z", 1)
  #def("c_s := 2 A != 2 V") #expect("c_s", 1)

  // simplify keeps undefined names, a function needn't define what it uses
  $q(x: A, ..) := 1 + 3/3x + y^2 -> simplify$ #result("x + y^(\"2\") + \"1\"")
  $0 = 1 + 3/x + y^2 -> solve(x)$ #result("x = -frac(\"3\", y^(\"2\") + \"1\")") #undefined("x")
  $f(x) := (x^2 - 1)/(x - 1) -> simplify$ #result("x + \"1\"")
  $c := 1/3 -> frac$ #expect("c", 1 / 3) #result("frac(\"1\", \"3\")")

  // definitions are symbolic: put in when defined, names without a value stay
  $h := 5 z$ $z := 20 cm$
  $h =$ #result("\"1\" thin upright(\"m\")")
  $U(x: m) := h*x*a ->$ #result("\"1\" thin upright(\"m\") dot a x")
  $h(x: m) := U(x)/a ->$ #result("\"1\" thin upright(\"m\") dot x")
  $y_h := h(3) =$ #expect("y_h", 3)
  // a name used before it's defined is looked up when the value is needed
  $k_l := 2 w_l$ $w_l := 4$ $k_l =$ #expect("k_l", 8)
  $p_u := r_u + 1 =$ #fails("'r_u' in 'p_u' isn't defined")
]

// solving: isolating, polynomials, systems, inequalities, numerically
#worksheet[
  $Q_0 := 3450 mAh$ $I := 30 mA$
  $Q_0/2 = I t -> solve(t)$ #expect("t", 207000, u: "h")
  $x^2 + 2x - 3 = 0 -> solve(x)$ #expect("x", 1)
  $a + b = 3, a - b = 1 -> solve(a, b)$ #expect("a", 2) #expect("b", 1)
  $cos(w) = w -> solve(w)$ #expect("w", 0.7390851332151607)
  $U := 5 V$
  $R <= U/I -> solve(R)$ #expect("R", 5 / 0.03)
  $x_3^3 - 6x_3^2 + 11x_3 - 6 = 0 -> solve(x_3)$ #passes()
  $e^v = 2 -> solve(v)$ #expect("v", calc.ln(2))
  $p^2 = -1 -> solve(p)$ #undefined("p")
  $Q_0/2 = I t_2$ // prose: t_2 isn't defined
  $s := 2 -> solve(s)$ #fails("solve(…) solves an equation")
]

// = states something: checked when everything is known, prose when not
#worksheet[
  $U := 5 V$ $R := 2 Ω$ $I := U/R$
  $U = R I$ #passes()
  $U = 6 V$ #fails("has another value than")
  $U = 5 A$ #fails("has another unit than")
  $E = m c^2$ #passes()
]

// functions: units of parameters, any, .., types taken from where they're passed, comparisons
#worksheet[
  #def("q(x: A) := x [2 Ω]", "f(I: mA, R) := I R")
  #def("y := q(3)", "z := q([2 mA])", "u := f(3, [2 Ω])")
  #expect("y", 6) #expect("z", 0.004) #expect("u", 0.006)
  #def("w := q([2 V])") #fails("q takes x in A, got V")
  #def("g(x, ..) := x m", "y2 := g(2, 4)") #expect("y2", 8)
  #def("r(y) := q(y)", "z2 := r(2)") #expect("z2", 4)
  #def("s(y: V) := q(y)") #fails("q takes x in A, got V")
  #def("U_c(x, y) := x = y", "a_c := U_c(2, 2)", "b_c := U_c(2, 3)") #expect("a_c", 1) #expect("b_c", 0)
  #def("L_c(x: A, y: A) := x < y", "c_c := L_c(1, 2)", "g_c := L_c(2, [1 A])") #expect("c_c", 1) #expect("g_c", 0)
  #def("k_r(x) := k_r(x) + 1") #fails("k_r isn't defined")
]

// casts: e as u is e's number in SI units as a value in u; after a number as is the unit, attosecond
#worksheet[
  #def("U := 5 V", "I := U as A", "J := U as mA", "n := 2 * U as 1", "K := (U * 2) as [mA]", "L := U^2 as V")
  #expect("I", 5) #expect("J", 0.005) #expect("n", 10) #expect("K", 0.01) #expect("L", 25)
  #def("a := 3 as", "b := U as as", "c := 3 as A")
  #expect("a", 3e-18) #expect("b", 5e-18) #expect("c", 3e-18)
  $M := U "as" mA$ #expect("M", 0.005)
  $e_a := 3 "as"$ #expect("e_a", 3e-18)
  #def("w := U as furlong") #fails("unknown unit 'furlong'")
]

// := assigns, the last assignment wins; a variable takes no type, a function returns what its body gives
#worksheet[
  $I_r := 4 A$ $I_r := 2 A$ #expect("I_r", 2)
  $I_r := I_r + 1 A$ #expect("I_r", 3)
  #def("I_t: mA := 3 A") #fails("takes no type")
  #def("Y_t(x): V := 3 x") #fails("a function returns what its body gives")
]

// code lines: a trailing = shows, -> solves, = alone states
#worksheet[
  ```solveng
  Q0 := 3450 mAh; I := 30 mA
  t := Q0/I = [h], digits(3)
  Q0/I =
  x^2 = 9 -> solve(x)
  a + b = 3; a - b = 1 -> solve(a, b)
  ```
  #expect("t", 414000, u: "h") #expect("x", 3) #expect("a", 2) #expect("b", 1)
]

// code in a worksheet: solveng code blocks are calculated, inline and untagged code never
#worksheet[
  `x := 3` ```solveng y := 2 m```
  ```
  z := 4
  ```
  ```solveng
  y := 2 m
  w := y 2 = [cm]
  ```
  #undefined("x") #undefined("z")
  #expect("y", 2) #expect("w", 4, u: "cm")
]
// evaluate: false leaves the worksheet's math alone, not its code blocks and calls, and isn't inherited
#worksheet[
  #worksheet(evaluate: false, inherit: true)[$d := 2$ ```solveng
  e := 3
  ```
  #expect("e", 3) #undefined("d")
  #worksheet($d_e := e + 1$) #expect("d_e", 4)
  #worksheet(inherit: true)[$f_e := 6$ #expect("f_e", 6)]
  ]
  $g := 5$ #expect("g", 5)
]
#context {
  let r = query(<solveng>).map(it => it.value).filter(x => x.kind == "math")
  assert(r.any(x => not x.active and not x.evaluate), message: "math in evaluate: false reported as calculated")
}
// a code block passed to worksheet is code without the solveng tag
#worksheet(```
t_c := 2 s
v_c := 4 m / t_c
```)
#expect("t_c", 2) #expect("v_c", 2)

// editors read what a worksheet saw: each element (by location) and what the block defined
#worksheet[$k := 2 V$ `m := 1`]
#context {
  let r = query(<solveng>).map(it => it.value)
  assert(r.any(x => x.kind == "math" and x.active and type(x.element) == location))
  assert(r.any(x => x.kind == "block" and x.defined.any(d => d.name == "k" and d.value == "2 V")))
}
// only the innermost worksheet reads an element
#worksheet[
  #worksheet[$h := 3$ #expect("h", 3)]
  $j := 4$ #expect("j", 4) #undefined("h")
]
#context {
  let r = query(<solveng>).map(it => it.value).filter(x => x.kind == "math" and x.active)
  assert.eq(r.len(), r.map(x => x.element).dedup().len(), message: "an element was read twice")
}
// worksheet-config calculates nothing on its own
#worksheet-config(digits: 3)[$o := 1$]
#undefined("o")

// calls: equations where they stand, hidden ones silently, options for them only
#worksheet($Delta_R := 5 cm$, "g_2 := 2 m")
#expect("Δ_R", 0.05) #expect("g_2", 2)
#worksheet($n_h := 7$, hidden: true) #expect("n_h", 7)
#worksheet[$n_i := 1$ #worksheet($n_j := n_i + 1$) #expect("n_j", 2)]

// clear: the names given (a value and a function share a name), or all: back to how the worksheet started
#worksheet[
  $r_1 := 1$ #clear(all: true) #undefined("r_1")
  $c_1 := 1$ $c_2 := 2$ $f_c(x) := 2 x$ $tau := 3$
  #clear("c_1", $f_c$, "tau") #passes() #undefined("c_1") #undefined("f_c") #undefined("τ") #expect("c_2", 2)
  $g_c(x) := x$ $g_c := 4$ #clear("g_c()") #undefined("g_c")
  #worksheet-config(errors: "document")[
    #clear("c_1") #fails("'c_1' isn't defined")
    #clear("1 + c_2") #fails("isn't a name")
  ]
  #expect("c_2", 2)
]
#worksheet(clear-at: heading)[
  $r_2 := 2$
  = Next
  #undefined("r_2")
]

// hidden: what a worksheet calculates isn't shown, the rest is; nested worksheets show theirs again
#worksheet[
  $a_w := 2$
  #let hid = worksheet(hidden: true, inherit: true)[
    $b_w := 3 a_w =$
    ```solveng
    c_w := 4 [m]
    ```
    #worksheet($d_w := b_w + 1 =$)
    #expect("d_w", 7)
  ]
  #hid
  #context {
    let empty = measure(worksheet(hidden: true)[$e_w := 1 =$ ```solveng
    f_w := 2
    ```])
    assert.eq(empty.height, 0pt, message: "hidden math and code shown")
    assert(measure(worksheet(hidden: true)[text $E = m c^2$]).width > 0pt, message: "hidden text and prose not shown")
    assert(measure(worksheet(hidden: true)[#worksheet($g_w := 1 =$)]).width > 0pt, message: "nested worksheet hidden")
  }
  // content doesn't inherit by default, equations do; inherit: false calculates them on their own, and what they
  // define comes back
  #worksheet[$a_w := 5$ #expect("a_w", 5)]
  #expect("a_w", 2) #undefined("b_w")
  #worksheet($k_w := a_w + 1$) #expect("k_w", 3)
  #worksheet($m_w := 7$, inherit: false) #expect("m_w", 7)
  #worksheet($n_w := a_w =$, inherit: false) #fails("'a_w' in 'n_w' isn't defined")
  #worksheet(hidden: true)[$a_w := 9$ #undefined("k_w")]
  #worksheet(hidden: true, inherit: true)[#expect("k_w", 3)]
]

// graph: curves written with := define what they name, as the same equation in a worksheet
#worksheet[
  $X_g(x) := x^2 + 1$
  #graph($y_g := X_g(x)$)
  #worksheet(inherit: true)[$x := 2$ $v := y_g$ #expect("v", 5)]
  #graph($y_g := 2 y_g$)
  #worksheet(inherit: true)[$x := 2$ $v := y_g$ #expect("v", 10)]
  #graph($y_g(y_g) := X_g(y_g)$)
  $v_g := y_g(3)$ #expect("v_g", 10)
  // a parameter named like a value, and a constant: a line over it
  $C_g(x) := 20$ $c_g := 4$
  #graph($c_g(c_g) := C_g(c_g)$) #passes()
  $v_c := c_g(1)$ #expect("v_c", 20)
  #graph("w_g(t) = X_g(t)") #undefined("w_g")
]
