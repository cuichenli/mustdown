extern crate regex;

pub mod inline_token;
pub mod line_token;

pub use inline_token::{
    DoubleSpecialToken, ImageToken, InlineToken, LinkToken, SpecialToken, TextToken,
};
pub use line_token::{CodeBlock, HeaderToken, LineToken, Paragraph, Quote};
use regex::Regex;
use std::collections::HashMap;

pub struct Tokenizer<'a> {
    text: &'a str,
    special_tokens: HashMap<char, &'a str>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(text: &'a str) -> Self {
        let special_tokens: HashMap<char, &str> =
            [('_', "_"), ('*', "\\*"), ('`', "`"), ('[', "["), ('!', "!")]
                .iter()
                .cloned()
                .collect();
        Self {
            text,
            special_tokens,
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

                        let token = HeaderToken {
                            level,
                            inline_tokens: self.inline_scanner(inner_text),
                        };
                        tokens.push(LineToken::HeaderToken(token));
                    }
                    None => (),
                }
            }
            _ => (),
        }
        let token = LineToken::Paragraph(Paragraph {
            inline_tokens: self.inline_scanner(inner_text),
        });
        tokens.push(token);
    }

    pub fn block_parser(&self, lines: &Vec<&str>, tokens: &mut Vec<LineToken>) {
        let text = lines.join("\n");
        let block = CodeBlock::new(text);
        tokens.push(LineToken::CodeBlock(block));
    }

    pub fn quote_parser(&self, lines: &Vec<&str>, tokens: &mut Vec<LineToken>) {
        let mut inline_tokens: Vec<InlineToken> = Vec::new();
        for l in lines {
            inline_tokens.append(&mut self.inline_scanner(l));
            inline_tokens.push(InlineToken::BreakToken);
        }
        inline_tokens.pop();
        let token = Quote { inline_tokens };
        tokens.push(LineToken::Quote(token));
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
                if chars[i] == '[' {
                    let re = Regex::new(r"\[(.*)\]\((.*)\)").unwrap();
                    let left_text = &inline_text[i..];
                    let caps = re.captures(left_text);
                    match caps {
                        Some(mat) => {
                            let length = mat.get(0).unwrap().as_str().len();
                            let alt = String::from(mat.get(1).unwrap().as_str());
                            let link = String::from(mat.get(2).unwrap().as_str());
                            i = i + length;
                            token = InlineToken::LinkToken(LinkToken { link, alt });
                        }
                        None => {
                            token = InlineToken::TextToken(TextToken {
                                text: chars[i].to_string(),
                            });
                            i += 1;
                        }
                    };
                } else if chars[i] == '!' {
                    let re = Regex::new(r"!\[(.*)\]\((.*)\)").unwrap();
                    let left_text = &inline_text[i..];
                    let caps = re.captures(left_text);
                    match caps {
                        Some(mat) => {
                            let length = mat.get(0).unwrap().as_str().len();
                            let alt = String::from(mat.get(1).unwrap().as_str());
                            let link = String::from(mat.get(2).unwrap().as_str());
                            i = i + length;
                            token = InlineToken::ImageToken(ImageToken { link, alt });
                        }
                        None => {
                            token = InlineToken::TextToken(TextToken {
                                text: chars[i].to_string(),
                            });
                            i += 1;
                        }
                    }
                } else {
                    let c = special_tokens.get(&chars[i]).unwrap();
                    let re = Regex::new(&format!(r"[^\\]?({}{}).*?({}{})", c, c, c, c)).unwrap();
                    let mut temp = i;
                    if i != 0 {
                        temp = i - 1;
                    }
                    let caps = re.find(&inline_text[temp..]);
                    match caps {
                        Some(mat) => {
                            let start = if temp < i { 3 } else { 2 };
                            let s = &mat.as_str()[start..mat.end() - 1];
                            token = InlineToken::DoubleSpecialToken(DoubleSpecialToken {
                                token: chars[i],
                                inline_tokens: self.inline_scanner(s),
                            });
                            i = temp + (mat.end() as usize);
                        }
                        None => {
                            let re = Regex::new(&format!(r"[^\\]?({}).*?({})", c, c)).unwrap();
                            let caps = re.find(&inline_text[temp..]);
                            match caps {
                                Some(mat) => {
                                    let start = if temp < i { 2 } else { 1 };
                                    let s = &mat.as_str()[start..mat.end() - 1];
                                    token = InlineToken::SpecialToken(SpecialToken {
                                        token: chars[i],
                                        inline_tokens: self.inline_scanner(s),
                                    });
                                    i = temp + (mat.end() as usize);
                                }
                                None => {
                                    token = InlineToken::TextToken(TextToken {
                                        text: chars[i].to_string(),
                                    });
                                    i = i + 1;
                                }
                            }
                        }
                    }
                }
            } else {
                let mut temp = i;
                while temp < n && !special_tokens.contains_key(&chars[temp]) {
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

    pub fn scanner(&self) -> Vec<LineToken> {
        let mut result: Vec<LineToken> = Vec::new();
        let lines = self.text.split("\n");
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
                self.block_parser(&block, &mut result);
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
                self.quote_parser(&temp, &mut result);
            } else {
                self.line_scanner(&line, &mut result);
            }
            i += 1;
        }
        result
    }
}
