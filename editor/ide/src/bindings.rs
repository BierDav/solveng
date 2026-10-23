//! Local Typst bindings take precedence over worksheet unit names in unquoted math.

use typst_syntax::{LinkedNode, SyntaxKind, ast::{AstNode, LetBinding}};
use crate::text::Doc;

struct Binding {
    name: String,
    start: usize,
    end: usize,
    visible: usize,
    scope_end: usize,
    unit: bool,
}

#[derive(Default)]
pub(crate) struct Bindings {
    definitions: Vec<Binding>,
    uses: Vec<(usize, usize, usize)>,
    bare: Vec<(usize, usize, usize)>,
    casts: Vec<(usize, usize)>,
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
            let mut parent = node.parent().cloned();
            let mut scope_end = doc.c.len();
            while let Some(scope) = parent {
                if matches!(scope.kind(), SyntaxKind::ContentBlock | SyntaxKind::CodeBlock | SyntaxKind::Closure) {
                    scope_end = doc.of_byte(scope.offset() + scope.len());
                    break;
                }
                parent = scope.parent().cloned();
            }
            let unit = binding.init().is_some_and(|value| {
                let value = value.to_untyped();
                value.kind() == SyntaxKind::FuncCall && value.children().next().and_then(crate::calls::callee).as_deref() == Some("unit")
            });
            for name in binding.kind().bindings() {
                visit(node, &mut |leaf| {
                    if leaf.get().span() == name.to_untyped().span() {
                        result.definitions.push(Binding { name: name.as_str().into(), start: doc.of_byte(leaf.offset()), end: doc.of_byte(leaf.offset() + leaf.len()), visible: doc.of_byte(node.offset() + node.len()), scope_end, unit });
                    }
                });
            }
        });
        visit(&root, &mut |node| {
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
                    && result.definitions[id].unit {
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

    pub fn contains(&self, start: usize) -> bool { self.uses.iter().any(|&(s, _, _)| s == start) }

    pub fn unit_hashes(&self, doc: &Doc) -> Vec<usize> {
        self.uses.iter().filter(|&&(start, _, id)| start > 0 && doc.c[start - 1] == '#' && self.definitions[id].unit).map(|&(start, _, _)| start - 1).collect()
    }

    pub fn bare_units(&self) -> impl Iterator<Item = (usize, usize, &str)> {
        self.bare.iter().map(|&(start, end, id)| (start, end, self.definitions[id].name.as_str()))
    }

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
    fixes
}
