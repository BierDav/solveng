// Names: spelled Greek letters in code and math.
#import "/tests/util.typ": *
#import units: *

// a spelled Greek letter alone or before an index is the letter, in code as in math; DeltaT is a name of its own
#worksheet[
  $Delta_R := 5 cm$
  ```solveng
  Delta_T := 3 K
  alpha2 := 2
  DeltaT := 4 K
  ```
  $x := 2 Delta_T$
  #def("f(alpha) := 2 alpha", "k := f(3)")
  #expect("Δ_R", 0.05) #expect("Δ_T", 3) #expect("α2", 2) #expect("DeltaT", 4) #expect("x", 6) #expect("k", 6)
  $Delta_x := 2 Delta_T$
  #def("q := 3 Delta_x -> substitute(Delta_x)")
  #expect("q", 18)
]

// empty strings glue names as in math: Delta""r_R is Δr_R in math, code and strings
#worksheet[
  $Delta""r_R := 5 cm$
  $Delta""Gamma""Omega := 2$
  ```solveng
  a := Delta""r_R * 2
  b := ΔΓΩ + Δr_R
  ```
  #def("c := Delta\"\"Gamma\"\"Omega")
  #expect("Δr_R", 0.05) #expect("ΔΓΩ", 2) #expect("a", 0.1) #expect("c", 2)
  #context assert(calc.abs(lambdify("Delta\"\"r_R * k", "k")(2) - 0.1) < 1e-12)
]

// names of more letters are quoted in math, as typst needs them to be; so are long subscripts, which may be grouped
#worksheet[
  $"tews" := 23 "V"$
  $"test"_("test" Delta) := 3$
  $"x"_"in" := 2$
  $R_(a b) := 4$
  ```solveng
  k := tews * 2
  y := test_test""Delta * x_in
  ```
  $q := "tews" / R_(a b)$
  #expect("tews", 23) #expect("test_testΔ", 3) #expect("x_in", 2) #expect("R_ab", 4) #expect("k", 46) #expect("y", 6) #expect("q", 5.75)
]

// Quotes transport names; digits stay in the identifier and only underscores make subscripts.
#worksheet[
  $"Delta2tau" := 7$
  $a := "Δ2τ" =$
  #passes() #expect("a", 7)
  #def("b := Delta2tau", "c := Δ2τ")
  #expect("b", 7) #expect("c", 7)
  $Delta""tau := 8$
  $d := "Delta\"\"tau" =$
  #passes() #expect("d", 8)
  $t""2 := 3$
  $t_2 := 5$
  $q := "t2" + t_2 =$
  #passes() #expect("q", 8)
  $"😅2Delta_🫠" := 9$
  #def("emoji := 😅2Δ_🫠")
  #expect("emoji", 9)
]

// The acute accent is a literal part of a name.
#worksheet[
  $x := 2$
  $x´ := 3$
  $x´´ := 5$
  $´x := 7$
  $x´_2 := 11$
  $a := x + x´ + x´´ + ´x + x´_2 =$
  #passes() #expect("a", 28)
  $f´(t´) := x´ t´$
  $b := f´(4) =$
  #passes() #expect("b", 12)
  ```solveng
  c := x´ + x´´ =
  ```
  #passes() #expect("c", 8)
]

#worksheet[
  $r := 2$
  $r` := 3$
  $a := r + r` =$
  #passes() #expect("a", 5)
]

// A join keeps leading digits in a name instead of multiplying a coefficient.
#worksheet[
  $x := 3$
  $2""x := 7$
  #passes() #expect("2x", 7)
  $a := 2""x =$
  #passes() #expect("a", 7)
  $b := 2x =$
  #passes() #expect("b", 6)
  #def("c := 2\"\"x")
  #passes() #expect("c", 7)
  $2"""foo" := 11$
  #passes() #expect("2foo", 11)
  $2""x_tau := 13$
  #passes() #expect("2x_τ", 13)
]
