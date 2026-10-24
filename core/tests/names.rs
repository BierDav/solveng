//! Names as the lexer reads them: sources are given as written, without the harness's translation.
mod support;
use solveng_core::{env::Env, eval::value, format::name};

fn run(env: &mut Env, source: &str) -> solveng_core::sheet::Out {
    let out = support::line_raw(env, source);
    assert!(out.errors.is_empty(), "{source}: {:?}", out.errors);
    out
}

#[test]
fn every_carrier_uses_the_same_identifier() {
    let mut env = Env::default();
    for spelling in ["Delta2tau", "\"Delta2tau\"", "Δ2τ"] {
        run(&mut env, &format!("{spelling} := 7"));
        assert_eq!(env.vars.len(), 1);
        for reference in ["Delta2tau", "\"Delta2tau\"", "Δ2τ"] {
            assert_eq!(run(&mut env, &format!("{reference} =")).result.as_deref(), Some("\"7\""));
        }
    }
    for spelling in ["Delta\"\"tau", "\"Delta\\\"\\\"tau\"", "\"Δτ\""] {
        run(&mut env, &format!("{spelling} := 8"));
        assert!(env.vars.contains_key("Δτ"));
        assert_eq!(run(&mut env, "Δτ =").result.as_deref(), Some("\"8\""));
    }
    run(&mut env, "😅2Delta_🫠 := 9");
    assert_eq!(run(&mut env, "\"😅2Δ_🫠\" =").result.as_deref(), Some("\"9\""));
}

#[test]
fn digits_belong_to_names_and_explicit_subscripts_stay_distinct() {
    let mut env = Env::default();
    for definition in ["t := 3", "t2 := 5", "t_2 := 7"] { run(&mut env, definition); }
    for (expression, expected) in [("t2", 5), ("t\"\"2", 5), ("\"t2\"", 5), ("2t", 6), ("2t2", 10), ("t 2", 6), ("t2^3", 125), ("t^23", 94143178827i64)] {
        run(&mut env, &format!("answer := {expression}"));
        assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), expected as f64, "{expression}");
    }
    assert_ne!(name("t2"), name("t_2"));
    assert!(name("t2").contains("t2"));
}

#[test]
fn quoted_function_names_and_parameters_are_normalized() {
    let mut env = Env::default();
    run(&mut env, "\"Delta2tau\"(\"alpha2beta\") := 2 alpha2beta");
    assert_eq!(run(&mut env, "Δ2τ(3) =").result.as_deref(), Some("\"6\""));
    run(&mut env, "x_tau := 4");
    assert_eq!(run(&mut env, "x_τ =").result.as_deref(), Some("\"4\""));
}

#[test]
fn delimiters_and_hidden_characters_cannot_be_escaped_into_names() {
    for source in ["result := t2.5", "\"t 2\" := 1", "t\u{200b}2 := 1", "\"t+2\" := 1"] {
        let out = support::line_raw(&mut Env::default(), source);
        assert!(!out.errors.is_empty(), "{source}");
    }
}

#[test]
fn acute_accents_are_unescaped_name_characters() {
    let mut env = Env::default();
    for definition in ["x := 2", "x´ := 3", "x´´ := 5", "´x := 7", "x´_2 := 11"] {
        run(&mut env, definition);
    }
    run(&mut env, "answer := x + x´ + x´´ + ´x + x´_2 =");
    assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 28.0);
    assert_eq!(run(&mut env, "\"x´\" =").result.as_deref(), Some("\"3\""));
    run(&mut env, "f´(t´) := x´ t´");
    assert_eq!(run(&mut env, "f´(4) =").result.as_deref(), Some("\"12\""));
}

#[test]
fn backticks_are_literal_name_characters() {
    let mut env = Env::default();
    run(&mut env, "r := 2");
    run(&mut env, "r` := 3");
    assert_eq!(run(&mut env, "r` + r =").result.as_deref(), Some("\"5\""));
}

#[test]
fn numeric_names_need_an_explicit_join_and_do_not_change_coefficients() {
    let mut env = Env::default();
    run(&mut env, "x := 3");
    run(&mut env, "2\"\"x := 7");
    assert_eq!(value(&env.vars["2x"].expr).unwrap().v.f(), 7.0);
    assert_eq!(run(&mut env, "2\"\"x =").result.as_deref(), Some("\"7\""));
    assert_eq!(run(&mut env, "2x =").result.as_deref(), Some("\"6\""));
    run(&mut env, "2\"\"f(t) := t + 1");
    assert_eq!(run(&mut env, "2\"\"f(3) =").result.as_deref(), Some("\"4\""));
    run(&mut env, &format!("length := 2{}", support::unit("m")));
}
