#[test]
fn test_log_buf_iteration() {
    use analogz::container::LogBuf;

    let logs = LogBuf::new("line 1\nline 2\nline 3".to_string());
    assert_eq!(logs.len(), 3);

    // Iterate through all lines
    for line in logs.iter() {
        println!("{}", line.as_str());
    }

    // Access a specific line
    if let Some(line) = logs.get(1) {
        assert_eq!(line.as_str(), "line 2");
    }
}
