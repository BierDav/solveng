//! Highlighting, regions, problems and hovers over a whole document.

use crate::calls::Role;
use crate::complete::{FUNCTIONS, defined, definitions};
use crate::text::{Doc, chars, is_letter_or_digit};
use crate::tokens::{CONSTANTS, Tok, code_tokens, visible_code_tokens, formula_tokens, math_tokens as math_toks};
use crate::units::{self, Custom};
use crate::{Analysis, Context, Hover, Problem, Region, Token, TokenKind as K};

fn utf16(doc: &Doc, base: usize, toks: Vec<Tok>) -> impl Iterator<Item = Token> + '_ {
    toks.into_iter().map(move |(s, l, kind)| {
        let start = doc.to16(base + s);
        Token { start, length: doc.to16(base + s + l) - start, kind }
    })
}

fn local(text: &str, toks: Vec<Tok>, math: bool) -> Vec<Token> {
    let doc = Doc::new(text);
    let element = crate::elements::Element { start: 0, end: doc.c.len(), body_start: 0, body_end: doc.c.len(), tag: crate::TAG.into(), math, block: true };
    let defs = definitions(&doc, &[element], &[], &|_| true);
    let mut tokens = utf16(&doc, 0, toks).collect::<Vec<_>>();
    let context = Context::default();
    let a = Analysis::new(text, &context);
    function_styles(&a, &mut tokens, &defs);
    tokens
}

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
        let op = a.bindings.operation(start).is_some_and(|op| FUNCTIONS.contains(&op)) && (!a.bindings.imported_at(start) || a.builtin_functions_at(start))
            || !a.bindings.contains(start) && !quoted && FUNCTIONS.contains(&name.as_str()) && {
                // In math Typst needs the name: its own math operators, or a shorthand imported from `functions`.
                let math = a.elements.iter().any(|e| e.math && e.body_start <= start && start < e.body_end);
                if math { !crate::params::automatic_math(&name) } else { a.builtin_functions_at(start) }
            };
        token.kind = if op && !defs.iter().any(|d| d.function && d.base() == name) {
            K::BUILTIN_FUNCTION
        } else { K::FUNCTION_CALL };
    }
}

/// Tokens of solveng code with kinds from [`K`]: numbers and their units, variables, functions, `@`-inlined
/// formulas, relations and `->`, comments.
pub fn tokens(code: &str) -> Vec<Token> {
    local(code, code_tokens(&chars(code), &|_| false), false)
}

/// solveng code as documentation shows it, with its tokens: powers and indices written out (`A^2` is A², `m^-1` m⁻¹,
/// `x_2` x₂, where every character has a subscript), spaces inside brackets and before commas left out
/// (`f ( x , y )` from math is f(x, y)).
/// `units` are the document's own unit names, read as units after a number like the built-in ones.
pub fn pretty(code: &str, units: &[String]) -> (String, Vec<Token>) {
    const SUP: &str = "⁰¹²³⁴⁵⁶⁷⁸⁹";
    const SUB_FROM: &str = "0123456789aehijklmnoprstuvx";
    const SUB: &str = "₀₁₂₃₄₅₆₇₈₉ₐₑₕᵢⱼₖₗₘₙₒₚᵣₛₜᵤᵥₓ";
    let sup = |d: char| SUP.chars().nth(d.to_digit(10).unwrap_or(0) as usize).unwrap_or(d);
    let sub = |ch: char| SUB_FROM.chars().position(|f| f == ch).and_then(|i| SUB.chars().nth(i));
    let c = chars(code);
    let toks = code_tokens(&c, &|w| units.iter().any(|u| u == w));
    // calls lose the space before their parenthesis: f ( x ) is f(x)
    let calls: Vec<usize> = toks.iter().filter(|t| t.2 == K::FUNCTION).map(|t| t.0 + t.1).collect();
    let mut out: Vec<char> = Vec::new();
    // where each character of the code went
    let mut at = vec![0; c.len() + 1];
    let mut i = 0;
    while i < c.len() {
        at[i] = out.len();
        let mut end = None;
        if c[i] == '^' && i > 0 {
            // ^2, ^-1, ^(2), ^(-1)
            let paren = c.get(i + 1) == Some(&'(');
            let mut j = i + 1 + paren as usize;
            let neg = matches!(c.get(j), Some('-' | '−'));
            j += neg as usize;
            let digits = j;
            while j < c.len() && c[j].is_ascii_digit() {
                j += 1;
            }
            if j > digits && (!paren || c.get(j) == Some(&')')) {
                if neg {
                    out.push('⁻');
                }
                out.extend(c[digits..j].iter().map(|&d| sup(d)));
                end = Some(j + paren as usize);
            }
        } else if c[i] == '_' && i > 0 && is_letter_or_digit(c[i - 1]) {
            let mut j = i + 1;
            while j < c.len() && is_letter_or_digit(c[j]) {
                j += 1;
            }
            if j > i + 1 && c[i + 1..j].iter().all(|&ch| sub(ch).is_some()) {
                out.extend(c[i + 1..j].iter().filter_map(|&ch| sub(ch)));
                end = Some(j);
            }
        } else if c[i].is_whitespace() {
            let next = c[i..].iter().find(|ch| !ch.is_whitespace());
            let call = next == Some(&'(') && calls.iter().any(|&e| e <= i && c[e..i].iter().all(|ch| ch.is_whitespace()));
            if matches!(next, Some(')' | ',')) || out.last() == Some(&'(') || call {
                end = Some(i + 1);
            }
        }
        match end {
            Some(e) => {
                // what's inside starts where the span does, so the exponent's token covers the ²
                for k in i + 1..e {
                    at[k] = at[i];
                }
                i = e;
            }
            None => {
                out.push(c[i]);
                i += 1;
            }
        }
    }
    at[c.len()] = out.len();
    let text: String = out.iter().collect();
    let moved = toks.into_iter().map(|(s, n, k)| (at[s], at[s + n] - at[s], k)).filter(|t| t.1 > 0).collect();
    let tokens = local(&text, moved, false);
    (text, tokens)
}

/// [`tokens`], with the document's own unit names (`bar`) read as units after a number.
pub fn tokens_with_units(code: &str, units: &[String]) -> Vec<Token> {
    local(code, code_tokens(&chars(code), &|w| units.iter().any(|u| u == w)), false)
}

/// Tokens of one equation's body.
pub fn math_tokens_of(math: &str) -> Vec<Token> {
    local(math, math_toks(&chars(math), &crate::is_unit, &|_| false), true)
}

/// Tokens of the equations solveng reads (in a `worksheet` that evaluates them, or passed to one),
/// over the whole document, coloured like solveng code: numbers, quoted units, variables, functions.
pub fn math_tokens(document: &str, context: &Context) -> Vec<Token> {
    let a = Analysis::new(document, context);
    let defs = definitions(&a.doc, &a.elements, &a.calls, &|e| a.active(e));
    let defined = |w: &str| defs.iter().any(|d| d.base() == w);
    let mut out = Vec::new();
    for e in a.elements.iter().filter(|e| e.math && a.active(e)) {
        let units = a.units_at(e.body_start);
        out.extend(utf16(&a.doc, e.body_start, math_toks(&a.doc.c[e.body_start..e.body_end], &|w| units::visible(w, &units), &defined)));
    }
    out.extend(call_tokens(&a).into_iter().filter(|token| a.elements.iter().any(|e|
        e.math && a.active(e) && a.doc.to16(e.body_start) <= token.start && token.start + token.length <= a.doc.to16(e.body_end))));
    function_styles(&a, &mut out, &defs);
    reference_tokens(&a, &mut out);
    out.retain(|token| a.elements.iter().any(|e| e.math && a.active(e)
        && a.doc.to16(e.body_start) <= token.start && token.start + token.length <= a.doc.to16(e.body_end)));
    out.sort_by_key(|t| t.start);
    out
}

/// Highlight constructor string contents without claiming their quotes or emitting overlapping tokens.
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

/// A unit string's tokens: every word is a unit, as after `->`, if it's one in effect there.
fn unit_tokens(code: &[char], custom: &[Custom]) -> Vec<Tok> {
    let mut with: Vec<char> = "-> ".chars().collect();
    with.extend_from_slice(code);
    code_tokens(&with, &|w| custom.iter().any(|u| u.name == w))
        .into_iter()
        .filter(|t| t.0 >= 3)
        .map(|(s, l, k)| {
            let word: String = with[s..s + l].iter().collect();
            let unit = k == K::UNIT && word.chars().all(|c| c.is_ascii_digit()) || units::known(&word, custom);
            (s - 3, l, if k == K::NUMBER { K::NUMBER } else if unit { K::UNIT } else { K::VARIABLE })
        })
        .collect()
}

/// Tokens of what the strings of solveng's calls hold: code lines in `#worksheet("…")`, curves in `#graph` and
/// `#lambdify`, units in `unit:`, names in `x:`, the values of options like `steps:`.
fn call_tokens(a: &Analysis) -> Vec<Token> {
    let unit = |w: &str| a.is_custom_unit(w);
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
            let mut settings = a.units_at(t.start);
            // a worksheet's units may use the ones it defines next to them
            for (name, value) in call.args.iter().filter(|(n, _)| n.as_deref() == Some("units")) {
                units::custom_units(&format!("{}: {value}", name.as_deref().unwrap_or_default()), &mut settings);
            }
            let toks = match t.role {
                // A graph's curves are formulas.
                Role::Code if call.is_curve(t) => formula_tokens(code, &unit),
                Role::Code => visible_code_tokens(code, &unit, &|w| units::visible(w, &settings)),
                Role::Unit => unit_tokens(code, &settings),
                Role::Name if !trimmed.is_empty() => whole(K::VARIABLE),
                Role::Choice => {
                    let param = call.function.named.iter().find(|p| Some(&p.name) == t.param.as_ref());
                    if param.is_some_and(|p| p.values.iter().any(|v| v == trimmed)) { whole(K::KEYWORD) } else { Vec::new() }
                }
                Role::Mark if ["crossings", "roots", "extrema"].contains(&trimmed) => whole(K::KEYWORD),
                Role::Mark => code_tokens(code, &unit),
                _ => Vec::new(),
            };
            out.extend(utf16(&a.doc, t.start, toks.into_iter().filter(|t| t.1 > 0).collect()));
        }
    }
    out
}

/// Every solveng token of a document, sorted: calculated raw code, calculated math and the strings of solveng's
/// calls. Editors without their own raw-block injection can take just this.
pub fn document_tokens(document: &str, context: &Context) -> Vec<Token> {
    let a = Analysis::new(document, context);
    let unit = |w: &str| a.is_custom_unit(w);
    let defs = definitions(&a.doc, &a.elements, &a.calls, &|e| a.shown(e));
    let defined = |w: &str| defs.iter().any(|d| d.base() == w);
    let mut out = Vec::new();
    for e in a.elements.iter().filter(|e| a.shown(e)) {
        let body = &a.doc.c[e.body_start..e.body_end];
        let units = a.units_at(e.body_start);
        let toks = if e.math { math_toks(body, &|w| units::visible(w, &units), &defined) } else {
            let units = a.units_at(e.body_start);
            visible_code_tokens(body, &unit, &|w| units::visible(w, &units))
        };
        out.extend(utf16(&a.doc, e.body_start, toks));
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
    for (start, end, name, imported) in a.bindings.operation_uses() {
        if a.elements.iter().any(|e| e.math && a.active(e) && e.body_start <= start && end <= e.body_end)
            && FUNCTIONS.contains(&name) && (!imported || a.builtin_functions_at(start)) {
            let from = a.doc.to16(start);
            out.retain(|t| t.start != from);
            out.push(Token { start: from, length: a.doc.to16(end) - from, kind: K::BUILTIN_FUNCTION });
        }
    }
    for (start, end, unit, imported) in a.bindings.unit_uses() {
        let units = a.units_at(start);
        let exists = if imported { units::visible(unit, &units) } else { units::known(unit, &units) };
        if exists && a.elements.iter().any(|e| e.math && a.active(e) && e.body_start <= start && end <= e.body_end) {
            let from = a.doc.to16(start);
            out.retain(|t| t.start != from);
            out.push(Token { start: from, length: a.doc.to16(end) - from, kind: K::UNIT });
            if start > 0 && a.doc.c[start - 1] == '#' {
                out.push(Token { start: a.doc.to16(start - 1), length: 1, kind: K::UNIT_REFERENCE });
            }
        }
    }
    reference_tokens(&a, &mut out);
    out.sort_by_key(|t| t.start);
    out
}

/// Where solveng reads the document: calculated code and math, the strings of its calls, and the calls'
/// arguments (`kind: "call"`), where its options complete. A language server next to tinymist answers there only.
pub fn regions(document: &str, context: &Context) -> Vec<Region> {
    let a = Analysis::new(document, context);
    let r = |s: usize, e: usize, kind| Region { start: a.doc.to16(s), end: a.doc.to16(e), kind };
    let mut out: Vec<Region> = Vec::new();
    for e in a.elements.iter().filter(|e| a.shown(e)) {
        out.push(r(e.body_start, e.body_end, if e.math { "math" } else { "code" }));
    }
    for call in &a.calls {
        out.push(r(call.args_start, call.args_end, "call"));
        out.extend(call.texts.iter().map(|t| r(t.start, t.end, "string")));
    }
    out.sort_by_key(|r| r.start);
    out
}

/// Problems: the errors the last compile reported (see [`crate::Report::errors`]) where they still apply, and source
/// inspections, unknown units, redefinitions and worksheet scope surprises.
pub fn diagnostics(document: &str, context: &Context) -> Vec<Problem> {
    let a = Analysis::new(document, context);
    let mut out = Vec::new();
    let mut report = |base: usize, expr: &str| {
        for (s, e, word) in units::unknown_words(expr, &a.units) {
            out.push(Problem {
                start: a.doc.to16(base + s),
                end: a.doc.to16(base + e),
                message: format!("unknown unit '{word}' – define it with the units option, worksheet(units: ({word}: \"…\"))"),
                severity: "warning",
            });
        }
    };
    let code = |start: usize, end: usize, report: &mut dyn FnMut(usize, &str)| {
        // Units after `->` and in brackets, line by line, comments left out.
        let c = &a.doc.c;
        let mut i = start;
        while i < end {
            let line_end = (i..end).find(|&j| c[j] == '\n').unwrap_or(end);
            let line: String = c[i..line_end].iter().collect();
            let line = crate::text::before(&line, "//");
            for (k, part) in line.split(';').scan(0usize, |at, p| {
                let s = *at;
                *at += p.chars().count() + 1;
                Some((s, p))
            }) {
                // the units of a clause, its keywords left out: t = [h], digits(2)
                let pc: Vec<char> = part.chars().collect();
                // (a bracketed unit is one whatever it says: [furlong])
                let loose = |w: &str| a.is_custom_unit(w);
                let bracketed: String = pc.iter().map(|ch| if *ch == '[' || *ch == ']' { ' ' } else { *ch }).collect();
                let marked = crate::clause::marker(&pc, &loose).or_else(|| crate::clause::marker(&chars(&bracketed), &|_| true).filter(|_| part.contains('[')));
                if let Some((_, e, _)) = marked {
                    let mut at = e;
                    for item in crate::clause::split_top(&crate::text::sub(&pc, e, pc.len())) {
                        let word: String = item.trim().chars().take_while(|ch| ch.is_alphabetic()).collect();
                        if !crate::clause::is_keyword(&word) {
                            let open = item.find('[').map_or(0, |b| item[..b].chars().count() + 1);
                            report(i + k + at + open, &item.replace(['[', ']'], " ")[item.find('[').map_or(0, |b| b + 1)..]);
                        }
                        at += item.chars().count() + 1;
                    }
                }
                let pc: Vec<char> = part.chars().collect();
                let mut j = 0;
                while j < pc.len() {
                    if pc[j] == '[' {
                        let close = (j..pc.len()).find(|&x| pc[x] == ']').unwrap_or(pc.len());
                        let inner: String = pc[j + 1..close].iter().collect();
                        // The value's number comes first: [5 V].
                        let skip = inner.chars().take_while(|ch| !ch.is_alphabetic() && !"°%‰µΩ".contains(*ch)).count();
                        if skip > 0 {
                            report(i + k + j + 1 + skip, &inner.chars().skip(skip).collect::<String>());
                        }
                        j = close;
                    }
                    j += 1;
                }
            }

            i = line_end + 1;
        }
    };
    for e in a.elements.iter().filter(|e| a.active(e)) {
        if e.math {
            // A quoted unit after a number: 30 "mA"; "k"Omega joins the prefix to what follows.
            let c = &a.doc.c;
            let mut i = e.body_start;
            while i < e.body_end {
                if c[i] == '"' {
                    let close = (i + 1..e.body_end).find(|&j| c[j] == '"').unwrap_or(e.body_end);
                    let before = c[e.body_start..i].iter().rev().find(|ch| !ch.is_whitespace());
                    let next = c.get(close + 1).copied().unwrap_or(' ');
                    if before.is_some_and(|b| b.is_ascii_digit()) && !next.is_alphabetic()
                        && solveng_names::resolve(&a.doc.sub(i, (close + 1).min(e.body_end))).is_ok() {
                        let text: String = c[i + 1..close].iter().collect();
                        report(i + 1, &text);
                    }
                    i = close + 1;
                } else {
                    i += 1;
                }
            }
        } else {
            code(e.body_start, e.body_end, &mut report);
        }
    }
    for call in &a.calls {
        for t in &call.texts {
            match t.role {
                Role::Unit if call.function.name != "unit" => report(t.start, &a.doc.sub(t.start, t.end)),
                Role::Code => code(t.start, t.end, &mut report),
                _ => {}
            }
        }
    }
    // A name assigned again in its scope: the last assignment wins, so the earlier one counts only until there.
    let defs = crate::complete::definitions(&a.doc, &a.elements, &a.calls, &|e| a.active(e));
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
/// solveng's functions and constants, or a parameter of solveng's functions with its docs.
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

fn hover_at(document: &str, context: &Context, offset: u32) -> Option<Hover> {
    let a = Analysis::new(document, context);
    let at = a.doc.from16(offset as usize);
    if let Some((name, unit, start, end)) = a.bindings.unit_shortcut(at) {
        let settings = a.unit_completion_context(at);
        if a.evaluates_at(at) {
            let detail = units::describe(unit, &settings).unwrap_or_else(|| format!("`{unit}` unit"));
            return Some(Hover { start: a.doc.to16(start), end: a.doc.to16(end),
                markdown: format!("`{name}` unit shortcut for `{unit}`\n\nDetected from `unit(…)`: an explicit unit reference. The unit is resolved through the active worksheet overlays at each use.\n\n{detail}") });
        }
    }
    if let Some((start, end, name)) = a.bindings.operator_strings().iter().find(|(s, e, _)| *s <= at && at <= *e) {
        if a.evaluates_at(at) && FUNCTIONS.contains(&name.as_str()) {
            let markdown = crate::params::builtin(name).map(function_docs).unwrap_or_else(|| format!("`{name}()` built-in function"));
            return Some(Hover { start: a.doc.to16(*start), end: a.doc.to16(*end), markdown });
        }
    }
    if a.bindings.at(&a.doc, offset) && a.bindings.operation(a.doc.from16(offset as usize)).is_none() { return None; }
    let at = a.doc.from16(offset as usize);
    let c = &a.doc.c;
    let part = |ch: char| is_letter_or_digit(ch) || "_°%‰µΩ".contains(ch);
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
    let inside = a.elements.iter().any(|el| el.body_start <= s && e <= el.body_end && a.shown(el))
        || a.calls.iter().any(|call| call.texts.iter().any(|t| t.start <= s && e <= t.end));
    if !inside {
        return None;
    }
    // solveng's words: as casting where it's highlighted so (after a number it's the unit attosecond), any as a type
    let (start, end) = (a.doc.to16(s) as u32, a.doc.to16(e) as u32);
    let kind = tokens.iter().find(|t| t.start <= start && end <= t.start + t.length).map(|t| t.kind);
    if kind == Some(K::KEYWORD)
        && let Some((k, d)) = crate::clause::KEYWORDS.iter().find(|(k, _)| *k == word) {
            return hover(format!("`{k}` keyword after `=` or `->`: {d}"));
        }
    if kind == Some(K::EVALUATE) || word.is_empty() {
        return None;
    }
    if word == "as" {
        if kind == Some(K::KEYWORD) {
            return hover("`as` cast: the number of what's before it, in SI units, as a value in the unit after it – with `U = 5 V`, `U as A` is 5 A, `U as mA` 5 mA, `U as 1` the plain number 5. It casts the term before it with its power: `2 U as A` is 2 · (U as A).".to_string());
        }
    }
    if word == "any" && a.doc.sub(0, s).trim_end().trim_end_matches('"').trim_end().ends_with(':') {
        return hover("`any` type: a parameter that takes every unit; the result's unit follows what's passed. `1` is the type of a plain number.".to_string());
    }
    // a unit where one is read, though a variable has its name: V in f(x: V) or x as "V"
    if kind == Some(K::UNIT)
        && let Some(d) = units::describe(&word, &a.units) {
            return hover(d);
        }
    if let Some(name) = crate::navigate::local_at(&a, at) { return hover(format!("`{name}` local sum index (unitless integer)")); }
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
    let builtin = if word == "sum_" { "sum".to_string() } else { word.clone() };
    if let Some(op) = a.bindings.operation(at).map(str::to_string).or_else(|| a.bindings.operator_strings().iter().find(|(s, e, _)| *s <= at && at <= *e).map(|(_, _, n)| n.clone())).or_else(||
        (a.builtin_functions_at(at) && FUNCTIONS.contains(&builtin.as_str())).then(|| builtin.clone())) {
        let docs = crate::params::builtin(&op).map(function_docs).unwrap_or_else(|| format!("`{op}()` built-in function"));
        return hover(if op == builtin { docs } else { format!("`{builtin}` stands for `{op}` (math.op)\n\n{docs}") });
    }
    if let Some((_, d)) = CONSTANTS.iter().find(|(n, _)| *n == word) {
        return hover(format!("`{word}` {d}"));
    }
    units::visible(&word, &a.units_at(at)).then(|| units::describe(&word, &a.units)).flatten().and_then(hover)
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
