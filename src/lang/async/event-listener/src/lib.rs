use std::cell::Cell;
use std::fmt;
use std::future::Future;
use std::mem::{self, ManuallyDrop};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::atomic::{self, AtomicPtr, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll, Waker};
use std::thread::{self, Thread};
use std::time::{Duration, Instant};
use std::convert::Infallible;

const NOTIFIED: usize = 1<<0;
const NOTIFIABLE: usize = 1<<1;


struct Inner {
    flags: AtomicUsize,
    list: Mutex<List>
}

impl Inner {
    fn lock(&self) -> ListGuard<'_> {
        ListGuard {
            inner: self,
            guard: self.list.lock().unwrap()
        }
    }
}

pub struct Event {
    inner: AtomicPtr<Inner>
}

unsafe impl Send for Event {}
unsafe impl Sync for Event {}

impl Event {
    #[inline]
    pub fn new() -> Event {
        Event {
            inner: AtomicPtr::default()
        }
    }

    #[cold]
    pub fn listen(&self) -> EventListener {
        let inner = self.inner();
        let listener = EventListener {
            inner: unsafe {
                Arc::clone(&ManuallyDrop::new(Arc::from_raw(inner)))
            },
            entry: Some(inner.lock().insert())
        };
        full_fence();
        listener
    }

    pub fn notify_one(&self) {
        let inner = self.inner();

        full_fence();
        let flags = inner.flags.load(Ordering::Relaxed);
        if flags & NOTIFIED == 0 && flags & NOTIFIABLE != 0 {
            inner.lock().notify(false)
        }
    }

    pub fn notify_all(&self) {
        let inner = self.inner();
        full_fence();
        if inner.flags.load(Ordering::Relaxed) & NOTIFIABLE != 0 {
            inner.lock().notify(true);
        }
    }

    fn inner(&self) -> &Inner {
        let mut inner = self.inner.load(Ordering::Acquire);
        if inner.is_null() {
            let new = Arc::new(Inner {
                flags: AtomicUsize::new(0),
                list: Mutex::new(List {
                    head: None,
                    tail: None,
                    len: 0,
                    notifiable: 0
                })
            });
            let new = Arc::into_raw(new) as *mut Inner;
            inner = self.inner.compare_and_swap(inner, new, Ordering::AcqRel);

            if inner.is_null() {
                inner = new;
            } else {
                unsafe {
                    drop(Arc::from_raw(new));
                }
            }
        }
        unsafe { &*inner }
    }
}

impl Drop for Event {
    #[inline]
    fn drop(&mut self) {
        let inner: *mut Inner = *self.inner.get_mut();
        if !inner.is_null() {
            unsafe {
                drop(Arc::from_raw(inner));
            }
        }
    }
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Event { .. }")
    }
}

impl Default for Event {
    fn default() -> Event {
        Event::new()
    }
}

pub struct EventListener {
    inner: Arc<Inner>,
    entry: Option<NonNull<Entry>>
}

impl EventListener {
    pub fn wait(self) {
        self.wait_internal(None);
    }

    pub fn wait_timeout(self, timeout: Duration) -> bool {
        self.wait_internal(Some(Instant::now() + timeout))
    }

    pub fn wait_deadline(self, deadline: Instant) -> bool {
        self.wait_internal(Some(deadline))
    }

    fn wait_internal(mut self, deadline: Option<Instant>) -> bool {
        let entry = match self.entry.take() {
            None => unreachable!("cannot wait twice on an `EventListener`"),
            Some(entry) => entry
        };
        {
            let mut list = self.inner.lock();
            let e = unsafe {
                entry.as_ref()
            };

            match e.state.replace(State::Notified) {
                State::Notified => {
                    list.remove(entry);
                    return true;
                }
                _ => e.state.set(State::Waiting(thread::current()))
            }
        }

        loop {
            match deadline {
                None => thread::park(),
                Some(deadline) => {
                    let now = Instant::now();
                    if now >= deadline {
                        return false;
                    }
                    thread::park_timeout(deadline - now);
                }
            }
            let mut list = self.inner.lock();
            let e = unsafe {
                entry.as_ref()
            };
            match e.state.replace(State::Notified) {
                State::Notified => {
                    list.remove(entry);
                    return true;
                }
                state => e.state.set(state)
            }
        }
    }
}

impl fmt::Debug for EventListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("EventListener { .. }")
    }
}

impl Future for EventListener {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut list = self.inner.lock();
        let entry = match self.entry {
            None => unreachable!("cannot poll a completed `EventListener` future"),
            Some(entry) => entry
        };
        let state = unsafe {
            &entry.as_ref().state
        };

        match state.replace(State::Notified) {
            State::Notified => {
                list.remove(entry);
                drop(list);
                self.entry = None;
                return Poll::Ready(());
            }
            State::Created => {
                state.set(State::Polling(cx.waker().clone()));
            }
            State::Polling(w) => {
                state.set(State::Polling(w));
            }
            State::Waiting(_) => {
                unreachable!("cannot poll and wait on `EventListener` at the same time");
            }
        }
        Poll::Pending
    }
}

impl Drop for EventListener {
    fn drop(&mut self) {
        if let Some(entry) = self.entry.take() {
            let mut list = self.inner.lock();
            if list.remove(entry).is_notified() {
                list.notify(false);
            }
        }
    }
}
/// A guard holding the linked list locked
struct ListGuard<'a> {
    inner: &'a Inner,
    guard: MutexGuard<'a, List>
}

impl Drop for ListGuard<'_> {

    #[inline]
    fn drop(&mut self) {
        let list = &mut **self;
        let mut flags = 0;
        if list.len - list.notifiable > 0 {
            flags |= NOTIFIED;
        }
        if list.notifiable > 0 {
            flags |= NOTIFIABLE;
        }
        self.inner.flags.store(flags, Ordering::Release);
    }
}

impl Deref for ListGuard<'_> {
    type Target = List;

    fn deref(&self) -> &List {
        &*self.guard
    }
}

impl DerefMut for ListGuard<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut List {
        &mut *self.guard
    }
}

/// The state of a listener
enum State {
    /// It has just been created
    Created,
    /// It has received a notification
    Notified,
    /// An async task is polling it
    Polling(Waker),
    /// A thread is blocked on it
    Waiting(Thread),
}

impl State {
    /// Returns `true` if this is the `Notified` state
    #[inline]
    fn is_notified(&self) -> bool {
        match self {
            State::Notified => true,
            State::Created | State::Polling(_) | State::Waiting(_) => false
        }
    }
}

struct Entry {
    /// The state of this listener
    state: Cell<State>,
    /// Previous entry in the linked list
    prev: Cell<Option<NonNull<Entry>>>,
    /// Next entry in the linked list
    next: Cell<Option<NonNull<Entry>>>
}

impl Entry {
    /// Returns `true` if this entry has been notified
    #[inline]
    fn is_notified(&self) -> bool {
        // Do a dummy replace operation in order to take out the state
        let state = self.state.replace(State::Notified);
        // Put back the state
        let is_notified = state.is_notified();
        self.state.set(state);
        is_notified
    }
}

struct List {

    head: Option<NonNull<Entry>>,
    tail: Option<NonNull<Entry>>,
    len: usize,
    /// Notifiable entries are those that haven't been notified yet
    notifiable: usize
}

impl List {
    fn insert(&mut self) -> NonNull<Entry> {
        unsafe {
            let entry = NonNull::new_unchecked(Box::into_raw(Box::new(Entry {
                state: Cell::new(State::Created),
                prev: Cell::new(self.tail),
                next: Cell::new(None)
            })));
            match mem::replace(&mut self.tail, Some(entry)) {
                None => self.head = Some(entry),
                Some(t) => t.as_ref().next.set(Some(entry))
            }

            self.len += 1;
            self.notifiable += 1;
            entry
        }
    }

    fn remove(&mut self, entry: NonNull<Entry>) -> State {
        unsafe {
            let prev = entry.as_ref().prev.get();
            let next = entry.as_ref().next.get();

            match prev {
                None => self.head = next,
                Some(p) => p.as_ref().next.set(next)
            }

            match next {
                None => self.tail = prev,
                Some(n) => n.as_ref().prev.set(prev)
            }

            let entry = Box::from_raw(entry.as_ptr());
            let state = entry.state.into_inner();

            if !state.is_notified() {
                self.notifiable -= 1;
            }

            self.len -= 1;
            state
        }
    }

    #[cold]
    fn notify(&mut self, notify_all: bool) {
        if !notify_all {
            let mut entry = self.tail;
            while let Some(e) = entry {
                let e = unsafe {
                    e.as_ref()
                };
                if e.is_notified() || entry == self.head {
                    break;
                }
                entry = e.prev.get();
            }
            while let Some(e) = entry {
                let e = unsafe {
                    e.as_ref()
                };
                self.set_notified(e);
                entry = e.next.get();
            }
        } else {
            if let Some(e) = self.head {
                let e =unsafe {
                    e.as_ref()
                };

                self.set_notified(e);
            }
        }
    }

    fn set_notified(&mut self, e: &Entry) -> bool {
        let state = e.state.replace(State::Notified);
        let was_notified = state.is_notified();

        match state {
            State::Notified => {},
            State::Created => {},
            State::Polling(w) => w.wake(),
            State::Waiting(t) => t.unpark()
        }

        if !was_notified {
            self.notifiable -= 1;
            true
        } else {
            false
        }
    }
}

#[inline]
fn full_fence() {

    if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
        let a = AtomicUsize::new(0);
        a.compare_and_swap(0, 1, Ordering::SeqCst);
    } else {
        atomic::fence(Ordering::SeqCst);
    }
}