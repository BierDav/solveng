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
    let solve = params::functions().iter().find(|f| f.name == "solve").unwrap();
    let steps = solve.named.iter().find(|p| p.name == "steps").unwrap();
    assert_eq!(steps.values, vec!["full", "formula", "result", "value"]);
    assert!(steps.doc.starts_with("What is shown"));
    assert_eq!(solve.positional[0].name, "eqs");
    let graph = params::functions().iter().find(|f| f.name == "graph").unwrap();
    // Options documented only at solve-config.
    assert!(!graph.named.iter().find(|p| p.name == "digits").unwrap().doc.is_empty());
    let config = params::functions().iter().find(|f| f.name == "solve-config").unwrap();
    assert_eq!(config.named.iter().find(|p| p.name == "integration").unwrap().values, vec!["full", "code", "math", "none"]);
    let defined = params::functions().iter().find(|f| f.name == "defined").unwrap();
    assert_eq!(defined.named.iter().find(|p| p.name == "style").unwrap().values, vec!["grid", "column", "inline", "list"]);
}

#[test]
fn highlights_the_strings_of_calls() {
    let doc = "#def(\"R = 7.2 kΩ\") #solve(\"Q_0/2 = I t\", unit: \"h\", steps: \"value\") #graph(\"P(..) = U^2/R\", mark: \"extrema\")";
    let t = found(doc);
    for want in [
        format!("R:{}", TokenKind::VARIABLE),
        format!("7.2:{}", TokenKind::NUMBER),
        format!("kΩ:{}", TokenKind::UNIT),
        format!("h:{}", TokenKind::UNIT),
        format!("value:{}", TokenKind::KEYWORD),
        format!("U:{}", TokenKind::PARAMETER),
        format!("extrema:{}", TokenKind::KEYWORD),
    ] {
        assert!(t.contains(&want), "{want} in {t:?}");
    }
    // Strings of other functions stay Typst's.
    assert!(found("#text(\"R = 3 V\")").is_empty());
    // A `$` in a string doesn't start math.
    assert!(found("#solve(\"x = 3 $\") $y$").iter().all(|t| !t.starts_with('y')));
}

#[test]
fn completes_in_the_strings_of_calls() {
    let names = labels("#def(\"R = 7.2 kΩ\")\n#solve(\"U = |\")");
    assert!(names.contains(&"R".into()) && names.contains(&"sqrt".into()));
    assert!(labels("#def(\"I = 30 m|\")").contains(&"mA".into()));
    let c = complete_at("#solve(\"x\", unit: \"k|\")", &Context::default()).unwrap();
    assert!(c.exclusive && c.items.iter().any(|i| i.label == "kΩ"));
    assert_eq!(labels("#solve(\"x\", steps: \"r|\")"), vec!["full", "formula", "result", "value"]);
    assert!(labels("#graph(\"y\", mark: \"|\")").contains(&"extrema".into()));
    assert!(labels("#lambdify(\"U^2/R\", \"U\", unit: \"m|\")").contains(&"mW".into()));
    // Unclosed strings are what's being typed.
    assert!(labels("#def(\"I = 30 m|").contains(&"mA".into()));
}

#[test]
fn completes_options_of_calls() {
    let c = complete_at("#solve(\"x\", st|)", &Context::default()).unwrap();
    assert!(!c.exclusive);
    let steps = c.items.iter().find(|i| i.label == "steps").unwrap();
    assert_eq!(steps.insert, "steps: ");
    assert_eq!(steps.kind, ItemKind::PARAMETER);
    assert!(steps.documentation.as_deref().unwrap().contains("What is shown"));
    // Given ones aren't offered again.
    assert!(!labels("#solve(\"x\", unit: \"h\", |)").contains(&"unit".into()));
    assert_eq!(labels("#solve(\"x\", steps: |)"), vec!["\"full\"", "\"formula\"", "\"result\"", "\"value\""]);
    assert!(labels("#show: solve-config.with(integration: |)").contains(&"\"full\"".into()));
    assert!(labels("#defined(style: \"|\")").contains(&"grid".into()));
    assert!(labels("#solve(\"x\", unit: |)").contains(&"\"mA\"".into()));
}

#[test]
fn knows_the_documents_units() {
    let doc = "#solve-config(units: (bar: \"100 kPa\", mbar: \"0.001 bar\"))\n```solveng\np = 3 mbar -> bar\n```";
    assert!(found(doc).contains(&format!("mbar:{}", TokenKind::UNIT)));
    let c = complete_at("#solve-config(units: (bar: \"100 kPa\", mbar: \"0.001 bar\"))\n#def(\"p = 3 |\")", &Context::default()).unwrap();
    assert_eq!(c.items.iter().find(|i| i.label == "mbar").unwrap().detail, "defined · = 0.001 bar (bar = 100 kPa)");
    // The compiler's units count too, e.g. from a package.
    let context = Context::new("").with_report(
        &Report::parse(r#"{"blocks":1,"units":[{"name":"rpm","kind":"","value":"1/min"}]}"#),
        "main.typ",
        true,
    );
    assert!(complete_at("#def(\"n = 3 |\")", &context).unwrap().items.iter().any(|i| i.label == "rpm" && i.detail == "defined · = 1/min"));
}

#[test]
fn shows_the_compilers_values() {
    let report = Report::parse(
        r#"{"blocks":1,"defined":[{"name":"I","kind":"given","value":"30 mA"},{"name":"t","kind":"computed","value":"115 h"},{"name":"f","kind":"function","value":"(x) = x^2"},{"name":"k","kind":"given","value":"3"}]}"#,
    );
    let context = Context::new("").with_report(&report, "main.typ", true);
    let c = complete_at("#def(\"I = 30 mA\", \"t = 3450 mAh / I\")\n#solve(\"x = |\")", &context).unwrap();
    let get = |l: &str| c.items.iter().find(|i| name(&i.label) == l).unwrap().detail.clone();
    assert_eq!(get("I"), "= 30 mA");
    assert_eq!(get("t"), "= 3450 mAh / I = 115 h");
    // Defined where the text doesn't say.
    assert_eq!(get("k"), "= 3");
    assert_eq!(get("f(x)"), "= x^2");
}

#[test]
fn finds_unknown_units() {
    let problems = diagnostics("#solve(\"x = 3\", unit: \"kWh\") #solve(\"y\", unit: \"furlong\")\n```solveng\nP = [5 Vv] I -> mW\n```", &Context::default());
    let words: Vec<_> = problems.iter().map(|p| p.message.split('\'').nth(1).unwrap().to_string()).collect();
    assert_eq!(words, vec!["Vv", "furlong"]);
    assert!(diagnostics("#solve-config(units: (bar: \"100 kPa\"))\n#solve(\"p\", unit: \"kbar\")", &Context::default()).is_empty());
    assert!(diagnostics("#show: solve-block\n$R = 7.2 \"k\"Omega$ $I = 3 \"mA\"$", &Context::default()).is_empty());
}

#[test]
fn hovers_names_units_and_options() {
    let doc = "#def(\"R = 7.2 kΩ\")\n#solve(\"U = R I\", steps: \"value\")";
    let at = |w: &str| u16len(&doc[..doc.rfind(w).unwrap()]) + 1;
    assert_eq!(hover(doc, &Context::default(), at("R I")).unwrap().markdown, "```solveng\nR: kΩ = 7.2 kΩ\n```");
    assert!(hover(doc, &Context::default(), at("steps")).unwrap().markdown.starts_with("`steps` (auto, str)"));
    assert_eq!(hover(doc, &Context::default(), at("kΩ") + 1).unwrap().markdown, "`kΩ` kiloohm · built-in unit\n\n`kΩ = 10³ Ω` · `Ω = V/A = kg·m²·s⁻³·A⁻²`");
    assert!(hover("text R here", &Context::default(), 6).is_none());
}

#[test]
fn lists_regions() {
    let doc = "#show: solve-block\n$x = 1$ `y` #solve(\"z\")";
    let kinds: Vec<_> = regions(doc, &Context::default()).iter().map(|r| r.kind).collect();
    assert_eq!(kinds, vec!["math", "call", "string"]);
}

#[test]
fn calculates_math_given_to_def() {
    // Math passed to def is calculated without a solve-block.
    let doc = "#def($I = 30 \"mA\"$)";
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
    let doc = "#show: solve-block\n$Delta_R = 5 \"cm\"$\n```solveng\nx = Del|\n```";
    let items = complete_at(doc, &Context::default()).unwrap().items;
    let item = items.iter().find(|i| i.label == "Δ_R: cm").unwrap();
    assert_eq!(item.insert, "Δ_R");
    assert_eq!(item.filter.as_deref(), Some("Delta_R"));
    // The spelled name in code is the math one.
    let doc = "#show: solve-block\n$Delta_R = 5 \"cm\"$\n```solveng\nx = Delta_R\ny = Δ_R\n```";
    let at = u16len(&doc[..doc.find("Δ_R\n").unwrap()]);
    assert_eq!(usages(doc, &Context::default(), at).len(), 3);
}

#[test]
fn highlights_inlined_formulas_in_math() {
    let doc = "#show: solve-block\n$x = y$\n$z = @x$\n$w = @Q_0 + 1$";
    let tokens = found(doc);
    assert!(tokens.contains(&"@x:7".to_string()), "{tokens:?}");
    assert!(tokens.contains(&"@Q_0:7".to_string()), "{tokens:?}");
    let at = u16len(&doc[..doc.find("@x").unwrap()]) + 1;
    assert_eq!(usages(doc, &Context::default(), at).len(), 2);
}

#[test]
fn glues_names_with_empty_strings() {
    assert_eq!(resolved_name("Delta\"\"r_R"), "Δr_R");
    assert_eq!(resolved_name("Delta\"\"Gamma\"\"Omega"), "ΔΓΩ");
    // Math and code mean the same variable; completions insert it so math can read it.
    let doc = "#show: solve-block\n$Delta\"\"r_R = 5 \"cm\"$\n```solveng\nx = Delta\"\"r_R + Δr_R\n```\n$y = Del|$";
    let items = complete_at(doc, &Context::default()).unwrap().items;
    let item = items.iter().find(|i| i.label == "Δr_R: cm").unwrap();
    assert_eq!(item.insert, "Δ\"\"r_R");
    assert_eq!(item.filter.as_deref(), Some("Delta\"\"r_R"));
    let doc = doc.replace('|', "");
    assert!(found(&doc).contains(&"Delta\"\"r:6".to_string()), "{:?}", found(&doc));
    let at = u16len(&doc[..doc.find("Δr_R").unwrap()]);
    assert_eq!(usages(&doc, &Context::default(), at).len(), 3);
}

#[test]
fn reads_base_units_and_numbers() {
    let doc = "#show: solve-block\n#solve-config(integration: \"full\", units: (ü: auto, mü: \"10ü\", dozen: 12))\n`x = 3 ü`\n`n = 2 dozen`\n`y = 5 mü`\n";
    let at = |n: &str| doc[..doc.find(n).unwrap()].encode_utf16().count() as u32;
    let h = hover(doc, &Context::default(), at("ü`") ).unwrap();
    assert_eq!(h.markdown, "`ü` defined base unit: a dimension of its own");
    assert_eq!(hover(doc, &Context::default(), at("dozen`")).unwrap().markdown, "`dozen` defined unit\n\n`dozen = 12`");
    assert_eq!(hover(doc, &Context::default(), at("mü`")).unwrap().markdown, "`mü` defined unit\n\n`mü = 10ü`");
    assert!(found(doc).contains(&"ü:9".to_string()) && found(doc).contains(&"dozen:9".to_string()));
    assert!(diagnostics(doc, &Context::default()).is_empty());
}

#[test]
fn completes_units_after_a_colon() {
    let ctx = Context::default();
    let full = "#show: solve-block\n#solve-config(integration: \"full\", units: (über: auto))\n";
    let only_units = |doc: &str| {
        let c = complete_at(&format!("{full}{doc}"), &ctx).unwrap();
        // and the types that aren't units, any and 1
        assert!(c.exclusive && c.items.iter().all(|i| i.kind == ItemKind::UNIT || i.kind == ItemKind::KEYWORD), "{doc}");
        c.items.into_iter().filter(|i| i.kind == ItemKind::UNIT).collect::<Vec<_>>()
    };
    // A parameter's unit, in code, strings and math.
    only_units("`f(x: m|) = x`");
    only_units("`f(x, y: |) = x`");
    only_units("#def(\"f(I: |) = I\")");
    let math = only_units("$ q(x: |) = x $");
    // Math quotes what's longer than a letter; Ω is found typing Omega.
    let k = math.iter().find(|i| i.label == "kΩ").unwrap();
    assert_eq!((k.insert.as_str(), k.filter.as_deref()), ("\"kΩ\"", Some("kOmega")));
    assert_eq!(math.iter().find(|i| i.label == "A").unwrap().insert, "A");
    assert!(math.iter().any(|i| i.label == "über" && i.insert == "\"über\""));
    // Variables and what a function returns take no type.
    let not_units = |doc: &str| complete_at(&format!("{full}{doc}"), &ctx).is_none_or(|c| !c.exclusive || c.items.iter().any(|i| i.kind != ItemKind::UNIT && i.kind != ItemKind::KEYWORD));
    assert!(not_units("`k = 2`\n`V: |`"));
    assert!(not_units("$ V: \"m|\" = 3 A $"));
    assert!(not_units("`f(x: A): |`"));
    // Not in an equation's value.
    assert!(complete_at(&format!("{full}`V = |`"), &ctx).unwrap().items.iter().any(|i| i.kind != ItemKind::UNIT));
}

#[test]
fn completes_keywords() {
    let insert = |doc: &str, label: &str| complete_at(doc, &Context::default()).and_then(|c| c.items.into_iter().find(|i| i.label == label && i.kind == ItemKind::KEYWORD)).map(|i| i.insert);
    // as after a term, in code and math (quoted), not after a number, where as is attosecond
    assert_eq!(insert("#def(\"U = 5 V\", \"I = U |\")", "as").as_deref(), Some("as"));
    assert_eq!(insert("#solve-block[$ I = U | $]", "as").as_deref(), Some("\"as\""));
    assert_eq!(insert("#def(\"t = 3 |\")", "as"), None);
    // any and 1 where a type goes, not any after a cast
    assert_eq!(insert("#def(\"f(x: |) = x\")", "any").as_deref(), Some("any"));
    assert_eq!(insert("#solve-block[$ f(x: |) = x $]", "any").as_deref(), Some("\"any\""));
    assert_eq!(insert("#def(\"f(x: |) = x\")", "1").as_deref(), Some("1"));
    assert_eq!(insert("#def(\"U = 5 V\", \"I = U as |\")", "any"), None);
}

#[test]
fn hovers_keywords() {
    let md = |doc: &str, word: &str| {
        let doc = format!("#show: solve-block\n#solve-config(integration: \"full\")\n{doc}");
        let at = u16len(&doc[..doc.find(word).unwrap()]) + 1;
        hover(&doc, &Context::default(), at).map(|h| h.markdown).unwrap_or_default()
    };
    assert!(md("`U = 5 V`\n`I = U as A`", "as A").starts_with("`as` cast"));
    assert!(md("$ I = U \"as\" \"A\" $", "as\"").starts_with("`as` cast"));
    // after a number it's the unit
    assert!(!md("`t = 3 as`", "as`").starts_with("`as` cast"));
    assert!(md("`f(x: any) = x`", "any").starts_with("`any` type"));
}

#[test]
fn hovers_parameters_with_their_function() {
    let md = |doc: &str, word: &str, context: &Context| {
        let doc = format!("#show: solve-block\n#solve-config(integration: \"full\")\n{doc}");
        let at = u16len(&doc[..doc.find(word).unwrap()]);
        hover(&doc, context, at).map(|h| h.markdown).unwrap_or_default()
    };
    let doc = "$ U(x: V) = f(x \"as\" \"mA\", y) $";
    assert_eq!(md(doc, "x \"as", &Context::default()), "`x: V` parameter of `U`");
    assert_eq!(md("`g(t) = 2 t`", "t`", &Context::default()), "`t` parameter of `g`");
    // the compiler's types, inferred ones too
    let mut context = Context::default();
    context.defined.push(ReportValue { name: "g".into(), kind: "function".into(), value: "(t: s): s = 2 t".into() });
    assert_eq!(md("`g(t) = 2 t`", "t`", &context), "`t: s` parameter of `g`");
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
    let doc = "#show: solve-block\n#solve-config(integration: \"full\", units: (bar: \"100 kPa\", mbar2: \"10 bar\"))\n`p = 3 mbar2`\n`q = 2 mA`\n`r = 4 kg`\n";
    let md = |w: &str| hover(doc, &Context::default(), u16len(&doc[..doc.find(w).unwrap()]) + 1).unwrap().markdown;
    assert_eq!(md("mbar2`"), "`mbar2` defined unit\n\n`mbar2 = 10 bar` · `bar = 100 kPa` · `kPa = 10³ Pa` · `Pa = N/m² = kg·m⁻¹·s⁻²`");
    assert_eq!(md("mA`"), "`mA` milliampere · built-in unit\n\n`mA = 10⁻³ A`");
    assert_eq!(md("kg`"), "`kg` kilogram · built-in SI base unit");
}

#[test]
fn units_named_like_variables_hover_as_units() {
    let doc = r#"#show: solve-block
#solve-config(integration: "full", units: (über: auto))
$V: "über" = 4 "über"$
$U(x: V) = f(x "as" "V", x)$
$W = V + 1 "V"$
"#;
    // the character after `before`
    let md = |before: &str| hover(doc, &Context::default(), u16len(&doc[..doc.find(before).unwrap() + before.len()])).unwrap().markdown;
    assert!(md("x: ").starts_with("`V` volt"), "{}", md("x: "));
    assert!(md(r#""as" ""#).starts_with("`V` volt"), "{}", md(r#""as" ""#));
    // the variable where one is read
    let v = md("W = ");
    assert!(v.starts_with("```solveng units=über\nV"), "{v}");
    // über after a number is the defined unit
    let code = v.lines().nth(1).unwrap();
    let (text, tokens) = pretty(code, &["über".to_string()]);
    let at = u16len(&text[..text.rfind("über").unwrap()]);
    assert_eq!(tokens.iter().find(|t| t.start == at).map(|t| t.kind), Some(TokenKind::UNIT), "{text}");
}

#[test]
fn completions_tell_built_in_and_defined_units() {
    let doc = "#show: solve-block\n#solve-config(integration: \"full\", units: (bar: \"100 kPa\"))\n`p = 3 |`";
    let items = complete_at(doc, &Context::default()).unwrap().items;
    let detail = |l: &str| items.iter().find(|i| i.label == l).map(|i| i.detail.clone()).unwrap_or_default();
    assert_eq!(detail("bar"), "defined · = 100 kPa");
    assert_eq!(detail("kΩ"), "kiloohm · built-in");
}

#[test]
fn warns_about_definitions_made_again() {
    let doc = "#show: solve-block\n#solve-config(integration: \"full\")\n$ I = 4 \"A\" $\n`f(x) = x`\n$ I := 2 \"A\" $\n`f(x) := 2 x`\n`R = 3`\n";
    let found: Vec<String> = diagnostics(doc, &Context::default()).into_iter().map(|p| format!("{}|{}", slice(doc, p.start, p.end - p.start), p.message)).collect();
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].starts_with("I|the variable 'I' is assigned again on line 5"), "{found:?}");
    assert!(found[1].starts_with("f|the function 'f' is assigned again on line 6"), "{found:?}");
}

#[test]
fn chains_define_with_their_last_step() {
    let doc = "#show: solve-block\n#solve-config(integration: \"full\", units: (über: auto))\n$V: \"über\" = 4 \"über\" = 2 \"A\"$\n$W = V$\n";
    let h = hover(doc, &Context::default(), u16len(&doc[..doc.find("W = ").unwrap() + 4])).unwrap().markdown;
    assert_eq!(h, "```solveng units=über\nV: über = 2 A\n```");
}

#[test]
fn completes_unquoted_units_with_the_whole_import() {
    let insert = |doc: &str, label: &str| complete_at(doc, &Context::default()).and_then(|c| c.items.into_iter().find(|i| i.label == label)).map(|i| i.insert);
    let all = "#import \"@preview/solveng:0.2.0\": *\n#show: solve-block\n#solve-config(integration: \"full\", units: (über: auto))\n";
    let some = "#import \"@preview/solveng:0.2.0\": def\n#show: solve-block\n#solve-config(integration: \"full\")\n";
    assert_eq!(insert(&format!("{all}$I = 30 |$"), "mA").as_deref(), Some("mA"));
    assert_eq!(insert(&format!("{all}$I = 30 |$"), "über").as_deref(), Some("\"über\""));
    assert_eq!(insert(&format!("{all}$f(x: |) = x$"), "kΩ").as_deref(), Some("kΩ"));
    assert_eq!(insert(&format!("{some}$I = 30 |$"), "mA").as_deref(), Some("\"mA\""));
}

#[test]
fn assignments_and_statements() {
    let doc = "#show: solve-block\n#solve-config(integration: \"full\")\n$I = 30 mA$\n$I = 0.03 A$\n`R := 2 Ω`\n`R := 3 Ω`\n$U(x: V) := 2 x$\n`U(o) = 20`\n`y = I R`\n";
    // := highlights as one operator, in code and math
    let toks = found(doc);
    assert!(toks.iter().filter(|t| t.starts_with(":=:")).count() == 3, "{toks:?}");
    // = on a defined name is a statement: no warning, the hover keeps the definition
    let warn: Vec<String> = diagnostics(doc, &Context::default()).into_iter().map(|p| p.message).collect();
    assert_eq!(warn.len(), 1, "{warn:?}");
    assert!(warn[0].starts_with("the variable 'R' is assigned again"), "{warn:?}");
    let at = u16len(&doc[..doc.find("I R").unwrap()]);
    assert_eq!(hover(doc, &Context::default(), at).unwrap().markdown, "```solveng\nI: mA = 30 mA\n```");
    let at = u16len(&doc[..doc.find("U(o)").unwrap()]);
    assert!(hover(doc, &Context::default(), at).unwrap().markdown.contains("U(x: V)"));
}

#[test]
fn solves_for_a_name_after_an_assigned_statement() {
    let doc = "```solveng\nL := Q = 3.3 kΩ = 3300 Ω -> Q\nP = [5 V] [2 A] -> mW\n```\n";
    let toks = found(doc);
    assert!(toks.contains(&"Q:6".to_string()) && toks.iter().filter(|t| *t == "Q:6").count() == 2, "{toks:?}");
    assert!(toks.contains(&"mW:9".to_string()), "{toks:?}");
}
