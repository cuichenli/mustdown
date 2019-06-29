extern crate regex;

pub mod inline_token;
pub mod line_token;

pub use inline_token::{
    DoubleSpecialToken, ImageToken, InlineToken, LinkToken, SpecialToken, TextToken,
};
pub use line_token::{
    CodeBlock, HeaderToken, LineToken, NoteToken, OrderedList, OrderedListBlock, Paragraph, Quote,
    UnorderedList, UnorderedListBlock,
};

pub struct Tokenizer {}

impl Tokenizer {
    pub fn tokenizer(text: &str) -> Vec<LineToken> {
        let mut result: Vec<LineToken> = Vec::new();
        let lines = text.split("\n");
        let lines: Vec<&str> = lines.collect();
        let mut i: usize = 0;
        while i < lines.len() {
            let line = lines[i];
            if line == "" {
                i += 1;
                continue;
            }
            if line == "```" {
                let (token, temp) = CodeBlock::tokenizer(&lines, i);
                i = temp;
                result.push(token);
            } else if line[0..1] == *">" {
                let (token, index) = Quote::tokenizer(&lines, i);
                i = index;
                result.push(token);
            } else if let Some(token) = LineToken::is_list(line) {
                if LineToken::same_list_block_as_prev(&token, &result) {
                    LineToken::push_to_last_list_block(&mut result, token);
                } else {
                    let token = LineToken::new_list_block(token);
                    result.push(token);
                }
            } else if let Some(token) = HeaderToken::try_tokenize(line) {
                result.push(token);
            } else if let Some(token) = NoteToken::try_tokenize(line) {
                result.push(token);
            } else if LineToken::is_horizontal_rule(line) {
                result.push(LineToken::HorizontalRule)
            } else {
                let token = Paragraph::tokenizer(line);
                result.push(token);
            }
            i += 1;
        }
        result
    }
}
