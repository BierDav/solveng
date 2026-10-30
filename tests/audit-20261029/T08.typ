#import "/lib.typ": *
#import "/tests/util.typ": passes, fails, result, reported, undefined
#set page(height: auto)
#mathscope[#math(chips: (type: (fill: 1)))[ $f(x: any) := x$ ]]
#reported("chips")
