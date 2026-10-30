#import "/lib.typ": *
#import "/tests/util.typ": passes, fails, result, reported, undefined
#set page(height: auto)
#show: math
$f(x) := x+1$
#mathscope(scope: "shared")[#clear("f")]
#undefined("f")
