extern crate regex;

pub mod inline_token;
pub mod line_token;

use regex::Regex;
use std::collections::HashMap;
pub use line_token::LineToken;
pub use inline_token::InlineToken;


pub struct Tokenizer<'a> {
    text: &'a str,
    special_tokens: HashMap<char, &'a str>
}

impl <'a> Tokenizer <'a> {
    pub fn new(text: &'a str) -> Self {
        let special_tokens: HashMap<char, &str> = [
            ('#', "#"),
            ('*', "\\*")].iter().cloned().collect();
        Self {
            text: text,
            special_tokens: special_tokens
        }
    }

    pub fn line_scanner(&self, line: &str, tokens: &mut Vec<LineToken>) {
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

                        let token = LineToken::HeaderToken {
                            level: level,
                            inline_tokens: self.inline_scanner(inner_text)
                        };
                        tokens.push(token);
                    },
                    None => ()
                }
            },
            _ => ()
        }
        tokens.push(LineToken::Paragraph {
            inline_tokens: self.inline_scanner(inner_text)
        })
    }

    pub fn inline_scanner(&self, inline_text: &str) -> Vec<InlineToken> {
        let mut tokens: Vec<InlineToken> = Vec::new();
        let n = inline_text.len();
        let chars: Vec<char> = inline_text.chars().collect();
        let special_tokens = &self.special_tokens;
        let mut i: usize = 0;
        while i < n {
            let token: InlineToken;
            if special_tokens.contains_key(&chars[i]) {
                let c = special_tokens.get(&chars[i]).unwrap();
                let re = Regex::new(&format!(r"[^\\]?({}).*? ({})", c, c)).unwrap();
                let mut temp = i;
                if i != 0 {
                    temp = i - 1;
                }
                let caps = re.find(&inline_text[temp..]);
                match caps {
                    Some(mat) => {
                        let start = if temp < i {
                            3
                        } else {
                            2
                        };
                        let s = &mat.as_str()[start..mat.end()-2];
                        token = InlineToken::SpecialToken {
                            token: chars[i],
                            inline_tokens: self.inline_scanner(s)
                        };
                        i = temp + (mat.end() as usize);
                    },
                    None => {
                        token = InlineToken::TextToken(chars[i].to_string());
                        i = i + 1;
                    }
                }
            } else {
                let mut temp = i;
                while temp < n && !special_tokens.contains_key(&chars[temp]) {
                    temp += 1;
                }
                token = InlineToken::TextToken(inline_text[i..temp].to_string());
                i = temp;
            }
            tokens.push(token);
        }
        tokens
    }

    pub fn scanner(&self) -> Vec<LineToken> {
        let mut result: Vec<LineToken> = Vec::new();
        let lines = self.text.split("\n");
        let lines: Vec<&str> = lines.collect();
        for line in lines {
            &mut self.line_scanner(line, &mut result);
        }
        result
    }
}