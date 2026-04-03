// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

use crate::token::*;

/// The layout pass: takes raw tokens from the scanner and inserts
/// INDENT / DEDENT / NEWLINE tokens based on indentation levels.
///
/// Algorithm (Python-style offside rule):
/// - Track a stack of indentation levels, starting with [0]
/// - At each line start, compare current indentation with stack top
/// - If deeper: push level, emit INDENT
/// - If same: emit NEWLINE (statement separator)
/// - If shallower: pop until matching, emit DEDENT for each pop
pub fn apply_layout(raw: Vec<SpannedToken>) -> Vec<SpannedToken> {
    let mut out: Vec<SpannedToken> = Vec::new();
    let mut indent_stack: Vec<u32> = vec![0];
    let mut i = 0;
    let tokens = &raw;
    let len = tokens.len();

    // skip leading newlines
    while i < len && tokens[i].token == Token::Newline {
        i += 1;
    }

    while i < len {
        let st = &tokens[i];

        if st.token == Token::Eof {
            // emit dedents for remaining levels
            while indent_stack.len() > 1 {
                indent_stack.pop();
                out.push(SpannedToken {
                    token: Token::Dedent,
                    span: st.span,
                });
            }
            out.push(st.clone());
            break;
        }

        if st.token == Token::Newline {
            // skip consecutive newlines
            while i < len && tokens[i].token == Token::Newline {
                i += 1;
            }
            if i >= len || tokens[i].token == Token::Eof {
                continue;
            }

            // determine indentation level of next non-newline token
            let next = &tokens[i];
            let col = next.span.start.col - 1; // 0-indexed indent level
            let top = *indent_stack.last().unwrap();

            if col > top {
                // deeper: emit INDENT
                indent_stack.push(col);
                out.push(SpannedToken {
                    token: Token::Indent,
                    span: next.span,
                });
            } else if col == top {
                // same level: emit NEWLINE as statement separator
                // but only if we have prior content
                if !out.is_empty() {
                    let last = out.last().map(|t| &t.token);
                    // avoid double newlines or newline after indent
                    if !matches!(last, Some(Token::Newline) | Some(Token::Indent) | None) {
                        out.push(SpannedToken {
                            token: Token::Newline,
                            span: next.span,
                        });
                    }
                }
            } else {
                // shallower: emit DEDENTs
                while indent_stack.len() > 1 && *indent_stack.last().unwrap() > col {
                    indent_stack.pop();
                    out.push(SpannedToken {
                        token: Token::Dedent,
                        span: next.span,
                    });
                }
                // emit newline after dedents
                if !out.is_empty() {
                    let last = out.last().map(|t| &t.token);
                    if !matches!(last, Some(Token::Newline) | Some(Token::Dedent) | None) {
                        out.push(SpannedToken {
                            token: Token::Newline,
                            span: next.span,
                        });
                    }
                }
            }
            // don't advance i — we'll process the non-newline token next iteration
            continue;
        }

        // regular token: just emit
        out.push(st.clone());
        i += 1;
    }

    // ensure EOF is present
    if out.is_empty() || out.last().map(|t| &t.token) != Some(&Token::Eof) {
        out.push(SpannedToken { token: Token::Eof, span: Span::dummy() });
    }

    out
}
