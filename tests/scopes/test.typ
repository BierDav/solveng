#import "/src/unit-words.typ" as units
#import "/src/constants.typ" as constants
#import "/src/builtins.typ": *
#import "/lib.typ": *
#import "/tests/util.typ": expect, undefined, passes, fails, reported
#import modifiers: solve
#set page(width: 18cm, height: auto)
#show: math

// Settings keep definitions; explicit scopes contain them.
$x := 10$
#math($a := x+1$)
#expect("a", 11)
#mathscope[#undefined("x") $x := 99$ $"local" := 1$]
#expect("x", 10) #undefined("local")
#mathscope(scope: "isolated")[#undefined("x")]

// inherit: sees the outer definitions, its own end with it
#mathscope(scope: "inherit")[$x := x+1$ #expect("x", 11) $"own" := 1$]
#expect("x", 10) #undefined("own")
#mathscope(scope: "inherit")[$x := 40$ $"temporary" := x+1$] #expect("x", 10) #undefined("temporary")
// shared: sees them, and what it defines stays, over the outer ones
#mathscope(scope: "shared")[$x := x+2$ $b := a+1$]
#expect("x", 12) #expect("b", 12)
#mathscope(scope: "shared")[$y_s := x + 1$] #expect("y_s", 13)
// isolated: starts empty, its definitions end with it
#mathscope(scope: "isolated")[#undefined("x") $x := 30$ #expect("x", 30)]
#expect("x", 12)
#mathscope(scope: "isolated")[$x := 50$ $"iso" := 2$] #expect("x", 12) #undefined("iso")
#mathscope(scope: "isolated")[$z_i := x =>$] #fails("'x' in 'z_i' isn't defined")

// shared keeps functions, solved values, code and graph definitions as well as variables
#mathscope(scope: "shared")[#math(hidden: true)[
  $U := 12 #units.V$
  $R := 100 #units.Ω$
  $U = I R -> solve(I)$
  $f(t) := t^2+1$
  $"coded" := 8$
  #graph($g(t) := t+2$, x-range: (0, 1))
]]
#expect("I", 0.12) #expect("coded", 8)
$y := f(3) =>$ #expect("y", 10)
$z := g(3) =>$ #expect("z", 5)

// shared reaches only the immediate parent; nested worksheets have their own defaults
#mathscope[
  #mathscope(scope: "shared")[$"nested" := 2$]
  #expect("nested", 2)
]
#undefined("nested")
#mathscope(scope: "shared")[
  #mathscope[Local calculation: $"private" := 3$]
  #undefined("private")
  #mathscope(scope: "shared")[$"nested" := 4$]
]
#expect("nested", 4) #undefined("private")

// clearing goes back to the worksheet's start, the outer definitions included for inherit and shared; the parent
// keeps what was cleared
#mathscope(scope: "shared")[
  #clear("x")
  #undefined("x")
  $"retained" := 6$
]
#expect("x", 12) #expect("retained", 6)
#mathscope(scope: "shared")[#math(clear-at: heading)[
  = First
  $"discarded" := 1$
  = Second
  #expect("x", 12) #undefined("discarded")
  $"kept" := 2$
]]
#undefined("discarded") #expect("kept", 2)
#mathscope(scope: "inherit")[$"discarded" := 3$ #clear(all: true) #expect("x", 12) #undefined("discarded")]
#mathscope(scope: "isolated")[$"discarded" := 3$ #clear(all: true) #undefined("x") #undefined("discarded")]

// a shared definition can change a name from a variable to a function and back
#mathscope(scope: "shared")[$x(t) := 2t$]
$"function_value" := x(3) =>$ #expect("function_value", 6)
#mathscope(scope: "shared")[$x := 60$]
#expect("x", 60)

// A single equation stays local in a scope, and stays shared in math.
#math[$"collapsed" := 1$]
#expect("collapsed", 1)
#mathscope(scope: "isolated")[$"collapsed" := 2$]
#expect("collapsed", 1)

// clear-at: every heading, headings of one level, or every element with a label
#mathscope[#math(clear-at: heading.where(level: 1))[
  = One
  $p := 1$
  == Two
  #expect("p", 1)
  = Three
  #undefined("p")
]]
#mathscope[#math(clear-at: <exercise>)[
  = Task <exercise>
  $y := 2$
  = Notes
  #expect("y", 2)
  = Task <exercise>
  #undefined("y")
  $"yt" := 3$
  #block[Part] <exercise>
  #undefined("yt")
]]
#mathscope[#math(clear-at: selector(heading))[
  = A
  $q := 1$
  = B
  #undefined("q")
]]

// what was replaced or isn't a scope
#mathscope[#math(inherit: true)[]]
#reported("unknown argument 'inherit' for math")
#math($e_x := 1$, export: false)
#reported("unknown argument 'export' for math")
#undefined("e_x")
#mathscope(scope: "local")[]
#reported("scope: \"isolated\", \"inherit\" or \"shared\", got \"local\"")
#mathscope(scope: true)[]
#reported("got true")

// units change what names mean: only an isolated scope can set them
