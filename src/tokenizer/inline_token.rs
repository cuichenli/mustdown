#[derive(Debug)]
pub enum InlineToken {
    TextToken(TextToken),
    SpecialToken(SpecialToken),
    DoubleSpecialToken(DoubleSpecialToken),
    LinkToken(LinkToken),
    ImageToken(ImageToken),
    BreakToken,
}
#[derive(Debug)]
pub struct TextToken {
    pub text: String,
}
#[derive(Debug)]
pub struct SpecialToken {
    pub token: char,
    pub inline_tokens: Vec<InlineToken>,
}

impl SpecialToken {
    pub fn new(token: char) -> Self {
        Self {
            token,
            inline_tokens: vec![]
        }
    }
}
#[derive(Debug)]
pub struct DoubleSpecialToken {
    pub token: char,
    pub inline_tokens: Vec<InlineToken>,
}
#[derive(Debug)]
pub struct LinkToken {
    pub alt: String,
    pub link: String,

}

impl LinkToken {
    pub fn new(alt: String, link: String ) -> Self {
        Self {
            alt, link
        }
    }

    pub fn len(&self) -> usize {
        self.alt.len() + self.link.len() + 4
    }
}

#[derive(Debug)]
pub struct ImageToken {
    pub alt: String,
    pub link: String,
}

impl ImageToken {
    pub fn new(alt: String, link: String ) -> Self {
        Self {
            alt, link
        }
    }

    pub fn len(&self) -> usize {
        self.alt.len() + self.link.len() + 5
    }
}

#[cfg(test)]
mod tests {
    use super::super::line_token::LineToken;
    use super::*;
    use crate::Tokenizer;

    #[test]
    fn test_special_token() {
        let tokenizer = Tokenizer::new("*Test*");
        let result = tokenizer.scanner();
        let inline_tokens: &Vec<InlineToken>;
        match &result[0] {
            LineToken::Paragraph(token) => {
                inline_tokens = &token.inline_tokens;
            }
            _ => panic!(),
        };
        assert_eq!(inline_tokens.len(), 1);
        let result = &inline_tokens[0];
        let inline_tokens: &Vec<InlineToken>;
        match result {
            InlineToken::SpecialToken(token) => {
                assert_eq!(token.token, '*');
                assert_eq!(token.inline_tokens.len(), 1);
                inline_tokens = &token.inline_tokens;
            }
            _ => panic!(),
        };
        match &inline_tokens[0] {
            InlineToken::TextToken(text) => {
                assert_eq!(text.text, "Test");
            }
            _ => panic!(),
        };
    }

    #[test]
    fn test_special_token_start_with_space() {
        let tokenizer = Tokenizer::new(" *Test*");
        let result = tokenizer.scanner();
        let inline_tokens: &Vec<InlineToken>;
        match &result[0] {
            LineToken::Paragraph(token) => {
                inline_tokens = &token.inline_tokens;
            }
            _ => panic!(),
        };
        assert_eq!(inline_tokens.len(), 2);
        let result = &inline_tokens[1];
        let text_token = &inline_tokens[0];
        match text_token {
            InlineToken::TextToken(text) => {
                assert_eq!(text.text, " ");
            }
            _ => panic!(),
        };
        let inline_tokens: &Vec<InlineToken>;
        match result {
            InlineToken::SpecialToken(token) => {
                assert_eq!(token.token, '*');
                assert_eq!(token.inline_tokens.len(), 1);
                inline_tokens = &token.inline_tokens;
            }
            _ => panic!(),
        };
        match &inline_tokens[0] {
            InlineToken::TextToken(text) => {
                assert_eq!(text.text, "Test");
            }
            _ => panic!(),
        };
    }

    #[test]
    fn test_special_token_with_start_space_and_end_words() {
        let tokenizer = Tokenizer::new(" *Test* another test");
        let result = tokenizer.scanner();
        assert_eq!(result.len(), 1);
        let inline_token = &result[0];
        let tokens: &Vec<InlineToken>;
        match inline_token {
            LineToken::Paragraph(token) => {
                tokens = &token.inline_tokens;
            }
            _ => panic!(),
        };
        assert_eq!(tokens.len(), 3);
        match &tokens[0] {
            InlineToken::TextToken(text) => {
                assert_eq!(text.text, " ");
            }
            _ => panic!(),
        };
        match &tokens[1] {
            InlineToken::SpecialToken(token) => {
                assert_eq!(token.token, '*');
                assert_eq!(token.inline_tokens.len(), 1);
                match &token.inline_tokens[0] {
                    InlineToken::TextToken(text) => {
                        assert_eq!(text.text, "Test");
                    }
                    _ => panic!(),
                };
            }
            _ => panic!(),
        };
        match &tokens[2] {
            InlineToken::TextToken(text) => assert_eq!(text.text, " another test"),
            _ => panic!(),
        };
    }

    #[test]
    fn test_double_special_token() {
        let tokenizer = Tokenizer::new("");
        let result = tokenizer.inline_scanner("**Test**");
        let token1 = &result[0];
        match token1 {
            InlineToken::DoubleSpecialToken(token) => {
                assert_eq!(token.token, '*');
                match &token.inline_tokens[0] {
                    InlineToken::TextToken(text) => {
                        assert_eq!(text.text, "Test");
                    }
                    _ => panic!(),
                }
            }
            _ => panic!(),
        };
    }

    #[test]
    fn test_double_special_token_with_start_space() {
        let tokenizer = Tokenizer::new("");
        let result = tokenizer.inline_scanner(" **Test**");
        assert_eq!(result.len(), 2);
        let token1 = &result[0];
        match token1 {
            InlineToken::TextToken(text) => assert_eq!(text.text, " "),
            _ => panic!(),
        };
        let token2 = &result[1];
        match token2 {
            InlineToken::DoubleSpecialToken(token) => {
                assert_eq!(token.token, '*');
                match &token.inline_tokens[0] {
                    InlineToken::TextToken(text) => {
                        assert_eq!(text.text, "Test");
                    }
                    _ => panic!(),
                }
            }
            _ => panic!(),
        };
    }

    #[test]
    fn test_code_special_token() {
        let tokenizer = Tokenizer::new("");
        let result = tokenizer.inline_scanner("`Test`");
        let token1 = &result[0];
        match token1 {
            InlineToken::SpecialToken(token) => {
                assert_eq!(token.token, '`');
                match &token.inline_tokens[0] {
                    InlineToken::TextToken(text) => {
                        assert_eq!(text.text, "Test");
                    }
                    _ => panic!(),
                }
            }
            _ => panic!(),
        };
    }

    #[test]
    fn test_link_token() {
        let text = "[Link](http://a.com)";
        let tokenizer = Tokenizer::new("");
        let result = tokenizer.inline_scanner(text);
        assert_eq!(result.len(), 1);
        let token = &result[0];
        match token {
            InlineToken::LinkToken(token) => {
                assert_eq!(token.alt, "Link");
                assert_eq!(token.link, "http://a.com");
            }
            _ => panic!(),
        };
    }

    #[test]
    fn test_link_token_with_surround_text() {
        let text = "this is [Link](to_test) to test";
        let tokenizer = Tokenizer::new("");
        let result = tokenizer.inline_scanner(text);
        assert_eq!(result.len(), 3);
        let token = &result[0];
        if let InlineToken::TextToken(token) = token {
            assert_eq!(token.text, "this is ");
        } else {
            panic!();
        }
        if let InlineToken::LinkToken(token) = &result[1] {
            assert_eq!(token.alt, "Link");
            assert_eq!(token.link, "to_test");
        } else {
            panic!()
        }
        if let InlineToken::TextToken(token) = &result[2] {
            assert_eq!(token.text, " to test");
        } else {
            panic!()
        }
    }

    #[test]
    fn test_image_token() {
        let text = "![Link](http://a.com)";
        let tokenizer = Tokenizer::new("");
        let result = tokenizer.inline_scanner(text);
        assert_eq!(result.len(), 1);
        let token = &result[0];
        match token {
            InlineToken::ImageToken(token) => {
                assert_eq!(token.alt, "Link");
                assert_eq!(token.link, "http://a.com");
            }
            _ => panic!(),
        };
    }

    #[test]
    fn test_image_token_with_surround_text() {
        let text = "this is ![Link](to_test) to test";
        let tokenizer = Tokenizer::new("");
        let result = tokenizer.inline_scanner(text);
        assert_eq!(result.len(), 3);
        let token = &result[0];
        if let InlineToken::TextToken(token) = token {
            assert_eq!(token.text, "this is ");
        } else {
            panic!();
        }
        if let InlineToken::ImageToken(token) = &result[1] {
            assert_eq!(token.alt, "Link");
            assert_eq!(token.link, "to_test");
        } else {
            panic!()
        }
        if let InlineToken::TextToken(token) = &result[2] {
            assert_eq!(token.text, " to test");
        } else {
            panic!()
        }
    }

    #[test]
    fn test_all_special_tokens_with_no_usage() {
        let text = "![*_`";
        let tokenizer = Tokenizer::new("");
        let result = tokenizer.inline_scanner(text);
        assert_eq!(result.len(), 5);
        for i in 0..text.len() {
            if let InlineToken::TextToken(token) = &result[i] {
                assert_eq!(token.text, text.get(i..i + 1).unwrap());
            } else {
                panic!()
            }
        }
    }

}
