// Regression coverage for option forwarding and string definition filters.
#import "/lib.typ": *
#import "/tests/util.typ": passes, result
#show: math.with(errors: "panic")
$x := 1$
#math($x/3 =>$ , digits: 2)
#passes()
#result("\"0.33\"")
#defined(kinds: "given")
#defined(kinds: ("given", "computed"))
