pub use super::tokenizer::line_token::LineToken;
pub use super::tokenizer::inline_token::InlineToken;
pub use super::tokenizer::Tokenizer;

pub struct Parser {
    tokens: Vec<LineToken>
}

impl Parser {
    pub fn new(text: &str) -> Self{
        let tokenizer = Tokenizer::new(&text);
        let tokens = tokenizer.scanner();
        Self {
            tokens
        }
    }

    pub fn inline_parse(&self, token: &InlineToken) -> String {
        let mut result = String::new();
        match token {
            InlineToken::TextToken(text) => {
                result.push_str(text);
            },
            InlineToken::SpecialToken { token: t, inline_tokens: tokens} => {
                match t {
                    '*' => {
                        result.push_str(&"<b>");
                        for t in tokens {
                            result.push_str(self.inline_parse(t).as_str());
                        }
                        result.push_str(&"</b>");
                    },
                    _ => panic!()
                };
            }
        };
        result
    }

    pub fn line_parse(&self, token: &LineToken) -> String {
        let mut result: String = String::new();
        match token {
            LineToken::HeaderToken { level, inline_tokens} => {
                result.push_str(&format!("<h{}>", level));
				for t in inline_tokens {
                    result.push_str(self.inline_parse(t).as_str());
                }
				result.push_str(&format!("</h{}>", level));
            },
            LineToken::Paragraph { inline_tokens } => {
                result.push_str(&format!("<p>\n"));
                for t in inline_tokens {
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
    fn test_inline_parsesr() {
        let parser = Parser {
            tokens: Vec::new()
        };
        let token = InlineToken::TextToken(String::from("this is a test"));
        let result = parser.inline_parse(&token);
        assert_eq!("this is a test", result);
    }

    #[test]
    fn test_bold_inline_parser() {
        let parser = Parser {
            tokens: Vec::new()
        };
        let token = InlineToken::SpecialToken {
            token: '*',
            inline_tokens: vec![InlineToken::TextToken(String::from("this is a test"))]
        };
        let result = parser.inline_parse(&token);
        assert_eq!("<b>this is a test</b>", result);
    }

    #[test]
    fn test_paragaph_parser() {
        let parser = Parser {
            tokens: Vec::new()
        };
        let token = LineToken::Paragraph {
            inline_tokens: vec![InlineToken::TextToken(String::from("this is a test"))]
        };
        let result = parser.line_parse(&token);
        assert_eq!("<p>\nthis is a test</p>\n", result);
    }

    #[test]
    fn test_paragaph_parser_with_multiple_inline_token() {
        let parser = Parser {
            tokens: Vec::new()
        };
        let token = LineToken::Paragraph {
            inline_tokens: vec![
                InlineToken::TextToken(String::from("this is a test")),
                InlineToken::SpecialToken {
                    token: '*',
                    inline_tokens: vec![InlineToken::TextToken(String::from("another test"))]
                }
            ]
        };
        let result = parser.line_parse(&token);
        assert_eq!("<p>\nthis is a test<b>another test</b></p>\n", result);
    }
}