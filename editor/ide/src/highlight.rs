//! Highlighting, regions, problems and hovers over a whole document.

use crate::bindings::{Meaning, Module};
use crate::calls::Role;
use crate::complete::{FUNCTIONS, defined, definitions};
use crate::text::{Doc, is_letter_or_digit};
use crate::tokens::{CONSTANTS, Tok, math_tokens as math_toks};
use crate::{Analysis, Context, Hover, Problem, Region, Token, TokenKind as K};

fn utf16(doc: &Doc, base: usize, toks: Vec<Tok>) -> impl Iterator<Item = Token> + '_ {
    toks.into_iter().map(move |(s, l, kind)| {
        let start = doc.to16(base + s);
        Token { start, length: doc.to16(base + s + l) - start, kind }
    })
}

/// Functions as defined, called by the user, or solveng's operations.
fn function_styles(a: &Analysis, tokens: &mut [Token], defs: &[crate::complete::Definition]) {
    let doc = &a.doc;
    for token in tokens.iter_mut().filter(|t| t.kind == K::FUNCTION || t.kind == K::VARIABLE && a.bindings.operation(doc.from16(t.start as usize)).is_some()) {
        let start = doc.from16(token.start as usize);
        let end = doc.from16((token.start + token.length) as usize);
        let mut tail = end;
        while doc.c.get(tail).is_some_and(|c| c.is_whitespace()) { tail += 1; }
        if doc.c.get(tail) == Some(&'(') {
            let mut depth = 0;
            while let Some(&c) = doc.c.get(tail) {
                tail += 1;
                if c == '(' { depth += 1; }
                if c == ')' { depth -= 1; }
                if depth == 0 { break; }
            }
        }
        while doc.c.get(tail).is_some_and(|c| c.is_whitespace()) { tail += 1; }
        let assigns = doc.c[tail..].starts_with(&[':', '=']) || doc.c.get(tail) == Some(&'≔');
        if defs.iter().any(|d| d.function && d.start == start) || assigns { continue; }
        let (text, _) = crate::text::unescape(&doc.c[start..end]);
        let name = crate::resolved_name(&text.iter().collect::<String>());
        let quoted = doc.c.get(start) == Some(&'"');
        // An operation binding, or a name Typst's math knows itself (sin, sqrt); solveng's own operations need
        // their binding in math.
        let op = a.bindings.operation(start).is_some_and(|op| FUNCTIONS.contains(&op))
            || !a.bindings.contains(start) && !quoted && FUNCTIONS.contains(&name.as_str()) && !crate::params::automatic_math(&name);
        token.kind = if op && !defs.iter().any(|d| d.function && d.base() == name) {
            K::BUILTIN_FUNCTION
        } else { K::FUNCTION_CALL };
    }
}

/// Tokens of one equation's body, on its own: the words of solveng's `units` module are its units.
pub fn math_tokens_of(math: &str) -> Vec<Token> {
    let context = Context::default();
    let a = Analysis::new(math, &context);
    let element = crate::elements::Element { start: 0, end: a.doc.c.len(), body_start: 0, body_end: a.doc.c.len() };
    let defs = definitions(&a.doc, &[element], &a.bindings, &|_| true);
    // On its own, no import says what's bound: the modules' names are their units and modifiers.
    let modifier = |w: &str| crate::clause::spelled(w).map(|m| crate::clause::Modifier { bound: true, ..m });
    let mut tokens: Vec<Token> = utf16(&a.doc, 0, math_toks(&a.doc.c, &crate::is_unit, &modifier)).collect();
    function_styles(&a, &mut tokens, &defs);
    tokens
}

/// Tokens of an equation's body, its unit bindings read where it is.
fn equation_tokens(a: &Analysis, e: &crate::elements::Element) -> Vec<Tok> {
    math_toks(&a.doc.c[e.body_start..e.body_end], &|w| !crate::bindings::needs_hash(w) && a.is_unit_at(e.body_start, w), &|w| a.bindings.modifier(e.body_start, w))
}

/// Tokens of the equations solveng reads (in a `worksheet` that evaluates them, or passed to one), over the whole
/// document: numbers, variables, functions, operators, results and their clauses.
pub fn math_tokens(document: &str, context: &Context) -> Vec<Token> {
    let a = Analysis::new(document, context);
    let defs = definitions(&a.doc, &a.elements, &a.bindings, &|e| a.active(e));
    let mut out = Vec::new();
    for e in a.elements.iter().filter(|e| a.active(e)) {
        out.extend(utf16(&a.doc, e.body_start, equation_tokens(&a, e)));
    }
    function_styles(&a, &mut out, &defs);
    reference_tokens(&a, &mut out);
    out.retain(|token| a.calculated_at(a.doc.from16(token.start as usize), a.doc.from16((token.start + token.length) as usize)).is_some());
    out.sort_by_key(|t| t.start);
    out
}

/// Highlight the names of explicit operation references without claiming their quotes or emitting overlapping tokens.
fn reference_tokens(a: &Analysis, out: &mut Vec<Token>) {
    for (start, end) in a.reference_strings() {
        let (inner_start, inner_end) = (a.doc.to16(start), a.doc.to16(end));
        let left = a.doc.to16(start.saturating_sub(1));
        let right = a.doc.to16(end + usize::from(a.doc.c.get(end) == Some(&'"')));
        out.retain(|t| t.start + t.length <= left || right <= t.start
            || inner_start <= t.start && t.start + t.length <= inner_end);
    }
    for (start, end, name) in a.bindings.operator_strings() {
        if a.evaluates_at(*start) && FUNCTIONS.contains(&name.as_str()) {
            out.push(Token { start: a.doc.to16(*start), length: a.doc.to16(*end) - a.doc.to16(*start), kind: K::BUILTIN_FUNCTION });
        }
    }
}

/// Tokens of the strings of solveng's calls: names and option values.
fn call_tokens(a: &Analysis) -> Vec<Token> {
    let mut out = Vec::new();
    for call in &a.calls {
        for t in &call.texts {
            let code = &a.doc.c[t.start..t.end];
            let text: String = code.iter().collect();
            let trimmed = text.trim();
            let whole = |kind: u8| {
                let lead = text.chars().take_while(|c| c.is_whitespace()).count();
                vec![(lead, trimmed.chars().count(), kind)]
            };
            let toks = match t.role {
                Role::Name if !trimmed.is_empty() => whole(K::VARIABLE),
                Role::Choice => {
                    let param = call.function.named.iter().find(|p| Some(&p.name) == t.param.as_ref());
                    if param.is_some_and(|p| p.values.iter().any(|v| v == trimmed)) { whole(K::KEYWORD) } else { Vec::new() }
                }
                Role::Mark if ["crossings", "roots", "extrema"].contains(&trimmed) => whole(K::KEYWORD),
                Role::Name | Role::Mark => Vec::new(),
            };
            out.extend(utf16(&a.doc, t.start, toks.into_iter().filter(|t| t.1 > 0).collect()));
        }
    }
    out
}

/// Every solveng token of a document, sorted: calculated math, the bindings it uses and the strings of solveng's
/// calls.
pub fn document_tokens(document: &str, context: &Context) -> Vec<Token> {
    let a = Analysis::new(document, context);
    let defs = definitions(&a.doc, &a.elements, &a.bindings, &|e| a.active(e));
    let mut out = Vec::new();
    for e in a.elements.iter().filter(|e| a.active(e)) {
        out.extend(utf16(&a.doc, e.body_start, equation_tokens(&a, e)));
    }
    out.extend(call_tokens(&a));
    // Parameters stand out, in their function and graphs.
    let params: Vec<(u32, u32)> = crate::navigate::parameter_ranges(&a).into_iter().map(|(s, e)| (a.doc.to16(s), a.doc.to16(e))).collect();
    for t in out.iter_mut().filter(|t| t.kind == K::VARIABLE) {
        if params.iter().any(|&(ps, pe)| ps <= t.start && t.start + t.length <= pe) {
            t.kind = K::PARAMETER;
        }
    }
    for token in &mut out {
        if a.bindings.contains(a.doc.from16(token.start as usize)) { token.kind = K::VARIABLE; }
    }
    function_styles(&a, &mut out, &defs);
    // Units, constants, operations and modifiers where calculated math uses their bindings, with the `#` before
    // them gray.
    for (start, end, meaning) in a.bindings.solveng_uses() {
        if a.calculated_at(start, end).is_none() { continue; }
        let kind = match meaning {
            Meaning::Operation(op) if FUNCTIONS.contains(&op.as_str()) => K::BUILTIN_FUNCTION,
            Meaning::Operation(_) => continue,
            Meaning::Constant { .. } => K::CONSTANT,
            Meaning::Modifier(_) => K::KEYWORD,
            _ => K::UNIT,
        };
        let from = a.doc.to16(start);
        out.retain(|t| t.start != from);
        out.push(Token { start: from, length: a.doc.to16(end) - from, kind });
        if start > 0 && a.doc.c[start - 1] == '#' {
            out.push(Token { start: a.doc.to16(start - 1), length: 1, kind: K::UNIT_REFERENCE });
        }
    }
    // Names imported from solveng's modules are units, constants, operations or modifiers where they're imported too; a
    // renamed one's module name isn't a name in the document, so it's plain text.
    for ((start, end), meaning, original) in a.bindings.shorthand_imports() {
        let kind = match meaning {
            Meaning::Operation(_) => K::BUILTIN_FUNCTION,
            Meaning::Constant { .. } => K::CONSTANT,
            Meaning::Modifier(_) => K::KEYWORD,
            _ => K::UNIT,
        };
        let spans = std::iter::once((start, end, kind))
            .chain(original.map(|(s, e)| (s, e, K::FUNCTION_CALL)));
        for (s, e, kind) in spans {
            let (from, to) = (a.doc.to16(s), a.doc.to16(e));
            out.retain(|t| t.start + t.length <= from || to <= t.start);
            out.push(Token { start: from, length: to - from, kind });
        }
    }
    reference_tokens(&a, &mut out);
    out.sort_by_key(|t| t.start);
    out
}

/// Where solveng reads the document: calculated math, the strings of its calls, and the calls' arguments
/// (`kind: "call"`), where its options complete. A language server next to tinymist answers there only.
pub fn regions(document: &str, context: &Context) -> Vec<Region> {
    let a = Analysis::new(document, context);
    let r = |s: usize, e: usize, kind| Region { start: a.doc.to16(s), end: a.doc.to16(e), kind };
    let mut out: Vec<Region> = Vec::new();
    for e in a.elements.iter().filter(|e| a.active(e)) {
        out.push(r(e.body_start, e.body_end, "math"));
    }
    for call in &a.calls {
        out.push(r(call.args_start, call.args_end, "call"));
        out.extend(call.texts.iter().map(|t| r(t.start, t.end, "string")));
    }
    out.sort_by_key(|r| r.start);
    out
}

/// Problems: the errors the last compile reported (see [`crate::Report::errors`]) where they still apply, and source
/// inspections: redefinitions, binding references and worksheet scope surprises.
pub fn diagnostics(document: &str, context: &Context) -> Vec<Problem> {
    let a = Analysis::new(document, context);
    let mut out = Vec::new();
    // A name assigned again in its scope: the last assignment wins, so the earlier one counts only until there.
    let defs = crate::complete::definitions(&a.doc, &a.elements, &a.bindings, &|e| a.active(e));
    for (earlier, later) in crate::navigate::replaced(&a) {
        let Some(d) = defs.iter().find(|d| d.start == earlier) else { continue };
        let line = a.doc.c[..later].iter().filter(|&&ch| ch == '\n').count() + 1;
        let what = if d.function { "function" } else { "variable" };
        out.push(Problem {
            start: a.doc.to16(d.start),
            end: a.doc.to16(d.start + d.written),
            message: format!("the {what} '{}' is assigned again on line {line} – the last assignment wins, so this one only counts until there", d.base()),
            severity: "warning",
        });
    }
    out.extend(crate::bindings::problems(&a));
    out.extend(crate::scope::export_warnings(&a.doc));
    let mut reported = crate::errors::reported(&a);
    reported.append(&mut out);
    reported
}

/// A function's docs as its hover: the summary and every parameter, with its types, default and description.
/// A modifier's docs, by its name in the `modifiers` module.
pub(crate) fn modifier_docs(name: &str) -> String {
    match crate::params::function(name) {
        Some(f) => format!("{}\n\nA modifier after `=` or `->`, from `#import modifiers: {name}`.", function_docs(f)),
        None => {
            let what = crate::clause::KEYWORDS.iter().find(|(k, _)| *k == name).map_or("", |(_, d)| d);
            format!("`{name}` (solveng)\n\nA modifier after `=` or `->`, from `#import modifiers: {name}`: {what}.")
        }
    }
}

pub(crate) fn function_docs(f: &crate::params::Function) -> String {
    let mut out = format!("`{}` (solveng)\n\n{}", f.name, f.doc);
    let params: Vec<String> = f.positional.iter().chain(&f.named).map(|p| {
        let name = if p.variadic { format!("..{}", p.name) } else { p.name.clone() };
        let types = if p.types.is_empty() { String::new() } else { format!(" ({})", p.types) };
        let default = if p.default.is_empty() { String::new() } else { format!(" = `{}`", p.default) };
        let doc = if p.doc.is_empty() { String::new() } else { format!(": {}", p.doc) };
        format!("- `{name}`{types}{default}{doc}")
    }).collect();
    if !params.is_empty() {
        out += &format!("\n\n### Parameters\n\n{}", params.join("\n"));
    }
    out
}

/// What the word at `offset` is: a defined name with its value (or formula) and what it depends on, a unit,
/// constant or operation binding, solveng's modifiers and functions, or a parameter of solveng's functions with its
/// docs.
pub fn hover(document: &str, context: &Context, offset: u32) -> Option<Hover> {
    let found = hover_at(document, context, offset);
    // A defined name, where it's defined or used: also what its formula depends on.
    let Some(target) = crate::rename_target(document, context, offset) else { return found };
    let Some(definition) = crate::definition(document, context, offset) else { return found };
    let a = Analysis::new(document, context);
    let name = crate::depends::key(&target.name);
    // not a parameter or a sum's index, though it may share a defined name
    let at = a.doc.from16(offset as usize);
    if crate::complete::defined_item(&a, &name).is_none() || crate::navigate::parameter_at(&a, at).is_some() || crate::navigate::local_at(&a, at).is_some() {
        return found;
    }
    let markdown = match found {
        Some(h) if (h.start, h.end) == (target.start, target.end) => h.markdown,
        _ => defined(&a, &name)?,
    };
    let markdown = match crate::depends::section(&a, document, definition, &name) {
        Some(section) => format!("{markdown}\n\n{section}"),
        None => markdown,
    };
    Some(Hover { start: target.start, end: target.end, markdown })
}

/// The docs of one of solveng's modules: the leading `///` block of its file.
fn module_docs(module: Module) -> String {
    module.source().lines().map_while(|l| l.strip_prefix("///")).map(|l| l.strip_prefix(' ').unwrap_or(l)).collect::<Vec<_>>().join("\n")
}

/// Hovers of solveng's modules in their imports, of the names imported from them, and of the document's own unit,
/// constant and operation bindings where math uses them.
fn binding_hover(a: &Analysis, at: usize) -> Option<Hover> {
    let hover = |s: usize, e: usize, markdown: String| Hover { start: a.doc.to16(s), end: a.doc.to16(e), markdown };
    if let Some((s, e, module)) = a.bindings.module_at(at) {
        let docs = module_docs(module);
        let docs = if docs.is_empty() { String::new() } else { format!("\n\n{docs}") };
        return Some(hover(s, e, format!("`{}` module of solveng{docs}", module.name())));
    }
    if let Some((s, e, name, original, module, meaning)) = a.bindings.shorthand_info(at) {
        let module = module.name();
        let title = if name == original { format!("`{name}`: shorthand from solveng's `{module}`") }
            else { format!("`{name}`: shorthand for `{original}` from solveng's `{module}`") };
        return Some(hover(s, e, format!("{title}\n\n{}\n\nSee `{module}` in the import for how shorthands work.", meaning.describe())));
    }
    // A binding of the document's own, where calculated math uses it.
    let (s, e, name, meaning) = a.bindings.use_at(at)?;
    a.calculated_at(s, e)?;
    Some(hover(s, e, format!("`{name}`: {}", meaning.describe())))
}

fn hover_at(document: &str, context: &Context, offset: u32) -> Option<Hover> {
    let a = Analysis::new(document, context);
    let at = a.doc.from16(offset as usize);
    if let Some(h) = binding_hover(&a, at) { return Some(h); }
    if let Some((start, end, name)) = a.bindings.operator_strings().iter().find(|(s, e, _)| *s <= at && at <= *e)
        && FUNCTIONS.contains(&name.as_str()) {
            let markdown = crate::params::builtin(name).map(function_docs).unwrap_or_else(|| format!("`{name}()` built-in function"));
            return Some(Hover { start: a.doc.to16(*start), end: a.doc.to16(*end), markdown });
        }
    // Typst's language server owns the other bindings.
    if a.bindings.at(&a.doc, offset) { return None; }
    let c = &a.doc.c;
    let part = |ch: char| is_letter_or_digit(ch) || ch == '_';
    let mut s = at;
    while s > 0 && part(c[s - 1]) {
        s -= 1;
    }
    let mut e = at;
    while e < c.len() && part(c[e]) {
        e += 1;
    }
    // The functions themselves, by their whole identifier (worksheet, not solve), in code anywhere.
    if let Some((start, end, name)) = identifier(&a.doc, at)
        && let Some(f) = crate::params::function(&name)
    {
        return Some(Hover { start: a.doc.to16(start), end: a.doc.to16(end), markdown: function_docs(f) });
    }
    if s == e {
        return None;
    }
    let tokens = document_tokens(document, context);
    let start = a.doc.to16(s);
    // A numeric coefficient and the symbol following it have separate token boundaries.
    if let Some(number) = tokens.iter().find(|t| t.kind == K::NUMBER && t.start <= start && start < t.start + t.length
        && t.start + t.length < a.doc.to16(e)) {
        let end = a.doc.from16((number.start + number.length) as usize);
        if at >= end { s = end; } else { e = end; }
    }
    let word = a.doc.sub(s, e);
    let hover = |markdown: String| Some(Hover { start: a.doc.to16(s), end: a.doc.to16(e), markdown });
    // Parameter names of solveng's calls.
    if let Some(call) = a.calls.iter().rev().find(|c| c.args_start <= at && at <= c.args_end)
        && let Some(n) = call.named.iter().find(|n| n.name_start <= at && at <= n.name_end) {
            let p = call.function.named.iter().find(|p| p.name == n.name)?;
            let default = if p.default.is_empty() { String::new() } else { format!(" = {}", p.default) };
            return hover(format!("`{}` ({}){default}\n\n{}", p.name, p.types, p.doc));
        }
    let inside = a.calculated_at(s, e).is_some() || a.calls.iter().any(|call| call.texts.iter().any(|t| t.start <= s && e <= t.end));
    if !inside {
        return None;
    }
    // solveng's words: the modifiers of a clause (their bindings' hovers), any as a type
    let (start, end) = (a.doc.to16(s), a.doc.to16(e));
    let kind = tokens.iter().find(|t| t.start <= start && end <= t.start + t.length).map(|t| t.kind);
    if kind == Some(K::KEYWORD)
        && let Some(m) = a.bindings.modifier(s, &word).filter(|m| m.bound) {
            return hover(modifier_docs(&m.name));
        }
    if kind == Some(K::EVALUATE) || word.is_empty() {
        return None;
    }
    if word == "any" && a.doc.sub(0, s).trim_end().ends_with(':') {
        return hover("`any` type: a parameter that takes every unit; the result's unit follows what's passed. `1` is the type of a plain number.".to_string());
    }
    if let Some((name,description)) = crate::navigate::local_at(&a, at) { return hover(format!("`{name}` {description}")); }
    if let Some((name, owner)) = crate::navigate::parameter_at(&a, at) {
        let Some((function, written)) = owner else { return hover(format!("`{word}` what the graph plots against, its x axis")) };
        // Its type as compiled (inferred ones too), else as written.
        let typed = |list: &[String]| list.iter().map(|p| p.trim().to_string()).find(|p| p.split(':').next().map(str::trim) == Some(name.as_str()));
        let reported = context.defined.iter().find(|d| d.name == function && d.kind == "function").and_then(|d| {
            let sig = d.value.split_once(" = ")?.0;
            let list = sig.strip_prefix('(')?.rsplit_once(')')?.0;
            typed(&list.split(", ").map(String::from).collect::<Vec<_>>())
        });
        let shown = reported.or_else(|| typed(&written)).unwrap_or(name);
        return hover(format!("`{shown}` parameter of `{function}`"));
    }
    if let Some(d) = defined(&a, &word) {
        return hover(d);
    }
    // Operations Typst's math knows itself (sin, sqrt) and the explicit references of solveng's.
    let builtin = if word == "sum_" { "sum".to_string() } else { word.clone() };
    let explicit = a.bindings.operator_strings().iter().find(|(s, e, _)| *s <= at && at <= *e).map(|(_, _, n)| n.clone());
    let native = (FUNCTIONS.contains(&builtin.as_str()) && !crate::params::automatic_math(&builtin)).then(|| builtin.clone());
    if let Some(op) = explicit.or(native) {
        return hover(crate::params::builtin(&op).map(function_docs).unwrap_or_else(|| format!("`{op}()` built-in function")));
    }
    CONSTANTS.iter().find(|(n, _)| *n == word).and_then(|(_, d)| hover(format!("`{word}` {d}")))
}

/// The typst identifier in code at character `at` (or just before it), not a named argument's: its range and text.
fn identifier(doc: &Doc, at: usize) -> Option<(usize, usize, String)> {
    let root = typst_syntax::parse(&doc.s);
    let node = typst_syntax::LinkedNode::new(&root);
    let byte = doc.s.char_indices().nth(at).map_or(doc.s.len(), |(b, _)| b);
    let leaf = [typst_syntax::Side::After, typst_syntax::Side::Before]
        .into_iter()
        .filter_map(|side| node.leaf_at(byte, side))
        .find(|l| l.kind() == typst_syntax::SyntaxKind::Ident && l.parent_kind() != Some(typst_syntax::SyntaxKind::Named))?;
    let (start, end) = (doc.of_byte(leaf.offset()), doc.of_byte(leaf.offset() + leaf.len()));
    Some((start, end, leaf.get().leaf_text().to_string()))
}
