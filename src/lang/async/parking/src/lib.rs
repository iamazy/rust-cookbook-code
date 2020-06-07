use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use core::panicking::panic_fmt;


pub struct Parker {
    unparker: Unparker,
    _marker: PhantomData<*const ()>
}

unsafe impl Send for Parker {}

impl Parker {
    pub fn new() -> Parker {
        Parker {
            unparker: Unparker {
                inner: Arc::new(Inner {
                    state: AtomicUsize::new(EMPTY),
                    lock: Mutex::new(()),
                    cvar: Condvar::new()
                }),
            },
            _marker: PhantomData
        }
    }

    pub fn park(&self) {
        self.unparker.inner.park(None);
    }

    pub fn park_timeout(&self, timeout: Duration) -> bool {
        self.unparker.inner.park(Some(timeout))
    }

    pub fn park_deadline(&self, deadline: Instant) -> bool {
        self.unparker.inner.park(
            Some(deadline.saturating_duration_since(Instant::now()))
        )
    }

    pub fn unpark(&self) {
        self.unparker.unpark()
    }

    pub fn unparker(&self) -> Unparker {
        self.unparker.clone()
    }
}

impl fmt::Debug for Parker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Parker { .. }")
    }
}

pub struct Unparker {
    inner: Arc<Inner>,
}

unsafe impl Send for Unparker {}
unsafe impl Sync for Unparker {}

impl Unparker {
    pub fn unpark(&self) {
        self.inner.unpark()
    }
}

impl fmt::Debug for Unparker {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) ->fmt::Result {
        f.pad("Unparker { .. }")
    }
}

impl Clone for Unparker {

    fn clone(&self) -> Unparker {
        Unparker {
            inner: self.inner.clone()
        }
    }
}

const EMPTY: usize = 0;
const PARKED: usize = 1;
const NOTIFIED: usize = 2;

struct Inner {
    state: AtomicUsize,
    lock: Mutex<()>,
    cvar: Condvar
}

impl Inner {
    fn park(&self, timeout: Option<Duration>) -> bool {
        if self.state.compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst)
            .is_ok() {
            return true;
        }
        if let Some(dur) = timeout {
            if dur == Duration::from_millis(0) {
                return false;
            }
        }

        let mut m = self.lock.lock().unwrap();

        match self.state.compare_exchange(EMPTY, PARKED, SeqCst, SeqCst) {
            Ok(_) => {},
            Err(NOTIFIED) => {
                let old = self.state.swap(EMPTY, SeqCst);
                assert_eq!(old, NOTIFIED, "park state change unexpectedly");
                return true;
            }
            Err(n) => panic!("inconsistent park_timeout state: {}", n);
        }

        match timeout {
            None => {
                loop {
                    m = self.cvar.wait(m).unwrap();
                    match self.state.compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst) {
                        Ok(_) => return true,
                        Err(_) => {}
                    }
                }
            }
            Some(timeout) => {
                let (_m, _result) = self.cvar.wait_timeout(m, timeout).unwrap();

                match self.state.swap(EMPTY, SeqCst) {
                    NOTIFIED => true,
                    PARKED => false,
                    n => panic!("inconsistent park_timeout state: {}",n)
                }
            }
        }
    }


    pub fn unpark(&self) {
        match self.state.swap(NOTIFIED, SeqCst) {
            EMPTY => return,
            NOTIFIED => return,
            PARKED => {},
            _ => panic!("inconsistent state in unpark")
        }
        drop(self.lock.lock().unwrap());
        self.cvar.notify_one();
    }
}