//! Highlighting, regions, problems and hovers over a whole document.

use crate::calls::Role;
use crate::complete::{FUNCTIONS, defined};
use crate::text::{Doc, chars, is_letter_or_digit};
use crate::tokens::{CONSTANTS, Tok, code_tokens, math_tokens as math_toks};
use crate::units::{self, Custom};
use crate::{Analysis, Context, Hover, Problem, Region, Token, TokenKind as K};

fn utf16(doc: &Doc, base: usize, toks: Vec<Tok>) -> impl Iterator<Item = Token> + '_ {
    toks.into_iter().map(move |(s, l, kind)| {
        let start = doc.to16(base + s);
        Token { start, length: doc.to16(base + s + l) - start, kind }
    })
}

fn local(text: &str, toks: Vec<Tok>) -> Vec<Token> {
    let doc = Doc::new(text);
    utf16(&doc, 0, toks).collect()
}

/// Tokens of solveng code with kinds from [`K`]: numbers and their units, variables, functions, `@`-inlined
/// formulas, relations and `->`, comments.
pub fn tokens(code: &str) -> Vec<Token> {
    local(code, code_tokens(&chars(code), &|_| false))
}

/// [`tokens`], with the document's own unit names (`bar`) read as units after a number.
pub fn tokens_with_units(code: &str, units: &[String]) -> Vec<Token> {
    local(code, code_tokens(&chars(code), &|w| units.iter().any(|u| u == w)))
}

/// Tokens of one equation's body.
pub fn math_tokens_of(math: &str) -> Vec<Token> {
    local(math, math_toks(&chars(math), &|_| false))
}

/// Tokens of the equations solveng reads (with [`crate::Integration::MATH`], or passed to `def` and the like),
/// over the whole document, coloured like solveng code: numbers, quoted units, variables, functions.
pub fn math_tokens(document: &str, context: &Context) -> Vec<Token> {
    let a = Analysis::new(document, context);
    let unit = |w: &str| a.is_custom_unit(w);
    let mut out = Vec::new();
    for e in a.elements.iter().filter(|e| e.math && a.active(e)) {
        out.extend(utf16(&a.doc, e.body_start, math_toks(&a.doc.c[e.body_start..e.body_end], &unit)));
    }
    out
}

/// A unit string's tokens: every word is a unit, as after `->`.
fn unit_tokens(code: &[char], custom: &[Custom]) -> Vec<Tok> {
    let mut with: Vec<char> = "-> ".chars().collect();
    with.extend_from_slice(code);
    code_tokens(&with, &|w| custom.iter().any(|u| u.name == w))
        .into_iter()
        .filter(|t| t.0 >= 3)
        .map(|(s, l, k)| (s - 3, l, if k == K::NUMBER { K::NUMBER } else { K::UNIT }))
        .collect()
}

/// Tokens of what the strings of solveng's calls hold: equations in `#def("…")`, `#solve`, `#graph` and
/// `#lambdify`, units in `unit:`, names in `x:`, the values of options like `steps:`.
fn call_tokens(a: &Analysis) -> Vec<Token> {
    let unit = |w: &str| a.is_custom_unit(w);
    let mut out = Vec::new();
    for call in &a.calls {
        for t in &call.texts {
            let code = &a.doc.c[t.start..t.end];
            let text: String = code.iter().collect();
            let trimmed = text.trim();
            let whole = |kind: u8| {
                let lead = text.chars().take_while(|c| c.is_whitespace()).count();
                vec![(lead, trimmed.chars().count(), kind)]
            };
            let toks = match t.role {
                Role::Code => code_tokens(code, &unit),
                Role::Unit => unit_tokens(code, &a.units),
                Role::Name if !trimmed.is_empty() => whole(K::VARIABLE),
                Role::Choice => {
                    let param = call.function.named.iter().find(|p| Some(&p.name) == t.param.as_ref());
                    if param.is_some_and(|p| p.values.iter().any(|v| v == trimmed)) { whole(K::KEYWORD) } else { Vec::new() }
                }
                Role::Mark if ["crossings", "roots", "extrema"].contains(&trimmed) => whole(K::KEYWORD),
                Role::Mark => code_tokens(code, &unit),
                _ => Vec::new(),
            };
            out.extend(utf16(&a.doc, t.start, toks.into_iter().filter(|t| t.1 > 0).collect()));
        }
    }
    out
}

/// Every solveng token of a document, sorted: calculated raw code, calculated math and the strings of solveng's
/// calls. Editors without their own raw-block injection can take just this.
pub fn document_tokens(document: &str, context: &Context) -> Vec<Token> {
    let a = Analysis::new(document, context);
    let unit = |w: &str| a.is_custom_unit(w);
    let mut out = Vec::new();
    for e in a.elements.iter().filter(|e| a.shown(e)) {
        let body = &a.doc.c[e.body_start..e.body_end];
        let toks = if e.math { math_toks(body, &unit) } else { code_tokens(body, &unit) };
        out.extend(utf16(&a.doc, e.body_start, toks));
    }
    out.extend(call_tokens(&a));
    out.sort_by_key(|t| t.start);
    out
}

/// Where solveng reads the document: calculated code and math, the strings of its calls, and the calls'
/// arguments (`kind: "call"`), where its options complete. A language server next to tinymist answers there only.
pub fn regions(document: &str, context: &Context) -> Vec<Region> {
    let a = Analysis::new(document, context);
    let r = |s: usize, e: usize, kind| Region { start: a.doc.to16(s), end: a.doc.to16(e), kind };
    let mut out: Vec<Region> = Vec::new();
    for e in a.elements.iter().filter(|e| a.shown(e)) {
        out.push(r(e.body_start, e.body_end, if e.math { "math" } else { "code" }));
    }
    for call in &a.calls {
        out.push(r(call.args_start, call.args_end, "call"));
        out.extend(call.texts.iter().map(|t| r(t.start, t.end, "string")));
    }
    out.sort_by_key(|r| r.start);
    out
}

/// Units neither solveng nor the document knows, in `-> unit`, `[5 V]`, `unit: "…"` and quoted math units.
pub fn diagnostics(document: &str, context: &Context) -> Vec<Problem> {
    let a = Analysis::new(document, context);
    let mut out = Vec::new();
    let mut report = |base: usize, expr: &str| {
        for (s, e, word) in units::unknown_words(expr, &a.units) {
            out.push(Problem {
                start: a.doc.to16(base + s),
                end: a.doc.to16(base + e),
                message: format!("unknown unit '{word}' – define it with solve-config(units: ({word}: \"…\"))"),
                severity: "warning",
            });
        }
    };
    let code = |start: usize, end: usize, report: &mut dyn FnMut(usize, &str)| {
        // Units after `->` and in brackets, line by line, comments left out.
        let c = &a.doc.c;
        let mut i = start;
        while i < end {
            let line_end = (i..end).find(|&j| c[j] == '\n').unwrap_or(end);
            let line: String = c[i..line_end].iter().collect();
            let line = crate::text::before(&line, "//");
            for (k, part) in line.split(';').scan(0usize, |at, p| {
                let s = *at;
                *at += p.chars().count() + 1;
                Some((s, p))
            }) {
                if let Some(arrow) = part.find("->") {
                    let after = &part[arrow + 2..];
                    report(i + k + part[..arrow + 2].chars().count(), after);
                }
                let pc: Vec<char> = part.chars().collect();
                let mut j = 0;
                while j < pc.len() {
                    if pc[j] == '[' {
                        let close = (j..pc.len()).find(|&x| pc[x] == ']').unwrap_or(pc.len());
                        let inner: String = pc[j + 1..close].iter().collect();
                        // The value's number comes first: [5 V].
                        let skip = inner.chars().take_while(|ch| !ch.is_alphabetic() && !"°%‰µΩ".contains(*ch)).count();
                        if skip > 0 {
                            report(i + k + j + 1 + skip, &inner.chars().skip(skip).collect::<String>());
                        }
                        j = close;
                    }
                    j += 1;
                }
            }

            i = line_end + 1;
        }
    };
    for e in a.elements.iter().filter(|e| a.active(e)) {
        if e.math {
            // A quoted unit after a number: 30 "mA"; "k"Omega joins the prefix to what follows.
            let c = &a.doc.c;
            let mut i = e.body_start;
            while i < e.body_end {
                if c[i] == '"' {
                    let close = (i + 1..e.body_end).find(|&j| c[j] == '"').unwrap_or(e.body_end);
                    let before = c[e.body_start..i].iter().rev().find(|ch| !ch.is_whitespace());
                    let next = c.get(close + 1).copied().unwrap_or(' ');
                    if before.is_some_and(|b| b.is_ascii_digit()) && !next.is_alphabetic() {
                        let text: String = c[i + 1..close].iter().collect();
                        report(i + 1, &text);
                    }
                    i = close + 1;
                } else {
                    i += 1;
                }
            }
        } else {
            code(e.body_start, e.body_end, &mut report);
        }
    }
    for call in &a.calls {
        for t in &call.texts {
            match t.role {
                Role::Unit => report(t.start, &a.doc.sub(t.start, t.end)),
                Role::Code => code(t.start, t.end, &mut report),
                _ => {}
            }
        }
    }
    out
}

/// What the word at `offset` is: a defined name with its value, a unit, solveng's functions and constants, or a
/// parameter of solveng's functions with its docs.
pub fn hover(document: &str, context: &Context, offset: u32) -> Option<Hover> {
    let a = Analysis::new(document, context);
    let at = a.doc.from16(offset as usize);
    let c = &a.doc.c;
    let part = |ch: char| is_letter_or_digit(ch) || "_°%‰µΩ".contains(ch);
    let mut s = at;
    while s > 0 && part(c[s - 1]) {
        s -= 1;
    }
    let mut e = at;
    while e < c.len() && part(c[e]) {
        e += 1;
    }
    if s == e {
        return None;
    }
    let word = a.doc.sub(s, e);
    let hover = |markdown: String| Some(Hover { start: a.doc.to16(s), end: a.doc.to16(e), markdown });
    // Parameter names of solveng's calls.
    if let Some(call) = a.calls.iter().rev().find(|c| c.args_start <= at && at <= c.args_end)
        && let Some(n) = call.named.iter().find(|n| n.name_start <= at && at <= n.name_end) {
            let p = call.function.named.iter().find(|p| p.name == n.name)?;
            let default = if p.default.is_empty() { String::new() } else { format!(" = {}", p.default) };
            return hover(format!("`{}` ({}){default}\n\n{}", p.name, p.types, p.doc));
        }
    let inside = a.elements.iter().any(|el| el.body_start <= s && e <= el.body_end && a.shown(el))
        || a.calls.iter().any(|call| call.texts.iter().any(|t| t.start <= s && e <= t.end));
    if !inside {
        // The functions themselves, anywhere.
        let f = crate::params::function(&word)?;
        if s == 0 || c[s - 1] != '#' && !a.calls.iter().any(|call| call.start == s) {
            return None;
        }
        return hover(format!("`{}` (solveng)\n\n{}", f.name, f.doc));
    }
    if let Some(d) = defined(&a, &word) {
        return hover(d);
    }
    if FUNCTIONS.contains(&word.as_str()) {
        return hover(format!("`{word}()` built-in function"));
    }
    if let Some((_, d)) = CONSTANTS.iter().find(|(n, _)| *n == word) {
        return hover(format!("`{word}` {d}"));
    }
    units::describe(&word, &a.units).and_then(hover)
}
