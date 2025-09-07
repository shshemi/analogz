use crate::{
    containers::{ArcStr, SocketAddr},
    misc::split::SplitExt,
};

#[derive(Debug, thiserror::Error)]
#[error("Ip address not found")]
pub struct SocketAddrNotFonud;

#[derive(Debug, Clone, Default)]
pub struct SocketAddrExtractor {}

impl SocketAddrExtractor {
    pub fn extract(&self, text: ArcStr) -> Option<SocketAddr> {
        text.split(" \"$'(),;<>@[]`{|}=")
            .find_map(|slice| slice.parse::<SocketAddr>().ok())
    }
}

#[cfg(test)]
mod socket_addr_extractor_tests {
    use super::*;

    fn arc(s: &str) -> ArcStr {
        ArcStr::from(s)
    }

    #[test]
    fn extracts_first_ipv4_socketaddr_in_plain_text() {
        let ex = SocketAddrExtractor::default();
        let got = ex.extract(arc("src=1.2.3.4:8080 dst=9.9.9.9:53"));
        assert_eq!(got.as_deref().unwrap().to_string(), "1.2.3.4:8080");
    }

    #[test]
    fn extracts_ipv4_surrounded_by_delimiters() {
        // Delimiters are: space, " $ ' ( ) , ; < > @ [ ] ` { | }
        let ex = SocketAddrExtractor::default();
        let got = ex.extract(arc(r#"<"1.2.3.4:80">"#));
        assert_eq!(got.as_deref().unwrap().to_string(), "1.2.3.4:80");
    }

    #[test]
    fn returns_none_when_no_socketaddr_present() {
        let ex = SocketAddrExtractor::default();
        let got = ex.extract(arc("no ip here 1.2.3.4 without port"));
        assert!(got.is_none());
    }

    #[test]
    fn skips_malformed_candidates_and_picks_next_valid() {
        // Invalids: missing host, invalid ipv4, out-of-range port
        let ex = SocketAddrExtractor::default();
        let text = "x=:80, y=300.1.1.1:10; z=10.0.0.1:65536 ok 10.0.0.1:443";
        let got = ex.extract(arc(text));
        assert_eq!(got.as_deref().unwrap().to_string(), "10.0.0.1:443");
    }

    #[test]
    fn handles_multiple_valid_candidates_returns_first() {
        let ex = SocketAddrExtractor::default();
        let text = "a 10.0.0.2:1234, then 8.8.8.8:53 and 1.1.1.1:80";
        let got = ex.extract(arc(text));
        assert_eq!(got.as_deref().unwrap().to_string(), "10.0.0.2:1234");
    }

    #[test]
    fn does_not_break_on_leading_or_trailing_delimiters() {
        let ex = SocketAddrExtractor::default();
        let text = ",;@ 9.9.9.9:5353 `|}";
        let got = ex.extract(arc(text));
        assert_eq!(got.as_deref().unwrap().to_string(), "9.9.9.9:5353");
    }

    #[test]
    fn bracketed_ipv6_is_not_extracted_due_to_bracket_delimiters() {
        // Current splitter treats '[' and ']' as delimiters, so "[::1]:80" is split apart
        // and cannot be parsed as a single slice. This test documents that limitation.
        let ex = SocketAddrExtractor::default();
        let got = ex.extract(arc("before [::1]:80 after"));
        assert!(got.is_none());
    }

    #[test]
    fn ignores_noise_and_unicode_around_the_address() {
        let ex = SocketAddrExtractor::default();
        let text = "✈️ logs: dst=<8.8.4.4:53>, status=ok";
        let got = ex.extract(arc(text));
        assert_eq!(got.as_deref().unwrap().to_string(), "8.8.4.4:53");
    }
}
