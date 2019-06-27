pub use super::tokenizer::inline_token::{
    DoubleSpecialToken, ImageToken, InlineToken, LinkToken, SpecialToken, TextToken,
};
pub use super::tokenizer::line_token::{
    HeaderToken, LineToken, OrderedList, OrderedListBlock, Paragraph, Quote, UnorderedList,
    UnorderedListBlock,
};
pub use super::tokenizer::Tokenizer;

use std::collections::HashMap;


pub struct Parser {
    notes: HashMap<String, String>,
}

impl Parser  {
    pub fn new() -> Self {
        let notes = HashMap::new();
        Self { notes }
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
                let link: &String;
                if token.need_note {
                    link = &self.notes[&token.link];
                } else {
                    link = &token.link;
                }
                result.push_str(
                    format!("<img src=\"{}\" alt=\"{}\">", link, token.alt).as_str(),
                );
            }
            InlineToken::LinkToken(token) => {
                let link: &String;
                if token.need_note {
                    link = &self.notes[&token.link];
                } else {
                    link = &token.link;
                }
                result.push_str(format!("<a href=\"{}\">{}</a>", link, token.alt).as_str());
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
                for t in &token.lists {
                    result.push_str(&self.line_parse(t));
                }
                result.push_str("</ul>")
            }
            LineToken::OrderedListBlock(token) => {
                let start = token.start;
                result.push_str(&format!("<ol start=\"{}\">", start));
                for t in &token.lists {
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
            LineToken::HorizontalRule => {
                result.push_str("<hr>");
            }
            LineToken::NoteToken(_) => ()
        }
        result
    }

    pub fn extract_notes(tokens: &Vec<LineToken>) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for token in tokens {
            if let LineToken::NoteToken(t) = token {
                let name = t.name.clone();
                let link = t.link.clone();
                result.insert(name, link);
            }
        }
        result
    }

    pub fn parse(&mut self, text: &str) -> String {
        let mut result = String::new();
        let tokens = Tokenizer::tokenizer(text);
        self.notes = Parser::extract_notes(&tokens);
        for token in tokens {
            result.push_str(self.line_parse(&token).as_str());
        }
        result
    }
}

#[cfg(test)]
mod test {

    pub use super::*;
    use crate::tokenizer::CodeBlock;
    use crate::tokenizer::LinkToken;

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
        let parser = Parser::new();
        let t = TextToken {
            text: String::from("this is a test"),
        };
        let token = InlineToken::TextToken(t);
        let result = parser.inline_parse(&token);
        assert_eq!("this is a test", result);
    }

    #[test]
    fn test_italic_inline_parser() {
        let parser = Parser::new();
        let tokens = special_token_factory('*', String::from("this is a test"));
        let result = parser.inline_parse(&tokens);
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
        let parser = Parser::new();

        let token = LineToken::Paragraph(paragraph);
        let result = parser.line_parse(&token);
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
        let parser = Parser::new();
        let result = parser.line_parse(&LineToken::Paragraph(token));
        assert_eq!("<p>\nthis is a test<em>another test</em></p>\n", result);
    }

    #[test]
    fn test_strong_inline_parser() {
        let parser = Parser::new();
        let token = double_special_token_factory('*', String::from("this is a test"));
        let result = parser.inline_parse(&token);
        assert_eq!("<strong>this is a test</strong>", result);
    }

    #[test]
    fn test_code_inline_parser() {
        let parser = Parser::new();
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
        let parser = Parser::new();
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
        let parser = Parser::new();
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
        let parser = Parser::new();
        let text = "this\nis\na\ntest";
        let token = CodeBlock::new(text.to_string());
        let result = parser.line_parse(&LineToken::CodeBlock(token));
        assert_eq!("<pre><code>\nthis\nis\na\ntest\n</code></pre>", result);
    }

    #[test]
    fn test_image_token() {
        let parser = Parser::new();
        let image_token = ImageToken {
            link: String::from("link"),
            alt: String::from("alt"),
            need_note: false,
        };
        let token = InlineToken::ImageToken(image_token);
        let result = parser.inline_parse(&token);
        assert_eq!("<img src=\"link\" alt=\"alt\">", result);
    }

    #[test]
    fn test_link_token() {
        let parser = Parser::new();
        let link_token = LinkToken {
            link: String::from("link"),
            alt: String::from("alt"),
            need_note: false,
        };
        let token = InlineToken::LinkToken(link_token);
        let result = parser.inline_parse(&token);
        assert_eq!("<a href=\"link\">alt</a>", result);
    }

    #[test]
    fn test_single_line_quote() {
        let parser = Parser::new();
        let inline_tokens = vec![InlineToken::TextToken(TextToken {
            text: String::from("text token"),
        })];
        let quote_token = Quote { inline_tokens };
        let result = parser.line_parse(&LineToken::Quote(quote_token));
        assert_eq!(result, "<blockquote><p>text token</p></blockquote>");
    }

    #[test]
    fn test_multiple_lines_quote() {
        let parser = Parser::new();
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
        let parser = Parser::new();
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
        let parser = Parser::new();
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
        let parser = Parser::new();
        let token = OrderedListBlock {
            start: '1',
            symbol: ')',
            lists: vec![
                LineToken::OrderedList(OrderedList {
                    order: '1',
                    symbol: ')',
                    inline_tokens: vec![InlineToken::TextToken(TextToken {
                        text: String::from("first"),
                    })],
                }),
                LineToken::OrderedList(OrderedList {
                    order: '1',
                    symbol: ')',
                    inline_tokens: vec![InlineToken::TextToken(TextToken {
                        text: String::from("second"),
                    })],
                }),
            ],
        };
        let result = parser.line_parse(&LineToken::OrderedListBlock(token));
        assert_eq!(result, "<ol start=\"1\"><li>first</li><li>second</li></ol>");
    }

    #[test]
    fn test_unordered_list() {
        let parser = Parser::new();
        let token = UnorderedListBlock {
            symbol: '*',
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
        let result = parser.line_parse(&LineToken::UnorderedListBlock(token));
        assert_eq!(result, "<ul><li>first</li><li>second</li></ul>");
    }

    #[test]
    fn test_link_with_need_note() {
        let text = "[alt][link]\n[link]:http://a.com";
        let mut parser = Parser::new();
        let result = parser.parse(text);
        assert_eq!(parser.notes.len(), 1);
        assert_eq!(parser.notes["link"], "http://a.com");
        assert_eq!(result, "<p>\n<a href=\"http://a.com\">alt</a></p>\n");
    }

    #[test]
    fn test_image_with_need_note() {
        let text = "![alt][link]\n[link]:http://a.com";
        let mut parser = Parser::new();
        let result = parser.parse(text);
        assert_eq!(parser.notes.len(), 1);
        assert_eq!(parser.notes["link"], "http://a.com");
        assert_eq!(result, "<p>\n<img src=\"http://a.com\" alt=\"alt\"></p>\n");
    }

    #[test]
    fn test_horizontal_rule() {
        let token = LineToken::HorizontalRule;
        let parser = Parser::new();
        let result = parser.line_parse(&token);
        assert_eq!("<hr>", result);
    }
}
