//! Calls of solveng's functions, read with typst's parser: their string arguments are calculations, units or
//! names, and their named arguments take the package's options.

use typst_syntax::{LinkedNode, SyntaxKind, SyntaxNode};

use crate::params::{self, Function};
use crate::text::Doc;

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

#[derive(Clone, Debug)]
pub(crate) struct Call {
    pub function: &'static Function,
    /// Just inside the parentheses.
    pub args_start: usize,
    pub args_end: usize,
    pub texts: Vec<Text>,
    pub named: Vec<Named>,
    /// Equations passed as positional arguments, which are calculated wherever they are.
    pub maths: Vec<(usize, usize)>,
    /// Arguments as (name, source), for conversions.
    pub args: Vec<(Option<String>, String)>,
    /// Where each argument is, and whether it's a bare identifier, which may be a parameter's name being typed.
    pub items: Vec<(usize, usize, bool)>,
}

/// The function a callee names: `worksheet`, `solveng.worksheet`, `worksheet-config.with`.
pub(crate) fn callee(node: &SyntaxNode) -> Option<String> {
    match node.kind() {
        // in math too: unit("bar")
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
            "worksheet" | "graph" => Some(Role::Code),
            "unit" => Some(Role::Unit),
            "lambdify" => Some(if index == 0 { Role::Code } else { Role::Name }),
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
    calls: Vec<Call>,
}

impl Reader<'_> {
    fn chars(&self, node: &LinkedNode) -> (usize, usize) {
        (self.doc.of_byte(node.offset()), self.doc.of_byte(node.offset() + node.len()))
    }

    fn visit(&mut self, node: &LinkedNode) {
        if matches!(node.kind(), SyntaxKind::FuncCall | SyntaxKind::MathCall) {
            self.call(node);
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

    fn call(&mut self, node: &LinkedNode) {
        let Some(first) = node.children().next() else { return };
        let Some(function) = callee(first.get()).and_then(|n| params::function(&n)) else { return };
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
                SyntaxKind::LeftParen | SyntaxKind::RightParen | SyntaxKind::Comma | SyntaxKind::ContentBlock => {}
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
                    if arg.kind() == SyntaxKind::Equation && role(&function.name, None, index) == Some(Role::Code) {
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

/// Every call of a solveng function in a document, outer ones first.
pub(crate) fn calls(doc: &Doc) -> Vec<Call> {
    let root = typst_syntax::parse(&doc.s);
    let mut reader = Reader { doc, calls: Vec::new() };
    reader.visit(&LinkedNode::new(&root));
    reader.calls
}
