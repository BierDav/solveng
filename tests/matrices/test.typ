// Matrices as Typst writes them: mat(…; …) with blocks, A^T and A^top, det, tr and norm, times; units per row and column.
#import "/lib.typ": *
#import "/tests/util.typ": expect, passes, fails, result
#import functions: *
#set page(width: 18cm, height: auto)
#show: worksheet.with(errors: "panic")

$A := mat(2, 1; 1, 3)$ $b := vec(3, 5)$
$x := A^(-1) b =$ #passes() #result("vec(\"0.8\", \"1.4\")")
$d := det(A) =$ #expect("d", 5)
$t := tr A =$ #expect("t", 5)
$n := |vec(3, 4)| =$ #expect("n", 5)
$n_2 := norm(vec(3, 4)) + ‖vec(0, 1)‖ =$ #expect("n_2", 6)
$B := A^(-1) A =$ #result("mat(\"1\", \"0\"; \"0\", \"1\")")
$C := A^T - A^top =$ #result("mat(\"0\", \"0\"; \"0\", \"0\")")
$s := vec(1, 2)^T A vec(1, 2) =$ #expect("s", 18)
$c := vec(1, 0, 0) times vec(0, 1, 0) =$ #result("vec(\"0\", \"0\", \"1\")")
$S := mat(A; mat(7, 8)) =$ #result("mat(\"2\", \"1\"; \"1\", \"3\"; \"7\", \"8\")")
$e := at(S, 2, 1) =$ #expect("e", 8)
$A vec(u, v) = b -> solve(u, v)$ #expect("u", 0.8) #expect("v", 1.4)

// units: a unit per row and per column
$R := mat(2 unit("Ω"), 1 unit("Ω"); 1 unit("Ω"), 3 unit("Ω"))$
$i := R^(-1) vec(1 unit("V"), 2 unit("V")) =$ #passes()
$i_1 := at(i, 0) =$ #expect("i_1", 0.2, u: "mA")

$K := mat(A, b; 0, 0, 1) =$ #result("mat(\"2\", \"1\", \"3\"; \"1\", \"3\", \"5\"; \"0\", \"0\", \"1\")")

// symbolic
$M := mat(p, q; r, w)$
$det(M) ->$ #result("p w - q r")

#worksheet(errors: "document")[
  $mat(1, 2; 3, 4) mat(1, 2, 3; 4, 5, 6; 7, 8, 9) =$ #fails("columns of the first must match the rows of the second")
  $vec(1, 2) vec(1, 2, 3) =$ #fails("same length")
]
