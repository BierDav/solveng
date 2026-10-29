#import "/lib.typ": *
#import "/tests/util.typ": expect
#show: worksheet.with(errors: "panic")
$ a &:= 2 $
$ b := &a + 1 $
#expect("a", 2)
#expect("b", 3)
