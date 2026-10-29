mod support;
use solveng_core::{api::{self,CompileReq,Arg,ClearReq},env::Env,units::Units,bindings::{self,BindingReq}};
fn unit(s:&str)->String{support::src(&format!("1 {s}"),&Env::default())}
fn request(s:&str)->CompileReq{CompileReq{cfg:support::cfg(),src:support::src(s,&Env::default()),params:vec!["x".into()],units:Default::default(),unit:None}}
fn typed()->api::Compiled{let mut r=request("x");r.units.insert("x".into(),unit("m"));api::compile(&Env::default(),&r).unwrap()}
fn arg(s:&str)->Arg{Arg::Text(support::src(s,&Env::default()))}
#[test]fn a01_automatic_parameter_preserves_quantity_dimension(){let r=request("x+1 m");let c=api::compile(&Env::default(),&r).unwrap();assert!(api::compiled_eval(&c,&[arg("2 s")]).is_err());}
#[test]fn a02_negative_quantity_argument(){assert_eq!(api::compiled_eval(&typed(),&[arg("-2 m")]).unwrap(),Some(-2.));}
#[test]fn a03_scientific_quantity_argument(){assert_eq!(api::compiled_eval(&typed(),&[arg("1e3 m")]).unwrap(),Some(1000.));}
#[test]fn a04_long_decimal_quantity_argument(){let v=api::compiled_eval(&typed(),&[arg("1.234567890123456789012345678901234 m")]).unwrap().unwrap();assert!((v-1.2345678901234567).abs()<1e-14);}
#[test]fn a05_unit_product_after_division(){let u=Units::builtin_table();assert_eq!(u.parse("m/s*m",false).unwrap().d,u.parse("m^2/s",false).unwrap().d);}
#[test]fn a06_clear_rejects_noncall_expression(){let mut e=Env::default();support::run(&mut e,"x := 2");let errors=api::clear(&mut e,&ClearReq{names:vec!["x(y)+z(t)".into()]});assert!(!errors.is_empty());assert!(e.vars.contains_key("x"));}
#[test]fn a07_compiled_scaled_result_is_finite(){let b=bindings::construct(BindingReq{kind:"unit".into(),src:support::src("10^(-300)*#m",&Env::default()),identity:None,presentation:"tiny".into(),description:"tiny".into(),auto:false}).unwrap();let mut r=request("x");r.units.insert("x".into(),unit("m"));r.unit=Some(format!("⟦{}⟧",b.encode().unwrap()));let c=api::compile(&Env::default(),&r).unwrap();let v=api::compiled_eval(&c,&[Arg::Num(1e300)]).unwrap();assert!(v.is_none()||v.is_some_and(f64::is_finite),"{v:?}");}
#[test]fn a08_diagnostic_does_not_leak_binding_serialization(){let e=api::compiled_eval(&typed(),&[arg("2 s")]).unwrap_err();assert!(!e.contains('⟦'),"{e}");assert!(e.contains('s'));}
#[test]fn a09_compile_rejects_unresolved_function(){let mut r=request("missing(2)");r.params.clear();assert!(api::compile(&Env::default(),&r).is_err());}
#[test]fn a10_unit_parser_rejects_missing_operands(){let u=Units::builtin_table();for s in ["m/","*m","m**s","m//s"]{assert!(u.parse(s,false).is_err(),"accepted {s}");}}

#[test]fn quantity_literals_keep_sign_exponents_and_precision(){
 for (literal, expected) in [("-2 m",-2.0),("1e+3 m",1000.0),("-1.5e-2 m",-0.015),("1.234567890123456789012345678901234 m",1.2345678901234567)]{
  let quantity=Arg::Text(format!("{} {}",literal.strip_suffix(" m").unwrap(),support::unit("m")));
  let actual=api::compiled_eval(&typed(),&[quantity]).unwrap().unwrap();
  assert!((actual-expected).abs()<expected.abs()*1e-14,"{literal}: {actual}");
 }
 let u=Units::builtin_table();
 assert_eq!(u.parse("J/kg K",false).unwrap().d,u.parse("J/(kg K)",false).unwrap().d);
 assert_eq!(u.parse("m/s*m/s",false).unwrap().d,u.parse("m^2/s^2",false).unwrap().d);
 for bad in ["m/","m*","m//s","m/*s","(m/)","()"]{assert!(u.parse(bad,false).is_err(),"{bad}");}
}
#[test]fn automatic_quantity_dimensions_and_clear_call_variants(){
 let c=api::compile(&Env::default(),&request("x+1 m")).unwrap();
 assert_eq!(api::compiled_eval(&c,&[arg("2 m")]).unwrap(),Some(3.0));
 let mut e=Env::default();support::run(&mut e,"f(x) := x");
 assert!(api::clear(&mut e,&ClearReq{names:vec!["f(x)".into()]}).is_empty());
 support::run(&mut e,"x := 2");
 for bad in ["x(y)+z(t)","x(a)(b)","x((a)"]{assert!(!api::clear(&mut e,&ClearReq{names:vec![bad.into()]}).is_empty());assert!(e.vars.contains_key("x"));}
}

#[test]fn automatic_parameter_checks_requested_output_dimension(){
 let mut r=request("x");r.unit=Some(unit("m"));
 let c=api::compile(&Env::default(),&r).unwrap();
 assert!(api::compiled_eval(&c,&[arg("2 s")]).is_err());
 assert_eq!(api::compiled_eval(&c,&[arg("2 m")]).unwrap(),Some(2.0));
}

#[test]fn unit_expression_power_overflow_is_diagnostic(){let u=Units::builtin_table();assert!(u.parse("(m^50000)^50000",false).is_err());}
