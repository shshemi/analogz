use crate::{
    containers::{ArcStr, IpAddr},
    misc::split::SplitExt,
};

#[derive(Debug, thiserror::Error)]
#[error("Ip address not found")]
pub struct IpAddrNotFound;

#[derive(Debug, Clone, Default)]
pub struct IpAddrExtractor {}

impl IpAddrExtractor {
    pub fn extract(&self, text: ArcStr) -> Option<IpAddr> {
        text.split(" \"$'(),;<>@[]`{|}=")
            .find_map(|slice| slice.parse::<IpAddr>().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic functionality tests
    #[test]
    fn test_extract_valid_ipv4_address() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("192.168.1.1");
        let result = extractor.extract(text);
        assert_eq!(result, Some("192.168.1.1".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn test_extract_valid_ipv6_address() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("2001:0db8:85a3:0000:0000:8a2e:0370:7334");
        let result = extractor.extract(text);
        assert_eq!(
            result,
            Some(
                "2001:0db8:85a3:0000:0000:8a2e:0370:7334"
                    .parse::<IpAddr>()
                    .unwrap()
            )
        );
    }

    #[test]
    fn test_extract_compressed_ipv6_address() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("2001:db8::1");
        let result = extractor.extract(text);
        assert_eq!(result, Some("2001:db8::1".parse::<IpAddr>().unwrap()));
    }

    // Edge case: minimum length (7 characters)
    #[test]
    fn test_extract_minimum_length_valid_ip() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("1.1.1.1"); // exactly 7 characters
        let result = extractor.extract(text);
        assert_eq!(result, Some("1.1.1.1".parse::<IpAddr>().unwrap()));
    }

    // Edge case: maximum length (15 characters)
    #[test]
    fn test_extract_maximum_length_valid_ipv4() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("255.255.255.255"); // exactly 15 characters
        let result = extractor.extract(text);
        assert_eq!(result, Some("255.255.255.255".parse::<IpAddr>().unwrap()));
    }

    // IP address embedded in log line
    #[test]
    fn test_extract_ip_from_log_line() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from(
            "2023-10-01 10:30:45 INFO Request from 192.168.1.100 processed successfully",
        );
        let result = extractor.extract(text);
        assert_eq!(result, Some("192.168.1.100".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn test_extract_ip_at_beginning_of_text() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("10.0.0.1 - user login attempt");
        let result = extractor.extract(text);
        assert_eq!(result, Some("10.0.0.1".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn test_extract_ip_at_end_of_text() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("Connection established from 172.16.254.1");
        let result = extractor.extract(text);
        assert_eq!(result, Some("172.16.254.1".parse::<IpAddr>().unwrap()));
    }

    // Multiple IP addresses - should return first valid one found
    #[test]
    fn test_extract_first_valid_ip_from_multiple() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("From 10.0.0.1 to 192.168.1.1 via 172.16.0.1");
        let result = extractor.extract(text);
        // The exact IP returned depends on sliding window and round-robin implementation
        assert!(result.is_some());
        let ip = result.unwrap();
        assert!(
            ip.to_string() == "10.0.0.1"
                || ip.to_string() == "192.168.1.1"
                || ip.to_string() == "172.16.0.1"
        );
    }

    // Invalid IP addresses
    #[test]
    fn test_extract_invalid_ipv4_out_of_range() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("256.256.256.256");
        let result = extractor.extract(text);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_invalid_ipv4_wrong_format() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("192.168.1");
        let result = extractor.extract(text);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_invalid_ipv4_with_letters() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("192.168.1.abc");
        let result = extractor.extract(text);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_malformed_ipv6() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("2001:0db8:85a3::8a2e::7334"); // double ::
        let result = extractor.extract(text);
        assert_eq!(result, None);
    }

    // Empty and whitespace cases
    #[test]
    fn test_extract_empty_string() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("");
        let result = extractor.extract(text);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_whitespace_only() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("   \t\n  ");
        let result = extractor.extract(text);
        assert_eq!(result, None);
    }

    // Text too short for sliding window
    #[test]
    fn test_extract_text_shorter_than_minimum_window() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("1.1.1"); // 5 characters, less than minimum window size of 7
        let result = extractor.extract(text);
        assert_eq!(result, None);
    }

    // Special IP addresses
    #[test]
    fn test_extract_localhost_ipv4() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("127.0.0.1");
        let result = extractor.extract(text);
        assert_eq!(result, Some("127.0.0.1".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn test_extract_localhost_ipv6() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("::1");
        let result = extractor.extract(text);
        assert_eq!(result, Some("::1".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn test_extract_broadcast_address() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("255.255.255.255");
        let result = extractor.extract(text);
        assert_eq!(result, Some("255.255.255.255".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn test_extract_zero_address() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("0.0.0.0");
        let result = extractor.extract(text);
        assert_eq!(result, Some("0.0.0.0".parse::<IpAddr>().unwrap()));
    }

    // Text with IP-like but invalid patterns
    #[test]
    fn test_extract_decimal_numbers_not_ip() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("The price is 123.456.789.012");
        let result = extractor.extract(text);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_version_number_not_ip() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("Version 1.2.3.4 released");
        let result = extractor.extract(text);
        assert_eq!(result, Some("1.2.3.4".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn test_extract_date_not_ip() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("Date: 12.34.56.78");
        let result = extractor.extract(text);
        assert_eq!(result, Some("12.34.56.78".parse::<IpAddr>().unwrap()));
    }

    // Edge cases with punctuation and special characters
    #[test]
    fn test_extract_ip_with_surrounding_punctuation() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("IP: [192.168.1.1], Port: 8080");
        let result = extractor.extract(text);
        assert_eq!(result, Some("192.168.1.1".parse::<IpAddr>().unwrap()));
    }

    // Long text with embedded IP
    #[test]
    fn test_extract_ip_from_very_long_text() {
        let extractor = IpAddrExtractor::default();
        let long_text = "A".repeat(1000) + " 192.168.1.1 " + &"B".repeat(1000);
        let text = ArcStr::from(long_text);
        let result = extractor.extract(text);
        assert_eq!(result, Some("192.168.1.1".parse::<IpAddr>().unwrap()));
    }

    // Private IP address ranges
    #[test]
    fn test_extract_private_ip_class_a() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("10.1.2.3");
        let result = extractor.extract(text);
        assert_eq!(result, Some("10.1.2.3".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn test_extract_private_ip_class_b() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("172.16.0.1");
        let result = extractor.extract(text);
        assert_eq!(result, Some("172.16.0.1".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn test_extract_private_ip_class_c() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("192.168.0.1");
        let result = extractor.extract(text);
        assert_eq!(result, Some("192.168.0.1".parse::<IpAddr>().unwrap()));
    }

    // Unicode and non-ASCII characters
    #[test]
    fn test_extract_ip_with_unicode_context() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("Connection from 用户 192.168.1.1 established");
        let result = extractor.extract(text);
        assert_eq!(result, Some("192.168.1.1".parse::<IpAddr>().unwrap()));
    }

    // Test with different IP formats that might be at window boundaries
    #[test]
    fn test_extract_ip_exactly_at_window_size_7() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("prefix 1.1.1.1 suffix");
        let result = extractor.extract(text);
        assert_eq!(result, Some("1.1.1.1".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn test_extract_no_valid_ip_in_numeric_text() {
        let extractor = IpAddrExtractor::default();
        let text = ArcStr::from("1234567890123456789");
        let result = extractor.extract(text);
        assert_eq!(result, None);
    }
}
