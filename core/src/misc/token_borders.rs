pub struct TokenBorders<'a> {
    haystack: &'a str,
    state: State,
}

impl<'a> TokenBorders<'a> {
    pub fn new(str: &'a str) -> Self {
        Self {
            haystack: str,
            state: State::Start,
        }
    }
}

impl<'a> Iterator for TokenBorders<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            State::Start => {
                self.state = State::Find(0);
                Some(0)
            }
            State::Find(offset) => {
                if let Some(idx) = self.haystack[offset..].find(pat) {
                    let idx = offset + idx;
                    self.state = State::Found(idx + 1);
                    Some(idx)
                } else {
                    self.state = State::End;
                    Some(self.haystack.len())
                }
            }
            State::Found(offset) => {
                self.state = State::Find(offset);
                Some(offset)
            }
            State::End => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum State {
    Start,
    Find(usize),
    Found(usize),
    End,
}

pub fn pat(c: char) -> bool {
    c.is_ascii_whitespace() || c.is_ascii_punctuation()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn borders(s: &str) -> Vec<usize> {
        TokenBorders::new(s).collect()
    }

    #[test]
    fn empty_string() {
        assert_eq!(borders(""), vec![0, 0]);
    }

    #[test]
    fn no_separators() {
        let s = "abc";
        assert_eq!(borders(s), vec![0, s.len()]);
    }

    #[test]
    fn only_spaces() {
        assert_eq!(borders("   "), vec![0, 0, 1, 1, 2, 2, 3, 3]);
    }

    #[test]
    fn only_punctuation() {
        assert_eq!(borders(".,!"), vec![0, 0, 1, 1, 2, 2, 3, 3]);
    }

    #[test]
    fn leading_punctuation() {
        assert_eq!(borders(",a"), vec![0, 1, 2]);
    }

    #[test]
    fn separator_between_words_space() {
        assert_eq!(borders("a b"), vec![0, 1, 2, 3]);
    }

    #[test]
    fn separator_between_words_comma() {
        assert_eq!(borders("a,b"), vec![0, 1, 2, 3]);
    }

    #[test]
    fn consecutive_separators() {
        assert_eq!(borders("a  b"), vec![0, 1, 2, 2, 3, 4]);
    }

    #[test]
    fn mixed_whitespace_and_punctuation() {
        let s = " Hello, world! ";
        assert_eq!(borders(s), vec![0, 0, 1, 6, 7, 7, 8, 13, 14, 14, 15, 15]);
    }

    #[test]
    fn simple_hello_world() {
        let s = "Hello world";
        assert_eq!(borders(s), vec![0, 5, 6, 11]);
    }
}
