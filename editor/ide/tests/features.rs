// What the Rust crate adds: solveng's calls, options, the document's bindings and the compiler's values.

use solveng_ide::*;

fn u16len(s: &str) -> u32 {
    s.encode_utf16().count() as u32
}

fn slice(s: &str, start: u32, len: u32) -> String {
    let u: Vec<u16> = s.encode_utf16().collect();
    String::from_utf16(&u[start as usize..(start + len) as usize]).unwrap()
}

fn complete_at(doc: &str, context: &Context) -> Option<Completions> {
    complete(&doc.replacen('|', "", 1), context, u16len(&doc[..doc.find('|').unwrap()]))
}

fn labels(doc: &str) -> Vec<String> {
    // Variables are labelled with their unit (V: A); the name is what's compared.
    complete_at(doc, &Context::default()).map(|c| c.items.into_iter().map(|i| name(&i.label)).collect()).unwrap_or_default()
}

fn name(label: &str) -> String {
    label.split(": ").next().unwrap_or(label).to_string()
}

fn found(doc: &str) -> Vec<String> {
    document_tokens(doc, &Context::default()).iter().map(|t| format!("{}:{}", slice(doc, t.start, t.length), t.kind)).collect()
}

const SOLVENG: &str = "#import \"@preview/solveng:1.0.0\": *\n";

#[test]
fn reads_the_package_signatures() {
    let ws = params::functions().iter().find(|f| f.name == "worksheet").unwrap();
    let form = ws.named.iter().find(|p| p.name == "form").unwrap();
    assert_eq!(form.values, vec!["auto", "decimal", "fraction", "input"]);
    assert_eq!(ws.positional[0].name, "body");
    // The unit table and automatic lookups are gone.
    for removed in ["functions", "units", "builtin-units", "builtin-functions", "strict", "show-casts", "steps", "layout", "implies"] {
        assert!(!ws.named.iter().any(|p| p.name == removed), "{removed}");
    }
    let graph = params::functions().iter().find(|f| f.name == "graph").unwrap();
    // Options documented at worksheet.
    assert!(!graph.named.iter().find(|p| p.name == "digits").unwrap().doc.is_empty());
    assert_eq!(ws.named.iter().find(|p| p.name == "evaluate").unwrap().default, "true");
    let defined = params::functions().iter().find(|f| f.name == "defined").unwrap();
    assert_eq!(defined.named.iter().find(|p| p.name == "style").unwrap().values, vec!["grid", "column", "inline", "list"]);
    // Keywords and the binding constructors are documented.
    for f in ["unit", "base-unit", "constant", "operation", "digits"] {
        assert!(params::functions().iter().any(|p| p.name == f), "{f}");
    }
}

#[test]
fn highlights_the_strings_of_calls() {
    let doc = "#import units: kΩ, kPa\n#worksheet($R := 7.2 kΩ$, form: \"fraction\") #graph($P(..) = U^2/R$, mark: \"extrema\") #worksheet[$p := 3 kPa$]";
    let t = found(doc);
    for want in [
        format!("R:{}", TokenKind::VARIABLE),
        format!("7.2:{}", TokenKind::NUMBER),
        format!("kΩ:{}", TokenKind::UNIT),
        format!("fraction:{}", TokenKind::KEYWORD),
        format!("U:{}", TokenKind::PARAMETER),
        format!("extrema:{}", TokenKind::KEYWORD),
        format!("kPa:{}", TokenKind::UNIT),
    ] {
        assert!(t.contains(&want), "{want} in {t:?}");
    }
    // Math outside a worksheet isn't calculated.
    assert!(found("$p := 3 x$").is_empty());
    // Strings of other functions stay Typst's.
    assert!(found("#text(\"R = 3 V\")").is_empty());
    // A `$` in a string doesn't start math.
    assert!(found("#graph($x = 3 $$) $y$").iter().all(|t| !t.starts_with('y')));
}

#[test]
fn completes_in_the_strings_of_calls() {
    let names = labels("#worksheet($R := 7.2$)\n#worksheet($U := |$)");
    assert!(names.contains(&"R".into()) && names.contains(&"sqrt".into()));
    assert!(labels("#import units: mA\n#worksheet($I := 30 m|$)").contains(&"mA".into()));
    let c = complete_at("#worksheet($x =$, form: \"f|\")", &Context::default()).unwrap();
    assert!(c.exclusive);
    assert_eq!(labels("#worksheet($x =$, form: \"f|\")"), vec!["auto", "decimal", "fraction", "input"]);
    assert!(labels("#graph($y$, mark: \"|\")").contains(&"extrema".into()));
    assert!(labels("#import units: kΩ\n#worksheet[$p := 3 k|$]").contains(&"kΩ".into()));
    // Strings of `worksheet` aren't calculated.
    assert!(labels("#worksheet(\"I := 30 m|").is_empty());
}

#[test]
fn completes_options_of_calls() {
    let c = complete_at("#worksheet($x =$, fo|)", &Context::default()).unwrap();
    assert!(!c.exclusive);
    let form = c.items.iter().find(|i| i.label == "form").unwrap();
    assert_eq!(form.insert, "form: ");
    assert_eq!(form.kind, ItemKind::PARAMETER);
    assert!(form.documentation.as_deref().unwrap().contains("How values are written"));
    // Given ones aren't offered again.
    assert!(!labels("#worksheet($x =$, digits: 2, |)").contains(&"digits".into()));
    assert_eq!(labels("#worksheet($x =$, form: |)"), vec!["\"auto\"", "\"decimal\"", "\"fraction\"", "\"input\"", "auto"]);
    assert!(labels("#worksheet(|)").contains(&"evaluate".into()));
    assert!(!labels("#worksheet(|)").contains(&"units".into()));
    assert!(labels("#defined(style: \"|\")").contains(&"grid".into()));
}

#[test]
fn completes_every_value_an_option_takes() {
    // keywords as the types allow: auto, none, true, false
    assert_eq!(labels("#worksheet($x =$, approximate: |)"), vec!["auto", "true", "false"]);
    assert_eq!(labels("#worksheet($x =$, grouping: |)"), vec!["auto", "none"]);
    assert_eq!(labels("#graph($x^2$, joined: |)"), vec!["auto", "true", "false"]);
    assert_eq!(labels("#graph($x^2$, joined: (true, |)"), vec!["auto", "true", "false"]);
    assert_eq!(labels("#show: worksheet.with(graph-joined: |)"), vec!["auto", "true", "false"]);
    // in an option's array, per item
    let lines = labels("#graph($x^2$, lines: (\"dotted\", |)");
    assert!(lines.contains(&"\"dashed\"".into()) && lines.contains(&"none".into()) && lines.contains(&"auto".into()), "{lines:?}");
    assert!(labels("#graph($x^2$, xrange: (1, |)").is_empty());
    // graph's styles, as graph's own arguments and as the worksheet's defaults, alone, in arrays and in strings
    let dots = labels("#graph($x^2$, dots: |)");
    assert!(dots.contains(&"\"star\"".into()) && dots.contains(&"none".into()), "{dots:?}");
    assert!(labels("#graph($x^2$, dots: (\"circle\", |)").contains(&"\"triangle-down\"".into()));
    assert!(labels("#graph($x^2$, dots: (\"circle\", \"|\"))").contains(&"hexagon".into()));
    assert!(labels("#graph($x^2$, lines: \"|\")").contains(&"dash-dotted".into()));
    assert!(labels("#show: worksheet.with(graph-dots: |)").contains(&"\"plus\"".into()));
    assert!(labels("#show: worksheet.with(graph-lines: (\"|\"))").contains(&"loosely-dotted".into()));
    // marks and prefixes
    assert!(labels("#graph($x^2$, mark: |)").contains(&"\"extrema\"".into()));
    assert!(labels("#graph($x^2$, xmark: |)").contains(&"\"roots\"".into()));
    assert!(labels("#worksheet($x =$, prefixes: (\"|\"))").contains(&"k".into()));
    assert!(labels("#worksheet($x =$, prefixes: |)").contains(&"\"all\"".into()));
}

#[test]
fn knows_the_documents_unit_bindings() {
    let setup = "#let lux = base-unit(\"illuminance\")\n#let klux = unit($1000 lux$)\n#show: worksheet\n";
    let doc = format!("{setup}$E := 3 klux + 2 lux$");
    let t = found(&doc);
    assert!(t.contains(&format!("klux:{}", TokenKind::UNIT)) && t.contains(&format!("lux:{}", TokenKind::UNIT)), "{t:?}");
    let c = complete_at(&format!("{setup}$E := 3 |$"), &Context::default()).unwrap();
    let detail = |l: &str| c.items.iter().find(|i| i.label == l).unwrap().detail.clone();
    assert_eq!(detail("klux"), "unit = 1000 lux");
    assert_eq!(detail("lux"), "base unit · illuminance");
    // Bindings are lexical: not before they're made, not outside their block.
    let c = complete_at("#show: worksheet\n$E := 3 |$\n#let lux = base-unit(\"illuminance\")", &Context::default()).unwrap();
    assert!(!c.items.iter().any(|i| i.label == "lux"));
    let c = complete_at("#[#let lux = base-unit(\"illuminance\")]\n#show: worksheet\n$E := 3 |$", &Context::default()).unwrap();
    assert!(!c.items.iter().any(|i| i.label == "lux"));
}

#[test]
fn shows_the_compilers_values() {
    let report = Report::parse(
        r#"{"blocks":1,"defined":[{"name":"I","kind":"given","value":"30 mA"},{"name":"t","kind":"computed","value":"115 h"},{"name":"f","kind":"function","value":"(x) = x^2"},{"name":"k","kind":"given","value":"3"}]}"#,
    );
    let context = Context::new("").with_report(&report, "main.typ", true);
    let c = complete_at("#import units: mA, mAh\n#worksheet($I := 30 mA$, $t := 3450 mAh / I$)\n#worksheet($x := |$)", &context).unwrap();
    let get = |l: &str| c.items.iter().find(|i| name(&i.label) == l).unwrap().detail.clone();
    assert_eq!(get("I"), "= 30 mA");
    assert_eq!(get("t"), "= 3450 mAh / I = 115 h");
    // Defined where the text doesn't say.
    assert_eq!(get("k"), "= 3");
    assert_eq!(get("f(x)"), "= x^2");
}

fn fix(doc: &str, needle: &str) -> Option<String> {
    let at = u16len(&doc[..doc.find(needle).unwrap()]);
    let fix = unit_fixes(doc, &Context::default(), at).into_iter().find(|f| f.title.starts_with("Import"))?;
    let mut edits: Vec<(u32, u32, String)> = vec![(fix.start, fix.end, fix.text.clone())];
    edits.extend(fix.edits.iter().map(|e| (e.start, e.end, e.text.clone())));
    edits.sort_by_key(|e| std::cmp::Reverse(e.0));
    let mut text: Vec<u16> = doc.encode_utf16().collect();
    for (s, e, t) in edits { text.splice(s as usize..e as usize, t.encode_utf16()); }
    Some(format!("{}\n{}", fix.title, String::from_utf16(&text).unwrap()))
}

#[test]
fn imports_the_modules_names_math_uses_unimported() {
    // Typst can't find `km` without its import: the fix adds it after solveng's import, or to the module's.
    let doc = format!("{SOLVENG}#show: worksheet\n$x := 2 km$");
    assert_eq!(fix(&doc, "km$").unwrap(), format!("Import km from solveng's units\n{SOLVENG}#import units: km\n#show: worksheet\n$x := 2 km$"));
    let doc = format!("{SOLVENG}#import units: m\n#show: worksheet\n$x := 2 km$");
    assert!(fix(&doc, "km$").unwrap().contains("#import units: m, km\n"));
    // constants and operations too
    let doc = format!("{SOLVENG}#show: worksheet\n$x := 2 pi + tri(1)$");
    assert!(fix(&doc, "pi +").unwrap().contains("#import constants: pi\n"));
    assert!(fix(&doc, "tri(").unwrap().contains("#import functions: tri\n"));
    // Not for names no module has, nor outside calculated math, nor without solveng's import.
    assert!(fix(&format!("{SOLVENG}#show: worksheet\n$x := 2 furlong$"), "furlong").is_none());
    assert!(fix(&format!("{SOLVENG}$x := 2 km$"), "km$").is_none());
    assert!(fix("#show: worksheet\n$x := 2 km$", "km$").is_none());
    // Keywords of a clause are no names to import, and nothing is wrong there.
    assert!(diagnostics("#import units: h\n#show: worksheet\n$x^2 = 4 -> solve(x)$\n$t = #h, digits(2), sci$", &Context::default()).is_empty());
}

#[test]
fn hovers_names_units_and_options() {
    let doc = "#import units: kΩ\n#worksheet($R := 7.2 kΩ$)\n#worksheet($U := R I$, form: \"fraction\")";
    let at = |w: &str| u16len(&doc[..doc.rfind(w).unwrap()]) + 1;
    assert_eq!(hover(doc, &Context::default(), at("R I")).unwrap().markdown, "```text\nR: kΩ = 7.2 kΩ\n```");
    assert!(hover(doc, &Context::default(), at("form")).unwrap().markdown.starts_with("`form` (auto, str)"));
    let unit = hover(doc, &Context::default(), at("kΩ$")).unwrap().markdown;
    assert!(unit.contains("`kΩ` kiloohm · built-in unit\n\n`kΩ = 10³ Ω` · `Ω = V/A = kg·m²·s⁻³·A⁻²`"), "{unit}");
    assert!(hover("text R here", &Context::default(), 6).is_none());
    // the modifiers of a clause
    let doc = "#import units: h, min\n#import modifiers: digits\n#show: worksheet\n$t := 3 #h$ $t = min, digits(2)$";
    let md = hover(doc, &Context::default(), u16len(&doc[..doc.rfind("digits").unwrap()]) + 1).unwrap().markdown;
    assert!(md.contains("`digits` (solveng)") && md.contains("modifiers"), "{md}");
}

#[test]
fn lists_regions() {
    // Raw text is never solveng's, a ```solveng block included.
    let doc = "#show: worksheet\n$x := 1$ `y` ```solveng\nz := 2\n``` #worksheet($z$)";
    let kinds: Vec<_> = regions(doc, &Context::default()).iter().map(|r| r.kind).collect();
    assert_eq!(kinds, vec!["math", "call", "math"]);
}

#[test]
fn calculates_math_given_to_a_worksheet() {
    // Math passed to worksheet is calculated without a worksheet around it.
    let doc = "#worksheet($I := 30 mA$)";
    assert!(!math_tokens(doc, &Context::default()).is_empty());
}

#[test]
fn resolves_spelled_greek_names() {
    assert_eq!(resolved_name("Delta_R"), "Δ_R");
    assert_eq!(resolved_name("alpha2"), "α2");
    assert_eq!(resolved_name("Delta"), "Δ");
    assert_eq!(resolved_name("DeltaT"), "DeltaT");
    assert_eq!(resolved_name("R_1"), "R_1");
    // Completions insert the stored name and match its spelling.
    let doc = "#import units: cm\n#show: worksheet\n$Delta_R := 5 cm$\n$x := Del|$";
    let items = complete_at(doc, &Context::default()).unwrap().items;
    let item = items.iter().find(|i| i.label == "Δ_R: cm").unwrap();
    assert_eq!(item.insert, "Δ_R");
    assert_eq!(item.filter.as_deref(), Some("Del"));
    // The spelled name is the letter.
    let doc = "#import units: cm\n#show: worksheet\n$Delta_R := 5 cm$\n$x := Delta_R$\n$y := Δ_R$";
    let at = u16len(&doc[..doc.find("Δ_R$").unwrap()]);
    assert_eq!(usages(doc, &Context::default(), at).len(), 3);
}

#[test]
fn value_reads_its_equation_as_a_calculation() {
    let doc = "#import units: V, mV\n#show: worksheet\n$U_2 := 5 #V$\nIt is #value($U_2 / 2 = mV$).";
    let at = u16len(&doc[..doc.rfind("U_2").unwrap()]);
    assert_eq!(definition(doc, &Context::default(), at).unwrap().start, u16len(&doc[..doc.find("U_2").unwrap()]));
    assert_eq!(rename(doc, &Context::default(), at, "U_3").len(), 2);
    let tokens = found(doc);
    assert!(tokens.contains(&format!("mV:{}", TokenKind::UNIT)), "{tokens:?}");
    assert!(labels("#show: worksheet\n$U_2 := 5$\n#value($U|$)").contains(&"U_2".to_string()));
}

#[test]
fn substitute_takes_names_of_the_worksheet() {
    // highlighted as names, also where a variable is spelled like a unit
    let doc = "#import units: m\n#import modifiers: substitute\n#show: worksheet\n$V := 2 #m^3$\n$Q_0 := V a$\n$w := Q_0 + 1 -> substitute(Q_0, V)$";
    let tokens = found(doc);
    assert!(tokens.contains(&"substitute:0".to_string()), "{tokens:?}");
    assert_eq!(tokens.iter().filter(|t| *t == "V:6").count(), 3, "{tokens:?}");
    // its names are uses: definition, usages and rename reach them
    let at = u16len(&doc[..doc.rfind("Q_0").unwrap()]);
    assert_eq!(definition(doc, &Context::default(), at).unwrap().start, u16len(&doc[..doc.find("Q_0").unwrap()]));
    assert_eq!(usages(doc, &Context::default(), at).len(), 3);
    let edits = rename(doc, &Context::default(), at, "Q_1");
    assert_eq!(edits.len(), 3, "{edits:?}");
    let doc = "#show: worksheet\n$k := a b$\n$z := k c -> substitute(k)$\n";
    let at = u16len(&doc[..doc.rfind('k').unwrap()]);
    assert_eq!(usages(doc, &Context::default(), at).len(), 3);
    assert_eq!(rename(doc, &Context::default(), at, "q").len(), 3);
    // it completes as a modifier where it's imported, and the defined names inside its parentheses
    let doc = "#import modifiers: substitute\n#show: worksheet\n$k := a b$\n$z := k c -> subst|$";
    assert!(labels(doc).contains(&"substitute".to_string()), "{:?}", labels(doc));
    let doc = "#show: worksheet\n$k := a b$\n$z := k c -> substitute(|$";
    assert!(labels(doc).contains(&"k".to_string()), "{:?}", labels(doc));
}

#[test]
fn glues_names_with_empty_strings() {
    assert_eq!(resolved_name("Delta\"\"r_R"), "Δr_R");
    assert_eq!(resolved_name("Delta\"\"Gamma\"\"Omega"), "ΔΓΩ");
    // Both spellings mean the same variable; completions insert it so math can read it.
    let doc = "#import units: cm\n#show: worksheet\n$Delta\"\"r_R := 5 cm$\n$x := Delta\"\"r_R + Δr_R$\n$y := Del|$";
    let items = complete_at(doc, &Context::default()).unwrap().items;
    let item = items.iter().find(|i| i.label == "Δr_R: cm").unwrap();
    assert_eq!(item.insert, "\"Δr\"_R");
    assert_eq!(item.filter.as_deref(), Some("Del"));
    let doc = doc.replace('|', "");
    assert!(found(&doc).contains(&"Delta\"\"r:6".to_string()), "{:?}", found(&doc));
    let at = u16len(&doc[..doc.find("Δr_R").unwrap()]);
    assert_eq!(usages(&doc, &Context::default(), at).len(), 3);
}

#[test]
fn hovers_base_units_and_derived_ones() {
    let doc = "#let lux = base-unit(\"illuminance\")\n#let klux = unit($1000 lux$)\n#let dozen = unit($12$)\n#show: worksheet\n$x := 3 lux$\n$n := 2 dozen$\n$y := 5 klux$\n";
    let at = |n: &str| u16len(&doc[..doc.find(n).unwrap()]);
    let md = |n: &str| hover(doc, &Context::default(), at(n)).unwrap().markdown;
    assert!(md("lux$").starts_with("`lux`: base unit of the dimension `illuminance`"), "{}", md("lux$"));
    assert_eq!(md("dozen$"), "`dozen`: unit: `12`");
    assert_eq!(md("klux$"), "`klux`: unit: `1000 lux`");
    // the binding itself is Typst's
    assert!(hover(doc, &Context::default(), at("klux =")).is_none());
    assert!(found(doc).contains(&"lux:9".to_string()) && found(doc).contains(&"dozen:9".to_string()));
    assert!(diagnostics(doc, &Context::default()).is_empty());
}

#[test]
fn completes_units_after_a_colon() {
    let ctx = Context::default();
    let setup = format!("{SOLVENG}#import units: m\n#let lux = base-unit(\"illuminance\")\n");
    let only_units = |doc: &str| {
        let c = complete_at(&format!("{setup}{doc}"), &ctx).unwrap();
        // and the types that aren't units, any and 1
        assert!(c.exclusive && c.items.iter().all(|i| i.kind == ItemKind::UNIT || i.kind == ItemKind::KEYWORD), "{doc}");
        c.items.into_iter().filter(|i| i.kind == ItemKind::UNIT).collect::<Vec<_>>()
    };
    // A parameter's unit.
    only_units("#show: worksheet\n$f(x: m|) := x$");
    only_units("#show: worksheet\n$f(x, y: |) := x$");
    only_units("#worksheet($f(I: |) := I$)");
    let math = only_units("#show: worksheet\n$ q(x: |) := x $");
    // Bindings of the document and its imports, then the module's units imported along; Ω is found typing its name.
    assert_eq!(math.iter().find(|i| i.label == "m").unwrap().insert, "#m");
    assert!(math.iter().any(|i| i.label == "lux" && i.insert == "lux" && i.edits.is_empty()));
    let k = math.iter().find(|i| i.label == "kΩ").unwrap();
    assert_eq!((k.insert.as_str(), k.filter.as_deref()), ("kΩ", Some("kΩ kOmega kiloohm")));
    assert!(k.edits.iter().any(|e| e.text == ", kΩ"));
    let a = math.iter().find(|i| i.label == "A").unwrap();
    assert_eq!(a.insert, "#A");
    // Variables and what a function returns take no type.
    let not_units = |doc: &str| complete_at(&format!("{setup}{doc}"), &ctx).is_none_or(|c| !c.exclusive || c.items.iter().any(|i| i.kind != ItemKind::UNIT && i.kind != ItemKind::KEYWORD));
    assert!(not_units("#show: worksheet\n$k := 2$\n$V: |$"));
    assert!(not_units("#show: worksheet\n$f(x: #A): |$"));
    // Not in an equation's value.
    assert!(not_units("#show: worksheet\n$V := |$"));
}

#[test]
fn completes_keywords() {
    let insert = |doc: &str, label: &str| complete_at(doc, &Context::default()).and_then(|c| c.items.into_iter().find(|i| i.label == label && i.kind == ItemKind::KEYWORD)).map(|i| i.insert);
    // any and 1 where a type goes, quoted as math needs them
    assert_eq!(insert("#worksheet[$ f(x: |) := x $]", "any").as_deref(), Some("any"));
    assert_eq!(insert("#worksheet($f(x: |) := x$)", "1").as_deref(), Some("1"));
    // the modifiers of a clause: those in scope, the module's others imported along
    assert_eq!(insert("#import modifiers: solve\n#worksheet[$ x^2 = 4 -> s| $]", "solve").as_deref(), Some("solve($0)"));
    assert_eq!(insert("#import units: h, min\n#import modifiers: sci\n#worksheet[$ t := 2 #h = min, | $]", "sci").as_deref(), Some("sci"));
    let doc = "#import \"@preview/solveng:1.0.0\": *\n#worksheet[$ x^2 = 4 -> s| $]";
    let solve = complete_at(doc, &Context::default()).unwrap().items.into_iter().find(|i| i.label == "solve").unwrap();
    assert_eq!((solve.kind, solve.insert.as_str()), (ItemKind::KEYWORD, "solve($0)"));
    assert_eq!(solve.edits.iter().map(|e| e.text.as_str()).collect::<Vec<_>>(), ["\n#import modifiers: solve"]);
    // renamed, under the name it's bound to
    let insert_renamed = insert("#import modifiers: solve as löse\n#worksheet[$ x^2 = 4 -> l| $]", "löse");
    assert_eq!(insert_renamed.as_deref(), Some("löse($0)"));
    // without solveng's import there's nothing to import them along with
    assert!(insert("#worksheet[$ x^2 = 4 -> s| $]", "solve").is_none());
}

#[test]
fn hovers_keywords() {
    let md = |doc: &str, word: &str| {
        let doc = format!("#import units: A\n#show: worksheet\n{doc}");
        let at = u16len(&doc[..doc.find(word).unwrap()]) + 1;
        hover(&doc, &Context::default(), at).map(|h| h.markdown).unwrap_or_default()
    };
    assert!(md("$f(x: any) := x$", "any").starts_with("`any` type"));
    let doc = "#import modifiers: solve\n$ x^2 = 4 -> solve(x) $";
    assert!(md(doc, "solve(x)").contains("`solve` (solveng)"), "{}", md(doc, "solve(x)"));
}

#[test]
fn hovers_parameters_with_their_function() {
    let md = |doc: &str, word: &str, context: &Context| {
        let doc = format!("#import units: V, mA\n#show: worksheet\n{doc}");
        let at = u16len(&doc[..doc.rfind(word).unwrap()]);
        hover(&doc, context, at).map(|h| h.markdown).unwrap_or_default()
    };
    let doc = "$ U(x: #V) := f(x mA, y) $";
    assert_eq!(md(doc, "x mA", &Context::default()), "`x: V` parameter of `U`");
    assert_eq!(md("$g(t) := 2 t$", "t$", &Context::default()), "`t` parameter of `g`");
    // the compiler's types, inferred ones too
    let mut context = Context::default();
    context.defined.push(ReportValue { name: "g".into(), kind: "function".into(), value: "(t: s): s = 2 t".into() });
    assert_eq!(md("$g(t) := 2 t$", "t$", &context), "`t: s` parameter of `g`");
    assert!(md("#graph($q(w) = w^2$)", "w^2", &Context::default()).contains("parameter of `q`"));
}

#[test]
fn units_show_their_heritage() {
    let doc = "#import units: mA, kg, kPa\n#show: worksheet\n$q := 2 mA$\n$r := 4 kg$\n$p := 3 kPa$\n";
    let md = |w: &str| hover(doc, &Context::default(), u16len(&doc[..doc.find(w).unwrap()]) + 1).unwrap().markdown;
    assert!(md("mA$").contains("`mA` milliampere · built-in unit\n\n`mA = 10⁻³ A`"), "{}", md("mA$"));
    assert!(md("kg$").contains("`kg` kilogram · built-in SI base unit"), "{}", md("kg$"));
    assert!(md("kPa$").contains("`kPa = 10³ Pa` · `Pa = N/m² = kg·m⁻¹·s⁻²`"), "{}", md("kPa$"));
}

#[test]
fn variables_and_units_spelled_alike_stay_apart() {
    // A bare letter is a variable, `#V` the unit binding.
    let doc = "#import units: V\n#show: worksheet\n$V := 4$\n$W := V + 1 #V$\n";
    let at = |before: &str| u16len(&doc[..doc.find(before).unwrap() + before.len()]);
    let variable = hover(doc, &Context::default(), at("W := ")).unwrap().markdown;
    assert!(variable.starts_with("```text\nV"), "{variable}");
    let unit = hover(doc, &Context::default(), at("1 #")).unwrap().markdown;
    assert!(unit.contains("`V` volt"), "{unit}");
    // the variable defined, the bare letter isn't a missing `#`
    assert!(diagnostics(doc, &Context::default()).is_empty());
}

#[test]
fn completions_tell_module_and_document_units() {
    let doc = format!("{SOLVENG}#import units: kPa\n#let bar = unit($100 kPa$)\n#show: worksheet\n$p := 3 |$");
    let items = complete_at(&doc, &Context::default()).unwrap().items;
    let detail = |l: &str| items.iter().find(|i| i.label == l).map(|i| i.detail.clone()).unwrap_or_default();
    assert_eq!(detail("bar"), "unit = 100 kPa");
    assert_eq!(detail("kPa"), "kilopascal · solveng units");
    assert_eq!(detail("kΩ"), "kiloohm · solveng units · imports units: kΩ");
}

#[test]
fn warns_about_definitions_made_again() {
    let doc = "#show: worksheet\n$ I := 4 $\n$f(x) := x$\n$ I := 2 $\n$f(x) := 2 x$\n$R := 3$\n";
    let found: Vec<String> = diagnostics(doc, &Context::default()).into_iter().map(|p| format!("{}|{}", slice(doc, p.start, p.end - p.start), p.message)).collect();
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].starts_with("I|the variable 'I' is assigned again on line 4"), "{found:?}");
    assert!(found[1].starts_with("f|the function 'f' is assigned again on line 5"), "{found:?}");
}

#[test]
fn chains_define_with_their_last_step() {
    let doc = "#import units: A\n#let lux = base-unit(\"illuminance\")\n#show: worksheet\n$V := 4 lux = 2 #A$\n$W := V$\n";
    let h = hover(doc, &Context::default(), u16len(&doc[..doc.find("W := ").unwrap() + 5])).unwrap().markdown;
    assert_eq!(h, "```text\nV: A = 2 A\n```");
}

#[test]
fn completes_module_units_with_their_import() {
    let item = |doc: &str, label: &str| complete_at(doc, &Context::default()).and_then(|c| c.items.into_iter().find(|i| i.label == label));
    let all = format!("{SOLVENG}#show: worksheet\n");
    // imported along: after solveng's import, or into the module's import in scope
    let ma = item(&format!("{all}$I := 30 |$"), "mA").unwrap();
    assert_eq!(ma.insert, "mA");
    assert_eq!(ma.edits.iter().map(|e| e.text.as_str()).collect::<Vec<_>>(), ["\n#import units: mA"]);
    let ma = item(&format!("{SOLVENG}#import units: s\n#show: worksheet\n$I := 30 |$"), "mA").unwrap();
    assert_eq!(ma.edits.iter().map(|e| e.text.as_str()).collect::<Vec<_>>(), [", mA"]);
    // a package import listing names gets the module too
    let some = "#import \"@preview/solveng:1.0.0\": worksheet\n#show: worksheet\n$I := 30 |$";
    let texts: Vec<String> = item(some, "mA").unwrap().edits.into_iter().map(|e| e.text).collect();
    assert_eq!(texts, [", units", "\n#import units: mA"]);
    // single letters with a #
    assert_eq!(item(&format!("{all}$I := 30 |$"), "A").unwrap().insert, "#A");
    // without solveng's import nothing to add them to: only the bindings in scope
    assert!(item("#show: worksheet\n$I := 30 |$", "mA").is_none());
    assert!(item("#import units: mA\n#show: worksheet\n$I := 30 |$", "mA").is_some_and(|i| i.edits.is_empty()));
}

#[test]
fn assignments_statements_and_results() {
    let doc = "#import units: mA, mW, kΩ, V\n#import modifiers: solve\n#show: worksheet\n$I := 30 mA$\n$I = 0.03 mA$\n$R := 2 kΩ$\n$R := 3 kΩ$\n$U(x: #V) := 2 x$\n$y := I R = mW$\n$x^2 = 4 -> solve(x)$\n";
    // := highlights as one operator; the = and -> showing a result bold
    let toks = found(doc);
    assert_eq!(toks.iter().filter(|t| t.starts_with(":=:")).count(), 5, "{toks:?}");
    assert!(toks.contains(&format!("=:{}", TokenKind::EVALUATE)) && toks.contains(&format!("->:{}", TokenKind::EVALUATE)), "{toks:?}");
    assert!(toks.contains(&format!("mW:{}", TokenKind::UNIT)) && toks.contains(&format!("solve:{}", TokenKind::KEYWORD)), "{toks:?}");
    // = states: no definition, no warning
    let warn: Vec<String> = diagnostics(doc, &Context::default()).into_iter().map(|p| p.message).collect();
    assert_eq!(warn.len(), 1, "{warn:?}");
    assert!(warn[0].starts_with("the variable 'R' is assigned again"), "{warn:?}");
    let at = u16len(&doc[..doc.find("I R").unwrap()]);
    assert_eq!(hover(doc, &Context::default(), at).unwrap().markdown, "```text\nI: mA = 30 mA\n```");
    // the name solved for is defined there
    let at = u16len(&doc[..doc.rfind("x)").unwrap()]);
    assert_eq!(definition(doc, &Context::default(), at).map(|l| l.start), Some(at));
    let names = labels(&format!("{doc}$z := |$"));
    assert!(names.contains(&"x".into()) && names.contains(&"y".into()));
}

/// Whether an editor filtering by substring, as Typewright and VS Code do, keeps a unit for what's typed.
fn offers_unit(doc: &str, typed: &str, label: &str) -> bool {
    let c = complete_at(doc, &Context::default()).unwrap();
    c.items.iter().any(|i| i.kind == ItemKind::UNIT && i.label == label && i.filter.as_deref().unwrap_or(&i.label).to_lowercase().contains(&typed.to_lowercase()))
}

#[test]
fn finds_units_by_their_names() {
    // typing a unit's name finds it, also where its symbol shares no letter with it: ohm is Ω
    for (doc, typed, label) in [
        ("$R := 5 ohm|$", "ohm", "Ω"),
        ("$R := 5 o|$", "o", "Ω"),
        ("$R := 5 kiloohm|$", "kiloohm", "kΩ"),
        ("$R := 5 Omega|$", "Omega", "Ω"),
        ("$q(x: ohm|) := x$", "ohm", "Ω"),
        ("$U := 5 volt|$", "volt", "V"),
        ("$P := 2 watt|$", "watt", "W"),
        ("$T := 20 Celsius|$", "Celsius", "degC"),
    ] {
        let doc = format!("{SOLVENG}#show: worksheet\n{doc}");
        assert!(offers_unit(&doc, typed, label), "{label} for {typed} in {doc}");
    }
}

#[test]
fn colours_called_subscripted_names_as_functions() {
    // m() and m_2() are both functions, defined and called, whatever their subscript
    let kinds = |doc: &str, name: &str| -> Vec<u8> {
        document_tokens(doc, &Context::default()).iter().filter(|t| slice(doc, t.start, t.length) == name).map(|t| t.kind).collect()
    };
    let doc = "#show: worksheet\n$m() := 2$\n$m_2() := 3$\n$R_\"load\"(x) := x$\n$v_(a b)(t) := t$\n$a := m() + m_2() + R_\"load\"(2) + v_(a b)(1)$";
    assert_eq!(kinds(doc, "m"), vec![TokenKind::FUNCTION, TokenKind::FUNCTION_CALL]);
    for name in ["m_2", "R_\"load\"", "v_(a b)"] {
        assert_eq!(kinds(doc, name), vec![TokenKind::FUNCTION, TokenKind::FUNCTION_CALL], "{name} in {:?}", found(doc));
    }
    // a subscripted variable, a sum and an integral stay as they were
    let doc = "#show: worksheet\n$x_1 := 2$\n$y := x_1 (3)$\n$s := sum_(k=0)^10 k$\n$i := integral_0^1 x dif x$";
    assert!(kinds(doc, "x_1").is_empty() && kinds(doc, "x").contains(&TokenKind::VARIABLE));
    assert_eq!(kinds(doc, "sum"), vec![TokenKind::OPERATOR]);
}

#[test]
fn reads_no_exponent_notation() {
    // 1.12e4 is the coefficient 1.12 and the name e4; 2.5e-10 is 2.5 · e − 10, e a variable unless bound
    let doc = "#show: worksheet\n$1.12e4$";
    assert_eq!(found(doc), ["1.12:3", "e4:6"]);
    let doc = "#show: worksheet\n$2.5e-10$";
    assert_eq!(found(doc), ["2.5:3", "e:6", "10:3"]);
    // the constant e is a binding, with its documentation
    let doc = "#import constants: e\n#value($2.5 #e^(-10)$)";
    let at = u16len(&doc[..doc.rfind('e').unwrap()]);
    assert!(found(doc).contains(&format!("e:{}", TokenKind::CONSTANT)), "{:?}", found(doc));
    assert!(hover(doc, &Context::default(), at).unwrap().markdown.contains("euler"));
}

#[test]
fn modifiers_are_bindings_of_their_module() {
    // renamed, a modifier is still one: highlighted, solving defines, hover reaches its docs
    let doc = format!("{SOLVENG}#import modifiers: solve as löse, digits\n#show: worksheet\n$2 w = 6 -> löse(w)$\n$v := w + 1$\n$w = digits(2)$");
    let toks = found(&doc);
    for want in ["löse:0", "digits:0", "->:11", "=:11"] { assert!(toks.contains(&want.to_string()), "{want}: {toks:?}"); }
    let at = u16len(&doc[..doc.rfind("w + 1").unwrap()]);
    assert_eq!(definition(&doc, &Context::default(), at).unwrap().start, u16len(&doc[..doc.rfind("(w)").unwrap()]) + 1);
    let at = u16len(&doc[..doc.rfind("löse(").unwrap()]) + 1;
    assert!(hover(&doc, &Context::default(), at).unwrap().markdown.contains("`solve` (solveng)"));
    // a binding of another kind under a modifier's name isn't one
    let doc = format!("{SOLVENG}#let solve = 3\n#show: worksheet\n$x^2 = 4 -> solve(x)$");
    assert!(!found(&doc).contains(&"solve:0".to_string()), "{:?}", found(&doc));
    // unimported, it's written as one but Typst can't read it: not highlighted, the import offered
    let doc = format!("{SOLVENG}#show: worksheet\n$x^2 = 4 -> solve(x)$");
    let toks = found(&doc);
    assert!(toks.contains(&"->:11".to_string()) && !toks.contains(&"solve:0".to_string()), "{toks:?}");
    let at = u16len(&doc[..doc.find("solve(").unwrap()]) + 1;
    let fix = unit_fixes(&doc, &Context::default(), at).into_iter().find(|f| f.title == "Import solve from solveng's modifiers").unwrap();
    assert_eq!(fix.edits.iter().map(|e| e.text.as_str()).collect::<Vec<_>>(), ["\n#import modifiers: solve"]);
    // inside a renamed solve(…), the names of the worksheet complete
    let doc = format!("{SOLVENG}#import modifiers: solve as löse\n#show: worksheet\n$k := a b$\n$k = 2 c -> löse(|$");
    assert!(labels(&doc).contains(&"c".to_string()), "{:?}", labels(&doc));
}

#[test]
fn comments_in_math_are_comments() {
    let tokens = |doc: &str| -> Vec<(String, u8)> {
        let u: Vec<u16> = doc.encode_utf16().collect();
        document_tokens(doc, &Context::default()).iter().map(|t| (String::from_utf16_lossy(&u[t.start as usize..(t.start + t.length) as usize]), t.kind)).collect()
    };
    let c = |s: &str| (s.to_string(), TokenKind::COMMENT);
    assert_eq!(tokens("#show: worksheet\n$ /* awef */$\n"), vec![c("/* awef */")]);
    assert_eq!(tokens("#show: worksheet\n$\n// aew\n$\n"), vec![c("// aew")]);
    // a comment's `$` doesn't end the equation; what follows a comment is read as before
    let ts = tokens("#show: worksheet\n$y := 3 /* a $ b /* nested */ */ + x$\n");
    assert!(ts.contains(&c("/* a $ b /* nested */ */")) && ts.contains(&("x".into(), TokenKind::VARIABLE)), "{ts:?}");
    // a line comment ends at the line break; the result marker before a comment still shows a result
    let ts = tokens("#show: worksheet\n$x := 2 // x = 3\n$\n$x = /* c */$\n");
    assert!(ts.contains(&c("// x = 3")) && ts.contains(&("=".into(), TokenKind::EVALUATE)), "{ts:?}");
    // quoted text isn't a comment
    assert!(tokens("#show: worksheet\n$\"a//b\" := 1$\n").iter().all(|t| t.1 != TokenKind::COMMENT));
}
