//! Go to definition and find usages of the names of solveng calculations.

use crate::calls::{Call, Role};
use crate::complete::{FUNCTIONS, definitions};
use crate::elements::Element;
use crate::text::Doc;
use crate::tokens::{Tok, code_tokens, formula_tokens, math_tokens};
use crate::scope::{self, File, Occurrence};
use crate::{Analysis, Context, Edit, Location, RenameTarget, TokenKind as K};

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
            K::VARIABLE => out.push(Name { key: crate::resolved_name(&text), start, end }),
            K::FUNCTION if !FUNCTIONS.contains(&text.as_str()) => out.push(Name { key: text, start, end }),
            K::ANNOTATION if text.starts_with('@') => out.push(Name { key: crate::resolved_name(&text[1..]), start: start + 1, end }),
            _ => {}
        }
    }
}

/// Names in math, where a subscript belongs to its name: `Q_0`, `t_"half"`, `"x"_("in" Delta)`.
fn math_names(doc: &Doc, base: usize, toks: Vec<Tok>, out: &mut Vec<Name>) {
    let c = &doc.c;
    let mut i = 0;
    while i < toks.len() {
        let (s, l, kind) = toks[i];
        let (start, mut end) = (base + s, base + s + l);
        i += 1;
        if kind == K::ANNOTATION {
            let key = crate::resolved_name(&doc.sub(start + 1, end).replace('"', ""));
            out.push(Name { key, start: start + 1, end });
            continue;
        }
        // A subscript on its own is part of a name before it.
        if !matches!(kind, K::VARIABLE | K::FUNCTION) || (start > 0 && c[start - 1] == '_') {
            continue;
        }
        let mut key = unquoted(&doc.sub(start, end));
        if kind == K::FUNCTION && FUNCTIONS.contains(&key.as_str()) {
            continue;
        }
        if c.get(end) == Some(&'_') {
            if c.get(end + 1) == Some(&'(') {
                // x_(a "b"): every token up to the parenthesis.
                let close = (end + 2..c.len()).find(|&j| c[j] == ')').unwrap_or(c.len());
                let mut subscript = String::new();
                while let Some(&(ns, nl, _)) = toks.get(i).filter(|t| base + t.0 < close) {
                    subscript += &math_piece(&doc.sub(base + ns, base + ns + nl));
                    i += 1;
                }
                if !subscript.is_empty() {
                    key = format!("{key}_{subscript}");
                    end = close + 1;
                }
            } else if let Some(&(ns, nl, K::VARIABLE)) = toks.get(i)
                && base + ns == end + 1
            {
                key = format!("{key}_{}", math_piece(&doc.sub(base + ns, base + ns + nl)));
                end = base + ns + nl;
                i += 1;
            }
        }
        let key = if kind == K::VARIABLE { crate::resolved_name(&key) } else { key };
        out.push(Name { key, start, end });
    }
}

/// The one name a stretch of math is, as (key, start, end) in characters; None for anything else.
pub(crate) fn math_name_of(math: &[char]) -> Option<(String, usize, usize)> {
    let doc = Doc::new(&math.iter().collect::<String>());
    let mut out = Vec::new();
    math_names(&doc, 0, math_tokens(math, &|_| false), &mut out);
    let toks = math_tokens(math, &|_| false);
    // Nothing but the name: no units, numbers or operators around it.
    let covered = |t: &Tok| out.first().is_some_and(|n| n.start <= t.0 && t.0 + t.1 <= n.end);
    if out.len() != 1 || !toks.iter().all(covered) || out[0].key.is_empty() {
        return None;
    }
    let n = out.remove(0);
    Some((n.key, n.start, n.end))
}

/// Quoted text's contents, other text as it is.
fn unquoted(text: &str) -> String {
    match text.strip_prefix('"').and_then(|t| t.strip_suffix('"')) {
        Some(inner) => inner.into(),
        None => text.into(),
    }
}

/// A piece of a subscript as solveng reads it: quotes go, a spelled Greek letter is the letter (`Delta` is Δ).
pub(crate) fn math_piece(text: &str) -> String {
    match crate::conceal::greek(text) {
        Some(letter) => letter.to_string(),
        None => unquoted(text),
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
            Role::Code | Role::Mark => {
                let code = &doc.c[t.start..t.end];
                let toks = if calls.iter().any(|c| c.is_curve(t)) { formula_tokens(code, unit) } else { code_tokens(code, unit) };
                code_names(doc, t.start, toks, &mut out)
            }
            Role::Name => {
                let text = doc.sub(t.start, t.end);
                let lead = text.chars().take_while(|ch| ch.is_whitespace()).count();
                let key = crate::resolved_name(text.trim());
                if !key.is_empty() {
                    out.push(Name { start: t.start + lead, end: t.start + lead + text.trim().chars().count(), key });
                }
            }
            _ => {}
        }
    }
    out.sort_by_key(|n| n.start);
    out
}

fn maths(elements: &[Element]) -> Vec<(usize, usize)> {
    elements.iter().filter(|e| e.math).map(|e| (e.body_start, e.body_end)).collect()
}

fn graphs(doc: &Doc, calls: &[Call]) -> Vec<(Vec<(usize, usize)>, Vec<(usize, usize)>)> {
    calls
        .iter()
        .filter(|c| c.function.name == "graph")
        .map(|c| {
            let curves = c.curves().map(|t| (t.start, t.end)).collect();
            let names = c.curves().filter_map(|t| curve_function(&doc.c[t.start..t.end]).map(|(s, e)| (t.start + s, t.start + e))).collect();
            (curves, names)
        })
        .collect()
}

/// Graph curves that are functions, `U(x: A) = …`: name start, name end and the curve's end.
fn curve_functions(doc: &Doc, calls: &[Call]) -> Vec<(usize, usize, usize)> {
    calls
        .iter()
        .flat_map(|c| c.curves())
        .filter_map(|t| curve_function(&doc.c[t.start..t.end]).map(|(s, e)| (t.start + s, t.start + e, t.end)))
        .collect()
}

/// The function a curve defines, `U` in `U(x) = …`, as a character range; not `cos(w) = w`, an equation.
fn curve_function(c: &[char]) -> Option<(usize, usize)> {
    let start = c.iter().position(|ch| !ch.is_whitespace())?;
    let mut end = start;
    while end < c.len() && (c[end].is_alphanumeric() || c[end] == '_') {
        end += 1;
    }
    let open = c[end..].iter().position(|ch| !ch.is_whitespace()).map(|i| end + i)?;
    let close = (open..c.len()).find(|&i| c[i] == ')')?;
    let eq = c[close + 1..].iter().position(|ch| !ch.is_whitespace()).map(|i| close + 1 + i)?;
    let name: String = c[start..end].iter().collect();
    (end > start && c[start].is_alphabetic() && c[open] == '(' && c[eq] == '=' && c.get(eq + 1) != Some(&'=') && !FUNCTIONS.contains(&name.as_str()))
        .then_some((start, end))
}

/// What each graph plots against: the names in its curves no definition covers, each linked to its first use in
/// that graph so they rename together. Returns those occurrences.
fn graphed(files: &[File], out: &mut [Occurrence]) -> Vec<usize> {
    let mut found = Vec::new();
    for (index, file) in files.iter().enumerate() {
        for (curves, names) in &file.graphs {
            let inside = |o: &Occurrence, spans: &[(usize, usize)]| spans.iter().any(|&(s, e)| s <= o.start && o.end <= e);
            let mut first: Vec<(String, usize)> = Vec::new();
            for i in 0..out.len() {
                let o = &out[i];
                if o.file != index || o.definition.is_some() || !inside(o, curves) || inside(o, names) {
                    continue;
                }
                let id = match first.iter().find(|(k, _)| *k == o.key) {
                    Some((_, id)) => *id,
                    None => {
                        first.push((o.key.clone(), i));
                        i
                    }
                };
                out[i].definition = Some(id);
                found.push(i);
            }
        }
    }
    found
}

/// Ranges of the document's parameters, in characters: of functions, where they're defined and used, and what a
/// graph plots against (see [`graphed`]).
pub(crate) fn parameter_ranges(a: &Analysis) -> Vec<(usize, usize)> {
    let (_, occurrences, graphed, params) = resolved_all(a);
    graphed.into_iter().chain(params.uses).map(|i| &occurrences[i]).filter(|o| o.file == 0).map(|o| (o.start, o.end)).collect()
}

/// Starts of the document's definitions with `=` of a name their scope already has, which state something about
/// it instead of defining it.
pub(crate) fn statements(a: &Analysis) -> Vec<usize> {
    let (_, _, _, params) = resolved_all(a);
    params.statements.into_iter().filter(|(f, _)| *f == 0).map(|(_, start)| start).collect()
}

/// The document's definitions that a later `:=` in their scope replaces: (start of the earlier, start of the later).
pub(crate) fn replaced(a: &Analysis) -> Vec<(usize, usize)> {
    let (_, _, _, params) = resolved_all(a);
    params.replaced.into_iter().filter(|((f, _), (g, _))| *f == 0 && *g == 0).map(|((_, a), (_, b))| (a, b)).collect()
}

/// The parameter at character `at` of the document: its name, and the function it belongs to with its parameters as
/// written and taken (`I: mA`), or None for what a graph plots against.
pub(crate) fn parameter_at(a: &Analysis, at: usize) -> Option<(String, Option<(String, Vec<String>)>)> {
    let (_, occurrences, graphed, params) = resolved_all(a);
    let hit = |i: usize| { let o = &occurrences[i]; o.file == 0 && o.start <= at && at <= o.end };
    if let Some((k, _)) = params.uses.iter().enumerate().find(|(_, i)| hit(**i)) {
        let f = params.owners[k];
        return Some((occurrences[params.uses[k]].key.clone(), Some((params.names[f].clone(), params.signatures[f].2.clone()))));
    }
    graphed.into_iter().find(|&i| hit(i)).map(|i| (occurrences[i].key.clone(), None))
}

/// Functions' parameters resolved (`..` replaced by what it takes), by the file's path (None for the document) and
/// the start of the function's name.
pub(crate) fn signatures(a: &Analysis) -> Vec<(Option<String>, usize, String)> {
    let (files, _, _, params) = resolved_all(a);
    params.signatures.into_iter().map(|(f, start, ps)| (if f == 0 { None } else { Some(files[f].path.clone()) }, start, ps.join(", "))).collect()
}

fn strings(calls: &[Call]) -> Vec<(usize, usize)> {
    calls.iter().flat_map(|c| &c.texts).map(|t| (t.start, t.end)).collect()
}

/// The document and the files around in compile order, every name with the definition it means, and the index
/// of the document among the files.
fn resolved<'a>(a: &'a Analysis) -> (Vec<File<'a>>, Vec<Occurrence>) {
    let (files, occurrences, ..) = resolved_all(a);
    (files, occurrences)
}

/// [`resolved`], which occurrences a graph plots against, and functions' parameters.
fn resolved_all<'a>(a: &'a Analysis) -> (Vec<File<'a>>, Vec<Occurrence>, Vec<usize>, scope::Params) {
    let unit = |w: &str| a.is_custom_unit(w);
    let named = |n: Vec<Name>| n.into_iter().map(|n| (n.key, n.start, n.end)).collect::<Vec<_>>();
    let mut files = vec![File {
        path: a.context.path.clone(),
        doc: &a.doc,
        definitions: definitions(&a.doc, &a.elements, &a.calls, &|e| a.active(e)),
        names: named(names(&a.doc, &a.elements, &a.calls, &|e| a.active(e), &unit)),
        maths: maths(&a.elements),
        strings: strings(&a.calls),
        graphs: graphs(&a.doc, &a.calls),
        functions: curve_functions(&a.doc, &a.calls),
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
            maths: maths(&part.elements),
            strings: strings(&part.calls),
            graphs: graphs(&part.doc, &part.calls),
            functions: curve_functions(&part.doc, &part.calls),
        });
    }
    let main = a.context.main.as_ref().and_then(|m| files.iter().position(|f| f.path == *m));
    let implicit = crate::implicit_parameters(&a.doc.s).or_else(|| crate::implicit_parameters(&a.context.text)).unwrap_or_else(|| "no".into());
    let (mut occurrences, params) = scope::occurrences(&files, 0, main, implicit);
    let graphed = graphed(&files, &mut occurrences);
    (files, occurrences, graphed, params)
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
/// just the name. On one of the document's own units (`ü` of `units: (ü: auto)`), its key in `units: (…)`.
pub fn definition(document: &str, context: &Context, offset: u32) -> Option<Location> {
    let a = Analysis::new(document, context);
    let (files, occurrences) = resolved(&a);
    let Some(i) = at_offset(&a, &occurrences, offset) else { return crate::unit_refs::definition(&a, offset) };
    Some(location(&files, &occurrences[occurrences[i].definition?]))
}

/// The usages of the definition the solveng name at `offset` means (see [`definition`]), the definition included:
/// the names after it while it's visible, up to a redefinition, a reset or the end of its block, in the document
/// and in the files it includes or is included from. Units spelled like it (`5 V` next to a variable `V`) don't count.
/// On one of the document's own units, its keys and every use, prefixed ones (`mü`) and other units' values included.
pub fn usages(document: &str, context: &Context, offset: u32) -> Vec<Location> {
    let a = Analysis::new(document, context);
    let (files, occurrences) = resolved(&a);
    let Some(i) = at_offset(&a, &occurrences, offset) else { return crate::unit_refs::usages(&a, offset).unwrap_or_default() };
    let Some(id) = occurrences[i].definition else { return Vec::new() };
    occurrences.iter().filter(|o| o.definition == Some(id)).map(|o| location(&files, o)).collect()
}

/// The solveng name at `offset` that [`rename`] renames, as it's written there: its range and text.
pub fn rename_target(document: &str, context: &Context, offset: u32) -> Option<RenameTarget> {
    let a = Analysis::new(document, context);
    let (files, occurrences) = resolved(&a);
    let Some(i) = at_offset(&a, &occurrences, offset) else { return crate::unit_refs::rename_target(&a, offset) };
    let o = &occurrences[i];
    o.definition?;
    let l = location(&files, o);
    Some(RenameTarget { start: l.start, end: l.end, name: a.doc.sub(o.start, o.end) })
}

/// Renames the solveng name at `offset` and its [`usages`] to `name`, typed as code (`R_load`, `Delta""r`) or
/// as math (`R_"load"`, `"tews"`). Each place gets the spelling it needs: code as typed, strings escaped, math
/// with longer names quoted (`"tews"`, `t_"half"`). Empty if there's nothing to rename or `name` isn't a name.
pub fn rename(document: &str, context: &Context, offset: u32, name: &str) -> Vec<Edit> {
    let a = Analysis::new(document, context);
    let (files, occurrences) = resolved(&a);
    let Some(i) = at_offset(&a, &occurrences, offset) else { return crate::unit_refs::rename(&a, offset, name).unwrap_or_default() };
    let Some(id) = occurrences[i].definition else { return Vec::new() };
    let typed = name.trim();
    let quoted = typed.replace("\"\"", "").contains('"');
    let key = if quoted {
        match math_name_of(&typed.chars().collect::<Vec<_>>()) {
            Some((key, _, _)) => key,
            None => return Vec::new(),
        }
    } else {
        crate::resolved_name(typed)
    };
    let function = files[occurrences[id].file].definitions.iter().any(|d| d.function && d.start == occurrences[id].start);
    let valid = key.chars().next().is_some_and(char::is_alphabetic) && key.chars().all(|c| c.is_alphanumeric() || c == '_');
    if !valid {
        return Vec::new();
    }
    let code = if quoted { key.clone() } else { typed.to_string() };
    occurrences
        .iter()
        .filter(|o| o.definition == Some(id))
        .map(|o| {
            let f = &files[o.file];
            let inside = |spans: &[(usize, usize)]| spans.iter().any(|&(s, e)| s <= o.start && o.end <= e);
            let text = if inside(&f.maths) && !function {
                crate::complete::math_name(&key)
            } else if inside(&f.strings) {
                code.replace('\\', "\\\\").replace('"', "\\\"")
            } else {
                code.clone()
            };
            let l = location(&files, o);
            Edit { path: l.path, start: l.start, end: l.end, text }
        })
        .collect()
}
