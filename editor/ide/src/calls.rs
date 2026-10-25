//! Calls of solveng's functions, read with typst's parser: the equations they read, their string arguments (names
//! and option values), and their named arguments, which take the package's options.

use std::collections::HashMap;

use typst_syntax::{LinkedNode, SyntaxKind, SyntaxNode, ast};

use crate::params::{self, Function};
use crate::text::Doc;
use crate::ContextFile;

/// What a string argument holds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Role {
    /// A variable's name: `#lambdify($U^2/R$, "U")`, `#clear("x")`.
    Name,
    /// One of an option's strings: `form: "fraction"`.
    Choice,
    /// A value, a defined name or a keyword like `"extrema"`: `mark: "R"`.
    Mark,
}

/// A string argument, in characters: its contents, without the quotes.
#[derive(Clone, Debug)]
pub(crate) struct Text {
    pub start: usize,
    pub end: usize,
    pub role: Role,
    /// The named argument it's in, if any.
    pub param: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct Named {
    pub name: String,
    pub name_start: usize,
    pub name_end: usize,
}

/// Where a `worksheet` reads content, in characters: one of its content blocks, or what follows a
/// `#show: worksheet` rule in the block around it. `evaluate` is what it says about the math there.
#[derive(Clone, Debug)]
pub(crate) struct Scope {
    pub start: usize,
    pub end: usize,
    pub evaluate: bool,
}

/// Whether the math at `at` is calculated as the innermost worksheet around says; None outside every one.
pub(crate) fn evaluates_at(scopes: &[Scope], at: usize) -> Option<bool> {
    scopes.iter().filter(|s| s.start <= at && at < s.end).max_by_key(|s| s.start).map(|s| s.evaluate)
}

/// A worksheet wrapper's options, as `worksheet`, a `#let` alias of it or `.with(…)` sets them. Content it wraps
/// evaluates its math unless `evaluate` is false.
#[derive(Clone, Debug)]
pub(crate) struct Wrapper {
    pub evaluate: bool,
    /// `scope: …` as written, without quotes.
    pub scope: Option<String>,
}

impl Default for Wrapper {
    fn default() -> Wrapper {
        Wrapper { evaluate: true, scope: None }
    }
}

impl Wrapper {
    /// These options with what a call's arguments set over them.
    fn with(&self, args: Option<&LinkedNode>) -> Wrapper {
        let mut wrapper = self.clone();
        if let Some(args) = args {
            if args.children().any(|n| n.kind() == SyntaxKind::Named && n.children().next().is_some_and(|c| c.get().leaf_text() == "evaluate")) {
                wrapper.evaluate = evaluate(args);
            }
            for named in args.children().filter(|n| n.kind() == SyntaxKind::Named) {
                let source = named.get().clone().full_text();
                if let Some((name, value)) = source.split_once(':')
                    && name.trim() == "scope" {
                        wrapper.scope = Some(value.trim().trim_matches('"').to_string()).filter(|s| s != "auto");
                    }
            }
        }
        wrapper
    }
}

/// The names a text binds to `worksheet` (or `worksheet.with(…)`), through `#let` and the files it imports.
pub(crate) type Aliases = HashMap<String, Wrapper>;

/// The worksheet aliases of a text, following `#let` chains and imports from the files around it.
pub(crate) fn aliases(doc: &Doc, files: &[ContextFile], path: &str) -> Aliases {
    let mut out = Aliases::new();
    let mut seen = Vec::new();
    collect_aliases(doc, files, path, &mut out, &mut seen);
    out
}

fn collect_aliases(doc: &Doc, files: &[ContextFile], path: &str, out: &mut Aliases, seen: &mut Vec<String>) {
    if seen.iter().any(|p| p == path) || seen.len() >= 32 {
        return;
    }
    seen.push(path.to_string());
    let root = typst_syntax::parse(&doc.s);
    for node in LinkedNode::new(&root).children() {
        match node.kind() {
            SyntaxKind::LetBinding => {
                let Some(binding) = node.get().cast::<ast::LetBinding>() else { continue };
                let Some(name) = binding.kind().bindings().into_iter().next() else { continue };
                let Some(value) = node.children().rev().find(|c| matches!(c.kind(), SyntaxKind::Ident | SyntaxKind::FuncCall | SyntaxKind::Closure)) else { continue };
                if let Some(wrapper) = wrapper_of(&value, out) {
                    out.insert(name.as_str().to_string(), wrapper);
                }
            }
            SyntaxKind::ModuleImport => {
                let Some(source) = node.children().find(|c| c.kind() == SyntaxKind::Str) else { continue };
                let source = source.get().leaf_text().trim_matches('"').to_string();
                if source.starts_with('@') {
                    continue;
                }
                let target = crate::scope::resolve(&source, path);
                let Some(file) = files.iter().find(|f| f.path == target) else { continue };
                let mut theirs = Aliases::new();
                collect_aliases(&Doc::new(&file.text), files, &file.path, &mut theirs, seen);
                let items = node.children().find(|c| c.kind() == SyntaxKind::ImportItems);
                match items.as_ref().map(imported) {
                    // `*`, or an import Typst doesn't take names from.
                    None | Some(None) => out.extend(theirs),
                    Some(Some(list)) => {
                        for (original, name) in list {
                            if let Some(wrapper) = theirs.get(&original) {
                                out.insert(name, wrapper.clone());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    seen.pop();
}

/// The wrapper an expression names: `worksheet`, an alias, or either with `.with(…)`.
fn wrapper_of(node: &LinkedNode, aliases: &Aliases) -> Option<Wrapper> {
    match node.kind() {
        SyntaxKind::Ident => match node.get().leaf_text().as_str() {
            "worksheet" => Some(Wrapper::default()),
            name => aliases.get(name).cloned(),
        },
        SyntaxKind::FuncCall => {
            let name = callee(node.children().next()?.get())?;
            let base = match name.as_str() {
                "worksheet" => Wrapper::default(),
                name => aliases.get(name)?.clone(),
            };
            Some(base.with(node.children().find(|c| c.kind() == SyntaxKind::Args).as_ref()))
        }
        SyntaxKind::Closure => {
            let closure = node.get().cast::<ast::Closure>()?;
            let body_name = closure.params().children().filter_map(|param| match param {
                ast::Param::Pos(pattern) => pattern.bindings().into_iter().next().map(|i| i.as_str().to_string()),
                _ => None,
            }).last()?;
            let body = node.children().last()?;
            forwarded(&body, &body_name, aliases, None)
        }
        _ => None,
    }
}

/// Follow a template's content parameter through worksheet calls and show rules. A worksheet elsewhere in
/// the closure does not make it a worksheet template: its body must actually reach the calculating wrapper.
fn forwarded(node: &LinkedNode, body: &str, aliases: &Aliases, ambient: Option<Wrapper>) -> Option<Wrapper> {
    match node.kind() {
        SyntaxKind::Ident if node.get().leaf_text() == body => ambient,
        SyntaxKind::CodeBlock | SyntaxKind::Code | SyntaxKind::ContentBlock | SyntaxKind::Markup => {
            let mut current = ambient;
            let mut result = None;
            for child in node.children() {
                if child.kind() == SyntaxKind::ShowRule {
                    // Only selectorless rules wrap the remaining content.
                    let parts: Vec<_> = child.children().filter(|c| !c.kind().is_trivia()).collect();
                    if parts.get(1).is_some_and(|c| c.kind() == SyntaxKind::Colon)
                        && let Some(wrapper) = parts.last().and_then(|n| wrapper_of(n, aliases)) {
                            current = Some(wrapper);
                        }
                } else if let Some(wrapper) = forwarded(&child, body, aliases, current.clone()) {
                    result = Some(wrapper);
                }
            }
            result
        }
        SyntaxKind::FuncCall => {
            let wrapper = wrapper_of(node, aliases).or(ambient);
            let args = node.children().find(|c| c.kind() == SyntaxKind::Args)?;
            args.children().filter(|c| !matches!(c.kind(), SyntaxKind::Named))
                .find_map(|arg| forwarded(&arg, body, aliases, wrapper.clone()))
        }
        SyntaxKind::Parenthesized => node.children().find_map(|child| forwarded(&child, body, aliases, ambient.clone())),
        // Let bindings, nested closures and conditional control flow need runtime information.
        _ => None,
    }
}

/// The names an import takes: `None` for `*`, else (as written there, as here).
fn imported(items: &LinkedNode) -> Option<Vec<(String, String)>> {
    if items.children().any(|c| c.kind() == SyntaxKind::Star) { return None; }
    Some(items.get().cast::<ast::ImportItems>()?.iter().map(|item| {
        (item.original_name().as_str().to_string(), item.bound_name().as_str().to_string())
    }).collect())
}

#[derive(Clone, Debug)]
pub(crate) struct Call {
    pub function: &'static Function,
    /// Just inside the parentheses.
    pub args_start: usize,
    pub args_end: usize,
    pub texts: Vec<Text>,
    pub named: Vec<Named>,
    /// Equations solveng reads wherever they are: passed to `worksheet`, `value`, `graph` and `lambdify`, the
    /// expressions of `unit` and `constant`, and unit expressions of options (`preferred-units: ($#m / #s$,)`).
    pub maths: Vec<(usize, usize)>,
}

/// The function a callee names: `worksheet`, `solveng.worksheet`, `worksheet.with`.
pub(crate) fn callee(node: &SyntaxNode) -> Option<String> {
    match node.kind() {
        // Calls in code and math.
        SyntaxKind::Ident | SyntaxKind::MathIdent => Some(node.leaf_text().to_string()),
        SyntaxKind::FieldAccess => {
            let children: Vec<_> = node.children().collect();
            let field = children.last()?;
            if field.leaf_text() == "with" { callee(children.first()?) } else { Some(field.leaf_text().to_string()) }
        }
        _ => None,
    }
}

/// Options whose values are unit expressions in math: `preferred-units: ($#m / #s$,)`, `unit: $#W$`,
/// `units: (U: $#V$)`.
const UNIT_OPTIONS: &[&str] = &["preferred-units", "unit", "units"];

fn role(function: &str, param: Option<&str>, index: usize) -> Option<Role> {
    let Some(param) = param else {
        return match function {
            "lambdify" => (index > 0).then_some(Role::Name),
            "clear" => Some(Role::Name),
            _ => None,
        };
    };
    Some(match param {
        "guess" | "xrange" | "yrange" | "ymark" => return None,
        "unknown" => Role::Name,
        "mark" | "xmark" => Role::Mark,
        _ => {
            let f = params::function(function)?;
            let p = f.named.iter().find(|p| p.name == param)?;
            if p.values.is_empty() {
                return None;
            }
            Role::Choice
        }
    })
}

struct Reader<'a> {
    doc: &'a Doc,
    aliases: &'a Aliases,
    calls: Vec<Call>,
    scopes: Vec<Scope>,
}

impl Reader<'_> {
    fn chars(&self, node: &LinkedNode) -> (usize, usize) {
        (self.doc.of_byte(node.offset()), self.doc.of_byte(node.offset() + node.len()))
    }

    fn visit(&mut self, node: &LinkedNode) {
        if matches!(node.kind(), SyntaxKind::FuncCall | SyntaxKind::MathCall) {
            self.call(node);
        }
        if node.kind() == SyntaxKind::ShowRule {
            self.show_rule(node);
        }
        for child in node.children() {
            self.visit(&child);
        }
    }

    /// Strings in an argument's value: the string itself, the items of an array, the values of a dictionary.
    fn strings(&self, node: &LinkedNode, role: Role, param: Option<&str>, out: &mut Vec<Text>) {
        match node.kind() {
            SyntaxKind::Str | SyntaxKind::Error if node.get().clone().full_text().starts_with('"') => {
                let (start, end) = self.chars(node);
                let closed = end > start + 1 && self.doc.c[end - 1] == '"';
                out.push(Text { start: start + 1, end: if closed { end - 1 } else { end }, role, param: param.map(String::from) });
            }
            SyntaxKind::Array | SyntaxKind::Parenthesized => {
                for child in node.children() {
                    self.strings(&child, role, param, out);
                }
            }
            SyntaxKind::Dict => {
                // `bar: "…"` and `"k q": "…"`
                for child in node.children().filter(|c| matches!(c.kind(), SyntaxKind::Named | SyntaxKind::Keyed)) {
                    if let Some(value) = child.children().filter(|c| c.kind() != SyntaxKind::Colon && !c.kind().is_trivia()).nth(1) {
                        self.strings(&value, role, param, out);
                    }
                }
            }
            _ => {}
        }
    }

    /// Equations in an argument's value: the equation itself, the items of an array, the values of a dictionary.
    fn equations(&self, node: &LinkedNode, out: &mut Vec<(usize, usize)>) {
        match node.kind() {
            SyntaxKind::Equation => out.push(self.chars(node)),
            SyntaxKind::Array | SyntaxKind::Parenthesized => {
                for child in node.children() {
                    self.equations(&child, out);
                }
            }
            SyntaxKind::Dict => {
                for child in node.children().filter(|c| matches!(c.kind(), SyntaxKind::Named | SyntaxKind::Keyed)) {
                    if let Some(value) = child.children().filter(|c| c.kind() != SyntaxKind::Colon && !c.kind().is_trivia()).nth(1) {
                        self.equations(&value, out);
                    }
                }
            }
            _ => {}
        }
    }

    /// `#show: worksheet`, `#show: worksheet.with(…)` and their aliases: a worksheet for the rest of the block around.
    fn show_rule(&mut self, node: &LinkedNode) {
        let Some(transform) = node.children().rev().find(|c| !c.kind().is_trivia()) else { return };
        let (function, args) = match transform.kind() {
            SyntaxKind::FuncCall => (transform.children().next(), transform.children().find(|c| c.kind() == SyntaxKind::Args)),
            _ => (Some(transform.clone()), None),
        };
        let Some(name) = function.and_then(|f| callee(f.get())) else { return };
        let base = match name.as_str() {
            "worksheet" => Wrapper::default(),
            name => match self.aliases.get(name) {
                Some(wrapper) => wrapper.clone(),
                None => return,
            },
        };
        let wrapper = base.with(args.as_ref());
        let end = node.parent().map_or(self.doc.c.len(), |p| self.chars(p).1);
        self.scopes.push(Scope { start: self.chars(node).1, end, evaluate: wrapper.evaluate });
    }

    fn call(&mut self, node: &LinkedNode) {
        let Some(first) = node.children().next() else { return };
        let name = callee(first.get());
        // A `#let` alias of `worksheet.with(…)` is a worksheet too.
        let (function, base) = match name.as_deref().and_then(params::function) {
            Some(function) => (function, Wrapper::default()),
            None => {
                let Some(wrapper) = name.as_deref().and_then(|n| self.aliases.get(n)) else { return };
                (params::function("worksheet").expect("worksheet is a function"), wrapper.clone())
            }
        };
        let Some(args) = node.children().find(|c| matches!(c.kind(), SyntaxKind::Args | SyntaxKind::MathArgs)) else { return };
        let (open, close) = self.chars(&args);
        let closes = args.children().any(|c| c.kind() == SyntaxKind::RightParen);
        let paren = |c: &LinkedNode, text: &str| c.kind() == SyntaxKind::Error && c.get().leaf_text() == text;
        let mut call = Call {
            function,
            args_start: open + 1,
            args_end: if closes { close.saturating_sub(1).max(open + 1) } else { close },
            texts: Vec::new(),
            named: Vec::new(),
            maths: Vec::new(),
        };
        let mut index = 0;
        for arg in args.children() {
            match arg.kind() {
                SyntaxKind::ContentBlock => {
                    if function.name == "worksheet" {
                        let (start, end) = self.chars(&arg);
                        let wrapper = base.with(Some(&args));
                        self.scopes.push(Scope { start, end, evaluate: wrapper.evaluate });
                    }
                }
                SyntaxKind::LeftParen | SyntaxKind::RightParen | SyntaxKind::Comma => {}
                k if k.is_trivia() => {}
                _ if paren(&arg, "(") || paren(&arg, ")") => {}
                SyntaxKind::Named => {
                    let mut parts = arg.children().filter(|c| !c.kind().is_trivia());
                    let Some(name) = parts.next() else { continue };
                    let value = parts.find(|c| c.kind() != SyntaxKind::Colon && !(c.kind() == SyntaxKind::Error && c.is_empty()));
                    let (name_start, name_end) = self.chars(&name);
                    let pname = name.get().leaf_text().to_string();
                    if let (Some(v), Some(r)) = (&value, role(&function.name, Some(&pname), 0)) {
                        self.strings(v, r, Some(&pname), &mut call.texts);
                    }
                    if let Some(v) = value.as_ref().filter(|_| UNIT_OPTIONS.contains(&pname.as_str())) {
                        self.equations(v, &mut call.maths);
                    }
                    call.named.push(Named { name: pname, name_start, name_end });
                }
                _ => {
                    if arg.kind() == SyntaxKind::Equation && (function.name == "worksheet" || index == 0 && matches!(function.name.as_str(), "value" | "graph" | "lambdify" | "unit") || function.name == "constant") {
                        let (s, e) = self.chars(&arg);
                        call.maths.push((s, e));
                    }
                    if let Some(r) = role(&function.name, None, index) {
                        self.strings(&arg, r, None, &mut call.texts);
                    }
                    index += 1;
                }
            }
        }
        self.calls.push(call);
    }
}

impl Call {
    /// The equations passed as graph curves, split at math line breaks.
    pub(crate) fn curves(&self, doc: &Doc) -> impl Iterator<Item = (usize, usize)> {
        let mut curves = Vec::new();
        if self.function.name == "graph" {
            for &(s, e) in &self.maths {
                let start = s + 1;
                let end = e.saturating_sub(usize::from(doc.c.get(e.saturating_sub(1)) == Some(&'$')));
                let Some(body) = doc.c.get(start..end) else { continue };
                for (a, b) in crate::text::equation_ranges(body) {
                    curves.push((start + a, start + b));
                }
            }
        }
        curves.into_iter()
    }


}


/// Whether a worksheet's arguments calculate the math of its content: unless `evaluate: false`.
fn evaluate(args: &LinkedNode) -> bool {
    !args.children().filter(|c| c.kind() == SyntaxKind::Named).any(|named| {
        let mut parts = named.children().filter(|c| !c.kind().is_trivia() && c.kind() != SyntaxKind::Colon);
        parts.next().is_some_and(|n| n.get().leaf_text() == "evaluate") && parts.next().is_some_and(|v| v.get().clone().full_text() == "false")
    })
}

/// Every call of a solveng function in a document, outer ones first, and where worksheets read content.
pub(crate) fn read(doc: &Doc, aliases: &Aliases) -> (Vec<Call>, Vec<Scope>) {
    let root = typst_syntax::parse(&doc.s);
    let mut reader = Reader { doc, aliases, calls: Vec::new(), scopes: Vec::new() };
    reader.visit(&LinkedNode::new(&root));
    (reader.calls, reader.scopes)
}

/// The content and code blocks of a document, in characters: where a show rule's worksheet ends.
pub(crate) fn blocks(doc: &Doc) -> Vec<(usize, usize)> {
    fn visit(node: &LinkedNode, doc: &Doc, out: &mut Vec<(usize, usize)>) {
        if matches!(node.kind(), SyntaxKind::ContentBlock | SyntaxKind::CodeBlock) {
            out.push((doc.of_byte(node.offset()), doc.of_byte(node.offset() + node.len())));
        }
        for child in node.children() {
            visit(&child, doc, out);
        }
    }
    let root = typst_syntax::parse(&doc.s);
    let mut out = Vec::new();
    visit(&LinkedNode::new(&root), doc, &mut out);
    out
}

/// The files a document includes (`#include "…"`), as written, and where in it (in characters).
pub(crate) fn includes(doc: &Doc) -> Vec<(String, usize)> {
    let root = typst_syntax::parse(&doc.s);
    let mut out = Vec::new();
    collect_includes(&LinkedNode::new(&root), doc, &mut out);
    out
}

fn collect_includes(node: &LinkedNode, doc: &Doc, out: &mut Vec<(String, usize)>) {
    if node.kind() == SyntaxKind::ModuleInclude
        && let Some(path) = node.children().find(|c| c.kind() == SyntaxKind::Str) {
            out.push((path.get().leaf_text().trim_matches('"').to_string(), doc.of_byte(node.offset())));
        }
    for child in node.children() {
        collect_includes(&child, doc, out);
    }
}
