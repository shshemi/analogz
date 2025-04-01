pub mod datetime;

use itertools::Itertools;

#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub start: usize,
    pub end: usize,
}

pub trait Extract {
    type Value;
    fn extract(&self, slice: &str) -> Option<(Location, Self::Value)>;
}

pub fn extract<Iter, Ext>(iter: Iter, ext: Ext) -> (Vec<Option<Location>>, Vec<Option<Ext::Value>>)
where
    Iter: IntoIterator,
    Iter::Item: AsRef<str>,
    Ext: Extract,
{
    iter.into_iter()
        .map(|slice| ext.extract(slice.as_ref()))
        .fold(
            (Vec::default(), Vec::default()),
            |(mut v1, mut v2), pair| {
                //
                if let Some((e1, e2)) = pair {
                    v1.push(Some(e1));
                    v2.push(Some(e2));
                } else {
                    v1.push(None);
                    v2.push(None);
                }
                (v1, v2)
            },
        )
}

pub fn par_extract<T, Ext>(
    slice: &[T],
    ext: Ext,
) -> (Vec<Option<Location>>, Vec<Option<Ext::Value>>)
where
    T: AsRef<str> + Send + Sync,
    Ext: Extract + Sync + 'static,
    Ext::Value: Send + Clone,
{
    let chunk_size = (slice.len() / num_cpus::get()).max(1);

    let vec = std::thread::scope(|scope| {
        let mut hndls = Vec::new();
        for chunk in slice.chunks(chunk_size) {
            let hndl = scope.spawn(|| {
                chunk
                    .iter()
                    .map(|str_ref| ext.extract(str_ref.as_ref()))
                    .collect::<Vec<_>>()
            });
            hndls.push(hndl);
        }
        hndls.into_iter().map(|h| h.join().unwrap()).collect_vec()
    });

    let mut locs = vec![None; slice.len()];
    let mut vals = vec![None; slice.len()];
    for (idx, vec) in vec.into_iter().enumerate() {
        let offset = idx * chunk_size;
        for (idx, item) in vec.into_iter().enumerate() {
            if let Some((loc, val)) = item {
                locs[offset + idx] = Some(loc);
                vals[offset + idx] = Some(val);
            }
        }
    }

    (locs, vals)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestExtractor;

    impl Extract for TestExtractor {
        type Value = String;

        fn extract(&self, slice: &str) -> Option<(Location, Self::Value)> {
            if slice.contains("extract") {
                let start = slice.find("extract").unwrap();
                let end = start + "extract".len();
                Some((Location { start, end }, "extracted".to_string()))
            } else {
                None
            }
        }
    }

    #[test]
    fn test_extract() {
        let inputs = vec![
            "this contains extract word",
            "this doesn't",
            "another extract example",
            "no match here",
        ];

        let extractor = TestExtractor;
        let (locations, values) = extract(inputs, extractor);

        assert_eq!(locations.len(), 4);
        assert_eq!(values.len(), 4);

        // First string contains "extract"
        assert!(locations[0].is_some());
        assert_eq!(locations[0].unwrap().start, 14);
        assert_eq!(locations[0].unwrap().end, 21);
        assert_eq!(values[0].as_ref().unwrap(), "extracted");

        // Second string doesn't contain "extract"
        assert!(locations[1].is_none());
        assert!(values[1].is_none());

        // Third string contains "extract"
        assert!(locations[2].is_some());
        assert_eq!(locations[2].unwrap().start, 8);
        assert_eq!(locations[2].unwrap().end, 15);
        assert_eq!(values[2].as_ref().unwrap(), "extracted");

        // Fourth string doesn't contain "extract"
        assert!(locations[3].is_none());
        assert!(values[3].is_none());
    }

    #[test]
    fn test_extract_empty() {
        let inputs: Vec<String> = vec![];
        let extractor = TestExtractor;
        let (locations, values) = extract(inputs, extractor);

        assert_eq!(locations.len(), 0);
        assert_eq!(values.len(), 0);
    }

    #[test]
    fn test_par_extract() {
        let inputs = vec![
            "this contains extract word",
            "this doesn't",
            "another extract example",
            "no match here",
            "extract can be in different places",
            "multiple extract words in one extract string",
            "EXTRACT in uppercase doesn't count",
            "extract at the beginning",
            "at the end extract",
            "extract-with-dash",
            "subtract is not matched",
            "this is a long string with the word extract somewhere in the middle",
            "   extract with leading whitespace",
            "extract with trailing whitespace   ",
        ];

        let extractor = TestExtractor;
        let (locations, values) = par_extract(&inputs, extractor);

        assert_eq!(locations.len(), inputs.len());
        assert_eq!(values.len(), inputs.len());

        // First string contains "extract"
        assert!(locations[0].is_some());
        assert_eq!(locations[0].unwrap().start, 14);
        assert_eq!(locations[0].unwrap().end, 21);
        assert_eq!(values[0].as_ref().unwrap(), "extracted");

        // Second string doesn't contain "extract"
        assert!(locations[1].is_none());
        assert!(values[1].is_none());

        // Third string contains "extract"
        assert!(locations[2].is_some());
        assert_eq!(locations[2].unwrap().start, 8);
        assert_eq!(locations[2].unwrap().end, 15);
        assert_eq!(values[2].as_ref().unwrap(), "extracted");

        // Fourth string doesn't contain "extract"
        assert!(locations[3].is_none());
        assert!(values[3].is_none());

        // Check fifth string contains "extract"
        assert!(locations[4].is_some());
        assert_eq!(locations[4].unwrap().start, 0);
        assert_eq!(locations[4].unwrap().end, 7);
        assert_eq!(values[4].as_ref().unwrap(), "extracted");

        // Check sixth string (first occurrence only)
        assert!(locations[5].is_some());
        assert_eq!(locations[5].unwrap().start, 9);
        assert_eq!(locations[5].unwrap().end, 16);
        assert_eq!(values[5].as_ref().unwrap(), "extracted");

        // Check seventh string doesn't match (uppercase)
        assert!(locations[6].is_none());
        assert!(values[6].is_none());

        // Check eighth string
        assert!(locations[7].is_some());
        assert_eq!(locations[7].unwrap().start, 0);
        assert_eq!(locations[7].unwrap().end, 7);
        assert_eq!(values[7].as_ref().unwrap(), "extracted");

        // Check ninth string
        assert!(locations[8].is_some());
        assert_eq!(locations[8].unwrap().start, 11);
        assert_eq!(locations[8].unwrap().end, 18);
        assert_eq!(values[8].as_ref().unwrap(), "extracted");

        // Check tenth string
        assert!(locations[9].is_some());
        assert_eq!(locations[9].unwrap().start, 0);
        assert_eq!(locations[9].unwrap().end, 7);
        assert_eq!(values[9].as_ref().unwrap(), "extracted");

        // Check eleventh string doesn't match
        assert!(locations[10].is_none());
        assert!(values[10].is_none());

        // Check twelfth string
        assert!(locations[11].is_some());
        assert_eq!(values[11].as_ref().unwrap(), "extracted");

        // Check thirteenth string
        assert!(locations[12].is_some());
        assert_eq!(locations[12].unwrap().start, 3);
        assert_eq!(locations[12].unwrap().end, 10);
        assert_eq!(values[12].as_ref().unwrap(), "extracted");

        // Check fourteenth string
        assert!(locations[13].is_some());
        assert_eq!(locations[13].unwrap().start, 0);
        assert_eq!(locations[13].unwrap().end, 7);
        assert_eq!(values[13].as_ref().unwrap(), "extracted");
    }

    #[test]
    fn test_par_extract_empty() {
        let inputs: Vec<String> = vec![];
        let extractor = TestExtractor;
        let (locations, values) = par_extract(&inputs, extractor);

        assert_eq!(locations.len(), 0);
        assert_eq!(values.len(), 0);
    }

    #[test]
    fn test_par_extract_single_thread() {
        // This test ensures par_extract works correctly even when chunking results in 1 item per chunk
        let inputs = vec!["extract"];
        let extractor = TestExtractor;
        let (locations, values) = par_extract(&inputs, extractor);

        assert_eq!(locations.len(), 1);
        assert_eq!(values.len(), 1);
        assert!(locations[0].is_some());
        assert_eq!(values[0].as_ref().unwrap(), "extracted");
    }
}
