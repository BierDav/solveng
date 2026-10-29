mod support;
use solveng_core::{env::Env,graph::{self,GraphReq,GraphOut,XV,Mark}};
fn req(e:&Env,cs:&[&str],a:f64,b:f64)->GraphReq{GraphReq{axes:Default::default(),cfg:support::cfg(),curves:cs.iter().map(|s|support::curve(s,e)).collect(),xrange:vec![XV::Num(a),XV::Num(b)],yrange:vec![],marks:vec![],ymark:vec![],samples:10}}
fn plot(e:&Env,cs:&[&str],a:f64,b:f64)->GraphOut{graph::graph(e,&req(e,cs,a,b)).unwrap()}
fn data()->Env{let mut e=Env::default();support::run(&mut e,"y := vec(-10^308,10^308)");support::run(&mut e,"t := vec(-10^308,10^308)");e}
#[test]fn p01_dimension_addition_overflow_is_diagnostic(){let o=support::line(&mut Env::default(),"1 m^17000000 * 1 m^17000000 * 1 m^17000000 =>");assert!(!o.errors.is_empty(),"{o:?}");}
#[test]fn p02_atan2_handles_minimum_integer(){support::run(&mut Env::default(),"atan2((-2)^127,1) =>");}
#[test]fn p03_empty_vector_transpose_does_not_panic(){let _=support::line(&mut Env::default(),"vec()^T =>");}
#[test]fn p04_rank_checks_dimension_consistency(){let o=support::line(&mut Env::default(),"rank(mat(1 m,1 s;1 s,1 m)) =>");assert!(!o.errors.is_empty(),"{o:?}");}
#[test]fn p05_wide_finite_graph_samples(){let g=plot(&Env::default(),&["x/10^308"],-1e308,1e308);assert!(g.series.iter().flat_map(|s|&s.points).count()>5);}
#[test]fn p06_small_nonconstant_graph_has_useful_scale(){let g=plot(&Env::default(),&["x*10^(-20)"],0.,1.);assert!(g.y1-g.y0<1e-18,"{:?}",(g.y0,g.y1));}
#[test]fn p07_every_graph_curve_checks_dimensions(){assert!(graph::graph(&Env::default(),&req(&Env::default(),&["x","x+1 m"],0.,1.)).is_err());}
#[test]fn p08_graph_rejects_nonfinite_bounds(){assert!(graph::graph(&Env::default(),&req(&Env::default(),&["x"],f64::NEG_INFINITY,f64::INFINITY)).is_err());}
#[test]fn p09_data_clipping_stays_in_range(){let e=data();let g=plot(&e,&["y(t)"],-1e307,1e307);let ps:Vec<_>=g.series.iter().flat_map(|s|&s.points).collect();assert!(ps.len()>=2);assert!(ps.iter().all(|p|p.0>=g.x0&&p.0<=g.x1&&p.1.is_finite()),"{ps:?}");}
#[test]fn p10_data_marks_interpolate_finitely(){let e=data();let mut r=req(&e,&["y(t)"],-1e307,1e307);r.marks=vec![(Mark::Val(XV::Num(0.)),true)];let g=graph::graph(&e,&r).unwrap();assert!(g.points.iter().any(|p|p.x==0.&&p.y==0.),"{:?}",g.points);let mut r=req(&e,&["y(t)"],-1e308,1e308);r.yrange=vec![XV::Num(-1e308),XV::Num(1e308)];r.marks=vec![(Mark::Key("roots".into()),true)];let g=graph::graph(&e,&r).unwrap();assert!(g.points.iter().any(|p|p.x==0.&&p.y==0.),"{:?}",g.points);}

#[test]
fn representative_boundary_variations() {
    // Empty arrays receive a diagnostic; large exponent powers cannot wrap.
    let mut e = Env::default();
    assert!(!support::line(&mut e, "vec()^T =>").errors.is_empty());
    assert!(!support::line(&mut e, "(1 m^17000000)^100000000000000000000 =>").errors.is_empty());
    assert!(!support::line(&mut e, "rank(mat(1/0,0;0,1)) =>").errors.is_empty());
    support::run(&mut e, "atan2(1,(-2)^127) =>");
    // Valid heterogeneous dimensions factor by rows and columns and retain rank.
    support::run(&mut e, "rank(mat(1 m,1 m;1 s,2 s)) =>");
    let g = plot(&e, &["-x/10^308"], -1e308, 1e308);
    assert!(g.series.iter().flat_map(|s| &s.points).all(|&(x,y)| x.is_finite() && y.is_finite()));
    for sources in [["x", "x+1 m"], ["x+1 m", "x"]] {
        assert!(graph::graph(&e, &req(&e, &sources, 0., 1.)).is_err());
    }
    let mut r = req(&e, &["x"], 0., 1.);
    r.ymark = vec![XV::Num(f64::INFINITY)];
    assert!(graph::graph(&e, &r).is_err());
    let g = plot(&e, &["-x*10^(-20)"], 0., 1.);
    assert!(g.y1 - g.y0 < 1e-18);
}
