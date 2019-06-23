use super::inline_token::InlineToken;
#[derive(Debug)]
pub enum LineToken {
    HeaderToken(HeaderToken),
    Paragraph(Paragraph),
    CodeBlock(CodeBlock),
    Quote(Quote),
    OrderedListBlock(OrderedListBlock),
    UnorderedListBlock(UnorderedListBlock),
    OrderedList(OrderedList),
    UnorderedList(UnorderedList),
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

#[derive(Debug)]
pub struct OrderedListBlock {
    pub lists: Vec<LineToken>,
}

impl OrderedListBlock {
    pub fn new(token: LineToken) -> Self {
        if let LineToken::OrderedList(_) = token {
            Self {
                lists: vec![token],
            }
        } else {
            panic!()
        }
    }

    pub fn push(&mut self, token: LineToken) {
        match token {
            LineToken::OrderedList(_) => self.lists.push(token),
            _ => panic!()
        }
    }
}

#[derive(Debug)]
pub struct UnorderedListBlock {
    pub lists: Vec<LineToken>,
}

impl UnorderedListBlock {
    pub fn new(token: LineToken) -> Self {
        if let LineToken::UnorderedList(_) = token {
            Self {
                lists: vec![token],
            }
        } else {
            panic!()
        }
    }

    pub fn get_symbol(&self) -> &char {
        let first = self.lists.first().unwrap();
        if let LineToken::UnorderedList(t) = first {
            &t.symbol
        } else {
            panic!()
        }
    }

    pub fn push(&mut self, token: LineToken) {
        match token {
            LineToken::UnorderedList(_) => self.lists.push(token),
            _ => panic!()
        }
    }
}

#[derive(Debug)]
pub struct OrderedList {
    pub inline_tokens: Vec<InlineToken>,
}

impl OrderedList {
    pub fn new(inline_tokens: Vec<InlineToken>) -> Self {
        Self {
            inline_tokens,
        }
    }
}

#[derive(Debug)]
pub struct UnorderedList {
    pub inline_tokens: Vec<InlineToken>,
    pub symbol: char
}

impl UnorderedList {
    pub fn new(symbol: char, inline_tokens: Vec<InlineToken>) -> Self {
        Self {
            symbol,
            inline_tokens
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::super::inline_token::tests::{assert_special_token_group, assert_text_token};
    use super::*;
    use crate::Tokenizer;

    #[test]
    fn test_line_scanner_header_token() {
        let result = Tokenizer::scanner("## Test");
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
        // let tokenizer = Tokenizer::scanner("");
        let mut tokens: Vec<LineToken> = Vec::new();
        Tokenizer::block_parser(&lines, &mut tokens);
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
        let result = Tokenizer::scanner(text);
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
        let result = Tokenizer::scanner(text);
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
        let result = Tokenizer::scanner(text);
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
        let result = Tokenizer::scanner(text);
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
        let result = Tokenizer::scanner(text);
        assert_eq!(result.len(), 1);
        if let LineToken::Quote(token) = &result[0] {
            let inline_tokens = &token.inline_tokens;
            assert_eq!(inline_tokens.len(), 2);
            assert_text_token(&inline_tokens[0], "this is");
            assert_special_token_group(&inline_tokens[1], "a bold", '*');
        } else {
            panic!();
        }
    }

    #[test]
    fn test_single_line_quote_with_lines_surrounding() {
        let text = "first paragraph\n>a quote\nsecond paragraph";
        let result = Tokenizer::scanner(text);
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
        let result = Tokenizer::scanner(text);
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
        let result = Tokenizer::scanner(text);
        assert_eq!(result.len(), 1);
        if let LineToken::Quote(token) = &result[0] {
            assert_eq!(token.inline_tokens.len(), 0);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_ordered_list() {
        let text = "1. this\n2. is\n3. a\n4. test";
        let result = Tokenizer::scanner(text);
        assert_eq!(result.len(), 1);
        if let LineToken::OrderedListBlock(token) = &result[0] {
            let list = &token.lists;
            assert_eq!(list.len(), 4);
            if let LineToken::OrderedList(token) = &list[0] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "this");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::OrderedList(token) = &list[1] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "is");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::OrderedList(token) = &list[2] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "a");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::OrderedList(token) = &list[3] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "test");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_unordered_list() {
        let text = "- this\n- is\n- a\n- test";
        let result = Tokenizer::scanner(text);
        assert_eq!(result.len(), 1);
        if let LineToken::UnorderedListBlock(token) = &result[0] {
            let list = &token.lists;
            assert_eq!(list.len(), 4);
            if let LineToken::UnorderedList(token) = &list[0] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "this");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::UnorderedList(token) = &list[1] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "is");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::UnorderedList(token) = &list[2] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "a");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::UnorderedList(token) = &list[3] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "test");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_unordered_list_with_lines_surrounded() {
        let text = "a simple line\n- a list\n- another\ntest";
        let result = Tokenizer::scanner(text);
        assert_eq!(result.len(), 3);
        if let LineToken::Paragraph(token) = &result[0] {
            let tokens = &token.inline_tokens;
            assert_eq!(tokens.len(), 1);
            if let InlineToken::TextToken(token) = &tokens[0] {
                assert_eq!(token.text, "a simple line");
            } else {
                panic!();
            }
        } else {
            panic!();
        }

        if let LineToken::UnorderedListBlock(token) = &result[1] {
            let tokens = &token.lists;
            assert_eq!(tokens.len(), 2);
            if let LineToken::UnorderedList(token) = &tokens[0] {
                let list = &token.inline_tokens;
                assert_eq!(list.len(), 1);
                if let InlineToken::TextToken(token) = &list[0] {
                    assert_eq!(token.text, "a list");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::UnorderedList(token) = &tokens[1] {
                let list = &token.inline_tokens;
                assert_eq!(list.len(), 1);
                if let InlineToken::TextToken(token) = &list[0] {
                    assert_eq!(token.text, "another");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }

        if let LineToken::Paragraph(token) = &result[2] {
            let tokens = &token.inline_tokens;
            assert_eq!(tokens.len(), 1);
            if let InlineToken::TextToken(token) = &tokens[0] {
                assert_eq!(token.text, "test");
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_ordered_list_with_lines_surrounded() {
        let text = "a simple line\n1. a list\n2. another\ntest";
        let result = Tokenizer::scanner(text);
        assert_eq!(result.len(), 3);
        if let LineToken::Paragraph(token) = &result[0] {
            let tokens = &token.inline_tokens;
            assert_eq!(tokens.len(), 1);
            if let InlineToken::TextToken(token) = &tokens[0] {
                assert_eq!(token.text, "a simple line");
            } else {
                panic!();
            }
        } else {
            panic!();
        }

        if let LineToken::OrderedListBlock(token) = &result[1] {
            let tokens = &token.lists;
            assert_eq!(tokens.len(), 2);
            if let LineToken::OrderedList(token) = &tokens[0] {
                let list = &token.inline_tokens;
                assert_eq!(list.len(), 1);
                if let InlineToken::TextToken(token) = &list[0] {
                    assert_eq!(token.text, "a list");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::OrderedList(token) = &tokens[1] {
                let list = &token.inline_tokens;
                assert_eq!(list.len(), 1);
                if let InlineToken::TextToken(token) = &list[0] {
                    assert_eq!(token.text, "another");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }

        if let LineToken::Paragraph(token) = &result[2] {
            let tokens = &token.inline_tokens;
            assert_eq!(tokens.len(), 1);
            if let InlineToken::TextToken(token) = &tokens[0] {
                assert_eq!(token.text, "test");
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }
}
