use crate::arc_str::ArcStr;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Token {
    Start,
    End,
    Alphabetic(ArcStr),
    Numeric(ArcStr),
    AlphaNumeric(ArcStr),
    Symbolic(ArcStr),
    Whitespace(ArcStr),
}

impl Token {
    pub fn str(&self) -> Option<&str> {
        match self {
            Token::Start => None,
            Token::End => None,
            Token::Alphabetic(slice) => Some(slice),
            Token::Numeric(slice) => Some(slice),
            Token::AlphaNumeric(slice) => Some(slice),
            Token::Symbolic(slice) => Some(slice),
            Token::Whitespace(slice) => Some(slice),
        }
    }

    pub fn u32(&self) -> Option<u32> {
        match &self {
            Token::Numeric(slice) => slice.parse::<u32>().ok(),
            _ => None,
        }
    }

    pub fn char(&self) -> Option<char> {
        match &self {
            Token::Symbolic(slice) | Token::Whitespace(slice) => slice.chars().next(),
            _ => None,
        }
    }

    fn new(val: ArcStr) -> Self {
        if val.len() == 1 {
            let c = val.chars().next().unwrap();
            if c.is_whitespace() {
                Token::Whitespace(val)
            } else if c.is_ascii_punctuation() {
                Token::Symbolic(val)
            } else if c.is_numeric() {
                Token::Numeric(val)
            } else {
                Token::Alphabetic(val)
            }
        } else if val.chars().all(|c| c.is_alphabetic()) {
            Token::Alphabetic(val)
        } else if val.chars().all(|c| c.is_numeric()) {
            Token::Numeric(val)
        } else {
            Token::AlphaNumeric(val)
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
pub struct TokenIter {
    slice: ArcStr,
    state: TokenIterState,
}

impl Iterator for TokenIter {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            TokenIterState::Start => {
                self.state = TokenIterState::Text;
                Some(Token::Start)
            }
            TokenIterState::Text if self.slice.is_empty() => {
                self.state = TokenIterState::End;
                Some(Token::End)
            }
            TokenIterState::Text => {
                let sidx = self
                    .slice
                    .char_indices()
                    .find(|(_, c)| c.is_whitespace() || c.is_ascii_punctuation());
                match sidx {
                    Some((0, c)) => {
                        let idx = c.len_utf8();
                        let (slice, next) = self.slice.split_at(idx);
                        self.slice = next;
                        Some(Token::new(slice))
                    }
                    Some((idx, _)) => {
                        let (slice, next) = self.slice.split_at(idx);
                        self.slice = next;
                        Some(Token::new(slice))
                    }
                    None => {
                        let (slice, next) = self.slice.split_at(self.slice.len());
                        self.slice = next;
                        Some(Token::new(slice))
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

impl Tokenize for ArcStr {
    fn tokenize(&self) -> TokenIter {
        TokenIter {
            slice: self.clone(),
            state: TokenIterState::Start,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let input = ArcStr::from("hello123 !world 456");
        let tokens: Vec<Token> = input.tokenize().collect();

        // Check all token types
        assert_eq!(tokens[0], Token::Start);

        match &tokens[1] {
            Token::AlphaNumeric(s) => assert_eq!(s.as_str(), "hello123"),
            _ => panic!("Expected AlphaNumeric token"),
        }

        match &tokens[2] {
            Token::Whitespace(s) => assert_eq!(s.as_str(), " "),
            _ => panic!("Expected Whitespace token"),
        }

        match &tokens[3] {
            Token::Symbolic(s) => assert_eq!(s.as_str(), "!"),
            _ => panic!("Expected Symbolic token"),
        }

        match &tokens[4] {
            Token::Alphabetic(s) => assert_eq!(s.as_str(), "world"),
            _ => panic!("Expected Alphabetic token"),
        }

        match &tokens[5] {
            Token::Whitespace(s) => assert_eq!(s.as_str(), " "),
            _ => panic!("Expected Whitespace token"),
        }

        match &tokens[6] {
            Token::Numeric(s) => assert_eq!(s.as_str(), "456"),
            _ => panic!("Expected Numeric token"),
        }

        assert_eq!(tokens[7], Token::End);
        assert_eq!(tokens.len(), 8);
    }

    #[test]
    fn test_empty_string() {
        let input = ArcStr::from("");
        let tokens: Vec<Token> = input.tokenize().collect();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Start);
        assert_eq!(tokens[1], Token::End);
    }

    #[test]
    fn test_whitespace_only() {
        let input = ArcStr::from("   ");
        let tokens: Vec<Token> = input.tokenize().collect();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::Start);
        match &tokens[1] {
            Token::Whitespace(s) => assert_eq!(s.as_str(), " "),
            _ => panic!("Expected Whitespace token"),
        }
        match &tokens[2] {
            Token::Whitespace(s) => assert_eq!(s.as_str(), " "),
            _ => panic!("Expected Whitespace token"),
        }
        match &tokens[3] {
            Token::Whitespace(s) => assert_eq!(s.as_str(), " "),
            _ => panic!("Expected Whitespace token"),
        }
        assert_eq!(tokens[4], Token::End);
    }

    #[test]
    fn test_multiple_consecutive_punctuation() {
        let input = ArcStr::from("!!!");
        let tokens: Vec<Token> = input.tokenize().collect();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::Start);
        match &tokens[1] {
            Token::Symbolic(s) => assert_eq!(s.as_str(), "!"),
            _ => panic!("Expected Symbolic token"),
        }
        match &tokens[2] {
            Token::Symbolic(s) => assert_eq!(s.as_str(), "!"),
            _ => panic!("Expected Symbolic token"),
        }
        match &tokens[3] {
            Token::Symbolic(s) => assert_eq!(s.as_str(), "!"),
            _ => panic!("Expected Symbolic token"),
        }
        assert_eq!(tokens[4], Token::End);
    }

    #[test]
    fn test_unicode_characters() {
        let input = ArcStr::from("你 好 世 界");
        let tokens: Vec<Token> = input.tokenize().collect();
        assert_eq!(tokens.len(), 9); // Start + 4 characters + 3 spaces + End
        assert_eq!(tokens[0], Token::Start);

        match &tokens[1] {
            Token::Alphabetic(s) => assert_eq!(s.as_str(), "你"),
            _ => panic!("Expected Alphabetic token"),
        }

        match &tokens[2] {
            Token::Whitespace(s) => assert_eq!(s.as_str(), " "),
            _ => panic!("Expected Whitespace token"),
        }

        match &tokens[3] {
            Token::Alphabetic(s) => assert_eq!(s.as_str(), "好"),
            _ => panic!("Expected Alphabetic token"),
        }

        match &tokens[4] {
            Token::Whitespace(s) => assert_eq!(s.as_str(), " "),
            _ => panic!("Expected Whitespace token"),
        }

        match &tokens[5] {
            Token::Alphabetic(s) => assert_eq!(s.as_str(), "世"),
            _ => panic!("Expected Alphabetic token"),
        }

        match &tokens[6] {
            Token::Whitespace(s) => assert_eq!(s.as_str(), " "),
            _ => panic!("Expected Whitespace token"),
        }

        match &tokens[7] {
            Token::Alphabetic(s) => assert_eq!(s.as_str(), "界"),
            _ => panic!("Expected Alphabetic token"),
        }

        assert_eq!(tokens[8], Token::End);
    }

    #[test]
    fn test_mixed_unicode_and_ascii() {
        let input = ArcStr::from("hello世界123");
        let tokens: Vec<Token> = input.tokenize().collect();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Start);
        match &tokens[1] {
            Token::AlphaNumeric(s) => assert_eq!(s.as_str(), "hello世界123"),
            _ => panic!("Expected AlphaNumeric token"),
        }
        assert_eq!(tokens[2], Token::End);
    }

    #[test]
    fn test_numeric_token_parsing() {
        let input = ArcStr::from("12345");
        let tokens: Vec<Token> = input.tokenize().collect();
        assert_eq!(tokens.len(), 3);
        match &tokens[1] {
            Token::Numeric(s) => {
                assert_eq!(s.as_str(), "12345");
                assert_eq!(tokens[1].u32(), Some(12345));
            }
            _ => panic!("Expected Numeric token"),
        }
    }

    #[test]
    fn test_char_extraction() {
        let input = ArcStr::from("! ");
        let tokens: Vec<Token> = input.tokenize().collect();
        match &tokens[1] {
            Token::Symbolic(_) => assert_eq!(tokens[1].char(), Some('!')),
            _ => panic!("Expected Symbolic token"),
        }
        match &tokens[2] {
            Token::Whitespace(_) => assert_eq!(tokens[2].char(), Some(' ')),
            _ => panic!("Expected Whitespace token"),
        }
    }

    #[test]
    fn test_str_method() {
        let input = ArcStr::from("abc 123 !");
        let tokens: Vec<Token> = input.tokenize().collect();

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
        let input = ArcStr::from("4294967295"); // u32::MAX
        let tokens: Vec<Token> = input.tokenize().collect();

        match &tokens[1] {
            Token::Numeric(_) => assert_eq!(tokens[1].u32(), Some(u32::MAX)),
            _ => panic!("Expected Numeric token"),
        }
    }

    #[test]
    fn test_u32_overflow() {
        let input = ArcStr::new("4294967296"); // u32::MAX + 1
        let tokens: Vec<Token> = input.tokenize().collect();

        match &tokens[1] {
            Token::Numeric(_) => assert_eq!(tokens[1].u32(), None),
            _ => panic!("Expected Numeric token"),
        }
    }

    #[test]
    fn test_long_token() {
        let long_string = ArcStr::from("a".repeat(10000));
        let tokens: Vec<Token> = long_string.tokenize().collect();

        assert_eq!(tokens.len(), 3);
        match &tokens[1] {
            Token::Alphabetic(s) => assert_eq!(s.len(), 10000),
            _ => panic!("Expected Alphabetic token"),
        }
    }
}
