use regex::Regex;


const SPECIAL_TOKENS: [char; 2] = ['#', '*'];

#[derive(Debug)]
pub enum Token {
    TextToken { text: String},
    HeaderToken(usize)
}

pub struct Tokenizer<'a> {
    text: &'a str,
}

impl <'a> Tokenizer <'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text: text,
        }
    }

    pub fn line_scanner(&self, line: &str, tokens: &mut Vec<Token>) {
        let chars = line.as_bytes();
        match chars[0] as char {
            '#' => {
                let re = Regex::new(r"^(#{1,6}) (.*)").unwrap();
                let caps = re.captures(line);
                match caps {
                    Some(v) => {
                        let level = v.get(1).unwrap().as_str().len();
                        let token = Token::HeaderToken(level);
                        tokens.push(token);
                        let token = Token::TextToken {
                            text: String::from(v.get(2).unwrap().as_str())
                        };
                        tokens.push(token)
                    },
                    None => ()
                }
            },
            _ => ()
        }
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


}
