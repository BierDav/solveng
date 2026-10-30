#import "/lib.typ": *
#import "/tests/util.typ": expect
#show: math.with(errors: "panic")
$ x &:= 1 $
#expect("x", 1)
