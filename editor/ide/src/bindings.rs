//! Local Typst bindings take precedence over worksheet unit names in unquoted math: `#let km = unit("km")` and
//! the shorthands imported from solveng's modules, `#import units: km` and `#import functions: tri`.

use typst_syntax::{LinkedNode, SyntaxKind, ast::{self, AstNode, LetBinding, ModuleImport}};
use crate::text::Doc;

struct Binding {
    name: String,
    start: usize,
    end: usize,
    visible: usize,
    scope_end: usize,
    operation: Option<String>,
    unit_name: Option<String>,
    value_range: Option<(usize, usize)>,
    /// A shorthand from solveng's `units` or `functions` module, which follows `builtin-units` and
    /// `builtin-functions` like any implicit name; explicit references (`unit("km")`, `math.op("tri")`) don't.
    imported: bool,
    /// For a name an import lists, the module's name as written there: `s` of `s as test` (the bound name's
    /// range when not renamed).
    original: Option<(usize, usize)>,
}

/// A complete constructor in math that can display just its literal string contents.
#[derive(Clone, Copy)]
pub(crate) struct InlineReference {
    pub start: usize,
    pub end: usize,
    pub text_start: usize,
    pub text_end: usize,
}

#[derive(Default)]
pub(crate) struct Bindings {
    definitions: Vec<Binding>,
    uses: Vec<(usize, usize, usize)>,
    casts: Vec<(usize, usize)>,
    bare: Vec<(usize, usize, usize)>,
    operators: Vec<(usize, usize, String)>,
    inline: Vec<InlineReference>,
    imports: Vec<Import>,
    /// Where imports name solveng's modules (`units` of `#import units: km`): start, end, whether it's `units`.
    modules: Vec<(usize, usize, bool)>,
}

/// An import that shorthands can be added to: solveng's package (`#import "@preview/solveng:1.0.0": *`) or one
/// of its modules (`#import units: km`).
#[derive(Clone, Debug)]
struct Import {
    /// `Some(true)` for `units`, `Some(false)` for `functions`, None for the package.
    module: Option<bool>,
    /// The names it lists; None for `*`.
    names: Option<Vec<String>>,
    /// Where another name goes: after the last one listed.
    append: Option<usize>,
    /// The end of its line, where a new import goes.
    line_end: usize,
    visible: usize,
    scope_end: usize,
}

impl Bindings {
    pub fn new(doc: &Doc) -> Self {
        fn visit(node: &LinkedNode, f: &mut impl FnMut(&LinkedNode)) {
            f(node);
            for child in node.children() { visit(&child, f); }
        }
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
            let operation = binding.init().and_then(|value| {
                let source = value.to_untyped().clone().full_text();
                operator_name(&source).or_else(|| result.binding(doc.of_byte(node.offset()), source.trim())
                    .and_then(|id| result.definitions[id].operation.clone()))
            });
            let unit_name = binding.init().and_then(|value| {
                let source = value.to_untyped().clone().full_text();
                unit_constructor(&source).or_else(|| result.binding(doc.of_byte(node.offset()), source.trim())
                    .and_then(|id| result.definitions[id].unit_name.clone()))
            });
            let value_range = unit_name.as_ref().and_then(|_| {
                let mut range = None;
                visit(node, &mut |child| {
                    if range.is_none() && child.kind() == SyntaxKind::Str {
                        range = Some((doc.of_byte(child.offset()) + 1, doc.of_byte(child.offset() + child.len()) - 1));
                    }
                });
                range
            });
            for name in binding.kind().bindings() {
                visit(node, &mut |leaf| {
                    if std::ptr::eq(leaf.get(), name.to_untyped()) {
                        result.definitions.push(Binding { name: name.as_str().into(), start: doc.of_byte(leaf.offset()), end: doc.of_byte(leaf.offset() + leaf.len()), visible: doc.of_byte(node.offset() + node.len()), scope_end, operation: operation.clone(), unit_name: unit_name.clone(), value_range, imported: false, original: None });
                    }
                });
            }
        });
        visit(&root, &mut |node| {
            if matches!(node.kind(), SyntaxKind::FuncCall | SyntaxKind::MathCall) {
                let source = node.get().clone().full_text();
                let callee = source.split_once('(').map(|(name, _)| name.trim());
                let operation = matches!(callee, Some("math.op" | "operation")) || callee == Some("op") && node.kind() == SyntaxKind::MathCall;
                let inline = node.kind() == SyntaxKind::MathCall && matches!(callee, Some("op" | "unit"));
                if operation || inline {
                    if let Some(args) = node.children().find(|c| matches!(c.kind(), SyntaxKind::Args | SyntaxKind::MathArgs)) {
                        if let Some(string) = args.children().find(|c| matches!(c.kind(), SyntaxKind::Str | SyntaxKind::Error) && c.get().clone().full_text().starts_with('"')) {
                            let source = string.get().clone().full_text();
                            let closed = source.len() > 1 && source.ends_with('"');
                            let start = doc.of_byte(string.offset()) + 1;
                            let end = doc.of_byte(string.offset() + string.len()) - usize::from(closed);
                            let (name, _) = crate::text::unescape(&doc.c[start..end]);
                            if operation { result.operators.push((start, end, name.into_iter().collect())); }
                            if inline && closed && end > start && args.children().any(|c| c.kind() == SyntaxKind::RightParen) {
                                result.inline.push(InlineReference { start: doc.of_byte(node.offset()), end: doc.of_byte(node.offset() + node.len()), text_start: start, text_end: end });
                            }
                        }
                    }
                }
            }
            let start = doc.of_byte(node.offset());
            let end = doc.of_byte(node.offset() + node.len());
            if node.kind() == SyntaxKind::MathIdent && node.get().leaf_text() == "as"
                && start > 0 && doc.c[start - 1].is_whitespace() && doc.c.get(end).is_some_and(|c| c.is_whitespace()) {
                result.casts.push((start, end));
            }
            if node.kind() == SyntaxKind::MathText && node.get().leaf_text().chars().count() == 1 {
                let end = doc.of_byte(node.offset() + node.len());
                if doc.c.get(end) != Some(&'_') && (start == 0 || doc.c[start - 1] != '_')
                    && node.parent_kind() != Some(SyntaxKind::MathAttach)
                    && let Some(id) = result.binding(start, node.get().leaf_text().as_str())
                    && result.definitions[id].unit_name.is_some() {
                        result.bare.push((start, end, id));
                    }
            }
            if !matches!(node.kind(), SyntaxKind::Ident | SyntaxKind::MathIdent) { return; }
            if result.definitions.iter().any(|b| b.start == start) { return; }
            if node.parent_kind() == Some(SyntaxKind::FieldAccess) && node.prev_sibling().is_some() { return; }
            if node.parent_kind() == Some(SyntaxKind::Named) && node.prev_sibling().is_none() { return; }
            if let Some(id) = result.binding(start, node.get().leaf_text().as_str()) {
                result.uses.push((start, doc.of_byte(node.offset() + node.len()), id));
            }
        });
        result
    }

    /// `#import units: km, mA as milli` or `#import functions: *`: shorthands, visible like other imports.
    fn import(&mut self, node: &LinkedNode, import: ModuleImport, doc: &Doc) {
        let visible = doc.of_byte(node.offset() + node.len());
        let scope_end = lexical_end(node, doc);
        let listed: Option<Vec<String>> = match import.imports() {
            Some(ast::Imports::Items(items)) => Some(items.iter().map(|i| i.bound_name().as_str().to_string()).collect()),
            _ => None,
        };
        let append = node.children().find(|c| c.kind() == SyntaxKind::ImportItems).map(|items| doc.of_byte(items.offset() + items.len()));
        let line_end = (visible..doc.c.len()).find(|&i| doc.c[i] == '\n').unwrap_or(doc.c.len());
        let module = match import.source() {
            ast::Expr::Ident(ident) => ident.as_str().to_string(),
            ast::Expr::FieldAccess(access) => access.field().as_str().to_string(),
            ast::Expr::Str(source) => {
                // solveng's package, whose `units` and `functions` modules hold the shorthands
                if source.get().contains("solveng") && import.imports().is_some() {
                    self.imports.push(Import { module: None, names: listed, append, line_end, visible, scope_end });
                }
                return;
            }
            _ => return,
        };
        let units = match module.as_str() { "units" => true, "functions" => false, _ => return };
        if let Some(source) = node.children().find(|c| matches!(c.kind(), SyntaxKind::Ident | SyntaxKind::FieldAccess)) {
            let end = doc.of_byte(source.offset() + source.len());
            self.modules.push((end - module.chars().count(), end, units));
        }
        if import.imports().is_some() {
            self.imports.push(Import { module: Some(units), names: listed.clone(), append, line_end, visible, scope_end });
        }
        let mut push = |name: &str, original: &str, start: usize, end: usize, written: Option<(usize, usize)>| {
            let (unit_name, operation) = if units {
                (crate::units::shorthand(original).then(|| original.to_string()), None)
            } else {
                (None, crate::params::automatic_math(original).then(|| original.to_string()))
            };
            if unit_name.is_some() || operation.is_some() {
                self.definitions.push(Binding { name: name.into(), start, end, visible, scope_end, operation, unit_name, value_range: None, imported: true, original: written });
            }
        };
        match import.imports() {
            // Not at a place in the text, so nothing there names them.
            Some(ast::Imports::Wildcard) => {
                let names = if units { crate::units::shorthands() } else { crate::params::automatic_names() };
                for name in names { push(&name, &name, usize::MAX, usize::MAX, None); }
            }
            Some(ast::Imports::Items(items)) => {
                for item in items.iter() {
                    let leaf_range = |ident: &typst_syntax::SyntaxNode| {
                        let mut range = (usize::MAX, usize::MAX);
                        visit_leaf(node, ident, &mut |leaf| range = (doc.of_byte(leaf.offset()), doc.of_byte(leaf.offset() + leaf.len())));
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

    /// The shorthand imported from solveng's modules at `at`: in its import (the name bound, or the module's
    /// name it renames) or where it's used.
    fn shorthand_at(&self, at: usize) -> Option<usize> {
        let within = |(s, e): (usize, usize)| s <= at && at <= e;
        self.definitions.iter().position(|b| b.imported && b.original.is_some() && (within((b.start, b.end)) || b.original.is_some_and(within)))
            .or_else(|| self.uses.iter().find(|&&(s, e, id)| within((s, e)) && self.definitions[id].imported).map(|&(_, _, id)| id))
    }

    /// Where the shorthand at `at` is imported: the name it's bound to.
    pub(crate) fn shorthand_definition(&self, at: usize) -> Option<(usize, usize)> {
        self.listed_at(at).map(|id| (self.definitions[id].start, self.definitions[id].end))
    }

    /// [`Self::shorthand_at`], when its import lists it (`*` binds it at no place in the text).
    fn listed_at(&self, at: usize) -> Option<usize> {
        self.shorthand_at(at).filter(|&id| self.definitions[id].original.is_some())
    }

    /// The shorthand at `at` where it's imported and every use.
    pub(crate) fn shorthand_usages(&self, at: usize) -> Vec<(usize, usize)> {
        let Some(id) = self.listed_at(at) else { return Vec::new() };
        std::iter::once((self.definitions[id].start, self.definitions[id].end))
            .chain(self.uses.iter().filter(|u| u.2 == id).map(|&(s, e, _)| (s, e))).collect()
    }

    /// The module name at `at` in an import of solveng's modules: its range and whether it's `units`.
    pub(crate) fn module_at(&self, at: usize) -> Option<(usize, usize, bool)> {
        self.modules.iter().find(|&&(s, e, _)| s <= at && at <= e).copied()
    }

    /// The shorthand at `at`, for its hover: the range there, the name it's bound to, the module's name for it and
    /// whether it's from `units`.
    pub(crate) fn shorthand_info(&self, at: usize) -> Option<(usize, usize, String, String, bool)> {
        let id = self.shorthand_at(at)?;
        let b = &self.definitions[id];
        let within = |(s, e): (usize, usize)| s <= at && at <= e;
        let range = self.uses.iter().find(|&&(s, e, i)| i == id && within((s, e))).map(|&(s, e, _)| (s, e))
            .or_else(|| b.original.filter(|o| within(*o))).or_else(|| b.original.map(|_| (b.start, b.end)))?;
        let original = b.unit_name.clone().or_else(|| b.operation.clone()).unwrap_or_else(|| b.name.clone());
        Some((range.0, range.1, b.name.clone(), original, b.unit_name.is_some()))
    }

    /// The name at `at` that renaming the shorthand there changes.
    pub(crate) fn shorthand_target(&self, at: usize) -> Option<(usize, usize, String)> {
        let id = self.listed_at(at)?;
        let b = &self.definitions[id];
        let (s, e) = self.uses.iter().find(|&&(s, e, i)| i == id && s <= at && at <= e).map(|&(s, e, _)| (s, e)).unwrap_or((b.start, b.end));
        Some((s, e, b.name.clone()))
    }

    /// Renames the shorthand at `at` and its uses to `name`, an alias in its import (`s as test`); back to the
    /// module's own name the alias goes. Edits as (start, end, text); None if `name` isn't a Typst identifier.
    pub(crate) fn shorthand_rename(&self, doc: &Doc, at: usize, name: &str) -> Option<Vec<(usize, usize, String)>> {
        let id = self.listed_at(at)?;
        if !typst_syntax::is_ident(name) { return None; }
        let b = &self.definitions[id];
        let (os, oe) = b.original?;
        let original = doc.sub(os, oe);
        let import = if name == original {
            (os, b.end, original)
        } else if (os, oe) != (b.start, b.end) {
            (b.start, b.end, name.to_string())
        } else {
            (os, oe, format!("{original} as {name}"))
        };
        Some(std::iter::once(import).chain(self.uses.iter().filter(|u| u.2 == id).map(|&(s, e, _)| (s, e, name.to_string()))).collect())
    }

    /// The names imports bind from solveng's modules: (start, end, from `units`), and where a renamed one's module
    /// name is (`s` of `s as test`), which isn't one.
    pub(crate) fn shorthand_imports(&self) -> impl Iterator<Item = ((usize, usize), bool, Option<(usize, usize)>)> + '_ {
        self.definitions.iter().filter(|b| b.imported && b.original.is_some())
            .map(|b| ((b.start, b.end), b.unit_name.is_some(), b.original.filter(|o| *o != (b.start, b.end))))
    }

    /// The edits that make the shorthand `name` of the `units` module (`functions` unless `units`) available at
    /// `at`: added to a `#import units: …` in scope, else a new one after solveng's import (which gets the module
    /// if it lists names). Empty when it's already there; None when solveng isn't imported in scope.
    pub(crate) fn import_edits(&self, doc: &Doc, at: usize, units: bool, name: &str) -> Option<Vec<crate::Edit>> {
        let edit = |at: usize, text: String| crate::Edit { path: None, start: doc.to16(at) as u32, end: doc.to16(at) as u32, text };
        let module = if units { "units" } else { "functions" };
        let in_scope = |i: &&Import| i.visible <= at && at < i.scope_end;
        if let Some(i) = self.imports.iter().filter(in_scope).filter(|i| i.module == Some(units)).max_by_key(|i| i.visible) {
            return Some(match (&i.names, i.append) {
                (Some(names), _) if names.iter().any(|n| n == name) => Vec::new(),
                (Some(_), Some(append)) => vec![edit(append, format!(", {name}"))],
                _ => Vec::new(),
            });
        }
        let package = self.imports.iter().filter(in_scope).filter(|i| i.module.is_none()).max_by_key(|i| i.visible)?;
        let mut edits = Vec::new();
        if let (Some(names), Some(append)) = (&package.names, package.append)
            && !names.iter().any(|n| n == module) {
                edits.push(edit(append, format!(", {module}")));
            }
        edits.push(edit(package.line_end, format!("\n#import {module}: {name}")));
        Some(edits)
    }

    /// The Typst binding `name` has at `at`: a shorthand for this unit (`Some(Some(unit))`), something else
    /// (`Some(None)`), or none.
    pub(crate) fn bound(&self, at: usize, name: &str) -> Option<Option<&str>> {
        self.binding(at, name).map(|id| self.definitions[id].unit_name.as_deref().or(self.definitions[id].operation.as_deref()))
    }

    fn binding(&self, at: usize, name: &str) -> Option<usize> {
        self.definitions.iter().enumerate().filter(|(_, b)| b.name == name && b.visible <= at && at < b.scope_end).max_by_key(|(_, b)| b.visible).map(|(id, _)| id)
    }

    /// Whether the binding used at `at` is a shorthand imported from solveng's modules.
    pub fn imported_at(&self, at: usize) -> bool {
        self.uses.iter().find(|&&(s, e, _)| s <= at && at < e).is_some_and(|&(_, _, id)| self.definitions[id].imported)
    }

    pub fn operation(&self, at: usize) -> Option<&str> {
        self.uses.iter().find(|&&(s, e, _)| s <= at && at < e)
            .and_then(|&(_, _, id)| self.definitions[id].operation.as_deref())
    }

    pub fn inline_references(&self) -> &[InlineReference] { &self.inline }

    pub fn operator_strings(&self) -> &[(usize, usize, String)] { &self.operators }

    /// Uses of unit shorthands: where, the unit, and whether it was imported (see [`Binding::imported`]).
    pub fn unit_uses(&self) -> impl Iterator<Item = (usize, usize, &str, bool)> {
        self.uses.iter().filter_map(|&(start, end, id)| self.definitions[id].unit_name.as_deref().map(|name| (start, end, name, self.definitions[id].imported)))
    }

    pub fn unit_shortcut(&self, at: usize) -> Option<(&str, &str, usize, usize)> {
        let id = self.target(at).or_else(|| self.definitions.iter().position(|b| b.value_range.is_some_and(|(s, e)| s <= at && at <= e)))?;
        let binding = &self.definitions[id];
        let unit = binding.unit_name.as_deref()?;
        let (start, end) = self.uses.iter().find(|&&(s, e, _)| s <= at && at <= e).map(|&(s, e, _)| (s, e))
            .or_else(|| binding.value_range.filter(|&(s, e)| s <= at && at <= e)).unwrap_or((binding.start, binding.end));
        Some((&binding.name, unit, start, end))
    }

    pub fn bare_units(&self) -> impl Iterator<Item = (usize, usize, &str)> {
        self.bare.iter().map(|&(start, end, id)| (start, end, self.definitions[id].name.as_str()))
    }

    pub fn unit_items(&self, at: usize) -> Vec<(String, String)> {
        self.definitions.iter().enumerate().filter(|(id, b)| self.binding(at, &b.name) == Some(*id))
            .filter_map(|(_, b)| b.unit_name.as_ref().map(|unit| (b.name.clone(), unit.clone()))).collect()
    }

    pub fn operation_uses(&self) -> impl Iterator<Item = (usize, usize, &str, bool)> {
        self.uses.iter().filter_map(|&(start, end, id)| self.definitions[id].operation.as_deref().map(|name| (start, end, name, self.definitions[id].imported)))
    }

    pub fn operation_items(&self, at: usize) -> Vec<(String, String)> {
        self.definitions.iter().enumerate().filter(|(id, b)| self.binding(at, &b.name) == Some(*id))
            .filter_map(|(_, b)| b.operation.as_ref().map(|op| (b.name.clone(), op.clone()))).collect()
    }

    pub fn contains(&self, start: usize) -> bool { self.uses.iter().any(|&(s, _, _)| s == start) }

    fn target(&self, at: usize) -> Option<usize> {
        self.definitions.iter().position(|b| b.start <= at && at <= b.end)
            .or_else(|| self.uses.iter().find(|&&(s, e, _)| s <= at && at <= e).map(|&(_, _, id)| id))
    }

    pub fn at(&self, doc: &Doc, offset: u32) -> bool { self.target(doc.from16(offset as usize)).is_some() }
}

fn unresolved_units(a: &crate::Analysis) -> Vec<(usize, usize, String)> {
    a.bindings.bare_units().filter(|&(start, _, _)| {
        a.elements.iter().any(|e| e.math && a.active(e) && e.body_start <= start && start < e.body_end)
            && crate::definition(&a.doc.s, a.context, a.doc.to16(start)).is_none()
    }).map(|(start, end, name)| (start, end, name.into())).collect()
}

fn bare_casts<'a>(a: &'a crate::Analysis<'_>) -> impl Iterator<Item = (usize, usize)> + 'a {
    a.bindings.casts.iter().copied().filter(move |&(start, _)| {
        a.elements.iter().any(|element| element.math && a.active(element) && element.body_start <= start && start < element.body_end)
    })
}

/// A built-in unit or operation math writes as a string where its shorthand would do: `"km"`, `unit("km")` or
/// `op("tri")` while `#import units: km` is in scope or can be added after solveng's import. Single letters stay
/// quoted (`"s"`), and the document's own units, hidden ones and taken names have no shorthand.
struct Nudge {
    start: usize,
    end: usize,
    name: String,
    units: bool,
    edits: Vec<crate::Edit>,
}

fn nudges(a: &crate::Analysis) -> Vec<Nudge> {
    let mut out = Vec::new();
    let defs = crate::complete::definitions(&a.doc, &a.elements, &a.calls, &|e| a.active(e));
    let math = |at: usize| a.elements.iter().any(|e| e.math && a.active(e) && e.body_start <= at && at < e.body_end);
    // The shorthand's edits, when it means the same there.
    let available = |at: usize, name: &str, units: bool| match a.bindings.bound(at, name) {
        Some(Some(meaning)) if meaning == name => Some(Vec::new()),
        Some(_) => None,
        None => a.bindings.import_edits(&a.doc, at, units, name),
    };
    let unit_ok = |at: usize, name: &str| {
        let units = a.units_at(at);
        name.chars().count() > 1 && crate::units::shorthand(name) && crate::units::visible(name, &units) && !units.iter().any(|u| u.name == name)
    };
    let references = a.reference_strings();
    for e in a.elements.iter().filter(|e| e.math && a.active(e)) {
        let units = a.units_at(e.body_start);
        let toks = crate::tokens::math_tokens(&a.doc.c[e.body_start..e.body_end], &|w| crate::units::visible(w, &units), &|w| defs.iter().any(|d| d.base() == w));
        for (s, l, kind) in toks {
            let (start, end) = (e.body_start + s, e.body_start + s + l);
            // the string of `unit("km")` counts with its call
            if kind != crate::TokenKind::UNIT || a.doc.c[start] != '"' || references.iter().any(|&(s, e)| s <= start + 1 && start < e) { continue; }
            let name = crate::resolved_name(&a.doc.sub(start + 1, end.saturating_sub(1)));
            if !unit_ok(start, &name) { continue; }
            if let Some(edits) = available(start, &name, true) { out.push(Nudge { start, end, name, units: true, edits }); }
        }
    }
    for r in a.bindings.inline_references() {
        if !math(r.start) { continue; }
        let name = a.doc.sub(r.text_start, r.text_end);
        let callee = a.doc.sub(r.start, r.text_start);
        let units = callee.trim_start().starts_with("unit");
        let ok = if units { unit_ok(r.start, &name) } else { crate::params::automatic_math(&name) && a.builtin_functions_at(r.start) };
        if let Some(edits) = ok.then(|| available(r.start, &name, units)).flatten() {
            out.push(Nudge { start: r.start, end: r.end, name, units, edits });
        }
    }
    out
}

pub(crate) fn problems(a: &crate::Analysis) -> Vec<crate::Problem> {
    let mut problems: Vec<_> = unresolved_units(a).into_iter().map(|(start, end, name)| crate::Problem {
        start: a.doc.to16(start), end: a.doc.to16(end), severity: "warning",
        message: format!("use '#{name}' to reference the Typst unit binding '{name}'; bare single letters are math symbols"),
    }).collect();
    problems.extend(nudges(a).into_iter().map(|n| crate::Problem {
        start: a.doc.to16(n.start), end: a.doc.to16(n.end), severity: "warning",
        message: format!("use the shorthand {} (#import {}: {})", n.name, if n.units { "units" } else { "functions" }, n.name),
    }));
    problems.extend(bare_casts(a).map(|(start, end)| crate::Problem {
        start: a.doc.to16(start), end: a.doc.to16(end), severity: "warning",
        message: "write \"as\" in worksheet math to cast to a unit".into(),
    }));
    problems
}

/// Fix unit references and cast syntax in worksheet math.
pub fn unit_fixes(document: &str, context: &crate::Context, offset: u32) -> Vec<crate::Conversion> {
    let a = crate::Analysis::new(document, context);
    let at = a.doc.from16(offset as usize);
    let mut fixes: Vec<_> = bare_casts(&a).filter(|&(start, end)| start <= at && at <= end).map(|(start, end)| crate::Conversion {
        title: "Quote \"as\" for a unit cast".into(), start: a.doc.to16(start), end: a.doc.to16(end), text: "\"as\"".into(), edits: Vec::new()
    }).collect();
    for token in crate::document_tokens(document, context).iter().filter(|token| token.kind == crate::TokenKind::VARIABLE) {
        let start = a.doc.from16(token.start as usize);
        let end = a.doc.from16((token.start + token.length) as usize);
        if !(start <= at && at <= end) || a.doc.c.get(start) == Some(&'"') || start > 0 && a.doc.c[start - 1] == '#'
            || a.bindings.contains(start) { continue; }
        if !a.elements.iter().any(|element| element.math && a.active(element) && element.body_start <= start && end <= element.body_end) { continue; }
        let name = crate::resolved_name(&a.doc.sub(start, end));
        if crate::units::visible(&name, &a.units_at(start))
            && crate::definition(document, context, token.start).is_none() {
            fixes.push(crate::Conversion { title: format!("Quote \"{name}\" as a unit"), start: token.start, end: token.start + token.length, text: format!("\"{name}\""), edits: Vec::new() });
        }
    }
    fixes.extend(unresolved_units(&a).into_iter().filter(|&(start, end, _)| start <= at && at <= end).map(|(start, _, name)| crate::Conversion {
        title: format!("Use #{name} to reference the Typst unit binding"), start: a.doc.to16(start), end: a.doc.to16(start), text: "#".into(), edits: Vec::new(),
    }));
    fixes.extend(nudges(&a).into_iter().filter(|n| n.start <= at && at <= n.end).map(|n| crate::Conversion {
        title: format!("Use the shorthand {}", n.name), start: a.doc.to16(n.start), end: a.doc.to16(n.end), text: n.name, edits: n.edits,
    }));
    fixes.extend(overlay_bindings(&a, at));
    fixes
}

/// Recognize explicit operator constructors, preserving the exact string for Typst to evaluate.
fn operator_name(source: &str) -> Option<String> {
    let (callee, args) = source.split_once('(')?;
    if !matches!(callee.trim(), "math.op" | "operation") { return None; }
    let args = args.trim_start();
    let (name, _) = solveng_names::quoted(&args.chars().collect::<Vec<_>>(), 0).ok()?;
    Some(name)
}

/// Typst scopes are lexical: generate declarations in the source for an overlay's names.
fn overlay_bindings(a: &crate::Analysis, at: usize) -> Vec<crate::Conversion> {
    fn visit(node: &LinkedNode, a: &crate::Analysis, at: usize, out: &mut Vec<crate::Conversion>) {
        let start = a.doc.of_byte(node.offset());
        let end = a.doc.of_byte(node.offset() + node.len());
        if !(start <= at && at <= end) { return; }
        if node.kind() == SyntaxKind::FuncCall {
            let call = a.calls.iter().find(|c| start <= c.args_start && c.args_end <= end);
            if call.is_some_and(|c| c.function.name == "worksheet") {
                if let Some(args) = node.children().find(|c| c.kind() == SyntaxKind::Args) {
                    let mut units = Vec::new();
                    for named in args.children().filter(|c| c.kind() == SyntaxKind::Named) {
                        let source = named.get().clone().full_text();
                        if source.split_once(':').is_some_and(|(name, _)| name.trim() == "units") {
                            crate::units::custom_units(&source, &mut units);
                        }
                    }
                    let names: Vec<_> = units.into_iter().filter(|u| u.value != "none"
                        && !u.name.is_empty() && typst_syntax::is_ident(&u.name)).collect();
                    if !names.is_empty() {
                        let body = args.children().find(|c| c.kind() == SyntaxKind::ContentBlock);
                        let mut position = body.as_ref().map(|c| a.doc.of_byte(c.offset()) + 1);
                        if position.is_none() {
                            let mut parent = node.parent().cloned();
                            while let Some(p) = parent {
                                if matches!(p.kind(), SyntaxKind::ShowRule | SyntaxKind::LetBinding) {
                                    position = Some(a.doc.of_byte(p.offset() + p.len())); break;
                                }
                                parent = p.parent().cloned();
                            }
                        }
                        if let Some(position) = position {
                            let scope_end = body.as_ref().map_or_else(|| lexical_end(node, &a.doc), |c| a.doc.of_byte(c.offset() + c.len()));
                            let text = names.iter().filter(|u| a.bindings.binding(position + 1, &u.name).is_none()
                                && !a.bindings.definitions.iter().any(|b| b.name == u.name && b.start >= position
                                    && b.scope_end == scope_end && b.unit_name.as_deref() == Some(u.name.as_str())))
                                .map(|u| format!("#let {} = unit(\"{}\")\n", u.name, u.name)).collect::<String>();
                            if !text.is_empty() {
                                out.push(crate::Conversion { title: "Declare unit overlay bindings".into(),
                                    start: a.doc.to16(position), end: a.doc.to16(position), text: format!("\n{text}"), edits: Vec::new() });
                            }
                        }
                    }
                }
            }
        }
        for child in node.children() { visit(&child, a, at, out); }
    }
    let syntax = typst_syntax::parse(&a.doc.s);
    let mut out = Vec::new();
    visit(&LinkedNode::new(&syntax), a, at, &mut out);
    out
}

fn visit_leaf(node: &LinkedNode, target: &typst_syntax::SyntaxNode, f: &mut impl FnMut(&LinkedNode)) {
    if std::ptr::eq(node.get(), target) { f(node); }
    for child in node.children() { visit_leaf(&child, target, f); }
}

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

fn unit_constructor(source: &str) -> Option<String> {
    let (callee, args) = source.split_once('(')?;
    if callee.trim() != "unit" { return None; }
    let (name, _) = solveng_names::quoted(&args.trim_start().chars().collect::<Vec<_>>(), 0).ok()?;
    Some(name)
}
