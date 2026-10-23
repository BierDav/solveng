use indexmap::IndexMap;
use solveng_core::{api::{self, DefinedReq, LineReq}, env::Env, format::Cfg, parse::Mode, sheet::Out, units::UnitOpt};

fn cfg(units: &[(&str, &str)], builtin_units: bool) -> Cfg {
    let units: IndexMap<String, UnitOpt> = units.iter().map(|(k, v)| (k.to_string(), match *v {
        "auto" => key("auto"),
        "none" => key("none"),
        v if v.parse::<f64>().is_ok() => UnitOpt::Num(v.parse().unwrap()),
        v => UnitOpt::Str(v.into()),
    })).collect();
    Cfg { units, builtin_units, ..Cfg::default() }
}

fn key(key: &str) -> UnitOpt {
    match key {
        "auto" => UnitOpt::Key(solveng_core::units::KeyOpt::Auto),
        _ => UnitOpt::Key(solveng_core::units::KeyOpt::None),
    }
}

fn run(env: &mut Env, cfg: &Cfg, mode: Mode, src: &str) -> Out {
    api::line(env, &LineReq { src: src.into(), mode, cfg: cfg.clone(), listed: true })
}

fn ok(env: &mut Env, cfg: &Cfg, mode: Mode, src: &str) -> String {
    let out = run(env, cfg, mode, src);
    assert!(out.errors.is_empty(), "{src}: {:?}", out.errors);
    out.result.or(out.show).unwrap_or_default()
}

#[test]
fn shown_units_follow_redefinitions() {
    let c = cfg(&[("km", "s")], true);
    let mut env = Env::default();
    for mode in [Mode::Math, Mode::Code] {
        let y = if mode == Mode::Math { "y := 10^3 \"m\" =" } else { "y := 10^3 [m] =" };
        let shown = ok(&mut env, &c, mode, y);
        assert!(shown.contains("1000") && !shown.contains("km"), "{shown}");
    }
    let shown = ok(&mut env, &c, Mode::Math, "z := 5 \"Mm\" =");
    assert!(shown.contains("\"5\"") && shown.contains("Mm"), "{shown}");
    let rows = api::defined(&env, &DefinedReq { cfg: c.clone() }).unwrap();
    let y = rows.iter().find(|r| r.name == "y").unwrap();
    assert_eq!(y.plain, "1000 m");
    // km is the second it was defined as
    let shown = ok(&mut env, &c, Mode::Math, "120 \"km\" -> min");
    assert!(shown.contains("\"2\"") && shown.contains("min"), "{shown}");
}

#[test]
fn numeric_powers_in_unit_definitions() {
    for source in ["10^3m", "10^3 m", "10^(3) m", "1000m", "10^4 dm"] {
        let c = cfg(&[("km_old", source)], true);
        let shown = ok(&mut Env::default(), &c, Mode::Math, "1 \"km_old\" -> m");
        assert!(shown.contains("\"1000\""), "{source}: {shown}");
    }
}

#[test]
fn without_builtin_units_only_own_units_exist() {
    let c = cfg(&[("m", "auto"), ("km", "1000 m"), ("t", "auto"), ("min", "60 t"), ("dz", "12")], false);
    let mut env = Env::default();
    let shown = ok(&mut env, &c, Mode::Math, "d := 5000 \"m\" =");
    assert!(shown.contains("5000") || shown.contains("\"5\""), "{shown}");
    let shown = ok(&mut env, &c, Mode::Math, "d = km");
    assert!(shown.contains("\"5\"") && shown.contains("km"), "{shown}");
    // prefixes go on own units
    let shown = ok(&mut env, &c, Mode::Math, "d -> mm");
    assert!(shown.contains("5000000") || shown.contains("10^"), "{shown}");
    let shown = ok(&mut env, &c, Mode::Code, "n := 2 dz + 1 =");
    assert!(shown.contains("25"), "{shown}");
    let shown = ok(&mut env, &c, Mode::Math, "v := 6 \"km\"/\"min\" =");
    assert!(shown.contains("km"), "{shown}");
    // a value of an own unit is picked in own units
    let shown = ok(&mut env, &c, Mode::Code, "w := 3 d / [2 t] =");
    assert!(!shown.contains("\"s\""), "{shown}");
    // unit words are the units of the worksheet
    let shown = ok(&mut env, &c, Mode::Math, "e := 2 ⦗km⦘ -> m");
    assert!(shown.contains("2000"), "{shown}");
    let hidden = cfg(&[("T", "none")], true);
    let mut env_t = Env::default();
    ok(&mut env_t, &hidden, Mode::Math, "T := 2");
    let shown = ok(&mut env_t, &hidden, Mode::Math, "y := 3 ⦗T⦘ =");
    assert!(shown.contains("\"6\"") && !shown.contains("upright"), "{shown}");
    // built-in units are gone: explicit, quoted and as targets
    for (mode, src) in [(Mode::Math, "x := 1 ⦃s⦄"), (Mode::Math, "d -> h"), (Mode::Math, "d = \"cm\" \"g\""), (Mode::Code, "x := [3 V]"), (Mode::Math, "1 ⦃kg⦄ ="), (Mode::Math, "x := 1 ⦗g⦘"), (Mode::Math, "1 ⦗kg⦘ -> km")] {
        let out = run(&mut env.clone(), &c, mode, src);
        assert!(!out.errors.is_empty() || out.prose, "{src}: {out:?}");
    }
    let out = run(&mut env.clone(), &c, Mode::Math, "x := 1 ⦃V⦄");
    assert!(out.errors.iter().any(|e| e.contains("built-in units are off")), "{out:?}");
    // nothing at all without units of one's own
    let none = cfg(&[], false);
    let out = run(&mut Env::default(), &none, Mode::Math, "x := 2 ⦃m⦄");
    assert!(!out.errors.is_empty(), "{out:?}");
    // a quoted name that is no unit is a variable
    let out = run(&mut Env::default(), &none, Mode::Math, "x := 2 \"m\" =");
    assert!(out.errors.iter().any(|e| e.contains("'m' in 'x' isn't defined")), "{out:?}");
}

#[test]
fn a_single_letter_after_the_arrow_is_the_unit() {
    let c = Cfg::default();
    for mode in [Mode::Math, Mode::Code] {
        let mut env = Env::default();
        ok(&mut env, &c, mode, if mode == Mode::Math { "x := 5 \"km\"" } else { "x := 5 km" });
        ok(&mut env, &c, mode, if mode == Mode::Math { "v := 72 \"km\"/\"h\"" } else { "v := 72 km/h" });
        for (src, want) in [("x -> m", "5000"), ("x -> m, digits(2)", "5000"), ("v -> m/s", "20"), ("v -> m s^(-1)", "20"), ("x/v -> s", "250"), ("x/v -> h", "0.06944"), ("x^2 -> m^2", "10^(7)"), ("1 / v -> s/m", "0.05")] {
            let shown = ok(&mut env, &c, mode, src);
            assert!(shown.contains(want), "{mode:?} {src}: {shown}");
        }
        // a unit of another dimension is a mistake, a variable isn't a unit
        assert!(!run(&mut env.clone(), &c, mode, "x -> s").errors.is_empty());
        let out = run(&mut env.clone(), &c, mode, "x -> q");
        assert!(!out.errors.is_empty() || out.prose, "{out:?}");
    }
    // after =, a single letter stays a variable: a statement
    let mut env = Env::default();
    ok(&mut env, &c, Mode::Math, "x := 5 \"km\"");
    let out = run(&mut env, &c, Mode::Math, "x = m");
    assert!(out.mark.is_none(), "{out:?}");
    // a variable named like a unit is still the unit after the arrow
    ok(&mut env, &c, Mode::Math, "m := 3");
    let shown = ok(&mut env, &c, Mode::Math, "x -> m");
    assert!(shown.contains("5000"), "{shown}");
}
