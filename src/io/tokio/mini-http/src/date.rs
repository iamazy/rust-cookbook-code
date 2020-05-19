use std::cell::RefCell;
use std::fmt::{self, Write, Formatter};
use std::str;

use time::{self, Duration};

pub struct Now(());

pub fn now() -> Now {
    Now(())
}

struct LastRenderedNow {
    bytes: [u8; 128],
    amt: usize,
    next_update: time::Timespec,
}

thread_local! {
    static LAST: RefCell<LastRenderedNow> = RefCell::new(LastRenderedNow {
        bytes: [0; 128],
        amt: 0,
        next_update: time::Timespec::new(0,0)
    });
}

impl fmt::Display for Now {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        LAST.with(|cache| {
            let mut cache = cache.borrow_mut();
            let now = time::get_time();
            if now > cache.next_update {
                cache.update(now);
            }
            f.write_str(cache.buffer())
        })
    }
}

impl LastRenderedNow {
    fn buffer(&self) -> &str {
        str::from_utf8(&self.bytes[..self.amt]).unwrap()
    }

    fn update(&mut self, now: time::Timespec) {
        self.amt = 0;
        write!(LocalBuffer(self), "{}", time::at(now).rfc822()).unwrap();
        self.next_update = now + Duration::seconds(1);
        self.next_update.nsec = 0;
    }
}

struct LocalBuffer<'a>(&'a mut LastRenderedNow);

impl<'a> fmt::Write for LocalBuffer<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let start = self.0.amt;
        let end = start + s.len();
        self.0.bytes[start..end].copy_from_slice(s.as_bytes());
        self.0.amt += s.len();
        Ok(())
    }
}