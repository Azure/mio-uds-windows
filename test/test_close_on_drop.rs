use {TryRead};
use bytes::ByteBuf;
use mio::{Events, Poll, PollOpt, Ready, Token};
use mio_uds_windows::{UnixListener, UnixStream};
use tempdir::TempDir;

use self::TestState::{Initial, AfterRead};

const SERVER: Token = Token(0);
const CLIENT: Token = Token(1);

#[derive(Debug, PartialEq)]
enum TestState {
    Initial,
    AfterRead,
}

struct TestHandler {
    srv: UnixListener,
    cli: UnixStream,
    state: TestState,
    shutdown: bool,
}

impl TestHandler {
    fn new(srv: UnixListener, cli: UnixStream) -> TestHandler {
        TestHandler {
            srv: srv,
            cli: cli,
            state: Initial,
            shutdown: false,
        }
    }

    fn handle_read(&mut self, poll: &mut Poll, tok: Token, events: Ready) {
        debug!("readable; tok={:?}; hint={:?}", tok, events);

        match tok {
            SERVER => {
                debug!("server connection ready for accept");
                let _ = self.srv.accept().unwrap();
            }
            CLIENT => {
                debug!("client readable");

                match self.state {
                    Initial => {
                        let mut buf = [0; 4096];
                        debug!("GOT={:?}", self.cli.try_read(&mut buf[..]));
                        self.state = AfterRead;
                    },
                    AfterRead => {}
                }

                let mut buf = ByteBuf::mut_with_capacity(1024);

                match self.cli.try_read_buf(&mut buf) {
                    Ok(Some(0)) => self.shutdown = true,
                    Ok(_) => panic!("the client socket should not be readable"),
                    Err(e) => panic!("Unexpected error {:?}", e)
                }
            }
            _ => panic!("received unknown token {:?}", tok)
        }
        poll.reregister(&self.cli, CLIENT, Ready::readable(), PollOpt::edge()).unwrap();
    }

    fn handle_write(&mut self, poll: &mut Poll, tok: Token, _: Ready) {
        match tok {
            SERVER => panic!("received writable for token 0"),
            CLIENT => {
                debug!("client connected");
                poll.reregister(&self.cli, CLIENT, Ready::readable(), PollOpt::edge()).unwrap();
            }
            _ => panic!("received unknown token {:?}", tok)
        }
    }
}

#[test]
pub fn test_close_on_drop() {
    let _ = ::env_logger::init();
    debug!("Starting TEST_CLOSE_ON_DROP");
    let mut poll = Poll::new().unwrap();
    let dir = TempDir::new("uds").unwrap();

    // == Create & setup server socket
    let srv = UnixListener::bind(dir.path().join("foo")).unwrap();
    let addr = srv.local_addr().unwrap();

    poll.register(&srv, SERVER, Ready::readable(), PollOpt::edge()).unwrap();

    // == Create & setup client socket
    let sock = UnixStream::connect(&addr.as_pathname().unwrap()).unwrap();

    poll.register(&sock, CLIENT, Ready::writable(), PollOpt::edge()).unwrap();

    // == Create storage for events
    let mut events = Events::with_capacity(1024);

    // == Setup test handler
    let mut handler = TestHandler::new(srv, sock);

    // == Run test
    while !handler.shutdown {
        poll.poll(&mut events, None).unwrap();

        for event in &events {
            if event.readiness().is_readable() {
                handler.handle_read(&mut poll, event.token(), event.readiness());
            }

            if event.readiness().is_writable() {
                handler.handle_write(&mut poll, event.token(), event.readiness());
            }
        }
    }
    assert!(handler.state == AfterRead, "actual={:?}", handler.state);
}
