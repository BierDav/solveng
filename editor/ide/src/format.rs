//! Whitespace formatting of calculation bodies, leaving surrounding Typst alone.

use crate::{Analysis, Context, Edit, TokenKind};
use crate::text::{starts_with, sub};

/// Whitespace edits in calculated math and call strings, in ascending UTF-16 order.
pub fn format(document: &str, context: &Context) -> Vec<Edit> {
    let a = Analysis::new(document, context);
    let highlighted = crate::document_tokens(document, context);
    let joins: Vec<(usize, usize)> = highlighted.windows(2).filter(|&t| t[0].kind == TokenKind::NUMBER && t[1].kind == TokenKind::UNIT).map(|t| (a.doc.from16((t[0].start + t[0].length) as usize), a.doc.from16(t[1].start as usize))).collect();
    let mut out = Vec::new();
    for e in a.elements.iter().filter(|e| a.active(e)) {
        let body = &a.doc.c[e.body_start..e.body_end];
        let local_joins: Vec<(usize, usize)> = joins.iter().filter(|&&(s, end)| s >= e.body_start && end < e.body_end)
            .map(|&(s, end)| (s - e.body_start, end - e.body_start)).collect();
        let mut text = format_body(body, &local_joins, None);
        if a.doc.c.get(e.body_end) == Some(&'$') {
            let display = body.first().is_some_and(|c| c.is_whitespace())
                && body.last().is_some_and(|c| c.is_whitespace());
            if !text.contains(['\n', '\r']) {
                text = if display { format!(" {} ", text.trim()) } else { text.trim().into() };
            } else if !display {
                // Preserve line breaks while repairing mismatched inline delimiters.
                text = text.trim_matches([' ', '\t']).into();
            }
        }
        if text != a.doc.sub(e.body_start, e.body_end) {
            out.push(Edit { path: None, start: a.doc.to16(e.body_start), end: a.doc.to16(e.body_end), text });
        }
    }
    for t in a.calls.iter().flat_map(|call| &call.texts) {
        let raw = &a.doc.c[t.start..t.end];
        let (decoded, offsets) = crate::text::unescape(raw);
        let local_joins: Vec<(usize, usize)> = joins.iter().filter(|&&(s, end)| s >= t.start && end < t.end)
            .map(|&(s, end)| (offsets.partition_point(|&i| i < s - t.start), offsets.partition_point(|&i| i < end - t.start))).collect();
        let text = format_body(&decoded, &local_joins, Some((raw, &offsets)));
        if text != a.doc.sub(t.start, t.end) {
            out.push(Edit { path: None, start: a.doc.to16(t.start), end: a.doc.to16(t.end), text });
        }
    }
    out.sort_by_key(|e| e.start);
    out
}

#[derive(Debug)]
struct Token {
    start: usize,
    end: usize,
    text: String,
    opaque: bool,
}

fn horizontal(c: char) -> bool { matches!(c, ' ' | '\t') }

fn lex(c: &[char]) -> Vec<Token> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < c.len() {
        if c[i].is_whitespace() { i += 1; continue; }
        let start = i;
        let mut opaque = false;
        if c[i] == '"' {
            i = quoted_end(c, i);
            opaque = true;
        } else if starts_with(c, i, "//") {
            while i < c.len() && !matches!(c[i], '\n' | '\r') { i += 1; }
            opaque = true;
        } else if starts_with(c, i, "/*") {
            let mut depth = 1;
            i += 2;
            while i < c.len() && depth > 0 {
                if starts_with(c, i, "/*") { depth += 1; i += 2; }
                else if starts_with(c, i, "*/") { depth -= 1; i += 2; }
                else { i += 1; }
            }
            opaque = true;
        } else if c[i] == '#' {
            // Embedded Typst code has different whitespace rules.
            i += 1;
            while i < c.len() && (c[i].is_alphanumeric() || "_-.".contains(c[i])) { i += 1; }
            while i < c.len() && "([{".contains(c[i]) {
                let mut stack = vec![c[i]];
                i += 1;
                while i < c.len() && !stack.is_empty() {
                    if c[i] == '"' { i = quoted_end(c, i); continue; }
                    if "([{".contains(c[i]) { stack.push(c[i]); }
                    else if ")]}".contains(c[i]) { stack.pop(); }
                    i += 1;
                }
            }
            opaque = true;
        } else if c[i] == '\\' {
            i = (i + 2).min(c.len());
            opaque = true;
        } else if c[i].is_ascii_digit() || (c[i] == '.' && c.get(i + 1).is_some_and(|c| c.is_ascii_digit())) {
            i += 1;
            while i < c.len() && c[i].is_ascii_digit() { i += 1; }
            if c.get(i).is_some_and(|c| matches!(c, '.' | ',')) && c.get(i + 1).is_some_and(|c| c.is_ascii_digit()) {
                i += 1;
                while i < c.len() && c[i].is_ascii_digit() { i += 1; }
            }
        } else if solveng_names::is_start(c[i]) {
            i = crate::tokens::name_end(c, i);
        } else if let Some(op) = [":=", "->", "<=", ">=", "!=", "==", "=>", "**"].iter().find(|op| starts_with(c, i, op)) {
            i += op.chars().count();
        } else if c[i] == '.' {
            let mut j = i + 1;
            while c.get(j).is_some_and(|c| horizontal(*c)) { j += 1; }
            i = if c.get(j) == Some(&'.') { j + 1 } else { i + 1 };
        } else {
            i += 1;
        }
        let text = sub(c, start, i);
        out.push(Token { start, end: i, text: if text.starts_with('.') && text.ends_with('.') && i > start + 1 { "..".into() } else { text }, opaque });
    }
    out
}

fn quoted_end(c: &[char], start: usize) -> usize {
    let mut i = start + 1;
    while i < c.len() {
        if c[i] == '\\' { i = (i + 2).min(c.len()); }
        else if c[i] == '"' { return i + 1; }
        else { i += 1; }
    }
    i
}

fn relation(s: &str) -> bool { matches!(s, ":=" | "≔" | "=" | "->" | "→" | "<=" | ">=" | "!=" | "==" | "<" | ">" | "≤" | "≥" | "≠" | "≈" | "=>") }
fn arithmetic(s: &str) -> bool { matches!(s, "+" | "-" | "−" | "*" | "/" | "**") }
fn range(s: &str) -> bool { matches!(s, ".." | "…") }
fn operand(t: &Token) -> bool {
    t.opaque && !t.text.starts_with(['/', '\\']) || t.text.ends_with([')', ']', '}'])
        || t.text.chars().next().is_some_and(|c| c.is_alphanumeric() || c == '_' || c == 'π')
}

fn format_body(c: &[char], joins: &[(usize, usize)], source: Option<(&[char], &[usize])>) -> String {
    let tokens = lex(c);
    let mut gaps: Vec<String> = tokens.windows(2).map(|t| sub(c, t[0].end, t[1].start)).collect();
    let set = |gaps: &mut Vec<String>, index: usize, value: &str| {
        if let Some(gap) = gaps.get_mut(index) && gap.chars().all(horizontal) { *gap = value.into(); }
    };
    for (i, t) in tokens.iter().enumerate() {
        if t.opaque { continue; }
        let s = t.text.as_str();
        if relation(s) || range(s) {
            let gap = if range(s) { "" } else { " " };
            if i > 0 { set(&mut gaps, i - 1, gap); }
            set(&mut gaps, i, gap);
        } else if arithmetic(s) {
            let binary = i > 0 && operand(&tokens[i - 1]);
            if binary {
                let spaced = c.get(t.start.wrapping_sub(1)).is_some_and(|c| horizontal(*c))
                    || c.get(t.end).is_some_and(|c| horizontal(*c));
                if spaced {
                    set(&mut gaps, i - 1, " ");
                    set(&mut gaps, i, " ");
                }
            } else if matches!(s, "+" | "-" | "−") {
                set(&mut gaps, i, "");
            }
        } else if matches!(s, ":" | "," | ";") {
            if i > 0 { set(&mut gaps, i - 1, ""); }
            set(&mut gaps, i, " ");
        }
    }
    // Delimiters beat comma spacing, as in f(a,) and empty calls.
    for (i, t) in tokens.iter().enumerate() {
        if t.text == "(" { set(&mut gaps, i, ""); }
        if t.text == ")" && i > 0 { set(&mut gaps, i - 1, ""); }
    }
    for (i, pair) in tokens.windows(2).enumerate() {
        if joins.contains(&(pair[0].end, pair[1].start)) && !pair[1].text.starts_with("\"\"") {
            set(&mut gaps, i, "");
        }
    }
    let Some(first) = tokens.first() else { return c.iter().collect() };
    let original = |start: usize, end: usize| match source {
        Some((raw, offsets)) => sub(raw, offsets[start], offsets[end]),
        None => sub(c, start, end),
    };
    let mut out = original(0, first.start);
    for (i, t) in tokens.iter().enumerate() {
        if t.text == sub(c, t.start, t.end) { out.push_str(&original(t.start, t.end)); }
        else { out.push_str(&t.text); }
        if let Some(gap) = gaps.get(i) { out.push_str(gap); }
    }
    out.push_str(&original(tokens.last().unwrap().end, c.len()));
    out
}
