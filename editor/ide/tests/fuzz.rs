//! Random documents through every entry point an editor calls: nothing may panic (a panic ends the language
//! server), and every range must lie in the document. FUZZ_N documents (default 300), FUZZ_SEED.

use solveng_ide::*;

struct Rng(u64);
impl Rng {
    fn new(salt: u64) -> Rng {
        let seed: u64 = std::env::var("FUZZ_SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(7);
        Rng(seed.wrapping_mul(6364136223846793005).wrapping_add(salt.wrapping_mul(1442695040888963407)) | 1)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
    fn pick<'a, T>(&mut self, v: &'a [T]) -> &'a T {
        &v[self.below(v.len())]
    }
}

fn n() -> usize {
    std::env::var("FUZZ_N").ok().and_then(|s| s.parse().ok()).unwrap_or(300)
}

/// Pieces of documents solveng reads, from every corner of the language.
const PIECES: &[&str] = &[
    "#show: worksheet\n",
    "#import \"@preview/solveng:1.0.0\": *\n",
    "#import units: mA, kΩ, s as sec, V\n#import constants: pi, e\n#import operations: tri\n",
    "#let minute = unit($60 #s$, display: () => $\"min\"$)\n#let time = base-unit(\"time\")\n",
    "#let i = constant(\"imaginary\", $sqrt(-1)$)\n#let triangle = operation(\"tri\")\n",
    "#show: worksheet.with(preferred-units: ($#m / #s$,), evaluate: false)\n",
    "$U := 5 V$ ",
    "$I := U / (2 kΩ) = mA$ ",
    "$R_\"load\"(x: kΩ) := 2 x$ ",
    "$m_2() := 3$ $m() := 2$ $a := m() + m_2()$\n",
    "$x^2 - 4 = 0 -> solve(x)$\n",
    "$s_1(t) := tri((t-1)/2) + op(\"rect\")(t)$ $p := 2 pi #e^x$\n",
    "$f := sum_(k=0)^10 k^2$ ",
    "$g(t) := cases(0 & t < 0, 1 & \"otherwise\")$\n",
    "$A := mat(1, 2; 3, 4)$ $d := det(A) =$\n",
    "$J := number(U, #A)$ $q(x: mA) := x$ $t := 2 s + 3 #sec = minute$\n",
    "#worksheet($z := q + 1 =$, form: \"fraction\")\n",
    "#graph($P(..) = U^2/R$, mark: \"extrema\", xrange: (\"0 Ω\", \"10 Ω\"))\n",
    "#lambdify($U^2/R$, \"U\", unit: $#W$, units: (U: $V$))\n",
    "#worksheet(evaluate: false)[$E = m c^2$]\n",
    "#worksheet(scope: \"shared\")[$T := 2 #sec$ $f := 1/T = Hz$]\n",
    "#include \"part.typ\"\n",
    "Text with `code` and a $ stray dollar.\n```solveng\nx := 1\n```\n",
    "$Delta\"\"r := 1$ $Δr = $ $t_\"half\" := 3 h$ #clear(\"t_half\")\n",
];

/// Characters edits insert: what opens and closes things, and text outside the BMP.
const CHARS: &[char] = &[
    '$', '`', '"', '\\', '_', '(', ')', '[', ']', '{', '}', ':', '=', ';', '\n', ' ', 'Ω', 'µ', 'é', '😀', '#', '@', '/',
    '^', '-', '>', 'x', '1', ',', '&',
];

fn document(r: &mut Rng) -> String {
    let mut s: String = (0..1 + r.below(6)).map(|_| *r.pick(PIECES)).collect();
    // edits, as typing leaves documents half-written
    for _ in 0..r.below(6) {
        let chars: Vec<char> = s.chars().collect();
        let at = r.below(chars.len() + 1);
        let mut chars = chars;
        if r.below(3) == 0 && at < chars.len() {
            let end = (at + 1 + r.below(8)).min(chars.len());
            chars.drain(at..end);
        } else {
            chars.insert(at, *r.pick(CHARS));
        }
        s = chars.into_iter().collect();
    }
    s
}

fn len16(s: &str) -> u32 {
    s.encode_utf16().count() as u32
}

fn in_doc(len: u32, start: u32, end: u32, what: &str, doc: &str) {
    assert!(start <= end && end <= len, "{what} {start}..{end} outside 0..{len} in {doc:?}");
}

/// A report of the document as if compiled: its elements, and errors at some of them.
fn report(r: &mut Rng, doc: &str) -> Report {
    let mut errors = Vec::new();
    let mut elements = Vec::new();
    let mut at = 0;
    while let Some(i) = doc[at..].find('$') {
        let start = at + i;
        let Some(j) = doc[start + 1..].find('$') else { break };
        let end = start + 1 + j + 1;
        let text = &doc[start..end];
        let s16 = doc[..start].encode_utf16().count();
        let e16 = s16 + text.encode_utf16().count();
        elements.push(ReportElement { path: "main.typ".into(), start: s16, end: e16, text: text.into(), kind: "math".into(), active: true, evaluate: true });
        if r.below(2) == 0 {
            errors.push(ReportError { path: "main.typ".into(), start: s16, end: e16, text: text.into(), source: String::new(), message: "error".into() });
        }
        at = end;
    }
    errors.push(ReportError { source: "z := q + 1 =".into(), message: "unplaced".into(), ..ReportError::default() });
    let defined = vec![ReportValue { name: "o".into(), kind: "function".into(), value: "(x: Ω): Ω = x".into() }, ReportValue { name: "U".into(), kind: "given".into(), value: "5 V".into() }];
    Report { blocks: 1, elements, errors, defined }
}

fn check(doc: &str, context: &Context, offsets: &[u32]) {
    let len = len16(doc);
    let mut last = 0;
    for t in document_tokens(doc, context) {
        in_doc(len, t.start, t.start + t.length, "token", doc);
        assert!(t.start >= last, "tokens out of order in {doc:?}");
        last = t.start;
    }
    for p in diagnostics(doc, context) {
        in_doc(len, p.start, p.end, "problem", doc);
    }
    for c in conceals(doc, context) {
        in_doc(len, c.start, c.start + c.length, "conceal", doc);
    }
    for g in regions(doc, context) {
        in_doc(len, g.start, g.end, "region", doc);
    }
    let _ = math_tokens(doc, context);
    for &at in offsets {
        if let Some(c) = complete(doc, context, at) {
            in_doc(len, c.from, c.to.unwrap_or(at).max(c.from), "completion", doc);
            assert!(c.from <= at || c.to.is_some_and(|to| to >= at), "completion from {} after {at} in {doc:?}", c.from);
        }
        if let Some(h) = hover(doc, context, at) {
            in_doc(len, h.start, h.end, "hover", doc);
        }
        if let Some(l) = definition(doc, context, at).filter(|l| l.path.is_none()) {
            in_doc(len, l.start, l.end, "definition", doc);
        }
        for l in usages(doc, context, at).into_iter().filter(|l| l.path.is_none()) {
            in_doc(len, l.start, l.end, "usage", doc);
        }
        if let Some(t) = rename_target(doc, context, at) {
            in_doc(len, t.start, t.end, "rename target", doc);
            for e in rename(doc, context, at, "renamed").into_iter().filter(|e| e.path.is_none()) {
                in_doc(len, e.start, e.end, "rename", doc);
            }
        }
        for c in unit_fixes(doc, context, at) {
            in_doc(len, c.start, c.end, "fix", doc);
            for e in c.edits.iter().filter(|e| e.path.is_none()) {
                in_doc(len, e.start, e.end, "fix edit", doc);
            }
        }
    }
}

#[test]
fn random_documents_never_panic() {
    let mut r = Rng::new(1);
    let part = ContextFile { path: "part.typ".into(), text: "#import units: A\n$P := U I$\n$I := 2 #A$".into() };
    for _ in 0..n() {
        let doc = document(&mut r);
        let len = len16(&doc);
        // offsets anywhere, the middle of a surrogate pair included
        let offsets: Vec<u32> = (0..6).map(|_| r.below(len as usize + 1) as u32).chain([0, len]).collect();
        check(&doc, &Context::default(), &offsets);
        let report = report(&mut r, &doc);
        let context = Context::new("").with_files(vec![ContextFile { path: "main.typ".into(), text: doc.clone() }, part.clone()]).with_report(&report, "main.typ", true).with_main("main.typ");
        check(&doc, &context, &offsets);
    }
}

#[test]
fn snippets_never_panic() {
    // what editors call on pieces alone: an equation's body, the files a document references
    let mut r = Rng::new(2);
    for _ in 0..n() {
        let doc = document(&mut r);
        let len = len16(&doc);
        for t in math_tokens_of(&doc) {
            in_doc(len, t.start, t.start + t.length, "token", &doc);
        }
        let _ = references(&doc);
    }
}
