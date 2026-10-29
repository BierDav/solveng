//! Complex numbers: values are calculated in the complex numbers, `i` is a constant with the identity imaginary,
//! and constants stay symbolic in exact results until their expressions are substituted.

use solveng_core::{api::{self, LineReq}, bindings::{self, Binding, BindingReq}, env::Env, eval::value, expr::Expr, format::Cfg};

fn binding(kind: &str, source: &str, identity: Option<&str>) -> Result<Binding, String> {
    bindings::construct(BindingReq { kind: kind.into(), src: source.into(), identity: identity.map(str::to_owned), presentation: String::new(), description: "Ω".into(), auto: false })
}
fn token(b: &Binding) -> String {
    format!("⟦{}⟧", b.encode().unwrap())
}
fn cfg() -> Cfg {
    Cfg::default()
}

/// Markup made readable: constants by their identities' usual symbols, unit references by their names.
fn readable(m: &str) -> String {
    let mut s = m.replace("upright(\"MΩ\")", "MΩ").replace("#svconst(\"imaginary\")", "i").replace("#svconst(\"pi\")", "π").replace("#svconst(\"euler\")", "e");
    while let Some(at) = s.find("#svref(\"") {
        let rest = &s[at + 9..];
        let name_at = rest.find(", \"").unwrap() + 3;
        let end = name_at + rest[name_at..].find("\")").unwrap();
        s = format!("{}{}{}", &s[..at], &rest[name_at..end], &rest[end + 2..]);
    }
    s
}

struct Sheet {
    env: Env,
    i: String,
    pi: String,
    e: String,
    ohm: String,
}

impl Sheet {
    fn new() -> Sheet {
        Sheet {
            env: Env::default(),
            i: token(&binding("constant", "⦇sqrt⦈(-1)", Some("imaginary")).unwrap()),
            pi: token(&binding("constant", "3.141592653589793", Some("pi")).unwrap()),
            e: token(&binding("constant", "2.718281828459045", Some("euler")).unwrap()),
            ohm: token(&binding("base-unit", "", Some("resistance")).unwrap()),
        }
    }
    /// The source with i, pi, e and Ω put in for their tokens.
    fn src(&self, s: &str) -> String {
        s.replace("Ω", &self.ohm).replace("pi", &self.pi).replace("#e", &self.e).replace("#i", &self.i)
    }
    fn run(&mut self, s: &str) -> Result<String, String> {
        let src = self.src(s);
        let out = api::line(&mut self.env, &LineReq { src, cfg: cfg(), listed: true });
        match out.errors.into_iter().next() {
            Some(e) => Err(e),
            None => Ok(readable(&out.result.unwrap_or_default())),
        }
    }
}

#[test]
fn i_is_exact_and_substitute_shows_its_expression() {
    let mut s = Sheet::new();
    assert_eq!(s.run("#i ->").unwrap(), "i");
    assert_eq!(s.run("#i =>").unwrap(), "i");
    assert_eq!(s.run("#i -> ⦅substitute(#i)⦆").unwrap(), "sqrt(\"−1\")");
    assert_eq!(s.run("2 #i -> ⦅substitute(#i)⦆").unwrap(), "\"2\" i", "√−1 is i again in a result");
    assert!(s.run("2 -> ⦅substitute(#i)⦆").unwrap_err().contains("doesn't occur"));
    assert_eq!(s.run("#i^2 ->").unwrap(), "\"−1\"");
    assert_eq!(s.run("#i^7 ->").unwrap(), "-i");
    assert_eq!(s.run("(2 #i)(3 #i) ->").unwrap(), "\"−6\"");
    assert_eq!(s.run("(1 + 2 #i)(3 - #i) ->").unwrap(), "\"5\" + \"5\" i");
    assert_eq!(s.run("1/(1 + #i) ->").unwrap(), "frac(\"1\", \"2\") - frac(i, \"2\")");
    assert_eq!(s.run("(1 + #i)^2 ->").unwrap(), "\"2\" i");
    assert_eq!(s.run("x · (2 #i) + x #i ->").unwrap(), "\"3\" i dot x");
}

#[test]
fn values_are_complex() {
    let mut s = Sheet::new();
    // roots and logarithms of negative numbers, exactly and as values
    assert_eq!(s.run("⦇sqrt⦈(-4) ->").unwrap(), "\"2\" i");
    assert_eq!(s.run("⦇sqrt⦈(-4) =>").unwrap(), "\"2\" i");
    assert_eq!(s.run("⦇sqrt⦈(-2) ->").unwrap(), "i dot sqrt(\"2\")");
    assert_eq!(s.run("(-9)^(3/2) ->").unwrap(), "-\"27\" i");
    assert_eq!(s.run("⦇ln⦈(-1) ->").unwrap(), "i dot π");
    assert_eq!(s.run("⦇ln⦈(-2) =>").unwrap(), "\"0.6931\" + \"3.142\" i");
    assert_eq!(s.run("⦇arcsin⦈(2) =>").unwrap(), "\"1.571\" - \"1.317\" i");
    // a real value where there is one: (−8)^(1/3) is −2
    assert_eq!(s.run("(-8)^(1/3) =>").unwrap(), "\"−2\"");
    assert_eq!(s.run("⦇sqrt⦈(-4) + #i =>").unwrap(), "\"3\" i");
    // only real values have an order
    assert!(s.run("#i < 1 =>").unwrap_err().contains("no order"));
}

#[test]
fn euler_and_the_unit_circle() {
    let mut s = Sheet::new();
    assert_eq!(s.run("#e^(#i pi) ->").unwrap(), "\"−1\"");
    assert_eq!(s.run("#e^(#i pi/2) ->").unwrap(), "i");
    assert_eq!(s.run("⦇exp⦈(-#i pi/2) ->").unwrap(), "-i");
    assert_eq!(s.run("#e^(#i pi/3) =>").unwrap(), "\"0.5\" + \"0.866\" i");
    assert_eq!(s.run("⦇sin⦈(pi/6) ->").unwrap(), "frac(\"1\", \"2\")");
    assert_eq!(s.run("⦇cos⦈(3 pi/4) ->").unwrap(), "-frac(sqrt(\"2\"), \"2\")");
    assert_eq!(s.run("⦇tan⦈(pi) ->").unwrap(), "\"0\"");
}

#[test]
fn parts_magnitude_and_angle() {
    let mut s = Sheet::new();
    assert_eq!(s.run("⦇Re⦈(3 + 4 #i) ->").unwrap(), "\"3\"");
    assert_eq!(s.run("⦇Im⦈(3 + 4 #i) ->").unwrap(), "\"4\"");
    assert_eq!(s.run("⦇abs⦈(3 + 4 #i) ->").unwrap(), "\"5\"");
    assert_eq!(s.run("⦇conj⦈(3 + 4 #i) ->").unwrap(), "\"3\" - \"4\" i");
    assert_eq!(s.run("⦇arg⦈(-2) ->").unwrap(), "π");
    assert_eq!(s.run("⦇arg⦈(#i) ->").unwrap(), "frac(π, \"2\")");
    assert_eq!(s.run("⦇arg⦈(1 + #i) =>").unwrap(), "\"0.7854\"");
    assert_eq!(s.run("(1 + #i) = (#i + 1) =>").unwrap(), "\"1\"");
    assert_eq!(s.run("(1 + #i) != (1 - #i) =>").unwrap(), "\"1\"");
}

#[test]
fn complex_values_carry_units() {
    let mut s = Sheet::new();
    s.run("Z := 100 Ω + #i 2 pi 50 0.01 Ω").unwrap();
    assert_eq!(s.run("Z =>").unwrap(), "lr((\"100\" + \"3.142\" i)) thin Ω");
    assert_eq!(s.run("⦇abs⦈(Z) =>").unwrap(), "\"100\" thin Ω");
    assert_eq!(s.run("⦇Im⦈(Z) =>").unwrap(), "\"3.142\" thin Ω");
    assert_eq!(s.run("⦇arg⦈(Z) =>").unwrap(), "\"0.03141\"");
    // a complex value where only a real one works is an error that says what to do
    let z = s.env.vars["Z"].expr.clone();
    assert!(value(&z).unwrap_err().message().contains("Re(…)"));
}

#[test]
fn equations_are_solved_in_the_complex_numbers() {
    let mut s = Sheet::new();
    assert_eq!(s.run("x · (2 + #i) = 3 -> ⦅solve(x)⦆").unwrap(), "x = \"1.2\" - \"0.6\" i");
    assert!(!s.env.vars.contains_key("x"));
    s.run("x := x · (2 + #i) = 3 -> ⦅solve(x)⦆").unwrap();
    assert_eq!(s.run("x =>").unwrap(), "\"1.2\" - \"0.6\" i");
    assert_eq!(s.run("y^2 = -4 -> ⦅solve(y)⦆").unwrap(), "y_(1) = -\"2\" i, quad y_(2) = \"2\" i");
    assert_eq!(s.run("w^2 + 2 w + 5 = 0 -> ⦅solve(w)⦆").unwrap(), "w_(1) = \"−1\" - \"2\" i, quad w_(2) = \"−1\" + \"2\" i");
    assert_eq!(s.run("q^2 = 2 #i -> ⦅solve(q)⦆").unwrap(), "q_(1) = \"−1\" - i, quad q_(2) = \"1\" + i");
    assert_eq!(s.run("v^2 + 2 #i v + 3 = 0 -> ⦅solve(v)⦆").unwrap(), "v_(1) = -\"3\" i, quad v_(2) = i");
    // every root of a polynomial with numbers: the real one exactly, the others numerically
    assert_eq!(s.run("u^3 = 8 -> ⦅solve(u)⦆").unwrap(), "u_(1) = \"2\", quad u_(2) = \"−1\" - \"1.732\" i, quad u_(3) = \"−1\" + \"1.732\" i");
    assert_eq!(s.run("p^4 + 1 = 0 -> ⦅solve(p)⦆").unwrap().matches("i").count(), 4);
}

#[test]
fn constants_stay_symbolic_until_substituted() {
    // the expression must have the identity's value: in the complex numbers, √−1 is i
    assert_eq!(binding("constant", "⦇sqrt⦈(-1)", Some("imaginary")).unwrap().expr, Expr::I);
    assert!(binding("constant", "-⦇sqrt⦈(-1)", Some("imaginary")).unwrap_err().contains("must have its value"));
    assert!(binding("constant", "3.14", Some("pi")).unwrap_err().contains("must have its value"));
    assert!(binding("constant", "3.141592653589793", None).unwrap_err().contains("identity is required"));
    assert!(binding("constant", "x + 1", Some("thing")).unwrap_err().contains("must have a value"));
    // an identity the engine doesn't know: shown by its identity, calculated with its expression
    let mut s = Sheet::new();
    let c = token(&binding("constant", &format!("299792458 {}", s.ohm), Some("speed")).unwrap());
    let shown = format!("#svconst(\"speed\", $\"299792458\" thin Ω$)");
    assert_eq!(s.run(&format!("{c} ->")).unwrap(), shown);
    assert_eq!(s.run(&format!("{c} / 2 ->")).unwrap(), format!("frac({shown}, \"2\")"));
    assert_eq!(s.run(&format!("{c} / 2 =>")).unwrap(), "\"149.9\" thin MΩ");
    assert_eq!(s.run(&format!("{c} / 2 -> ⦅substitute({c})⦆")).unwrap(), "\"149.9\" thin MΩ");
    s.run(&format!("E := m {c}^2")).unwrap();
    assert_eq!(s.run("E ->").unwrap(), format!("m dot {shown}^(\"2\")"));
    let j2 = token(&binding("constant", &format!("2 {}", s.i), Some("two-i")).unwrap());
    assert_eq!(s.run(&format!("{j2}^2 =>")).unwrap(), "\"−4\"");
    assert_eq!(s.run(&format!("{j2}^2 -> ⦅substitute({j2})⦆")).unwrap(), "\"−4\"");
}

#[test]
fn number_is_a_value_in_a_unit_and_checks_its_dimension() {
    let mut s = Sheet::new();
    let milli = token(&binding("unit", &format!("frac(1, 1000) {}", s.ohm), None).unwrap());
    s.run("R := 2 Ω").unwrap();
    assert_eq!(s.run(&format!("⦇number⦈(R, {milli}) =>")).unwrap(), "\"2000\"");
    assert_eq!(s.run("⦇number⦈(R, Ω) / 4 ->").unwrap(), "frac(\"1\", \"2\")");
    assert!(s.run("⦇number⦈(R^2, Ω) =>").unwrap_err().contains("number: the value is in"));
    // inside a function the check waits for the argument
    s.run("f(x) := ⦇number⦈(x, Ω) + 1").unwrap();
    assert_eq!(s.run("f(3 Ω) =>").unwrap(), "\"4\"");
    assert!(s.run("f(3) =>").unwrap_err().contains("number: the value is in"));
    // a typed parameter tells it at once
    assert!(s.run("g(x: Ω) := ⦇number⦈(x^2, Ω)").unwrap_err().contains("number: the value is in"));
    assert!(s.run("⦇number⦈(R, 0 Ω) =>").unwrap_err().contains("positive"));
}

#[test]
fn infinities_times_i_stay_products() {
    use solveng_core::expr::{Expr, Val, mul, simp};
    use solveng_core::num::Num;
    // ∞·i has no real and imaginary part (∞·0 is NaN): folding it into one used to recurse forever
    let infinite = Expr::C(Val::new(Num::F(f64::NEG_INFINITY), Default::default()));
    let e = simp(&mul(vec![infinite.clone(), Expr::I]));
    assert!(e.contains(&|x| *x == Expr::I), "{e:?}");
    let e = simp(&solveng_core::expr::add(vec![Expr::C(Val::new(Num::F(f64::NAN), Default::default())), mul(vec![infinite, Expr::I])]));
    assert!(e.contains(&|x| *x == Expr::I), "{e:?}");
}
