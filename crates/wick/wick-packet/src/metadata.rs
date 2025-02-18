use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use wasmrs_frames::ex_err;

pub const DONE_FLAG: u8 = /******/ 0b1000_0000;
pub const OPEN_BRACKET: u8 = /***/ 0b0100_0000;
pub const CLOSE_BRACKET: u8 = /**/ 0b0010_0000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Flags {
  Done,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct WickMetadata {
  pub(crate) flags: u8,
  pub(crate) port: String,
}

impl Default for WickMetadata {
  fn default() -> Self {
    Self {
      flags: 0,
      port: "<component>".to_owned(),
    }
  }
}

impl WickMetadata {
  pub fn new(port: impl AsRef<str>, flags: u8) -> Self {
    Self {
      flags,
      port: port.as_ref().to_owned(),
    }
  }

  #[must_use]
  pub fn flags(&self) -> u8 {
    self.flags
  }

  #[must_use]
  pub fn port(&self) -> &str {
    &self.port
  }

  #[must_use]
  pub fn is_done(&self) -> bool {
    self.flags & DONE_FLAG == DONE_FLAG
  }

  #[must_use]
  pub fn is_open_bracket(&self) -> bool {
    self.flags & OPEN_BRACKET == OPEN_BRACKET
  }

  #[must_use]
  pub fn is_close_bracket(&self) -> bool {
    self.flags & CLOSE_BRACKET == CLOSE_BRACKET
  }

  pub fn decode(mut bytes: Bytes) -> Result<Self, wasmrs_frames::Error> {
    let flags = bytes.get_u8();
    let name_len = bytes.get_u16();
    let name_bytes = bytes
      .get(0..(name_len as _))
      .ok_or_else(|| ex_err("Could not read port name bytes"))?;
    let port_name = String::from_utf8(name_bytes.to_vec()).map_err(|_| ex_err("Could not parse port name"))?;
    Ok(WickMetadata::new(port_name, flags))
  }

  #[must_use]
  pub fn encode(self) -> Bytes {
    let mut bytes = BytesMut::new();
    bytes.put_u8(self.flags);
    bytes.put_u16(self.port.len() as _);
    bytes.put(self.port.as_bytes());
    bytes.freeze()
  }
}

#[cfg(test)]
mod test {

  use anyhow::Result;

  use super::*;

  #[test]
  fn test_metadata_decode() -> Result<()> {
    let md = WickMetadata::new("left", DONE_FLAG | CLOSE_BRACKET);
    println!("md: {:?}", md);
    let bytes = md.encode();
    println!("bytes: {:02x?}", bytes.to_vec());
    let meta = WickMetadata::decode(bytes)?;
    assert_eq!(meta.port, "left");
    assert!(meta.is_done());
    assert!(meta.is_close_bracket());
    Ok(())
  }
}
