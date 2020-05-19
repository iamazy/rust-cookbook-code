use std::{fmt, io, slice, str};
use bytes::BytesMut;
use httparse;
use std::fmt::Formatter;

pub struct Request {
    method: Slice,
    path: Slice,
    version: u8,
    headers: Vec<(Slice, Slice)>,
    data: BytesMut
}

type Slice = (usize, usize);

pub struct RequestHeaders<'req> {
    headers: slice::Iter<'req, (Slice, Slice)>,
    req: &'req Request
}

impl Request {
    pub fn method(&self) -> &str{
        str::from_utf8(self.slice(&self.method)).unwrap()
    }

    pub fn path(&self) -> &str {
        str::from_utf8(self.slice(&self.path)).unwrap()
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn headers(&self) -> RequestHeaders {
        RequestHeaders {
            headers: self.headers.iter(),
            req: self,
        }
    }

    fn slice(&self, slice: &Slice) -> &[u8] {
        &self.data[slice.0..slice.1]
    }

    pub fn decode(buf: &mut BytesMut) -> io::Result<Option<Request>> {

        let (method, path, version, headers, amt) = {
            let mut headers = [httparse::EMPTY_HEADER; 16];
            let mut r = httparse::Request::new(&mut headers);
            let status = r.parse(buf).map_err(|e| {
                let msg = format!("failed to parse http request: {:?}", e);
                io::Error::new(io::ErrorKind::Other, msg)
            })?;

            let amt = match status {
                httparse::Status::Complete(amt) => amt,
                httparse::Status::Partial => return Ok(None)
            };

            let toSlice = |a: &[u8]| {
                let start = a.as_ptr() as usize - buf.as_ptr() as usize;
                assert!(start < buf.len());
                (start, start + a.len())
            };

            (
                toSlice(r.method.unwrap().as_bytes()),
                toSlice(r.path.unwrap().as_bytes()),
                r.version.unwrap(),
                r.headers.iter()
                    .map(|h| (toSlice(h.name.as_bytes()), toSlice(h.value)))
                    .collect(),
                amt,
            )
        };

        Ok(Request {
            method,
            path,
            version,
            headers,
            data: buf.split_to(amt)
        }.into())
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<HTTP Request {} {}>", self.method(), self.path())
    }
}

impl<'req> Iterator for RequestHeaders<'req> {
    type Item = (&'req str, &'req [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        self.headers.next().map(|&(ref a,ref b)| {
            let a = self.req.slice(a);
            let b = self.req.slice(b);
            (str::from_utf8(a).unwrap(), b)
        })
    }
}