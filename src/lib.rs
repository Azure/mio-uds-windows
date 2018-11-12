//! Mio bindings for Unix domain sockets on Windows

#![doc(html_root_url = "https://docs.rs/mio-uds-windows/0.1.0")]
#![deny(missing_docs, missing_debug_implementations)]
#![cfg_attr(test, deny(warnings))]

extern crate lazycell;
extern crate mio;
extern crate iovec;

#[cfg(windows)]
extern crate miow;

#[cfg(windows)]
extern crate winapi;

#[cfg(windows)]
extern crate ws2_32;

#[cfg(windows)]
extern crate kernel32;

#[macro_use]
extern crate log;

mod listener;
mod poll;
mod stdnet;
mod stream;
mod sys;

pub mod net {
    //! The Windows equivalent of std::os::unix::net
    pub use stdnet::{
        AcceptAddrs, AcceptAddrsBuf, SocketAddr, UnixListener, UnixListenerExt,
        UnixStream, UnixStreamExt
    };
}

pub use listener::UnixListener;
pub use stream::UnixStream;

