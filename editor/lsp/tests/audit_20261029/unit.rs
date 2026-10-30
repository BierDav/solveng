use super::*;
fn server()->Server{Server{open:HashMap::new(),root:None,main:None,report:Report::default()}}
#[test]fn e08_localhost_file_uri_is_absolute(){assert_eq!(path_of("file://localhost/tmp/solveng-audit.typ"),Some(PathBuf::from("/tmp/solveng-audit.typ")));}
#[test]fn e09_equivalent_file_uris_share_unsaved_buffer(){let mut s=server();s.open.insert("file:///tmp/solveng-audit%2Etyp".into(),"unsaved".into());assert_eq!(s.text_of(Path::new("/tmp/solveng-audit.typ")),Some("unsaved".into()));}
#[test]fn e10_multiline_comment_tokens_cover_each_line(){let mut s=server();let uri="file:///tmp/solveng-comment.typ";s.open.insert(uri.into(),"#show: math\n$x:=1 /* first\nsecond */ +2$".into());let v=s.request(Request::new(1.into(),"textDocument/semanticTokens/full".into(),json!({"textDocument":{"uri":uri}})));let data=v["data"].as_array().unwrap();let mut line=0;let mut found=false;for token in data.chunks(5){line+=token[0].as_u64().unwrap();if line==2&&token[3]==2{found=true;}}assert!(found,"{v}");}

#[test] fn equivalent_open_notifications_replace_one_buffer() {
    let mut s=server();
    for (uri,text) in [("file://localhost/tmp/solveng-alias%2Etyp","old"),("file:///tmp/solveng-alias.typ","new")] {
        s.notification(Notification::new("textDocument/didOpen".into(),json!({"textDocument":{"uri":uri,"text":text}})));
    }
    assert_eq!(s.open.len(),1);
    assert_eq!(s.text_of(Path::new("/tmp/solveng-alias.typ")),Some("new".into()));
}
