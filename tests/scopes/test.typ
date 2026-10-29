#import "/src/unit-words.typ" as units
#import "/src/constants.typ" as constants
#import "/src/builtins.typ": *
#import "/lib.typ": *
#import "/tests/util.typ": expect, undefined, passes, fails, reported
#import modifiers: solve
#set page(width: 18cm, height: auto)
#show: worksheet

// auto: equations are shared, content is isolated
$x := 10$
#calculate($a := x+1$)
#expect("a", 11)
#worksheet[#undefined("x") $x := 99$ $"local" := 1$]
#expect("x", 10) #undefined("local")
#worksheet(scope: auto)[#undefined("x")]

// inherit: sees the outer definitions, its own end with it
#worksheet(scope: "inherit")[$x := x+1$ #expect("x", 11) $"own" := 1$]
#expect("x", 10) #undefined("own")
#worksheet($x := 40$, $"temporary" := x+1$, scope: "inherit") #expect("x", 10) #undefined("temporary")
// shared: sees them, and what it defines stays, over the outer ones
#worksheet(scope: "shared")[$x := x+2$ $b := a+1$]
#expect("x", 12) #expect("b", 12)
#worksheet($y_s := x + 1$, scope: "shared") #expect("y_s", 13)
// isolated: starts empty, its definitions end with it
#worksheet(scope: "isolated")[#undefined("x") $x := 30$ #expect("x", 30)]
#expect("x", 12)
#worksheet($x := 50$, $"iso" := 2$, scope: "isolated") #expect("x", 12) #undefined("iso")
#worksheet($z_i := x =>$, scope: "isolated") #fails("'x' in 'z_i' isn't defined")

// shared keeps functions, solved values, code and graph definitions as well as variables
#worksheet(scope: "shared", hidden: true)[
  $U := 12 #units.V$
  $R := 100 #units.Ω$
  $U = I R -> solve(I)$
  $f(t) := t^2+1$
  $"coded" := 8$
  #graph($g(t) := t+2$, x-range: (0, 1))
]
#expect("I", 0.12) #expect("coded", 8)
$y := f(3) =>$ #expect("y", 10)
$z := g(3) =>$ #expect("z", 5)

// shared reaches only the immediate parent; nested worksheets have their own defaults
#worksheet[
  #worksheet(scope: "shared")[$"nested" := 2$]
  #expect("nested", 2)
]
#undefined("nested")
#worksheet(scope: "shared")[
  #worksheet[Local calculation: $"private" := 3$]
  #undefined("private")
  #worksheet(scope: "shared")[$"nested" := 4$]
]
#expect("nested", 4) #undefined("private")

// clearing goes back to the worksheet's start, the outer definitions included for inherit and shared; the parent
// keeps what was cleared
#worksheet(scope: "shared")[
  #clear("x")
  #undefined("x")
  $"retained" := 6$
]
#expect("x", 12) #expect("retained", 6)
#worksheet(scope: "shared", clear-at: heading)[
  = First
  $"discarded" := 1$
  = Second
  #expect("x", 12) #undefined("discarded")
  $"kept" := 2$
]
#undefined("discarded") #expect("kept", 2)
#worksheet(scope: "inherit")[$"discarded" := 3$ #clear(all: true) #expect("x", 12) #undefined("discarded")]
#worksheet(scope: "isolated")[$"discarded" := 3$ #clear(all: true) #undefined("x") #undefined("discarded")]

// a shared definition can change a name from a variable to a function and back
#worksheet(scope: "shared")[$x(t) := 2t$]
$"function_value" := x(3) =>$ #expect("function_value", 6)
#worksheet(scope: "shared")[$x := 60$]
#expect("x", 60)

// Typst collapses a content block of one equation into the equation: shared, unless scope says otherwise
#worksheet[$"collapsed" := 1$]
#expect("collapsed", 1)
#worksheet(scope: "isolated")[$"collapsed" := 2$]
#expect("collapsed", 1)

// clear-at: every heading, headings of one level, or every element with a label
#worksheet(clear-at: heading.where(level: 1))[
  = One
  $p := 1$
  == Two
  #expect("p", 1)
  = Three
  #undefined("p")
]
#worksheet(clear-at: <exercise>)[
  = Task <exercise>
  $y := 2$
  = Notes
  #expect("y", 2)
  = Task <exercise>
  #undefined("y")
  $"yt" := 3$
  #block[Part] <exercise>
  #undefined("yt")
]
#worksheet(clear-at: selector(heading))[
  = A
  $q := 1$
  = B
  #undefined("q")
]

// what was replaced or isn't a scope
#worksheet(inherit: true)[]
#reported("inherit was replaced by scope: \"isolated\" | \"inherit\" | \"shared\"")
#calculate($e_x := 1$, export: false)
#reported("export was replaced by scope")
#undefined("e_x")
#worksheet(scope: "local")[]
#reported("scope: auto, \"isolated\", \"inherit\" or \"shared\", got \"local\"")
#worksheet(scope: true)[]
#reported("got true")

// units change what names mean: only an isolated scope can set them
