//! Go to definition and find usages of the names of solveng calculations.

use crate::calls::{Call, Role};
use crate::complete::{FUNCTIONS, definitions};
use crate::elements::Element;
use crate::text::Doc;
use crate::tokens::{Tok, code_tokens, math_tokens};
use crate::scope::{self, File, Occurrence};
use crate::{Analysis, Context, Edit, Location, RenameTarget, TokenKind as K};

/// A name used where solveng calculates: its spelling in code (math's `t_"half"` is `t_half`) and where it is,
/// in characters.
struct Name {
    key: String,
    start: usize,
    end: usize,
}

/// Names in solveng code: variables and functions other than the built-in ones.
fn code_names(doc: &Doc, base: usize, toks: Vec<Tok>, out: &mut Vec<Name>) {
    for (s, l, kind) in toks {
        let (start, end) = (base + s, base + s + l);
        let raw: Vec<char> = doc.sub(start, end).chars().collect();
        let (decoded, _) = crate::text::unescape(&raw);
        if solveng_names::scan(&decoded, 0).is_err() { continue; }
        let text: String = decoded.into_iter().collect();
        match kind {
            K::VARIABLE => out.push(Name { key: crate::resolved_name(&text), start, end }),
            K::FUNCTION if !FUNCTIONS.contains(&text.as_str()) => out.push(Name { key: crate::resolved_name(&text), start, end }),
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
        // A subscript on its own is part of a name before it.
        if !matches!(kind, K::VARIABLE | K::FUNCTION) || (start > 0 && c[start - 1] == '_') {
            continue;
        }
        if solveng_names::scan(&c[start..end], 0).is_err() { continue; }
        let mut key = crate::resolved_name(&doc.sub(start, end));
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
        let key = crate::resolved_name(&key);
        out.push(Name { key, start, end });
    }
}

/// The one name a stretch of math is, as (key, start, end) in characters; None for anything else.
pub(crate) fn math_name_of(math: &[char]) -> Option<(String, usize, usize)> {
    // A quoted name is a name even when it spells a unit: `"km" := 23` defines a variable.
    let lead = math.iter().take_while(|c| c.is_whitespace()).count();
    let end = math.len().saturating_sub(math.iter().rev().take_while(|c| c.is_whitespace()).count());
    let trimmed: String = math[lead.min(end)..end].iter().collect();
    if let Ok(name) = solveng_names::resolve(&trimmed) {
        return Some((name, lead, end));
    }
    let text: String = math.iter().collect();
    let doc = Doc::new(&text);
    let mut out = Vec::new();
    math_names(&doc, 0, math_tokens(math, &crate::is_unit, &|_| false), &mut out);
    let toks = math_tokens(math, &crate::is_unit, &|_| false);
    // Nothing but the name: no units, numbers or operators around it.
    let covered = |t: &Tok| out.first().is_some_and(|n| n.start <= t.0 && t.0 + t.1 <= n.end);
    if out.len() != 1 || !toks.iter().all(covered) || out[0].key.is_empty() {
        return None;
    }
    let n = out.remove(0);
    Some((n.key, n.start, n.end))
}

/// A piece of a subscript as solveng reads it: quotes go, a spelled Greek letter is the letter (`Delta` is Δ).
pub(crate) fn math_piece(text: &str) -> String { crate::resolved_name(text) }

/// Every name solveng reads in a text: in calculated code and math, and in the strings of its calls.
fn names(doc: &Doc, elements: &[Element], calls: &[Call], active: &dyn Fn(&Element) -> bool, unit: &dyn Fn(&str) -> bool, defined: &dyn Fn(&str) -> bool, units_at: &dyn Fn(usize) -> Vec<crate::units::Custom>) -> Vec<Name> {
    let mut out = Vec::new();
    for e in elements.iter().filter(|e| active(e)) {
        let body = &doc.c[e.body_start..e.body_end];
        if e.math {
            let units = units_at(e.body_start);
            math_names(doc, e.body_start, math_tokens(body, &|w| crate::units::visible(w, &units), defined), &mut out);
        } else {
            code_names(doc, e.body_start, code_tokens(body, unit), &mut out);
        }
    }
    for t in calls.iter().flat_map(|c| &c.texts) {
        match t.role {
            Role::Name => {
                let text = doc.sub(t.start, t.end);
                let lead = text.chars().take_while(|ch| ch.is_whitespace()).count();
                // a function by its name: clear("f()")
                let name = text.split('(').next().unwrap_or("").trim();
                let key = crate::resolved_name(name);
                if !key.is_empty() {
                    out.push(Name { start: t.start + lead, end: t.start + lead + name.chars().count(), key });
                }
            }
            _ => {}
        }
    }
    // Constructor strings name operations and units, never worksheet variables with the same spelling.
    let bindings = crate::bindings::Bindings::new(doc);
    let references: Vec<_> = bindings.operator_strings().iter().map(|(s, e, _)| (*s, *e))
        .chain(calls.iter().filter(|c| c.function.name == "unit").flat_map(|c| c.texts.iter().map(|t| (t.start, t.end)))).collect();
    out.retain(|n| !references.iter().any(|&(s, e)| n.start < e + 1 && s.saturating_sub(1) < n.end));
    // Typst bindings (`#let km = unit("km")`, `#import units: km`) are Typst's: solveng sees their values.
    out.retain(|n| !bindings.contains(n.start));
    out.sort_by_key(|n| n.start);
    out
}

fn locals(doc: &Doc, elements: &[Element], _calls: &[Call], active: &dyn Fn(&Element) -> bool) -> Vec<crate::locals::Local> {
    let mut out = Vec::new();
    for e in elements.iter().filter(|e| active(e)) {
        out.extend(crate::locals::read(&doc.c[e.body_start..e.body_end], e.body_start, false, !e.math));
    }
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
            let curves = c.curves(doc).collect();
            let names = c.curves(doc).filter_map(|(start, end)| curve_function(&doc.c[start..end]).map(|(s, e)| (start + s, start + e))).collect();
            (curves, names)
        })
        .collect()
}

/// Graph curves that are functions, `U(x: A) = …`: name start, name end and the curve's end.
fn curve_functions(doc: &Doc, calls: &[Call]) -> Vec<(usize, usize, usize)> {
    calls
        .iter()
        .flat_map(|c| c.curves(doc))
        .filter_map(|(start, end)| curve_function(&doc.c[start..end]).map(|(s, e)| (start + s, start + e, end)))
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
                if o.file != index || o.definition.is_some() || FUNCTIONS.contains(&o.key.as_str()) || !inside(o, curves) || inside(o, names) {
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
    graphed.into_iter().chain(params.uses).chain(params.locals).map(|i| &occurrences[i]).filter(|o| o.file == 0).map(|o| (o.start, o.end)).collect()
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
    if params.locals.iter().any(|&i| hit(i)) { return None; }
    if let Some((k, _)) = params.uses.iter().enumerate().find(|(_, i)| hit(**i)) {
        let f = params.owners[k];
        return Some((occurrences[params.uses[k]].key.clone(), Some((params.names[f].clone(), params.signatures[f].2.clone()))));
    }
    graphed.into_iter().find(|&i| hit(i)).map(|i| (occurrences[i].key.clone(), None))
}

/// A sum index at a declaration or use, before global variable hovers.
pub(crate) fn local_at(a: &Analysis, at: usize) -> Option<String> {
    let (_, occurrences, _, params) = resolved_all(a);
    params.locals.iter().map(|&i| &occurrences[i]).find(|o| o.file == 0 && o.start <= at && at < o.end).map(|o| o.key.clone())
}

/// Locals visible for completion, with inner binders first.
pub(crate) fn local_names(a: &Analysis, at: usize) -> Vec<String> {
    let mut locals = locals(&a.doc, &a.elements, &a.calls, &|e| a.active(e));
    locals.sort_by_key(|l| l.body.1 - l.body.0);
    let mut names = Vec::new();
    for local in locals {
        if local.body.0 <= at && at <= local.body.1 && !names.contains(&local.key) { names.push(local.key); }
    }
    names
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

/// The definitions visible at character `at` of the document: by file (None for the document) and start, and the
/// files the compile order reached (others' definitions can't be judged). None when that can't be told.
pub(crate) fn visible_at(a: &Analysis, at: usize) -> Option<(Vec<(Option<String>, usize)>, Vec<Option<String>>)> {
    let files = files(a);
    let main = a.context.main.as_ref().and_then(|m| files.iter().position(|f| f.path == *m));
    let (visible, reached) = scope::visible_at(&files, 0, main, at)?;
    let path = |f: usize| (f != 0).then(|| files[f].path.clone());
    Some((visible.into_iter().map(|(f, start)| (path(f), start)).collect(), reached.iter().enumerate().filter(|(_, r)| **r).map(|(f, _)| path(f)).collect()))
}

/// The names a `solve(…)` or `substitute(…)` (`keyword`) of a clause can take, for the statement whose head (what's
/// before the marker) is `head` in characters of the document, with `listed` (the keys already given) left out.
/// `substitute` puts in variables' formulas: the variables named in the head and, through the ones listed, those
/// their formulas name, transitively. `solve` takes the names in the head that aren't functions or parameters,
/// undefined ones first. Each with the definition it means (by file, None for the document, and start).
pub(crate) fn clause_names(a: &Analysis, keyword: &str, head: (usize, usize), listed: &[String]) -> Vec<(String, Option<(Option<String>, usize)>)> {
    let (files, occurrences, graphed, params) = resolved_all(a);
    let parameter = |id: usize| params.uses.contains(&id) || params.locals.contains(&id) || graphed.contains(&id);
    // A definition occurrence of a variable.
    let variable = |id: usize| {
        let o = &occurrences[id];
        o.definition == Some(id) && !parameter(id) && files[o.file].definitions.iter().any(|d| d.start == o.start && !d.function)
    };
    let calls = |o: &Occurrence| files[o.file].doc.c[o.end..].iter().find(|c| !c.is_whitespace()) == Some(&'(');
    // The names used in a stretch of a file, with what they mean.
    let used = |file: usize, (start, end): (usize, usize)| -> Vec<(String, Option<usize>)> {
        occurrences.iter().enumerate()
            .filter(|(i, o)| o.file == file && start <= o.start && o.end <= end && o.definition != Some(*i) && !parameter(*i) && !calls(o))
            .map(|(_, o)| (o.key.clone(), o.definition.filter(|d| !parameter(*d))))
            .collect()
    };
    let mut out: Vec<(String, Option<usize>)> = Vec::new();
    let push = |key: String, definition: Option<usize>, out: &mut Vec<(String, Option<usize>)>| {
        if !out.iter().any(|(k, _)| *k == key) { out.push((key, definition)); }
    };
    if keyword == "substitute" {
        let mut queue: Vec<(String, Option<usize>)> = used(0, head).into_iter().filter(|(_, d)| d.is_some_and(variable)).collect();
        while let Some((key, definition)) = (!queue.is_empty()).then(|| queue.remove(0)) {
            if out.iter().any(|(k, _)| *k == key) { continue; }
            push(key.clone(), definition, &mut out);
            // A listed variable's formula is put in, so the variables it names can be too.
            let Some(id) = definition.filter(|_| listed.contains(&key)) else { continue };
            let o = &occurrences[id];
            let Some(d) = files[o.file].definitions.iter().find(|d| d.start == o.start) else { continue };
            queue.extend(used(o.file, (d.end, d.statement_end)).into_iter().filter(|(_, d)| d.is_some_and(variable)));
        }
    } else {
        let names = used(0, head);
        for (key, definition) in names.iter().filter(|(_, d)| d.is_none()).chain(names.iter().filter(|(_, d)| d.is_some())) {
            push(key.clone(), *definition, &mut out);
        }
    }
    let path = |f: usize| (f != 0).then(|| files[f].path.clone());
    out.into_iter()
        .filter(|(k, _)| !listed.contains(k))
        .map(|(k, d)| (k, d.map(|d| (path(occurrences[d].file), occurrences[d].start))))
        .collect()
}

/// [`resolved`], which occurrences a graph plots against, and functions' parameters.
fn resolved_all<'a>(a: &'a Analysis) -> (Vec<File<'a>>, Vec<Occurrence>, Vec<usize>, scope::Params) {
    let files = files(a);
    let main = a.context.main.as_ref().and_then(|m| files.iter().position(|f| f.path == *m));
    let implicit = crate::implicit_parameters(&a.doc.s).or_else(|| crate::implicit_parameters(&a.context.text)).unwrap_or_else(|| "no".into());
    let (mut occurrences, params) = scope::occurrences(&files, 0, main, implicit);
    let graphed = graphed(&files, &mut occurrences);
    (files, occurrences, graphed, params)
}

/// The document and the files around, the document first.
fn files<'a>(a: &'a Analysis) -> Vec<File<'a>> {
    let unit = |w: &str| a.is_custom_unit(w);
    let named = |n: Vec<Name>| n.into_iter().map(|n| (n.key, n.start, n.end)).collect::<Vec<_>>();
    let defs = definitions(&a.doc, &a.elements, &a.calls, &|e| a.active(e));
    let found = {
        // Quoted names of defined variables stand outside unit positions ("km" := 23, then "km" + 1).
        let defined = |w: &str| defs.iter().any(|d| d.base() == w);
        named(names(&a.doc, &a.elements, &a.calls, &|e| a.active(e), &unit, &defined, &|at| a.units_at(at)))
    };
    let mut files = vec![File {
        path: a.context.path.clone(),
        doc: &a.doc,
        definitions: defs,
        names: found,
        maths: maths(&a.elements),
        strings: strings(&a.calls),
        graphs: graphs(&a.doc, &a.calls),
        functions: curve_functions(&a.doc, &a.calls),
        locals: locals(&a.doc, &a.elements, &a.calls, &|e| a.active(e)),
    }];
    for part in a.around() {
        let Some(path) = &part.path else { continue };
        if *path == a.context.path {
            continue;
        }
        let settings = Analysis::new(&part.doc.s, a.context);
        let defs = definitions(&part.doc, &part.elements, &part.calls, &|e| part.active(e));
        let found = {
            let defined = |w: &str| defs.iter().any(|d| d.base() == w);
            named(names(&part.doc, &part.elements, &part.calls, &|e| part.active(e), &unit, &defined, &|at| settings.units_at(at)))
        };
        files.push(File {
            path: path.clone(),
            doc: &part.doc,
            definitions: defs,
            names: found,
            maths: maths(&part.elements),
            strings: strings(&part.calls),
            graphs: graphs(&part.doc, &part.calls),
            functions: curve_functions(&part.doc, &part.calls),
            locals: locals(&part.doc, &part.elements, &part.calls, &|e| part.active(e)),
        });
    }
    files
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
/// `#include` inlines a file, a `worksheet` starts empty (or with the outer definitions with `inherit: true`)
/// and its definitions end with it, and `clear` forgets. On a definition, that definition. The range covers
/// just the name. On one of the document's own units (`ü` of `units: (ü: auto)`), its key in `units: (…)`.
pub fn definition(document: &str, context: &Context, offset: u32) -> Option<Location> {
    let a = Analysis::new(document, context);
    // a shorthand from solveng's modules: where it's imported
    if let Some((s, e)) = a.bindings.shorthand_definition(a.doc.from16(offset as usize)) {
        return Some(Location { path: None, start: a.doc.to16(s) as u32, end: a.doc.to16(e) as u32 });
    }
    if a.bindings.at(&a.doc, offset) { return None; }
    let (files, occurrences) = resolved(&a);
    let Some(i) = at_offset(&a, &occurrences, offset) else { return crate::unit_refs::definition(&a, offset) };
    Some(location(&files, &occurrences[occurrences[i].definition?]))
}

/// The usages of the definition the solveng name at `offset` means (see [`definition`]), the definition included:
/// the names after it while it's visible, up to a redefinition, a `clear` or the end of its block, in the document
/// and in the files it includes or is included from. Units spelled like it (`5 V` next to a variable `V`) don't count.
/// On one of the document's own units, its keys and every use, prefixed ones (`mü`) and other units' values included.
pub fn usages(document: &str, context: &Context, offset: u32) -> Vec<Location> {
    let a = Analysis::new(document, context);
    let shorthand = a.bindings.shorthand_usages(a.doc.from16(offset as usize));
    if !shorthand.is_empty() {
        return shorthand.into_iter().map(|(s, e)| Location { path: None, start: a.doc.to16(s) as u32, end: a.doc.to16(e) as u32 }).collect();
    }
    if a.bindings.at(&a.doc, offset) { return Vec::new(); }
    let (files, occurrences) = resolved(&a);
    let Some(i) = at_offset(&a, &occurrences, offset) else { return crate::unit_refs::usages(&a, offset).unwrap_or_default() };
    let Some(id) = occurrences[i].definition else { return Vec::new() };
    occurrences.iter().filter(|o| o.definition == Some(id)).map(|o| location(&files, o)).collect()
}

/// The solveng name at `offset` that [`rename`] renames, as it's written there: its range and text.
pub fn rename_target(document: &str, context: &Context, offset: u32) -> Option<RenameTarget> {
    let a = Analysis::new(document, context);
    if let Some((s, e, name)) = a.bindings.shorthand_target(a.doc.from16(offset as usize)) {
        return Some(RenameTarget { start: a.doc.to16(s) as u32, end: a.doc.to16(e) as u32, name });
    }
    if a.bindings.at(&a.doc, offset) { return None; }
    let (files, occurrences) = resolved(&a);
    let Some(i) = at_offset(&a, &occurrences, offset) else { return crate::unit_refs::rename_target(&a, offset) };
    let o = &occurrences[i];
    o.definition?;
    let l = location(&files, o);
    Some(RenameTarget { start: l.start, end: l.end, name: a.doc.sub(o.start, o.end) })
}

/// Renames the solveng name at `offset` and its [`usages`] to `name`, typed as code (`R_load`, `Delta""r`) or
/// as math (`R_"load"`, `"tews"`). Each place gets the spelling it needs: code as typed, strings escaped, math
/// with longer names quoted (`"tews"`, `t_"half"`). Empty if there's nothing to rename or the input is empty.
pub fn rename(document: &str, context: &Context, offset: u32, name: &str) -> Vec<Edit> {
    let a = Analysis::new(document, context);
    // a shorthand gets an alias in its import, `s as test`
    if let Some(edits) = a.bindings.shorthand_rename(&a.doc, a.doc.from16(offset as usize), name.trim()) {
        return edits.into_iter().map(|(s, e, text)| Edit { path: None, start: a.doc.to16(s) as u32, end: a.doc.to16(e) as u32, text }).collect();
    }
    if a.bindings.at(&a.doc, offset) { return Vec::new(); }
    let (files, occurrences) = resolved(&a);
    let Some(i) = at_offset(&a, &occurrences, offset) else { return crate::unit_refs::rename(&a, offset, name).unwrap_or_default() };
    let Some(id) = occurrences[i].definition else { return Vec::new() };
    let raw: Vec<char> = name.trim().chars().collect();
    let (decoded, _) = crate::text::unescape(&raw);
    let decoded: String = decoded.into_iter().collect();
    let typed = decoded.as_str();
    let quoted = typed.replace("\"\"", "").contains('"');
    let candidate = if quoted {
        match math_name_of(&typed.chars().collect::<Vec<_>>()) {
            Some((key, _, _)) => key,
            None => return Vec::new(),
        }
    } else {
        crate::resolved_name(typed)
    };
    let Some(key) = solveng_names::for_rename(&candidate) else { return Vec::new() };
    let code = solveng_names::code(&key);
    occurrences
        .iter()
        .filter(|o| o.definition == Some(id))
        .map(|o| {
            let f = &files[o.file];
            let inside = |spans: &[(usize, usize)]| spans.iter().any(|&(s, e)| s <= o.start && o.end <= e);
            // Math quotes longer names, functions too: `"pytagoras"(a, b, c)`.
            let text = if inside(&f.maths) {
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
