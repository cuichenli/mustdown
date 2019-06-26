extern crate regex;
use regex::Regex;

const SPECIAL_TOKEN: &'static [char] = &['_', '*', '`', '[', '!'];

#[derive(Debug)]
pub enum InlineToken {
    TextToken(TextToken),
    SpecialToken(SpecialToken),
    DoubleSpecialToken(DoubleSpecialToken),
    LinkToken(LinkToken),
    ImageToken(ImageToken),
    BreakToken,
}

impl InlineToken {
    pub fn is_prev_backslash(text: &str, index: usize) -> bool {
        if index == 0 {
            false
        } else {
            text[index - 1..index] == *"\\"
        }
    }

    pub fn try_special_token(text: &str, first_token: &char) -> (Option<InlineToken>, usize) {
        // TODO: Clean this mess
        let re_symbol: &str;
        let temp = &[*first_token as u8];
        let borrow = std::str::from_utf8(temp).unwrap();
        let symbol = borrow.chars().next().unwrap();
        if symbol == '*' {
            re_symbol = r"\*";
        } else {
            re_symbol = borrow;
        }
        if let (Some(t), i) = DoubleSpecialToken::try_tokenize(text, symbol, re_symbol) {
            (Some(t), i)
        } else if let (Some(t), i) = SpecialToken::try_tokenize(text, symbol, re_symbol) {
            (Some(t), i)
        } else {
            (None, 0)
        }
    }

    pub fn get_text_token(text: String) -> InlineToken {
        InlineToken::TextToken(TextToken { text })
    }

    pub fn get_nth_cap(mat: &regex::Captures, n: usize) -> String {
        String::from(mat.get(n).unwrap().as_str())
    }

    pub fn get_alt_and_link(mat: &regex::Captures) -> (String, String) {
        let alt = InlineToken::get_nth_cap(&mat, 1);
        let link = InlineToken::get_nth_cap(&mat, 2);
        (alt, link)
    }

    pub fn tokenizer(inline_text: &str) -> Vec<InlineToken> {
        let mut tokens: Vec<InlineToken> = Vec::new();
        let n = inline_text.len();
        let chars: Vec<char> = inline_text.chars().collect();
        let special_tokens = SPECIAL_TOKEN;
        let mut i: usize = 0;
        while i < n {
            let token: InlineToken;
            if special_tokens.contains(&chars[i]) && !InlineToken::is_prev_backslash(inline_text, i)
            {
                let left_text = &inline_text[i..];
                if chars[i] == '[' {
                    if let Some(t) = LinkToken::try_tokenize(left_text) {
                        i = i + t.len();
                        token = InlineToken::LinkToken(t);
                    } else {
                        token = InlineToken::get_text_token(chars[i].to_string());
                        i += 1;
                    }
                } else if chars[i] == '!' {
                    if let Some(t) = ImageToken::try_tokenize(left_text) {
                        i = i + t.len();
                        token = InlineToken::ImageToken(t);
                    } else {
                        token = InlineToken::get_text_token(chars[i].to_string());
                        i += 1;
                    }
                } else {
                    let (option, step) = InlineToken::try_special_token(left_text, &chars[i]);
                    if let Some(t) = option {
                        i += step;
                        token = t;
                    } else {
                        token = InlineToken::get_text_token(chars[i].to_string());
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
            inline_tokens,
        }
    }

    pub fn try_tokenize(text: &str, symbol: char, re_symbol: &str) -> (Option<InlineToken>, usize) {
        let re = Regex::new(&format!(
            r"^{}([^{}]+?[^\\]{})",
            re_symbol, symbol, re_symbol
        ))
        .unwrap();
        let caps = re.captures(text);
        if let Some(mat) = caps {
            let inner_text = mat.get(1).unwrap().as_str();
            let token = SpecialToken::new(
                symbol,
                InlineToken::tokenizer(&inner_text[..inner_text.len() - 1]),
            );
            (Some(InlineToken::SpecialToken(token)), inner_text.len() + 1)
        } else {
            (None, 0)
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
            inline_tokens,
        }
    }

    pub fn try_tokenize(text: &str, symbol: char, re_symbol: &str) -> (Option<InlineToken>, usize) {
        let re = Regex::new(&format!(r"^{}{{2}}(.+?[^\\]{}{{2}})", re_symbol, re_symbol)).unwrap();
        let caps = re.captures(text);
        if let Some(mat) = caps {
            let inner_text = mat.get(1).unwrap().as_str();
            let token = DoubleSpecialToken::new(
                symbol,
                InlineToken::tokenizer(&inner_text[..inner_text.len() - 2]),
            );
            (
                Some(InlineToken::DoubleSpecialToken(token)),
                inner_text.len() + 2,
            )
        } else {
            (None, 0)
        }
    }
}
#[derive(Debug)]
pub struct LinkToken {
    pub alt: String,
    pub link: String,
    pub need_note: bool,
}

impl LinkToken {
    pub fn new(alt: String, link: String, need_note: bool) -> Self {
        Self { alt, link, need_note }
    }

    pub fn len(&self) -> usize {
        self.alt.len() + self.link.len() + 4
    }

    pub fn try_tokenize_with_real_link(text: &str) -> Option<LinkToken> {
        let re = Regex::new(r"\[(.*)\]\((.*)\)").unwrap();
        let caps = re.captures(text);
        if let Some(mat) = caps {
            let (alt, link) = InlineToken::get_alt_and_link(&mat);
            Some(LinkToken::new(alt, link, false))
        } else {
            None
        }
    }

    pub fn try_tokenize_with_need_note(text: &str) -> Option<LinkToken> {
        let re = Regex::new(r"\[(.*)\]\[(.*)\]").unwrap();
        let caps = re.captures(text);
        if let Some(mat) = caps {
            let (alt, link) = InlineToken::get_alt_and_link(&mat);
            Some(LinkToken::new(alt, link, true))
        } else {
            None
        }
    }

    pub fn try_tokenize(text: &str) -> Option<LinkToken> {
        if let Some(token) = LinkToken::try_tokenize_with_real_link(text) {
            Some(token)
        } else if let Some(token) = LinkToken::try_tokenize_with_need_note(text) {
            Some(token)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ImageToken {
    pub alt: String,
    pub link: String,
    pub need_note: bool,
}

impl ImageToken {
    pub fn new(alt: String, link: String, need_note: bool) -> Self {
        Self { alt, link, need_note }
    }

    pub fn len(&self) -> usize {
        self.alt.len() + self.link.len() + 5
    }

    pub fn try_tokenize_with_real_link(text: &str) -> Option<ImageToken> {
        let re = Regex::new(r"!\[(.*)\]\((.*)\)").unwrap();
        let caps = re.captures(text);
        if let Some(mat) = caps {
            let (alt, link) = InlineToken::get_alt_and_link(&mat);
            return Some(ImageToken::new(alt, link, false));
        }
        None
    }

    pub fn try_tokenize_with_need_note(text: &str) -> Option<ImageToken> {
        let re = Regex::new(r"!\[(.*)\]\[(.*)\]").unwrap();
        let caps = re.captures(text);
        if let Some(mat) = caps {
            let (alt, link) = InlineToken::get_alt_and_link(&mat);
            return Some(ImageToken::new(alt, link, true));
        }
        None
    }

    pub fn try_tokenize(text: &str) -> Option<ImageToken> {
        if let Some(token) = ImageToken::try_tokenize_with_real_link(text) {
            Some(token)
        } else if let Some(token) = ImageToken::try_tokenize_with_need_note(text) {
            Some(token)
        } else {
            None
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

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
        let result = InlineToken::tokenizer(text);
        assert_text_token(&result[0], "*");
        assert_text_token(&result[1], "*");
    }

    #[test]
    fn test_escape_asterisk() {
        let text = r"\*Test*";
        let result = InlineToken::tokenizer(text);
        assert_text_token(&result[0], r"\");
        assert_text_token(&result[1], "*Test");
        assert_text_token(&result[2], "*");
    }

    #[test]
    fn test_right_escape_asterisk() {
        let text = r"*Test\*";
        let result = InlineToken::tokenizer(text);
        assert_text_token(&result[0], "*");
        assert_text_token(&result[1], r"Test\");
        assert_text_token(&result[2], "*");
    }

    #[test]
    fn test_escape_double_asterisk() {
        let text = r"\**Test*";
        let result = InlineToken::tokenizer(text);
        assert_text_token(&result[0], r"\");
        assert_text_token(&result[1], r"*");
        assert_special_token_group(&result[2], "Test", '*');
    }

    #[test]
    fn test_escape_right_double_asterisk() {
        let text = r"*Test\**";
        let result = InlineToken::tokenizer(text);
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
        let result = InlineToken::tokenizer(text);
        assert_text_token(&result[0], r"\");
        assert_text_token(&result[1], r"*");
        assert_special_token_group(&result[2], "Test", '*');
        assert_text_token(&result[3], r"*");
    }

    #[test]
    fn test_escape_more_double_asterisk2() {
        let text = r"**Test\**";
        let result = InlineToken::tokenizer(text);
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
        let result = InlineToken::tokenizer(text);
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
        let result = InlineToken::tokenizer("*Test*");
        assert_special_token_group(&result[0], "Test", '*');
    }

    #[test]
    fn test_special_token_start_with_space() {
        let result = InlineToken::tokenizer(" *Test*");
        assert_text_token(&result[0], " ");
        assert_special_token_group(&result[1], "Test", '*');
    }

    #[test]
    fn test_special_token_with_start_space_and_end_words() {
        let result = InlineToken::tokenizer(" *Test* another test");
        assert_eq!(result.len(), 3);
        assert_text_token(&result[0], " ");
        assert_special_token_group(&result[1], "Test", '*');
        assert_text_token(&result[2], " another test");
    }

    #[test]
    fn test_double_special_token() {
        let result = InlineToken::tokenizer("**Test**");
        assert_double_special_token_group(&result[0], "Test", '*');
    }

    #[test]
    fn test_double_special_token_with_start_space() {
        let result = InlineToken::tokenizer(" **Test**");
        assert_text_token(&result[0], " ");
        assert_double_special_token_group(&result[1], "Test", '*');
    }

    #[test]
    fn test_code_special_token() {
        let result = InlineToken::tokenizer("`Test`");
        let token1 = &result[0];
        assert_special_token(token1, '`');
    }

    pub fn assert_link_token(token: &InlineToken, alt: &str, link: &str, need_note: bool) {
        match token {
            InlineToken::LinkToken(token) => {
                assert_eq!(token.alt, alt);
                assert_eq!(token.link, link);
                assert_eq!(token.need_note, need_note);
            }
            _ => panic!(),
        }
    }

    pub fn assert_image_token(token: &InlineToken, alt: &str, link: &str, need_note: bool) {
        match token {
            InlineToken::ImageToken(token) => {
                assert_eq!(token.alt, alt);
                assert_eq!(token.link, link);
                assert_eq!(token.need_note, need_note);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_link_token() {
        let text = "[Link](http://a.com)";
        let result = InlineToken::tokenizer(text);
        assert_eq!(result.len(), 1);
        assert_link_token(&result[0], "Link", "http://a.com", false);
    }

    #[test]
    fn test_link_with_need_note() {
        let text = "[Link][1]";
        let result = InlineToken::tokenizer(text);
        assert_eq!(result.len(), 1);
        assert_link_token(&result[0], "Link", "1", true);
    }

    #[test]
    fn test_link_token_with_surround_text() {
        let text = "this is [Link](to_test) to test";
        let result = InlineToken::tokenizer(text);
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
        let result = InlineToken::tokenizer(text);
        assert_eq!(result.len(), 1);
        assert_image_token(&result[0], "Link", "http://a.com", false);
        // let token = &result[0];
        // match token {
        //     InlineToken::ImageToken(token) => {
        //         assert_eq!(token.alt, "Link");
        //         assert_eq!(token.link, "http://a.com");
        //     }
        //     _ => panic!(),
        // };
    }

    #[test]
    fn test_image_token_with_need_note() {
        let text = "![Link][1]";
        let result = InlineToken::tokenizer(text);
        assert_eq!(result.len(), 1);
        assert_image_token(&result[0], "Link", "1", true);
        // let token = &result[0];
        // match token {
        //     InlineToken::ImageToken(token) => {
        //         assert_eq!(token.alt, "Link");
        //         assert_eq!(token.link, "http://a.com");
        //     }
        //     _ => panic!(),
        // };
    }

    #[test]
    fn test_image_token_with_surround_text() {
        let text = "this is ![Link](to_test) to test";
        let result = InlineToken::tokenizer(text);
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
        let result = InlineToken::tokenizer(text);
        assert_eq!(result.len(), 5);
        assert_text_token(&result[0], "!");
        assert_text_token(&result[1], "[");
        assert_text_token(&result[2], "*");
        assert_text_token(&result[3], "_");
        assert_text_token(&result[4], "`");
    }

}
