#import "/lib.typ": *
#import "/tests/util.typ": expect, undefined, passes
#set page(width: 18cm, height: auto)
#show: worksheet-config.with(errors: "panic")
#show: worksheet

// Auto exports expressions and keeps blocks local.
$x := 10$
#worksheet($a := x+1$, export: auto)
#expect("a", 11)
#worksheet(export: auto)[$x := 99$ $"local" := 1$]
#expect("x", 10) #undefined("local")

// Read from and write to the parent independently.
#worksheet(inherit: true, export: false)[$x := x+1$ #expect("x", 11)]
#expect("x", 10)
#worksheet(inherit: true, export: true)[$x := x+2$ $b := a+1$]
#expect("x", 12) #expect("b", 12)
#worksheet(inherit: false, export: true)[#undefined("x") $x := 20$]
#expect("x", 20) #expect("a", 11)
#worksheet(inherit: false, export: false)[#undefined("x") $x := 30$]
#expect("x", 20)

// Expression calls can also be isolated in either direction.
#worksheet($x := 40$, "temporary := x+1", export: false)
#expect("x", 20) #undefined("temporary")
#worksheet("x := 50", inherit: false, export: false)
#expect("x", 20)
#worksheet("from_call := 7", inherit: false, export: auto)
#expect("from_call", 7)

// Export functions, solved values, code and graph definitions as well as variables.
#worksheet(export: true, hidden: true)[
  $U := 12 V$
  $R := 100 Ω$
  $U = I R -> solve(I)$
  $f(t) := t^2+1$
  ```solveng
  coded := 8
  ```
  #graph("g(t) := t+2", xrange: (0, 1))
]
#expect("I", 0.12) #expect("coded", 8)
$y := f(3) =$ #expect("y", 10)
$z := g(3) =$ #expect("z", 5)

// Export reaches only the immediate parent; nested worksheets keep their own defaults.
#worksheet[
  #worksheet(export: true)[$"nested" := 2$]
  #expect("nested", 2)
]
#undefined("nested")
#worksheet(export: true)[
  #worksheet[Local calculation: $"private" := 3$]
  #undefined("private")
  #worksheet(export: true)[$"nested" := 4$]
]
#expect("nested", 4) #undefined("private")

// Clearing names or restarting a child does not delete definitions in the parent.
#worksheet(inherit: true, export: true)[
  #clear("x")
  $"retained" := 6$
]
#expect("x", 20) #expect("retained", 6)
#worksheet(export: true, clear-at: heading)[
  = First
  $"discarded" := 1$
  = Second
  $"kept" := 2$
]
#undefined("discarded") #expect("kept", 2)
#worksheet(export: true)[$"discarded" := 3$ #clear(all: true) $"kept_after_clear" := 4$]
#undefined("discarded") #expect("kept_after_clear", 4)

// An exported definition can change a name from a variable to a function and back.
#worksheet(export: true)[$x(t) := 2t$]
$"function_value" := x(3) =$ #expect("function_value", 6)
#worksheet(export: true)[$x := 60$]
#expect("x", 60)

// Typst collapses a single-expression content block; explicit false keeps it local.
#worksheet[$"collapsed" := 1$]
#expect("collapsed", 1)
#worksheet(export: false)[$"collapsed" := 2$]
#expect("collapsed", 1)
