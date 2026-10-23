// What the Rust crate adds: solveng's calls, options, the document's units and the compiler's values.

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

#[test]
fn reads_the_package_signatures() {
    let ws = params::functions().iter().find(|f| f.name == "worksheet").unwrap();
    let steps = ws.named.iter().find(|p| p.name == "steps").unwrap();
    assert_eq!(steps.values, vec!["full", "formula", "result", "value", "auto"].into_iter().filter(|v| *v != "auto").collect::<Vec<_>>());
    assert_eq!(ws.positional[0].name, "body");
    assert!(!ws.named.iter().any(|p| p.name == "functions"));
    let graph = params::functions().iter().find(|f| f.name == "graph").unwrap();
    // Options documented at worksheet.
    assert!(!graph.named.iter().find(|p| p.name == "digits").unwrap().doc.is_empty());
    assert!(ws.named.iter().find(|p| p.name == "units").unwrap().doc.contains("user overlay"));
    assert_eq!(ws.named.iter().find(|p| p.name == "evaluate").unwrap().default, "true");
    let defined = params::functions().iter().find(|f| f.name == "defined").unwrap();
    assert_eq!(defined.named.iter().find(|p| p.name == "style").unwrap().values, vec!["grid", "column", "inline", "list"]);
    // Keywords and the explicit unit constructor are documented.
    assert!(params::functions().iter().any(|f| f.name == "unit"));
    assert!(params::functions().iter().any(|f| f.name == "digits"));
}

#[test]
fn highlights_the_strings_of_calls() {
    let doc = "#worksheet(\"R := 7.2 kΩ\", steps: \"value\") #graph(\"P(..) = U^2/R\", mark: \"extrema\") #worksheet(units: (bar: \"100 kPa\"))[$p := 3 \"bar\"$]";
    let t = found(doc);
    for want in [
        format!("R:{}", TokenKind::VARIABLE),
        format!("7.2:{}", TokenKind::NUMBER),
        format!("kΩ:{}", TokenKind::UNIT),
        format!("value:{}", TokenKind::KEYWORD),
        format!("U:{}", TokenKind::PARAMETER),
        format!("extrema:{}", TokenKind::KEYWORD),
        format!("\"bar\":{}", TokenKind::UNIT),
    ] {
        assert!(t.contains(&want), "{want} in {t:?}");
    }
    // A solveng call's string is read wherever it is, though math outside a worksheet isn't.
    assert!(found("$p := 3 \"bar\"$").is_empty());
    // Strings of other functions stay Typst's.
    assert!(found("#text(\"R = 3 V\")").is_empty());
    // A `$` in a string doesn't start math.
    assert!(found("#graph(\"x = 3 $\") $y$").iter().all(|t| !t.starts_with('y')));
}

#[test]
fn completes_in_the_strings_of_calls() {
    let names = labels("#worksheet(\"R := 7.2 kΩ\")\n#worksheet(\"U := |\")");
    assert!(names.contains(&"R".into()) && names.contains(&"sqrt".into()));
    assert!(labels("#worksheet(\"I := 30 m|\")").contains(&"mA".into()));
    let c = complete_at("#worksheet(\"x =\", steps: \"r|\")", &Context::default()).unwrap();
    assert!(c.exclusive);
    assert_eq!(labels("#worksheet(\"x =\", steps: \"r|\")"), vec!["full", "formula", "result", "value"]);
    assert!(labels("#graph(\"y\", mark: \"|\")").contains(&"extrema".into()));
    assert!(labels("#lambdify(\"U^2/R\", \"U\", unit: \"m|\")").contains(&"mW".into()));
    assert!(labels("#worksheet[$p := 3 \"k|\"$]").contains(&"kΩ".into()));
    // Unclosed strings are what's being typed.
    assert!(labels("#worksheet(\"I := 30 m|").contains(&"mA".into()));
}

#[test]
fn completes_options_of_calls() {
    let c = complete_at("#worksheet($x =$, st|)", &Context::default()).unwrap();
    assert!(!c.exclusive);
    let steps = c.items.iter().find(|i| i.label == "steps").unwrap();
    assert_eq!(steps.insert, "steps: ");
    assert_eq!(steps.kind, ItemKind::PARAMETER);
    assert!(steps.documentation.as_deref().unwrap().contains("Default for `steps`"));
    // Given ones aren't offered again.
    assert!(!labels("#worksheet($x =$, digits: 2, |)").contains(&"digits".into()));
    assert_eq!(labels("#worksheet($x =$, steps: |)"), vec!["\"full\"", "\"formula\"", "\"result\"", "\"value\""]);
    assert!(labels("#worksheet(|)").contains(&"evaluate".into()));
    assert!(labels("#defined(style: \"|\")").contains(&"grid".into()));
}

#[test]
fn knows_the_documents_units() {
    let doc = "#show: worksheet.with(units: (bar: \"100 kPa\", mbar: \"0.001 bar\"), evaluate: false)\n```solveng\np := 3 mbar = [bar]\n```";
    assert!(found(doc).contains(&format!("mbar:{}", TokenKind::UNIT)));
    let c = complete_at("#show: worksheet.with(units: (bar: \"100 kPa\", mbar: \"0.001 bar\"), evaluate: false)\n#worksheet(\"p := 3 |\")", &Context::default()).unwrap();
    assert_eq!(c.items.iter().find(|i| i.label == "mbar").unwrap().detail, "defined · = 0.001 bar (bar = 100 kPa)");
    // The compiler's units count too, e.g. from a package.
    let context = Context::new("").with_report(
        &Report::parse(r#"{"blocks":1,"units":[{"name":"rpm","kind":"","value":"1/min"}]}"#),
        "main.typ",
        true,
    );
    assert!(complete_at("#worksheet(\"n := 3 |\")", &context).unwrap().items.iter().any(|i| i.label == "rpm" && i.detail == "defined · = 1/min"));
}

#[test]
fn shows_the_compilers_values() {
    let report = Report::parse(
        r#"{"blocks":1,"defined":[{"name":"I","kind":"given","value":"30 mA"},{"name":"t","kind":"computed","value":"115 h"},{"name":"f","kind":"function","value":"(x) = x^2"},{"name":"k","kind":"given","value":"3"}]}"#,
    );
    let context = Context::new("").with_report(&report, "main.typ", true);
    let c = complete_at("#worksheet(\"I := 30 mA\", \"t := 3450 mAh / I\")\n#worksheet(\"x := |\")", &context).unwrap();
    let get = |l: &str| c.items.iter().find(|i| name(&i.label) == l).unwrap().detail.clone();
    assert_eq!(get("I"), "= 30 mA");
    assert_eq!(get("t"), "= 3450 mAh / I = 115 h");
    // Defined where the text doesn't say.
    assert_eq!(get("k"), "= 3");
    assert_eq!(get("f(x)"), "= x^2");
}

#[test]
fn finds_unknown_units() {
    let problems = diagnostics("#lambdify(\"x\", \"x\", unit: \"furlong\")\n```solveng\nP := [5 Vv] I = [mW]\nQ := 3 = [parsec], digits(2)\n```", &Context::default());
    let words: Vec<_> = problems.iter().map(|p| p.message.split('\'').nth(1).unwrap().to_string()).collect();
    assert_eq!(words, vec!["Vv", "parsec", "furlong"]);
    assert!(diagnostics("#show: worksheet.with(units: (bar: \"100 kPa\"), evaluate: false)\n#lambdify(\"p\", \"p\", unit: \"kbar\")", &Context::default()).is_empty());
    // keywords aren't units
    assert!(diagnostics("```solveng\nx^2 = 4 -> solve(x)\nt = [h], digits(2), sci\n```", &Context::default()).is_empty());
    assert!(diagnostics("#show: worksheet\n$R := 7.2 kΩ$ $I := 3 \"mA\"$", &Context::default()).is_empty());
}

#[test]
fn hovers_names_units_and_options() {
    let doc = "#worksheet(\"R := 7.2 kΩ\")\n#worksheet(\"U := R I\", steps: \"value\")";
    let at = |w: &str| u16len(&doc[..doc.rfind(w).unwrap()]) + 1;
    assert_eq!(hover(doc, &Context::default(), at("R I")).unwrap().markdown, "```solveng\nR: kΩ = 7.2 kΩ\n```");
    assert!(hover(doc, &Context::default(), at("steps")).unwrap().markdown.starts_with("`steps` (auto, str)"));
    assert_eq!(hover(doc, &Context::default(), at("kΩ") + 1).unwrap().markdown, "`kΩ` kiloohm · built-in unit\n\n`kΩ = 10³ Ω` · `Ω = V/A = kg·m²·s⁻³·A⁻²`");
    assert!(hover("text R here", &Context::default(), 6).is_none());
    // the keywords of a clause
    let doc = "#show: worksheet\n$t := 3 h$ $t = min, digits(2)$";
    assert!(hover(doc, &Context::default(), u16len(&doc[..doc.find("digits").unwrap()]) + 1).unwrap().markdown.starts_with("`digits` keyword"));
}

#[test]
fn lists_regions() {
    let doc = "#show: worksheet\n$x := 1$ `y` ```solveng\nz := 2\n``` #worksheet(\"z\")";
    let kinds: Vec<_> = regions(doc, &Context::default()).iter().map(|r| r.kind).collect();
    assert_eq!(kinds, vec!["math", "code", "call", "string"]);
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
    let doc = "#show: worksheet\n$Delta_R := 5 cm$\n```solveng\nx := Del|\n```";
    let items = complete_at(doc, &Context::default()).unwrap().items;
    let item = items.iter().find(|i| i.label == "Δ_R: cm").unwrap();
    assert_eq!(item.insert, "Δ_R");
    assert_eq!(item.filter.as_deref(), Some("Del"));
    // The spelled name in code is the math one.
    let doc = "#show: worksheet\n$Delta_R := 5 cm$\n```solveng\nx := Delta_R\ny := Δ_R\n```";
    let at = u16len(&doc[..doc.find("Δ_R\n").unwrap()]);
    assert_eq!(usages(doc, &Context::default(), at).len(), 3);
}

#[test]
fn value_reads_its_string_as_a_calculation() {
    let doc = "#show: worksheet\n$U_2 := 5 V$\nIt is #value(\"U_2 / 2 = mV\").";
    let at = u16len(&doc[..doc.rfind("U_2").unwrap()]);
    assert_eq!(definition(doc, &Context::default(), at).unwrap().start, u16len(&doc[..doc.find("U_2").unwrap()]));
    assert_eq!(rename(doc, &Context::default(), at, "U_3").len(), 2);
    let tokens = found(doc);
    assert!(tokens.contains(&"mV:9".to_string()), "{tokens:?}");
    assert!(labels("#show: worksheet\n$U_2 := 5 V$\n#value(\"U|\")").contains(&"U_2".to_string()));
}

#[test]
fn substitute_takes_names_of_the_worksheet() {
    // highlighted as names, also where a name is spelled like a unit
    let doc = "#show: worksheet\n$V := 2 m^3$\n$Q_0 := V a$\n$w := Q_0 + 1 -> substitute(Q_0, V)$";
    let tokens = found(doc);
    assert!(tokens.contains(&"substitute:0".to_string()), "{tokens:?}");
    assert_eq!(tokens.iter().filter(|t| *t == "V:6").count(), 3, "{tokens:?}");
    // its names are uses: definition, usages and rename reach them, in math and code
    let at = u16len(&doc[..doc.rfind("Q_0").unwrap()]);
    assert_eq!(definition(doc, &Context::default(), at).unwrap().start, u16len(&doc[..doc.find("Q_0").unwrap()]));
    assert_eq!(usages(doc, &Context::default(), at).len(), 3);
    let edits = rename(doc, &Context::default(), at, "Q_1");
    assert_eq!(edits.len(), 3, "{edits:?}");
    let doc = "```solveng\nk := a b\nz := k c -> substitute(k)\n```\n";
    let at = u16len(&doc[..doc.rfind('k').unwrap()]);
    assert_eq!(usages(doc, &Context::default(), at).len(), 3);
    assert_eq!(rename(doc, &Context::default(), at, "m").len(), 3);
    // it completes as a keyword, and the defined names inside its parentheses
    let doc = "```solveng\nk := a b\nz := k c -> subst|";
    assert!(labels(doc).contains(&"substitute".to_string()), "{:?}", labels(doc));
    let doc = "```solveng\nk := a b\nz := k c -> substitute(|";
    assert!(labels(doc).contains(&"k".to_string()), "{:?}", labels(doc));
}

#[test]
fn glues_names_with_empty_strings() {
    assert_eq!(resolved_name("Delta\"\"r_R"), "Δr_R");
    assert_eq!(resolved_name("Delta\"\"Gamma\"\"Omega"), "ΔΓΩ");
    // Math and code mean the same variable; completions insert it so math can read it.
    let doc = "#show: worksheet\n$Delta\"\"r_R := 5 cm$\n```solveng\nx := Delta\"\"r_R + Δr_R\n```\n$y := Del|$";
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
fn reads_base_units_and_numbers() {
    let doc = "#show: worksheet.with(units: (ü: auto, mü: \"10ü\", dozen: 12))\n```solveng\nx := 3 ü\nn := 2 dozen\ny := 5 mü\n```\n";
    let at = |n: &str| doc[..doc.find(n).unwrap()].encode_utf16().count() as u32;
    let h = hover(doc, &Context::default(), at("ü\n")).unwrap();
    assert_eq!(h.markdown, "`ü` defined base unit: a dimension of its own");
    assert_eq!(hover(doc, &Context::default(), at("dozen\n")).unwrap().markdown, "`dozen` defined unit\n\n`dozen = 12`");
    assert_eq!(hover(doc, &Context::default(), at("mü\n")).unwrap().markdown, "`mü` defined unit\n\n`mü = 10ü`");
    assert!(found(doc).contains(&"ü:9".to_string()) && found(doc).contains(&"dozen:9".to_string()));
    assert!(diagnostics(doc, &Context::default()).is_empty());
}

#[test]
fn completes_units_after_a_colon() {
    let ctx = Context::default();
    let full = "#show: worksheet.with(units: (über: auto))\n";
    let only_units = |doc: &str| {
        let c = complete_at(&format!("{full}{doc}"), &ctx).unwrap();
        // and the types that aren't units, any and 1
        assert!(c.exclusive && c.items.iter().all(|i| i.kind == ItemKind::UNIT || i.kind == ItemKind::KEYWORD), "{doc}");
        c.items.into_iter().filter(|i| i.kind == ItemKind::UNIT).collect::<Vec<_>>()
    };
    // A parameter's unit, in code, strings and math.
    only_units("```solveng\nf(x: m|) := x\n```");
    only_units("```solveng\nf(x, y: |) := x\n```");
    only_units("#worksheet(\"f(I: |) := I\")");
    let math = only_units("$ q(x: |) := x $");
    // Math quotes what's longer than a letter (without solveng's names); Ω is found typing Omega or its name.
    let k = math.iter().find(|i| i.label == "kΩ").unwrap();
    assert_eq!((k.insert.as_str(), k.filter.as_deref()), ("\"kΩ\"", Some("kΩ kOmega kiloohm")));
    assert_eq!(math.iter().find(|i| i.label == "A").unwrap().insert, "\"A\"");
    assert!(math.iter().any(|i| i.label == "über" && i.insert == "\"über\""));
    // Variables and what a function returns take no type.
    let not_units = |doc: &str| complete_at(&format!("{full}{doc}"), &ctx).is_none_or(|c| !c.exclusive || c.items.iter().any(|i| i.kind != ItemKind::UNIT && i.kind != ItemKind::KEYWORD));
    assert!(not_units("```solveng\nk := 2\nV: |\n```"));
    assert!(not_units("```solveng\nf(x: A): |\n```"));
    // Not in an equation's value.
    assert!(complete_at(&format!("{full}```solveng\nV := |\n```"), &ctx).unwrap().items.iter().any(|i| i.kind != ItemKind::UNIT));
}

#[test]
fn completes_keywords() {
    let insert = |doc: &str, label: &str| complete_at(doc, &Context::default()).and_then(|c| c.items.into_iter().find(|i| i.label == label && i.kind == ItemKind::KEYWORD)).map(|i| i.insert);
    // as after a term, in code and math (quoted), not after a number, where as is attosecond
    assert_eq!(insert("#worksheet(\"U := 5 V\", \"I := U |\")", "as").as_deref(), Some("as"));
    assert_eq!(insert("#worksheet[$ I := U | $]", "as").as_deref(), Some("\"as\""));
    assert_eq!(insert("#worksheet(\"t := 3 |\")", "as"), None);
    // any and 1 where a type goes, not any after a cast
    assert_eq!(insert("#worksheet(\"f(x: |) := x\")", "any").as_deref(), Some("any"));
    assert_eq!(insert("#worksheet[$ f(x: |) := x $]", "any").as_deref(), Some("\"any\""));
    assert_eq!(insert("#worksheet(\"f(x: |) := x\")", "1").as_deref(), Some("1"));
    assert_eq!(insert("#worksheet(\"U := 5 V\", \"I := U as |\")", "any"), None);
    // the keywords of a clause
    assert_eq!(insert("#worksheet[$ x^2 = 4 -> s| $]", "solve").as_deref(), Some("solve($0)"));
    assert_eq!(insert("#worksheet[$ t := 2 h = min, | $]", "sci").as_deref(), Some("sci"));
}

#[test]
fn hovers_keywords() {
    let md = |doc: &str, word: &str| {
        let doc = format!("#show: worksheet\n{doc}");
        let at = u16len(&doc[..doc.find(word).unwrap()]) + 1;
        hover(&doc, &Context::default(), at).map(|h| h.markdown).unwrap_or_default()
    };
    assert!(md("```solveng\nU := 5 V\nI := U as A\n```", "as A").starts_with("`as` cast"));
    assert!(md("$ I := U \"as\" A $", "as\"").starts_with("`as` cast"));
    // after a number it's the unit
    assert!(!md("```solveng\nt := 3 as\n```", "as\n").starts_with("`as` cast"));
    assert!(md("```solveng\nf(x: any) := x\n```", "any").starts_with("`any` type"));
    assert!(md("$ x^2 = 4 -> solve(x) $", "solve").starts_with("`solve` keyword"));
}

#[test]
fn hovers_parameters_with_their_function() {
    let md = |doc: &str, word: &str, context: &Context| {
        let doc = format!("#show: worksheet\n{doc}");
        let at = u16len(&doc[..doc.rfind(word).unwrap()]);
        hover(&doc, context, at).map(|h| h.markdown).unwrap_or_default()
    };
    let doc = "$ U(x: V) := f(x \"as\" mA, y) $";
    assert_eq!(md(doc, "x \"as", &Context::default()), "`x: V` parameter of `U`");
    assert_eq!(md("```solveng\ng(t) := 2 t\n```", "t\n```", &Context::default()), "`t` parameter of `g`");
    // the compiler's types, inferred ones too
    let mut context = Context::default();
    context.defined.push(ReportValue { name: "g".into(), kind: "function".into(), value: "(t: s): s = 2 t".into() });
    assert_eq!(md("```solveng\ng(t) := 2 t\n```", "t\n```", &context), "`t: s` parameter of `g`");
    assert!(md("#graph(\"q(w) = w^2\")", "w^2", &Context::default()).contains("parameter of `q`"));
}

#[test]
fn pretty_code() {
    let (text, tokens) = pretty("U(x_2: V): A^2 = f ( x_2 as [A] , x_2 ) * [3 m^-1]", &[]);
    assert_eq!(text, "U(x₂: V): A² = f(x₂ as [A], x₂) * [3 m⁻¹]");
    let at = |s: &str| tokens.iter().find(|t| t.start == u16len(&text[..text.find(s).unwrap()])).map(|t| (t.length, t.kind));
    assert_eq!(at("x₂ as"), Some((2, TokenKind::VARIABLE)));
    assert_eq!(at("as ["), Some((2, TokenKind::KEYWORD)));
    // R_load has no subscript letters for every character
    assert_eq!(pretty("R_load = 2", &[]).0, "R_load = 2");
}

#[test]
fn units_show_their_heritage() {
    let doc = "#show: worksheet.with(units: (bar: \"100 kPa\", mbar2: \"10 bar\"))\n```solveng\np := 3 mbar2\nq := 2 mA\nr := 4 kg\n```\n";
    let md = |w: &str| hover(doc, &Context::default(), u16len(&doc[..doc.find(w).unwrap()]) + 1).unwrap().markdown;
    assert_eq!(md("mbar2\n"), "`mbar2` defined unit\n\n`mbar2 = 10 bar` · `bar = 100 kPa` · `kPa = 10³ Pa` · `Pa = N/m² = kg·m⁻¹·s⁻²`");
    assert_eq!(md("mA\n"), "`mA` milliampere · built-in unit\n\n`mA = 10⁻³ A`");
    assert_eq!(md("kg\n"), "`kg` kilogram · built-in SI base unit");
}

#[test]
fn units_named_like_variables_hover_as_units() {
    let doc = r#"#show: worksheet.with(units: (über: auto))
$V := 4 "über"$
$U(x: V) := f(x "as" V, x)$
$W := V + 1 V$
"#;
    // the character after `before`
    let md = |before: &str| hover(doc, &Context::default(), u16len(&doc[..doc.find(before).unwrap() + before.len()])).unwrap().markdown;
    assert!(md("x: ").starts_with("`V` volt"), "{}", md("x: "));
    assert!(md(r#""as" "#).starts_with("`V` volt"), "{}", md(r#""as" "#));
    // the variable where one is read
    let v = md("W := ");
    assert!(v.starts_with("```solveng units=über\nV"), "{v}");
}

#[test]
fn completions_tell_built_in_and_defined_units() {
    let doc = "#show: worksheet.with(units: (bar: \"100 kPa\"))\n```solveng\np := 3 |\n```";
    let items = complete_at(doc, &Context::default()).unwrap().items;
    let detail = |l: &str| items.iter().find(|i| i.label == l).map(|i| i.detail.clone()).unwrap_or_default();
    assert_eq!(detail("bar"), "defined · = 100 kPa");
    assert_eq!(detail("kΩ"), "kiloohm · built-in");
}

#[test]
fn warns_about_definitions_made_again() {
    let doc = "#show: worksheet\n$ I := 4 A $\n```solveng\nf(x) := x\n```\n$ I := 2 A $\n```solveng\nf(x) := 2 x\nR := 3\n```\n";
    let found: Vec<String> = diagnostics(doc, &Context::default()).into_iter().map(|p| format!("{}|{}", slice(doc, p.start, p.end - p.start), p.message)).collect();
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].starts_with("I|the variable 'I' is assigned again on line 6"), "{found:?}");
    assert!(found[1].starts_with("f|the function 'f' is assigned again on line 8"), "{found:?}");
}

#[test]
fn chains_define_with_their_last_step() {
    let doc = "#show: worksheet.with(units: (über: auto))\n$V := 4 \"über\" = 2 \"A\"$\n$W := V$\n";
    let h = hover(doc, &Context::default(), u16len(&doc[..doc.find("W := ").unwrap() + 5])).unwrap().markdown;
    assert_eq!(h, "```solveng units=über\nV: A = 2 A\n```");
}

#[test]
fn completes_quoted_units_with_the_whole_import() {
    let insert = |doc: &str, label: &str| complete_at(doc, &Context::default()).and_then(|c| c.items.into_iter().find(|i| i.label == label)).map(|i| i.insert);
    let all = "#import \"@preview/solveng:1.0.0\": *\n#show: worksheet.with(units: (über: auto))\n";
    let some = "#import \"@preview/solveng:1.0.0\": worksheet\n#show: worksheet\n";
    // built-in units go by their shorthand, imported along
    assert_eq!(insert(&format!("{all}$I := 30 |$"), "mA").as_deref(), Some("mA"));
    assert_eq!(insert(&format!("{all}$I := 30 |$"), "über").as_deref(), Some("\"über\""));
    assert_eq!(insert(&format!("{all}$f(x: |) := x$"), "kΩ").as_deref(), Some("kΩ"));
    assert_eq!(insert(&format!("{some}$I := 30 |$"), "mA").as_deref(), Some("mA"));
    assert_eq!(insert(&format!("{all}$I := 30 |$"), "A").as_deref(), Some("\"A\""));
    // after a trailing =, a unit of the document's own is "…"
    assert_eq!(insert(&format!("{all}$I := 30 mA$ $I = |$"), "über").as_deref(), Some("\"über\""));
}

#[test]
fn assignments_statements_and_results() {
    let doc = "#show: worksheet\n$I := 30 \"mA\"$\n$I = 0.03 A$\n```solveng\nR := 2 Ω\nR := 3 Ω\n```\n$U(x: V) := 2 x$\n$y := I R = \"mW\"$\n$x^2 = 4 -> solve(x)$\n";
    // := highlights as one operator, in code and math; the = and -> showing a result bold
    let toks = found(doc);
    assert_eq!(toks.iter().filter(|t| t.starts_with(":=:")).count(), 5, "{toks:?}");
    assert!(toks.contains(&format!("=:{}", TokenKind::EVALUATE)) && toks.contains(&format!("->:{}", TokenKind::EVALUATE)), "{toks:?}");
    assert!(toks.contains(&format!("\"mW\":{}", TokenKind::UNIT)) && toks.contains(&format!("solve:{}", TokenKind::KEYWORD)), "{toks:?}");
    // = states: no definition, no warning
    let warn: Vec<String> = diagnostics(doc, &Context::default()).into_iter().map(|p| p.message).collect();
    assert_eq!(warn.len(), 1, "{warn:?}");
    assert!(warn[0].starts_with("the variable 'R' is assigned again"), "{warn:?}");
    let at = u16len(&doc[..doc.find("I R").unwrap()]);
    assert_eq!(hover(doc, &Context::default(), at).unwrap().markdown, "```solveng\nI: mA = 30 mA\n```");
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
        ("#show: worksheet\n$R := 5 ohm|$", "ohm", "Ω"),
        ("#show: worksheet\n$R := 5 o|$", "o", "Ω"),
        ("#show: worksheet\n$R := 5 kiloohm|$", "kiloohm", "kΩ"),
        ("#show: worksheet\n$R := 5 Omega|$", "Omega", "Ω"),
        ("#show: worksheet\n$q(x: ohm|) := x$", "ohm", "Ω"),
        ("```solveng\nR := 5 ohm|\n```", "ohm", "Ω"),
        ("```solveng\nU := 5 volt|\n```", "volt", "V"),
        ("#worksheet(\"P := 2 watt|\")", "watt", "W"),
        ("```solveng\nT := 20 Celsius|\n```", "Celsius", "°C"),
    ] {
        assert!(offers_unit(doc, typed, label), "{label} for {typed} in {doc}");
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
    // 1.12e4 is the coefficient 1.12 and the name e4; 2.5e-10 is 2.5 · e − 10
    let doc = "#show: worksheet\n$1.12e4$";
    assert_eq!(found(doc), ["1.12:3", "e4:6"]);
    let doc = "```solveng\n2.5e-10\n```";
    assert_eq!(found(doc), ["2.5:3", "e:0", "10:3"]);
    // the constant e alone keeps its documentation
    let doc = "#value(\"2.5e-10\")";
    let at = u16len(&doc[..doc.rfind('e').unwrap()]);
    assert!(hover(doc, &Context::default(), at).unwrap().markdown.contains("Euler"));
}
