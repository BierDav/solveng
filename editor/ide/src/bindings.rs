//! Local Typst bindings take precedence over worksheet unit names in unquoted math.

use typst_syntax::{LinkedNode, SyntaxKind, ast::{AstNode, LetBinding}};
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
                        result.definitions.push(Binding { name: name.as_str().into(), start: doc.of_byte(leaf.offset()), end: doc.of_byte(leaf.offset() + leaf.len()), visible: doc.of_byte(node.offset() + node.len()), scope_end, operation: operation.clone(), unit_name: unit_name.clone(), value_range });
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

    fn binding(&self, at: usize, name: &str) -> Option<usize> {
        self.definitions.iter().enumerate().filter(|(_, b)| b.name == name && b.visible <= at && at < b.scope_end).max_by_key(|(_, b)| b.visible).map(|(id, _)| id)
    }

    pub fn operation(&self, at: usize) -> Option<&str> {
        self.uses.iter().find(|&&(s, e, _)| s <= at && at < e)
            .and_then(|&(_, _, id)| self.definitions[id].operation.as_deref())
    }

    pub fn inline_references(&self) -> &[InlineReference] { &self.inline }

    pub fn operator_strings(&self) -> &[(usize, usize, String)] { &self.operators }

    pub fn unit_uses(&self) -> impl Iterator<Item = (usize, usize, &str)> {
        self.uses.iter().filter_map(|&(start, end, id)| self.definitions[id].unit_name.as_deref().map(|name| (start, end, name)))
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

    pub fn operation_uses(&self) -> impl Iterator<Item = (usize, usize, &str)> {
        self.uses.iter().filter_map(|&(start, end, id)| self.definitions[id].operation.as_deref().map(|name| (start, end, name)))
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

pub(crate) fn problems(a: &crate::Analysis) -> Vec<crate::Problem> {
    let mut problems: Vec<_> = unresolved_units(a).into_iter().map(|(start, end, name)| crate::Problem {
        start: a.doc.to16(start), end: a.doc.to16(end), severity: "warning",
        message: format!("use '#{name}' to reference the Typst unit binding '{name}'; bare single letters are math symbols"),
    }).collect();
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
        title: "Quote \"as\" for a unit cast".into(), start: a.doc.to16(start), end: a.doc.to16(end), text: "\"as\"".into(),
    }).collect();
    for token in crate::document_tokens(document, context).iter().filter(|token| token.kind == crate::TokenKind::VARIABLE) {
        let start = a.doc.from16(token.start as usize);
        let end = a.doc.from16((token.start + token.length) as usize);
        if !(start <= at && at <= end) || a.doc.c.get(start) == Some(&'"') || start > 0 && a.doc.c[start - 1] == '#'
            || a.bindings.contains(start) { continue; }
        if !a.elements.iter().any(|element| element.math && a.active(element) && element.body_start <= start && end <= element.body_end) { continue; }
        let name = crate::resolved_name(&a.doc.sub(start, end));
        if crate::units::visible(&name, &a.units_at(start)) && (name.chars().count() == 1 || !crate::units::bare(&name, &a.units_at(start)))
            && crate::definition(document, context, token.start).is_none() {
            fixes.push(crate::Conversion { title: format!("Use \"{name}\" from the unit overlay"), start: token.start, end: token.start + token.length, text: format!("\"{name}\"") });
        }
    }
    fixes.extend(unresolved_units(&a).into_iter().filter(|&(start, end, _)| start <= at && at <= end).map(|(start, _, name)| crate::Conversion {
        title: format!("Use #{name} to reference the Typst unit binding"), start: a.doc.to16(start), end: a.doc.to16(start), text: "#".into(),
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
                                    start: a.doc.to16(position), end: a.doc.to16(position), text: format!("\n{text}") });
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
