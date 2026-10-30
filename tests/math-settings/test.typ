#import "/lib.typ" as package
#assert(not ("worksheet" in dictionary(package)))
#import "/lib.typ": *
#import "/tests/util.typ": expect, undefined
#import "/src/api.typ": _cf
#set page(height: auto)
// Importing does not process ordinary math.
$"not_active" := 1$
#undefined("not_active")
#show: math.with(errors: "panic", digits: 4, clear-at: heading.where(level: 1))
$x := 12$
// Settings do not introduce variable scopes, even for single equations.
#math(digits: 8)[$y := x/7 =>$]
#expect("y", 12/7)
#context assert.eq(_cf.get().digits, 4)
#math(evaluate: false)[$x := 99$]
#expect("x", 12)
#math(evaluate: false)[
  $x := 98$
  #math(digits: 8)[$x := 97$]
  #mathscope(scope: "inherit")[$x := 96$]
  #expect("x", 12)
  #math(evaluate: true)[$z := x+1$]
]
#expect("x", 12) #expect("z", 13)
// Nesting settings doesn't duplicate evaluation.
#math[$x := x+1$]
#expect("x", 13)
#math[#math[$x := x+1$]]
#expect("x", 14)
// Scope semantics are independent of content shape.
#mathscope[$x := 99$]
#expect("x", 14)
#mathscope(scope: "inherit")[$x := x+1$ #expect("x", 15)]
#expect("x", 14)
#mathscope(scope: "shared")[$x := x+1$]
#expect("x", 15)
#mathscope(scope: "isolated")[#undefined("x") $q := 1$]
#undefined("q")
// Repeated show rules configure the same document scope.
#show: math.with(digits: 6)
#expect("x", 15)
#context assert.eq(_cf.get().digits, 6)
// An inner selector overrides the outer one, including none.
#math(clear-at: none)[
  = No reset
  #expect("x", 15)
]
#expect("x", 15)
#math(clear-at: heading.where(level: 2))[
  = No reset either
  #expect("x", 15)
  == Reset global
  #undefined("x")
  $x := 20$
]
#expect("x", 20)
#mathscope(scope: "inherit")[
  $x := 21$
  = Reset local only
  #expect("x", 20)
  $x := 22$
]
#expect("x", 20)
= Reset global again
#undefined("x")
#math(clear-at: <exercise>)[
  $q := 1$
  #block[Reset] <exercise>
  #undefined("q")
]
// Native math stays accessible when math is imported.
#std.math.equation($1+1$.body)
// Templates and documents can both install math without extra variable scopes.
#let template(body) = {
  show: math.with(clear-at: heading.where(level: 1))
  math(hidden: true)[$sigma(t) := t$]
  body
}
#mathscope[
  #show: template
  #show: math.with(clear-at: heading.where(level: 2))
  = Exercise
  $b := 2$
  #math(hidden: true)[
    $b := b+1$
    #math(hidden: false)[$q := b+1$]
  ]
  #expect("b", 3) #expect("q", 4)
  ==== Shift
  #math(evaluate: false)[
    $s((t-t_0)/T) = g(t-t_0)$
  ]
  #expect("b", 3)
  == Reset
  #undefined("b") #undefined("q")
]
// Reapplying the same selector resets once, and labels remain usable.
#math(clear-at: <reset>)[
  #math(clear-at: <reset>)[
    $x := 1$
    = Reset <reset>
    #undefined("x")
    #context assert.eq(query(<reset>).len(), 1)
  ]
]
// Labels keep identifying their equations when processing is disabled.
#math(evaluate: false)[$x := 100$ <disabled-equation>]
#context assert.eq(query(<disabled-equation>).len(), 1)
