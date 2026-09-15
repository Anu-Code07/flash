use flash_parser::parse;
use flash_span::FileId;

#[test]
fn parse_interp() {
    let src = r#"screen H { Text("Count: ${count}") }"#;
    parse(src, FileId(0)).expect("parse failed");
}

#[test]
fn parse_button_handler() {
    let src = r#"screen H { Button("Go") { count++ } }"#;
    parse(src, FileId(0)).expect("parse failed");
}

#[test]
fn parse_column() {
    let src = r#"screen H { Column { Text("a") Button("b") { x++ } } }"#;
    parse(src, FileId(0)).expect("parse failed");
}
