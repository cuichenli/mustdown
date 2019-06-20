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
        let tokenizer = Tokenizer::new(&text);
        let tokens = tokenizer.scanner();
        Self { tokens }
    }

    pub fn __inline_parse<'a>(tokens: &'a Vec<InlineToken>, token_records: &mut Vec<&'a SpecialToken>, text_record: &mut Vec<String>, temp_text: &mut Vec<String>, index: usize) {
        if index >= tokens.len() {
            return
        }
        let mut index = index;
        match &tokens[index] {
            InlineToken::TextToken(_) => {
                let mut result = String::new();
                while let InlineToken::TextToken(token) = &tokens[index]  {
                    result.push_str(&token.text);
                    index += 1;
                    if index >= tokens.len() {
                        break;
                    }
                }
                index -= 1;
                if token_records.len() > 0 {
                    temp_text.push(result);
                } else {
                    text_record.push(result);
                }
            }
            InlineToken::SpecialToken(token) => {
                if token_records.len() == 0 || temp_text.len() == 0 {
                    token_records.push(&token);
                } else {
                    let last = token_records.last().unwrap();
                    if last.token == token.token {
                        let mut symbol = "em";
                        token_records.pop();
                        if token_records.len() > 0 {
                            let last = token_records.last().unwrap();
                            if index + 1 < tokens.len() {
                                if let InlineToken::SpecialToken(token) = &tokens[index + 1] {
                                    if token.token == last.token {
                                        symbol = "strong";
                                        index += 1;
                                        token_records.pop();
                                    }
                                }
                            }
                        }

                        let record_string = temp_text.pop().unwrap();
                        let result = format!("<{}>{}</{}>", symbol, record_string, symbol);
                        text_record.push(result);
                    } else {
                        token_records.push(&token);
                    }
                }
            }
            InlineToken::ImageToken(token) => {
                text_record.push(
                    format!("<img src=\"{}\" alt=\"{}\">", token.link, token.alt),
                );
            }
            InlineToken::LinkToken(token) => {
                text_record.push(format!("<a href=\"{}\">{}</a>", token.link, token.alt));
            }
            InlineToken::BreakToken => {
                text_record.push(String::from("<br>"));
            }
            _ => ()
        }
        Parser::__inline_parse(tokens, token_records, text_record, temp_text, index + 1);
    }

    pub fn _inline_parse(tokens: &Vec<InlineToken>) -> String {
        let mut token_records: Vec<&SpecialToken> = Vec::new();
        let mut text_record: Vec<String> = Vec::new();
        let mut temp_text: Vec<String> = Vec::new();
        Parser::__inline_parse(tokens, &mut token_records, &mut text_record, &mut temp_text, 0);
        println!("{:?}", text_record);
        text_record.join("")
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

    pub fn special_token_factory(token: char) -> InlineToken {
        let t = SpecialToken::new(token);
        InlineToken::SpecialToken(t)
    }

    pub fn text_token_factory(text: String) -> InlineToken {
        let t = TextToken {
            text
        };
        InlineToken::TextToken(t)
    }

    pub fn special_token_group_factory(token: char, text: String) -> Vec<InlineToken> {
        vec![special_token_factory('*'), text_token_factory(text), special_token_factory('*')]
    }

    #[test]
    fn test_inline_parser() {
        // let parser = Parser { tokens: Vec::new() };
        let t = TextToken {
            text: String::from("this is a test"),
        };
        let token = InlineToken::TextToken(t);
        let result = Parser::_inline_parse(&vec![token]);
        assert_eq!("this is a test", result);
    }

    #[test]
    fn test_italic_inline_parser() {
        // let parser = Parser { tokens: Vec::new() };
        // let text_token = TextToken {
        //     text: String::from("this is a test"),
        // };
        // let inline_tokens = vec![InlineToken::TextToken(text_token), InlineToken::SpecialToken];
        // let special_token = SpecialToken {
        //     token: '*',
        //     inline_tokens: inline_tokens,
        // };
        // let token = InlineToken::SpecialToken(special_token);
        // let result = parser.inline_parse(&token);
        let tokens = special_token_group_factory('*', String::from("this is a test"));
        let result = Parser::_inline_parse(&tokens);
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
                LineToken::OrderedList(OrderedList {
                    order: 1,
                    inline_tokens: vec![InlineToken::TextToken(TextToken {
                        text: String::from("first"),
                    })],
                }),
                LineToken::OrderedList(OrderedList {
                    order: 1,
                    inline_tokens: vec![InlineToken::TextToken(TextToken {
                        text: String::from("second"),
                    })],
                }),
            ],
        };
        let result = parser.line_parse(&LineToken::OrderedListBlock(token));
        assert_eq!(result, "<ol><li>first</li><li>second</li></ol>");
    }

    #[test]
    fn test_unordered_list() {
        let parser = Parser { tokens: Vec::new() };
        let token = UnorderedListBlock {
            unordered_lists: vec![
                LineToken::UnorderedList(UnorderedList {
                    inline_tokens: vec![InlineToken::TextToken(TextToken {
                        text: String::from("first"),
                    })],
                }),
                LineToken::UnorderedList(UnorderedList {
                    inline_tokens: vec![InlineToken::TextToken(TextToken {
                        text: String::from("second"),
                    })],
                }),
            ],
        };
        let result = parser.line_parse(&LineToken::UnorderedListBlock(token));
        assert_eq!(result, "<ul><li>first</li><li>second</li></ul>");
    }
}
