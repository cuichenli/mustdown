use super::inline_token::InlineToken;
#[derive(Debug)]
pub enum LineToken {
    HeaderToken(HeaderToken),
    Paragraph(Paragraph),
    CodeBlock(CodeBlock),
    Quote(Quote),
}
#[derive(Debug)]
pub struct HeaderToken {
    pub level: usize,
    pub inline_tokens: Vec<InlineToken>,
}
#[derive(Debug)]
pub struct Paragraph {
    pub inline_tokens: Vec<InlineToken>,
}
#[derive(Debug)]
pub struct CodeBlock {
    pub text: String,
}

impl CodeBlock {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}
#[derive(Debug)]
pub struct Quote {
    pub inline_tokens: Vec<InlineToken>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tokenizer;

    #[test]
    fn test_line_scanner_header_token() {
        let tokenizer = Tokenizer::new("## Test");
        let result = tokenizer.scanner();
        let head_token = &result[0];
        let level: usize;
        let inline_token: &Vec<InlineToken>;
        match head_token {
            LineToken::HeaderToken(token) => {
                level = token.level;
                inline_token = &token.inline_tokens;
            }
            _ => panic!(),
        };
        assert_eq!(level, 2 as usize);
        match &inline_token[0] {
            InlineToken::TextToken(t) => assert_eq!(t.text, "Test"),
            _ => panic!(),
        };
    }

    #[test]
    fn test_block_parser() {
        let lines = vec!["this", "is", "a", "test"];
        let tokenizer = Tokenizer::new("");
        let mut tokens: Vec<LineToken> = Vec::new();
        tokenizer.block_parser(&lines, &mut tokens);
        assert_eq!(tokens.len(), 1);
        let token = tokens.first().unwrap();
        if let LineToken::CodeBlock(token) = token {
            assert_eq!(token.text, "this\nis\na\ntest");
        } else {
            panic!();
        }
    }

    #[test]
    fn test_scanner_with_code_block() {
        let text = "```\n\
                    this is a test \n\
                    ```";
        let tokenizer = Tokenizer::new(text);
        let result = tokenizer.scanner();
        assert_eq!(result.len(), 1);
        let token = result.first().unwrap();
        if let LineToken::CodeBlock(token) = token {
            assert_eq!(token.text, "this is a test ");
        } else {
            panic!();
        }
    }

    #[test]
    fn test_scanner_with_code_block_and_paragraph() {
        let text = "```\n\
                    this is a test \n\
                    ```\n\
                    this is another test";
        let tokenizer = Tokenizer::new(text);
        let result = tokenizer.scanner();
        assert_eq!(result.len(), 2);
        let token = result.first().unwrap();
        if let LineToken::CodeBlock(token) = token {
            assert_eq!(token.text, "this is a test ");
        } else {
            panic!();
        }

        let token = result.last().unwrap();
        if let LineToken::Paragraph(token) = token {
            assert_eq!(token.inline_tokens.len(), 1);
            if let InlineToken::TextToken(token) = token.inline_tokens.first().unwrap() {
                assert_eq!(token.text, "this is another test")
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_quote_with_multiple_lines() {
        let text = ">this is  \na quote";
        let tokenizer = Tokenizer::new(text);
        let result = tokenizer.scanner();
        assert_eq!(result.len(), 1);
        if let LineToken::Quote(token) = &result[0] {
            let inline_tokens = &token.inline_tokens;
            assert_eq!(inline_tokens.len(), 3);
            if let InlineToken::TextToken(token) = &inline_tokens[0] {
                assert_eq!(token.text, "this is  ");
            } else {
                panic!();
            }
            if let InlineToken::BreakToken = &inline_tokens[1] {
                assert!(true);
            } else {
                panic!();
            }
            if let InlineToken::TextToken(token) = &inline_tokens[2] {
                assert_eq!(token.text, "a quote");
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_quote_with_single_line() {
        let text = ">this is\n";
        let tokenizer = Tokenizer::new(text);
        let result = tokenizer.scanner();
        assert_eq!(result.len(), 1);
        if let LineToken::Quote(token) = &result[0] {
            let inline_tokens = &token.inline_tokens;
            assert_eq!(inline_tokens.len(), 1);
            if let InlineToken::TextToken(token) = &inline_tokens[0] {
                assert_eq!(token.text, "this is");
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_quote_with_inline_tokens_with_single_line() {
        let text = ">this is*a bold*\n";
        let tokenizer = Tokenizer::new(text);
        let result = tokenizer.scanner();
        assert_eq!(result.len(), 1);
        if let LineToken::Quote(token) = &result[0] {
            let inline_tokens = &token.inline_tokens;
            assert_eq!(inline_tokens.len(), 2);
            if let InlineToken::TextToken(token) = &inline_tokens[0] {
                assert_eq!(token.text, "this is");
            } else {
                panic!();
            }
            if let InlineToken::SpecialToken(token) = &inline_tokens[1] {
                assert_eq!(token.token, '*');
                assert_eq!(token.inline_tokens.len(), 1);
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "a bold");
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_single_line_quote_with_lines_surrounding() {
        let text = "first paragraph\n>a quote\nsecond paragraph";
        let tokenizer = Tokenizer::new(text);
        let result = tokenizer.scanner();
        assert_eq!(result.len(), 3);
        if let LineToken::Quote(token) = &result[1] {
            assert_eq!(token.inline_tokens.len(), 1);
            if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                assert_eq!(token.text, "a quote");
            }
        }

        if let LineToken::Paragraph(token) = &result[0] {
            assert_eq!(token.inline_tokens.len(), 1);
            if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                assert_eq!(token.text, "first paragraph");
            }
        } else {
            panic!();
        }
        if let LineToken::Paragraph(token) = &result[2] {
            assert_eq!(token.inline_tokens.len(), 1);
            if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                assert_eq!(token.text, "second paragraph");
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_multiple_lines_quote_with_lines_surrounding() {
        let text = "first paragraph\n>a quote  \nanother quote\nsecond paragraph";
        let tokenizer = Tokenizer::new(text);
        let result = tokenizer.scanner();
        assert_eq!(result.len(), 3);
        if let LineToken::Quote(token) = &result[1] {
            assert_eq!(token.inline_tokens.len(), 3);
            if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                assert_eq!(token.text, "a quote  ");
            }
            if let InlineToken::TextToken(token) = &token.inline_tokens[2] {
                assert_eq!(token.text, "another quote");
            }
            if let InlineToken::BreakToken = &token.inline_tokens[1] {
                assert!(true);
            } else {
                panic!();
            }
        }

        if let LineToken::Paragraph(token) = &result[0] {
            assert_eq!(token.inline_tokens.len(), 1);
            if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                assert_eq!(token.text, "first paragraph");
            }
        } else {
            panic!();
        }
        if let LineToken::Paragraph(token) = &result[2] {
            assert_eq!(token.inline_tokens.len(), 1);
            if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                assert_eq!(token.text, "second paragraph");
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_quote_with_empty_quote() {
        let text = ">";
        let tokenizer = Tokenizer::new(text);
        let result = tokenizer.scanner();
        assert_eq!(result.len(), 1);
        if let LineToken::Quote(token) = &result[0] {
            assert_eq!(token.inline_tokens.len(), 0);
        } else {
            panic!();
        }
    }
}
