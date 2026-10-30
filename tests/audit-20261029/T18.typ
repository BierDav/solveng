#import "/lib.typ": *
#import "/tests/util.typ": passes, fails, result, reported, undefined
#set page(height: auto)
#show: math
$x := 1$
#mathscope(scope: "shared")[#clear("x")]
#undefined("x")
