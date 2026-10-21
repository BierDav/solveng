//! Typst operator bindings retain their source identity independently of calculation names.

use std::collections::HashMap;
use typst_syntax::{LinkedNode, SyntaxKind, ast};
use crate::{Analysis, Context, Edit, Item, ItemKind, Location, RenameTarget, TokenKind as K};
use crate::text::Doc;

#[derive(Clone)]
struct Binding {
    name: String,
    key: String,
    operation: String,
    start: usize,
    end: usize,
}

#[derive(Clone)]
struct Use {
    start: usize,
    end: usize,
    binding: usize,
    math: bool,
}

struct Reader<'a> {
    a: &'a Analysis<'a>,
    bindings: Vec<Binding>,
    uses: Vec<Use>,
    visible: HashMap<String, usize>,
    scopes: Vec<(usize, usize, HashMap<String, usize>)>,
}

fn key(name: &str) -> String { crate::resolved_name(name) }

impl Reader<'_> {
    fn visit(&mut self, node: &LinkedNode) {
        let doc = &self.a.doc;
        let (start, end) = (doc.of_byte(node.offset()), doc.of_byte(node.offset() + node.len()));
        match node.kind() {
            SyntaxKind::ContentBlock | SyntaxKind::CodeBlock => {
                let outer = self.visible.clone();
                for child in node.children() { self.visit(&child); }
                self.visible = outer;
                return;
            }
            SyntaxKind::LetBinding => {
                let parts: Vec<_> = node.children().filter(|c| !c.kind().is_trivia()).collect();
                let Some(name) = parts.get(1).filter(|n| n.kind() == SyntaxKind::Ident) else {
                    for child in parts.iter().skip(1) { self.visit(child); }
                    if let Some(binding) = node.get().cast::<ast::LetBinding>() {
                        for name in binding.kind().bindings() { self.visible.remove(&key(name.get())); }
                    }
                    return;
                };
                for child in parts.iter().skip(2) { self.visit(child); }
                let name_text = name.get().leaf_text().to_string();
                let name_key = key(&name_text);
                self.visible.remove(&name_key);
                let operation = parts.last().filter(|n| n.kind() == SyntaxKind::FuncCall).and_then(|call| {
                    let callee = call.children().next()?.get().clone().full_text();
                    if callee.chars().filter(|c| !c.is_whitespace()).collect::<String>() != "math.op" { return None; }
                    let args = call.children().find(|n| n.kind() == SyntaxKind::Args)?;
                    args.children().find_map(|n| n.get().cast::<ast::Str>().map(|s| s.get().to_string()))
                });
                if let Some(operation) = operation {
                    let (s, e) = (doc.of_byte(name.offset()), doc.of_byte(name.offset() + name.len()));
                    let id = self.bindings.len();
                    self.bindings.push(Binding { name: name_text, key: name_key.clone(), operation, start: s, end: e });
                    self.uses.push(Use { start: s, end: e, binding: id, math: false });
                    self.visible.insert(name_key, id);
                }
                return;
            }
            SyntaxKind::Closure => {
                let outer = self.visible.clone();
                if let Some(closure) = node.get().cast::<ast::Closure>() {
                    if let Some(name) = closure.name() { self.visible.remove(&key(name.get())); }
                    for param in closure.params().children() {
                        let names = match param {
                            ast::Param::Pos(pattern) => pattern.bindings(),
                            ast::Param::Named(named) => vec![named.name()],
                            ast::Param::Spread(spread) => spread.sink_ident().into_iter().collect(),
                        };
                        for name in names { self.visible.remove(&key(name.get())); }
                    }
                }
                if let Some(body) = node.children().last() { self.visit(&body); }
                self.visible = outer;
                return;
            }
            SyntaxKind::Equation => {
                self.scopes.push((start, end, self.visible.clone()));
                let body_start = start + 1;
                let body_end = if doc.c.get(end.saturating_sub(1)) == Some(&'$') { end - 1 } else { end };
                for (s, len, kind) in crate::tokens::math_tokens(&doc.c[body_start..body_end], &|_| false) {
                    if !matches!(kind, K::VARIABLE | K::FUNCTION) { continue; }
                    let (s, e) = (body_start + s, body_start + s + len);
                    let spelling = doc.sub(s, e);
                    // Quoted math is a calculation name, never a Typst variable.
                    if spelling.starts_with('"') { continue; }
                    if let Some(&binding) = self.visible.get(&key(&spelling)) {
                        self.uses.push(Use { start: s, end: e, binding, math: true });
                    }
                }
                return;
            }
            SyntaxKind::Ident => {
                let field = node.parent_kind() == Some(SyntaxKind::FieldAccess) && node.index() > 0;
                let label = node.parent_kind() == Some(SyntaxKind::Named) && node.index() == 0;
                if field || label { return; }
                if let Some(&binding) = self.visible.get(&key(node.get().leaf_text())) {
                    self.uses.push(Use { start, end, binding, math: false });
                }
            }
            _ => {}
        }
        self.scopes.push((start, end, self.visible.clone()));
        for child in node.children() { self.visit(&child); }
    }
}

fn read<'a>(a: &'a Analysis<'a>) -> Reader<'a> {
    let mut reader = Reader { a, bindings: Vec::new(), uses: Vec::new(), visible: HashMap::new(), scopes: Vec::new() };
    let root = typst_syntax::parse(&a.doc.s);
    reader.visit(&LinkedNode::new(&root));
    reader
}

fn location(doc: &Doc, u: &Use) -> Location {
    Location { path: None, start: doc.to16(u.start), end: doc.to16(u.end) }
}

fn target(r: &Reader, offset: u32) -> Option<usize> {
    let at = r.a.doc.from16(offset as usize);
    r.uses.iter().find(|u| u.start <= at && at <= u.end).map(|u| u.binding)
}

pub(crate) fn definition(a: &Analysis, offset: u32) -> Option<Location> {
    let r = read(a);
    let b = &r.bindings[target(&r, offset)?];
    Some(Location { path: None, start: a.doc.to16(b.start), end: a.doc.to16(b.end) })
}

pub(crate) fn usages(a: &Analysis, offset: u32) -> Option<Vec<Location>> {
    let r = read(a);
    let id = target(&r, offset)?;
    Some(r.uses.iter().filter(|u| u.binding == id).map(|u| location(&a.doc, u)).collect())
}

pub(crate) fn rename_target(a: &Analysis, offset: u32) -> Option<RenameTarget> {
    let r = read(a);
    target(&r, offset)?;
    let at = a.doc.from16(offset as usize);
    let u = r.uses.iter().find(|u| u.start <= at && at <= u.end)?;
    Some(RenameTarget { start: a.doc.to16(u.start), end: a.doc.to16(u.end), name: a.doc.sub(u.start, u.end) })
}

pub(crate) fn rename(a: &Analysis, offset: u32, name: &str) -> Option<Vec<Edit>> {
    let r = read(a);
    let id = target(&r, offset)?;
    let requested = name.trim();
    let normalized = key(requested);
    let identifier = |name: &str| {
        let parsed = typst_syntax::parse_code(name);
        parsed.children().count() == 1 && parsed.children().next().is_some_and(|n| n.kind() == SyntaxKind::Ident)
    };
    let name = if identifier(requested) { requested }
        else if requested.contains("\"\"") && !requested.replace("\"\"", "").contains('"') && identifier(&normalized) { &normalized }
        else { return Some(Vec::new()); };
    if r.bindings.iter().enumerate().any(|(i, b)| i != id && b.key == normalized) { return Some(Vec::new()); }
    Some(r.uses.iter().filter(|u| u.binding == id).map(|u| Edit {
        path: None, start: a.doc.to16(u.start), end: a.doc.to16(u.end), text: name.into(),
    }).collect())
}

pub(crate) fn hover(a: &Analysis, offset: u32) -> Option<crate::Hover> {
    let r = read(a);
    let id = target(&r, offset)?;
    let b = &r.bindings[id];
    let at = a.doc.from16(offset as usize);
    let u = r.uses.iter().find(|u| u.start <= at && at <= u.end)?;
    Some(crate::Hover { start: a.doc.to16(u.start), end: a.doc.to16(u.end), markdown: format!("`{}` (Typst operator)\n\n`math.op({:?})`", b.name, b.operation) })
}

pub(crate) fn completions(a: &Analysis, at: usize) -> Vec<Item> {
    let r = read(a);
    let visible = r.scopes.iter().rev().find(|(s, e, _)| *s <= at && at <= *e).map(|(_, _, v)| v).unwrap_or(&r.visible);
    visible.values().map(|&i| {
        let b = &r.bindings[i];
        let mut item = Item::new(&format!("{}(x)", b.name), ItemKind::FUNCTION, &format!("Typst: math.op({:?})", b.operation), &format!("{}($0)", b.name));
        item.typst = true;
        item
    }).collect()
}

/// Canonical Typst identifiers for compilation; math ranges must be read as identifiers, including literal symbols.
pub fn operator_edits(document: &str) -> Vec<OperatorEdit> {
    let context = Context::default();
    let a = Analysis::new(document, &context);
    let r = read(&a);
    let canonical: Vec<_> = r.bindings.iter().enumerate().map(|(id, b)| {
        r.uses.iter().any(|u| u.binding == id && u.math && a.doc.sub(u.start, u.end) != b.name)
    }).collect();
    r.uses.iter().map(|u| {
        let b = &r.bindings[u.binding];
        OperatorEdit {
            start: a.doc.to16(u.start), end: a.doc.to16(u.end),
            name: if canonical[u.binding] { b.key.clone() } else { b.name.clone() }, math: u.math,
        }
    }).collect()
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct OperatorEdit {
    pub start: u32,
    pub end: u32,
    pub name: String,
    pub math: bool,
}

/// Source-level operator definition for an unresolved intrinsic in math.
pub fn operator_declaration(name: &str) -> Option<String> {
    (name != "cases" && crate::complete::FUNCTIONS.contains(&name)).then(|| format!("#let {name} = math.op({name:?})"))
}
