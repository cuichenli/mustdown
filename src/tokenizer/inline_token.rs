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
    pub fn new(token: char, inline_tokens: Vec<InlineToken>) -> Self {
        Self {
            token,
            inline_tokens
        }
    }
}
#[derive(Debug)]
pub struct DoubleSpecialToken {
    pub token: char,
    pub inline_tokens: Vec<InlineToken>,
}

impl DoubleSpecialToken {
    pub fn new(token: char, inline_tokens: Vec<InlineToken>) -> Self {
        Self {
            token,
            inline_tokens
        }
    }
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
pub mod tests {
    use super::*;
    use crate::Tokenizer;

    pub fn assert_special_token(token: &InlineToken, symbol: char) {
        if let InlineToken::SpecialToken(token) = token {
            assert_eq!(token.token, symbol);
        } else {
            println!("{:?}", token);
            panic!()
        }
    }

    pub fn assert_special_token_group(token: &InlineToken, text: &str, symbol: char) {
        assert_special_token(token, symbol);
        if let InlineToken::SpecialToken(t) = token {
            assert_eq!(t.inline_tokens.len(), 1 as usize);
            assert_text_token(&t.inline_tokens[0], text);
        } else {
            panic!();
        }
    }

    pub fn assert_double_special_token(token: &InlineToken, symbol: char) {
        if let InlineToken::DoubleSpecialToken(token) = token {
            assert_eq!(token.token, symbol);
        } else {
            println!("{:?}", token);
            panic!()
        }
    }
    pub fn assert_double_special_token_group(token: &InlineToken, text: &str, symbol: char) {
        assert_double_special_token(token, symbol);
        if let InlineToken::DoubleSpecialToken(t) = token {
            assert_eq!(t.inline_tokens.len(), 1 as usize);
            assert_text_token(&t.inline_tokens[0], text);
        } else {
            println!("{:?}", token);
            panic!();
        }
    }

    pub fn assert_text_token(token: &InlineToken, text: &str) {
        if let InlineToken::TextToken(token) = token {
            assert_eq!(token.text, text);
        } else {
            println!("{:?}", token);
            panic!();
        }
    }

    #[test]
    fn test_two_asterisk() {
        let text = "**";
        let result = Tokenizer::inline_scanner(text);
        assert_text_token(&result[0], "*");
        assert_text_token(&result[1], "*");
    }

    #[test]
    fn test_escape_asterisk() {
        let text = r"\*Test*";
        let result = Tokenizer::inline_scanner(text);
        assert_text_token(&result[0], r"\");
        assert_text_token(&result[1], "*Test");
        assert_text_token(&result[2], "*");
    }

    #[test]
    fn test_right_escape_asterisk() {
        let text = r"*Test\*";
        let result = Tokenizer::inline_scanner(text);
        assert_text_token(&result[0], "*");
        assert_text_token(&result[1], r"Test\");
        assert_text_token(&result[2], "*");
    }

    #[test]
    fn test_escape_double_asterisk() {
        let text = r"\**Test*";
        let result = Tokenizer::inline_scanner(text);
        assert_text_token(&result[0], r"\");
        assert_text_token(&result[1], r"*");
        assert_special_token_group(&result[2], "Test", '*');
    }

    #[test]
    fn test_escape_right_double_asterisk() {
        let text = r"*Test\**";
        let result = Tokenizer::inline_scanner(text);
        let token = &result[0];
        assert_special_token(token, '*');
        if let InlineToken::SpecialToken(t) = token {
            assert_eq!(t.inline_tokens.len(), 2 as usize);
            assert_text_token(&t.inline_tokens[0], r"Test\");
            assert_text_token(&t.inline_tokens[1], "*");
        } else {
            println!("{:?}", token);
            panic!();
        }
    }

    #[test]
    fn test_escape_more_double_asterisk1() {
        let text = r"\**Test**";
        let result = Tokenizer::inline_scanner(text);
        assert_text_token(&result[0], r"\");
        assert_text_token(&result[1], r"*");
        assert_special_token_group(&result[2], "Test", '*');
        assert_text_token(&result[3], r"*");
    }

    #[test]
    fn test_escape_more_double_asterisk2() {
        let text = r"**Test\**";
        let result = Tokenizer::inline_scanner(text);
        assert_text_token(&result[0], r"*");
        if let InlineToken::SpecialToken(t) = &result[1] {
            assert_eq!(t.inline_tokens.len(), 2 as usize);
            assert_text_token(&t.inline_tokens[0], r"Test\");
            assert_text_token(&t.inline_tokens[1], "*");
        } else {
            println!("{:?}", &result[1]);
            panic!();
        }
    }

    #[test]
    fn test_asterisk_and_underscore() {
        let text = r"*_Test*_";
        let result = Tokenizer::inline_scanner(text);
        if let InlineToken::SpecialToken(t) = &result[0] {
            assert_eq!(t.inline_tokens.len(), 2 as usize);
            assert_text_token(&t.inline_tokens[0], r"_");
            assert_text_token(&t.inline_tokens[1], "Test");
        } else {
            println!("{:?}", &result[1]);
            panic!();
        }
        assert_text_token(&result[1], r"_");
    }


    #[test]
    fn test_special_token() {
        let result = Tokenizer::inline_scanner("*Test*");
        assert_special_token_group(&result[0], "Test", '*');
    }

    #[test]
    fn test_special_token_start_with_space() {
        let result = Tokenizer::inline_scanner(" *Test*");
        assert_text_token(&result[0], " ");
        assert_special_token_group(&result[1], "Test", '*');
    }

    #[test]
    fn test_special_token_with_start_space_and_end_words() {
        let result = Tokenizer::inline_scanner(" *Test* another test");
        assert_eq!(result.len(), 3);
        assert_text_token(&result[0], " ");
        assert_special_token_group(&result[1], "Test", '*');
        assert_text_token(&result[2], " another test");
    }

    #[test]
    fn test_double_special_token() {
        let result = Tokenizer::inline_scanner("**Test**");
        assert_double_special_token_group(&result[0], "Test", '*');
    }

    #[test]
    fn test_double_special_token_with_start_space() {
        let result = Tokenizer::inline_scanner(" **Test**");
        assert_text_token(&result[0], " ");
        assert_double_special_token_group(&result[1], "Test", '*');
    }

    #[test]
    fn test_code_special_token() {
        let result = Tokenizer::inline_scanner("`Test`");
        let token1 = &result[0];
        assert_special_token(token1, '`');
    }

    #[test]
    fn test_link_token() {
        let text = "[Link](http://a.com)";
        let result = Tokenizer::inline_scanner(text);
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
        let result = Tokenizer::inline_scanner(text);
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
        let result = Tokenizer::inline_scanner(text);
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
        let result = Tokenizer::inline_scanner(text);
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
        let result = Tokenizer::inline_scanner(text);
        assert_eq!(result.len(), 5);
        assert_text_token(&result[0], "!");
        assert_text_token(&result[1], "[");
        assert_text_token(&result[2], "*");
        assert_text_token(&result[3], "_");
        assert_text_token(&result[4], "`");

    }

}
