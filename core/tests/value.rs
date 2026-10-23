use solveng_core::{api::{self, LineReq}, env::Env, format::Cfg, parse::Mode};

fn req(src: &str, mode: Mode) -> LineReq {
    LineReq { src: src.into(), mode, cfg: Cfg::default(), listed: true }
}

fn sheet(mode: Mode) -> Env {
    let mut env = Env::default();
    for src in ["R := 4.7 kΩ", "I := 2 mA", "U := R I", "f(x: Ω) := 2 x", "g(t) := a t"] {
        let out = api::line(&mut env, &req(src, mode));
        assert!(out.errors.is_empty(), "{src}: {:?}", out.errors);
    }
    env
}

#[test]
fn shows_the_result_alone_in_code_and_math() {
    for mode in [Mode::Code, Mode::Math] {
        let env = sheet(mode);
        let value = |src: &str| api::show_value(&env, &req(src, mode));
        assert_eq!(value("U").unwrap(), "\"9.4\" thin upright(\"V\")");
        // a clause of its own picks the unit
        assert!(value("U = mV").unwrap().contains("9400"), "{:?}", value("U = mV"));
        assert_eq!(value("U / 2").unwrap(), "\"4.7\" thin upright(\"V\")");
        assert!(value("f(2 \"Ω\")").unwrap().contains("4"), "{:?}", value("f(2 \"Ω\")"));
        // a function alone is its body; a symbolic result shows as one
        assert!(value("g").unwrap().contains('a'), "{:?}", value("g"));
        assert!(value("g(2) ->").unwrap().contains('a'), "{:?}", value("g(2) ->"));
    }
}

#[test]
fn leaves_the_worksheet_as_it_is() {
    let env = sheet(Mode::Code);
    assert!(api::show_value(&env, &req("x := 2", Mode::Code)).unwrap_err().contains("nothing can be defined"));
    assert!(api::show_value(&env, &req("x := 2 =", Mode::Code)).unwrap_err().contains("nothing can be defined"));
    assert!(!env.vars.contains_key("x"));
    assert!(api::show_value(&env, &req("q + 1", Mode::Code)).unwrap_err().contains("'q'"));
}
