//! Solveng code shown as it renders while the caret is elsewhere, like math: `Delta_T` as `Δ_T`, `Q0` as `Q₀`,
//! `k""Omega` as `kΩ`, `->` as `→`, `m^2` as `m²`.

use std::sync::LazyLock;

use regex::Regex;

use crate::TokenKind as K;
use crate::text::{Doc, before, chars, sub};
use crate::tokens::code_tokens;
use crate::calls::Role;
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

fn greek_name(c: char) -> Option<&'static str> {
    GREEK.iter().find(|(_, g)| *g == c).map(|(n, _)| *n)
}

/// A name spelled out for typing, as math writes it: `Δ_R` → `Delta_R`, `Δr_R` → `Delta""r_R`.
pub(crate) fn spelled(name: &str) -> String {
    glued(name, |c| greek_name(c).map(String::from))
}

/// A name as typst math can read it, Greek letters glued to their neighbours: `Δr_R` → `Δ""r_R`.
pub(crate) fn math_glued(name: &str) -> String {
    glued(name, |c| greek_name(c).map(|_| c.to_string()))
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

/// A variable's name as solveng stores it: a Greek letter spelled out, alone or before an index, is the letter, as
/// math reads it (`Delta_R` → `Δ_R`, `alpha2` → `α2`); `DeltaT` stays a name of its own.
/// Empty strings glue names as in math: `Delta""r_R` → `Δr_R`, `Delta""Gamma` → `ΔΓ`.
pub fn resolved_name(name: &str) -> String {
    name.split("\"\"")
        .map(|part| {
            let split = part.find(|c: char| c == '_' || c.is_ascii_digit()).unwrap_or(part.len());
            match greek(&part[..split]) {
                Some(letter) => format!("{letter}{}", &part[split..]),
                None => part.into(),
            }
        })
        .collect()
}

const SUBSCRIPTS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];
const SUPERSCRIPTS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];

static TRAILING_DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\p{L}+?)(\d+)$").unwrap());

/// Conceals of one part of a name at `at`: a spelled Greek letter before an index (`Delta_R`, `alpha1`), digits
/// as a subscript (`Q0`).
fn piece_conceals(piece: &str, at: usize, out: &mut Vec<(usize, usize, char)>) {
    let base = before(piece, "_");
    if let Some(g) = greek(base) {
        out.push((at, base.chars().count(), g));
        return;
    }
    let Some(m) = TRAILING_DIGITS.captures(piece) else { return };
    let (letters, digits) = (&m[1], &m[2]);
    if letters.chars().count() > 1 && greek(letters).is_none() {
        return;
    }
    let n = letters.chars().count();
    if let Some(g) = greek(letters) {
        out.push((at, n, g));
    }
    for (j, d) in digits.chars().enumerate() {
        out.push((at + n + j, 1, SUBSCRIPTS[d.to_digit(10).unwrap() as usize]));
    }
}

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
                // Each part of a glued name on its own (Delta""Omega is ΔΩ), the empty strings hidden; a longer
                // word starting like a letter (Deltas) is a name of its own.
                let mut at = start;
                for (k, piece) in word.split("\"\"").enumerate() {
                    if k > 0 {
                        out.push((at, 1, '\0'));
                        out.push((at + 1, 1, '\0'));
                        at += 2;
                    }
                    piece_conceals(piece, at, &mut out);
                    at += piece.chars().count();
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
