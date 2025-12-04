use itertools::Itertools;

use crate::{containers::DateTime, extractors::Match, misc::token_borders::TokenBorders};

// pub fn extract(haystack: &str) -> impl Iterator<Item = Match<DateTime>> {
//     haystack.char_indices().map(f)
//     TokenBorders::new(haystack)
//         .unique()
//         .map(|start| (start, &haystack[start..]))
//         .filter_map(|(start, slice)| {
//             let (value, rem) = DateTime::parse_and_remainder(slice).ok()?;
//             let end = rem.as_ptr() as usize - haystack.as_ptr() as usize;
//             Some(Match { start, end, value })
//         })
// }
