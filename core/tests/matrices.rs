use solveng_core::{api::{self, LineReq}, env::Env, eval::value, expr::{Expr, simp}, format::Cfg, parse::Mode};

fn run(env: &mut Env, src: &str, mode: Mode) -> solveng_core::sheet::Out {
    let out = api::line(env, &LineReq { src: src.into(), mode, cfg: Cfg::default(), listed: true });
    assert!(out.errors.is_empty(), "{src}: {:?}", out.errors);
    out
}

fn fails(env: &mut Env, src: &str, mode: Mode) -> String {
    let out = api::line(env, &LineReq { src: src.into(), mode, cfg: Cfg::default(), listed: true });
    assert!(!out.errors.is_empty(), "{src} should fail: {out:?}");
    out.errors.join("; ")
}

fn result(env: &mut Env, src: &str, mode: Mode) -> String {
    run(env, src, mode).result.unwrap_or_default()
}

/// The elements of a defined vector or matrix as numbers in SI, row by row.
fn numbers(env: &Env, name: &str) -> Vec<f64> {
    let e = simp(&env.bind(&env.vars[name].expr, &Default::default()).unwrap());
    let elements: Vec<Expr> = match e {
        Expr::Vector(v) => v,
        Expr::Matrix(rs) => rs.into_iter().flatten().collect(),
        e => vec![e],
    };
    elements.iter().map(|e| value(e).unwrap().v.f()).collect()
}

fn close(a: &[f64], b: &[f64]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 1e-12 * x.abs().max(y.abs()).max(1.0))
}

#[test]
fn arithmetic_follows_the_order_of_products() {
    for mode in [Mode::Code, Mode::Math] {
        let mut env = Env::default();
        run(&mut env, "A := mat(1, 2; 3, 4)", mode);
        run(&mut env, "B := mat(0, 1; 1, 0)", mode);
        run(&mut env, "v := vec(1, 2)", mode);
        for (src, expected) in [
            ("AB := A B", vec![2.0, 1.0, 4.0, 3.0]),
            ("BA := B A", vec![3.0, 4.0, 1.0, 2.0]),
            ("S := A + 2 B - 1", vec![0.0, 3.0, 4.0, 3.0]),
            ("Av := A v", vec![5.0, 11.0]),
            ("vA := v^T A", vec![7.0, 10.0]),
            ("dot := v v", vec![5.0]),
            ("P := A^3", vec![37.0, 54.0, 81.0, 118.0]),
            ("P0 := A^0", vec![1.0, 0.0, 0.0, 1.0]),
            ("Q := A^(-2) A^2", vec![1.0, 0.0, 0.0, 1.0]),
            ("T := A^T", vec![1.0, 3.0, 2.0, 4.0]),
            ("D := A/2", vec![0.5, 1.0, 1.5, 2.0]),
        ] {
            run(&mut env, src, mode);
            let name = src.split(" :=").next().unwrap();
            assert!(close(&numbers(&env, name), &expected), "{src}: {:?}", numbers(&env, name));
        }
        // functions keep the order their body is written in
        run(&mut env, "f(X, Y) := Y X", mode);
        run(&mut env, "F := f(A, B)", mode);
        assert!(close(&numbers(&env, "F"), &[3.0, 4.0, 1.0, 2.0]));
    }
}

#[test]
fn determinants_inverses_and_systems_are_exact() {
    for mode in [Mode::Code, Mode::Math] {
        let mut env = Env::default();
        run(&mut env, "A := mat(2, -3, 1; 2, 0, -1; 1, 4, 5)", mode);
        assert_eq!(result(&mut env, "det(A) =", mode), "\"49\"");
        assert_eq!(result(&mut env, "|A| =", mode), "\"49\"");
        assert_eq!(result(&mut env, "tr(A) =", mode), "\"7\"");
        assert_eq!(result(&mut env, "rank(mat(1, 2; 2, 4)) =", mode), "\"1\"");
        assert_eq!(result(&mut env, "mat(1, 2; 3, 4)^(-1) ->", mode), "mat(\"−2\", \"1\"; frac(\"3\", \"2\"), -frac(\"1\", \"2\"))");
        run(&mut env, "I_3 := A A^(-1)", mode);
        assert!(close(&numbers(&env, "I_3"), &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]));
        run(&mut env, "x := A^(-1) vec(1, 2, 3)", mode);
        let x = numbers(&env, "x");
        assert!(close(&x, &[51.0 / 49.0, 19.0 / 49.0, 4.0 / 49.0]), "{x:?}");
        assert_eq!(result(&mut env, "|vec(3, 4)| =", mode), "\"5\"");
        assert_eq!(result(&mut env, "rows(A) + cols(vec(1, 2)) =", mode), "\"4\"");
    }
}

#[test]
fn symbolic_entries_give_formulas() {
    let mut env = Env::default();
    run(&mut env, "M := mat(a, b; c, d)", Mode::Code);
    assert_eq!(result(&mut env, "det(M) ->", Mode::Code), "a d - b c");
    run(&mut env, "N := M M^(-1) -> simplify", Mode::Code);
    assert_eq!(result(&mut env, "N ->", Mode::Code), "mat(\"1\", \"0\"; \"0\", \"1\")");
    // names get their values later
    run(&mut env, "a := 1", Mode::Code);
    run(&mut env, "b := 2", Mode::Code);
    run(&mut env, "c := 3", Mode::Code);
    run(&mut env, "d := 4", Mode::Code);
    assert_eq!(result(&mut env, "det(M) =", Mode::Code), "\"−2\"");
    // a 4×4 symbolic determinant by fraction-free elimination
    let out = result(&mut env, "det(mat(p, 1, 0, 0; 1, p, 1, 0; 0, 1, p, 1; 0, 0, 1, p)) ->", Mode::Code);
    assert_eq!(out, "p^(\"4\") - \"3\" dot p^(\"2\") + \"1\"");
}

#[test]
fn units_follow_rows_and_columns() {
    for mode in [Mode::Code, Mode::Math] {
        let mut env = Env::default();
        let (ohm, volt) = if mode == Mode::Code { ("Ω", "V") } else { ("⦃Ω⦄", "⦃V⦄") };
        run(&mut env, &format!("R := mat(2 {ohm}, 1 {ohm}; 1 {ohm}, 3 {ohm})"), mode);
        run(&mut env, &format!("i := R^(-1) vec(1 {volt}, 2 {volt})"), mode);
        assert!(close(&numbers(&env, "i"), &[0.2, 0.6]));
        assert!(result(&mut env, "i =", mode).contains("mA"));
        assert!(result(&mut env, "R^(-1) =", mode).contains("mS"));
        assert_eq!(result(&mut env, "det(R) =", mode), "\"5\" thin upright(\"Ω\")^(2)");
        // a row of metres and a row of seconds: a unit per row
        run(&mut env, "G := mat(1 m, 2 m; 3 s, 4 s)", Mode::Code);
        assert!(result(&mut env, "det(G) =", Mode::Code).contains("m"));
        assert!(fails(&mut env, "det(mat(1 m, 2 s; 3 s, 4 m)) =", Mode::Code).contains("unit per row and per column"));
    }
}

#[test]
fn norms_blocks_and_solving_instead_of_inverting() {
    for mode in [Mode::Code, Mode::Math] {
        let mut env = Env::default();
        assert_eq!(result(&mut env, "norm(vec(3, 4)) =", mode), "\"5\"");
        // of a matrix, the Frobenius norm
        assert_eq!(result(&mut env, "‖mat(1, 2; 2, 4)‖ =", mode), "\"5\"");
        assert_eq!(result(&mut env, "2 ‖vec(0, 3)‖ =", mode), "\"6\"");
        run(&mut env, "A := mat(2, 1; 1, 3)", mode);
        // a block of the identity next to A, and A over a row
        run(&mut env, "B := mat(A, identity(2); 0, 0, 1, 1)", mode);
        assert!(close(&numbers(&env, "B"), &[2.0, 1.0, 1.0, 0.0, 1.0, 3.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0]));
        // A^(-1) b is solved, exactly, without the inverse
        assert_eq!(result(&mut env, "A^(-1) vec(3, 5) ->", mode), "vec(frac(\"4\", \"5\"), frac(\"7\", \"5\"))");
    }
}

#[test]
fn cross_products_and_element_access() {
    for mode in [Mode::Code, Mode::Math] {
        let mut env = Env::default();
        run(&mut env, "a := vec(1, 0, 0)", mode);
        run(&mut env, "b := vec(0, 1, 0)", mode);
        run(&mut env, "c := a × b", mode);
        assert!(close(&numbers(&env, "c"), &[0.0, 0.0, 1.0]));
        run(&mut env, "k := b × a", mode);
        assert!(close(&numbers(&env, "k"), &[0.0, 0.0, -1.0]));
        // × of scalars stays the product
        assert_eq!(result(&mut env, "2 × 3 =", mode), "\"6\"");
        run(&mut env, "M := mat(1, 2, 3; 4, 5, 6)", mode);
        assert_eq!(result(&mut env, "at(M, 1, 2) =", mode), "\"6\"");
        assert_eq!(result(&mut env, "at(c, 2) =", mode), "\"1\"");
        run(&mut env, "W := mat(M, vec(7, 8))", mode);
        assert!(close(&numbers(&env, "W"), &[1.0, 2.0, 3.0, 7.0, 4.0, 5.0, 6.0, 8.0]));
        run(&mut env, "H := mat(M; vec(7, 8, 9)^T)", mode);
        assert!(close(&numbers(&env, "H"), &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]));
        run(&mut env, "E := identity(2)", mode);
        assert!(close(&numbers(&env, "E"), &[1.0, 0.0, 0.0, 1.0]));
    }
}

#[test]
fn matrix_equations_solve_element_by_element() {
    for mode in [Mode::Code, Mode::Math] {
        let mut env = Env::default();
        run(&mut env, "A := mat(2, 1; 1, 3)", mode);
        run(&mut env, "b := vec(3, 5)", mode);
        run(&mut env, "A vec(x, y) = b -> solve(x, y)", mode);
        assert_eq!(value(&env.vars["x"].expr).unwrap().v.f(), 0.8);
        assert_eq!(value(&env.vars["y"].expr).unwrap().v.f(), 1.4);
    }
}

#[test]
fn sizes_that_do_not_fit_are_reported() {
    let mut env = Env::default();
    run(&mut env, "A := mat(1, 2; 3, 4)", Mode::Code);
    run(&mut env, "C := mat(1, 2, 3; 4, 5, 6)", Mode::Code);
    for (src, message) in [
        ("A + vec(1, 2, 3) =", "sizes must match"),
        ("C A =", "columns of the first must match the rows of the second"),
        ("C^2 =", "only a square matrix"),
        ("det(C) =", "square matrix"),
        ("mat(1, 2; 2, 4)^(-1) =", "singular"),
        ("mat(1, 2; 2, 4)^(-1) vec(1, 2) =", "singular"),
        ("mat(A, vec(1, 2, 3)) =", "same number of rows"),
        ("mat(A; vec(1, 2, 3)^T) =", "same number of columns"),
        ("at(A, 2, 0) =", "outside a 2×2 matrix"),
        ("at(A, 1) =", "at(matrix, row, column)"),
        ("A =  // fine", ""),
        ("sin(A) =", "sin takes a number"),
        ("mat(1, 2; 3) =", "same number of elements"),
        ("2 mat(1, 2; 3) mat(1, 2; 3, 4) =", "same number of elements"),
        ("mat(A, vec(1, 2, 3)) A =", "same number of rows"),
        ("vec(1, 2) × vec(1, 2) =", "3 components"),
    ] {
        if message.is_empty() {
            run(&mut env, src, Mode::Code);
            continue;
        }
        let e = fails(&mut env, src, Mode::Code);
        assert!(e.contains(message), "{src}: {e}");
    }
}

#[test]
fn shown_as_typst_matrices() {
    let mut env = Env::default();
    let out = run(&mut env, "A := mat(1, 2; 3, 4) =", Mode::Code);
    assert_eq!(out.result.as_deref(), Some("mat(\"1\", \"2\"; \"3\", \"4\")"));
    // a matrix of values is given, not computed: no step repeating it
    let out = run(&mut env, "A =", Mode::Code);
    assert_eq!(out.show.as_deref(), Some("A = mat(\"1\", \"2\"; \"3\", \"4\")"));
    let out = run(&mut env, "A^T det(A) ->", Mode::Code);
    assert_eq!(out.show.as_deref(), Some("A^(T) det(A) -> mat(\"−2\", \"−6\"; \"−4\", \"−8\")"));
    let out = run(&mut env, "rank(A) =", Mode::Code);
    assert_eq!(out.show.as_deref(), Some("op(\"rank\")(A) = \"2\""));
}

#[test]
fn worksheet_functions_replace_matrix_function_names() {
    let mut env = Env::default();
    run(&mut env, "identity(p) := p + 1", Mode::Code);
    assert_eq!(result(&mut env, "identity(2) =", Mode::Code), "\"3\"");
    run(&mut env, "T := 300", Mode::Code);
    // with T defined, x^T is a power, and a matrix's ^T still transposes
    assert_eq!(result(&mut env, "2^T > 1000 =", Mode::Code), "\"1\"");
    run(&mut env, "M := mat(1, 2; 3, 4)", Mode::Code);
    run(&mut env, "Mt := M^T", Mode::Code);
    assert!(close(&numbers(&env, "Mt"), &[1.0, 3.0, 2.0, 4.0]));
}
