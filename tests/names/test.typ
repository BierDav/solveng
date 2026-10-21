// Names: spelled Greek letters in code and math.
#import "/tests/util.typ": *

// a spelled Greek letter alone or before an index is the letter, in code as in math; DeltaT is a name of its own
#solve-block[
  #solve-config(integration: "full")[
    $Delta_R = 5 "cm"$
    `Delta_R`
    ```
    Delta_T = 3 K
    alpha2 = 2
    DeltaT = 4 K
    ```
    $x = 2 Delta_T$
    #def("f(alpha) = 2 alpha")
    #solve("k = f(3)")
    #expect("Δ_T", 3) #expect("α2", 2) #expect("DeltaT", 4) #expect("x", 6) #expect("k", 6)
    $Delta_x = 2 Delta_T$
    #solve("q = @Delta_x")
    #expect("q", 6)
  ]
]

// empty strings glue names as in math: Delta""r_R is Δr_R in math, code and strings
#solve-block[
  #solve-config(integration: "full")[
    $Delta""r_R = 5 "cm"$
    $Delta""Gamma""Omega = 2$
    ```
    a = Delta""r_R * 2
    b = ΔΓΩ + Δr_R
    ```
    #solve("c = Delta\"\"Gamma\"\"Omega")
    #expect("Δr_R", 0.05) #expect("ΔΓΩ", 2) #expect("a", 0.1) #expect("c", 2)
    #context assert(calc.abs(lambdify("Delta\"\"r_R * k", "k")(2) - 0.1) < 1e-12)
  ]
]

// names of more letters are quoted in math, as typst needs them to be; so are long subscripts, which may be grouped
#solve-block[
  #solve-config(integration: "full")[
    $"tews" = 23 "V"$
    $"test"_("test" Delta) = 3$
    $"x"_"in" = 2$
    $R_(a b) = 4$
    ```
    k = tews * 2
    y = test_test""Delta * x_in
    ```
    $q = "tews" / R_(a b)$
    #expect("tews", 23) #expect("test_testΔ", 3) #expect("x_in", 2) #expect("R_ab", 4) #expect("k", 46) #expect("y", 6) #expect("q", 5.75)
  ]
]
