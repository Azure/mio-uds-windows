extern crate mio;

use mio::{Events, Poll, Token, Ready, PollOpt};
use mio_uds_windows::UnixListener;
use std::time::Duration;
use tempdir::TempDir;

#[test]
fn run_once_with_nothing() {
    let mut events = Events::with_capacity(1024);
    let poll = Poll::new().unwrap();
    poll.poll(&mut events, Some(Duration::from_millis(100))).unwrap();
}

#[test]
fn add_then_drop() {
    let mut events = Events::with_capacity(1024);
    let dir = TempDir::new("uds").unwrap();
    let l = UnixListener::bind(dir.path().join("foo")).unwrap();
    let poll = Poll::new().unwrap();
    poll.register(&l, Token(1), Ready::readable() | Ready::writable(), PollOpt::edge()).unwrap();
    drop(l);
    poll.poll(&mut events, Some(Duration::from_millis(100))).unwrap();

}
