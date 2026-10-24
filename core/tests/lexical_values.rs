mod support;
use solveng_core::{api::{self, DefinedReq, LineReq}, bindings::{self, Binding, BindingReq}, env::Env, eval::value, expr::Expr};
use support::cfg;

fn binding(kind: &str, source: &str, identity: Option<&str>, display: &str) -> Binding {
    bindings::construct(BindingReq { kind: kind.into(), src: source.into(), identity: identity.map(str::to_owned), presentation: display.into(), description: display.into() }).unwrap()
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
    run(&mut env, &format!("u := t + 3 {} =", token(&other_second)));
    assert_eq!(value(&env.vars["u"].expr).unwrap().v.f(), 123.0);
    let out = api::line(&mut env, &LineReq { src: format!("t + {} =", token(&metre)), cfg: cfg(), listed: true });
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
    run(&mut env, &format!("v := {} =", token(&light)));
    assert_eq!(value(&env.vars["v"].expr).unwrap().v.f(), 299792458.0);
    let out = run(&mut env, "e + pi ->");
    assert!(out.result.unwrap().contains('e'));
    let e = binding("constant", &std::f64::consts::E.to_string(), Some("euler"), "e");
    assert_eq!(e.expr, Expr::E);
    let out = run(&mut env, &format!("⦇ln⦈({}) ->", token(&e)));
    assert_eq!(out.result.as_deref(), Some("\"1\""));
}
#[test]
fn definitions_require_closed_values_and_positive_unit_scales() {
    for source in ["x", "x := 2", "1 =", "0", "-3"] {
        assert!(bindings::construct(BindingReq { kind: "unit".into(), src: source.into(), identity: None, presentation: String::new(), description: String::new() }).is_err(), "{source}");
    }
    let a = binding("base-unit", "", Some("alpha"), "alpha");
    let b = binding("base-unit", "", Some("α"), "α");
    assert_ne!(value(&a.expr).unwrap().d, value(&b.expr).unwrap().d);
    let mut product = solveng_core::dim::Dim::ONE;
    for i in 0..32 { product = product.mul(&solveng_core::dim::Dim::named(&format!("dimension-{i}"))); }
    assert_eq!(product.0.len(), 32);
}
