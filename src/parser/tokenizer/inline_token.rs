
pub enum InlineToken {
    TextToken (String),
    SpecialToken {
        token: char,
        inline_tokens: Vec<InlineToken>
    }
}


#[cfg(test)]
mod tests {
    use super::super::line_token::LineToken;
    use super::*;
    use crate::parser::Tokenizer;

    #[test]
    fn test_special_token() {
        let tokenizer = Tokenizer::new("* Test *");
        let result = tokenizer.scanner();
        let inline_tokens: &Vec<InlineToken>;
        match &result[0] {
            LineToken::Paragraph{  inline_tokens: i } => {
                inline_tokens = i;
            },
            _ => panic!()
        };
        assert_eq!(inline_tokens.len(), 1);
        let result = &inline_tokens[0];
        let inline_tokens: &Vec<InlineToken>;
        match result {
            InlineToken::SpecialToken{ token: t, inline_tokens: i} => {
                assert_eq!(*t, '*');
                assert_eq!(i.len(), 1);
                inline_tokens = i;
            },
            _ => panic!()
        };
        match &inline_tokens[0] {
            InlineToken::TextToken(text) => {
                assert_eq!(text, "Test");
            },
            _ => panic!()
        };
    }

    #[test]
    fn test_special_token_start_with_space() {
        let tokenizer = Tokenizer::new(" * Test *");
        let result = tokenizer.scanner();
        let inline_tokens: &Vec<InlineToken>;
        match &result[0] {
            LineToken::Paragraph{  inline_tokens: i } => {
                inline_tokens = i;
            },
            _ => panic!()
        };
        assert_eq!(inline_tokens.len(), 2);
        let result = &inline_tokens[1];
        let text_token = &inline_tokens[0];
        match text_token {
            InlineToken::TextToken(text) => {
                assert_eq!(text, " ");
            },
            _ => panic!()
        };
        let inline_tokens: &Vec<InlineToken>;
        match result {
            InlineToken::SpecialToken{ token: t, inline_tokens: i} => {
                assert_eq!(*t, '*');
                assert_eq!(i.len(), 1);
                inline_tokens = i;
            },
            _ => {
                panic!()
            }
        };
        match &inline_tokens[0] {
            InlineToken::TextToken(text) => {
                assert_eq!(text, "Test");
            },
            _ => panic!()
        };
    }

    #[test]
    fn test_special_token_with_start_space_and_end_words() {
        let tokenizer = Tokenizer::new(" * Test * another test");
        let result = tokenizer.scanner();
        assert_eq!(result.len(), 1);
        let inline_token = &result[0];
        let tokens: &Vec<InlineToken>;
        match inline_token {
            LineToken::Paragraph{ inline_tokens: t } => {
                tokens = &t;
            },
            _ => panic!()
        };
        assert_eq!(tokens.len(), 3);
        match &tokens[0] {
            InlineToken::TextToken(text) => {
                assert_eq!(text, &" ");
            },
            _ => panic!()
        };
        match &tokens[1] {
            InlineToken::SpecialToken{ token: t, inline_tokens: ts} => {
                assert_eq!(t, &'*');
                assert_eq!(ts.len(), 1);
                match &ts[0] {
                    InlineToken::TextToken(text) => {
                        assert_eq!(text, &"Test");
                    },
                    _ => panic!()
                };
            },
            _ => panic!()
        };
        match &tokens[2] {
            InlineToken::TextToken(text) => {
                assert_eq!(text, &" another test")
            },
            _ => panic!()
        };

    }
}