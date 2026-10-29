#import "/lib.typ": *
#import "/tests/util.typ": passes, fails, result, reported, undefined
#set page(height: auto)
#worksheet(words: (any: 1))[ $f(x: any) := x$ ]
#reported("words")
