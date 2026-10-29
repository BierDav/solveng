#import "/lib.typ": *
#import "/tests/util.typ": passes, fails, result, reported, undefined
#set page(height: auto)
#show: worksheet
$x := 1$
#worksheet(scope: "shared")[#clear("x")]
#undefined("x")
