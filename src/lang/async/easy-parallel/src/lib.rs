use std::thread;
use std::sync::mpsc;
use std::process;
use std::fmt;
use std::fmt::Formatter;
use std::mem;
use std::panic;

/// A builder that runs closures in parallel
#[derive(Default)]
#[must_use]
pub struct Parallel<'a, T> {
    /// Closures to run
    closures: Vec<Box<dyn FnOnce() -> T + Send + 'a>>
}

impl<'a, T> Parallel<'a, T> {

    /// Creates a builder for running closures in parallel
    pub fn new() -> Parallel<'a, T> {
        Parallel {
            closures: Vec::new()
        }
    }

    /// Adds a closures to the list
    pub fn add<F>(mut self, f: F) -> Parallel<'a, T>
    where
        F: FnOnce() -> T + Send + 'a,
        T: Send + 'a
    {
        self.closures.push(Box::new(f));
        self
    }

    /// Adds a cloned closure for each itme in an iterator
    pub fn each<A, I, F>(mut self, iter: I, f: F) -> Parallel<'a, T>
    where
        I: IntoIterator<Item = A>,
        F: FnOnce(A) -> T + Clone + Send + 'a,
        A: Send + 'a,
        T: Send + 'a,
    {
        for t in iter.into_iter() {
            let f = f.clone();
            self.closures.push(Box::new(||f(t)));
        }
        self
    }

    /// Runs each closure on a separate thread and collects their results.
    ///
    /// Result are collected in the order in which closures were added. One of
    /// the closures always runs on the main thread because there is no point
    /// in spawning an extra thread for it
    ///
    /// If a closure panics, panicking will resume in the main thread after
    /// all threads are joined
    pub fn run(self) -> Vec<T>
    where
        T: Send + 'a
    {
        // Get the first closure
        let mut closures = self.closures.into_iter();
        let f = match closures.next() {
            None => return Vec::new(),
            Some(f) => f,
        };

        // Set up a guard that aborts on panic
        let guard = NoPanic;

        // Join handles for spawned threads
        let mut handles = Vec::new();

        // Channels to collect results from spawned threads
        let mut receivers = Vec::new();

        // Spawn a thread for each closure after the first one
        for f in closures {
            // Wrap into a closure that sends the result back
            let (sender, receiver) = mpsc::channel();
            let f = move || sender.send(f()).unwrap();

            // Erase the `'a` lifetime
            let f: Box<dyn FnOnce() + Send + 'a> = Box::new(f);
            let f: Box<dyn FnOnce() + Send + 'static> = unsafe {
                mem::transmute(f)
            };

            // Spawn a thread for the closure
            handles.push(thread::spawn(f));
            receivers.push(receiver);
        }

        let mut results = Vec::new();
        let mut last_err = None;

        // Run the first closure on the main thread
        match panic::catch_unwind(panic::AssertUnwindSafe(f)) {
            Ok(r) => results.push(r),
            Err(err) => last_err = Some(err),
        }

        // Join threads and save the last panic if there was one
        for h in handles {
            if let Err(err) = h.join() {
                last_err = Some(err);
            }
        }

        // Drop the guard because we may resume a panic now
        drop(guard);

        // If a closure panicked, resume the last panic
        if let Some(err) = last_err {
            panic::resume_unwind(err);
        }

        // Collect the results from threads
        for receiver in receivers {
            results.push(receiver.recv().unwrap());
        }
        results
    }
}



impl<T> fmt::Debug for Parallel<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parallel")
            .field("len", &self.closures.len())
            .finish()
    }
}


/// Aborts the process if dropped while panicking
struct NoPanic;

impl Drop for NoPanic {
    fn drop(&mut self) {
        if thread::panicking() {
            process::abort();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Parallel;

    fn par_sum(v: &[i32]) -> i32 {
        const THRESHOLD: usize = 100;
        if v.len() <= THRESHOLD {
            v.iter().copied().sum()
        } else {
            let half = (v.len() + 1) / 2;
            let sums = Parallel::new().each(v.chunks(half), par_sum).run();
            sums.into_iter().sum()
        }
    }

    #[test]
    fn test_parallel() {
        let mut v = Vec::new();
        for i in 0..10_000 {
            v.push(i);
        }
        let sum = dbg!(par_sum(&v));
        assert_eq!(sum, v.into_iter().sum());
    }
}