#![forbid(unsafe_code)]

use crate::ast::*;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParserError {
    #[error("unexpected end of input")]
    Eof,
    #[error("unexpected token: expected {expected}, found {found:?}")]
    UnexpectedToken {
        expected: &'static str,
        found: Token,
    },
    #[error("invalid number literal: {0}")]
    InvalidNumber(String),
    #[error("lexer error: {0}")]
    Lexer(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Ident(String),
    Int(i64),
    Str(String),
    Bool(bool),

    KwImport,
    KwGlobal,
    KwMut,
    KwType,
    KwIf,
    KwThen,
    KwElse,
    KwCopy,

    LBrace,
    RBrace,
    LParen,
    RParen,
    Colon,
    Comma,
    Dot,
    Assign,
    Arrow,
    Amp,
    Plus,
    Minus,
    Star,
    Slash,
    Lt,
    EqEq,
    AndAnd,
    OrOr,
    Bang,

    Eof,
}

pub struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    _src: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Result<Self, ParserError> {
        let tokens = lex(source)?;
        Ok(Self {
            tokens,
            pos: 0,
            _src: source,
        })
    }

    pub fn parse_program(&mut self) -> Result<Program, ParserError> {
        let mut decls = Vec::new();
        while !self.check(Token::Eof) {
            decls.push(self.parse_decl()?);
        }
        Ok(Program { decls })
    }

    fn parse_decl(&mut self) -> Result<Decl, ParserError> {
        if self.matches(&[Token::KwImport]) {
            let module = self.expect_ident("module name")?;
            return Ok(Decl::Import(ImportDecl { module }));
        }

        if self.matches(&[Token::KwGlobal]) {
            let binding = self.parse_binding()?;
            return Ok(Decl::Global(binding));
        }

        if self.matches(&[Token::KwType]) {
            let name = self.expect_ident("type name")?;
            self.expect(&Token::Assign, "'=' after type name")?;
            let ty = self.parse_type()?;
            return Ok(Decl::Type(TypeDecl { name, ty }));
        }

        // function vs let binding: lookahead for '('
        if self.peek_is_ident() && self.peek_next_is(Token::LParen) {
            let name = self.expect_ident("function name")?;
            self.expect(&Token::LParen, "'(' after function name")?;
            let params = if self.check(Token::RParen) {
                Vec::new()
            } else {
                self.parse_params()?
            };
            self.expect(&Token::RParen, "')' after params")?;
            let ret = if self.matches(&[Token::Arrow]) {
                Some(self.parse_type()?)
            } else {
                None
            };
            self.expect(&Token::Assign, "'=' before function body")?;
            let body = self.parse_expr()?;
            return Ok(Decl::Func(FuncDecl {
                name,
                params,
                ret,
                body,
            }));
        }

        let binding = self.parse_binding()?;
        Ok(Decl::Let(binding))
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParserError> {
        let mut params = Vec::new();
        loop {
            let mutable = self.matches(&[Token::KwMut]);
            let name = self.expect_ident("parameter name")?;
            self.expect(&Token::Colon, "':' after parameter name")?;
            let ty = self.parse_type()?;
            params.push(Param { mutable, name, ty });
            if !self.matches(&[Token::Comma]) {
                break;
            }
        }
        Ok(params)
    }

    fn parse_binding(&mut self) -> Result<Binding, ParserError> {
        let mutable = self.matches(&[Token::KwMut]);
        let name = self.expect_ident("binding name")?;
        self.expect(&Token::Colon, "':' after binding name")?;
        let ty = self.parse_type()?;
        self.expect(&Token::Assign, "'=' after binding type")?;
        let value = self.parse_expr()?;
        Ok(Binding {
            mutable,
            name,
            ty,
            value,
        })
    }

    fn parse_type(&mut self) -> Result<Type, ParserError> {
        if self.matches(&[Token::Amp]) {
            let inner = self.parse_type()?;
            return Ok(Type::Ref(Box::new(inner)));
        }

        if self.matches(&[Token::LBrace]) {
            let mut fields = Vec::new();
            if !self.matches(&[Token::RBrace]) {
                loop {
                    let name = self.expect_ident("field name")?;
                    self.expect(&Token::Colon, "':' after field name")?;
                    let ty = self.parse_type()?;
                    fields.push(FieldType { name, ty });
                    if self.matches(&[Token::Comma]) {
                        continue;
                    }
                    self.expect(&Token::RBrace, "'}' to close record type")?;
                    break;
                }
            }
            return Ok(Type::Record(fields));
        }

        let name = self.expect_ident("type name")?;
        Ok(Type::Named(name))
    }

    fn parse_block(&mut self) -> Result<Block, ParserError> {
        self.expect(&Token::LBrace, "'{' to start block")?;
        let mut stmts = Vec::new();
        let mut tail = None;

        loop {
            if self.check(Token::RBrace) {
                self.advance();
                break;
            }
            if self.check(Token::Eof) {
                return Err(ParserError::Eof);
            }
            let stmt = self.parse_stmt()?;
            if self.check(Token::RBrace) {
                if let Stmt::Expr(e) = stmt {
                    tail = Some(Box::new(e));
                } else {
                    stmts.push(stmt);
                }
                self.advance();
                break;
            }
            stmts.push(stmt);
        }

        Ok(Block { stmts, tail })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        // binding starts with mut or ident followed by ':'
        if self.matches(&[Token::KwMut]) {
            // binding
            let name = self.expect_ident("binding name")?;
            self.expect(&Token::Colon, "':' after binding name")?;
            let ty = self.parse_type()?;
            self.expect(&Token::Assign, "'=' after binding type")?;
            let value = self.parse_expr()?;
            return Ok(Stmt::Binding(Binding {
                mutable: true,
                name,
                ty,
                value,
            }));
        }

        if self.peek_is_ident() && self.peek_next_is(Token::Colon) {
            let binding = self.parse_binding()?;
            return Ok(Stmt::Binding(binding));
        }

        // assignment: Path '=' Expr (but not '==')
        if self.peek_is_ident() {
            let save = self.pos;
            if let Ok(path) = self.try_parse_path() {
                if self.matches(&[Token::Assign]) {
                    let value = self.parse_expr()?;
                    return Ok(Stmt::Assign(Assign {
                        target: path,
                        value,
                    }));
                }
            }
            self.pos = save; // rewind if not assign
        }

        // expression statement
        let expr = self.parse_expr()?;
        Ok(Stmt::Expr(expr))
    }

    fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_and()?;
        while self.matches(&[Token::OrOr]) {
            let right = self.parse_and()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                op: BinaryOp::Or,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_eq()?;
        while self.matches(&[Token::AndAnd]) {
            let right = self.parse_eq()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                op: BinaryOp::And,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn parse_eq(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_rel()?;
        while self.matches(&[Token::EqEq]) {
            let right = self.parse_rel()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                op: BinaryOp::Eq,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn parse_rel(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_add()?;
        while self.matches(&[Token::Lt]) {
            let right = self.parse_add()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                op: BinaryOp::Lt,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn parse_add(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_mul()?;
        loop {
            if self.matches(&[Token::Plus]) {
                let right = self.parse_mul()?;
                expr = Expr::Binary(BinaryExpr {
                    left: Box::new(expr),
                    op: BinaryOp::Add,
                    right: Box::new(right),
                });
            } else if self.matches(&[Token::Minus]) {
                let right = self.parse_mul()?;
                expr = Expr::Binary(BinaryExpr {
                    left: Box::new(expr),
                    op: BinaryOp::Sub,
                    right: Box::new(right),
                });
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_mul(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_unary()?;
        loop {
            if self.matches(&[Token::Star]) {
                let right = self.parse_unary()?;
                expr = Expr::Binary(BinaryExpr {
                    left: Box::new(expr),
                    op: BinaryOp::Mul,
                    right: Box::new(right),
                });
            } else if self.matches(&[Token::Slash]) {
                let right = self.parse_unary()?;
                expr = Expr::Binary(BinaryExpr {
                    left: Box::new(expr),
                    op: BinaryOp::Div,
                    right: Box::new(right),
                });
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParserError> {
        if self.matches(&[Token::Minus]) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary(UnaryExpr {
                op: UnaryOp::Neg,
                expr: Box::new(expr),
            }));
        }
        if self.matches(&[Token::Bang]) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary(UnaryExpr {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            }));
        }
        if self.matches(&[Token::KwCopy]) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Copy(Box::new(expr)));
        }
        if self.matches(&[Token::Amp]) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Ref(Box::new(expr)));
        }
        self.parse_if()
    }

    fn parse_if(&mut self) -> Result<Expr, ParserError> {
        if self.matches(&[Token::KwIf]) {
            let cond = self.parse_expr()?;
            self.expect(&Token::KwThen, "'then' in if expression")?;
            let then_branch = self.parse_expr()?;
            self.expect(&Token::KwElse, "'else' in if expression")?;
            let else_branch = self.parse_expr()?;
            return Ok(Expr::If(Box::new(IfExpr {
                cond,
                then_branch,
                else_branch,
            })));
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.matches(&[Token::LParen]) {
                // function call; callee must be a Path
                let path = if let Expr::Path(p) = expr {
                    p
                } else {
                    return Err(ParserError::UnexpectedToken {
                        expected: "callable path",
                        found: self.prev().clone(),
                    });
                };
                let args = if self.matches(&[Token::RParen]) {
                    Vec::new()
                } else {
                    let mut args = Vec::new();
                    loop {
                        args.push(self.parse_expr()?);
                        if self.matches(&[Token::Comma]) {
                            continue;
                        }
                        self.expect(&Token::RParen, "')' after call args")?;
                        break;
                    }
                    args
                };
                expr = Expr::FuncCall(FuncCall { callee: path, args });
                continue;
            }
            break;
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParserError> {
        match self.advance() {
            Token::Ident(name) => {
                let mut idents = vec![Ident(name)];
                while self.matches(&[Token::Dot]) {
                    let seg = self.expect_ident("path segment")?;
                    idents.push(seg);
                }
                Ok(Expr::Path(Path(idents)))
            }
            Token::Int(v) => Ok(Expr::Literal(Literal::Int(v))),
            Token::Str(s) => Ok(Expr::Literal(Literal::Str(s))),
            Token::Bool(b) => Ok(Expr::Literal(Literal::Bool(b))),
            Token::LParen => {
                if self.matches(&[Token::RParen]) {
                    return Ok(Expr::Literal(Literal::Unit));
                }
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen, "')' after expression")?;
                Ok(expr)
            }
            Token::LBrace => {
                // disambiguate record literal vs block with simple lookahead
                if self.check(Token::RBrace) {
                    self.advance();
                    return Ok(Expr::Block(Block {
                        stmts: Vec::new(),
                        tail: None,
                    }));
                }
                if self.looks_like_record_literal() {
                    let mut fields = Vec::new();
                    loop {
                        let name = self.expect_ident("field name")?;
                        self.expect(&Token::Colon, "':' after field name")?;
                        let value = self.parse_expr()?;
                        fields.push(FieldInit { name, value });
                        if self.matches(&[Token::Comma]) {
                            continue;
                        }
                        self.expect(&Token::RBrace, "'}' after record literal")?;
                        break;
                    }
                    Ok(Expr::RecordLit(RecordLit { fields }))
                } else {
                    // rewind by one to let parse_block consume '{'
                    self.pos -= 1;
                    Ok(Expr::Block(self.parse_block()?))
                }
            }
            other => Err(ParserError::UnexpectedToken {
                expected: "expression",
                found: other,
            }),
        }
    }

    // --- path helper ---
    fn try_parse_path(&mut self) -> Result<Path, ParserError> {
        let mut idents = Vec::new();
        let first = self.expect_ident("path start")?;
        idents.push(first);
        while self.matches(&[Token::Dot]) {
            let ident = self.expect_ident("path segment")?;
            idents.push(ident);
        }
        Ok(Path(idents))
    }

    // --- token helpers ---
    fn matches(&mut self, tokens: &[Token]) -> bool {
        for t in tokens {
            if self.check(t.clone()) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token: Token) -> bool {
        self.peek() == &token
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn peek_is_ident(&self) -> bool {
        matches!(self.peek(), Token::Ident(_))
    }

    fn peek_next_is(&self, expected: Token) -> bool {
        self.tokens.get(self.pos + 1) == Some(&expected)
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn prev(&self) -> &Token {
        self.tokens
            .get(self.pos.saturating_sub(1))
            .unwrap_or(&Token::Eof)
    }

    fn expect(&mut self, token: &Token, msg: &'static str) -> Result<(), ParserError> {
        if self.check(token.clone()) {
            self.advance();
            Ok(())
        } else {
            Err(ParserError::UnexpectedToken {
                expected: msg,
                found: self.peek().clone(),
            })
        }
    }

    fn expect_ident(&mut self, msg: &'static str) -> Result<Ident, ParserError> {
        match self.advance() {
            Token::Ident(name) => Ok(Ident(name)),
            other => Err(ParserError::UnexpectedToken {
                expected: msg,
                found: other,
            }),
        }
    }

    fn looks_like_record_literal(&self) -> bool {
        // Assumes current position is just after '{'
        let mut idx = self.pos;
        // need ident ':' pattern
        let Some(tok0) = self.tokens.get(idx) else {
            return false;
        };
        if !matches!(tok0, Token::Ident(_)) {
            return false;
        }
        let Some(tok1) = self.tokens.get(idx + 1) else {
            return false;
        };
        if tok1 != &Token::Colon {
            return false;
        }
        idx += 2;
        let mut depth_paren = 0usize;
        let mut depth_brace = 0usize;
        while let Some(tok) = self.tokens.get(idx) {
            match tok {
                Token::LParen => depth_paren += 1,
                Token::RParen => depth_paren = depth_paren.saturating_sub(1),
                Token::LBrace => depth_brace += 1,
                Token::RBrace => {
                    if depth_brace == 0 && depth_paren == 0 {
                        // reached end of first field
                        return true;
                    }
                    depth_brace = depth_brace.saturating_sub(1);
                }
                Token::Comma if depth_brace == 0 && depth_paren == 0 => return true,
                Token::Assign if depth_brace == 0 && depth_paren == 0 => return false,
                _ => {}
            }
            idx += 1;
        }
        false
    }
}

// --- lexer ---
fn lex(src: &str) -> Result<Vec<Token>, ParserError> {
    let mut tokens = Vec::new();
    let mut chars = src.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            c if c.is_whitespace() => {
                chars.next();
            }
            '/' => {
                chars.next();
                if chars.peek() == Some(&'/') {
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == '\n' {
                            break;
                        }
                    }
                } else {
                    tokens.push(Token::Slash);
                }
            }
            '{' => {
                chars.next();
                tokens.push(Token::LBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::RBrace);
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            ':' => {
                chars.next();
                tokens.push(Token::Colon);
            }
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
            }
            '.' => {
                chars.next();
                tokens.push(Token::Dot);
            }
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }
            '<' => {
                chars.next();
                tokens.push(Token::Lt);
            }
            '!' => {
                chars.next();
                tokens.push(Token::Bang);
            }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::EqEq);
                } else if chars.peek() == Some(&'>') {
                    // not in grammar, ignore
                } else {
                    tokens.push(Token::Assign);
                }
            }
            '-' => {
                chars.next();
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::Arrow);
                } else {
                    tokens.push(Token::Minus);
                }
            }
            '|' => {
                chars.next();
                if chars.peek() == Some(&'|') {
                    chars.next();
                    tokens.push(Token::OrOr);
                } else {
                    return Err(ParserError::Lexer("unexpected '|'".into()));
                }
            }
            '&' => {
                chars.next();
                if chars.peek() == Some(&'&') {
                    chars.next();
                    tokens.push(Token::AndAnd);
                } else {
                    tokens.push(Token::Amp);
                }
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '"' {
                        break;
                    }
                    s.push(c);
                }
                tokens.push(Token::Str(s));
            }
            '0'..='9' => {
                let mut num = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() {
                        num.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let val: i64 = num
                    .parse()
                    .map_err(|_| ParserError::InvalidNumber(num.clone()))?;
                tokens.push(Token::Int(val));
            }
            c if is_ident_start(c) => {
                let mut ident = String::new();
                ident.push(c);
                chars.next();
                while let Some(&c2) = chars.peek() {
                    if is_ident_continue(c2) {
                        ident.push(c2);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let tok = match ident.as_str() {
                    "import" => Token::KwImport,
                    "global" => Token::KwGlobal,
                    "mut" => Token::KwMut,
                    "type" => Token::KwType,
                    "if" => Token::KwIf,
                    "then" => Token::KwThen,
                    "else" => Token::KwElse,
                    "copy" => Token::KwCopy,
                    "true" => Token::Bool(true),
                    "false" => Token::Bool(false),
                    _ => Token::Ident(ident),
                };
                tokens.push(tok);
            }
            c => return Err(ParserError::Lexer(format!("unexpected char '{}'", c))),
        }
    }
    tokens.push(Token::Eof);
    Ok(tokens)
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(src: &str) -> Program {
        let mut p = Parser::new(src).unwrap();
        p.parse_program().unwrap()
    }

    #[test]
    fn parse_hello_world() {
        let src = r#"
        global greeting: Str = "hello"

        print(msg: Str) = {
          msg
        }

        main() = {
          msg: Str = greeting + " world"
          print(msg)
        }
        "#;
        let program = parse_ok(src);
        assert_eq!(program.decls.len(), 3);
    }

    #[test]
    fn parse_calc() {
        let src = r#"
        add(a: i32, b: i32) -> i32 = a + b

        main() = {
          x: i32 = 10
          y: i32 = 20
          sum: i32 = add(x, y)
          copy sum
        }
        "#;
        let program = parse_ok(src);
        assert_eq!(program.decls.len(), 2);
    }

    #[test]
    fn parse_record_and_ref() {
        let src = r#"
        type Point = { x: i32, y: i32 }

        shift(p: Point, dx: i32, dy: i32) -> Point = {
          mut moved: Point = p
          moved.x = moved.x + dx
          moved.y = moved.y + dy
          moved
        }

        length_x(p: &Point) -> i32 = p.x

        main() = {
          origin: Point = { x: 0, y: 0 }
          p1: Point = shift(origin, 5, 0)
          px: i32 = length_x(&p1)
          copy px
        }
        "#;
        let program = parse_ok(src);
        assert_eq!(program.decls.len(), 4);
    }

    #[test]
    fn fails_on_incomplete_if() {
        let src = "if true then 1";
        let mut parser = Parser::new(src).unwrap();
        let err = parser.parse_program().unwrap_err();
        assert!(matches!(err, ParserError::UnexpectedToken { .. }));
    }
}
