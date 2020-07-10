

use core::fmt;
use core::ops::{Deref, DerefMut};
use std::fmt::Formatter;

#[derive(Clone, Copy, Hash, Default, Eq, PartialEq)]
pub struct CachePadded<T>(T);

impl<T> CachePadded<T> {
    pub const fn new(t: T) -> CachePadded<T> {
        CachePadded(t)
    }


    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for CachePadded<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for CachePadded<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for CachePadded<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CachePadded").field(&self.0).finish()
    }
}

impl<T> From<T> for CachePadded<T> {
    fn from(t: T) -> Self {
        CachePadded::new(t)
    }
}