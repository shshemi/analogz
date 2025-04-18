#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TokenValue<'a> {
    Start,
    End,
    Alphabetic(&'a str, usize, usize),
    Numeric(&'a str, usize, usize),
    AlphaNumeric(&'a str, usize, usize),
    Symbolic(&'a str, usize),
    Whitespace(&'a str, usize),
}

impl<'a> TokenValue<'a> {
    pub fn str(&self) -> Option<&str> {
        match self {
            TokenValue::Start => None,
            TokenValue::End => None,
            TokenValue::Alphabetic(slice, _, _) => Some(slice),
            TokenValue::Numeric(slice, _, _) => Some(slice),
            TokenValue::AlphaNumeric(slice, _, _) => Some(slice),
            TokenValue::Symbolic(slice, _) => Some(slice),
            TokenValue::Whitespace(slice, _) => Some(slice),
        }
    }

    pub fn u32(&self) -> Option<u32> {
        match &self {
            TokenValue::Numeric(slice, _, _) => slice.parse::<u32>().ok(),
            _ => None,
        }
    }

    pub fn char(&self) -> Option<char> {
        match &self {
            TokenValue::Symbolic(slice, _) | TokenValue::Whitespace(slice, _) => {
                slice.chars().next()
            }
            _ => None,
        }
    }

    fn new(val: &'a str, start: usize, end: usize) -> Self {
        if val.len() == 1 {
            let c = val.chars().next().unwrap();
            if c.is_whitespace() {
                TokenValue::Whitespace(val, start)
            } else if c.is_ascii_punctuation() {
                TokenValue::Symbolic(val, start)
            } else if c.is_numeric() {
                TokenValue::Numeric(val, start, end)
            } else {
                TokenValue::Alphabetic(val, start, end)
            }
        } else if val.chars().all(|c| c.is_alphabetic()) {
            TokenValue::Alphabetic(val, start, end)
        } else if val.chars().all(|c| c.is_numeric()) {
            TokenValue::Numeric(val, start, end)
        } else {
            TokenValue::AlphaNumeric(val, start, end)
        }
    }
}

#[derive(Debug)]
enum TokenIterState {
    Start,
    Text,
    End,
}

#[derive(Debug)]
pub struct TokenIter<'a> {
    slice: &'a str,
    offset: usize,
    state: TokenIterState,
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = TokenValue<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            TokenIterState::Start => {
                self.state = TokenIterState::Text;
                Some(TokenValue::Start)
            }
            TokenIterState::Text if self.slice.is_empty() => {
                self.state = TokenIterState::End;
                Some(TokenValue::End)
            }
            TokenIterState::Text => {
                let sidx = self
                    .slice
                    .char_indices()
                    .find(|(_, c)| c.is_whitespace() || c.is_ascii_punctuation());
                match sidx {
                    Some((0, c)) => {
                        let start = self.offset;
                        let end = c.len_utf8();
                        let (slice, next) = self.slice.split_at(end);
                        self.slice = next;
                        self.offset += end;
                        Some(TokenValue::new(slice, start, end))
                    }
                    Some((i, _)) => {
                        let start = self.offset;
                        let end = i;
                        let (slice, next) = self.slice.split_at(end);
                        self.slice = next;
                        self.offset += end;
                        Some(TokenValue::new(slice, start, end))
                    }
                    None => {
                        let start = self.offset;
                        let end = self.slice.len();
                        let (slice, next) = self.slice.split_at(end);
                        self.slice = next;
                        self.offset += end;
                        Some(TokenValue::new(slice, start, end))
                    }
                }
            }
            TokenIterState::End => None,
        }
    }
}

pub trait Tokenize {
    fn tokenize(&self) -> TokenIter;
}

impl Tokenize for str {
    fn tokenize(&self) -> TokenIter {
        TokenIter {
            slice: self,
            offset: 0,
            state: TokenIterState::Start,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let input = "hello123 !world 456";
        let tokens: Vec<TokenValue> = input.tokenize().collect();

        // Check all token types
        assert_eq!(tokens[0], TokenValue::Start);

        match &tokens[1] {
            TokenValue::AlphaNumeric(s, _, _) => assert_eq!(*s, "hello123"),
            _ => panic!("Expected AlphaNumeric token"),
        }

        match &tokens[2] {
            TokenValue::Whitespace(s, _) => assert_eq!(*s, " "),
            _ => panic!("Expected Whitespace token"),
        }

        match &tokens[3] {
            TokenValue::Symbolic(s, _) => assert_eq!(*s, "!"),
            _ => panic!("Expected Symbolic token"),
        }

        match &tokens[4] {
            TokenValue::Alphabetic(s, _, _) => assert_eq!(*s, "world"),
            _ => panic!("Expected Alphabetic token"),
        }

        match &tokens[5] {
            TokenValue::Whitespace(s, _) => assert_eq!(*s, " "),
            _ => panic!("Expected Whitespace token"),
        }

        match &tokens[6] {
            TokenValue::Numeric(s, _, _) => assert_eq!(*s, "456"),
            _ => panic!("Expected Numeric token"),
        }

        assert_eq!(tokens[7], TokenValue::End);
        assert_eq!(tokens.len(), 8);
    }

    #[test]
    fn test_empty_string() {
        let input = "";
        let tokens: Vec<TokenValue> = input.tokenize().collect();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], TokenValue::Start);
        assert_eq!(tokens[1], TokenValue::End);
    }

    #[test]
    fn test_whitespace_only() {
        let input = "   ";
        let tokens: Vec<TokenValue> = input.tokenize().collect();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], TokenValue::Start);
        match &tokens[1] {
            TokenValue::Whitespace(s, _) => assert_eq!(*s, " "),
            _ => panic!("Expected Whitespace token"),
        }
        match &tokens[2] {
            TokenValue::Whitespace(s, _) => assert_eq!(*s, " "),
            _ => panic!("Expected Whitespace token"),
        }
        match &tokens[3] {
            TokenValue::Whitespace(s, _) => assert_eq!(*s, " "),
            _ => panic!("Expected Whitespace token"),
        }
        assert_eq!(tokens[4], TokenValue::End);
    }

    #[test]
    fn test_multiple_consecutive_punctuation() {
        let input = "!!!";
        let tokens: Vec<TokenValue> = input.tokenize().collect();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], TokenValue::Start);
        match &tokens[1] {
            TokenValue::Symbolic(s, _) => assert_eq!(*s, "!"),
            _ => panic!("Expected Symbolic token"),
        }
        match &tokens[2] {
            TokenValue::Symbolic(s, _) => assert_eq!(*s, "!"),
            _ => panic!("Expected Symbolic token"),
        }
        match &tokens[3] {
            TokenValue::Symbolic(s, _) => assert_eq!(*s, "!"),
            _ => panic!("Expected Symbolic token"),
        }
        assert_eq!(tokens[4], TokenValue::End);
    }

    #[test]
    fn test_unicode_characters() {
        let input = "你 好 世 界";
        let tokens: Vec<TokenValue> = input.tokenize().collect();
        assert_eq!(tokens.len(), 9); // Start + 4 characters + 3 spaces + End
        assert_eq!(tokens[0], TokenValue::Start);

        match &tokens[1] {
            TokenValue::Alphabetic(s, _, _) => assert_eq!(*s, "你"),
            _ => panic!("Expected Alphabetic token"),
        }

        match &tokens[2] {
            TokenValue::Whitespace(s, _) => assert_eq!(*s, " "),
            _ => panic!("Expected Whitespace token"),
        }

        match &tokens[3] {
            TokenValue::Alphabetic(s, _, _) => assert_eq!(*s, "好"),
            _ => panic!("Expected Alphabetic token"),
        }

        match &tokens[4] {
            TokenValue::Whitespace(s, _) => assert_eq!(*s, " "),
            _ => panic!("Expected Whitespace token"),
        }

        match &tokens[5] {
            TokenValue::Alphabetic(s, _, _) => assert_eq!(*s, "世"),
            _ => panic!("Expected Alphabetic token"),
        }

        match &tokens[6] {
            TokenValue::Whitespace(s, _) => assert_eq!(*s, " "),
            _ => panic!("Expected Whitespace token"),
        }

        match &tokens[7] {
            TokenValue::Alphabetic(s, _, _) => assert_eq!(*s, "界"),
            _ => panic!("Expected Alphabetic token"),
        }

        assert_eq!(tokens[8], TokenValue::End);
    }

    #[test]
    fn test_mixed_unicode_and_ascii() {
        let input = "hello世界123";
        let tokens: Vec<TokenValue> = input.tokenize().collect();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], TokenValue::Start);
        match &tokens[1] {
            TokenValue::AlphaNumeric(s, _, _) => assert_eq!(*s, "hello世界123"),
            _ => panic!("Expected AlphaNumeric token"),
        }
        assert_eq!(tokens[2], TokenValue::End);
    }

    #[test]
    fn test_numeric_token_parsing() {
        let input = "12345";
        let tokens: Vec<TokenValue> = input.tokenize().collect();
        assert_eq!(tokens.len(), 3);
        match &tokens[1] {
            TokenValue::Numeric(s, _, _) => {
                assert_eq!(*s, "12345");
                assert_eq!(tokens[1].u32(), Some(12345));
            }
            _ => panic!("Expected Numeric token"),
        }
    }

    #[test]
    fn test_char_extraction() {
        let input = "! ";
        let tokens: Vec<TokenValue> = input.tokenize().collect();
        match &tokens[1] {
            TokenValue::Symbolic(_, _) => assert_eq!(tokens[1].char(), Some('!')),
            _ => panic!("Expected Symbolic token"),
        }
        match &tokens[2] {
            TokenValue::Whitespace(_, _) => assert_eq!(tokens[2].char(), Some(' ')),
            _ => panic!("Expected Whitespace token"),
        }
    }

    #[test]
    fn test_str_method() {
        let input = "abc 123 !";
        let tokens: Vec<TokenValue> = input.tokenize().collect();

        assert_eq!(tokens[0].str(), None); // Start token
        assert_eq!(tokens[1].str(), Some("abc")); // Alphabetic
        assert_eq!(tokens[2].str(), Some(" ")); // Whitespace
        assert_eq!(tokens[3].str(), Some("123")); // Numeric
        assert_eq!(tokens[4].str(), Some(" ")); // Whitespace
        assert_eq!(tokens[5].str(), Some("!")); // Symbolic
        assert_eq!(tokens[6].str(), None); // End token
    }

    #[test]
    fn test_u32_max_value() {
        let input = "4294967295"; // u32::MAX
        let tokens: Vec<TokenValue> = input.tokenize().collect();

        match &tokens[1] {
            TokenValue::Numeric(_, _, _) => assert_eq!(tokens[1].u32(), Some(u32::MAX)),
            _ => panic!("Expected Numeric token"),
        }
    }

    #[test]
    fn test_u32_overflow() {
        let input = "4294967296"; // u32::MAX + 1
        let tokens: Vec<TokenValue> = input.tokenize().collect();

        match &tokens[1] {
            TokenValue::Numeric(_, _, _) => assert_eq!(tokens[1].u32(), None),
            _ => panic!("Expected Numeric token"),
        }
    }

    #[test]
    fn test_long_token() {
        let long_string = "a".repeat(10000);
        let tokens: Vec<TokenValue> = long_string.tokenize().collect();

        assert_eq!(tokens.len(), 3);
        match &tokens[1] {
            TokenValue::Alphabetic(s, _, _) => assert_eq!(s.len(), 10000),
            _ => panic!("Expected Alphabetic token"),
        }
    }
}
