use crate::containers::ArcStr;

#[derive(Debug)]
pub struct Chars {
    char_indices: CharIndices,
}

impl Iterator for Chars {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.char_indices.next().map(|(_, c)| c)
    }
}

impl From<ArcStr> for Chars {
    fn from(value: ArcStr) -> Self {
        Self {
            char_indices: value.into(),
        }
    }
}

#[derive(Debug)]
pub struct CharIndices {
    astr: ArcStr,
    offset: usize,
}

impl Iterator for CharIndices {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        self.astr.as_str()[self.offset..].chars().next().map(|c| {
            let o = self.offset;
            let l = c.len_utf8();
            self.offset += l;
            (o, c)
        })
    }
}

impl From<ArcStr> for CharIndices {
    fn from(value: ArcStr) -> Self {
        CharIndices {
            astr: value,
            offset: 0,
        }
    }
}
