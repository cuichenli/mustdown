pub use super::tokenizer::inline_token::{InlineToken, SpecialToken, DoubleSpecialToken, TextToken};
pub use super::tokenizer::line_token::{LineToken, HeaderToken, Paragraph};
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
            },
            InlineToken::SpecialToken(token)=> {
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
            },
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
            },
        };
        result
    }

    pub fn line_parse(&self, token: &LineToken) -> String {
        let mut result: String = String::new();
        match token {
            LineToken::HeaderToken (token) => {
                let level = token.level;
                let tokens = &token.inline_tokens;
                result.push_str(&format!("<h{}>", level));
                for t in tokens {
                    result.push_str(self.inline_parse(t).as_str());
                }
                result.push_str(&format!("</h{}>", level));
            },
            LineToken::Paragraph(token) => {
                result.push_str(&format!("<p>\n"));
                for t in &token.inline_tokens {
                    result.push_str(self.inline_parse(t).as_str());
                }
                result.push_str(&format!("</p>\n"));
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

    #[test]
    fn test_inline_parser() {
        let parser = Parser { tokens: Vec::new() };
        let t = TextToken{ text: String::from("this is a test")};
        let token = InlineToken::TextToken(t);
        let result = parser.inline_parse(&token);
        assert_eq!("this is a test", result);
    }

    #[test]
    fn test_italic_inline_parser() {
        let parser = Parser { tokens: Vec::new() };
        let text_token = TextToken{ text: String::from("this is a test") };
        let inline_tokens = vec![InlineToken::TextToken(text_token)];
        let special_token = SpecialToken {
            token: '*',
            inline_tokens: inline_tokens
        };
        let token = InlineToken::SpecialToken(special_token);
        let result = parser.inline_parse(&token);
        assert_eq!("<em>this is a test</em>", result);
    }

    #[test]
    fn test_paragraph_parser() {
        let parser = Parser { tokens: Vec::new() };
        let text_token = TextToken{ text: String::from("this is a test")};
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
                InlineToken::TextToken(TextToken{ text : String::from("this is a test")} ),
                InlineToken::SpecialToken(SpecialToken{
                    token: '*',
                    inline_tokens: vec![InlineToken::TextToken(TextToken{ text: String::from("another test")})],
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
            inline_tokens: vec![InlineToken::TextToken(TextToken{text: String::from("this is a test")})],
        });
        let result = parser.inline_parse(&token);
        assert_eq!("<strong>this is a test</strong>", result);
    }

    #[test]
    fn test_code_inline_parser() {
        let parser = Parser { tokens: Vec::new() };
        let token = InlineToken::SpecialToken(SpecialToken {
            token: '`',
            inline_tokens: vec![InlineToken::TextToken(TextToken{ text: String::from("this is a test")})],
        });
        let result = parser.inline_parse(&token);
        assert_eq!("<code>this is a test</code>", result);
    }

    #[test]
    fn test_paragraph_parser_with_strong_inline_token() {
        let parser = Parser { tokens: Vec::new() };
        let token = LineToken::Paragraph(Paragraph {
            inline_tokens: vec![
                InlineToken::TextToken(TextToken{ text: String::from("this is a test")}),
                InlineToken::DoubleSpecialToken(DoubleSpecialToken {
                    token: '*',
                    inline_tokens: vec![InlineToken::TextToken(TextToken{ text: String::from("another test")})],
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
                InlineToken::TextToken(TextToken{ text: String::from("this is a test")}),
                InlineToken::DoubleSpecialToken(DoubleSpecialToken {
                    token: '*',
                    inline_tokens: vec![InlineToken::TextToken(TextToken{ text: String::from("another test")})],
                }),
                InlineToken::SpecialToken(SpecialToken {
                    token: '*',
                    inline_tokens: vec![InlineToken::TextToken(TextToken{text: String::from("another test")})],
                })
            ],
        });
        let result = parser.line_parse(&token);
        assert_eq!(
            "<p>\nthis is a test<strong>another test</strong><em>another test</em></p>\n",
            result
        );
    }
}
