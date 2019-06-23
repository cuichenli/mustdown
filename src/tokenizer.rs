extern crate regex;

pub mod inline_token;
pub mod line_token;

pub use inline_token::{
    DoubleSpecialToken, ImageToken, InlineToken, LinkToken, SpecialToken, TextToken,
};
pub use line_token::{
    CodeBlock, HeaderToken, LineToken, OrderedList, OrderedListBlock, Paragraph, Quote,
    UnorderedList, UnorderedListBlock,
};
use regex::Regex;

const ORDERED_LIST: u8 = 1;
const UNORDERED_LIST: u8 = 2;
const NOT_LIST: u8 = 0;

pub struct Tokenizer {}

const SPECIAL_TOKEN: &'static [char] = &['_', '*', '`', '[', '!'];

impl Tokenizer {
    pub fn is_list(line: &str) -> Option<LineToken> {
        let re = Regex::new(r"^((?P<ordered>\d+\. )|(?P<unordered>[-*] ))(.+)").unwrap();
        let caps = re.captures(line);
        if let Some(mat) = caps {
            let left_text = mat.get(mat.len() - 1).unwrap().as_str();
            let inline_tokens = Tokenizer::inline_scanner(left_text);
            if let Some(_) = mat.name("ordered") {
                let token = OrderedList::new(inline_tokens);
                Some(LineToken::OrderedList(token))
            } else {
                let symbol = line.chars().next().unwrap();
                let token = UnorderedList::new(symbol, inline_tokens);
                Some(LineToken::UnorderedList(token))
            }
        } else {
            None
        }
    }

    pub fn is_prev_list(tokens: &Vec<LineToken>) -> u8 {
        let last = &tokens.last();
        if let Some(LineToken::UnorderedListBlock(_)) = last {
            return UNORDERED_LIST;
        } else if let Some(LineToken::OrderedListBlock(_)) = last {
            return ORDERED_LIST;
        }
        NOT_LIST
    }

    pub fn same_list_block_as_prev(token: &LineToken, tokens: &Vec<LineToken>) -> bool {
        let prev = Tokenizer::is_prev_list(tokens);
        if let LineToken::UnorderedList(_) = token {
            return prev == UNORDERED_LIST;
        } else if let LineToken::OrderedList(_) = token {
            return prev == ORDERED_LIST;
        }
        return false;
    }

    pub fn line_scanner(line: &str, tokens: &mut Vec<LineToken>) {
        let chars = line.as_bytes();
        let mut inner_text = line;
        match chars[0] as char {
            '#' => {
                let re = Regex::new(r"^(#{1,6}) (.*)").unwrap();
                let caps = re.captures(line);
                match caps {
                    Some(v) => {
                        let level = v.get(1).unwrap().as_str().len();
                        inner_text = v.get(2).unwrap().as_str();

                        let token = HeaderToken {
                            level,
                            inline_tokens: Tokenizer::inline_scanner(inner_text),
                        };
                        tokens.push(LineToken::HeaderToken(token));
                    }
                    None => (),
                }
            }
            _ => (),
        }
        let token = LineToken::Paragraph(Paragraph {
            inline_tokens: Tokenizer::inline_scanner(inner_text),
        });
        tokens.push(token);
    }

    pub fn block_parser(lines: &Vec<&str>, tokens: &mut Vec<LineToken>) {
        let text = lines.join("\n");
        let block = CodeBlock::new(text);
        tokens.push(LineToken::CodeBlock(block));
    }

    pub fn quote_parser(lines: &Vec<&str>, tokens: &mut Vec<LineToken>) {
        let mut inline_tokens: Vec<InlineToken> = Vec::new();
        for l in lines {
            inline_tokens.append(&mut Tokenizer::inline_scanner(l));
            inline_tokens.push(InlineToken::BreakToken);
        }
        inline_tokens.pop();
        let token = Quote { inline_tokens };
        tokens.push(LineToken::Quote(token));
    }

    pub fn try_image_token(text: &str) -> Option<ImageToken> {
        let re = Regex::new(r"!\[(.*)\]\((.*)\)").unwrap();
        let caps = re.captures(text);
        if let Some(mat) = caps {
            let (alt, link) = Tokenizer::get_alt_and_link(&mat);
            return Some(ImageToken::new(alt, link));
        }
        None
    }

    pub fn try_link_token(text: &str) -> Option<LinkToken> {
        let re = Regex::new(r"\[(.*)\]\((.*)\)").unwrap();
        let caps = re.captures(text);
        if let Some(mat) = caps {
            let (alt, link) = Tokenizer::get_alt_and_link(&mat);
            return Some(LinkToken::new(alt, link));
        }
        None
    }

    pub fn try_special_token(text: &str, first_token: &char) -> (Option<InlineToken>, usize) {
        // TODO: Clean this mess
        let c: &str;
        let temp = &[*first_token as u8];
        let borrow = std::str::from_utf8(temp).unwrap();
        let f = borrow.chars().next().unwrap();
        if f == '*' {
            c = r"\*";
        } else {
            c = borrow;
        }
        let re = Regex::new(&format!(r"^{}{{2}}(.+?[^\\]{}{{2}})", c, c)).unwrap();
        let caps = re.captures(text);
        if let Some(mat) = caps {
            let inner_text = mat.get(1).unwrap().as_str();
            let token = DoubleSpecialToken::new(
                f,
                Tokenizer::inline_scanner(&inner_text[..inner_text.len() - 2]),
            );
            return (
                Some(InlineToken::DoubleSpecialToken(token)),
                inner_text.len() + 2,
            );
        }
        let re = Regex::new(&format!(r"^{}([^{}]+?[^\\]{})", c, borrow, c)).unwrap();
        let caps = re.captures(text);
        if let Some(mat) = caps {
            let inner_text = mat.get(1).unwrap().as_str();
            let token = SpecialToken::new(
                f,
                Tokenizer::inline_scanner(&inner_text[..inner_text.len() - 1]),
            );
            return (Some(InlineToken::SpecialToken(token)), inner_text.len() + 1);
        }

        (None, 0)
    }

    pub fn get_text_token(text: String) -> InlineToken {
        InlineToken::TextToken(TextToken { text })
    }

    pub fn get_nth_cap(mat: &regex::Captures, n: usize) -> String {
        String::from(mat.get(n).unwrap().as_str())
    }

    pub fn get_alt_and_link(mat: &regex::Captures) -> (String, String) {
        let alt = Tokenizer::get_nth_cap(&mat, 1);
        let link = Tokenizer::get_nth_cap(&mat, 2);
        (alt, link)
    }

    pub fn is_prev_backslash(text: &str, index: usize) -> bool {
        if index == 0 {
            false
        } else {
            text[index - 1..index] == *"\\"
        }
    }

    pub fn inline_scanner(inline_text: &str) -> Vec<InlineToken> {
        let mut tokens: Vec<InlineToken> = Vec::new();
        let n = inline_text.len();
        let chars: Vec<char> = inline_text.chars().collect();
        let special_tokens = SPECIAL_TOKEN;
        let mut i: usize = 0;
        while i < n {
            let token: InlineToken;
            if special_tokens.contains(&chars[i]) && !Tokenizer::is_prev_backslash(inline_text, i) {
                let left_text = &inline_text[i..];
                if chars[i] == '[' {
                    if let Some(t) = Tokenizer::try_link_token(left_text) {
                        i = i + t.len();
                        token = InlineToken::LinkToken(t);
                    } else {
                        token = Tokenizer::get_text_token(chars[i].to_string());
                        i += 1;
                    }
                } else if chars[i] == '!' {
                    if let Some(t) = Tokenizer::try_image_token(left_text) {
                        i = i + t.len();
                        token = InlineToken::ImageToken(t);
                    } else {
                        token = Tokenizer::get_text_token(chars[i].to_string());
                        i += 1;
                    }
                } else {
                    let (option, step) = Tokenizer::try_special_token(left_text, &chars[i]);
                    if let Some(t) = option {
                        i += step;
                        token = t;
                    } else {
                        token = Tokenizer::get_text_token(chars[i].to_string());
                        i += 1;
                    }
                }
            } else {
                let mut temp = i + 1;
                while temp < n && !special_tokens.contains(&chars[temp]) {
                    temp += 1;
                }
                token = InlineToken::TextToken(TextToken {
                    text: inline_text[i..temp].to_string(),
                });
                i = temp;
            }
            tokens.push(token);
        }
        tokens
    }

    pub fn scanner(text: &str) -> Vec<LineToken> {
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
                let mut block: Vec<&str> = Vec::new();
                i += 1;
                while i < lines.len() && lines[i] != "```" {
                    block.push(lines[i]);
                    i += 1;
                }
                // result should not be passed in to this function
                Tokenizer::block_parser(&block, &mut result);
            } else if line[0..1] == *">" {
                let mut temp = vec![&lines[i][1..]];
                i += 1;
                while i < lines.len() && lines[i].ends_with("  ") {
                    temp.push(lines[i]);
                    i += 1;
                }
                if i < lines.len() && lines[i - 1].ends_with("  ") {
                    temp.push(lines[i]);
                } else {
                    i -= 1;
                }
                Tokenizer::quote_parser(&temp, &mut result);
            } else if let Some(token) = Tokenizer::is_list(line) {
                if Tokenizer::same_list_block_as_prev(&token, &result) {
                    let last = result.last_mut().unwrap();
                    match last {
                        LineToken::OrderedListBlock(t) => t.ordered_lists.push(token),
                        LineToken::UnorderedListBlock(t) => t.unordered_lists.push(token),
                        _ => panic!(),
                    }
                } else {
                    match token {
                        LineToken::UnorderedList(_) => result.push(LineToken::UnorderedListBlock(
                            UnorderedListBlock::new(token),
                        )),
                        LineToken::OrderedList(_) => {
                            result.push(LineToken::OrderedListBlock(OrderedListBlock::new(token)))
                        }
                        _ => panic!(),
                    }
                }
            } else {
                Tokenizer::line_scanner(&line, &mut result);
            }
            i += 1;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_is_list() {
        let l = "- this";
        Tokenizer::is_list(l).unwrap();
        let result = Tokenizer::is_list("1. this");
        if result.is_none() {
            panic!();
        }
        let result = Tokenizer::is_list("* this");
        if result.is_none() {
            panic!();
        }
        let result = Tokenizer::is_list("2. this");
        if result.is_none() {
            panic!();
        }
        let result = Tokenizer::is_list("23. this");
        if result.is_none() {
            panic!();
        }
    }

    #[test]
    fn test_is_list_should_panic() {
        let l = "-this";
        if Tokenizer::is_list(l).is_some() {
            panic!();
        }
        if Tokenizer::is_list("1.this").is_some() {
            panic!();
        }
        if Tokenizer::is_list("*dafsa").is_some() {
            panic!();
        }
        if Tokenizer::is_list("-").is_some() {
            panic!();
        }
        if Tokenizer::is_list("- ").is_some() {
            panic!();
        }
        if Tokenizer::is_list("1. ").is_some() {
            panic!();
        }
        if Tokenizer::is_list("* ").is_some() {
            panic!();
        }
    }

    #[test]
    fn test_is_prev_list() {
        let tokens = vec![LineToken::OrderedListBlock(OrderedListBlock {
            ordered_lists: vec![],
        })];
        assert_eq!(Tokenizer::is_prev_list(&tokens), ORDERED_LIST);
        let tokens = vec![LineToken::UnorderedListBlock(UnorderedListBlock {
            unordered_lists: vec![],
        })];
        assert_eq!(Tokenizer::is_prev_list(&tokens), UNORDERED_LIST);
        let tokens = vec![LineToken::Paragraph(Paragraph {
            inline_tokens: vec![],
        })];
        assert_eq!(Tokenizer::is_prev_list(&tokens), NOT_LIST);
        let tokens = vec![];
        assert_eq!(Tokenizer::is_prev_list(&tokens), NOT_LIST);
    }

}
