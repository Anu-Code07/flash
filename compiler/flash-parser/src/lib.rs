//! Hand-written recursive descent parser for Flash `.ui` files.

use flash_ast::*;
use flash_lexer::{tokenize, Token, TokenKind};
use flash_span::{FileId, Interner, Span, Symbol};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    ast: Ast,
    interner: Interner,
    file: FileId,
}

pub struct ParseResult {
    pub ast: Ast,
    pub interner: Interner,
}

pub fn parse(source: &str, file: FileId) -> Result<ParseResult, Vec<ParseError>> {
    let mut interner = Interner::new();
    let tokens = tokenize(source, file, &mut interner);
    let mut parser = Parser {
        tokens,
        pos: 0,
        ast: Ast::new(),
        interner,
        file,
    };
    parser.parse_program()?;
    Ok(ParseResult {
        ast: parser.ast,
        interner: parser.interner,
    })
}

#[derive(Clone, Debug)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl Parser {
    fn parse_program(&mut self) -> Result<(), Vec<ParseError>> {
        let mut errors = Vec::new();
        while !self.check(&TokenKind::Eof) {
            match self.parse_item() {
                Ok(item) => self.ast.items.push(item),
                Err(e) => {
                    errors.push(e);
                    self.synchronize();
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        if self.check(&TokenKind::Screen) {
            return Ok(Item::Screen(self.parse_screen()?));
        }
        if self.check(&TokenKind::Component) {
            return Ok(Item::Component(self.parse_component()?));
        }
        if self.check(&TokenKind::Use) {
            return Ok(Item::Import(self.parse_import()?));
        }
        Err(self.error("expected screen, component, or use"))
    }

    fn parse_screen(&mut self) -> Result<ScreenDef, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Screen)?;
        let name = self.expect_ident()?;
        let params = if self.check(&TokenKind::LParen) {
            self.parse_params()?
        } else {
            Vec::new()
        };
        self.expect(&TokenKind::LBrace)?;

        let mut state = Vec::new();
        let mut lifecycle = Vec::new();
        let mut body = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            if self.check(&TokenKind::State) {
                state.push(self.parse_state()?);
            } else if self.check(&TokenKind::OnLoad) || self.check(&TokenKind::OnAppear) || self.check(&TokenKind::OnDispose) {
                lifecycle.push(self.parse_lifecycle()?);
            } else {
                body.push(self.parse_node()?);
            }
        }
        self.expect(&TokenKind::RBrace)?;
        let end = self.current_span();

        Ok(ScreenDef {
            name,
            params,
            state,
            lifecycle,
            body,
            span: start.merge(end),
        })
    }

    fn parse_component(&mut self) -> Result<ComponentDef, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Component)?;
        let name = self.expect_ident()?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::LBrace)?;
        let mut body = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            body.push(self.parse_node()?);
        }
        self.expect(&TokenKind::RBrace)?;
        let end = self.current_span();
        Ok(ComponentDef { name, params, body, span: start.merge(end) })
    }

    fn parse_import(&mut self) -> Result<ImportDef, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Use)?;
        let mut path = vec![self.expect_ident()?];
        while self.check(&TokenKind::ColonColon) {
            self.advance();
            path.push(self.expect_ident()?);
        }
        let items = if self.check(&TokenKind::ColonColon) {
            self.advance();
            self.expect(&TokenKind::LBrace)?;
            let mut items = Vec::new();
            while !self.check(&TokenKind::RBrace) {
                items.push(self.expect_ident()?);
                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }
            self.expect(&TokenKind::RBrace)?;
            items
        } else {
            Vec::new()
        };
        Ok(ImportDef { path, items, span: start.merge(self.current_span()) })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        self.expect(&TokenKind::LParen)?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                params.push(self.parse_param()?);
                if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RParen)?;
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let start = self.current_span();
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        let default = if self.check(&TokenKind::Eq) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok(Param { name, ty, default, span: start.merge(self.current_span()) })
    }

    fn parse_state(&mut self) -> Result<StateDef, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::State)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        self.expect(&TokenKind::Eq)?;
        let init = self.parse_expr()?;
        Ok(StateDef { name, ty, init, span: start.merge(self.current_span()) })
    }

    fn parse_lifecycle(&mut self) -> Result<LifecycleDef, ParseError> {
        let start = self.current_span();
        let kind = if self.check(&TokenKind::OnLoad) {
            self.advance();
            LifecycleKind::OnLoad
        } else if self.check(&TokenKind::OnAppear) {
            self.advance();
            LifecycleKind::OnAppear
        } else {
            self.advance();
            LifecycleKind::OnDispose
        };
        let handler = self.parse_handler()?;
        Ok(LifecycleDef { kind, handler, span: start.merge(self.current_span()) })
    }

    fn parse_node(&mut self) -> Result<NodeId, ParseError> {
        let start = self.current_span();

        if self.check(&TokenKind::If) {
            return self.parse_if_node();
        }
        if self.is_ident("List") {
            return self.parse_list_node();
        }

        let kind = self.expect_ident()?;
        let args = if self.check(&TokenKind::LParen) {
            self.parse_args()?
        } else {
            Vec::new()
        };

        let mut modifiers = Vec::new();
        while self.check(&TokenKind::Dot) {
            modifiers.push(self.parse_modifier()?);
        }

        let (children, handler) = if self.check(&TokenKind::LBrace) {
            // Disambiguate: Button/TextField → handler, containers → children
            let is_handler = matches!(
                self.interner.get(kind),
                "Button" | "TextField"
            );
            if is_handler {
                let h = self.parse_handler()?;
                (Vec::new(), Some(h))
            } else {
                (self.parse_child_block()?, None)
            }
        } else {
            (Vec::new(), None)
        };

        let span = start.merge(self.current_span());
        let node = AstNode::Element {
            kind,
            args,
            modifiers,
            children,
            handler,
            span,
        };
        Ok(self.ast.add_node(node, span))
    }

    fn parse_if_node(&mut self) -> Result<NodeId, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::If)?;
        let cond = self.parse_expr()?;
        let then_ = self.parse_child_block()?;
        let else_ = if self.check(&TokenKind::Else) {
            self.advance();
            if self.check(&TokenKind::If) {
                Some(vec![self.parse_if_node()?])
            } else {
                Some(self.parse_child_block()?)
            }
        } else {
            None
        };
        let span = start.merge(self.current_span());
        let node = AstNode::If { cond, then_, else_, span };
        Ok(self.ast.add_node(node, span))
    }

    fn parse_list_node(&mut self) -> Result<NodeId, ParseError> {
        let start = self.current_span();
        self.expect_ident()?; // "List"
        self.expect(&TokenKind::LParen)?;
        let source = self.parse_expr()?;
        self.expect(&TokenKind::RParen)?;

        let key = if self.is_ident("key") {
            self.advance();
            self.expect(&TokenKind::Colon)?;
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.expect(&TokenKind::LBrace)?;
        let binding = self.expect_ident()?;
        self.expect(&TokenKind::In)?;
        let mut body = Vec::new();
        while !self.check(&TokenKind::RBrace) {
            body.push(self.parse_node()?);
        }
        self.expect(&TokenKind::RBrace)?;

        let span = start.merge(self.current_span());
        let node = AstNode::List { source, key, binding, body, span };
        Ok(self.ast.add_node(node, span))
    }

    fn parse_child_block(&mut self) -> Result<Vec<NodeId>, ParseError> {
        self.expect(&TokenKind::LBrace)?;
        let mut nodes = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            if self.check(&TokenKind::State) {
                return Err(self.error("state declarations must appear before UI nodes"));
            }
            nodes.push(self.parse_node()?);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(nodes)
    }

    fn parse_handler(&mut self) -> Result<HandlerId, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&TokenKind::RBrace)?;
        let span = start.merge(self.current_span());
        let handler = Handler { stmts, span };
        Ok(self.ast.add_handler(handler, span))
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        if self.check(&TokenKind::If) {
            self.advance();
            let cond = self.parse_expr()?;
            self.expect(&TokenKind::LBrace)?;
            let mut then_ = Vec::new();
            while !self.check(&TokenKind::RBrace) {
                then_.push(self.parse_stmt()?);
            }
            self.expect(&TokenKind::RBrace)?;
            let else_ = if self.check(&TokenKind::Else) {
                self.advance();
                self.expect(&TokenKind::LBrace)?;
                let mut els = Vec::new();
                while !self.check(&TokenKind::RBrace) {
                    els.push(self.parse_stmt()?);
                }
                self.expect(&TokenKind::RBrace)?;
                Some(els)
            } else {
                None
            };
            return Ok(Stmt::If { cond, then_, else_, span: start.merge(self.current_span()) });
        }

        let lvalue = self.parse_lvalue()?;
        if self.check(&TokenKind::PlusPlus) {
            self.advance();
            return Ok(Stmt::IncDec { target: lvalue, op: IncDecOp::Inc, span: start.merge(self.current_span()) });
        }
        if self.check(&TokenKind::MinusMinus) {
            self.advance();
            return Ok(Stmt::IncDec { target: lvalue, op: IncDecOp::Dec, span: start.merge(self.current_span()) });
        }
        let op = self.parse_assign_op()?;
        let value = self.parse_expr()?;
        Ok(Stmt::Assign { target: lvalue, op, value, span: start.merge(self.current_span()) })
    }

    fn parse_lvalue(&mut self) -> Result<LValue, ParseError> {
        let start = self.current_span();
        let mut path = vec![self.expect_ident()?];
        while self.check(&TokenKind::Dot) {
            self.advance();
            path.push(self.expect_ident()?);
        }
        Ok(LValue { path, span: start.merge(self.current_span()) })
    }

    fn parse_assign_op(&mut self) -> Result<AssignOp, ParseError> {
        let op = match &self.current().kind {
            TokenKind::Eq => AssignOp::Set,
            TokenKind::PlusEq => AssignOp::Add,
            TokenKind::MinusEq => AssignOp::Sub,
            TokenKind::StarEq => AssignOp::Mul,
            TokenKind::SlashEq => AssignOp::Div,
            _ => return Err(self.error("expected assignment operator")),
        };
        self.advance();
        Ok(op)
    }

    fn parse_args(&mut self) -> Result<Vec<Arg>, ParseError> {
        self.expect(&TokenKind::LParen)?;
        let mut args = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                args.push(self.parse_arg()?);
                if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RParen)?;
        Ok(args)
    }

    fn parse_arg(&mut self) -> Result<Arg, ParseError> {
        let start = self.current_span();
        let name = if self.peek_name_colon() {
            let n = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            Some(n)
        } else {
            None
        };
        let value = self.parse_expr()?;
        Ok(Arg { name, value, span: start.merge(self.current_span()) })
    }

    fn parse_modifier(&mut self) -> Result<Modifier, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Dot)?;
        let name = self.expect_ident()?;
        let args = if self.check(&TokenKind::LParen) {
            self.parse_args()?
        } else {
            Vec::new()
        };
        Ok(Modifier { name, args, span: start.merge(self.current_span()) })
    }

    fn parse_type(&mut self) -> Result<TypeRef, ParseError> {
        let start = self.current_span();
        let name = self.expect_ident()?;
        let mut generics = Vec::new();
        if self.check(&TokenKind::LAngle) {
            self.advance();
            loop {
                generics.push(self.parse_type()?);
                if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(&TokenKind::RAngle)?;
        }
        let optional = if self.check(&TokenKind::Question) {
            self.advance();
            true
        } else {
            false
        };
        Ok(TypeRef { name, generics, optional, span: start.merge(self.current_span()) })
    }

    // ── Expression parsing (Pratt / precedence climbing) ──

    fn parse_expr(&mut self) -> Result<ExprId, ParseError> {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Result<ExprId, ParseError> {
        let start = self.current_span();
        let (mut lhs, mut next_bp) = self.parse_prefix()?;

        loop {
            let Some(op) = self.current_binop() else { break };
            let (l_bp, r_bp) = op.precedence();
            if l_bp < min_bp {
                break;
            }
            self.advance();
            let rhs = self.parse_expr_bp(r_bp)?;
            let span = start.merge(self.current_span());
            lhs = self.ast.add_expr(Expr::Binary { op, lhs, rhs, span }, span);
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<(ExprId, u8), ParseError> {
        let start = self.current_span();

        if self.check(&TokenKind::Await) {
            self.advance();
            let inner = self.parse_expr_bp(0)?;
            let span = start.merge(self.current_span());
            return Ok((self.ast.add_expr(Expr::Await(inner, span), span), 0));
        }

        if self.check(&TokenKind::Bang) {
            self.advance();
            let rhs = self.parse_expr_bp(8)?;
            let span = start.merge(self.current_span());
            return Ok((self.ast.add_expr(Expr::Unary { op: UnOp::Not, rhs, span }, span), 0));
        }

        if self.check(&TokenKind::Minus) {
            self.advance();
            let rhs = self.parse_expr_bp(8)?;
            let span = start.merge(self.current_span());
            return Ok((self.ast.add_expr(Expr::Unary { op: UnOp::Neg, rhs, span }, span), 0));
        }

        if self.check(&TokenKind::LParen) {
            self.advance();
            let expr = self.parse_expr()?;
            self.expect(&TokenKind::RParen)?;
            return Ok((expr, 0));
        }

        if self.check(&TokenKind::StrStart) {
            return self.parse_string_interp();
        }

        if let TokenKind::Int(v) = &self.current().kind {
            let v = *v;
            self.advance();
            let span = start.merge(self.current_span());
            return Ok((self.ast.add_expr(Expr::Int(v), span), 0));
        }

        if let TokenKind::Float(v) = &self.current().kind {
            let v = *v;
            self.advance();
            let span = start.merge(self.current_span());
            return Ok((self.ast.add_expr(Expr::Float(v), span), 0));
        }

        if let TokenKind::Bool(v) = &self.current().kind {
            let v = *v;
            self.advance();
            let span = start.merge(self.current_span());
            return Ok((self.ast.add_expr(Expr::Bool(v), span), 0));
        }

        if let TokenKind::Ident(sym) = self.current().kind {
            self.advance();
            let mut expr = self.ast.add_expr(Expr::Ident(sym), start.merge(self.current_span()));

            // Postfix: field access, calls
            loop {
                if self.check(&TokenKind::Dot) {
                    self.advance();
                    let field = self.expect_ident()?;
                    let span = start.merge(self.current_span());
                    expr = self.ast.add_expr(Expr::Field { base: expr, field, span }, span);
                } else if self.check(&TokenKind::LParen) {
                    let args = self.parse_args()?;
                    let span = start.merge(self.current_span());
                    expr = self.ast.add_expr(Expr::Call { callee: expr, args, span }, span);
                } else {
                    break;
                }
            }
            return Ok((expr, 0));
        }

        Err(self.error("expected expression"))
    }

    fn parse_string_interp(&mut self) -> Result<(ExprId, u8), ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::StrStart)?;

        let mut parts = Vec::new();

        while !self.check(&TokenKind::StrEnd) && !self.check(&TokenKind::Eof) {
            if let TokenKind::StrText(sym) = &self.current().kind {
                let sym = *sym;
                self.advance();
                parts.push(InterpPart::Text(sym));
            } else if self.check(&TokenKind::InterpStart) {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::InterpEnd)?;
                parts.push(InterpPart::Expr(expr));
            } else {
                break;
            }
        }
        self.expect(&TokenKind::StrEnd)?;
        let span = start.merge(self.current_span());
        Ok((self.ast.add_expr(Expr::Interp(parts), span), 0))
    }

    // ── Helpers ──

    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn current_span(&self) -> Span {
        self.current().span
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind)
            || match (kind, &self.current().kind) {
                (TokenKind::Ident(a), TokenKind::Ident(b)) => a == b,
                (TokenKind::Int(a), TokenKind::Int(b)) => a == b,
                (TokenKind::Bool(a), TokenKind::Bool(b)) => a == b,
                _ => false,
            }
    }

    fn advance(&mut self) {
        if !self.check(&TokenKind::Eof) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<(), ParseError> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(&format!("expected {:?}", kind)))
        }
    }

    fn expect_ident(&mut self) -> Result<Symbol, ParseError> {
        if let TokenKind::Ident(sym) = self.current().kind {
            self.advance();
            Ok(sym)
        } else {
            Err(self.error("expected identifier"))
        }
    }

    fn peek_name_colon(&self) -> bool {
        if !matches!(self.current().kind, TokenKind::Ident(_)) {
            return false;
        }
        matches!(self.tokens.get(self.pos + 1).map(|t| &t.kind), Some(TokenKind::Colon))
    }

    fn current_binop(&self) -> Option<BinOp> {
        match &self.current().kind {
            TokenKind::OrOr => Some(BinOp::Or),
            TokenKind::AndAnd => Some(BinOp::And),
            TokenKind::EqEq => Some(BinOp::Eq),
            TokenKind::NotEq => Some(BinOp::NotEq),
            TokenKind::Lt => Some(BinOp::Lt),
            TokenKind::Gt => Some(BinOp::Gt),
            TokenKind::LtEq => Some(BinOp::LtEq),
            TokenKind::GtEq => Some(BinOp::GtEq),
            TokenKind::Plus => Some(BinOp::Add),
            TokenKind::Minus => Some(BinOp::Sub),
            TokenKind::Star => Some(BinOp::Mul),
            TokenKind::Slash => Some(BinOp::Div),
            TokenKind::Percent => Some(BinOp::Mod),
            _ => None,
        }
    }

    fn synchronize(&mut self) {
        while !self.check(&TokenKind::Eof) {
            if self.check(&TokenKind::Screen)
                || self.check(&TokenKind::Component)
                || self.check(&TokenKind::State)
                || self.check(&TokenKind::RBrace)
            {
                return;
            }
            self.advance();
        }
    }

    fn is_ident(&self, name: &str) -> bool {
        if let TokenKind::Ident(sym) = &self.current().kind {
            self.interner.get(*sym) == name
        } else {
            false
        }
    }

    fn error(&self, msg: &str) -> ParseError {
        ParseError {
            message: msg.to_string(),
            span: self.current_span(),
        }
    }
}
