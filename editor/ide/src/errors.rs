//! The errors the last compile reported, placed in the document as it is now.

use crate::text::index_of;
use crate::{Analysis, Problem, ReportError};

/// Each reported error of the document where it still applies: in the element it was reported for (the one
/// with the same source, nearest to where it was), at its statement when the report names one. An element
/// edited since has no error until the next compile tells. Errors of explicit value/graph calls can have no
/// element; their statement is looked up in the equations passed to calls instead.
pub(crate) fn reported(a: &Analysis) -> Vec<Problem> {
    let mut out: Vec<Problem> = Vec::new();
    for err in &a.context.errors {
        let range = if err.text.is_empty() {
            unplaced(a, &err.source)
        } else if err.path == a.context.path {
            placed(a, err)
        } else {
            None
        };
        let Some((start, end)) = range else { continue };
        let (start, end) = invalid_name(a, start, end, &err.message).unwrap_or((start, end));
        let problem = Problem { start: a.doc.to16(start), end: a.doc.to16(end), message: err.message.clone(), severity: "error" };
        if !out.contains(&problem) {
            out.push(problem);
        }
    }
    out
}

/// In the element the error was reported for.
fn placed(a: &Analysis, err: &ReportError) -> Option<(usize, usize)> {
    let e = a
        .elements
        .iter()
        .filter(|e| a.evaluates_at(e.body_start) && a.doc.sub(e.start, e.end) == err.text)
        .min_by_key(|e| (i64::from(a.doc.to16(e.start)) - err.start as i64).abs())?;
    let found = (!err.source.is_empty()).then(|| statement(a, e.body_start, e.body_end, &err.source)).flatten();
    Some(found.unwrap_or_else(|| trimmed(a, e.body_start, e.body_end)))
}

/// A statement passed to a calculation call: in its math arguments.
fn unplaced(a: &Analysis, source: &str) -> Option<(usize, usize)> {
    if source.is_empty() {
        return None;
    }
    a.elements.iter()
        .filter(|e| a.calls.iter().any(|c| c.maths.contains(&(e.start, e.end))))
        .find_map(|e| statement(a, e.body_start, e.body_end, source))
}

/// Where `source` is in `start..end`, also as a Typst string writes it (`\"`, `\\`).
fn statement(a: &Analysis, start: usize, end: usize, source: &str) -> Option<(usize, usize)> {
    let escaped = source.replace('\\', "\\\\").replace('"', "\\\"");
    [source, escaped.as_str()].into_iter().find_map(|s| index_of(&a.doc.c[..end], s, start).map(|at| (at, at + s.chars().count())))
}

/// `start..end` without the whitespace around.
fn trimmed(a: &Analysis, start: usize, end: usize) -> (usize, usize) {
    let c = &a.doc.c;
    let s = (start..end).find(|&i| !c[i].is_whitespace()).unwrap_or(start);
    let e = (s..end).rev().find(|&i| !c[i].is_whitespace()).map_or(s, |i| i + 1);
    (s, e)
}

// Match validation errors to their name while preserving escaped source offsets.
fn invalid_name(a: &Analysis, start: usize, end: usize, message: &str) -> Option<(usize, usize)> {
    let transported = a.calls.iter().flat_map(|c| &c.texts).any(|t| t.start <= start && end <= t.end);
    let source = &a.doc.c[start..end];
    let (text, offsets) = if transported { crate::text::unescape(source) }
        else { (source.to_vec(), (0..=source.len()).collect()) };
    let mut i = 0;
    while i < text.len() {
        if text[i] == '"' || solveng_names::is_start(text[i]) || solveng_names::numeric_start(&text, i) {
            let end = crate::tokens::name_end(&text, i);
            if solveng_names::scan(&text, i).is_err_and(|error| error == message) {
                return Some((start + offsets[i], start + offsets[end]));
            }
            i = end;
        } else { i += 1; }
    }
    None
}
