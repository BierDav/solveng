//! What solveng adds to math shown as it renders while the caret is elsewhere: names in the strings of its calls
//! (`Delta_R` as `Δ_R`), explicit operation references as their name (`op("tri")` as `tri`), and the `#` before a
//! binding (`#s`). Typst's math itself editors conceal on their own.

use crate::calls::Role;
use crate::{Analysis, Conceal, Context};

pub(crate) use solveng_names::GREEK;
pub(crate) use solveng_names::greek;

fn greek_name(c: char) -> Option<&'static str> {
    GREEK.iter().find(|(_, g)| *g == c).map(|(n, _)| *n)
}

/// A name spelled out for typing, as math writes it: `Δ_R` → `Delta_R`, `Δr_R` → `Delta""r_R`.
pub(crate) fn spelled(name: &str) -> String {
    glued(name, |c| greek_name(c).map(String::from))
}

/// The name with each Greek letter of its base written by `letter`, and `""` between it and other letters.
fn glued(name: &str, letter: impl Fn(char) -> Option<String>) -> String {
    let (base, rest) = name.split_at(name.find('_').unwrap_or(name.len()));
    let mut out = String::new();
    let mut previous: Option<bool> = None;
    for c in base.chars() {
        let written = letter(c);
        let is_greek = written.is_some();
        if let Some(p) = previous
            && (p || is_greek)
            && c.is_alphabetic()
        {
            out.push_str("\"\"");
        }
        match written {
            Some(w) => out.push_str(&w),
            None => out.push(c),
        }
        previous = c.is_alphabetic().then_some(is_greek);
    }
    out + rest
}

/// The engine and every editor operation use the same identifier normalization.
pub fn resolved_name(name: &str) -> String {
    solveng_names::resolve(name).unwrap_or_else(|_| solveng_names::normalize(name))
}

/// Only explicit underscores make subscripts; digits inside names stay on the baseline.
fn piece_conceals(piece: &str, at: usize, out: &mut Vec<(usize, usize, char)>) {
    let c: Vec<char> = piece.chars().collect();
    let mut i = 0;
    while i < c.len() {
        if c[i].is_ascii_alphabetic() {
            let start = i;
            while i < c.len() && c[i].is_ascii_alphabetic() { i += 1; }
            if let Some(symbol) = greek(&c[start..i].iter().collect::<String>()) { out.push((at + start, i - start, symbol)); }
        } else { i += 1; }
    }
}

/// Conceals of a name in a string, in characters: spelled Greek letters as the letter, the quotes gluing names
/// (`Delta\"\"tau`, escaped in the string) hidden.
fn name_conceals(name: &[char]) -> Vec<(usize, usize, char)> {
    let mut out = Vec::new();
    for (i, &ch) in name.iter().enumerate() {
        if ch == '"' || (ch == '\\' && name.get(i + 1) == Some(&'"')) { out.push((i, 1, '\0')); }
    }
    piece_conceals(&name.iter().collect::<String>(), 0, &mut out);
    out.sort_by_key(|t| t.0);
    out
}

/// Conceals of the calculated math of a document and the strings of solveng's calls, each with the element it
/// belongs to, like Typewright's math conceals: shown while the caret is outside that element.
pub fn conceals(document: &str, context: &Context) -> Vec<Conceal> {
    let a = Analysis::new(document, context);
    let mut out = Vec::new();
    let mut push = |base: usize, found: Vec<(usize, usize, char)>, element: (usize, usize)| {
        for (s, l, c) in found {
            let start = a.doc.to16(base + s);
            out.push(Conceal {
                start,
                length: a.doc.to16(base + s + l) - start,
                character: c as u32,
                element_start: a.doc.to16(element.0),
                element_end: a.doc.to16(element.1),
            });
        }
    };
    // The names in the strings of solveng's calls, each shown as typed while the caret is in it (quotes included).
    for t in a.calls.iter().flat_map(|c| &c.texts).filter(|t| t.role == Role::Name) {
        push(t.start, name_conceals(&a.doc.c[t.start..t.end]), (t.start.saturating_sub(1), (t.end + 1).min(a.doc.c.len())));
    }
    // Explicit operation references show their name; their display follows syntax, so stale compile reports
    // cannot expand it while typing.
    for reference in a.bindings.inline_references() {
        if !a.elements.iter().any(|e| e.body_start <= reference.start && reference.end <= e.body_end) { continue; }
        let start = a.doc.to16(reference.start);
        let end = a.doc.to16(reference.end);
        for (from, to) in [(reference.start, reference.text_start), (reference.text_end, reference.end)] {
            out.push(Conceal { start: a.doc.to16(from), length: a.doc.to16(to) - a.doc.to16(from), character: 0, element_start: start, element_end: end });
        }
    }
    // The `#` before a unit, constant or operation binding in math.
    for (start, end, _) in a.bindings.solveng_uses() {
        if start == 0 || a.doc.c[start - 1] != '#' || !a.elements.iter().any(|e| e.body_start < start && end <= e.body_end) { continue; }
        out.push(Conceal { start: a.doc.to16(start - 1), length: 1, character: 0, element_start: a.doc.to16(start - 1), element_end: a.doc.to16(end) });
    }
    out.sort_by_key(|c| (c.start, c.length, c.character, c.element_start, c.element_end));
    out.dedup();
    out
}
