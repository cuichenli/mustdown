extern crate regex;
use regex::Regex;
use std::collections::HashMap;


pub enum Token {
    HeaderToken {
        level: usize,
        inline_tokens: Vec<InlineToken>
    },
    Paragraph { inline_tokens: Vec<InlineToken> }
}

pub enum InlineToken {
    TextToken (String),
    SpecialToken {
        token: char,
        inline_tokens: Vec<InlineToken>
    }
}

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

    pub fn line_scanner(&self, line: &str, tokens: &mut Vec<Token>) {
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

                        let token = Token::HeaderToken {
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
        tokens.push(Token::Paragraph {
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

    pub fn scanner(&self) -> Vec<Token> {
        let mut result: Vec<Token> = Vec::new();
        let lines = self.text.split("\n");
        let lines: Vec<&str> = lines.collect();
        for line in lines {
            &mut self.line_scanner(line, &mut result);
        }
        result
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_scanner_header_token() {
        let tokenizer = Tokenizer::new("## Test");
        let result = tokenizer.scanner();
        let head_token = &result[0];
        let level: usize;
        let inline_token: &Vec<InlineToken>;
        match head_token {
            Token::HeaderToken{ level: l, inline_tokens: i} => {
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

    #[test]
    fn test_special_token() {
        let tokenizer = Tokenizer::new("* Test *");
        let result = tokenizer.scanner();
        let inline_tokens: &Vec<InlineToken>;
        match &result[0] {
            Token::Paragraph{  inline_tokens: i } => {
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
            Token::Paragraph{  inline_tokens: i } => {
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
            Token::Paragraph{ inline_tokens: t } => {
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
