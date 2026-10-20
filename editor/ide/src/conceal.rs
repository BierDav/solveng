//! Solveng code shown as it renders while the caret is elsewhere, like math: `DeltaT` as `ΔT`, `Q0` as `Q₀`,
//! `kOhm` as `kΩ`, `->` as `→`, `m^2` as `m²`.

use std::sync::LazyLock;

use regex::Regex;

use crate::TokenKind as K;
use crate::text::{Doc, before, chars, sub};
use crate::tokens::code_tokens;
use crate::{Analysis, Conceal, Context};

pub(crate) const GREEK: &[(&str, char)] = &[
    ("alpha", 'α'), ("beta", 'β'), ("gamma", 'γ'), ("Gamma", 'Γ'), ("delta", 'δ'), ("Delta", 'Δ'), ("epsilon", 'ε'),
    ("zeta", 'ζ'), ("eta", 'η'), ("theta", 'θ'), ("Theta", 'Θ'), ("iota", 'ι'), ("kappa", 'κ'), ("lambda", 'λ'),
    ("Lambda", 'Λ'), ("mu", 'μ'), ("nu", 'ν'), ("xi", 'ξ'), ("Xi", 'Ξ'), ("pi", 'π'), ("Pi", 'Π'), ("rho", 'ρ'),
    ("sigma", 'σ'), ("Sigma", 'Σ'), ("tau", 'τ'), ("upsilon", 'υ'), ("phi", 'φ'), ("Phi", 'Φ'), ("chi", 'χ'),
    ("psi", 'ψ'), ("Psi", 'Ψ'), ("omega", 'ω'), ("Omega", 'Ω'),
];

pub(crate) fn greek(name: &str) -> Option<char> {
    GREEK.iter().find(|(n, _)| *n == name).map(|(_, c)| *c)
}

const SUBSCRIPTS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];
const SUPERSCRIPTS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];

static TRAILING_DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\p{L}+?)(\d+)$").unwrap());

/// Conceals of one piece of code in characters: start, length, the character shown instead.
fn conceals_in(code: &[char], unit: &dyn Fn(&str) -> bool) -> Vec<(usize, usize, char)> {
    let mut out = Vec::new();
    let mut comments = Vec::new();
    for (start, len, kind) in code_tokens(code, unit) {
        let word = sub(code, start, start + len);
        let digit = |w: &str| w.chars().next().and_then(|c| c.to_digit(10)).map(|d| d as usize);
        match kind {
            K::VARIABLE | K::FUNCTION | K::KEYWORD => {
                if word == "sqrt" {
                    out.push((start, 4, '√'));
                    continue;
                }
                let base = before(&word, "_");
                if let Some(g) = greek(base) {
                    out.push((start, base.chars().count(), g));
                    continue;
                }
                if base.starts_with("Delta") && base.len() > 5 {
                    out.push((start, 5, 'Δ'));
                    continue;
                }
                // Q0, alpha1: the digits as a subscript.
                let Some(m) = TRAILING_DIGITS.captures(&word) else { continue };
                let (letters, digits) = (&m[1], &m[2]);
                if letters.chars().count() > 1 && greek(letters).is_none() {
                    continue;
                }
                let n = letters.chars().count();
                if let Some(g) = greek(letters) {
                    out.push((start, n, g));
                }
                for (j, d) in digits.chars().enumerate() {
                    out.push((start + n + j, 1, SUBSCRIPTS[d.to_digit(10).unwrap() as usize]));
                }
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
                for spelling in ["Ohm", "ohm", "Omega"] {
                    if word.ends_with(spelling) {
                        out.push((start + n - spelling.len(), spelling.len(), 'Ω'));
                    }
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
            K::OPERATOR => match word.as_str() {
                "->" => out.push((start, 2, '→')),
                "<=" => out.push((start, 2, '≤')),
                ">=" => out.push((start, 2, '≥')),
                _ => {}
            },
            K::COMMENT => comments.push(start..start + len),
            _ => {}
        }
    }
    for (i, c) in code.iter().enumerate() {
        if *c == '*' && !comments.iter().any(|r| r.contains(&i)) {
            out.push((i, 1, '·'));
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
    for e in a.elements.iter().filter(|e| !e.math && a.shown(e)) {
        for (s, l, c) in conceals_in(&a.doc.c[e.body_start..e.body_end], &unit) {
            let start = a.doc.to16(e.body_start + s);
            out.push(Conceal {
                start,
                length: a.doc.to16(e.body_start + s + l) - start,
                character: c as u32,
                element_start: a.doc.to16(e.start),
                element_end: a.doc.to16(e.end),
            });
        }
    }
    out
}
