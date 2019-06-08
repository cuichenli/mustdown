use super::inline_token::InlineToken;

pub enum LineToken {
    HeaderToken {
        level: usize,
        inline_tokens: Vec<InlineToken>
    },
    Paragraph { inline_tokens: Vec<InlineToken> }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Tokenizer;

    #[test]
    fn test_line_scanner_header_token() {
        let tokenizer = Tokenizer::new("## Test");
        let result = tokenizer.scanner();
        let head_token = &result[0];
        let level: usize;
        let inline_token: &Vec<InlineToken>;
        match head_token {
            LineToken::HeaderToken{ level: l, inline_tokens: i} => {
                level = *l;
                inline_token = i;
            },
            _ => panic!()
        };
        assert_eq!(level, 2 as usize);
        match &inline_token[0] {
            InlineToken::TextToken(t) => assert_eq!(t, "Test"),
            _ => panic!()
        };
    }
}
