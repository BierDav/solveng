#import "/lib.typ": *
#import "/tests/util.typ": passes, fails, result, reported, undefined
#set page(height: auto)
#show: worksheet
$f(x) := x+1$
#worksheet(scope: "shared")[#clear("f")]
#undefined("f")
