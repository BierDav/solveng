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
    check("```solveng\nx:=1 .. 10\nf( a :V,b ) := a+b\n```",
          "```solveng\nx := 1..10\nf(a: V, b) := a+b\n```");
    check("#show: worksheet\n$x:=f( a :V,b )=1 . . 10$",
          "#show: worksheet\n$x := f(a: V, b) = 1..10$");
}

#[test]
fn arithmetic_keeps_compact_and_repairs_asymmetric_spacing() {
    check("```solveng\na/b +c- d*e / f\nx:=- 2+3e-4\nx=-y\n```",
          "```solveng\na/b + c - d*e / f\nx := -2+3e-4\nx = -y\n```");
    check("#show: worksheet\n$a:=b/c+d *e-f /g=$",
          "#show: worksheet\n$a := b/c+d * e-f / g =$");
}

#[test]
fn math_delimiters_preserve_inline_and_display() {
    check("#show: worksheet\n$ x:=1$ $x:=2 $ $  x:=3  $ $x:=4$",
          "#show: worksheet\n$x := 1$ $x := 2$ $ x := 3 $ $x := 4$");
    check("#show: worksheet\n$\n  x:=1\\\n  y:=2\n$",
          "#show: worksheet\n$\n  x := 1\\\n  y := 2\n$");
}

#[test]
fn leaves_other_languages_and_inactive_math_alone() {
    let doc = "Text $ x:=1$ `a:=b`\n```rust\nf( a,b )\n```\n#worksheet(evaluate: false)[$ x:=2$]";
    check(doc, doc);
}

#[test]
fn preserves_comments_strings_embedded_code_and_line_structure() {
    check("😀\n```solveng\n  x:=1 // a:=b /c\n  /* f( a,b ) */ y:=2\n  f(\n    a :V,\n    b\n  )=\n```",
          "😀\n```solveng\n  x := 1 // a:=b /c\n  /* f( a,b ) */ y := 2\n  f(\n    a: V,\n    b\n  ) =\n```");
    check("#show: worksheet\n$ x:=\"a:=b /c\" +#foo( a:1,b:2 ) $",
          "#show: worksheet\n$ x := \"a:=b /c\" + #foo( a:1,b:2 ) $");
    check("```solveng\r\nx:=1\r\nf( a,b, )\r\n```",
          "```solveng\r\nx := 1\r\nf(a, b,)\r\n```");
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
fn joins_only_numbers_and_recognized_units_in_injected_regions() {
    check("#show: worksheet\n$ t:=-2 unit(\"s\") + 3 unit(\"mA\") + 4 x$\n#worksheet(evaluate: false)[$-2 unit(\"s\")$]",
          "#show: worksheet\n$t := -2 unit(\"s\") + 3 unit(\"mA\") + 4 x$\n#worksheet(evaluate: false)[$-2 unit(\"s\")$]");
    check("```solveng\nx:=2 s + 3 mA + 4 unknown\n```",
          "```solveng\nx := 2s + 3mA + 4 unknown\n```");
    check("#show: worksheet\n$x:=2 s + 3 unit(\"ms\")$",
          "#show: worksheet\n$x := 2 s + 3 unit(\"ms\")$");
    check("```solveng\nx:=0,95 s\nf(1,2)=\n```",
          "```solveng\nx := 0,95s\nf(1,2) =\n```");
}

#[test]
fn formats_injected_call_strings_and_preserves_escaping() {
    check(r#"#worksheet("x:=f( a :V,b ) + 2 s")"#,
          r#"#worksheet("x:=f( a :V,b ) + 2 s")"#);
    check(r#"#worksheet("x:=-2 \"s\" // a:=b")"#,
          r#"#worksheet("x:=-2 \"s\" // a:=b")"#);
    check(r#"#let prose = "x:=f( a,b )""#, r#"#let prose = "x:=f( a,b )""#);
}
