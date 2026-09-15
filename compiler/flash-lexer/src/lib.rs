//! Lexer for the Flash `.ui` language with string interpolation support.

use flash_span::{FileId, Span, Symbol, Interner};

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Int(i64),
    Float(f64),
    Bool(bool),
    StrStart,
    StrText(Symbol),
    InterpStart,
    InterpEnd,
    StrEnd,
    Ident(Symbol),
    Screen,
    Component,
    State,
    Use,
    If,
    Else,
    Match,
    In,
    Await,
    OnLoad,
    OnAppear,
    OnDispose,
    Platform,
    LBrace,
    RBrace,
    LParen,
    RParen,
    LAngle,
    RAngle,
    Comma,
    Colon,
    ColonColon,
    Dot,
    Arrow,
    Question,
    Underscore,
    Eq,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PlusPlus,
    MinusMinus,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    EqEq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    AndAnd,
    OrOr,
    Eof,
    Error(LexError),
}

#[derive(Clone, Debug, PartialEq)]
pub enum LexError {
    UnexpectedChar(char),
    UnterminatedString,
    UnterminatedInterpolation,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub fn tokenize(source: &str, file: FileId, interner: &mut Interner) -> Vec<Token> {
    let mut tokens = Vec::new();
    let bytes = source.as_bytes();
    let mut pos = 0usize;

    fn emit(tokens: &mut Vec<Token>, file: FileId, lo: usize, hi: usize, kind: TokenKind) {
        tokens.push(Token {
            kind,
            span: Span::new(file, lo as u32, hi as u32),
        });
    }

    fn skip_ws(bytes: &[u8], pos: &mut usize) {
        while *pos < bytes.len() {
            if bytes[*pos..].starts_with(b"//") {
                while *pos < bytes.len() && bytes[*pos] != b'\n' {
                    *pos += 1;
                }
            } else if bytes[*pos].is_ascii_whitespace() {
                *pos += 1;
            } else {
                break;
            }
        }
    }

    fn keyword_or_ident(s: &str, interner: &mut Interner) -> TokenKind {
        match s {
            "screen" => TokenKind::Screen,
            "component" => TokenKind::Component,
            "state" => TokenKind::State,
            "use" => TokenKind::Use,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "match" => TokenKind::Match,
            "in" => TokenKind::In,
            "await" => TokenKind::Await,
            "onLoad" => TokenKind::OnLoad,
            "onAppear" => TokenKind::OnAppear,
            "onDispose" => TokenKind::OnDispose,
            "platform" => TokenKind::Platform,
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            _ => TokenKind::Ident(interner.intern(s)),
        }
    }

    fn lex_string(
        source: &str,
        bytes: &[u8],
        pos: &mut usize,
        file: FileId,
        tokens: &mut Vec<Token>,
        interner: &mut Interner,
    ) {
        let quote_pos = *pos - 1;
        emit(tokens, file, quote_pos, *pos, TokenKind::StrStart);
        let mut str_start = *pos;

        while *pos < bytes.len() {
            if bytes[*pos] == b'"' {
                if *pos > str_start {
                    let text = interner.intern(&source[str_start..*pos]);
                    emit(tokens, file, str_start, *pos, TokenKind::StrText(text));
                }
                *pos += 1;
                emit(tokens, file, *pos - 1, *pos, TokenKind::StrEnd);
                return;
            }
            if bytes[*pos..].starts_with(b"${") {
                if *pos > str_start {
                    let text = interner.intern(&source[str_start..*pos]);
                    emit(tokens, file, str_start, *pos, TokenKind::StrText(text));
                }
                emit(tokens, file, *pos, *pos + 2, TokenKind::InterpStart);
                *pos += 2;
                lex_interp(source, bytes, pos, file, tokens, interner);
                str_start = *pos;
                continue;
            }
            if bytes[*pos] == b'\\' && *pos + 1 < bytes.len() {
                *pos += 2;
                continue;
            }
            *pos += 1;
        }
        emit(
            tokens,
            file,
            str_start,
            *pos,
            TokenKind::Error(LexError::UnterminatedString),
        );
    }

    fn lex_interp(
        source: &str,
        bytes: &[u8],
        pos: &mut usize,
        file: FileId,
        tokens: &mut Vec<Token>,
        interner: &mut Interner,
    ) {
        let mut depth = 0i32;
        while *pos < bytes.len() {
            skip_ws(bytes, pos);
            if *pos >= bytes.len() {
                break;
            }
            if bytes[*pos] == b'}' && depth == 0 {
                emit(tokens, file, *pos, *pos + 1, TokenKind::InterpEnd);
                *pos += 1;
                return;
            }
            if bytes[*pos] == b'{' {
                depth += 1;
            }
            if bytes[*pos] == b'}' {
                depth -= 1;
            }
            lex_single(source, bytes, pos, file, tokens, interner);
        }
    }

    fn lex_single(
        source: &str,
        bytes: &[u8],
        pos: &mut usize,
        file: FileId,
        tokens: &mut Vec<Token>,
        interner: &mut Interner,
    ) {
        let lo = *pos;
        if *pos >= bytes.len() {
            return;
        }

        macro_rules! two_byte {
            ($b:expr, $k:expr) => {
                if bytes[*pos..].starts_with($b) {
                    *pos += $b.len();
                    emit(tokens, file, lo, *pos, $k);
                    return;
                }
            };
        }

        match bytes[*pos] {
            b'{' => { *pos += 1; emit(tokens, file, lo, *pos, TokenKind::LBrace); }
            b'}' => { *pos += 1; emit(tokens, file, lo, *pos, TokenKind::RBrace); }
            b'(' => { *pos += 1; emit(tokens, file, lo, *pos, TokenKind::LParen); }
            b')' => { *pos += 1; emit(tokens, file, lo, *pos, TokenKind::RParen); }
            b',' => { *pos += 1; emit(tokens, file, lo, *pos, TokenKind::Comma); }
            b'.' => { *pos += 1; emit(tokens, file, lo, *pos, TokenKind::Dot); }
            b'"' => {
                *pos += 1;
                lex_string(source, bytes, pos, file, tokens, interner);
            }
            b':' => {
                if bytes[*pos..].starts_with(b"::") {
                    *pos += 2;
                    emit(tokens, file, lo, *pos, TokenKind::ColonColon);
                } else {
                    *pos += 1;
                    emit(tokens, file, lo, *pos, TokenKind::Colon);
                }
            }
            b'+' => {
                two_byte!(b"++", TokenKind::PlusPlus);
                two_byte!(b"+=", TokenKind::PlusEq);
                *pos += 1; emit(tokens, file, lo, *pos, TokenKind::Plus);
            }
            b'-' => {
                two_byte!(b"--", TokenKind::MinusMinus);
                two_byte!(b"-=", TokenKind::MinusEq);
                *pos += 1; emit(tokens, file, lo, *pos, TokenKind::Minus);
            }
            b'*' => {
                two_byte!(b"*=", TokenKind::StarEq);
                *pos += 1; emit(tokens, file, lo, *pos, TokenKind::Star);
            }
            b'/' => {
                two_byte!(b"/=", TokenKind::SlashEq);
                *pos += 1; emit(tokens, file, lo, *pos, TokenKind::Slash);
            }
            b'%' => { *pos += 1; emit(tokens, file, lo, *pos, TokenKind::Percent); }
            b'=' => {
                two_byte!(b"==", TokenKind::EqEq);
                *pos += 1; emit(tokens, file, lo, *pos, TokenKind::Eq);
            }
            b'!' => {
                two_byte!(b"!=", TokenKind::NotEq);
                *pos += 1; emit(tokens, file, lo, *pos, TokenKind::Bang);
            }
            b'<' => {
                two_byte!(b"<=", TokenKind::LtEq);
                *pos += 1; emit(tokens, file, lo, *pos, TokenKind::LAngle);
            }
            b'>' => {
                two_byte!(b">=", TokenKind::GtEq);
                *pos += 1; emit(tokens, file, lo, *pos, TokenKind::RAngle);
            }
            b'&' => {
                two_byte!(b"&&", TokenKind::AndAnd);
                emit(tokens, file, lo, lo + 1, TokenKind::Error(LexError::UnexpectedChar('&')));
                *pos += 1;
            }
            b'|' => {
                two_byte!(b"||", TokenKind::OrOr);
                emit(tokens, file, lo, lo + 1, TokenKind::Error(LexError::UnexpectedChar('|')));
                *pos += 1;
            }
            b'0'..=b'9' => {
                while *pos < bytes.len() && bytes[*pos].is_ascii_digit() {
                    *pos += 1;
                }
                if *pos < bytes.len() && bytes[*pos] == b'.' && bytes.get(*pos + 1).map(|b| b.is_ascii_digit()).unwrap_or(false) {
                    *pos += 1;
                    while *pos < bytes.len() && bytes[*pos].is_ascii_digit() {
                        *pos += 1;
                    }
                    let v: f64 = source[lo..*pos].parse().unwrap_or(0.0);
                    emit(tokens, file, lo, *pos, TokenKind::Float(v));
                } else {
                    let v: i64 = source[lo..*pos].parse().unwrap_or(0);
                    emit(tokens, file, lo, *pos, TokenKind::Int(v));
                }
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                while *pos < bytes.len() && (bytes[*pos].is_ascii_alphanumeric() || bytes[*pos] == b'_') {
                    *pos += 1;
                }
                let kind = keyword_or_ident(&source[lo..*pos], interner);
                emit(tokens, file, lo, *pos, kind);
            }
            ch => {
                *pos += 1;
                emit(tokens, file, lo, *pos, TokenKind::Error(LexError::UnexpectedChar(ch as char)));
            }
        }
    }

    while pos < bytes.len() {
        skip_ws(bytes, &mut pos);
        if pos >= bytes.len() {
            break;
        }
        lex_single(source, bytes, &mut pos, file, &mut tokens, interner);
    }

    emit(&mut tokens, file, pos, pos, TokenKind::Eof);
    tokens
}
