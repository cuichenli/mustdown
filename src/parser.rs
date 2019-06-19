pub use super::tokenizer::inline_token::{
    DoubleSpecialToken, ImageToken, InlineToken, LinkToken, SpecialToken, TextToken
};
pub use super::tokenizer::line_token::{HeaderToken, LineToken, Paragraph, Quote, OrderedList, OrderedListBlock, UnorderedList, UnorderedListBlock};
pub use super::tokenizer::Tokenizer;

pub struct Parser {
    tokens: Vec<LineToken>,
}

impl Parser {
    pub fn new(text: &str) -> Self {
        let tokenizer = Tokenizer::new(&text);
        let tokens = tokenizer.scanner();
        Self { tokens }
    }

    pub fn inline_parse(&self, token: &InlineToken) -> String {
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
                            result.push_str(self.inline_parse(t).as_str());
                        }
                        result.push_str(&"</em>");
                    }
                    '`' => {
                        result.push_str(&"<code>");
                        for t in tokens {
                            result.push_str(self.inline_parse(t).as_str());
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
                            result.push_str(self.inline_parse(t).as_str());
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

    pub fn line_parse(&self, token: &LineToken) -> String {
        let mut result: String = String::new();
        match token {
            LineToken::HeaderToken(token) => {
                let level = token.level;
                let tokens = &token.inline_tokens;
                result.push_str(&format!("<h{}>", level));
                for t in tokens {
                    result.push_str(self.inline_parse(t).as_str());
                }
                result.push_str(&format!("</h{}>", level));
            }
            LineToken::Paragraph(token) => {
                result.push_str(&format!("<p>\n"));
                for t in &token.inline_tokens {
                    result.push_str(self.inline_parse(t).as_str());
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
                    result.push_str(&self.inline_parse(t));
                }
                result.push_str("</p></blockquote>")
            }
            LineToken::UnorderedListBlock(token) => {
                result.push_str("<ul>");
                for t in &token.unordered_lists {
                    result.push_str(&self.line_parse(t));
                }
                result.push_str("</ul>")
            }
            LineToken::OrderedListBlock(token) => {
                result.push_str("<ol>");
                for t in &token.ordered_lists {
                    result.push_str(&self.line_parse(t));
                }
                result.push_str("</ol>")
            }
            LineToken::OrderedList(token) => {
                result.push_str("<li>");
                for t in &token.inline_tokens {
                    result.push_str(&self.inline_parse(t));
                }
                result.push_str("</li>")
            }
            LineToken::UnorderedList(token) => {
                result.push_str("<li>");
                for t in &token.inline_tokens {
                    result.push_str(&self.inline_parse(t));
                }
                result.push_str("</li>")
            }
        }
        result
    }

    pub fn parse(&self) -> String {
        let mut result = String::new();
        for token in &self.tokens {
            result.push_str(self.line_parse(token).as_str());
        }
        result
    }
}

#[cfg(test)]
mod test {

    pub use super::*;
    use crate::tokenizer::CodeBlock;

    #[test]
    fn test_inline_parser() {
        let parser = Parser { tokens: Vec::new() };
        let t = TextToken {
            text: String::from("this is a test"),
        };
        let token = InlineToken::TextToken(t);
        let result = parser.inline_parse(&token);
        assert_eq!("this is a test", result);
    }

    #[test]
    fn test_italic_inline_parser() {
        let parser = Parser { tokens: Vec::new() };
        let text_token = TextToken {
            text: String::from("this is a test"),
        };
        let inline_tokens = vec![InlineToken::TextToken(text_token)];
        let special_token = SpecialToken {
            token: '*',
            inline_tokens: inline_tokens,
        };
        let token = InlineToken::SpecialToken(special_token);
        let result = parser.inline_parse(&token);
        assert_eq!("<em>this is a test</em>", result);
    }

    #[test]
    fn test_paragraph_parser() {
        let parser = Parser { tokens: Vec::new() };
        let text_token = TextToken {
            text: String::from("this is a test"),
        };
        let paragraph = Paragraph {
            inline_tokens: vec![InlineToken::TextToken(text_token)],
        };
        let token = LineToken::Paragraph(paragraph);
        let result = parser.line_parse(&token);
        assert_eq!("<p>\nthis is a test</p>\n", result);
    }

    #[test]
    fn test_paragraph_parser_with_italic_inline_token() {
        let parser = Parser { tokens: Vec::new() };
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
        let result = parser.line_parse(&LineToken::Paragraph(token));
        assert_eq!("<p>\nthis is a test<em>another test</em></p>\n", result);
    }

    #[test]
    fn test_strong_inline_parser() {
        let parser = Parser { tokens: Vec::new() };
        let token = InlineToken::DoubleSpecialToken(DoubleSpecialToken {
            token: '*',
            inline_tokens: vec![InlineToken::TextToken(TextToken {
                text: String::from("this is a test"),
            })],
        });
        let result = parser.inline_parse(&token);
        assert_eq!("<strong>this is a test</strong>", result);
    }

    #[test]
    fn test_code_inline_parser() {
        let parser = Parser { tokens: Vec::new() };
        let token = InlineToken::SpecialToken(SpecialToken {
            token: '`',
            inline_tokens: vec![InlineToken::TextToken(TextToken {
                text: String::from("this is a test"),
            })],
        });
        let result = parser.inline_parse(&token);
        assert_eq!("<code>this is a test</code>", result);
    }

    #[test]
    fn test_paragraph_parser_with_strong_inline_token() {
        let parser = Parser { tokens: Vec::new() };
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
        let result = parser.line_parse(&token);
        assert_eq!(
            "<p>\nthis is a test<strong>another test</strong></p>\n",
            result
        );
    }

    #[test]
    fn test_paragraph_parser_with_strong_and_italic_inline_token() {
        let parser = Parser { tokens: Vec::new() };
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
        let result = parser.line_parse(&token);
        assert_eq!(
            "<p>\nthis is a test<strong>another test</strong><em>another test</em></p>\n",
            result
        );
    }

    #[test]
    fn test_code_block() {
        let parser = Parser { tokens: Vec::new() };
        let text = "this\nis\na\ntest";
        let token = CodeBlock::new(text.to_string());
        let result = parser.line_parse(&LineToken::CodeBlock(token));
        assert_eq!("<pre><code>\nthis\nis\na\ntest\n</code></pre>", result);
    }

    #[test]
    fn test_image_token() {
        let parser = Parser { tokens: Vec::new() };
        let image_token = ImageToken {
            link: String::from("link"),
            alt: String::from("alt"),
        };
        let token = InlineToken::ImageToken(image_token);
        let result = parser.inline_parse(&token);
        assert_eq!("<img src=\"link\" alt=\"alt\">", result);
    }

    #[test]
    fn test_link_token() {
        let parser = Parser { tokens: Vec::new() };
        let link_token = LinkToken {
            link: String::from("link"),
            alt: String::from("alt"),
        };
        let token = InlineToken::LinkToken(link_token);
        let result = parser.inline_parse(&token);
        assert_eq!("<a href=\"link\">alt</a>", result);
    }

    #[test]
    fn test_single_line_quote() {
        let parser = Parser { tokens: Vec::new() };
        let inline_tokens = vec![InlineToken::TextToken(TextToken {
            text: String::from("text token"),
        })];
        let quote_token = Quote { inline_tokens };
        let result = parser.line_parse(&LineToken::Quote(quote_token));
        assert_eq!(result, "<blockquote><p>text token</p></blockquote>");
    }

    #[test]
    fn test_multiple_lines_quote() {
        let parser = Parser { tokens: Vec::new() };
        let inline_tokens = vec![
            InlineToken::TextToken(TextToken {
                text: String::from("text token"),
            }),
            InlineToken::BreakToken,
        ];
        let quote_token = Quote { inline_tokens };
        let result = parser.line_parse(&LineToken::Quote(quote_token));
        assert_eq!(result, "<blockquote><p>text token<br></p></blockquote>");
    }

    #[test]
    fn test_more_lines_quote() {
        let parser = Parser { tokens: Vec::new() };
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
        let result = parser.line_parse(&LineToken::Quote(quote_token));
        assert_eq!(
            result,
            "<blockquote><p>text token<br>another token</p></blockquote>"
        );
    }

    #[test]
    fn test_more_lines_quote_with_special_token() {
        let parser = Parser { tokens: Vec::new() };
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
        let result = parser.line_parse(&LineToken::Quote(quote_token));
        assert_eq!(
            result,
            "<blockquote><p>text token<br>another token<em>another test</em></p></blockquote>"
        );
    }

    #[test]
    fn test_ordered_list() {
        let parser = Parser { tokens: Vec::new() };
        let token = OrderedListBlock {
            ordered_lists: vec![
                LineToken::OrderedList( OrderedList {
                    order: 1,
                    inline_tokens: vec![
                        InlineToken::TextToken( TextToken{
                            text: String::from("first")
                        })
                    ]
                }),
                LineToken::OrderedList( OrderedList {
                    order: 1,
                    inline_tokens: vec![
                        InlineToken::TextToken( TextToken{
                            text: String::from("second")
                        })
                    ]
                })
            ]
        };
        let result = parser.line_parse(&LineToken::OrderedListBlock(token));
        assert_eq!(result, "<ol><li>first</li><li>second</li></ol>");
    }

    #[test]
    fn test_unordered_list() {
        let parser = Parser { tokens: Vec::new() };
        let token = UnorderedListBlock {
            unordered_lists: vec![
                LineToken::UnorderedList( UnorderedList {
                    inline_tokens: vec![
                        InlineToken::TextToken( TextToken{
                            text: String::from("first")
                        })
                    ]
                }),
                LineToken::UnorderedList( UnorderedList {
                    inline_tokens: vec![
                        InlineToken::TextToken( TextToken{
                            text: String::from("second")
                        })
                    ]
                })
            ]
        };
        let result = parser.line_parse(&LineToken::UnorderedListBlock(token));
        assert_eq!(result, "<ul><li>first</li><li>second</li></ul>");
    }
}
