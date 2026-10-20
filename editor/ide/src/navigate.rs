//! Go to definition and find usages of the names of solveng calculations.

use crate::calls::{Call, Role};
use crate::complete::{FUNCTIONS, definitions};
use crate::elements::Element;
use crate::text::Doc;
use crate::tokens::{Tok, code_tokens, math_tokens};
use crate::scope::{self, File, Occurrence};
use crate::{Analysis, Context, Location, TokenKind as K};

/// A name used where solveng calculates: its spelling in code (math's `t_"half"` is `t_half`) and where it is,
/// in characters.
struct Name {
    key: String,
    start: usize,
    end: usize,
}

/// Names in solveng code: variables, functions other than the built-in ones, and `@`-inlined formulas.
fn code_names(doc: &Doc, base: usize, toks: Vec<Tok>, out: &mut Vec<Name>) {
    for (s, l, kind) in toks {
        let (start, end) = (base + s, base + s + l);
        let text = doc.sub(start, end);
        match kind {
            K::VARIABLE => out.push(Name { key: text, start, end }),
            K::FUNCTION if !FUNCTIONS.contains(&text.as_str()) => out.push(Name { key: text, start, end }),
            K::ANNOTATION if text.starts_with('@') => out.push(Name { key: text[1..].into(), start: start + 1, end }),
            _ => {}
        }
    }
}

/// Names in math, where a subscript belongs to its name: `Q_0`, `t_"half"`.
fn math_names(doc: &Doc, base: usize, toks: Vec<Tok>, out: &mut Vec<Name>) {
    let c = &doc.c;
    let mut i = 0;
    while i < toks.len() {
        let (s, l, kind) = toks[i];
        let (start, mut end) = (base + s, base + s + l);
        i += 1;
        // A subscript on its own is part of a name before it.
        if !matches!(kind, K::VARIABLE | K::FUNCTION) || (start > 0 && c[start - 1] == '_') {
            continue;
        }
        let mut key = doc.sub(start, end);
        if kind == K::FUNCTION && FUNCTIONS.contains(&key.as_str()) {
            continue;
        }
        if let Some(&(ns, nl, K::VARIABLE)) = toks.get(i)
            && c.get(end) == Some(&'_')
            && base + ns == end + 1
        {
            key = format!("{key}_{}", doc.sub(base + ns, base + ns + nl).replace('"', ""));
            end = base + ns + nl;
            i += 1;
        }
        out.push(Name { key, start, end });
    }
}

/// Every name solveng reads in a text: in calculated code and math, and in the strings of its calls.
fn names(doc: &Doc, elements: &[Element], calls: &[Call], active: &dyn Fn(&Element) -> bool, unit: &dyn Fn(&str) -> bool) -> Vec<Name> {
    let mut out = Vec::new();
    for e in elements.iter().filter(|e| active(e)) {
        let body = &doc.c[e.body_start..e.body_end];
        if e.math {
            math_names(doc, e.body_start, math_tokens(body, unit), &mut out);
        } else {
            code_names(doc, e.body_start, code_tokens(body, unit), &mut out);
        }
    }
    for t in calls.iter().flat_map(|c| &c.texts) {
        match t.role {
            Role::Code | Role::Mark => code_names(doc, t.start, code_tokens(&doc.c[t.start..t.end], unit), &mut out),
            Role::Name => {
                let text = doc.sub(t.start, t.end);
                let lead = text.chars().take_while(|ch| ch.is_whitespace()).count();
                let key = text.trim().to_string();
                if !key.is_empty() {
                    out.push(Name { start: t.start + lead, end: t.start + lead + key.chars().count(), key });
                }
            }
            _ => {}
        }
    }
    out.sort_by_key(|n| n.start);
    out
}

/// The document and the files around in compile order, every name with the definition it means, and the index
/// of the document among the files.
fn resolved<'a>(a: &'a Analysis) -> (Vec<File<'a>>, Vec<Occurrence>) {
    let unit = |w: &str| a.is_custom_unit(w);
    let named = |n: Vec<Name>| n.into_iter().map(|n| (n.key, n.start, n.end)).collect::<Vec<_>>();
    let mut files = vec![File {
        path: a.context.path.clone(),
        doc: &a.doc,
        definitions: definitions(&a.doc, &a.elements, &a.calls, &|e| a.active(e)),
        names: named(names(&a.doc, &a.elements, &a.calls, &|e| a.active(e), &unit)),
    }];
    for part in a.around() {
        let Some(path) = &part.path else { continue };
        if *path == a.context.path {
            continue;
        }
        files.push(File {
            path: path.clone(),
            doc: &part.doc,
            definitions: definitions(&part.doc, &part.elements, &part.calls, &|e| part.active(e)),
            names: named(names(&part.doc, &part.elements, &part.calls, &|e| part.active(e), &unit)),
        });
    }
    let main = a.context.main.as_ref().and_then(|m| files.iter().position(|f| f.path == *m));
    let occurrences = scope::occurrences(&files, 0, main);
    (files, occurrences)
}

/// The occurrence of a name at a UTF-16 offset of the document.
fn at_offset(a: &Analysis, occurrences: &[Occurrence], offset: u32) -> Option<usize> {
    let at = a.doc.from16(offset as usize);
    occurrences.iter().position(|o| o.file == 0 && o.start <= at && at <= o.end)
}

fn location(files: &[File], o: &Occurrence) -> Location {
    let f = &files[o.file];
    Location { path: if o.file == 0 { None } else { Some(f.path.clone()) }, start: f.doc.to16(o.start), end: f.doc.to16(o.end) }
}

/// Where the solveng name at `offset` is defined, as the compile sees it: the latest definition before it that's
/// visible there, reading the document in compile order from the main file (see [`Context::with_main`]), where
/// `#include` inlines a file, a `solve-block` starts empty (or with the outer definitions with `inherit: true`)
/// and its definitions end with it, and `reset` starts over. On a definition, that definition. The range covers
/// just the name.
pub fn definition(document: &str, context: &Context, offset: u32) -> Option<Location> {
    let a = Analysis::new(document, context);
    let (files, occurrences) = resolved(&a);
    let o = &occurrences[at_offset(&a, &occurrences, offset)?];
    Some(location(&files, &occurrences[o.definition?]))
}

/// The usages of the definition the solveng name at `offset` means (see [`definition`]), the definition included:
/// the names after it while it's visible, up to a redefinition, a reset or the end of its block, in the document
/// and in the files it includes or is included from. Units spelled like it (`5 V` next to a variable `V`) don't count.
pub fn usages(document: &str, context: &Context, offset: u32) -> Vec<Location> {
    let a = Analysis::new(document, context);
    let (files, occurrences) = resolved(&a);
    let Some(id) = at_offset(&a, &occurrences, offset).and_then(|i| occurrences[i].definition) else { return Vec::new() };
    occurrences.iter().filter(|o| o.definition == Some(id)).map(|o| location(&files, o)).collect()
}
