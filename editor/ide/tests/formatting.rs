use solveng_ide::{Context, format};

fn formatted(doc: &str) -> String {
    let mut text: Vec<u16> = doc.encode_utf16().collect();
    for e in format(doc, &Context::default()).into_iter().rev() {
        text.splice(e.start as usize..e.end as usize, e.text.encode_utf16());
    }
    String::from_utf16(&text).unwrap()
}

fn check(before: &str, after: &str) {
    assert_eq!(formatted(before), after);
    assert_eq!(formatted(after), after, "formatting must be idempotent");
}

#[test]
fn ranges_and_function_parameters() {
    check("#import units: V\n#show: worksheet\n$x:=f( a :V,b )=1 . . 10$",
          "#import units: V\n#show: worksheet\n$x := f(a: V, b) = 1..10$");
}

#[test]
fn arithmetic_keeps_compact_and_repairs_asymmetric_spacing() {
    check("#show: worksheet\n$a:=b/c+d *e-f /g=$",
          "#show: worksheet\n$a := b/c+d * e-f / g =$");
    check("#show: worksheet\n$x:=- 2+3$ $x=-y$",
          "#show: worksheet\n$x := -2+3$ $x = -y$");
}

#[test]
fn math_delimiters_preserve_inline_and_display() {
    check("#show: worksheet\n$ x:=1$ $x:=2 $ $  x:=3  $ $x:=4$",
          "#show: worksheet\n$x := 1$ $x := 2$ $ x := 3 $ $x := 4$");
    check("#show: worksheet\n$\n  x:=1\\\n  y:=2\n$",
          "#show: worksheet\n$\n  x := 1\\\n  y := 2\n$");
}

#[test]
fn leaves_raw_text_and_inactive_math_alone() {
    // solveng code blocks are ordinary raw text now
    let doc = "#show: worksheet\nText `a:=b`\n```rust\nf( a,b )\n```\n```solveng\nx:=1\n```\n#worksheet(evaluate: false)[$ x:=2$]";
    check(doc, doc);
    check("Text $ x:=1$", "Text $ x:=1$");
}

#[test]
fn preserves_comments_strings_embedded_code_and_line_structure() {
    check("😀\n#show: worksheet\n$\n  x:=1 // a:=b /c\n  /* f( a,b ) */ y:=2\n$",
          "😀\n#show: worksheet\n$\n  x := 1 // a:=b /c\n  /* f( a,b ) */ y := 2\n$");
    check("#show: worksheet\n$ x:=\"a:=b /c\" +#foo( a:1,b:2 ) $",
          "#show: worksheet\n$ x := \"a:=b /c\" + #foo( a:1,b:2 ) $");
    check("#show: worksheet\r\n$x:=f( a,b, )$\r\n",
          "#show: worksheet\r\n$x := f(a, b,)$\r\n");
}

#[test]
fn context_enables_included_math() {
    let doc = "$ x:=1$";
    assert!(format(doc, &Context::default()).is_empty());
    let edits = format(doc, &Context { evaluate: Some(true), ..Context::default() });
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].text, "x := 1");
}

#[test]
fn joins_numbers_only_to_unit_bindings() {
    // a unit binding joins its number; a `#` reference, a variable and inactive math stay as written
    check("#import units: mA, s\n#show: worksheet\n$ t:=-2 #s + 3 mA + 4 x$\n#worksheet(evaluate: false)[$-2 mA$]",
          "#import units: mA, s\n#show: worksheet\n$t := -2 #s + 3mA + 4 x$\n#worksheet(evaluate: false)[$-2 mA$]");
    // numbers are written with a point; a comma always separates (1,2 is 1 and 2)
    check("#import units: ms\n#show: worksheet\n$x:=0.95 ms$ $f(1,2)=$",
          "#import units: ms\n#show: worksheet\n$x := 0.95ms$ $f(1, 2) =$");
}

#[test]
fn leaves_strings_of_calls_alone() {
    check(r#"#worksheet("x:=f( a :V,b ) + 2 s")"#,
          r#"#worksheet("x:=f( a :V,b ) + 2 s")"#);
    check(r#"#let prose => "x:=f( a,b )""#, r#"#let prose => "x:=f( a,b )""#);
}
