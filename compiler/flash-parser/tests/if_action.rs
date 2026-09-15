use flash_parser::parse;
use flash_span::FileId;

#[test]
fn parse_if_in_action() {
    let src = r#"
@provider C {
    @action inc() {
        if count > 0 {
            count++
        }
    }
}
"#;
    let result = parse(src, FileId(0));
    assert!(result.is_ok(), "{:?}", result.err());
}
