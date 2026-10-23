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

#[test]
fn a_unit_after_the_trailing_mark_stays_a_unit_in_code() {
    for mode in [Mode::Code, Mode::Math] {
        let env = sheet(mode);
        let value = |src: &str| api::show_value(&env, &req(src, Mode::Code));
        for src in ["U = mV", "U = [mV]", "U -> mV", "U = mV, digits(2)"] {
            let shown = value(src).unwrap_or_else(|e| panic!("{src}: {e}"));
            assert!(shown.contains("9400") && shown.contains("mV"), "{src}: {shown}");
        }
    }
    // a value that can't be calculated says why, not that the unit is a variable
    let mut env = Env::default();
    assert!(api::line(&mut env, &req("U := 5 V", Mode::Math)).errors.is_empty());
    let e = api::show_value(&env, &req("U = mV", Mode::Code)).unwrap_err();
    assert!(e.contains("'V'") && !e.contains("'mV'"), "{e}");
    let mut code = Env::default();
    for src in ["Y := 3 V", "Y = mV", "Y -> mV", "Y = [mV]"] {
        let out = api::line(&mut code, &req(src, Mode::Code));
        assert!(out.errors.is_empty(), "{src}: {:?}", out.errors);
        if src != "Y := 3 V" { assert!(out.show.unwrap().contains("3000"), "{src}"); }
    }
}
