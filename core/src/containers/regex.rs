#[derive(Debug, Clone)]
pub struct Regex {
    inner: regex::Regex,
}

impl Regex {
    pub fn new(re: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            inner: regex::Regex::new(re)?,
        })
    }
    pub fn into_inner(self) -> regex::Regex {
        self.inner
    }
}
impl<T> From<T> for Regex
where
    T: Into<regex::Regex>,
{
    fn from(value: T) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl std::ops::Deref for Regex {
    type Target = regex::Regex;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
