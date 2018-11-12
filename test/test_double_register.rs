//! A smoke test for windows compatibility

#[test]
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub fn test_double_register() {
    use mio::*;
    use mio_uds_windows::UnixListener;
    use tempdir::TempDir;

    let poll = Poll::new().unwrap();
    let dir = TempDir::new("uds").unwrap();

    // Create the listener
    let l = UnixListener::bind(dir.path().join("foo")).unwrap();

    // Register the listener with `Poll`
    poll.register(&l, Token(0), Ready::readable(), PollOpt::edge()).unwrap();
    assert!(poll.register(&l, Token(1), Ready::readable(), PollOpt::edge()).is_err());
}
