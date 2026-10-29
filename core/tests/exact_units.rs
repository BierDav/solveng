// A symbolic result (->) asked for in a unit stays exact: the factor of one unit, then the unit.

mod support;
use support::*;
use solveng_core::env::Env;

#[test]
fn exact_results_in_a_unit_keep_their_constants() {
    let mut env = Env::default();
    for (line, want) in [
        ("2 cm pi -> #m", "frac(π, \"50\") thin upright(\"m\")"),
        ("1 #m e^3 -> #m", "e^(\"3\") thin upright(\"m\")"),
        ("-2 cm pi -> #m", "-frac(π, \"50\") thin upright(\"m\")"),
        ("sqrt(2) #cm -> #m", "frac(sqrt(\"2\"), \"100\") thin upright(\"m\")"),
        ("2 cm pi + 1 #m -> #m", "lr((frac(π, \"50\") + \"1\")) thin upright(\"m\")"),
    ] {
        let got = result(&mut env, line).replace("#svconst(\"pi\")", "π").replace("#svconst(\"euler\")", "e");
        assert_eq!(got, want, "{line}");
    }
    // values stay values, and = is numeric
    assert_eq!(result(&mut env, "3 cm -> #m"), result(&mut env, "3 cm => #m"));
    assert!(!result(&mut env, "2 cm pi => #m").contains("svconst"));
    // the wrong unit is still an error
    assert!(fails(&mut env, "2 cm pi -> #s").contains("unit"));
}

#[test]
fn symbols_come_before_the_unit() {
    let mut env = Env::default();
    let got = result(&mut env, "3 pi #m ->").replace("#svconst(\"pi\")", "π");
    assert!(got.ends_with("upright(\"m\")") && got.contains('π'), "{got}");
}
