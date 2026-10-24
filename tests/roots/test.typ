#import "/src/unit-words.typ" as units
#import "/src/constants.typ" as constants
#import "/src/builtins.typ": *
#import "/lib.typ": *
#import "/tests/util.typ": expect, passes
#set page(width: 18cm, height: auto)
#show: worksheet
$f(x) := sqrt(2)$
#passes()
$ a := f(3) = $
#expect("a", calc.sqrt(2))
$f(x) := sqrt(x)$
#passes()
$ b := f(9) = $
#expect("b", 3)
$f(x) := sqrt(x^2) ->$
#passes()
$ c := f(-3) = $
#expect("c", 3)
$g(x) := root(3, x)$
#passes()
$ d := g(8) = $
#expect("d", 2)
$h(x) := sqrt(1 + sqrt(x))$
#passes()
$ k := h(9) = $
#expect("k", 2)
#worksheet($f(x) := sqrt(x^2) ->$)
#passes()
#worksheet($m := f(-4) =$)
#expect("m", 4)
