use flash_parser::parse;
use flash_span::FileId;

#[test]
fn parse_minimal_provider() {
    let src = "@provider C { @state x: Int = 0 }";
    let result = parse(src, FileId(0));
    assert!(result.is_ok(), "parse failed: {:?}", result.err());
    assert_eq!(result.unwrap().ast.items.len(), 1);
}
