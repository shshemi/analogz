use crate::{arc_str::ArcStr, tokenizer};

#[derive(Debug, Clone)]
pub enum Token {
    Start,
    End,
    Numeric(u32),
    Range(u32, u32),
    String(ArcStr),
    Symbolic(char),
    Whitespace(char),
    Unkown,
}

impl From<tokenizer::Token> for Token {
    fn from(value: tokenizer::Token) -> Self {
        match value {
            tokenizer::Token::Start => Token::Start,
            tokenizer::Token::End => Token::End,
            tokenizer::Token::Alphabetic(s) | tokenizer::Token::AlphaNumeric(s) => Token::String(s),
            tokenizer::Token::Symbolic(_) | tokenizer::Token::Whitespace(_) => {
                Token::Whitespace(value.char().unwrap())
            }
            tokenizer::Token::Numeric(_) => Token::Numeric(value.u32().unwrap()),
        }
    }
}

impl Token {
    pub fn with(a: tokenizer::Token, b: tokenizer::Token) -> Self {
        match (a, b) {
            (tokenizer::Token::Start, tokenizer::Token::Start) => Token::Start,
            (tokenizer::Token::End, tokenizer::Token::End) => Token::End,
            (tokenizer::Token::Alphabetic(a), tokenizer::Token::Alphabetic(b)) if a == b => {
                Token::String(a)
            }
            (tokenizer::Token::AlphaNumeric(a), tokenizer::Token::AlphaNumeric(b)) if a == b => {
                Token::String(a)
            }
            (tokenizer::Token::Symbolic(a), tokenizer::Token::Symbolic(b)) if a == b => {
                Token::Symbolic(a.chars().next().unwrap())
            }
            (tokenizer::Token::Whitespace(a), tokenizer::Token::Whitespace(b)) if a == b => {
                Token::Symbolic(a.chars().next().unwrap())
            }
            (tokenizer::Token::Numeric(a), tokenizer::Token::Numeric(b)) => {
                let a = a.parse::<u32>().unwrap();
                let b = b.parse::<u32>().unwrap();
                if a == b {
                    Token::Numeric(a)
                } else {
                    Token::Range(a.min(b), a.max(b))
                }
            }
            _ => Token::Unkown,
        }
    }
}
