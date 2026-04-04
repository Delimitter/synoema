// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

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

    /// Collect consecutive DocComment tokens, skipping newlines between them.
    /// Returns the collected doc lines (empty if no doc-comments found).
    fn collect_doc_comments(&mut self) -> Vec<String> {
        let mut doc = Vec::new();
        loop {
            match self.peek().clone() {
                Token::DocComment(text) => {
                    doc.push(text);
                    self.advance();
                    // skip optional newline after doc comment
                    self.eat(&Token::Newline);
                }
                Token::Newline => { self.advance(); }
                _ => break,
            }
        }
        doc
    }

    fn error(&self, msg: impl Into<String>) -> ParseError {
        ParseError::new(msg, self.peek_span())
    }

    // ── Program ───────────────────────────────────────────

    pub fn parse_program(&mut self) -> PResult<Program> {
        let (program, errors) = self.parse_program_recovering();
        if errors.is_empty() {
            Ok(program)
        } else {
            Err(errors.into_iter().next().unwrap())
        }
    }

    /// Parse program collecting all errors instead of stopping at the first one.
    /// Returns partial AST + all accumulated errors.
    pub fn parse_program_recovering(&mut self) -> (Program, Vec<ParseError>) {
        let mut imports = Vec::new();
        let mut decls = Vec::new();
        let mut modules = Vec::new();
        let mut uses = Vec::new();
        let mut errors = Vec::new();
        self.skip_newlines();

        while self.peek() != &Token::Eof {
            // Collect doc-comments before the next declaration
            let doc = self.collect_doc_comments();

            match self.peek().clone() {
                Token::KwImport => {
                    match self.parse_import() {
                        Ok(import_decl) => {
                            let _ = doc;
                            imports.push(import_decl);
                        }
                        Err(e) => {
                            errors.push(e);
                            self.skip_to_next_decl();
                        }
                    }
                }
                Token::KwMod => {
                    match self.parse_module() {
                        Ok(mut module) => {
                            module.doc = doc;
                            modules.push(module);
                        }
                        Err(e) => {
                            errors.push(e);
                            self.skip_to_next_decl();
                        }
                    }
                }
                Token::KwTest => {
                    match self.parse_test_decl() {
                        Ok(decl) => {
                            let _ = doc;
                            decls.push(decl);
                        }
                        Err(e) => {
                            errors.push(e);
                            self.skip_to_next_decl();
                        }
                    }
                }
                Token::KwUse => {
                    match self.parse_use() {
                        Ok(use_decl) => {
                            let _ = doc;
                            uses.push(use_decl);
                        }
                        Err(e) => {
                            errors.push(e);
                            self.skip_to_next_decl();
                        }
                    }
                }
                Token::Eof => break,
                _ => {
                    match self.parse_decl() {
                        Ok(mut decl) => {
                            attach_doc(&mut decl, doc);
                            decls.push(decl);
                        }
                        Err(e) => {
                            errors.push(e);
                            self.skip_to_next_decl();
                        }
                    }
                }
            }
            self.skip_newlines();
        }

        // Group function equations by name
        let decls = group_func_equations(decls);

        (Program { imports, decls, modules, uses }, errors)
    }

    /// Skip tokens until the next top-level declaration boundary.
    /// A declaration boundary is: a token at the beginning of a line
    /// (after Newline) that could start a new declaration.
    fn skip_to_next_decl(&mut self) {
        loop {
            match self.peek() {
                Token::Eof => break,
                Token::Newline => {
                    self.advance();
                    // After newline, check if next token can start a declaration
                    match self.peek() {
                        Token::LowerId(_) | Token::UpperId(_)
                        | Token::KwMod | Token::KwUse | Token::KwTrait
                        | Token::KwImpl | Token::KwType | Token::Eof => break,
                        Token::DocComment(_) => break,
                        _ => {} // keep skipping
                    }
                }
                _ => { self.advance(); }
            }
        }
    }

    fn parse_import(&mut self) -> PResult<ImportDecl> {
        let span = self.peek_span();
        self.expect(&Token::KwImport)?;
        match self.peek().clone() {
            Token::Str(path) => {
                self.advance();
                self.eat(&Token::Newline);
                Ok(ImportDecl { path, span })
            }
            _ => Err(self.error("expected string literal after 'import'")),
        }
    }

    fn parse_module(&mut self) -> PResult<ModuleDecl> {
        self.expect(&Token::KwMod)?;
        let name = self.expect_upper_id()?;
        self.eat(&Token::Newline);
        self.expect(&Token::Indent)?;

        let mut body = Vec::new();
        self.skip_newlines();
        while self.peek() != &Token::Dedent && self.peek() != &Token::Eof {
            let doc = self.collect_doc_comments();
            if self.peek() == &Token::Dedent || self.peek() == &Token::Eof {
                break;
            }
            let mut decl = self.parse_decl()?;
            attach_doc(&mut decl, doc);
            body.push(decl);
            self.skip_newlines();
        }
        self.eat(&Token::Dedent);

        let body = group_func_equations(body);
        Ok(ModuleDecl { name, body, doc: Vec::new() })
    }

    fn parse_use(&mut self) -> PResult<UseDecl> {
        let span = self.peek_span();
        self.expect(&Token::KwUse)?;
        let module = self.expect_upper_id()?;
        self.expect(&Token::LParen)?;
        let names = if self.peek() == &Token::Star {
            self.advance();
            vec!["*".into()]
        } else {
            let mut ns = Vec::new();
            while self.peek() != &Token::RParen && self.peek() != &Token::Eof {
                ns.push(self.expect_lower_id()?);
            }
            ns
        };
        self.expect(&Token::RParen)?;
        self.eat(&Token::Newline);
        Ok(UseDecl { module, names, span })
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

            // Type class
            Token::KwTrait => self.parse_trait_decl(),

            // Type class implementation
            Token::KwImpl => self.parse_impl_decl(),

            // Type alias: type Name params = TypeExpr
            Token::KwType => self.parse_type_alias(),

            _ => Err(self.error(format!("Expected declaration, got {:?}", self.peek()))),
        }
    }

    fn parse_trait_decl(&mut self) -> PResult<Decl> {
        let span = self.peek_span();
        self.expect(&Token::KwTrait)?;
        let name = self.expect_upper_id()?;
        let ty_param = self.expect_lower_id()?;
        self.eat(&Token::Newline);
        self.expect(&Token::Indent)?;

        let mut methods = Vec::new();
        self.skip_newlines();
        while self.peek() != &Token::Dedent && self.peek() != &Token::Eof {
            if self.is_type_sig() {
                if let Decl::TypeSig(sig) = self.parse_type_sig()? {
                    methods.push(sig);
                }
            } else {
                break;
            }
            self.skip_newlines();
        }
        self.eat(&Token::Dedent);
        Ok(Decl::TraitDecl { name, ty_param, methods, span, doc: Vec::new() })
    }

    fn parse_impl_decl(&mut self) -> PResult<Decl> {
        let span = self.peek_span();
        self.expect(&Token::KwImpl)?;
        let trait_name = self.expect_upper_id()?;
        let ty_name = self.expect_upper_id()?;
        self.eat(&Token::Newline);
        self.expect(&Token::Indent)?;

        let mut methods: Vec<(String, Vec<Equation>)> = Vec::new();
        self.skip_newlines();
        while self.peek() != &Token::Dedent && self.peek() != &Token::Eof {
            if let Token::LowerId(_) = self.peek() {
                if self.is_type_sig() {
                    // Skip type signatures inside impl blocks
                    self.parse_type_sig()?;
                } else {
                    let decl = self.parse_func_def()?;
                    if let Decl::Func { name, equations, .. } = decl {
                        if let Some((_, existing)) = methods.iter_mut().find(|(n, _)| n == &name) {
                            existing.extend(equations);
                        } else {
                            methods.push((name, equations));
                        }
                    }
                }
            } else {
                break;
            }
            self.skip_newlines();
        }
        self.eat(&Token::Dedent);
        Ok(Decl::ImplDecl { trait_name, ty_name, methods, span })
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
            doc: Vec::new(), // filled by caller via attach_doc
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

        // Parse optional deriving clause: deriving (Show, Eq, Ord)
        let derives = if self.eat(&Token::KwDerive) {
            self.expect(&Token::LParen)?;
            let mut ds = vec![self.expect_upper_id()?];
            while self.eat(&Token::Comma) {
                ds.push(self.expect_upper_id()?);
            }
            self.expect(&Token::RParen)?;
            ds
        } else {
            Vec::new()
        };

        self.eat(&Token::Newline);

        Ok(Decl::TypeDef { name, params, variants, span, doc: Vec::new(), derives })
    }

    fn parse_type_alias(&mut self) -> PResult<Decl> {
        let span = self.peek_span();
        self.expect(&Token::KwType)?;
        let name = self.expect_upper_id()?;

        let mut params = Vec::new();
        while let Token::LowerId(_) = self.peek() {
            params.push(self.expect_lower_id()?);
        }

        self.expect(&Token::Assign)?;
        let body = self.parse_type()?;
        self.eat(&Token::Newline);

        Ok(Decl::TypeAlias { name, params, body, span })
    }

    fn parse_test_decl(&mut self) -> PResult<Decl> {
        let span = self.peek_span();
        self.expect(&Token::KwTest)?;
        let name = match self.peek() {
            Token::Str(_) => {
                if let Token::Str(s) = self.advance().token.clone() { s } else { unreachable!() }
            }
            _ => return Err(self.error(format!("Expected test name (string), got {:?}", self.peek()))),
        };
        self.expect(&Token::Assign)?;
        let body = self.parse_expr()?;
        self.eat(&Token::Newline);
        Ok(Decl::Test { name, body, span })
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
                // [] → Nil, [x] → Cons(x, Nil), [x y z] → Cons(x, Cons(y, Cons(z, Nil)))
                if self.eat(&Token::RBracket) {
                    return Ok(Pat::Con("Nil".into(), vec![]));
                }
                let mut elems = Vec::new();
                while self.peek() != &Token::RBracket && self.peek() != &Token::Eof {
                    elems.push(self.parse_pattern()?);
                }
                self.expect(&Token::RBracket)?;
                // Fold right: [a b c] → Cons(a, Cons(b, Cons(c, Nil)))
                let mut pat = Pat::Con("Nil".into(), vec![]);
                for elem in elems.into_iter().rev() {
                    pat = Pat::Cons(Box::new(elem), Box::new(pat));
                }
                Ok(pat)
            }

            Token::LBrace => {
                self.advance();
                let mut fields = Vec::new();
                while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
                    let fname = self.expect_lower_id()?;
                    let pat = if self.peek() == &Token::Assign {
                        self.advance();
                        self.parse_pattern()?
                    } else {
                        // Pattern punning: {x} → {x = x}
                        Pat::Var(fname.clone())
                    };
                    fields.push((fname, pat));
                    self.eat(&Token::Comma);
                }
                self.expect(&Token::RBrace)?;
                Ok(Pat::Record(fields))
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
        } else if self.eat(&Token::Comma) {
            // (a, b) tuple pattern → desugars to {fst = a, snd = b}
            let p2 = self.parse_pattern()?;
            self.expect(&Token::RParen)?;
            Ok(Pat::Record(vec![
                ("fst".to_string(), p),
                ("snd".to_string(), p2),
            ]))
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
                Token::Semicolon => (BinOp::Seq,     0, 1), // lowest precedence, right-assoc
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
                Token::StarStar  => (BinOp::Pow,     17, 17), // right-assoc, higher than Mul/Div (rbp==lbp for right-assoc)
                Token::Compose   => (BinOp::Compose, 18, 18), // right-assoc

                // Conditional: `body when cond` (right-assoc, above |>, below ||)
                Token::KwWhen => {
                    let lbp = 3u8;
                    let rbp = 3u8;
                    if lbp < min_bp { break; }
                    self.advance();
                    let rhs = self.parse_pratt(rbp)?;
                    let span = lhs.span;
                    lhs = Expr::new(ExprKind::When(Box::new(lhs), Box::new(rhs)), span);
                    continue;
                }

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
                // Allow newline/indent before `:` for multi-line conditionals:
                //   ? cond -> then_expr
                //   : else_expr
                // Also handle nested ternary where `:` is at deeper indent:
                //   ? cond -> ? inner_cond -> expr
                //     : else_expr
                self.skip_newlines();
                let ate_indent = self.eat(&Token::Indent);
                self.expect(&Token::Colon)?;
                let else_e = if self.eat(&Token::Indent) {
                    // Indented block after `:` — parse as block body
                    self.parse_block_body()?
                } else if self.is_binding_ahead() {
                    // Inline where-bindings after `:`:
                    //   ? cond -> expr
                    //   : y = x + 1
                    //     z = y * 2
                    //     z
                    self.parse_ternary_else_bindings(span)?
                } else {
                    self.parse_expr()?
                };
                if ate_indent {
                    self.eat(&Token::Dedent);
                }
                Ok(Expr::new(ExprKind::Cond(
                    Box::new(cond), Box::new(then_e), Box::new(else_e)
                ), span))
            }

            // scope { body }  — body can be single-line or a multi-line INDENT block
            Token::KwScope => {
                self.advance();
                self.expect(&Token::LBrace)?;
                let body = if self.eat(&Token::Indent) {
                    let b = self.parse_block_body()?;
                    self.skip_newlines();
                    b
                } else {
                    self.parse_expr()?
                };
                self.expect(&Token::RBrace)?;
                Ok(Expr::new(ExprKind::Scope(Box::new(body)), span))
            }

            // spawn expr  (parses one application-level expression)
            Token::KwSpawn => {
                self.advance();
                let expr = self.parse_pratt(20)?;
                Ok(Expr::new(ExprKind::Spawn(Box::new(expr)), span))
            }

            // prop x y -> body  (property generator)
            Token::KwProp => {
                self.advance();
                let mut vars = Vec::new();
                while self.peek() != &Token::Arrow {
                    vars.push(self.expect_lower_id()?);
                }
                self.expect(&Token::Arrow)?;
                let body = self.parse_expr()?;
                Ok(Expr::new(ExprKind::Prop(vars, Box::new(body)), span))
            }

            _ => self.parse_atom(),
        }
    }

    /// Can the next token start an atom (for function application)?
    fn can_start_atom(&self) -> bool {
        matches!(self.peek(),
            Token::LowerId(_) | Token::UpperId(_) |
            Token::Int(_) | Token::Float(_) | Token::Str(_) | Token::Char(_) |
            Token::StringFragment(_) | Token::InterpStart |
            Token::KwTrue | Token::KwFalse |
            Token::LParen | Token::LBracket | Token::LBrace
        )
    }

    /// Parse an atomic expression (highest binding power)
    fn parse_atom(&mut self) -> PResult<Expr> {
        let span = self.peek_span();

        match self.peek().clone() {
            Token::Int(n) => { self.advance(); Ok(Expr::new(ExprKind::Lit(Lit::Int(n)), span)) }
            Token::Float(f) => { self.advance(); Ok(Expr::new(ExprKind::Lit(Lit::Float(f)), span)) }
            Token::Str(s) => { self.advance(); Ok(Expr::new(ExprKind::Lit(Lit::Str(s)), span)) }

            // String interpolation: StringFragment ... InterpStart expr InterpEnd ...
            Token::StringFragment(_) | Token::InterpStart => {
                return self.parse_interp_string(span);
            }
            Token::Char(c) => { self.advance(); Ok(Expr::new(ExprKind::Lit(Lit::Char(c)), span)) }
            Token::KwTrue => { self.advance(); Ok(Expr::new(ExprKind::Lit(Lit::Bool(true)), span)) }
            Token::KwFalse => { self.advance(); Ok(Expr::new(ExprKind::Lit(Lit::Bool(false)), span)) }

            Token::LowerId(s) => { self.advance(); Ok(Expr::new(ExprKind::Var(s), span)) }
            Token::UpperId(s) => { self.advance(); Ok(Expr::new(ExprKind::Con(s), span)) }

            Token::LParen => {
                self.advance();
                // () is the unit literal
                if self.peek() == &Token::RParen {
                    self.advance();
                    return Ok(Expr::new(ExprKind::Lit(Lit::Unit), span));
                }
                let e = self.parse_expr()?;
                // (a, b) tuple syntax → desugars to {fst = a, snd = b}
                if self.eat(&Token::Comma) {
                    let e2 = self.parse_expr()?;
                    self.expect(&Token::RParen)?;
                    return Ok(Expr::new(ExprKind::Record(vec![
                        ("fst".to_string(), e),
                        ("snd".to_string(), e2),
                    ]), span));
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::new(ExprKind::Paren(Box::new(e)), span))
            }

            Token::LBracket => self.parse_list_expr(span),

            Token::LBrace => {
                self.advance();
                self.skip_newlines();
                if self.peek() == &Token::DotDotDot {
                    // Record update: {...base, field = val, ...}
                    self.advance(); // consume ...
                    let base = self.parse_expr()?;
                    let mut updates = Vec::new();
                    if self.peek() == &Token::Comma {
                        self.advance();
                        self.skip_newlines();
                        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
                            let fname = self.expect_lower_id()?;
                            self.expect(&Token::Assign)?;
                            let val = self.parse_expr()?;
                            updates.push((fname, val));
                            self.eat(&Token::Comma);
                            self.skip_newlines();
                        }
                    }
                    self.expect(&Token::RBrace)?;
                    Ok(Expr::new(ExprKind::RecordUpdate { base: Box::new(base), updates }, span))
                } else {
                    let mut fields = Vec::new();
                    while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
                        let fname = self.expect_lower_id()?;
                        let val = if self.peek() == &Token::Assign {
                            self.advance();
                            self.parse_expr()?
                        } else {
                            // Record punning: {x} → {x = x}
                            Expr::new(ExprKind::Var(fname.clone()), span)
                        };
                        fields.push((fname, val));
                        self.eat(&Token::Comma);
                        self.skip_newlines();
                    }
                    self.expect(&Token::RBrace)?;
                    Ok(Expr::new(ExprKind::Record(fields), span))
                }
            }

            _ => Err(self.error(format!("Expected expression, got {:?}", self.peek()))),
        }
    }

    /// Parse an interpolated string: sequence of StringFragment and InterpStart..InterpEnd.
    /// Desugars to `show` calls joined by `++`.
    ///
    /// `"hello ${name}, ${x + 1} end"` becomes:
    /// `"hello " ++ show name ++ ", " ++ show (x + 1) ++ " end"`
    fn parse_interp_string(&mut self, span: Span) -> PResult<Expr> {
        let mut segments: Vec<Expr> = Vec::new();
        loop {
            match self.peek().clone() {
                Token::StringFragment(s) => {
                    self.advance();
                    if !s.is_empty() {
                        segments.push(Expr::new(ExprKind::Lit(Lit::Str(s)), span));
                    }
                }
                Token::InterpStart => {
                    self.advance();
                    let expr = self.parse_expr()?;
                    self.expect(&Token::InterpEnd)?;
                    // Wrap in `show expr`
                    let show = Expr::new(ExprKind::Var("show".to_string()), span);
                    segments.push(Expr::new(
                        ExprKind::App(Box::new(show), Box::new(expr)),
                        span,
                    ));
                }
                _ => break,
            }
        }
        // Fold segments with ++ (left-associative)
        match segments.len() {
            0 => Ok(Expr::new(ExprKind::Lit(Lit::Str(String::new())), span)),
            1 => Ok(segments.remove(0)),
            _ => {
                let mut result = segments.remove(0);
                for seg in segments {
                    result = Expr::new(
                        ExprKind::BinOp(BinOp::Concat, Box::new(result), Box::new(seg)),
                        span,
                    );
                }
                Ok(result)
            }
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
    /// mix of `name = expr` bindings and standalone effect expressions,
    /// ending with a final expression, then DEDENT.
    /// Standalone effects are desugared to `BinOp::Seq` chains.
    fn parse_block_body(&mut self) -> PResult<Expr> {
        let span = self.peek_span();
        self.skip_newlines();

        enum Stmt { Bind(Binding), Effect(Expr) }
        let mut stmts: Vec<Stmt> = Vec::new();

        loop {
            if self.peek() == &Token::Dedent || self.peek() == &Token::Eof {
                break;
            }

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
                stmts.push(Stmt::Bind(Binding { name, value, span: bspan }));
            } else {
                let expr = self.parse_expr()?;
                self.skip_newlines();
                stmts.push(Stmt::Effect(expr));
            }
        }

        self.eat(&Token::Dedent);

        // Last stmt must be an expression (the block's value)
        if stmts.is_empty() || matches!(stmts.last(), Some(Stmt::Bind(_))) {
            return Err(self.error("Block must end with an expression"));
        }

        // Extract the final expression
        let final_expr = match stmts.pop().unwrap() {
            Stmt::Effect(e) => e,
            Stmt::Bind(_) => unreachable!(),
        };

        // Build AST from remaining stmts (right to left):
        // consecutive Binds → Block; Effects → Seq chain
        let mut result = final_expr;
        let mut pending: Vec<Binding> = Vec::new();

        for stmt in stmts.into_iter().rev() {
            match stmt {
                Stmt::Bind(b) => pending.insert(0, b),
                Stmt::Effect(e) => {
                    if !pending.is_empty() {
                        result = Expr::new(
                            ExprKind::Block(std::mem::take(&mut pending), Box::new(result)),
                            span,
                        );
                    }
                    result = Expr::new(
                        ExprKind::BinOp(BinOp::Seq, Box::new(e), Box::new(result)),
                        span,
                    );
                }
            }
        }

        if !pending.is_empty() {
            result = Expr::new(ExprKind::Block(pending, Box::new(result)), span);
        }

        Ok(result)
    }

    /// Parse where-bindings that appear inline after ternary `:`.
    ///
    /// Handles patterns like:
    /// ```text
    /// ? cond -> expr
    /// : y = x + 1
    ///   y * 2
    /// ```
    fn parse_ternary_else_bindings(&mut self, span: Span) -> PResult<Expr> {
        let mut bindings = Vec::new();

        // Parse inline binding(s): name = expr
        let bspan = self.peek_span();
        let bname = self.expect_lower_id()?;
        self.expect(&Token::Assign)?;
        let value = if self.eat(&Token::Indent) {
            self.parse_block_body()?
        } else {
            self.parse_expr()?
        };
        bindings.push(Binding { name: bname, value, span: bspan });

        // If an INDENT follows, parse remaining bindings + final expression
        if self.eat(&Token::Indent) {
            let body = self.parse_block_body()?;
            Ok(Expr::new(ExprKind::Block(bindings, Box::new(body)), span))
        } else {
            // No indented block — error: we need a final expression after bindings
            Err(self.error("Where-binding in ternary else must be followed by an indented expression"))
        }
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
        } else if self.eat(&Token::LinearArrow) {
            let rhs = self.parse_type()?; // right-associative
            Ok(TypeExpr::new(TypeExprKind::LinearArrow(Box::new(t), Box::new(rhs)), span))
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
            Decl::Func { name, equations, span, doc } => {
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
                result.push(Decl::Func { name, equations, span, doc });
            }
            other => result.push(other),
        }
    }

    result
}

/// Attach doc-comments to a declaration.
fn attach_doc(decl: &mut Decl, doc: Vec<String>) {
    if doc.is_empty() { return; }
    match decl {
        Decl::Func { doc: d, .. } => *d = doc,
        Decl::TypeDef { doc: d, .. } => *d = doc,
        Decl::TraitDecl { doc: d, .. } => *d = doc,
        Decl::TypeSig(_) | Decl::ImplDecl { .. } | Decl::TypeAlias { .. } | Decl::Test { .. } => {}
    }
}
