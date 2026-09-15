use flash_lexer::{tokenize, TokenKind};
use flash_span::{FileId, Interner};

#[test]
fn counter_interpolation_tokens() {
    let src = r#"Text("Count: ${count}")"#;
    let mut interner = Interner::new();
    let tokens = tokenize(src, FileId(0), &mut interner);
    println!("token count: {}", tokens.len());
    for t in &tokens {
        println!("{:?}", t.kind);
    }
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::InterpStart)));
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::StrEnd)));
    assert!(tokens.len() < 50, "too many tokens — possible infinite lex");
}
