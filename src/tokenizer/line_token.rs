use super::inline_token::InlineToken;

pub enum LineToken {
    HeaderToken(HeaderToken),
    Paragraph(Paragraph),
}

pub struct HeaderToken {
    pub level: usize,
    pub inline_tokens: Vec<InlineToken>,
}

pub struct Paragraph {
    pub inline_tokens: Vec<InlineToken>,
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tokenizer;

    #[test]
    fn test_line_scanner_header_token() {
        let tokenizer = Tokenizer::new("## Test");
        let result = tokenizer.scanner();
        let head_token = &result[0];
        let level: usize;
        let inline_token: &Vec<InlineToken>;
        match head_token {
            LineToken::HeaderToken(token) => {
                level = token.level;
                inline_token = &token.inline_tokens;
            }
            _ => panic!(),
        };
        assert_eq!(level, 2 as usize);
        match &inline_token[0] {
            InlineToken::TextToken(t) => assert_eq!(t.text, "Test"),
            _ => panic!(),
        };
    }
}
