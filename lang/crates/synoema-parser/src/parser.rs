use synoema_lexer::{SpannedToken, Token, Span};
use crate::ast::*;
use crate::error::ParseError;

type PResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone, Copy)]
enum ListType { Regular, Range, Comprehension }

/// Synoema parser: converts token stream into AST.
///
/// Uses Pratt parsing (top-down operator precedence) for expressions
/// and recursive descent for declarations and patterns.
pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    // ── Token stream helpers ──────────────────────────────

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).map(|st| &st.token).unwrap_or(&Token::Eof)
    }

    fn peek_span(&self) -> Span {
        self.tokens.get(self.pos).map(|st| st.span).unwrap_or(Span::dummy())
    }

    #[allow(dead_code)]
    fn at(&self, t: &Token) -> bool {
        self.peek() == t
    }

    fn advance(&mut self) -> &SpannedToken {
        let st = &self.tokens[self.pos];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        st
    }

    fn expect(&mut self, expected: &Token) -> PResult<Span> {
        if self.peek() == expected {
            let span = self.peek_span();
            self.advance();
            Ok(span)
        } else {
            Err(ParseError::expected(
                &format!("{:?}", expected),
                self.peek(),
                self.peek_span(),
            ))
        }
    }

    fn eat(&mut self, t: &Token) -> bool {
        if self.peek() == t {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_newlines(&mut self) {
        while self.peek() == &Token::Newline {
            self.advance();
        }
    }

    fn error(&self, msg: impl Into<String>) -> ParseError {
        ParseError::new(msg, self.peek_span())
    }

    // ── Program ───────────────────────────────────────────

    pub fn parse_program(&mut self) -> PResult<Program> {
        let mut decls = Vec::new();
        self.skip_newlines();

        while self.peek() != &Token::Eof {
            let decl = self.parse_decl()?;
            decls.push(decl);
            self.skip_newlines();
        }

        // Group function equations by name
        let decls = group_func_equations(decls);

        Ok(Program { decls })
    }

    // ── Declarations ──────────────────────────────────────

    fn parse_decl(&mut self) -> PResult<Decl> {
        match self.peek().clone() {
            // ADT: UpperId params = Variant | Variant
            Token::UpperId(_) => self.parse_type_def(),

            // Function def or type sig: lowerId ...
            Token::LowerId(_) => {
                // Look ahead: is this `name : Type` (sig) or `name pats = expr` (def)?
                if self.is_type_sig() {
                    self.parse_type_sig()
                } else {
                    self.parse_func_def()
                }
            }

            _ => Err(self.error(format!("Expected declaration, got {:?}", self.peek()))),
        }
    }

    fn is_type_sig(&self) -> bool {
        // name : ...  (where : is at top level, not inside parens)
        let mut i = self.pos + 1;
        let mut depth = 0i32;
        while i < self.tokens.len() {
            match &self.tokens[i].token {
                Token::LParen | Token::LBracket => { depth += 1; i += 1; }
                Token::RParen | Token::RBracket => { depth -= 1; i += 1; }
                Token::Colon if depth == 0 => return true,
                Token::Assign if depth == 0 => return false,
                Token::Newline | Token::Eof => return false,
                _ => i += 1,
            }
        }
        false
    }

    fn parse_type_sig(&mut self) -> PResult<Decl> {
        let span = self.peek_span();
        let name = self.expect_lower_id()?;
        self.expect(&Token::Colon)?;
        let ty = self.parse_type()?;
        self.eat(&Token::Newline);
        Ok(Decl::TypeSig(TypeSig { name, ty, span }))
    }

    fn parse_func_def(&mut self) -> PResult<Decl> {
        let span = self.peek_span();
        let name = self.expect_lower_id()?;
        let pats = self.parse_patterns_until_assign()?;
        self.expect(&Token::Assign)?;

        let body = if self.eat(&Token::Indent) {
            // Body is an indented block: bindings + final expr
            self.parse_block_body()?
        } else {
            // Body starts on the same line
            let inline_body = self.parse_expr()?;
            self.eat(&Token::Newline);

            // Check for trailing indented where-bindings:
            //   f x = expr
            //     a = val1
            //     b = val2
            // Desugars to: Block([a=val1, b=val2], expr)
            if self.eat(&Token::Indent) {
                let mut bindings = Vec::new();
                self.skip_newlines();
                while self.peek() != &Token::Dedent && self.peek() != &Token::Eof {
                    if self.is_binding_ahead() {
                        let bspan = self.peek_span();
                        let bname = self.expect_lower_id()?;
                        self.expect(&Token::Assign)?;
                        let value = if self.eat(&Token::Indent) {
                            self.parse_block_body()?
                        } else {
                            self.parse_expr()?
                        };
                        self.skip_newlines();
                        bindings.push(Binding { name: bname, value, span: bspan });
                    } else {
                        break;
                    }
                }
                self.eat(&Token::Dedent);
                if bindings.is_empty() {
                    inline_body
                } else {
                    Expr::new(ExprKind::Block(bindings, Box::new(inline_body)), span)
                }
            } else {
                inline_body
            }
        };

        self.eat(&Token::Newline);

        Ok(Decl::Func {
            name: name.clone(),
            equations: vec![Equation { pats, body, span }],
            span,
        })
    }

    fn parse_type_def(&mut self) -> PResult<Decl> {
        let span = self.peek_span();
        let name = self.expect_upper_id()?;

        let mut params = Vec::new();
        while let Token::LowerId(_) = self.peek() {
            params.push(self.expect_lower_id()?);
        }

        self.expect(&Token::Assign)?;

        let mut variants = vec![self.parse_variant()?];
        while self.eat(&Token::Bar) {
            variants.push(self.parse_variant()?);
        }

        self.eat(&Token::Newline);

        Ok(Decl::TypeDef { name, params, variants, span })
    }

    fn parse_variant(&mut self) -> PResult<Variant> {
        let span = self.peek_span();
        let name = self.expect_upper_id()?;

        let mut fields = Vec::new();
        loop {
            match self.peek() {
                Token::UpperId(_) => {
                    let ty = self.parse_type_atom()?;
                    fields.push(ty);
                }
                Token::LowerId(_) => {
                    let ty = self.parse_type_atom()?;
                    fields.push(ty);
                }
                Token::LParen => {
                    let ty = self.parse_type_atom()?;
                    fields.push(ty);
                }
                _ => break,
            }
        }

        Ok(Variant { name, fields, span })
    }

    // ── Patterns ──────────────────────────────────────────

    fn parse_patterns_until_assign(&mut self) -> PResult<Vec<Pat>> {
        let mut pats = Vec::new();
        while self.peek() != &Token::Assign {
            pats.push(self.parse_pattern()?);
        }
        Ok(pats)
    }

    fn parse_pattern(&mut self) -> PResult<Pat> {
        match self.peek().clone() {
            Token::Underscore => { self.advance(); Ok(Pat::Wildcard) }

            Token::Int(n) => { self.advance(); Ok(Pat::Lit(Lit::Int(n))) }
            Token::Float(f) => { self.advance(); Ok(Pat::Lit(Lit::Float(f))) }
            Token::Str(s) => { self.advance(); Ok(Pat::Lit(Lit::Str(s))) }
            Token::Char(c) => { self.advance(); Ok(Pat::Lit(Lit::Char(c))) }
            Token::KwTrue => { self.advance(); Ok(Pat::Lit(Lit::Bool(true))) }
            Token::KwFalse => { self.advance(); Ok(Pat::Lit(Lit::Bool(false))) }

            Token::LowerId(s) => { self.advance(); Ok(Pat::Var(s)) }

            Token::UpperId(name) => {
                self.advance();
                // Constructor with arguments — but only simple atoms
                let mut args = Vec::new();
                loop {
                    match self.peek() {
                        Token::LowerId(_) | Token::Underscore
                        | Token::Int(_) | Token::Float(_)
                        | Token::Str(_) | Token::Char(_)
                        | Token::KwTrue | Token::KwFalse => {
                            args.push(self.parse_pattern()?);
                        }
                        Token::LParen => {
                            args.push(self.parse_paren_pattern()?);
                        }
                        _ => break,
                    }
                }
                if args.is_empty() {
                    Ok(Pat::Con(name, vec![]))
                } else {
                    Ok(Pat::Con(name, args))
                }
            }

            Token::LParen => self.parse_paren_pattern(),

            Token::LBracket => {
                self.advance();
                self.expect(&Token::RBracket)?;
                Ok(Pat::Con("Nil".into(), vec![]))
            }

            _ => Err(self.error(format!("Expected pattern, got {:?}", self.peek()))),
        }
    }

    fn parse_paren_pattern(&mut self) -> PResult<Pat> {
        self.expect(&Token::LParen)?;
        let p = self.parse_pattern()?;
        if self.eat(&Token::Colon) {
            let rest = self.parse_pattern()?;
            self.expect(&Token::RParen)?;
            Ok(Pat::Cons(Box::new(p), Box::new(rest)))
        } else {
            self.expect(&Token::RParen)?;
            Ok(Pat::Paren(Box::new(p)))
        }
    }

    // ── Expressions (Pratt parsing) ───────────────────────

    pub fn parse_expr(&mut self) -> PResult<Expr> {
        self.parse_pratt(0)
    }

    fn parse_pratt(&mut self, min_bp: u8) -> PResult<Expr> {
        let mut lhs = self.parse_prefix()?;

        loop {
            // Check for binary operator
            let (op, lbp, rbp) = match self.peek() {
                Token::Pipe      => (BinOp::Pipe,    2, 3),
                Token::Or        => (BinOp::Or,      4, 5),
                Token::And       => (BinOp::And,     6, 7),
                Token::Eq        => (BinOp::Eq,      8, 9),
                Token::Neq       => (BinOp::Neq,     8, 9),
                Token::Lt        => (BinOp::Lt,      10, 11),
                Token::Gt        => (BinOp::Gt,      10, 11),
                Token::Lte       => (BinOp::Lte,     10, 11),
                Token::Gte       => (BinOp::Gte,     10, 11),
                Token::Concat    => (BinOp::Concat,  12, 12), // right-assoc
                Token::Colon     => (BinOp::Cons,    13, 13), // right-assoc (cons)
                Token::Plus      => (BinOp::Add,     14, 15),
                Token::Minus     => (BinOp::Sub,     14, 15),
                Token::Star      => (BinOp::Mul,     16, 17),
                Token::Slash     => (BinOp::Div,     16, 17),
                Token::Percent   => (BinOp::Mod,     16, 17),
                Token::Compose   => (BinOp::Compose, 18, 18), // right-assoc

                // Field access (highest precedence for postfix)
                Token::Dot => {
                    if min_bp > 22 { break; }
                    self.advance();
                    let field = self.expect_lower_id()?;
                    let span = lhs.span;
                    lhs = Expr::new(ExprKind::Field(Box::new(lhs), field), span);
                    continue;
                }

                // Function application: next token starts an atom
                _ if self.can_start_atom() && min_bp <= 20 => {
                    let arg = self.parse_atom()?;
                    let span = lhs.span;
                    lhs = Expr::new(ExprKind::App(Box::new(lhs), Box::new(arg)), span);
                    continue;
                }

                _ => break,
            };

            if lbp < min_bp {
                break;
            }

            self.advance(); // consume the operator
            let rhs = self.parse_pratt(rbp)?;
            let span = lhs.span;
            lhs = Expr::new(ExprKind::BinOp(op, Box::new(lhs), Box::new(rhs)), span);
        }

        Ok(lhs)
    }

    /// Parse a prefix expression (unary -, lambda, conditional, or atom)
    fn parse_prefix(&mut self) -> PResult<Expr> {
        let span = self.peek_span();

        match self.peek().clone() {
            // Unary minus
            Token::Minus => {
                self.advance();
                let e = self.parse_pratt(21)?; // high binding power
                Ok(Expr::new(ExprKind::Neg(Box::new(e)), span))
            }

            // Lambda: \x y -> body
            Token::Backslash => {
                self.advance();
                let mut pats = Vec::new();
                while self.peek() != &Token::Arrow {
                    pats.push(self.parse_pattern()?);
                }
                self.expect(&Token::Arrow)?;
                let body = self.parse_expr()?;
                Ok(Expr::new(ExprKind::Lam(pats, Box::new(body)), span))
            }

            // Conditional: ? cond -> then : else
            // The then-branch is parsed at pratt level 14 (above cons/concat)
            // so that `:` is consumed as the else-separator, not as cons.
            Token::Question => {
                self.advance();
                // Condition: parse full expression. Arrow is not an infix op,
                // so parse_pratt(0) naturally stops before ->
                let cond = self.parse_pratt(0)?;
                self.expect(&Token::Arrow)?;
                // Then-branch: parse above Cons (:) level so : is separator, not cons
                let then_e = self.parse_pratt(14)?;
                // Allow newline before `:` for multi-line conditionals:
                //   ? cond -> then_expr
                //   : else_expr
                self.skip_newlines();
                self.expect(&Token::Colon)?;
                let else_e = self.parse_expr()?;
                Ok(Expr::new(ExprKind::Cond(
                    Box::new(cond), Box::new(then_e), Box::new(else_e)
                ), span))
            }

            _ => self.parse_atom(),
        }
    }

    /// Can the next token start an atom (for function application)?
    fn can_start_atom(&self) -> bool {
        matches!(self.peek(),
            Token::LowerId(_) | Token::UpperId(_) |
            Token::Int(_) | Token::Float(_) | Token::Str(_) | Token::Char(_) |
            Token::KwTrue | Token::KwFalse |
            Token::LParen | Token::LBracket
        )
    }

    /// Parse an atomic expression (highest binding power)
    fn parse_atom(&mut self) -> PResult<Expr> {
        let span = self.peek_span();

        match self.peek().clone() {
            Token::Int(n) => { self.advance(); Ok(Expr::new(ExprKind::Lit(Lit::Int(n)), span)) }
            Token::Float(f) => { self.advance(); Ok(Expr::new(ExprKind::Lit(Lit::Float(f)), span)) }
            Token::Str(s) => { self.advance(); Ok(Expr::new(ExprKind::Lit(Lit::Str(s)), span)) }
            Token::Char(c) => { self.advance(); Ok(Expr::new(ExprKind::Lit(Lit::Char(c)), span)) }
            Token::KwTrue => { self.advance(); Ok(Expr::new(ExprKind::Lit(Lit::Bool(true)), span)) }
            Token::KwFalse => { self.advance(); Ok(Expr::new(ExprKind::Lit(Lit::Bool(false)), span)) }

            Token::LowerId(s) => { self.advance(); Ok(Expr::new(ExprKind::Var(s), span)) }
            Token::UpperId(s) => { self.advance(); Ok(Expr::new(ExprKind::Con(s), span)) }

            Token::LParen => {
                self.advance();
                let e = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::new(ExprKind::Paren(Box::new(e)), span))
            }

            Token::LBracket => self.parse_list_expr(span),

            _ => Err(self.error(format!("Expected expression, got {:?}", self.peek()))),
        }
    }

    /// Parse list: `[]`, `[1 2 3]`, `[1..10]`, `[x | x <- xs, guard]`
    fn parse_list_expr(&mut self, span: Span) -> PResult<Expr> {
        self.expect(&Token::LBracket)?;

        // Empty list
        if self.eat(&Token::RBracket) {
            return Ok(Expr::new(ExprKind::List(vec![]), span));
        }

        // Look ahead to determine list type:
        // scan for `|` (comprehension), `..` (range), or neither (regular list)
        let list_type = self.classify_list();

        match list_type {
            ListType::Comprehension => {
                let body = self.parse_expr()?;
                self.expect(&Token::Bar)?;
                let gens = self.parse_generators()?;
                self.expect(&Token::RBracket)?;
                Ok(Expr::new(ExprKind::ListComp(Box::new(body), gens), span))
            }
            ListType::Range => {
                let start = self.parse_atom()?;
                self.expect(&Token::DotDot)?;
                let end = self.parse_atom()?;
                self.expect(&Token::RBracket)?;
                Ok(Expr::new(ExprKind::Range(Box::new(start), Box::new(end)), span))
            }
            ListType::Regular => {
                // Space-separated atoms: [1 2 3], [(f x) (g y)]
                let mut elems = Vec::new();
                while self.peek() != &Token::RBracket && self.peek() != &Token::Eof {
                    elems.push(self.parse_atom()?);
                }
                self.expect(&Token::RBracket)?;
                Ok(Expr::new(ExprKind::List(elems), span))
            }
        }
    }

    /// Scan ahead (without consuming) to classify the list expression type
    fn classify_list(&self) -> ListType {
        let mut i = self.pos;
        let mut depth = 1i32; // inside [ already
        while i < self.tokens.len() {
            match &self.tokens[i].token {
                Token::LBracket | Token::LParen => depth += 1,
                Token::RBracket => {
                    depth -= 1;
                    if depth == 0 { return ListType::Regular; }
                }
                Token::RParen => depth -= 1,
                Token::Bar if depth == 1 => return ListType::Comprehension,
                Token::DotDot if depth == 1 => return ListType::Range,
                Token::Eof => return ListType::Regular,
                _ => {}
            }
            i += 1;
        }
        ListType::Regular
    }

    fn parse_generators(&mut self) -> PResult<Vec<Generator>> {
        let mut gens = vec![self.parse_generator()?];
        while self.eat(&Token::Comma) {
            gens.push(self.parse_generator()?);
        }
        Ok(gens)
    }

    fn parse_generator(&mut self) -> PResult<Generator> {
        // Try: `id <-` for bind
        if let Token::LowerId(name) = self.peek().clone() {
            if self.tokens.get(self.pos + 1).map(|t| &t.token) == Some(&Token::BackArrow) {
                self.advance(); // name
                self.advance(); // <-
                let expr = self.parse_expr()?;
                return Ok(Generator::Bind(name, expr));
            }
        }
        // Otherwise: guard expression
        let expr = self.parse_expr()?;
        Ok(Generator::Guard(expr))
    }

    /// Parse a block body (after INDENT):
    /// bindings... then final expression, then DEDENT
    fn parse_block_body(&mut self) -> PResult<Expr> {
        let span = self.peek_span();
        self.skip_newlines();

        let mut bindings = Vec::new();

        // Parse bindings: `name = expr` followed by newline
        // The last expression (without `=`) is the block's value
        loop {
            if self.peek() == &Token::Dedent || self.peek() == &Token::Eof {
                break;
            }

            // Try to parse as binding: look for `lower =`
            if self.is_binding_ahead() {
                let bspan = self.peek_span();
                let name = self.expect_lower_id()?;
                self.expect(&Token::Assign)?;
                let value = if self.eat(&Token::Indent) {
                    self.parse_block_body()?
                } else {
                    self.parse_expr()?
                };
                self.skip_newlines();
                bindings.push(Binding { name, value, span: bspan });
            } else {
                // Final expression
                let expr = self.parse_expr()?;
                self.skip_newlines();
                self.eat(&Token::Dedent);

                if bindings.is_empty() {
                    return Ok(expr);
                } else {
                    return Ok(Expr::new(ExprKind::Block(bindings, Box::new(expr)), span));
                }
            }
        }

        self.eat(&Token::Dedent);

        // If we only got bindings and no final expression, error
        Err(self.error("Block must end with an expression"))
    }

    /// Look ahead to see if current position is `lowerId =` (binding)
    fn is_binding_ahead(&self) -> bool {
        if let Token::LowerId(_) = self.peek() {
            // Check if immediately followed by `=` (not `==`)
            if let Some(next) = self.tokens.get(self.pos + 1) {
                return next.token == Token::Assign;
            }
        }
        false
    }

    // ── Types ─────────────────────────────────────────────

    fn parse_type(&mut self) -> PResult<TypeExpr> {
        let span = self.peek_span();
        let t = self.parse_type_app()?;

        if self.eat(&Token::Arrow) {
            let rhs = self.parse_type()?; // right-associative
            Ok(TypeExpr::new(TypeExprKind::Arrow(Box::new(t), Box::new(rhs)), span))
        } else {
            Ok(t)
        }
    }

    fn parse_type_app(&mut self) -> PResult<TypeExpr> {
        let mut t = self.parse_type_atom()?;

        // Type application: `Maybe Int`, `List a`
        while self.can_start_type_atom() {
            let arg = self.parse_type_atom()?;
            let span = t.span;
            t = TypeExpr::new(TypeExprKind::App(Box::new(t), Box::new(arg)), span);
        }

        Ok(t)
    }

    fn can_start_type_atom(&self) -> bool {
        matches!(self.peek(), Token::UpperId(_) | Token::LowerId(_) | Token::LParen)
    }

    fn parse_type_atom(&mut self) -> PResult<TypeExpr> {
        let span = self.peek_span();

        match self.peek().clone() {
            Token::UpperId(s) => {
                self.advance();
                Ok(TypeExpr::new(TypeExprKind::Con(s), span))
            }
            Token::LowerId(s) => {
                self.advance();
                Ok(TypeExpr::new(TypeExprKind::Var(s), span))
            }
            Token::LParen => {
                self.advance();
                let t = self.parse_type()?;
                self.expect(&Token::RParen)?;
                Ok(TypeExpr::new(TypeExprKind::Paren(Box::new(t)), span))
            }
            _ => Err(self.error(format!("Expected type, got {:?}", self.peek()))),
        }
    }

    // ── Identifier helpers ────────────────────────────────

    fn expect_lower_id(&mut self) -> PResult<String> {
        match self.peek().clone() {
            Token::LowerId(s) => { self.advance(); Ok(s) }
            _ => Err(ParseError::expected("identifier", self.peek(), self.peek_span())),
        }
    }

    fn expect_upper_id(&mut self) -> PResult<String> {
        match self.peek().clone() {
            Token::UpperId(s) => { self.advance(); Ok(s) }
            _ => Err(ParseError::expected("type name", self.peek(), self.peek_span())),
        }
    }
}

// ── Post-processing ──────────────────────────────────────

/// Group consecutive function equations with the same name into a single Decl::Func.
/// e.g. `fac 0 = 1` + `fac n = n * fac (n-1)` → one Func with 2 equations.
fn group_func_equations(decls: Vec<Decl>) -> Vec<Decl> {
    let mut result: Vec<Decl> = Vec::new();

    for decl in decls {
        match decl {
            Decl::Func { name, equations, span } => {
                if let Some(Decl::Func {
                    name: ref prev_name,
                    equations: ref mut prev_eqs,
                    ..
                }) = result.last_mut()
                {
                    if *prev_name == name {
                        prev_eqs.extend(equations);
                        continue;
                    }
                }
                result.push(Decl::Func { name, equations, span });
            }
            other => result.push(other),
        }
    }

    result
}
