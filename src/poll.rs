use mio::{Poll, PollOpt, Ready, Registration, SetReadiness, Token};
use std::io;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/// Used to associate an IO type with a Selector
#[derive(Debug)]
pub struct SelectorId {
    id: AtomicUsize,
}

impl SelectorId {
    pub fn new() -> SelectorId {
        SelectorId {
            id: AtomicUsize::new(0),
        }
    }

    pub fn associate_selector(&self, poll: &Poll) -> io::Result<()> {
        let selector_id = self.id.load(Ordering::SeqCst);
        let poll_id = skinny::selector_id(poll);

        if selector_id != 0 && selector_id != poll_id {
            Err(io::Error::new(io::ErrorKind::Other, "socket already registered"))
        } else {
            self.id.store(poll_id, Ordering::SeqCst);
            Ok(())
        }
    }
}

impl Clone for SelectorId {
    fn clone(&self) -> SelectorId {
        SelectorId {
            id: AtomicUsize::new(self.id.load(Ordering::SeqCst)),
        }
    }
}

// TODO: get rid of this, windows depends on it for now
pub fn new_registration(poll: &Poll, token: Token, ready: Ready, opt: PollOpt)
        -> (Registration, SetReadiness)
{
    #[allow(deprecated)]
    Registration::new(poll, token, ready, opt)
}

// The skinny module duplicates the minimal set of internal mio types that allow
// I/O objects in this crate (UnixListener, UnixStream) to fully integrate with
// the Binding and Overlapped types from mio::windows.
//
// The accessor functions in this module transmute the mio::windows types into
// "skinny" types with an identical memory layout to gain access to internals.
//
// For example, when UnixStream::write_bufs needs to use the same scatter/gather
// buffer logic that TcpStream uses (but which is hidden inside Binding), it
// uses skinny::get/put_buffer(binding). When UnixStream and UnixListener need
// to verify that a Poll object isn't already registered to a different socket,
// they use skinny::selector_id(poll).
//
// The transmute is obviously dangerous, but the alternative is to build up
// a lot of machinery that already exists inside Binding and Overlapped (esp.
// Binding).
pub mod skinny {
    use std::mem;
    use std::sync::{atomic::AtomicUsize, Arc, Condvar, Mutex};
    use mio;

    pub mod sys {
        use std::mem;
        use std::sync::{Arc, Mutex};
        use lazycell::AtomicLazyCell;
        use mio::windows::Binding as MiowBinding;
        use miow::iocp::CompletionPort;

        struct Binding {
            selector: AtomicLazyCell<Arc<SelectorInner>>,
        }

        struct BufferPool {
            pool: Vec<Vec<u8>>,
        }

        impl BufferPool {
            #[allow(dead_code)]
            pub fn new(cap: usize) -> BufferPool {
                BufferPool { pool: Vec::with_capacity(cap) }
            }

            pub fn get(&mut self, default_cap: usize) -> Vec<u8> {
                self.pool.pop().unwrap_or_else(|| Vec::with_capacity(default_cap))
            }

            pub fn put(&mut self, mut buf: Vec<u8>) {
                if self.pool.len() < self.pool.capacity(){
                    unsafe { buf.set_len(0); }
                    self.pool.push(buf);
                }
            }
        }

        pub struct Selector {
            inner: Arc<SelectorInner>,
        }

        impl Selector {
            /// Return the `Selector`'s identifier
            pub fn id(&self) -> usize {
                self.inner.id
            }
        }

        struct SelectorInner {
            /// Unique identifier of the `Selector`
            id: usize,

            /// The actual completion port that's used to manage all I/O
            #[allow(dead_code)]
            port: CompletionPort,

            /// A pool of buffers usable by this selector.
            ///
            /// Primitives will take buffers from this pool to perform I/O operations,
            /// and once complete they'll be put back in.
            buffers: Mutex<BufferPool>,
        }

        fn as_binding(binding: &MiowBinding) -> &Binding {
            unsafe { mem::transmute(&binding as *const _ as * const _) }
        }

        pub fn get_buffer(binding: &MiowBinding, size: usize) -> Vec<u8> {
            match as_binding(binding).selector.borrow() {
                Some(i) => i.buffers.lock().unwrap().get(size),
                None => Vec::with_capacity(size),
            }
        }

        pub fn put_buffer(binding: &MiowBinding, buf: Vec<u8>) {
            if let Some(i) = as_binding(binding).selector.borrow() {
                i.buffers.lock().unwrap().put(buf);
            }
        }
    }

    #[allow(dead_code)]
    struct Poll {
        // Platform specific IO selector
        selector: sys::Selector,

        // Custom readiness queue
        readiness_queue: ReadinessQueue,

        // Use an atomic to first check if a full lock will be required. This is a
        // fast-path check for single threaded cases avoiding the extra syscall
        lock_state: AtomicUsize,

        // Sequences concurrent calls to `Poll::poll`
        lock: Mutex<()>,

        // Wakeup the next waiter
        condvar: Condvar,
    }

    struct ReadinessQueue {
        #[allow(dead_code)]
        inner: Arc<ReadinessQueueInner>,
    }

    struct ReadinessQueueInner {}

    fn as_poll(poll: &mio::Poll) -> &Poll {
        unsafe { mem::transmute(&poll as *const _ as *const _) }
    }

    // accessors

    pub fn selector_id(poll: &mio::Poll) -> usize {
        as_poll(poll).selector.id()
    }

    pub fn get_buffer(binding: &mio::windows::Binding, size: usize) -> Vec<u8> {
        sys::get_buffer(binding, size)
    }

    pub fn put_buffer(binding: &mio::windows::Binding, buf: Vec<u8>) {
        sys::put_buffer(binding, buf)
    }
}
