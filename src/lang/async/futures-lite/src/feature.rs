use std::task::{Context, Poll};
use core::fmt;
use std::fmt::Formatter;
use std::pin::Pin;
use std::future::Future;

/// Future for the [`poll_fn`] function
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct PollFn<F> {
    f: F
}

impl<F> Unpin for PollFn<F> {}

/// Creates a new future wrapping around a function returning ['Poll']
///
/// Polling the returned future delegate to the wrapped function
pub fn poll_fn<T, F>(f: F) -> PollFn<F>
where
    F: FnMut(&mut Context<'_>) -> Poll<T>,
{
    PollFn { f }
}

impl<F> fmt::Debug for PollFn<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PollFn").finish()
    }
}

impl<T, F> Future for PollFn<F>
where
    F: FnMut(&mut Context<'_>) -> Poll<T>,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        (&mut self.f)(cx)
    }
}