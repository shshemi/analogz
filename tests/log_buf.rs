#[test]
fn test_log_buf_iteration() {
    use analogz::container::Buffer;

    let hdfs_logs = std::fs::read_to_string("HDFS.log").unwrap();

    let _ = Buffer::new(hdfs_logs);
}
