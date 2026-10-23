//! Calls of solveng's functions, read with typst's parser: their string arguments are calculations, units or
//! names, and their named arguments take the package's options.

use std::collections::HashMap;

use typst_syntax::{LinkedNode, SyntaxKind, SyntaxNode};

use crate::params::{self, Function};
use crate::text::Doc;
use crate::units::Custom;
use crate::ContextFile;

/// What a string argument holds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Role {
    /// An equation or expression: `"R = U/I"`.
    Code,
    /// A unit: `unit: "mA"`.
    Unit,
    /// A variable's name: `x: "U"`.
    Name,
    /// One of an option's strings: `steps: "value"`.
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
    pub units: Vec<crate::units::Custom>,
    pub builtin_functions: Option<bool>,
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
    pub units: Vec<Custom>,
    pub builtin_functions: Option<bool>,
}

impl Default for Wrapper {
    fn default() -> Wrapper {
        Wrapper { evaluate: true, units: Vec::new(), builtin_functions: None }
    }
}

impl Wrapper {
    /// These options with what a call's arguments set over them.
    fn with(&self, args: Option<&LinkedNode>) -> Wrapper {
        let mut wrapper = self.clone();
        if let Some(args) = args {
            wrapper.evaluate = evaluate(args);
            wrapper.units.extend(unit_settings(args));
            for named in args.children().filter(|n| n.kind() == SyntaxKind::Named) {
                let source = named.get().clone().full_text();
                if let Some((name, value)) = source.split_once(':') {
                    if name.trim() == "builtin-functions" {
                        wrapper.builtin_functions = value.trim().parse().ok();
                    }
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
                let mut parts = node.children().filter(|c| !c.kind().is_trivia());
                let (Some(kw), Some(name)) = (parts.next(), parts.next()) else { continue };
                let Some(value) = node.children().rev().find(|c| matches!(c.kind(), SyntaxKind::Ident | SyntaxKind::FuncCall)) else { continue };
                if kw.kind() != SyntaxKind::Let || name.kind() != SyntaxKind::Ident {
                    continue;
                }
                if let Some(wrapper) = wrapper_of(&value, out) {
                    out.insert(name.get().leaf_text().to_string(), wrapper);
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
        _ => None,
    }
}

/// The names an import takes: `None` for `*`, else (as written there, as here).
fn imported(items: &LinkedNode) -> Option<Vec<(String, String)>> {
    let mut out = Vec::new();
    if items.children().any(|c| c.kind() == SyntaxKind::Star) {
        return None;
    }
    for item in items.children() {
        let names: Vec<String> = item.children().filter(|c| c.kind() == SyntaxKind::Ident).map(|c| c.get().leaf_text().to_string()).collect();
        match (item.kind(), names.as_slice()) {
            (SyntaxKind::ImportItemPath, [name]) => out.push((name.clone(), name.clone())),
            (SyntaxKind::RenamedImportItem, [original, name]) => out.push((original.clone(), name.clone())),
            _ => {}
        }
    }
    Some(out)
}

#[derive(Clone, Debug)]
pub(crate) struct Call {
    pub function: &'static Function,
    /// Just inside the parentheses.
    pub args_start: usize,
    pub args_end: usize,
    pub texts: Vec<Text>,
    pub named: Vec<Named>,
    /// Equations and raw elements passed as positional arguments, which are calculated wherever they are.
    pub maths: Vec<(usize, usize)>,
    /// Arguments as (name, source), for conversions.
    pub args: Vec<(Option<String>, String)>,
    /// Where each argument is, and whether it's a bare identifier, which may be a parameter's name being typed.
    pub items: Vec<(usize, usize, bool)>,
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

fn role(function: &str, param: Option<&str>, index: usize) -> Option<Role> {
    let Some(param) = param else {
        return match function {
            "worksheet" | "graph" | "value" => Some(Role::Code),
            "unit" => Some(Role::Unit),
            "lambdify" => Some(if index == 0 { Role::Code } else { Role::Name }),
            "clear" => Some(Role::Name),
            _ => None,
        };
    };
    Some(match param {
        "unit" | "preferred-units" => Role::Unit,
        "units" if function == "lambdify" => Role::Unit,
        "units" | "guess" | "xrange" | "yrange" | "ymark" => Role::Code,
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
        self.scopes.push(Scope { start: self.chars(node).1, end, evaluate: wrapper.evaluate, units: wrapper.units, builtin_functions: wrapper.builtin_functions });
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
            args: Vec::new(),
            items: Vec::new(),
        };
        let mut index = 0;
        for arg in args.children() {
            match arg.kind() {
                SyntaxKind::ContentBlock => {
                    if function.name == "worksheet" {
                        let (start, end) = self.chars(&arg);
                        let wrapper = base.with(Some(&args));
                        self.scopes.push(Scope { start, end, evaluate: wrapper.evaluate, units: wrapper.units, builtin_functions: wrapper.builtin_functions });
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
                    let (_, value_end) = match &value {
                        Some(v) => self.chars(v),
                        None => (self.chars(&arg).1, self.chars(&arg).1),
                    };
                    let pname = name.get().leaf_text().to_string();
                    if let (Some(v), Some(r)) = (&value, role(&function.name, Some(&pname), 0)) {
                        self.strings(v, r, Some(&pname), &mut call.texts);
                    }
                    call.args.push((Some(pname.clone()), value.as_ref().map(|v| v.get().clone().full_text().to_string()).unwrap_or_default()));
                    call.items.push((name_start, value_end.max(name_end), false));
                    call.named.push(Named { name: pname, name_start, name_end });
                }
                _ => {
                    let code = arg.kind() == SyntaxKind::Raw && function.name == "worksheet";
                    if (arg.kind() == SyntaxKind::Equation || code) && role(&function.name, None, index) == Some(Role::Code) {
                        let (s, e) = self.chars(&arg);
                        call.maths.push((s, e));
                    }
                    if let Some(r) = role(&function.name, None, index) {
                        self.strings(&arg, r, None, &mut call.texts);
                    }
                    call.args.push((None, arg.get().clone().full_text().to_string()));
                    let (s, e) = self.chars(&arg);
                    call.items.push((s, e, arg.kind() == SyntaxKind::Ident));
                    index += 1;
                }
            }
        }
        if function.name == "graph" {
            call.texts = call.texts.into_iter().flat_map(|t| {
                if t.param.is_some() || t.role != Role::Code { return vec![t]; }
                let (decoded, offsets) = crate::text::unescape(&self.doc.c[t.start..t.end]);
                crate::text::equation_ranges(&decoded, true).into_iter().map(|(a, b)| Text {
                    start: t.start + offsets[a], end: t.start + offsets[b], ..t.clone()
                }).collect()
            }).collect();
        }
        self.calls.push(call);
    }
}

impl Call {
    /// The strings that are a graph's curves: `"q = 12 A"`, read as formulas.
    pub(crate) fn curves(&self) -> impl Iterator<Item = &Text> {
        self.texts.iter().filter(move |t| self.function.name == "graph" && t.param.is_none() && t.role == Role::Code)
    }

    pub(crate) fn is_curve(&self, t: &Text) -> bool {
        self.function.name == "graph" && t.param.is_none() && t.role == Role::Code
    }
}

fn unit_settings(args: &LinkedNode) -> Vec<crate::units::Custom> {
    let mut units = Vec::new();
    for named in args.children().filter(|node| node.kind() == SyntaxKind::Named) {
        let name = named.children().find(|child| !child.kind().is_trivia());
        if name.is_some_and(|name| matches!(name.get().leaf_text().as_str(), "units" | "builtin-units")) {
            crate::units::custom_units(&named.get().clone().full_text(), &mut units);
        }
    }
    units
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
