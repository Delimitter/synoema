use synoema_lexer::*;

// Helper: lex and extract just the token variants, excluding Eof
fn toks(src: &str) -> Vec<Token> {
    lex_tokens(src).unwrap().into_iter().filter(|t| *t != Token::Eof).collect()
}

// ═══════════════════════════════════════════
//  Literals
// ═══════════════════════════════════════════

#[test]
fn lex_integer() {
    assert_eq!(toks("42"), vec![Token::Int(42)]);
}

#[test]
fn lex_zero() {
    assert_eq!(toks("0"), vec![Token::Int(0)]);
}

#[test]
fn lex_large_int() {
    assert_eq!(toks("9999999"), vec![Token::Int(9999999)]);
}

#[test]
fn lex_float() {
    assert_eq!(toks("3.14"), vec![Token::Float(3.14)]);
}

#[test]
fn lex_string() {
    assert_eq!(toks(r#""hello""#), vec![Token::Str("hello".into())]);
}

#[test]
fn lex_string_escape() {
    assert_eq!(toks(r#""a\nb""#), vec![Token::Str("a\nb".into())]);
}

#[test]
fn lex_char() {
    assert_eq!(toks("'x'"), vec![Token::Char('x')]);
}

#[test]
fn lex_bool_true() {
    assert_eq!(toks("true"), vec![Token::KwTrue]);
}

#[test]
fn lex_bool_false() {
    assert_eq!(toks("false"), vec![Token::KwFalse]);
}

// ═══════════════════════════════════════════
//  Identifiers and keywords
// ═══════════════════════════════════════════

#[test]
fn lex_lower_id() {
    assert_eq!(toks("foo"), vec![Token::LowerId("foo".into())]);
}

#[test]
fn lex_upper_id() {
    assert_eq!(toks("Maybe"), vec![Token::UpperId("Maybe".into())]);
}

#[test]
fn lex_id_with_numbers() {
    assert_eq!(toks("x1"), vec![Token::LowerId("x1".into())]);
}

#[test]
fn lex_id_with_underscore() {
    assert_eq!(toks("my_func"), vec![Token::LowerId("my_func".into())]);
}

#[test]
fn lex_keywords() {
    assert_eq!(toks("mod"), vec![Token::KwMod]);
    assert_eq!(toks("use"), vec![Token::KwUse]);
    assert_eq!(toks("trait"), vec![Token::KwTrait]);
    assert_eq!(toks("impl"), vec![Token::KwImpl]);
    assert_eq!(toks("lazy"), vec![Token::KwLazy]);
}

#[test]
fn lex_keyword_prefix_is_id() {
    // "module" is not a keyword, "mod" is
    assert_eq!(toks("module"), vec![Token::LowerId("module".into())]);
    assert_eq!(toks("implementation"), vec![Token::LowerId("implementation".into())]);
}

#[test]
fn lex_wildcard() {
    assert_eq!(toks("_"), vec![Token::Underscore]);
}

// ═══════════════════════════════════════════
//  Operators (BPE-aligned)
// ═══════════════════════════════════════════

#[test]
fn lex_arrow() {
    assert_eq!(toks("->"), vec![Token::Arrow]);
}

#[test]
fn lex_back_arrow() {
    assert_eq!(toks("<-"), vec![Token::BackArrow]);
}

#[test]
fn lex_pipe() {
    assert_eq!(toks("|>"), vec![Token::Pipe]);
}

#[test]
fn lex_concat() {
    assert_eq!(toks("++"), vec![Token::Concat]);
}

#[test]
fn lex_compose() {
    assert_eq!(toks(">>"), vec![Token::Compose]);
}

#[test]
fn lex_comparison_ops() {
    assert_eq!(toks("=="), vec![Token::Eq]);
    assert_eq!(toks("!="), vec![Token::Neq]);
    assert_eq!(toks("<"),  vec![Token::Lt]);
    assert_eq!(toks(">"),  vec![Token::Gt]);
    assert_eq!(toks("<="), vec![Token::Lte]);
    assert_eq!(toks(">="), vec![Token::Gte]);
}

#[test]
fn lex_logical_ops() {
    assert_eq!(toks("&&"), vec![Token::And]);
    assert_eq!(toks("||"), vec![Token::Or]);
}

#[test]
fn lex_arithmetic_ops() {
    assert_eq!(toks("+"), vec![Token::Plus]);
    assert_eq!(toks("-"), vec![Token::Minus]);
    assert_eq!(toks("*"), vec![Token::Star]);
    assert_eq!(toks("/"), vec![Token::Slash]);
    assert_eq!(toks("%"), vec![Token::Percent]);
}

#[test]
fn lex_special_ops() {
    assert_eq!(toks("?"), vec![Token::Question]);
    assert_eq!(toks(":"), vec![Token::Colon]);
    assert_eq!(toks("."), vec![Token::Dot]);
    assert_eq!(toks(".."), vec![Token::DotDot]);
    assert_eq!(toks("="), vec![Token::Assign]);
    assert_eq!(toks("@"), vec![Token::At]);
    assert_eq!(toks("|"), vec![Token::Bar]);
    assert_eq!(toks("\\"), vec![Token::Backslash]);
    assert_eq!(toks(","), vec![Token::Comma]);
}

#[test]
fn lex_delimiters() {
    assert_eq!(toks("()"), vec![Token::LParen, Token::RParen]);
    assert_eq!(toks("[]"), vec![Token::LBracket, Token::RBracket]);
}

// ═══════════════════════════════════════════
//  Operator disambiguation
// ═══════════════════════════════════════════

#[test]
fn lex_minus_vs_arrow() {
    // "->" is arrow, "-" alone is minus
    assert_eq!(toks("- >"), vec![Token::Minus, Token::Gt]);
    assert_eq!(toks("->"), vec![Token::Arrow]);
}

#[test]
fn lex_lt_vs_backarrow() {
    assert_eq!(toks("< -"), vec![Token::Lt, Token::Minus]);
    assert_eq!(toks("<-"), vec![Token::BackArrow]);
}

#[test]
fn lex_bar_vs_pipe_vs_or() {
    assert_eq!(toks("|"), vec![Token::Bar]);
    assert_eq!(toks("|>"), vec![Token::Pipe]);
    assert_eq!(toks("||"), vec![Token::Or]);
}

#[test]
fn lex_plus_vs_concat() {
    assert_eq!(toks("+"), vec![Token::Plus]);
    assert_eq!(toks("++"), vec![Token::Concat]);
}

#[test]
fn lex_eq_vs_assign() {
    assert_eq!(toks("="), vec![Token::Assign]);
    assert_eq!(toks("=="), vec![Token::Eq]);
}

// ═══════════════════════════════════════════
//  Comments
// ═══════════════════════════════════════════

#[test]
fn lex_comment_ignored() {
    let t = toks("x -- this is a comment");
    assert_eq!(t, vec![Token::LowerId("x".into())]);
}

#[test]
fn lex_only_comment() {
    let t = toks("-- just a comment");
    // should produce empty (comment becomes newline, filtered)
    assert!(t.is_empty() || t.iter().all(|t| matches!(t, Token::Newline)));
}

// ═══════════════════════════════════════════
//  Multi-token expressions
// ═══════════════════════════════════════════

#[test]
fn lex_simple_function_def() {
    let t = toks("add x y = x + y");
    assert_eq!(t, vec![
        Token::LowerId("add".into()),
        Token::LowerId("x".into()),
        Token::LowerId("y".into()),
        Token::Assign,
        Token::LowerId("x".into()),
        Token::Plus,
        Token::LowerId("y".into()),
    ]);
}

#[test]
fn lex_pattern_match_def() {
    let t = toks("fac 0 = 1");
    assert_eq!(t, vec![
        Token::LowerId("fac".into()),
        Token::Int(0),
        Token::Assign,
        Token::Int(1),
    ]);
}

#[test]
fn lex_conditional() {
    let t = toks("? x > 0 -> x : 0");
    assert_eq!(t, vec![
        Token::Question,
        Token::LowerId("x".into()),
        Token::Gt,
        Token::Int(0),
        Token::Arrow,
        Token::LowerId("x".into()),
        Token::Colon,
        Token::Int(0),
    ]);
}

#[test]
fn lex_lambda() {
    let t = toks("\\x -> x + 1");
    assert_eq!(t, vec![
        Token::Backslash,
        Token::LowerId("x".into()),
        Token::Arrow,
        Token::LowerId("x".into()),
        Token::Plus,
        Token::Int(1),
    ]);
}

#[test]
fn lex_list() {
    let t = toks("[1 2 3]");
    assert_eq!(t, vec![
        Token::LBracket,
        Token::Int(1),
        Token::Int(2),
        Token::Int(3),
        Token::RBracket,
    ]);
}

#[test]
fn lex_list_comprehension() {
    let t = toks("[x | x <- xs , x > 0]");
    assert_eq!(t, vec![
        Token::LBracket,
        Token::LowerId("x".into()),
        Token::Bar,
        Token::LowerId("x".into()),
        Token::BackArrow,
        Token::LowerId("xs".into()),
        Token::Comma,
        Token::LowerId("x".into()),
        Token::Gt,
        Token::Int(0),
        Token::RBracket,
    ]);
}

#[test]
fn lex_pipe_chain() {
    let t = toks("xs |> filter even |> sum");
    assert_eq!(t, vec![
        Token::LowerId("xs".into()),
        Token::Pipe,
        Token::LowerId("filter".into()),
        Token::LowerId("even".into()),
        Token::Pipe,
        Token::LowerId("sum".into()),
    ]);
}

#[test]
fn lex_adt_definition() {
    let t = toks("Maybe a = Just a | None");
    assert_eq!(t, vec![
        Token::UpperId("Maybe".into()),
        Token::LowerId("a".into()),
        Token::Assign,
        Token::UpperId("Just".into()),
        Token::LowerId("a".into()),
        Token::Bar,
        Token::UpperId("None".into()),
    ]);
}

#[test]
fn lex_type_signature() {
    let t = toks("add : Int Int -> Int");
    assert_eq!(t, vec![
        Token::LowerId("add".into()),
        Token::Colon,
        Token::UpperId("Int".into()),
        Token::UpperId("Int".into()),
        Token::Arrow,
        Token::UpperId("Int".into()),
    ]);
}

// ═══════════════════════════════════════════
//  Layout (INDENT / DEDENT)
// ═══════════════════════════════════════════

#[test]
fn layout_simple_block() {
    let t = toks("result =\n  a = 10\n  a");
    // result = INDENT a = 10 NEWLINE a DEDENT
    assert!(t.contains(&Token::Indent));
    assert!(t.contains(&Token::Dedent));
}

#[test]
fn layout_no_indent_single_line() {
    let t = toks("x = 1");
    assert!(!t.contains(&Token::Indent));
    assert!(!t.contains(&Token::Dedent));
}

#[test]
fn layout_two_top_level_defs() {
    let t = toks("f x = x\ng y = y");
    // should have a Newline between the two defs
    assert!(t.contains(&Token::Newline));
}

#[test]
fn layout_nested_blocks() {
    let src = "outer =\n  inner =\n    42\n  inner";
    let t = toks(src);
    let indent_count = t.iter().filter(|t| **t == Token::Indent).count();
    let dedent_count = t.iter().filter(|t| **t == Token::Dedent).count();
    assert_eq!(indent_count, 2, "should have 2 indents");
    assert_eq!(dedent_count, 2, "should have 2 dedents");
}

// ═══════════════════════════════════════════
//  Error cases
// ═══════════════════════════════════════════

#[test]
fn lex_error_unterminated_string() {
    let result = lex("\"hello");
    assert!(result.is_err());
}

#[test]
fn lex_error_unexpected_char() {
    let result = lex("~");
    assert!(result.is_err());
}

#[test]
fn lex_error_lone_bang() {
    let result = lex("!");
    assert!(result.is_err());
}

// ═══════════════════════════════════════════
//  Full program (from spec §7)
// ═══════════════════════════════════════════

#[test]
fn lex_factorial_program() {
    let src = "fac 0 = 1\nfac n = n * fac (n - 1)";
    let t = toks(src);
    // should parse without errors and contain expected tokens
    assert!(t.contains(&Token::LowerId("fac".into())));
    assert!(t.contains(&Token::Star));
    assert!(t.contains(&Token::LParen));
    assert!(t.contains(&Token::RParen));
}

#[test]
fn lex_quicksort_program() {
    let src = "qsort [] = []\nqsort (p:xs) = qsort lo ++ [p] ++ qsort hi\n  lo = [x | x <- xs , x <= p]\n  hi = [x | x <- xs , x > p]";
    let result = lex(src);
    assert!(result.is_ok(), "quicksort should lex without errors");
    let tokens = result.unwrap();
    let t: Vec<_> = tokens.iter().map(|s| &s.token).collect();
    assert!(t.contains(&&Token::Concat));
    assert!(t.contains(&&Token::BackArrow));
    assert!(t.contains(&&Token::Lte));
    assert!(t.contains(&&Token::Indent));
    assert!(t.contains(&&Token::Dedent));
}

