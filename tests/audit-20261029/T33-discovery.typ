#import "/lib.typ": *
#import "/tests/util.typ": expect
#show: worksheet.with(errors: "panic")
$ x &:= 1 $
#expect("x", 1)
