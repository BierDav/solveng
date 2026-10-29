mod support;
use solveng_core::{api::{self, DefinedReq, LineReq}, bindings::{self, Binding, BindingReq}, env::Env, eval::value, expr::Expr};
use support::cfg;

fn binding(kind: &str, source: &str, identity: Option<&str>, display: &str) -> Binding {
    bindings::construct(BindingReq { kind: kind.into(), src: source.into(), identity: identity.map(str::to_owned), presentation: display.into(), description: display.into(), auto: false }).unwrap()
}
fn token(binding: &Binding) -> String { format!("⟦{}⟧", binding.encode().unwrap()) }
fn run(env: &mut Env, source: &str) -> solveng_core::sheet::Out {
    let out = api::line(env, &LineReq { src: source.into(), cfg: cfg(), listed: true });
    assert!(out.errors.is_empty(), "{source}: {:?}", out.errors);
    out
}
#[test]
fn base_identities_survive_aliases_order_and_worksheet_serialization() {
    let second = binding("base-unit", "", Some("time"), "s");
    let other_second = binding("base-unit", "", Some("time"), "second");
    let metre = binding("base-unit", "", Some("length"), "m");
    let minute = binding("unit", &format!("60 {}", token(&second)), None, "min");
    let mut env = Env::default();
    run(&mut env, &format!("t := 2 {}", token(&minute)));
    let mut bytes = Vec::new();
    ciborium::into_writer(&env, &mut bytes).unwrap();
    let mut env: Env = ciborium::from_reader(&bytes[..]).unwrap();
    run(&mut env, &format!("u := t + 3 {} =>", token(&other_second)));
    assert_eq!(value(&env.vars["u"].expr).unwrap().v.f(), 123.0);
    let out = api::line(&mut env, &LineReq { src: format!("t + {} =>", token(&metre)), cfg: cfg(), listed: true });
    assert!(!out.errors.is_empty());
    assert!(api::defined(&env, &DefinedReq { cfg: cfg() }).unwrap().iter().all(|r| !r.plain.contains("sv")));
}
#[test]
fn constants_carry_units_and_stable_identities_without_name_lookup() {
    let metre = binding("base-unit", "", Some("length"), "m");
    let second = binding("base-unit", "", Some("time"), "s");
    let light = binding("constant", &format!("299792458 {} / {}", token(&metre), token(&second)), Some("speed-of-light"), "c");
    assert_eq!(light.identity.as_deref(), Some("speed-of-light"));
    assert_eq!(Binding::decode(&light.encode().unwrap()).unwrap().identity, light.identity);
    let mut env = Env::default();
    run(&mut env, &format!("v := {} =>", token(&light)));
    assert_eq!(value(&env.vars["v"].expr).unwrap().v.f(), 299792458.0);
    let out = run(&mut env, "e + pi ->");
    assert!(out.result.unwrap().contains('e'));
    let e = binding("constant", &std::f64::consts::E.to_string(), Some("euler"), "e");
    assert_eq!(e.expr, Expr::E);
    let out = run(&mut env, &format!("⦇ln⦈({}) ->", token(&e)));
    assert_eq!(out.result.as_deref(), Some("\"1\""));
}
fn auto_unit(source: &str) -> Binding {
    bindings::construct(BindingReq { kind: "unit".into(), src: source.into(), identity: None, presentation: String::new(), description: String::new(), auto: true }).unwrap()
}

#[test]
fn units_shown_as_their_expression_are_one_factor() {
    let metre = binding("base-unit", "", Some("length"), "m");
    let second = binding("base-unit", "", Some("time"), "s");
    let cm = binding("unit", &format!("frac(1, 100) {}", token(&metre)), None, "cm");
    let mm = binding("unit", &format!("frac(1, 1000) {}", token(&metre)), None, "mm");
    let inch = auto_unit(&format!("2.54 {}", token(&cm)));
    let def = &inch.units[&inch.id];
    assert!(def.numeric && def.compound, "{def:?}");
    assert_eq!(def.display.as_deref(), Some("2.54 cm"));
    // written as results are: the number, a thin space, the unit
    let written = bindings::written(&format!("2.54 {}", token(&cm)), &cfg()).unwrap();
    assert!(written.starts_with("\"2.54\" thin #svref("), "{written}");
    // the same value written differently is another unit, as another display would be
    assert_ne!(auto_unit(&format!("25.4 {}", token(&mm))).id, inch.id);
    // a unit of a single atom is neither
    let metres = auto_unit(&token(&metre));
    assert!(!metres.units[&metres.id].numeric && !metres.units[&metres.id].compound);
    let metre_second = auto_unit(&format!("{} {}", token(&metre), token(&second)));
    assert!(!metre_second.units[&metre_second.id].numeric && metre_second.units[&metre_second.id].compound);

    let i = token(&inch);
    let mut env = Env::default();
    // after a number: a dot, never a space (1 2.54 cm reads as 12.54 cm)
    let out = run(&mut env, &format!("1 {i} = {i}"));
    assert!(out.result.as_deref().unwrap().starts_with("\"1\" dot #svref("), "{out:?}");
    let out = run(&mut env, &format!("x := 3 {i} =>"));
    assert!(out.result.as_deref().unwrap().contains(" dot #svref("), "{out:?}");
    // under a power, after a slash or next to another unit: parenthesized
    let out = run(&mut env, &format!("{i}^2 = {i}^2"));
    assert!(out.result.as_deref().unwrap().contains("lr((#svref(") && out.result.as_deref().unwrap().contains("))^(2)"), "{out:?}");
    let out = run(&mut env, &format!("2 {} / {i} =>", token(&second)));
    assert!(out.result.as_deref().unwrap().contains("\\/lr((#svref("), "{out:?}");
    let out = run(&mut env, &format!("5 {i} {} =>", token(&second)));
    assert!(out.result.as_deref().unwrap().contains("lr((#svref("), "{out:?}");
    // the formula as written joins it the same way
    let out = run(&mut env, &format!("y := 2 {i} ->"));
    assert!(out.result.as_deref().unwrap().contains(" dot "), "{out:?}");
    // a unit with a display of its own is one atom, joined by a thin space
    let shown_in = binding("unit", &format!("2.54 {}", token(&cm)), None, "in");
    let out = run(&mut env, &format!("1 {} = {}", token(&shown_in), token(&shown_in)));
    assert!(out.result.as_deref().unwrap().starts_with("\"1\" thin #svref("), "{out:?}");
    // a display of its own that starts with a number is joined by a dot
    let two = binding("unit", &format!("2 {}", token(&metre)), None, "2m");
    assert!(two.units[&two.id].numeric && !two.units[&two.id].compound);
}

#[test]
fn definitions_require_closed_values_and_positive_unit_scales() {
    for source in ["x", "x := 2", "1 =>", "0", "-3"] {
        assert!(bindings::construct(BindingReq { kind: "unit".into(), src: source.into(), identity: None, presentation: String::new(), description: String::new(), auto: false }).is_err(), "{source}");
    }
    let a = binding("base-unit", "", Some("alpha"), "alpha");
    let b = binding("base-unit", "", Some("α"), "α");
    assert_ne!(value(&a.expr).unwrap().d, value(&b.expr).unwrap().d);
    let mut product = solveng_core::dim::Dim::ONE;
    for i in 0..32 { product = product.mul(&solveng_core::dim::Dim::named(&format!("dimension-{i}"))); }
    assert_eq!(product.0.len(), 32);
}
