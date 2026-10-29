#import "/lib.typ": *
#import "/tests/util.typ": passes, fails, result, reported, undefined
#set page(height: auto)
#graph($x$, x-range: ([bad],1))
#reported("x-range")
