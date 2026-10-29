mod support;
use solveng_core::{env::Env,linalg::Array,expr::simp};
fn array(e:&Env,n:&str)->Array{Array::of(&simp(&support::bind(e,&e.vars[n].expr))).unwrap()}
#[test]fn b01_ranges_preserve_strict_endpoint(){let mut e=Env::default();support::run(&mut e,"r := range(0,1/10000000000)");assert_eq!(array(&e,"r").a.len(),1);support::run(&mut e,"r := range(0,10000000001/10000000000)");assert_eq!(array(&e,"r").a.len(),2);}
#[test]fn b02_ranges_scale_finite_span(){let mut e=Env::default();support::run(&mut e,"r := range(-10^308,10^308,step:10^308)");assert_eq!(array(&e,"r").a.len(),2);support::run(&mut e,"r := range(-10^308,11*10^307,step:10^308)");let a=array(&e,"r");assert_eq!(a.a.len(),3);assert!(solveng_core::eval::value(&a.a[2]).unwrap().v.f().is_finite());}
#[test]fn b03_single_column_index_update_preserves_other_cells(){let mut e=Env::default();support::run(&mut e,"A := mat(1;2)");support::run(&mut e,"A[0,0] := 9");let a=array(&e,"A");assert_eq!(a.rows,2);assert_eq!(a.a[1],solveng_core::expr::int(2));}
#[test]fn b04_nested_matrix_zero_power_preserves_shape_and_domain(){let mut e=Env::default();support::run(&mut e,"A := (mat(1,2;3,4)^2)^0");let a=array(&e,"A");assert_eq!((a.rows,a.cols),(2,2));let o=support::line(&mut e,"(mat(1,2;2,4)^(-1))^0 =>");assert!(!o.errors.is_empty());}
#[test]fn b05_rank_rejects_undefined_entries(){let o=support::line(&mut Env::default(),"rank(mat(1/0,0;0,1)) =>");assert!(!o.errors.is_empty(),"{o:?}");}
#[test]fn b06_index_assignment_bounds_dense_storage(){let o=support::line(&mut Env::default(),"A[1000,1000] := 1");assert!(!o.errors.is_empty(),"dense matrix exceeded one-million element resource budget");}
