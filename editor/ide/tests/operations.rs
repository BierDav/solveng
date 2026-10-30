// Operations in the editor: named arguments, indices, convolve and range, and math functions.

use solveng_ide::*;

const DOC: &str = "#import \"@preview/solveng:1.0.0\": *\n#import operations: *\n#show: math\n$N := 200$, $i := 0..N$\n$x[i] := tri(i/N)$\n$y := convolve(x, x, step: 2)$\n$f(x, n: 2) := x^n$, $a := f(3, n: 3) =$\n$q := range(1, 10, step: 3)$\n$z := conv(x, x)$\n$w := x[2] + y[i]$\n$r := rect(0)$\n";

fn kind_of(doc: &str, word: &str, nth: usize) -> u8 {
    let at = doc.match_indices(word).nth(nth).unwrap().0;
    let start = doc[..at].encode_utf16().count() as u32;
    document_tokens(doc, &Context::default()).into_iter().find(|t| t.start == start).map_or(255, |t| t.kind)
}

fn hover_at(doc: &str, word: &str) -> String {
    let at = doc.find(word).unwrap() + 1;
    hover(doc, &Context::default(), doc[..at].encode_utf16().count() as u32).map(|h| h.markdown).unwrap_or_default()
}

#[test]
fn named_arguments_are_parameters() {
    assert_eq!(kind_of(DOC, "step", 0), TokenKind::PARAMETER);
    assert_eq!(kind_of(DOC, "step", 1), TokenKind::PARAMETER);
    // a call's named argument, and a function head's named parameter
    assert_eq!(kind_of(DOC, "n: 3", 0), TokenKind::PARAMETER);
    assert_eq!(kind_of(DOC, "n: 2", 0), TokenKind::PARAMETER);
    // a type stays a unit: f(I: mA), f(t: #s)
    let typed = "#import units: mA, s\n#show: math\n$q(I: mA) := I$\n$p(t: #s) := t$\n";
    assert_eq!(kind_of(typed, "mA", 1), TokenKind::UNIT);
    assert_eq!(kind_of(typed, "s) :=", 0), TokenKind::UNIT);
}

#[test]
fn indices_are_no_units() {
    assert_ne!(kind_of(DOC, "i]", 1), TokenKind::UNIT);
    // a unit binding after a number is the unit, also inside brackets; a bare single letter is a symbol
    let unit = "#import units: V\n#show: math\n$U := R [5 #V]$\n$W := R [5 V]$\n";
    assert_eq!(kind_of(unit, "V]$\n$W", 0), TokenKind::UNIT);
    assert_eq!(kind_of(unit, "V]$\n", 1), TokenKind::VARIABLE);
}

#[test]
fn operations_and_user_calls() {
    assert_eq!(kind_of(DOC, "convolve(x", 0), TokenKind::BUILTIN_FUNCTION);
    assert_eq!(kind_of(DOC, "range", 0), TokenKind::BUILTIN_FUNCTION);
    assert_eq!(kind_of(DOC, "conv(", 0), TokenKind::FUNCTION_CALL);
    // The names `#import operations: *` binds.
    assert_eq!(kind_of(DOC, "rect(0)", 0), TokenKind::BUILTIN_FUNCTION);
    assert!(hover_at(DOC, "convolve(x").contains("Convolution"));
    assert!(hover_at(DOC, "range(1").contains("range"));
    // no problems reported for the new syntax
    assert!(diagnostics(DOC, &Context::default()).is_empty(), "{:?}", diagnostics(DOC, &Context::default()));
    // Without the import solveng's operations are no names math knows: a user call.
    let bare = DOC.replace("#import operations: *\n", "");
    assert_eq!(kind_of(&bare, "rect(0)", 0), TokenKind::FUNCTION_CALL);
}
