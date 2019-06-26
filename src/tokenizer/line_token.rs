use super::inline_token::InlineToken;
extern crate regex;
use regex::Regex;

#[derive(Debug)]
pub enum LineToken {
    HeaderToken(HeaderToken),
    Paragraph(Paragraph),
    CodeBlock(CodeBlock),
    Quote(Quote),
    OrderedListBlock(OrderedListBlock),
    UnorderedListBlock(UnorderedListBlock),
    OrderedList(OrderedList),
    UnorderedList(UnorderedList),
    NoteToken(NoteToken)
}

const ORDERED_LIST: char = '1';
const NOT_LIST: char = 'a';

impl LineToken {
    pub fn new_list_block(token: LineToken) -> LineToken {
        match token {
            LineToken::UnorderedList(_) => {
                LineToken::UnorderedListBlock(UnorderedListBlock::new(token))
            }
            LineToken::OrderedList(_) => LineToken::OrderedListBlock(OrderedListBlock::new(token)),
            _ => panic!(),
        }
    }

    pub fn is_list(line: &str) -> Option<LineToken> {
        let re = Regex::new(r"^((?P<ordered>\d+\. )|(?P<unordered>[-*] ))(.+)").unwrap();
        let caps = re.captures(line);
        if let Some(mat) = caps {
            let left_text = mat.get(mat.len() - 1).unwrap().as_str();
            let inline_tokens = InlineToken::tokenizer(left_text);
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

    pub fn is_prev_list(tokens: &Vec<LineToken>) -> char {
        let last = &tokens.last();
        if let Some(LineToken::UnorderedListBlock(t)) = last {
            t.get_symbol()
        } else if let Some(LineToken::OrderedListBlock(_)) = last {
            ORDERED_LIST
        } else {
            NOT_LIST
        }
    }

    pub fn same_list_block_as_prev(token: &LineToken, tokens: &Vec<LineToken>) -> bool {
        let prev = LineToken::is_prev_list(tokens);
        if let LineToken::UnorderedList(t) = token {
            return t.symbol == prev;
        } else if let LineToken::OrderedList(_) = token {
            return prev == ORDERED_LIST;
        }
        return false;
    }

    pub fn push_to_last_list_block(tokens: &mut Vec<LineToken>, token: LineToken) {
        let last = tokens.last_mut().unwrap();
        match last {
            LineToken::OrderedListBlock(t) => t.push(token),
            LineToken::UnorderedListBlock(t) => t.push(token),
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct HeaderToken {
    pub level: usize,
    pub inline_tokens: Vec<InlineToken>,
}

impl HeaderToken {
    pub fn try_tokenize(line: &str) -> Option<LineToken> {
        let re = Regex::new(r"^(#{1,6}) (.*)").unwrap();
        let caps = re.captures(line);
        match caps {
            Some(v) => {
                let level = v.get(1).unwrap().as_str().len();
                let inner_text = v.get(2).unwrap().as_str();
                let token = HeaderToken {
                    level,
                    inline_tokens: InlineToken::tokenizer(inner_text),
                };
                Some(LineToken::HeaderToken(token))
            }
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct Paragraph {
    pub inline_tokens: Vec<InlineToken>,
}

impl Paragraph {
    pub fn tokenizer(line: &str) -> LineToken {
        LineToken::Paragraph(Paragraph {
            inline_tokens: InlineToken::tokenizer(line),
        })
    }
}

#[derive(Debug)]
pub struct CodeBlock {
    pub text: String,
}

impl CodeBlock {
    pub fn new(text: String) -> Self {
        Self { text }
    }

    pub fn tokenizer(lines: &Vec<&str>, mut index: usize) -> (LineToken, usize) {
        index = index + 1;
        let mut block: Vec<&str> = Vec::new();
        while index < lines.len() && lines[index] != "```" {
            block.push(lines[index]);
            index += 1;
        }
        let text = block.join("\n");
        let block = CodeBlock::new(text);
        let token = LineToken::CodeBlock(block);
        (token, index)
    }
}
#[derive(Debug)]
pub struct Quote {
    pub inline_tokens: Vec<InlineToken>,
}

impl Quote {
    pub fn tokenizer(lines: &Vec<&str>, mut index: usize) -> (LineToken, usize) {
        let mut temp = vec![&lines[index][1..]];
        index += 1;
        while index < lines.len() && lines[index].ends_with("  ") {
            temp.push(lines[index]);
            index += 1;
        }
        if index < lines.len() && lines[index - 1].ends_with("  ") {
            temp.push(lines[index]);
        } else {
            index -= 1;
        };
        let mut inline_tokens: Vec<InlineToken> = Vec::new();
        for l in temp {
            inline_tokens.append(&mut InlineToken::tokenizer(l));
            inline_tokens.push(InlineToken::BreakToken);
        }
        inline_tokens.pop();
        let token = Quote { inline_tokens };
        (LineToken::Quote(token), index)
    }
}

#[derive(Debug)]
pub struct OrderedListBlock {
    pub lists: Vec<LineToken>,
}

impl OrderedListBlock {
    pub fn new(token: LineToken) -> Self {
        if let LineToken::OrderedList(_) = token {
            Self { lists: vec![token] }
        } else {
            panic!()
        }
    }

    pub fn push(&mut self, token: LineToken) {
        match token {
            LineToken::OrderedList(_) => self.lists.push(token),
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct UnorderedListBlock {
    pub symbol: char,
    pub lists: Vec<LineToken>,
}

impl UnorderedListBlock {
    pub fn new(token: LineToken) -> Self {
        if let LineToken::UnorderedList(ref t) = token {
            let symbol = t.symbol;
            Self {
                symbol,
                lists: vec![token],
            }
        } else {
            panic!()
        }
    }

    pub fn get_symbol(&self) -> char {
        self.symbol
    }

    pub fn push(&mut self, token: LineToken) {
        match token {
            LineToken::UnorderedList(_) => self.lists.push(token),
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct OrderedList {
    pub inline_tokens: Vec<InlineToken>,
}

impl OrderedList {
    pub fn new(inline_tokens: Vec<InlineToken>) -> Self {
        Self { inline_tokens }
    }
}

#[derive(Debug)]
pub struct UnorderedList {
    pub inline_tokens: Vec<InlineToken>,
    pub symbol: char,
}

impl UnorderedList {
    pub fn new(symbol: char, inline_tokens: Vec<InlineToken>) -> Self {
        Self {
            symbol,
            inline_tokens,
        }
    }
}

#[derive(Debug)]
pub struct NoteToken {
    pub name: String,
    pub link: String,
}

impl NoteToken {

    pub fn new(name: String, link: String) -> Self {
        Self { name, link }
    }
    pub fn try_tokenize(line: &str) -> Option<LineToken> {
        let re = Regex::new(r"\[(.*)\]:(.*)").unwrap();
        if let Some(mat) = re.captures(line) {
            let name = String::from(mat.get(1).unwrap().as_str());
            let link = String::from(mat.get(2).unwrap().as_str());
            Some(LineToken::NoteToken(NoteToken::new(name, link)))
        } else {
            None
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::super::inline_token::tests::{assert_special_token_group, assert_text_token};
    use super::*;
    use crate::Tokenizer;

    pub fn assert_unordered_block_symbol(token: &LineToken, symbol: char) {
        if let LineToken::UnorderedListBlock(token) = token {
            assert_eq!(token.get_symbol(), symbol);
        } else {
            panic!();
        }
    }
    #[test]
    fn test_is_list() {
        let l = "- this";
        LineToken::is_list(l).unwrap();
        let result = LineToken::is_list("1. this");
        if result.is_none() {
            panic!();
        }
        let result = LineToken::is_list("* this");
        if result.is_none() {
            panic!();
        }
        let result = LineToken::is_list("2. this");
        if result.is_none() {
            panic!();
        }
        let result = LineToken::is_list("23. this");
        if result.is_none() {
            panic!();
        }
    }

    #[test]
    fn test_is_list_should_panic() {
        let l = "-this";
        if LineToken::is_list(l).is_some() {
            panic!();
        }
        if LineToken::is_list("1.this").is_some() {
            panic!();
        }
        if LineToken::is_list("*dafsa").is_some() {
            panic!();
        }
        if LineToken::is_list("-").is_some() {
            panic!();
        }
        if LineToken::is_list("- ").is_some() {
            panic!();
        }
        if LineToken::is_list("1. ").is_some() {
            panic!();
        }
        if LineToken::is_list("* ").is_some() {
            panic!();
        }
    }

    #[test]
    fn test_is_prev_list() {
        let tokens = vec![LineToken::OrderedListBlock(OrderedListBlock {
            lists: vec![],
        })];
        assert_eq!(LineToken::is_prev_list(&tokens), ORDERED_LIST);
        let unordered_list = LineToken::UnorderedList(UnorderedList::new('*', vec![]));
        let tokens = vec![LineToken::UnorderedListBlock(UnorderedListBlock {
            symbol: '*',
            lists: vec![unordered_list],
        })];
        assert_eq!(LineToken::is_prev_list(&tokens), '*');
        let tokens = vec![LineToken::Paragraph(Paragraph {
            inline_tokens: vec![],
        })];
        assert_eq!(LineToken::is_prev_list(&tokens), NOT_LIST);
        let tokens = vec![];
        assert_eq!(LineToken::is_prev_list(&tokens), NOT_LIST);
    }

    #[test]
    fn test_is_prev_same_block() {
        let unordered_list = LineToken::UnorderedList(UnorderedList::new('*', vec![]));
        let block = &vec![LineToken::UnorderedListBlock(UnorderedListBlock::new(
            unordered_list,
        ))];
        let unordered_list = LineToken::UnorderedList(UnorderedList::new('*', vec![]));
        assert!(LineToken::same_list_block_as_prev(&unordered_list, block));
        let unordered_list = LineToken::UnorderedList(UnorderedList::new('-', vec![]));
        assert_eq!(
            false,
            LineToken::same_list_block_as_prev(&unordered_list, block)
        );
    }

    #[test]
    fn test_line_scanner_header_token() {
        let result = Tokenizer::tokenizer("## Test");
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

    #[test]
    fn test_scanner_with_code_block() {
        let text = "```\n\
                    this is a test \n\
                    ```";
        let result = Tokenizer::tokenizer(text);
        assert_eq!(result.len(), 1);
        let token = result.first().unwrap();
        if let LineToken::CodeBlock(token) = token {
            assert_eq!(token.text, "this is a test ");
        } else {
            panic!();
        }
    }

    #[test]
    fn test_scanner_with_code_block_and_paragraph() {
        let text = "```\n\
                    this is a test \n\
                    ```\n\
                    this is another test";
        let result = Tokenizer::tokenizer(text);
        assert_eq!(result.len(), 2);
        let token = result.first().unwrap();
        if let LineToken::CodeBlock(token) = token {
            assert_eq!(token.text, "this is a test ");
        } else {
            panic!();
        }

        let token = result.last().unwrap();
        if let LineToken::Paragraph(token) = token {
            assert_eq!(token.inline_tokens.len(), 1);
            if let InlineToken::TextToken(token) = token.inline_tokens.first().unwrap() {
                assert_eq!(token.text, "this is another test")
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_quote_with_multiple_lines() {
        let text = ">this is  \na quote";
        let result = Tokenizer::tokenizer(text);
        assert_eq!(result.len(), 1);
        if let LineToken::Quote(token) = &result[0] {
            let inline_tokens = &token.inline_tokens;
            assert_eq!(inline_tokens.len(), 3);
            if let InlineToken::TextToken(token) = &inline_tokens[0] {
                assert_eq!(token.text, "this is  ");
            } else {
                panic!();
            }
            if let InlineToken::BreakToken = &inline_tokens[1] {
                assert!(true);
            } else {
                panic!();
            }
            if let InlineToken::TextToken(token) = &inline_tokens[2] {
                assert_eq!(token.text, "a quote");
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_quote_with_single_line() {
        let text = ">this is\n";
        let result = Tokenizer::tokenizer(text);
        assert_eq!(result.len(), 1);
        if let LineToken::Quote(token) = &result[0] {
            let inline_tokens = &token.inline_tokens;
            assert_eq!(inline_tokens.len(), 1);
            if let InlineToken::TextToken(token) = &inline_tokens[0] {
                assert_eq!(token.text, "this is");
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_quote_with_inline_tokens_with_single_line() {
        let text = ">this is*a bold*\n";
        let result = Tokenizer::tokenizer(text);
        assert_eq!(result.len(), 1);
        if let LineToken::Quote(token) = &result[0] {
            let inline_tokens = &token.inline_tokens;
            assert_eq!(inline_tokens.len(), 2);
            assert_text_token(&inline_tokens[0], "this is");
            assert_special_token_group(&inline_tokens[1], "a bold", '*');
        } else {
            panic!();
        }
    }

    #[test]
    fn test_single_line_quote_with_lines_surrounding() {
        let text = "first paragraph\n>a quote\nsecond paragraph";
        let result = Tokenizer::tokenizer(text);
        assert_eq!(result.len(), 3);
        if let LineToken::Quote(token) = &result[1] {
            assert_eq!(token.inline_tokens.len(), 1);
            if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                assert_eq!(token.text, "a quote");
            }
        }

        if let LineToken::Paragraph(token) = &result[0] {
            assert_eq!(token.inline_tokens.len(), 1);
            if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                assert_eq!(token.text, "first paragraph");
            }
        } else {
            panic!();
        }
        if let LineToken::Paragraph(token) = &result[2] {
            assert_eq!(token.inline_tokens.len(), 1);
            if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                assert_eq!(token.text, "second paragraph");
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_multiple_lines_quote_with_lines_surrounding() {
        let text = "first paragraph\n>a quote  \nanother quote\nsecond paragraph";
        let result = Tokenizer::tokenizer(text);
        assert_eq!(result.len(), 3);
        if let LineToken::Quote(token) = &result[1] {
            assert_eq!(token.inline_tokens.len(), 3);
            if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                assert_eq!(token.text, "a quote  ");
            }
            if let InlineToken::TextToken(token) = &token.inline_tokens[2] {
                assert_eq!(token.text, "another quote");
            }
            if let InlineToken::BreakToken = &token.inline_tokens[1] {
                assert!(true);
            } else {
                panic!();
            }
        }

        if let LineToken::Paragraph(token) = &result[0] {
            assert_eq!(token.inline_tokens.len(), 1);
            if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                assert_eq!(token.text, "first paragraph");
            }
        } else {
            panic!();
        }
        if let LineToken::Paragraph(token) = &result[2] {
            assert_eq!(token.inline_tokens.len(), 1);
            if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                assert_eq!(token.text, "second paragraph");
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_quote_with_empty_quote() {
        let text = ">";
        let result = Tokenizer::tokenizer(text);
        assert_eq!(result.len(), 1);
        if let LineToken::Quote(token) = &result[0] {
            assert_eq!(token.inline_tokens.len(), 0);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_ordered_list() {
        let text = "1. this\n2. is\n3. a\n4. test";
        let result = Tokenizer::tokenizer(text);
        assert_eq!(result.len(), 1);
        if let LineToken::OrderedListBlock(token) = &result[0] {
            let list = &token.lists;
            assert_eq!(list.len(), 4);
            if let LineToken::OrderedList(token) = &list[0] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "this");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::OrderedList(token) = &list[1] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "is");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::OrderedList(token) = &list[2] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "a");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::OrderedList(token) = &list[3] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "test");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_unordered_list() {
        let text = "- this\n- is\n- a\n- test";
        let result = Tokenizer::tokenizer(text);
        assert_eq!(result.len(), 1);
        if let LineToken::UnorderedListBlock(token) = &result[0] {
            let list = &token.lists;
            assert_eq!(list.len(), 4);
            if let LineToken::UnorderedList(token) = &list[0] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "this");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::UnorderedList(token) = &list[1] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "is");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::UnorderedList(token) = &list[2] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "a");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::UnorderedList(token) = &list[3] {
                if let InlineToken::TextToken(token) = &token.inline_tokens[0] {
                    assert_eq!(token.text, "test");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_two_different_symbol_unordered_list() {
        let text = "- first\n* second";
        let result = Tokenizer::tokenizer(text);
        assert_eq!(result.len(), 2);
        assert_unordered_block_symbol(&result[0], '-');
        assert_unordered_block_symbol(&result[1], '*');
    }

    #[test]
    fn test_unordered_list_with_lines_surrounded() {
        let text = "a simple line\n- a list\n- another\ntest";
        let result = Tokenizer::tokenizer(text);
        assert_eq!(result.len(), 3);
        if let LineToken::Paragraph(token) = &result[0] {
            let tokens = &token.inline_tokens;
            assert_eq!(tokens.len(), 1);
            if let InlineToken::TextToken(token) = &tokens[0] {
                assert_eq!(token.text, "a simple line");
            } else {
                panic!();
            }
        } else {
            panic!();
        }

        if let LineToken::UnorderedListBlock(token) = &result[1] {
            let tokens = &token.lists;
            assert_eq!(tokens.len(), 2);
            if let LineToken::UnorderedList(token) = &tokens[0] {
                let list = &token.inline_tokens;
                assert_eq!(list.len(), 1);
                if let InlineToken::TextToken(token) = &list[0] {
                    assert_eq!(token.text, "a list");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::UnorderedList(token) = &tokens[1] {
                let list = &token.inline_tokens;
                assert_eq!(list.len(), 1);
                if let InlineToken::TextToken(token) = &list[0] {
                    assert_eq!(token.text, "another");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }

        if let LineToken::Paragraph(token) = &result[2] {
            let tokens = &token.inline_tokens;
            assert_eq!(tokens.len(), 1);
            if let InlineToken::TextToken(token) = &tokens[0] {
                assert_eq!(token.text, "test");
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_ordered_list_with_lines_surrounded() {
        let text = "a simple line\n1. a list\n2. another\ntest";
        let result = Tokenizer::tokenizer(text);
        assert_eq!(result.len(), 3);
        if let LineToken::Paragraph(token) = &result[0] {
            let tokens = &token.inline_tokens;
            assert_eq!(tokens.len(), 1);
            if let InlineToken::TextToken(token) = &tokens[0] {
                assert_eq!(token.text, "a simple line");
            } else {
                panic!();
            }
        } else {
            panic!();
        }

        if let LineToken::OrderedListBlock(token) = &result[1] {
            let tokens = &token.lists;
            assert_eq!(tokens.len(), 2);
            if let LineToken::OrderedList(token) = &tokens[0] {
                let list = &token.inline_tokens;
                assert_eq!(list.len(), 1);
                if let InlineToken::TextToken(token) = &list[0] {
                    assert_eq!(token.text, "a list");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            if let LineToken::OrderedList(token) = &tokens[1] {
                let list = &token.inline_tokens;
                assert_eq!(list.len(), 1);
                if let InlineToken::TextToken(token) = &list[0] {
                    assert_eq!(token.text, "another");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }

        if let LineToken::Paragraph(token) = &result[2] {
            let tokens = &token.inline_tokens;
            assert_eq!(tokens.len(), 1);
            if let InlineToken::TextToken(token) = &tokens[0] {
                assert_eq!(token.text, "test");
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    pub fn assert_note_token(token: &LineToken, name: &str, link: &str) {
        if let LineToken::NoteToken(token) = token {
            assert_eq!(token.name, name);
            assert_eq!(token.link, link);
        } else {
            panic!()
        }
    }

    #[test]
    fn test_note_token() {
        let text = "[1]:http://a.com";
        let result = NoteToken::try_tokenize(text);
        match result {
            Some(token) => {
                assert_note_token(&token, "1", "http://a.com");
            }
            _ => panic!()
        }
    }

    #[test]
    fn test_note_token_in_tokenizer() {
        let text = "[1]:http://a.com";
        let result = Tokenizer::tokenizer(text);
        assert_eq!(result.len(), 1);
        assert_note_token(&result[0], "1", "http://a.com")
    }
}
