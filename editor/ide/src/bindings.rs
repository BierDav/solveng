//! The Typst bindings solveng reads in math: units, constants, operations and modifiers. They come from solveng's
//! modules (`#import units: km`, `#import constants: pi`, `#import functions: tri`, `#import modifiers: solve`) or are
//! made with `base-unit`, `unit`, `constant`, `operation` and `math.op`. Typst resolves them lexically, so a binding means the same in every
//! worksheet; solveng sees their values. Quoted text in math is a worksheet variable and never one of them.
//!
//! Limits: bindings are read from the document itself; names imported from other local files (`#import
//! "defaults.typ": minute`) and closure parameters aren't followed.

use std::sync::LazyLock;

use typst_syntax::{LinkedNode, SyntaxKind, ast::{self, AstNode, LetBinding, ModuleImport}};

use crate::text::Doc;

/// One of solveng's modules of bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Module {
    Units,
    Functions,
    Constants,
    Modifiers,
}

impl Module {
    pub fn name(self) -> &'static str {
        match self {
            Module::Units => "units",
            Module::Functions => "functions",
            Module::Constants => "constants",
            Module::Modifiers => "modifiers",
        }
    }

    fn of(name: &str) -> Option<Module> {
        [Module::Units, Module::Functions, Module::Constants, Module::Modifiers].into_iter().find(|m| m.name() == name)
    }

    /// Every name the module exports, read from the package.
    pub fn exports(self) -> Vec<String> {
        match self {
            Module::Units => crate::units::shorthands(),
            Module::Functions => crate::params::automatic_names(),
            Module::Constants => MODULE_CONSTANTS.iter().map(|(name, _)| name.clone()).collect(),
            Module::Modifiers => crate::clause::KEYWORDS.iter().map(|(name, _)| name.to_string()).collect(),
        }
    }

    /// What the module's binding `name` means; None if it doesn't export it.
    pub fn meaning(self, name: &str) -> Option<Meaning> {
        match self {
            Module::Units => crate::units::shorthand(name).then(|| Meaning::ModuleUnit(name.into())),
            Module::Functions => crate::params::automatic_math(name).then(|| Meaning::Operation(name.into())),
            Module::Constants => MODULE_CONSTANTS.iter().find(|(n, _)| n == name).map(|(_, m)| m.clone()),
            Module::Modifiers => crate::clause::KEYWORDS.iter().any(|(k, _)| *k == name).then(|| Meaning::Modifier(name.into())),
        }
    }

    /// The source of the module, whose leading `///` block documents it.
    pub fn source(self) -> &'static str {
        match self {
            Module::Units => include_str!("../../../src/unit-words.typ"),
            Module::Functions => include_str!("../../../src/builtins.typ"),
            Module::Constants => include_str!("../../../src/constants.typ"),
            Module::Modifiers => include_str!("../../../src/modifiers.typ"),
        }
    }
}

/// The constants of solveng's `constants` module, read from the package as any document's `constant` bindings.
static MODULE_CONSTANTS: LazyLock<Vec<(String, Meaning)>> = LazyLock::new(|| {
    let bindings = Bindings::new(&Doc::new(Module::Constants.source()));
    bindings.definitions.into_iter().filter(|b| matches!(b.meaning, Meaning::Constant { .. })).map(|b| (b.name, b.meaning)).collect()
});

/// What a Typst binding is to solveng.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Meaning {
    /// A unit of solveng's `units` module, by its name there: `km`.
    ModuleUnit(String),
    /// `base-unit("time")`: an independent dimension, by its identity.
    BaseUnit(String),
    /// `unit($60 #s$)`: a unit scaled from others, by its expression.
    Unit(String),
    /// `constant(…)`: its identity (`pi`), if any, and expression.
    Constant { identity: Option<String>, expression: String },
    /// An operation: `math.op("tri")`, `operation("convolve")` or one of the `functions` module.
    Operation(String),
    /// A modifier of the clause after a result, by its name in the `modifiers` module: `solve`.
    Modifier(String),
    /// Anything else Typst binds, which solveng reads as a plain value.
    Other,
}

impl Meaning {
    pub fn is_unit(&self) -> bool {
        matches!(self, Meaning::ModuleUnit(_) | Meaning::BaseUnit(_) | Meaning::Unit(_))
    }

    /// What it is, in a word or two: `kilometre`, `base unit · time`, `unit = 60 #s`.
    pub fn detail(&self) -> String {
        match self {
            Meaning::ModuleUnit(name) => crate::units::unit_name(name).map_or_else(|| "unit".into(), |n| format!("{n} · solveng units")),
            Meaning::BaseUnit(identity) => format!("base unit · {identity}"),
            Meaning::Unit(expression) => format!("unit = {expression}"),
            Meaning::Constant { identity: Some(identity), expression } => format!("constant {identity} = {expression}"),
            Meaning::Constant { identity: None, expression } => format!("constant = {expression}"),
            Meaning::Operation(name) => format!("operation {name}"),
            Meaning::Modifier(name) => crate::clause::KEYWORDS.iter().find(|(k, _)| k == name).map_or_else(|| "modifier".into(), |(_, d)| format!("modifier · {d}")),
            Meaning::Other => String::new(),
        }
    }

    /// What it is for a hover, in markdown: a unit's heritage, a constant's identity and value.
    pub fn describe(&self) -> String {
        match self {
            Meaning::ModuleUnit(name) => crate::units::describe(name).unwrap_or_else(|| format!("`{name}` unit")),
            Meaning::BaseUnit(identity) => format!("base unit of the dimension `{identity}`: declarations with the same identity are the same dimension, in every file and worksheet"),
            Meaning::Unit(expression) => format!("unit: `{expression}`"),
            Meaning::Constant { identity, expression } => {
                let identity = identity.as_ref().map_or_else(String::new, |i| format!(", identity `{i}`"));
                format!("constant: `{expression}`{identity}")
            }
            Meaning::Operation(name) => crate::params::builtin(name).map(crate::highlight::function_docs).unwrap_or_else(|| format!("`{name}()` operation")),
            Meaning::Modifier(name) => crate::highlight::modifier_docs(name),
            Meaning::Other => String::new(),
        }
    }

    /// The kind in a word, for messages.
    fn kind(&self) -> &'static str {
        match self {
            Meaning::Constant { .. } => "constant",
            Meaning::Operation(_) => "operation",
            Meaning::Modifier(_) => "modifier",
            Meaning::Other => "",
            _ => "unit",
        }
    }
}

struct Binding {
    name: String,
    start: usize,
    end: usize,
    visible: usize,
    scope_end: usize,
    meaning: Meaning,
    /// The module an import takes it from, and its name there (`s` of `s as test`).
    module: Option<(Module, String)>,
    /// For a name an import lists, the module's name as written there: `s` of `s as test` (the bound name's
    /// range when not renamed).
    original: Option<(usize, usize)>,
    /// For a name a wildcard import gives (`#import constants: *`): where an import renaming it goes, the end of that
    /// import's line, and the line's indentation.
    wildcard: Option<(usize, String)>,
}

/// An explicit operation reference in math (`op("tri")`) that can show just its name.
#[derive(Clone, Copy)]
pub(crate) struct InlineReference {
    pub start: usize,
    pub end: usize,
    pub text_start: usize,
    pub text_end: usize,
}

/// An import that names can be added to: solveng's package (`#import "@preview/solveng:1.0.0": *`) or one of its
/// modules (`#import units: km`).
#[derive(Clone, Debug)]
struct Import {
    /// None for the package.
    module: Option<Module>,
    /// The names it lists; None for `*`.
    names: Option<Vec<String>>,
    /// Where another name goes: after the last one listed.
    append: Option<usize>,
    /// The end of its line, where a new import goes.
    line_end: usize,
    visible: usize,
    scope_end: usize,
}

#[derive(Default)]
pub(crate) struct Bindings {
    definitions: Vec<Binding>,
    /// Identifiers that use a binding: start, end, the binding.
    uses: Vec<(usize, usize, usize)>,
    /// Identifiers in math that no binding of the document resolves: start, end, name.
    unbound: Vec<(usize, usize, String)>,
    /// Single letters in math written without `#` where a solveng binding of that name is in scope.
    bare: Vec<(usize, usize, usize)>,
    operators: Vec<(usize, usize, String)>,
    inline: Vec<InlineReference>,
    imports: Vec<Import>,
    /// Where imports name solveng's modules (`units` of `#import units: km`).
    modules: Vec<(usize, usize, Module)>,
}

fn visit(node: &LinkedNode, f: &mut impl FnMut(&LinkedNode)) {
    f(node);
    for child in node.children() { visit(&child, f); }
}

/// The callee of a call as written (`unit`, `math.op`) and its arguments.
fn call_parts<'a>(node: &LinkedNode<'a>) -> Option<(String, LinkedNode<'a>)> {
    if !matches!(node.kind(), SyntaxKind::FuncCall | SyntaxKind::MathCall) { return None; }
    let callee = node.children().next()?.get().clone().full_text().trim().to_string();
    let args = node.children().find(|c| matches!(c.kind(), SyntaxKind::Args | SyntaxKind::MathArgs))?;
    Some((callee, args))
}

/// The positional arguments of a call.
fn positional<'a>(args: &LinkedNode<'a>) -> impl Iterator<Item = LinkedNode<'a>> {
    args.children().filter(|c| !c.kind().is_trivia() && !matches!(c.kind(), SyntaxKind::LeftParen | SyntaxKind::RightParen | SyntaxKind::Comma | SyntaxKind::Named))
}

/// A named argument's value.
fn named_value<'a>(args: &LinkedNode<'a>, name: &str) -> Option<LinkedNode<'a>> {
    args.children().filter(|c| c.kind() == SyntaxKind::Named).find_map(|named| {
        let mut parts = named.children().filter(|c| !c.kind().is_trivia() && c.kind() != SyntaxKind::Colon);
        (parts.next()?.get().leaf_text() == name).then(|| parts.next()).flatten()
    })
}

/// A string literal's value.
fn string(node: &LinkedNode) -> Option<String> {
    (node.kind() == SyntaxKind::Str).then(|| node.get().cast::<ast::Str>().map(|s| s.get().to_string())).flatten()
}

/// An equation's body as written.
fn equation(node: &LinkedNode) -> Option<String> {
    (node.kind() == SyntaxKind::Equation).then(|| node.get().clone().full_text().trim_matches('$').trim().to_string())
}

impl Bindings {
    pub fn new(doc: &Doc) -> Self {
        let syntax = typst_syntax::parse(&doc.s);
        let root = LinkedNode::new(&syntax);
        let mut result = Self::default();
        visit(&root, &mut |node| {
            if let Some(import) = node.get().cast::<ModuleImport>() {
                result.import(node, import, doc);
                return;
            }
            let Some(binding) = node.get().cast::<LetBinding>() else { return };
            let scope_end = lexical_end(node, doc);
            let meaning = binding.init().map_or(Meaning::Other, |_| {
                let value = node.children().rev().find(|c| !c.kind().is_trivia());
                value.map_or(Meaning::Other, |value| result.meaning_of(&value, doc))
            });
            for name in binding.kind().bindings() {
                visit(node, &mut |leaf| {
                    if std::ptr::eq(leaf.get(), name.to_untyped()) {
                        result.definitions.push(Binding { name: name.as_str().into(), start: doc.of_byte(leaf.offset()), end: doc.of_byte(leaf.offset() + leaf.len()), visible: doc.of_byte(node.offset() + node.len()), scope_end, meaning: meaning.clone(), module: None, original: None, wildcard: None });
                    }
                });
            }
        });
        visit(&root, &mut |node| result.read_use(node, doc));
        result
    }

    /// What a `#let`'s value means: a constructor's result, another binding's meaning, a module's binding.
    fn meaning_of(&self, value: &LinkedNode, doc: &Doc) -> Meaning {
        let at = doc.of_byte(value.offset());
        match value.kind() {
            SyntaxKind::Ident => self.binding(at, value.get().leaf_text().as_str()).map_or(Meaning::Other, |id| self.definitions[id].meaning.clone()),
            // `units.km`: a module's binding by its field
            SyntaxKind::FieldAccess => {
                let parts: Vec<_> = value.children().filter(|c| c.kind() == SyntaxKind::Ident).collect();
                match parts.as_slice() {
                    [module, field] => Module::of(module.get().leaf_text().as_str()).and_then(|m| m.meaning(field.get().leaf_text().as_str())).unwrap_or(Meaning::Other),
                    _ => Meaning::Other,
                }
            }
            SyntaxKind::FuncCall => {
                let Some((callee, args)) = call_parts(value) else { return Meaning::Other };
                let first = positional(&args).next();
                match callee.as_str() {
                    "unit" => first.as_ref().and_then(equation).map_or(Meaning::Other, Meaning::Unit),
                    "base-unit" => first.as_ref().and_then(string).map_or(Meaning::Other, Meaning::BaseUnit),
                    "constant" => {
                        // `constant("pi", $…$)`, or the expression first with `identity: "pi"`
                        let identity = positional(&args).find_map(|a| string(&a)).or_else(|| named_value(&args, "identity").as_ref().and_then(string));
                        positional(&args).find_map(|a| equation(&a)).map_or(Meaning::Other, |expression| Meaning::Constant { identity, expression })
                    }
                    "math.op" | "operation" => first.as_ref().and_then(string).map_or(Meaning::Other, Meaning::Operation),
                    _ => Meaning::Other,
                }
            }
            _ => Meaning::Other,
        }
    }

    /// What a node uses: a binding, an explicit operation, a single letter that has a binding.
    fn read_use(&mut self, node: &LinkedNode, doc: &Doc) {
        let start = doc.of_byte(node.offset());
        let end = doc.of_byte(node.offset() + node.len());
        if let Some((callee, args)) = call_parts(node) {
            let operation = matches!(callee.as_str(), "math.op" | "operation") || callee == "op" && node.kind() == SyntaxKind::MathCall;
            if operation
                && let Some(string) = args.children().find(|c| matches!(c.kind(), SyntaxKind::Str | SyntaxKind::Error) && c.get().clone().full_text().starts_with('"')) {
                    let source = string.get().clone().full_text();
                    let closed = source.len() > 1 && source.ends_with('"');
                    let text_start = doc.of_byte(string.offset()) + 1;
                    let text_end = doc.of_byte(string.offset() + string.len()) - usize::from(closed);
                    let (name, _) = crate::text::unescape(&doc.c[text_start..text_end]);
                    self.operators.push((text_start, text_end, name.into_iter().collect()));
                    if node.kind() == SyntaxKind::MathCall && callee == "op" && closed && text_end > text_start && args.children().any(|c| c.kind() == SyntaxKind::RightParen) {
                        self.inline.push(InlineReference { start, end, text_start, text_end });
                    }
                }
        }
        // A single letter in math is a symbol, not a binding: `s` of `$2 s$` while `s` is a unit binding.
        if node.kind() == SyntaxKind::MathText && node.get().leaf_text().chars().count() == 1
            && doc.c.get(end) != Some(&'_') && (start == 0 || doc.c[start - 1] != '_')
            && node.parent_kind() != Some(SyntaxKind::MathAttach)
            && let Some(id) = self.binding(start, node.get().leaf_text().as_str())
            && self.definitions[id].meaning != Meaning::Other {
                self.bare.push((start, end, id));
            }
        if !matches!(node.kind(), SyntaxKind::Ident | SyntaxKind::MathIdent) { return; }
        if self.definitions.iter().any(|b| b.start == start) { return; }
        if node.parent_kind() == Some(SyntaxKind::FieldAccess) && node.prev_sibling().is_some() { return; }
        if node.parent_kind() == Some(SyntaxKind::Named) && node.prev_sibling().is_none() { return; }
        let name = node.get().leaf_text();
        match self.binding(start, name.as_str()) {
            Some(id) => self.uses.push((start, end, id)),
            None if in_math(node) => self.unbound.push((start, end, name.to_string())),
            None => {}
        }
    }

    /// `#import units: km, mA as milli` or `#import constants: *`: the names a module gives, visible like other imports.
    fn import(&mut self, node: &LinkedNode, import: ModuleImport, doc: &Doc) {
        let visible = doc.of_byte(node.offset() + node.len());
        let scope_end = lexical_end(node, doc);
        let listed: Option<Vec<String>> = match import.imports() {
            Some(ast::Imports::Items(items)) => Some(items.iter().map(|i| i.bound_name().as_str().to_string()).collect()),
            _ => None,
        };
        let append = node.children().find(|c| c.kind() == SyntaxKind::ImportItems).map(|items| doc.of_byte(items.offset() + items.len()));
        let line_end = (visible..doc.c.len()).find(|&i| doc.c[i] == '\n').unwrap_or(doc.c.len());
        let name = match import.source() {
            ast::Expr::Ident(ident) => ident.as_str().to_string(),
            ast::Expr::FieldAccess(access) => access.field().as_str().to_string(),
            ast::Expr::Str(source) => {
                // solveng's package, which holds the modules
                if source.get().contains("solveng") && import.imports().is_some() {
                    self.imports.push(Import { module: None, names: listed, append, line_end, visible, scope_end });
                }
                return;
            }
            _ => return,
        };
        let Some(module) = Module::of(&name) else { return };
        if let Some(source) = node.children().find(|c| matches!(c.kind(), SyntaxKind::Ident | SyntaxKind::FieldAccess)) {
            let end = doc.of_byte(source.offset() + source.len());
            self.modules.push((end - name.chars().count(), end, module));
        }
        if import.imports().is_some() {
            self.imports.push(Import { module: Some(module), names: listed, append, line_end, visible, scope_end });
        }
        let line_start = doc.c[..doc.of_byte(node.offset())].iter().rposition(|&c| c == '\n').map_or(0, |i| i + 1);
        let indent: String = doc.c[line_start..].iter().take_while(|c| **c == ' ' || **c == '\t').collect();
        let wildcard = matches!(import.imports(), Some(ast::Imports::Wildcard)).then(|| (line_end, indent));
        let mut push = |name: &str, original: &str, start: usize, end: usize, written: Option<(usize, usize)>| {
            if let Some(meaning) = module.meaning(original) {
                self.definitions.push(Binding { name: name.into(), start, end, visible, scope_end, meaning, module: Some((module, original.into())), original: written, wildcard: wildcard.clone() });
            }
        };
        match import.imports() {
            // Not at a place in the text, so nothing there names them.
            Some(ast::Imports::Wildcard) => {
                for name in module.exports() { push(&name, &name, usize::MAX, usize::MAX, None); }
            }
            Some(ast::Imports::Items(items)) => {
                for item in items.iter() {
                    let leaf_range = |ident: &typst_syntax::SyntaxNode| {
                        let mut range = (usize::MAX, usize::MAX);
                        visit(node, &mut |leaf| if std::ptr::eq(leaf.get(), ident) { range = (doc.of_byte(leaf.offset()), doc.of_byte(leaf.offset() + leaf.len())) });
                        range
                    };
                    let bound = item.bound_name();
                    let range = leaf_range(bound.to_untyped());
                    let original = leaf_range(item.original_name().to_untyped());
                    push(bound.as_str(), item.original_name().as_str(), range.0, range.1, Some(original));
                }
            }
            None => {}
        }
    }

    /// The name an import binds from solveng's modules at `at`: in its import (the name bound, or the module's
    /// name it renames) or where it's used.
    fn shorthand_at(&self, at: usize) -> Option<usize> {
        let within = |(s, e): (usize, usize)| s <= at && at <= e;
        self.definitions.iter().position(|b| b.module.is_some() && b.original.is_some() && (within((b.start, b.end)) || b.original.is_some_and(within)))
            .or_else(|| self.uses.iter().find(|&&(s, e, id)| within((s, e)) && self.definitions[id].module.is_some()).map(|&(_, _, id)| id))
    }

    /// Where the imported name at `at` is imported: the name it's bound to.
    pub(crate) fn shorthand_definition(&self, at: usize) -> Option<(usize, usize)> {
        self.listed_at(at).map(|id| (self.definitions[id].start, self.definitions[id].end))
    }

    /// [`Self::shorthand_at`], when its import lists it (`*` binds it at no place in the text).
    fn listed_at(&self, at: usize) -> Option<usize> {
        self.shorthand_at(at).filter(|&id| self.definitions[id].original.is_some())
    }

    /// [`Self::shorthand_at`], when its import lists it or gives it with `*`: a name that can be renamed.
    fn renamable_at(&self, at: usize) -> Option<usize> {
        self.shorthand_at(at).filter(|&id| self.definitions[id].original.is_some() || self.definitions[id].wildcard.is_some())
    }

    /// The imported name at `at` where its import lists it, and every use.
    pub(crate) fn shorthand_usages(&self, at: usize) -> Vec<(usize, usize)> {
        let Some(id) = self.renamable_at(at) else { return Vec::new() };
        let b = &self.definitions[id];
        b.original.map(|_| (b.start, b.end)).into_iter()
            .chain(self.uses.iter().filter(|u| u.2 == id).map(|&(s, e, _)| (s, e))).collect()
    }

    /// Whether `at` is on a name in an import list, where renaming writes an alias (`pi as tau`).
    pub(crate) fn in_import_list(&self, at: usize) -> bool {
        let within = |(s, e): (usize, usize)| s <= at && at <= e;
        self.definitions.iter().any(|b| b.module.is_some() && b.original.is_some_and(|o| within(o) || within((b.start, b.end))))
    }

    /// The module name at `at` in an import of solveng's modules, with its range.
    pub(crate) fn module_at(&self, at: usize) -> Option<(usize, usize, Module)> {
        self.modules.iter().find(|&&(s, e, _)| s <= at && at <= e).copied()
    }

    /// The imported name at `at`, for its hover: the range there, the name it's bound to, the module's name for
    /// it, the module and what it means.
    pub(crate) fn shorthand_info(&self, at: usize) -> Option<(usize, usize, String, String, Module, &Meaning)> {
        let id = self.shorthand_at(at)?;
        let b = &self.definitions[id];
        let within = |(s, e): (usize, usize)| s <= at && at <= e;
        let range = self.uses.iter().find(|&&(s, e, i)| i == id && within((s, e))).map(|&(s, e, _)| (s, e))
            .or_else(|| b.original.filter(|o| within(*o))).or_else(|| b.original.map(|_| (b.start, b.end)))?;
        let (module, original) = b.module.clone()?;
        Some((range.0, range.1, b.name.clone(), original, module, &b.meaning))
    }

    /// The name at `at` that renaming the imported name there changes.
    pub(crate) fn shorthand_target(&self, at: usize) -> Option<(usize, usize, String)> {
        let id = self.renamable_at(at)?;
        let b = &self.definitions[id];
        let (s, e) = self.uses.iter().find(|&&(s, e, i)| i == id && s <= at && at <= e).map(|&(s, e, _)| (s, e)).unwrap_or((b.start, b.end));
        Some((s, e, b.name.clone()))
    }

    /// Renames the imported name at `at` and its uses to `name`, an alias in its import (`s as test`); back to the
    /// module's own name the alias goes. A name `*` gives gets an import of its own after that one
    /// (`#import constants: pi as tau`). Edits as (start, end, text); None if `name` isn't a Typst identifier.
    pub(crate) fn shorthand_rename(&self, doc: &Doc, at: usize, name: &str) -> Option<Vec<(usize, usize, String)>> {
        let id = self.renamable_at(at)?;
        if !typst_syntax::is_ident(name) { return None; }
        let b = &self.definitions[id];
        let uses = self.uses.iter().filter(|u| u.2 == id).map(|&(s, e, _)| (s, e, name.to_string()));
        if let (None, Some((line_end, indent)), Some((module, original))) = (b.original, &b.wildcard, &b.module) {
            let import = (name != original).then(|| (*line_end, *line_end, format!("\n{indent}#import {}: {original} as {name}", module.name())));
            return Some(import.into_iter().chain(uses).collect());
        }
        let (os, oe) = b.original?;
        let original = doc.sub(os, oe);
        let import = if name == original {
            (os, b.end, original)
        } else if (os, oe) != (b.start, b.end) {
            (b.start, b.end, name.to_string())
        } else {
            (os, oe, format!("{original} as {name}"))
        };
        Some(std::iter::once(import).chain(uses).collect())
    }

    /// The names imports list from solveng's modules: their range, what they mean, and where a renamed one's module
    /// name is (`s` of `s as test`), which isn't a name of the document.
    pub(crate) fn shorthand_imports(&self) -> impl Iterator<Item = ((usize, usize), &Meaning, Option<(usize, usize)>)> + '_ {
        self.definitions.iter().filter(|b| b.module.is_some() && b.original.is_some())
            .map(|b| ((b.start, b.end), &b.meaning, b.original.filter(|o| *o != (b.start, b.end))))
    }

    /// The edits that make `name` of `module` available at `at`: added to an import of the module in scope, else a
    /// new one after solveng's import (which gets the module if it lists names). Empty when it's already there;
    /// None when solveng isn't imported in scope.
    pub(crate) fn import_edits(&self, doc: &Doc, at: usize, module: Module, name: &str) -> Option<Vec<crate::Edit>> {
        let edit = |at: usize, text: String| crate::Edit { path: None, start: doc.to16(at), end: doc.to16(at), text };
        let in_scope = |i: &&Import| i.visible <= at && at < i.scope_end;
        if let Some(i) = self.imports.iter().filter(in_scope).filter(|i| i.module == Some(module)).max_by_key(|i| i.visible) {
            return Some(match (&i.names, i.append) {
                (Some(names), _) if names.iter().any(|n| n == name) => Vec::new(),
                (Some(_), Some(append)) => vec![edit(append, format!(", {name}"))],
                _ => Vec::new(),
            });
        }
        let package = self.imports.iter().filter(in_scope).filter(|i| i.module.is_none()).max_by_key(|i| i.visible)?;
        let mut edits = Vec::new();
        if let (Some(names), Some(append)) = (&package.names, package.append)
            && !names.iter().any(|n| n == module.name()) {
                edits.push(edit(append, format!(", {}", module.name())));
            }
        edits.push(edit(package.line_end, format!("\n#import {}: {name}", module.name())));
        Some(edits)
    }

    /// What the Typst binding `name` visible at `at` means; None without one.
    pub(crate) fn bound(&self, at: usize, name: &str) -> Option<&Meaning> {
        self.binding(at, name).map(|id| &self.definitions[id].meaning)
    }

    /// The modifier `word` is at `at`: by the binding in scope, else by its spelling (see [`crate::clause::Modifier`]).
    pub(crate) fn modifier(&self, at: usize, word: &str) -> Option<crate::clause::Modifier> {
        match self.bound(at, word) {
            Some(Meaning::Modifier(name)) => Some(crate::clause::Modifier { name: name.clone(), bound: true }),
            Some(_) => None,
            None => crate::clause::spelled(word),
        }
    }

    /// The unit binding `name` visible at `at`.
    pub(crate) fn unit_at(&self, at: usize, name: &str) -> Option<&Meaning> {
        self.bound(at, name).filter(|m| m.is_unit())
    }

    fn binding(&self, at: usize, name: &str) -> Option<usize> {
        self.definitions.iter().enumerate().filter(|(_, b)| b.name == name && b.visible <= at && at < b.scope_end).max_by_key(|(_, b)| b.visible).map(|(id, _)| id)
    }

    /// The operation a use at `at` means.
    pub fn operation(&self, at: usize) -> Option<&str> {
        self.uses.iter().find(|&&(s, e, _)| s <= at && at < e)
            .and_then(|&(_, _, id)| match &self.definitions[id].meaning { Meaning::Operation(op) => Some(op.as_str()), _ => None })
    }

    pub fn inline_references(&self) -> &[InlineReference] { &self.inline }

    pub fn operator_strings(&self) -> &[(usize, usize, String)] { &self.operators }

    /// Uses of solveng bindings (units, constants, operations): where, and what they mean.
    pub fn solveng_uses(&self) -> impl Iterator<Item = (usize, usize, &Meaning)> {
        self.uses.iter().map(|&(s, e, id)| (s, e, &self.definitions[id].meaning)).filter(|(_, _, m)| **m != Meaning::Other)
    }

    /// The use of a solveng binding at `at`: its range, name and meaning.
    pub fn use_at(&self, at: usize) -> Option<(usize, usize, &str, &Meaning)> {
        self.uses.iter().find(|&&(s, e, _)| s <= at && at <= e)
            .map(|&(s, e, id)| (s, e, self.definitions[id].name.as_str(), &self.definitions[id].meaning))
            .filter(|(_, _, _, m)| **m != Meaning::Other)
    }

    /// The solveng bindings visible at `at`, each name once, with what they mean.
    pub fn items(&self, at: usize) -> Vec<(String, &Meaning)> {
        self.definitions.iter().enumerate().filter(|(id, b)| b.meaning != Meaning::Other && self.binding(at, &b.name) == Some(*id))
            .map(|(_, b)| (b.name.clone(), &b.meaning)).collect()
    }

    pub fn contains(&self, start: usize) -> bool { self.uses.iter().any(|&(s, _, _)| s == start) }

    fn target(&self, at: usize) -> Option<usize> {
        self.definitions.iter().position(|b| b.start <= at && at <= b.end)
            .or_else(|| self.uses.iter().find(|&&(s, e, _)| s <= at && at <= e).map(|&(_, _, id)| id))
    }

    /// Whether a Typst binding or a use of one is at `offset`, which Typst's language server owns.
    pub fn at(&self, doc: &Doc, offset: u32) -> bool { self.target(doc.from16(offset as usize)).is_some() }
}

/// Whether a node is in an equation.
fn in_math(node: &LinkedNode) -> bool {
    let mut parent = node.parent();
    while let Some(p) = parent {
        match p.kind() {
            SyntaxKind::Equation => return true,
            SyntaxKind::CodeBlock | SyntaxKind::ContentBlock | SyntaxKind::Closure => return false,
            _ => parent = p.parent(),
        }
    }
    false
}

/// Bare single letters in calculated math that a solveng binding has, where no worksheet variable is defined.
fn bare_letters(a: &crate::Analysis) -> Vec<(usize, usize, String, &'static str)> {
    a.bindings.bare.iter().filter(|&&(start, end, _)| {
        a.calculated_at(start, end).is_some() && crate::definition(&a.doc.s, a.context, a.doc.to16(start)).is_none()
    }).map(|&(start, end, id)| (start, end, a.bindings.definitions[id].name.clone(), a.bindings.definitions[id].meaning.kind())).collect()
}

/// An explicit operation reference math writes where the `functions` module's name would do: `op("tri")` while
/// `#import functions: tri` is in scope or can be added after solveng's import.
struct Nudge {
    start: usize,
    end: usize,
    name: String,
    edits: Vec<crate::Edit>,
}

fn nudges(a: &crate::Analysis) -> Vec<Nudge> {
    let mut out = Vec::new();
    for r in a.bindings.inline_references() {
        if a.calculated_at(r.start, r.end).is_none() { continue; }
        let name = a.doc.sub(r.text_start, r.text_end);
        if !crate::params::automatic_math(&name) { continue; }
        // The name's edits, when it means the same there.
        let edits = match a.bindings.bound(r.start, &name) {
            Some(Meaning::Operation(op)) if *op == name => Some(Vec::new()),
            Some(_) => None,
            None => a.bindings.import_edits(&a.doc, r.start, Module::Functions, &name),
        };
        if let Some(edits) = edits {
            out.push(Nudge { start: r.start, end: r.end, name, edits });
        }
    }
    out
}

/// Names in calculated math no binding resolves that one of solveng's modules exports: `km` without
/// `#import units: km`, which Typst can't find.
fn importable(a: &crate::Analysis) -> Vec<(usize, usize, String, Module)> {
    let modules = [Module::Units, Module::Constants, Module::Functions, Module::Modifiers];
    a.bindings.unbound.iter().filter(|(start, end, _)| a.calculated_at(*start, *end).is_some())
        .filter_map(|(start, end, name)| modules.into_iter().find(|m| m.meaning(name).is_some()).map(|m| (*start, *end, name.clone(), m)))
        .collect()
}

pub(crate) fn problems(a: &crate::Analysis) -> Vec<crate::Problem> {
    let mut problems: Vec<_> = bare_letters(a).into_iter().map(|(start, end, name, kind)| crate::Problem {
        start: a.doc.to16(start), end: a.doc.to16(end), severity: "warning",
        message: format!("use '#{name}' to reference the Typst {kind} binding '{name}'; bare single letters are math symbols"),
    }).collect();
    problems.extend(nudges(a).into_iter().map(|n| crate::Problem {
        start: a.doc.to16(n.start), end: a.doc.to16(n.end), severity: "warning",
        message: format!("use the shorthand {} (#import functions: {})", n.name, n.name),
    }));
    problems
}

/// Quick fixes for the bindings in worksheet math at `offset`: a `#` for a single-letter
/// binding, importing a name of solveng's modules, and an operation's shorthand instead of `op("…")`.
pub fn unit_fixes(document: &str, context: &crate::Context, offset: u32) -> Vec<crate::Conversion> {
    let a = crate::Analysis::new(document, context);
    let at = a.doc.from16(offset as usize);
    let here = |start: usize, end: usize| start <= at && at <= end;
    let mut fixes: Vec<crate::Conversion> = Vec::new();
    fixes.extend(bare_letters(&a).into_iter().filter(|&(start, end, _, _)| here(start, end)).map(|(start, _, name, kind)| crate::Conversion {
        title: format!("Use #{name} to reference the Typst {kind} binding"), start: a.doc.to16(start), end: a.doc.to16(start), text: "#".into(), edits: Vec::new(),
    }));
    for (start, end, name, module) in importable(&a).into_iter().filter(|&(start, end, _, _)| here(start, end)) {
        if let Some(edits) = a.bindings.import_edits(&a.doc, start, module, &name).filter(|e| !e.is_empty()) {
            fixes.push(crate::Conversion { title: format!("Import {name} from solveng's {}", module.name()), start: a.doc.to16(start), end: a.doc.to16(end), text: name, edits });
        }
    }
    fixes.extend(nudges(&a).into_iter().filter(|n| here(n.start, n.end)).map(|n| crate::Conversion {
        title: format!("Use the shorthand {}", n.name), start: a.doc.to16(n.start), end: a.doc.to16(n.end), text: n.name, edits: n.edits,
    }));
    fixes
}

/// Where the lexical scope of a binding ends: the end of the block or closure around it.
fn lexical_end(node: &LinkedNode, doc: &Doc) -> usize {
    let mut parent = node.parent().cloned();
    while let Some(scope) = parent {
        if matches!(scope.kind(), SyntaxKind::ContentBlock | SyntaxKind::CodeBlock | SyntaxKind::Closure) {
            return doc.of_byte(scope.offset() + scope.len());
        }
        parent = scope.parent().cloned();
    }
    doc.c.len()
}
