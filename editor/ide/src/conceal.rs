//! Solveng code shown as it renders while the caret is elsewhere, like math: `Delta_T` as `Δ_T`, `Q0` on the baseline,
//! `k""Omega` as `kΩ`, `->` as `→`, `m^2` as `m²`.

use crate::TokenKind as K;
use crate::text::{Doc, chars, sub};
use crate::tokens::code_tokens;
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

const SUPERSCRIPTS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];

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

/// Conceals of one piece of code in characters: start, length, the character shown instead.
fn conceals_in(code: &[char], unit: &dyn Fn(&str) -> bool) -> Vec<(usize, usize, char)> {
    let mut out = Vec::new();
    for (start, len, kind) in code_tokens(code, unit) {
        let word = sub(code, start, start + len);
        let digit = |w: &str| w.chars().next().and_then(|c| c.to_digit(10)).map(|d| d as usize);
        match kind {
            K::VARIABLE | K::FUNCTION | K::KEYWORD => {
                if word == "sqrt" {
                    out.push((start, 4, '√'));
                    continue;
                }
                let c: Vec<char> = word.chars().collect();
                for (i, &ch) in c.iter().enumerate() {
                    if ch == '"' || (ch == '\\' && c.get(i + 1) == Some(&'"')) { out.push((start + i, 1, '\0')); }
                }
                piece_conceals(&word, start, &mut out);
            }
            K::UNIT => {
                let exponent = start > 0 && code[start - 1] == '^';
                if exponent {
                    if word.chars().count() == 1
                        && let Some(d) = digit(&word) {
                            out.push((start - 1, 2, SUPERSCRIPTS[d]));
                        }
                    continue;
                }
                let n = word.chars().count();
                // Omega, and glued to a prefix: k""Omega
                if word == "Omega" || word.ends_with("\"\"Omega") {
                    if n > 5 {
                        out.push((start + n - 7, 1, '\0'));
                        out.push((start + n - 6, 1, '\0'));
                    }
                    out.push((start + n - 5, 5, 'Ω'));
                }
                if n > 1 && word.starts_with('u') && crate::is_unit(&word[1..]) {
                    out.push((start, 1, 'µ'));
                }
                if word.starts_with("deg") {
                    out.push((start, 3, '°'));
                }
            }
            K::NUMBER => {
                if start > 0 && code[start - 1] == '^' && word.chars().count() == 1 {
                    out.push((start - 1, 2, SUPERSCRIPTS[digit(&word).unwrap()]));
                }
            }
            K::OPERATOR | K::EVALUATE => match word.as_str() {
                "->" => out.push((start, 2, '→')),
                "<=" => out.push((start, 2, '≤')),
                ">=" => out.push((start, 2, '≥')),
                _ => {}
            },
            _ => {}
        }
    }
    out.sort_by_key(|t| t.0);
    out
}

/// Conceals of one piece of code; `element_start` and `element_end` are 0.
pub fn conceals_of(code: &str) -> Vec<Conceal> {
    let doc = Doc::new(code);
    conceals_in(&chars(code), &|_| false)
        .into_iter()
        .map(|(s, l, c)| Conceal { start: doc.to16(s), length: doc.to16(s + l) - doc.to16(s), character: c as u32, element_start: 0, element_end: 0 })
        .collect()
}

/// Conceals of the calculated solveng code of a document, with the raw element each belongs to, like Typewright's
/// math conceals: shown while the caret is outside that element.
pub fn conceals(document: &str, context: &Context) -> Vec<Conceal> {
    let a = Analysis::new(document, context);
    let unit = |w: &str| a.is_custom_unit(w);
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
    for e in a.elements.iter().filter(|e| !e.math && a.shown(e)) {
        push(e.body_start, conceals_in(&a.doc.c[e.body_start..e.body_end], &unit), (e.start, e.end));
    }
    // The strings of solveng's calls, each shown as typed while the caret is in it (quotes included).
    for t in a.calls.iter().flat_map(|c| &c.texts) {
        let code = &a.doc.c[t.start..t.end];
        let found = match t.role {
            Role::Code | Role::Mark | Role::Name => conceals_in(code, &unit),
            // Every word of a unit is a unit, as after `->`.
            Role::Unit => {
                let mut with: Vec<char> = "-> ".chars().collect();
                with.extend_from_slice(code);
                conceals_in(&with, &unit).into_iter().filter(|x| x.0 >= 3).map(|(s, l, c)| (s - 3, l, c)).collect()
            }
            Role::Choice => Vec::new(),
        };
        push(t.start, found, (t.start.saturating_sub(1), (t.end + 1).min(a.doc.c.len())));
    }
    out.sort_by_key(|c| c.start);
    out
}
