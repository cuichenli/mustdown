pub use super::tokenizer::inline_token::{
    DoubleSpecialToken, ImageToken, InlineToken, LinkToken, SpecialToken, TextToken,
};
pub use super::tokenizer::line_token::{
    HeaderToken, LineToken, OrderedList, OrderedListBlock, Paragraph, Quote, UnorderedList,
    UnorderedListBlock,
};
pub use super::tokenizer::Tokenizer;

pub struct Parser {
    tokens: Vec<LineToken>,
}

impl Parser {
    pub fn new(text: &str) -> Self {
        let tokens = Tokenizer::scanner(text);
        Self { tokens }
    }

    pub fn inline_parse(token: &InlineToken) -> String {
        let mut result = String::new();
        match token {
            InlineToken::TextToken(token) => {
                result.push_str(&token.text);
            }
            InlineToken::SpecialToken(token) => {
                let tokens = &token.inline_tokens;
                match token.token {
                    '*' | '_' => {
                        result.push_str(&"<em>");
                        for t in tokens {
                            result.push_str(Parser::inline_parse(t).as_str());
                        }
                        result.push_str(&"</em>");
                    }
                    '`' => {
                        result.push_str(&"<code>");
                        for t in tokens {
                            result.push_str(Parser::inline_parse(t).as_str());
                        }
                        result.push_str("</code>")
                    }
                    _ => panic!(),
                };
            }
            InlineToken::DoubleSpecialToken(token) => {
                let tokens = &token.inline_tokens;
                match token.token {
                    '*' | '_' => {
                        result.push_str(&"<strong>");
                        for t in tokens {
                            result.push_str(Parser::inline_parse(t).as_str());
                        }
                        result.push_str(&"</strong>");
                    }
                    _ => panic!(),
                }
            }
            InlineToken::ImageToken(token) => {
                result.push_str(
                    format!("<img src=\"{}\" alt=\"{}\">", token.link, token.alt).as_str(),
                );
            }
            InlineToken::LinkToken(token) => {
                result.push_str(format!("<a href=\"{}\">{}</a>", token.link, token.alt).as_str());
            }
            InlineToken::BreakToken => {
                result.push_str("<br>");
            }
        };
        result
    }

    pub fn line_parse(token: &LineToken) -> String {
        let mut result: String = String::new();
        match token {
            LineToken::HeaderToken(token) => {
                let level = token.level;
                let tokens = &token.inline_tokens;
                result.push_str(&format!("<h{}>", level));
                for t in tokens {
                    result.push_str(Parser::inline_parse(t).as_str());
                }
                result.push_str(&format!("</h{}>", level));
            }
            LineToken::Paragraph(token) => {
                result.push_str(&format!("<p>\n"));
                for t in &token.inline_tokens {
                    result.push_str(Parser::inline_parse(t).as_str());
                }
                result.push_str(&format!("</p>\n"));
            }
            LineToken::CodeBlock(token) => {
                result.push_str("<pre><code>\n");
                result.push_str(&token.text);
                result.push_str("\n</code></pre>");
            }
            LineToken::Quote(token) => {
                result.push_str("<blockquote><p>");
                for t in &token.inline_tokens {
                    result.push_str(&Parser::inline_parse(t));
                }
                result.push_str("</p></blockquote>")
            }
            LineToken::UnorderedListBlock(token) => {
                result.push_str("<ul>");
                for t in &token.lists {
                    result.push_str(&Parser::line_parse(t));
                }
                result.push_str("</ul>")
            }
            LineToken::OrderedListBlock(token) => {
                result.push_str("<ol>");
                for t in &token.lists {
                    result.push_str(&Parser::line_parse(t));
                }
                result.push_str("</ol>")
            }
            LineToken::OrderedList(token) => {
                result.push_str("<li>");
                for t in &token.inline_tokens {
                    result.push_str(&Parser::inline_parse(t));
                }
                result.push_str("</li>")
            }
            LineToken::UnorderedList(token) => {
                result.push_str("<li>");
                for t in &token.inline_tokens {
                    result.push_str(&Parser::inline_parse(t));
                }
                result.push_str("</li>")
            }
        }
        result
    }

    pub fn parse(&self) -> String {
        let mut result = String::new();
        for token in &self.tokens {
            result.push_str(Parser::line_parse(token).as_str());
        }
        result
    }
}

#[cfg(test)]
mod test {

    pub use super::*;
    use crate::tokenizer::CodeBlock;

    pub fn text_token_factory(text: String) -> InlineToken {
        let t = TextToken { text };
        InlineToken::TextToken(t)
    }

    pub fn special_token_factory(token: char, text: String) -> InlineToken {
        let text_token = text_token_factory(text);
        let t = SpecialToken::new(token, vec![text_token]);
        InlineToken::SpecialToken(t)
    }

    pub fn double_special_token_factory(token: char, text: String) -> InlineToken {
        let text_token = text_token_factory(text);
        let t = DoubleSpecialToken::new(token, vec![text_token]);
        InlineToken::DoubleSpecialToken(t)
    }

    #[test]
    fn test_inline_parser() {
        //
        let t = TextToken {
            text: String::from("this is a test"),
        };
        let token = InlineToken::TextToken(t);
        let result = Parser::inline_parse(&token);
        assert_eq!("this is a test", result);
    }

    #[test]
    fn test_italic_inline_parser() {
        let tokens = special_token_factory('*', String::from("this is a test"));
        let result = Parser::inline_parse(&tokens);
        assert_eq!("<em>this is a test</em>", result);
    }

    #[test]
    fn test_paragraph_parser() {
        let text_token = TextToken {
            text: String::from("this is a test"),
        };
        let paragraph = Paragraph {
            inline_tokens: vec![InlineToken::TextToken(text_token)],
        };
        let token = LineToken::Paragraph(paragraph);
        let result = Parser::line_parse(&token);
        assert_eq!("<p>\nthis is a test</p>\n", result);
    }

    #[test]
    fn test_paragraph_parser_with_italic_inline_token() {
        let token = Paragraph {
            inline_tokens: vec![
                InlineToken::TextToken(TextToken {
                    text: String::from("this is a test"),
                }),
                InlineToken::SpecialToken(SpecialToken {
                    token: '*',
                    inline_tokens: vec![InlineToken::TextToken(TextToken {
                        text: String::from("another test"),
                    })],
                }),
            ],
        };
        let result = Parser::line_parse(&LineToken::Paragraph(token));
        assert_eq!("<p>\nthis is a test<em>another test</em></p>\n", result);
    }

    #[test]
    fn test_strong_inline_parser() {
        let token = double_special_token_factory('*', String::from("this is a test"));
        let result = Parser::inline_parse(&token);
        assert_eq!("<strong>this is a test</strong>", result);
    }

    #[test]
    fn test_code_inline_parser() {
        let token = InlineToken::SpecialToken(SpecialToken {
            token: '`',
            inline_tokens: vec![InlineToken::TextToken(TextToken {
                text: String::from("this is a test"),
            })],
        });
        let result = Parser::inline_parse(&token);
        assert_eq!("<code>this is a test</code>", result);
    }

    #[test]
    fn test_paragraph_parser_with_strong_inline_token() {
        let token = LineToken::Paragraph(Paragraph {
            inline_tokens: vec![
                InlineToken::TextToken(TextToken {
                    text: String::from("this is a test"),
                }),
                InlineToken::DoubleSpecialToken(DoubleSpecialToken {
                    token: '*',
                    inline_tokens: vec![InlineToken::TextToken(TextToken {
                        text: String::from("another test"),
                    })],
                }),
            ],
        });
        let result = Parser::line_parse(&token);
        assert_eq!(
            "<p>\nthis is a test<strong>another test</strong></p>\n",
            result
        );
    }

    #[test]
    fn test_paragraph_parser_with_strong_and_italic_inline_token() {
        let token = LineToken::Paragraph(Paragraph {
            inline_tokens: vec![
                InlineToken::TextToken(TextToken {
                    text: String::from("this is a test"),
                }),
                InlineToken::DoubleSpecialToken(DoubleSpecialToken {
                    token: '*',
                    inline_tokens: vec![InlineToken::TextToken(TextToken {
                        text: String::from("another test"),
                    })],
                }),
                InlineToken::SpecialToken(SpecialToken {
                    token: '*',
                    inline_tokens: vec![InlineToken::TextToken(TextToken {
                        text: String::from("another test"),
                    })],
                }),
            ],
        });
        let result = Parser::line_parse(&token);
        assert_eq!(
            "<p>\nthis is a test<strong>another test</strong><em>another test</em></p>\n",
            result
        );
    }

    #[test]
    fn test_code_block() {
        let text = "this\nis\na\ntest";
        let token = CodeBlock::new(text.to_string());
        let result = Parser::line_parse(&LineToken::CodeBlock(token));
        assert_eq!("<pre><code>\nthis\nis\na\ntest\n</code></pre>", result);
    }

    #[test]
    fn test_image_token() {
        let image_token = ImageToken {
            link: String::from("link"),
            alt: String::from("alt"),
        };
        let token = InlineToken::ImageToken(image_token);
        let result = Parser::inline_parse(&token);
        assert_eq!("<img src=\"link\" alt=\"alt\">", result);
    }

    #[test]
    fn test_link_token() {
        let link_token = LinkToken {
            link: String::from("link"),
            alt: String::from("alt"),
        };
        let token = InlineToken::LinkToken(link_token);
        let result = Parser::inline_parse(&token);
        assert_eq!("<a href=\"link\">alt</a>", result);
    }

    #[test]
    fn test_single_line_quote() {
        let inline_tokens = vec![InlineToken::TextToken(TextToken {
            text: String::from("text token"),
        })];
        let quote_token = Quote { inline_tokens };
        let result = Parser::line_parse(&LineToken::Quote(quote_token));
        assert_eq!(result, "<blockquote><p>text token</p></blockquote>");
    }

    #[test]
    fn test_multiple_lines_quote() {
        let inline_tokens = vec![
            InlineToken::TextToken(TextToken {
                text: String::from("text token"),
            }),
            InlineToken::BreakToken,
        ];
        let quote_token = Quote { inline_tokens };
        let result = Parser::line_parse(&LineToken::Quote(quote_token));
        assert_eq!(result, "<blockquote><p>text token<br></p></blockquote>");
    }

    #[test]
    fn test_more_lines_quote() {
        let inline_tokens = vec![
            InlineToken::TextToken(TextToken {
                text: String::from("text token"),
            }),
            InlineToken::BreakToken,
            InlineToken::TextToken(TextToken {
                text: String::from("another token"),
            }),
        ];
        let quote_token = Quote { inline_tokens };
        let result = Parser::line_parse(&LineToken::Quote(quote_token));
        assert_eq!(
            result,
            "<blockquote><p>text token<br>another token</p></blockquote>"
        );
    }

    #[test]
    fn test_more_lines_quote_with_special_token() {
        let inline_tokens = vec![
            InlineToken::TextToken(TextToken {
                text: String::from("text token"),
            }),
            InlineToken::BreakToken,
            InlineToken::TextToken(TextToken {
                text: String::from("another token"),
            }),
            InlineToken::SpecialToken(SpecialToken {
                token: '*',
                inline_tokens: vec![InlineToken::TextToken(TextToken {
                    text: String::from("another test"),
                })],
            }),
        ];
        let quote_token = Quote { inline_tokens };
        let result = Parser::line_parse(&LineToken::Quote(quote_token));
        assert_eq!(
            result,
            "<blockquote><p>text token<br>another token<em>another test</em></p></blockquote>"
        );
    }

    #[test]
    fn test_ordered_list() {
        let token = OrderedListBlock {
            lists: vec![
                LineToken::OrderedList(OrderedList {
                    inline_tokens: vec![InlineToken::TextToken(TextToken {
                        text: String::from("first"),
                    })],
                }),
                LineToken::OrderedList(OrderedList {
                     inline_tokens: vec![InlineToken::TextToken(TextToken {
                        text: String::from("second"),
                    })],
                }),
            ],
        };
        let result = Parser::line_parse(&LineToken::OrderedListBlock(token));
        assert_eq!(result, "<ol><li>first</li><li>second</li></ol>");
    }

    #[test]
    fn test_unordered_list() {
        let token = UnorderedListBlock {
            lists: vec![
                LineToken::UnorderedList(UnorderedList {
                    symbol: '*',
                    inline_tokens: vec![InlineToken::TextToken(TextToken {
                        text: String::from("first"),
                    })],
                }),
                LineToken::UnorderedList(UnorderedList {
                    symbol: '*',
                    inline_tokens: vec![InlineToken::TextToken(TextToken {
                        text: String::from("second"),
                    })],
                }),
            ],
        };
        let result = Parser::line_parse(&LineToken::UnorderedListBlock(token));
        assert_eq!(result, "<ul><li>first</li><li>second</li></ul>");
    }
}
