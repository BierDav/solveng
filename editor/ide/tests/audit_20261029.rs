use solveng_ide::*;
fn off(s:&str,n:&str)->u32{s[..s.find(n).unwrap()].encode_utf16().count() as u32}
fn spelling(s:&str,t:&Token)->String{String::from_utf16(&s.encode_utf16().skip(t.start as usize).take(t.length as usize).collect::<Vec<_>>()).unwrap()}
#[test]fn e01_argument_commas_are_not_number_tokens(){let s="#show: math\n$x:=sum(k,k,0,10)$";let ts=document_tokens(s,&Context::default());assert!(!ts.iter().any(|t|t.kind==TokenKind::NUMBER&&spelling(s,t).contains(',')),"{ts:?}");}
#[test]fn e02_sum_local_binding_ignores_comments(){let s="#show: math\n$k:=10$\n$x:=sum(k,/* c */k,0,10)$";let at=off(s,"k,/*");let d=definition(s,&Context::default(),at);assert!(d.is_none()||d.unwrap().start>off(s,"k:=10"));}
#[test]fn e03_calculus_binding_shadows_outer_names(){for (op,body,needle) in [("diff","diff(k^2,k)","k^2"),("integrate","integrate(k,k,0,10)","k,k"),("taylor","taylor(k^2,k,0,2)","k^2"),("jacobian","jacobian(vec(k^2),vec(k))","k^2")]{let s=format!("#import operations: {op}\n#show: math\n$k:=10$\n$x:={body}$");let at=off(&s,needle);let d=definition(&s,&Context::default(),at);assert!(d.is_none()||d.unwrap().start>off(&s,"k:=10"),"{op}");}}
#[test]fn e04_solve_does_not_define_global(){let s="#show: math\n$x^2=4 -> solve(x)$\n$x =>$";assert!(definition(s,&Context::default(),off(s,"x =>")).is_none());}
#[test]fn e05_comments_do_not_become_hover_result(){let s="#show: math\n$x := 1 /* = 2 */$\n$x =>$";let h=hover(s,&Context::default(),off(s,"x =>")).unwrap();assert!(!h.markdown.contains("2 */"),"{h:?}");assert!(h.markdown.contains("1"));}
#[test]fn e06_declarations_can_follow_comments(){for body in ["/* c */ x := 1","x /* c */ := 1"]{let s=format!("#show: math\n${body}$\n$x =>$ ");assert!(definition(&s,&Context::default(),off(&s,"x =>")).is_some(),"{body}");}}
#[test]fn e07_clear_decodes_unicode_escapes(){let s="#show: math\n$x := 1$\n#clear(\"\\u{78}\")\n$x =>$";assert!(definition(s,&Context::default(),off(s,"x =>")).is_none());}

#[test] fn comments_preserve_nested_binder_offsets_and_bound_scope() {
    let s="#show: math\n$k:=10$\n$x:=sum(k,/* outer /*inner*/ */k,0,k)$";
    let local=definition(s,&Context::default(),off(s,"k,/*")).unwrap();
    let bound=definition(s,&Context::default(),off(s,"k)$")).unwrap();
    assert!(local.start>off(s,"k:=10"));
    assert_eq!(bound.start,off(s,"k:=10"));
}
#[test] fn vector_calculus_variables_have_independent_local_binders() {
    let s="#import operations: jacobian\n#show: math\n$x:=10$\n$y:=20$\n$J:=jacobian(vec(x*y),vec(x,y))$";
    for needle in ["x*y","y),vec"] {
        let d=definition(s,&Context::default(),off(s,needle)).unwrap();
        assert!(d.start>off(s,"J:=jacobian"));
    }
}
