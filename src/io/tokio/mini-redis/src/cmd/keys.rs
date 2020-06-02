use crate::cmd::{Parse};
use crate::{Connection, Db, Frame};

use bytes::Bytes;
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct Keys {
    /// list the keys name which match the pattern
    ///
    /// example
    ///
    /// ```txt
    /// keys chin*
    /// ```
    ///
    /// return
    ///   - chinese
    ///   - china
    pattern: String,
}

impl Keys {

    pub(crate) fn new(pattern: impl ToString) -> Keys {
        Keys {
            pattern: pattern.to_string(),
        }
    }


    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Keys> {
        let pattern = parse.next_string()?;
        Ok(Keys { pattern })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = if let Some(value) = db.keys(&self.pattern) {
            Frame::Bulk(value)
        } else {
            Frame::Null
        };

        debug!(?response);

        // Write the response back to the client
        dst.write_frame(&response).await?;

        Ok(())
    }

    pub(crate) fn into_frame(self) -> Frame {
        let mut frame = Frame::array();
        frame.push_bulk(Bytes::from("keys".as_bytes()));
        frame.push_bulk(Bytes::from(self.pattern.into_bytes()));
        frame
    }
}