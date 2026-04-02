use crate::*;

fn toks(src: &str) -> Vec<Token> {
    lex_tokens(src).expect("lex failed")
}

fn raw(src: &str) -> Vec<Token> {
    lex_raw(src).expect("lex_raw failed")
        .into_iter().map(|st| st.token).collect()
}

// ── Literals ──────────────────────────────────────────────

#[test]
fn integers() {
    assert_eq!(raw("0"), vec![Token::Int(0), Token::Eof]);
    assert_eq!(raw("42"), vec![Token::Int(42), Token::Eof]);
    assert_eq!(raw("1000000"), vec![Token::Int(1000000), Token::Eof]);
}

#[test]
fn floats() {
    assert_eq!(raw("3.14"), vec![Token::Float(3.14), Token::Eof]);
    assert_eq!(raw("0.5"), vec![Token::Float(0.5), Token::Eof]);
}

#[test]
fn strings() {
    assert_eq!(raw("\"hello\""), vec![Token::Str("hello".into()), Token::Eof]);
    assert_eq!(raw("\"a\\nb\""), vec![Token::Str("a\nb".into()), Token::Eof]);
}

#[test]
fn chars() {
    assert_eq!(raw("'a'"), vec![Token::Char('a'), Token::Eof]);
    assert_eq!(raw("'\\n'"), vec![Token::Char('\n'), Token::Eof]);
}

#[test]
fn booleans() {
    assert_eq!(raw("true"), vec![Token::KwTrue, Token::Eof]);
    assert_eq!(raw("false"), vec![Token::KwFalse, Token::Eof]);
}

// ── Identifiers ───────────────────────────────────────────

#[test]
fn lower_ids() {
    assert_eq!(raw("foo"), vec![Token::LowerId("foo".into()), Token::Eof]);
    assert_eq!(raw("myFunc"), vec![Token::LowerId("myFunc".into()), Token::Eof]);
    assert_eq!(raw("x1"), vec![Token::LowerId("x1".into()), Token::Eof]);
}

#[test]
fn upper_ids() {
    assert_eq!(raw("Int"), vec![Token::UpperId("Int".into()), Token::Eof]);
    assert_eq!(raw("Maybe"), vec![Token::UpperId("Maybe".into()), Token::Eof]);
}

// ── Keywords ──────────────────────────────────────────────

#[test]
fn keywords() {
    assert_eq!(raw("mod"), vec![Token::KwMod, Token::Eof]);
    assert_eq!(raw("use"), vec![Token::KwUse, Token::Eof]);
    assert_eq!(raw("trait"), vec![Token::KwTrait, Token::Eof]);
    assert_eq!(raw("impl"), vec![Token::KwImpl, Token::Eof]);
    assert_eq!(raw("lazy"), vec![Token::KwLazy, Token::Eof]);
}

#[test]
fn keyword_prefix_is_identifier() {
    assert_eq!(raw("module"), vec![Token::LowerId("module".into()), Token::Eof]);
    assert_eq!(raw("implementation"), vec![Token::LowerId("implementation".into()), Token::Eof]);
}

// ── Two-char Operators ────────────────────────────────────

#[test]
fn two_char_ops() {
    assert_eq!(raw("->"), vec![Token::Arrow, Token::Eof]);
    assert_eq!(raw("<-"), vec![Token::BackArrow, Token::Eof]);
    assert_eq!(raw("|>"), vec![Token::Pipe, Token::Eof]);
    assert_eq!(raw(">>"), vec![Token::Compose, Token::Eof]);
    assert_eq!(raw("=="), vec![Token::Eq, Token::Eof]);
    assert_eq!(raw("!="), vec![Token::Neq, Token::Eof]);
    assert_eq!(raw("<="), vec![Token::Lte, Token::Eof]);
    assert_eq!(raw(">="), vec![Token::Gte, Token::Eof]);
    assert_eq!(raw("&&"), vec![Token::And, Token::Eof]);
    assert_eq!(raw("||"), vec![Token::Or, Token::Eof]);
    assert_eq!(raw("++"), vec![Token::Concat, Token::Eof]);
    assert_eq!(raw(".."), vec![Token::DotDot, Token::Eof]);
}

// ── Single-char Operators ─────────────────────────────────

#[test]
fn single_char_ops() {
    assert_eq!(raw("+"), vec![Token::Plus, Token::Eof]);
    assert_eq!(raw("-"), vec![Token::Minus, Token::Eof]);
    assert_eq!(raw("*"), vec![Token::Star, Token::Eof]);
    assert_eq!(raw("/"), vec![Token::Slash, Token::Eof]);
    assert_eq!(raw("%"), vec![Token::Percent, Token::Eof]);
    assert_eq!(raw("."), vec![Token::Dot, Token::Eof]);
    assert_eq!(raw(":"), vec![Token::Colon, Token::Eof]);
    assert_eq!(raw("="), vec![Token::Assign, Token::Eof]);
    assert_eq!(raw("@"), vec![Token::At, Token::Eof]);
    assert_eq!(raw("|"), vec![Token::Bar, Token::Eof]);
    assert_eq!(raw("?"), vec![Token::Question, Token::Eof]);
    assert_eq!(raw("\\"), vec![Token::Backslash, Token::Eof]);
    assert_eq!(raw("_"), vec![Token::Underscore, Token::Eof]);
    assert_eq!(raw(","), vec![Token::Comma, Token::Eof]);
}

#[test]
fn delimiters() {
    assert_eq!(raw("()"), vec![Token::LParen, Token::RParen, Token::Eof]);
    assert_eq!(raw("[]"), vec![Token::LBracket, Token::RBracket, Token::Eof]);
}

// ── Compound Expressions ─────────────────────────────────

#[test]
fn function_def() {
    assert_eq!(raw("fac 0 = 1"), vec![
        Token::LowerId("fac".into()), Token::Int(0), Token::Assign, Token::Int(1), Token::Eof,
    ]);
}

#[test]
fn lambda() {
    assert_eq!(raw("\\x -> x + 1"), vec![
        Token::Backslash, Token::LowerId("x".into()), Token::Arrow,
        Token::LowerId("x".into()), Token::Plus, Token::Int(1), Token::Eof,
    ]);
}

#[test]
fn conditional() {
    assert_eq!(raw("? x < 0 -> y : z"), vec![
        Token::Question, Token::LowerId("x".into()), Token::Lt, Token::Int(0),
        Token::Arrow, Token::LowerId("y".into()), Token::Colon, Token::LowerId("z".into()),
        Token::Eof,
    ]);
}

#[test]
fn pipe_chain() {
    assert_eq!(raw("xs |> filter even |> sum"), vec![
        Token::LowerId("xs".into()), Token::Pipe,
        Token::LowerId("filter".into()), Token::LowerId("even".into()), Token::Pipe,
        Token::LowerId("sum".into()), Token::Eof,
    ]);
}

#[test]
fn list_literal() {
    assert_eq!(raw("[1 2 3]"), vec![
        Token::LBracket, Token::Int(1), Token::Int(2), Token::Int(3), Token::RBracket, Token::Eof,
    ]);
}

#[test]
fn list_comprehension() {
    assert_eq!(raw("[x | x <- xs , x > 0]"), vec![
        Token::LBracket, Token::LowerId("x".into()), Token::Bar,
        Token::LowerId("x".into()), Token::BackArrow, Token::LowerId("xs".into()),
        Token::Comma, Token::LowerId("x".into()), Token::Gt, Token::Int(0),
        Token::RBracket, Token::Eof,
    ]);
}

#[test]
fn type_signature() {
    assert_eq!(raw("add : Int Int -> Int"), vec![
        Token::LowerId("add".into()), Token::Colon,
        Token::UpperId("Int".into()), Token::UpperId("Int".into()),
        Token::Arrow, Token::UpperId("Int".into()), Token::Eof,
    ]);
}

#[test]
fn adt_definition() {
    assert_eq!(raw("Maybe a = Just a | None"), vec![
        Token::UpperId("Maybe".into()), Token::LowerId("a".into()), Token::Assign,
        Token::UpperId("Just".into()), Token::LowerId("a".into()), Token::Bar,
        Token::UpperId("None".into()), Token::Eof,
    ]);
}

// ── Layout / Indentation ─────────────────────────────────

#[test]
fn layout_simple_block() {
    let t = toks("result =\n  a = 10\n  a");
    assert!(t.contains(&Token::Indent));
    assert!(t.contains(&Token::Dedent));
}

#[test]
fn layout_no_indent() {
    let t = toks("x = 1\ny = 2");
    assert!(!t.contains(&Token::Indent));
    assert!(t.contains(&Token::Newline));
}

#[test]
fn layout_nested() {
    let t = toks("f x =\n  g y =\n    y + 1\n  g x");
    let indents = t.iter().filter(|t| **t == Token::Indent).count();
    let dedents = t.iter().filter(|t| **t == Token::Dedent).count();
    assert_eq!(indents, 2);
    assert_eq!(dedents, 2);
}

#[test]
fn layout_multi_dedent() {
    let t = toks("f =\n  g =\n    42\nx = 1");
    let dedents = t.iter().filter(|t| **t == Token::Dedent).count();
    assert_eq!(dedents, 2);
}

// ── Full Programs ─────────────────────────────────────────

#[test]
fn full_factorial() {
    let t = toks("fac 0 = 1\nfac n = n * fac (n - 1)");
    assert!(t.last() == Some(&Token::Eof));
    assert!(t.contains(&Token::LowerId("fac".into())));
    assert!(t.contains(&Token::Star));
}

#[test]
fn full_quicksort() {
    let src = "qsort [] = []\nqsort (p:xs) = qsort lo ++ [p] ++ qsort hi\n  lo = [x | x <- xs , x <= p]\n  hi = [x | x <- xs , x > p]";
    assert!(lex(src).is_ok(), "Failed to lex quicksort");
}

#[test]
fn full_fizzbuzz() {
    let src = "fizzbuzz n =\n  ? n % 15 == 0 -> \"FizzBuzz\"\n  : ? n % 3 == 0 -> \"Fizz\"\n  : ? n % 5 == 0 -> \"Buzz\"\n  : show n";
    assert!(lex(src).is_ok(), "Failed to lex fizzbuzz");
}

// ── Module Syntax ─────────────────────────────────────────

#[test]
fn mod_use_sequence() {
    // Tokenises a minimal module program without errors
    let result = lex("mod Math\n  square x = x * x\n\nuse Math (square)\n\nmain = square 5");
    assert!(result.is_ok(), "Expected lex to succeed for mod/use program");
    let tokens = result.unwrap();
    // Should contain KwMod and KwUse
    assert!(tokens.iter().any(|st| st.token == Token::KwMod));
    assert!(tokens.iter().any(|st| st.token == Token::KwUse));
}

// ── Error Cases ──────────────────────────────────────────

#[test]
fn error_unterminated_string() {
    assert!(lex_raw("\"hello").is_err());
}

#[test]
fn error_unexpected_char() {
    assert!(lex_raw("~").is_err());
}

#[test]
fn record_braces() {
    assert_eq!(raw("{name = 42}"), vec![
        Token::LBrace,
        Token::LowerId("name".into()),
        Token::Assign,
        Token::Int(42),
        Token::RBrace,
        Token::Eof,
    ]);
}
