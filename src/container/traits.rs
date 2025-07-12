use crate::container::ArcStr;

pub trait Find {
    fn find(&self, corpus: &ArcStr) -> Option<ArcStr>;
}
