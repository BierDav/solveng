//! The document's own units (`solve-config(units: (ü: auto, mü: "10ü"))`): where they're defined and used, for go
//! to definition, usages and rename.

use std::sync::LazyLock;

use regex::Regex;

use crate::calls::{Call, Role};
use crate::elements::Element;
use crate::text::{Doc, char_index};
use crate::tokens::{code_tokens, formula_tokens, math_tokens};
use crate::units::{Custom, PREFIXES};
use crate::{Analysis, Edit, Location, RenameTarget, TokenKind as K};

/// Where a unit is written: its key in `units: (…)` or a use, as a character range of just the unit's name
/// (`ü` in `mü`).
struct Spot {
    key: String,
    start: usize,
    end: usize,
    /// A use written bare in math, where a longer name needs quotes.
    bare_math: bool,
}

static UNITS_ARG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"units\s*:\s*\(").unwrap());
static KEY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?:^|[(,])\s*([\p{L}][\p{L}\p{N}_-]*|"[^"]+")\s*:"#).unwrap());
static WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\p{L}°%‰µΩ]+").unwrap());

/// The keys of `units: (…)` dictionaries.
fn definitions(doc: &Doc, out: &mut Vec<Spot>) {
    let text = &doc.s;
    for m in UNITS_ARG.find_iter(text) {
        let start = m.end();
        let end = text[start..].find(')').map_or(text.len(), |i| start + i);
        // `(` back in so the first key matches like the others.
        let inner = &text[start - 1..end];
        for c in KEY.captures_iter(inner) {
            let k = c.get(1).unwrap();
            let quoted = k.as_str().starts_with('"');
            let (s, e) = (start - 1 + k.start() + quoted as usize, start - 1 + k.end() - quoted as usize);
            out.push(Spot { key: text[s..e].to_string(), start: char_index(text, s), end: char_index(text, e), bare_math: false });
        }
    }
}

/// The custom unit a written unit word means: itself, or a prefix and one (`mü`), with the range of its name.
fn resolve(word: &str, custom: &[Custom]) -> Option<(String, usize)> {
    if custom.iter().any(|u| u.name == word) {
        return Some((word.into(), 0));
    }
    let first = word.chars().next()?;
    let rest = &word[first.len_utf8()..];
    let prefix = PREFIXES.iter().any(|(p, _)| p.chars().eq([first]));
    (prefix && !rest.is_empty() && custom.iter().any(|u| u.name == rest)).then(|| (rest.to_string(), 1))
}

/// Uses of custom units in unit tokens of `code` (at `base`).
fn uses(doc: &Doc, base: usize, toks: Vec<(usize, usize, u8)>, math: bool, custom: &[Custom], out: &mut Vec<Spot>) {
    for (s, l, kind) in toks {
        if kind != K::UNIT {
            continue;
        }
        let text = doc.sub(base + s, base + s + l);
        let quoted = text.starts_with('"');
        for w in WORD.find_iter(&text) {
            let Some((key, skip)) = resolve(w.as_str(), custom) else { continue };
            let start = base + s + char_index(&text, w.start()) + skip;
            out.push(Spot { start, end: start + key.chars().count(), key, bare_math: math && !quoted });
        }
    }
}

/// Every definition and use of a custom unit in a text.
fn spots(doc: &Doc, elements: &[Element], calls: &[Call], active: &dyn Fn(&Element) -> bool, custom: &[Custom]) -> Vec<Spot> {
    let unit = |w: &str| custom.iter().any(|u| u.name == w);
    let mut out = Vec::new();
    definitions(doc, &mut out);
    for e in elements.iter().filter(|e| active(e)) {
        let body = &doc.c[e.body_start..e.body_end];
        let toks = if e.math { math_tokens(body, &unit) } else { code_tokens(body, &unit) };
        uses(doc, e.body_start, toks, e.math, custom, &mut out);
    }
    for call in calls {
        for t in &call.texts {
            let code = &doc.c[t.start..t.end];
            let toks = match t.role {
                Role::Code if call.is_curve(t) => formula_tokens(code, &unit),
                Role::Code | Role::Mark => code_tokens(code, &unit),
                // Every word of a unit argument is a unit, as after `->`.
                Role::Unit => {
                    let mut with: Vec<char> = "-> ".chars().collect();
                    with.extend_from_slice(code);
                    code_tokens(&with, &unit).into_iter().filter(|x| x.0 >= 3).map(|(s, l, k)| (s - 3, l, k)).collect()
                }
                _ => Vec::new(),
            };
            uses(doc, t.start, toks, false, custom, &mut out);
        }
    }
    out
}

/// The spots of the document (`path` None) and the files around.
fn all<'a>(a: &'a Analysis) -> Vec<(Option<String>, &'a Doc, Vec<Spot>)> {
    let mut out = vec![(None, &a.doc, spots(&a.doc, &a.elements, &a.calls, &|e| a.active(e), &a.units))];
    for part in a.around() {
        let Some(path) = &part.path else { continue };
        if *path == a.context.path {
            continue;
        }
        out.push((Some(path.clone()), &part.doc, spots(&part.doc, &part.elements, &part.calls, &|e| part.active(e), &a.units)));
    }
    out
}

/// The custom unit at a UTF-16 offset of the document, and the spot there.
fn at(a: &Analysis, all: &[(Option<String>, &Doc, Vec<Spot>)], offset: u32) -> Option<(String, usize, usize)> {
    let c = a.doc.from16(offset as usize);
    all[0].2.iter().find(|s| s.start <= c && c <= s.end).map(|s| (s.key.clone(), s.start, s.end))
}

fn location(path: &Option<String>, doc: &Doc, s: &Spot) -> Location {
    Location { path: path.clone(), start: doc.to16(s.start), end: doc.to16(s.end) }
}

/// Where the custom unit at `offset` is defined: its key in the last `units: (…)` that sets it.
pub(crate) fn definition(a: &Analysis, offset: u32) -> Option<Location> {
    let all = all(a);
    let (key, ..) = at(a, &all, offset)?;
    let mut found = None;
    for (path, doc, spots) in &all {
        for s in spots.iter().filter(|s| s.key == key && is_key(doc, s)) {
            found = Some(location(path, doc, s));
        }
    }
    found
}

/// Whether a spot is a key of `units: (…)`: a colon follows it.
fn is_key(doc: &Doc, s: &Spot) -> bool {
    let after = doc.c[s.end..].iter().find(|c| !c.is_whitespace() && **c != '"');
    after == Some(&':')
}

/// Every definition and use of the custom unit at `offset`; None off custom units.
pub(crate) fn usages(a: &Analysis, offset: u32) -> Option<Vec<Location>> {
    let all = all(a);
    let (key, ..) = at(a, &all, offset)?;
    Some(all.iter().flat_map(|(path, doc, spots)| spots.iter().filter(|s| s.key == key).map(|s| location(path, doc, s))).collect())
}

pub(crate) fn rename_target(a: &Analysis, offset: u32) -> Option<RenameTarget> {
    let all = all(a);
    let (key, start, end) = at(a, &all, offset)?;
    Some(RenameTarget { start: a.doc.to16(start), end: a.doc.to16(end), name: key })
}

static NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^[^\s\d*/^()\[\]+\-−·⋅=,.;"]+$"#).unwrap());
static IDENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[\p{L}_][\p{L}\p{N}_-]*$").unwrap());

/// Renames the custom unit at `offset`: its keys in `units: (…)` (quoted if typst needs it) and every use, quoted
/// in math when a bare one gets longer. None off custom units; empty if `name` can't be a unit name.
pub(crate) fn rename(a: &Analysis, offset: u32, name: &str) -> Option<Vec<Edit>> {
    let all = all(a);
    let (key, ..) = at(a, &all, offset)?;
    let name = name.trim();
    if !NAME.is_match(name) {
        return Some(Vec::new());
    }
    let mut out = Vec::new();
    for (path, doc, spots) in &all {
        for s in spots.iter().filter(|s| s.key == key) {
            let quoted = s.start > 0 && doc.c[s.start - 1] == '"' && doc.c.get(s.end) == Some(&'"');
            let text = if is_key(doc, s) && !quoted && !IDENT.is_match(name) {
                format!("\"{name}\"")
            } else if s.bare_math && name.chars().count() > 1 {
                format!("\"{name}\"")
            } else {
                name.to_string()
            };
            let l = location(path, doc, s);
            out.push(Edit { path: l.path, start: l.start, end: l.end, text });
        }
    }
    Some(out)
}
