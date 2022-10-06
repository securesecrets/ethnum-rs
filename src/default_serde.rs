//! Serde serialization implementation for 256-bit integer types.
//!
//! This implementation is very JSON-centric in that it serializes the integer
//! types as `QUANTITIES` as specified in the Ethereum RPC. That is, integers
//! are encoded as `"0x"` prefixed strings without extrenuous leading `0`s. For
//! negative signed integers, the string is prefixed with a `"-"` sign.
//!
//! Note that this module contains alternative serialization schemes that can
//! be used with `#[serde(with = "...")]`.
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```text
//! #[derive(Deserialize, Serialize)]
//! struct Example {
//!     a: U256, // "0x2a"
//!     #[serde(with = "ethnum::serde::decimal")]
//!     b: I256, // "-42"
//!     #[serde(with = "ethnum::serde::prefixed")]
//!     c: U256, // "0x2a" or "42"
//!     #[serde(with = "ethnum::serde::permissive")]
//!     d: I256, // "-0x2a" or "-42" or -42
//!     #[serde(with = "ethnum::serde::bytes::be")]
//!     e: U256, // [0x2a, 0x00, ..., 0x00]
//!     #[serde(with = "ethnum::serde::bytes::le")]
//!     f: I256, // [0xd6, 0xff, ..., 0xff]
//!     #[serde(with = "ethnum::serde::compressed_bytes::be")]
//!     g: U256, // [0x2a]
//!     #[serde(with = "ethnum::serde::compressed_bytes::le")]
//!     h: I256, // [0xd6]
//! }
//! ```

use crate::{int::I256, uint::U256};
use core::{
    fmt::{self, Display, Formatter, Write},
    mem::MaybeUninit,
    ptr, slice, str,
};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use core::num::ParseIntError;

#[doc(hidden)]
pub trait Decimal: Sized {
    fn from_str_decimal(src: &str) -> Result<Self, ParseIntError>;
    fn write_decimal(&self, f: &mut impl Write);
}

impl Decimal for I256 {
    fn from_str_decimal(src: &str) -> Result<Self, ParseIntError> {
        Self::from_str_radix(src, 10)
    }
    fn write_decimal(&self, f: &mut impl Write) {
        write!(f, "{self}").expect("unexpected formatting error")
    }
}

impl Decimal for U256 {
    fn from_str_decimal(src: &str) -> Result<Self, ParseIntError> {
        Self::from_str_radix(src, 10)
    }
    fn write_decimal(&self, f: &mut impl Write) {
        write!(f, "{self}").expect("unexpected formatting error")
    }
}

impl Serialize for I256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut f = FormatBuffer::decimal();
        self.write_decimal(&mut f);
        serializer.serialize_str(f.as_str())
    }
}

impl Serialize for U256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut f = FormatBuffer::decimal();
        self.write_decimal(&mut f);
        serializer.serialize_str(f.as_str())
    }
}

impl<'de> Deserialize<'de> for I256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FormatVisitor(Self::from_str_decimal))
    }
}

impl<'de> Deserialize<'de> for U256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FormatVisitor(Self::from_str_decimal))
    }
}

/// Internal visitor struct implementation to facilitate implementing different
/// serialization formats.
struct FormatVisitor<F>(F);

impl<'de, T, E, F> Visitor<'de> for FormatVisitor<F>
where
    E: Display,
    F: FnOnce(&str) -> Result<T, E>,
{
    type Value = T;

    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("a formatted 256-bit integer")
    }

    fn visit_str<E_>(self, v: &str) -> Result<Self::Value, E_>
    where
        E_: de::Error,
    {
        self.0(v).map_err(de::Error::custom)
    }

    fn visit_bytes<E_>(self, v: &[u8]) -> Result<Self::Value, E_>
    where
        E_: de::Error,
    {
        let string = str::from_utf8(v)
            .map_err(|_| de::Error::invalid_value(de::Unexpected::Bytes(v), &self))?;
        self.visit_str(string)
    }
}

/// A stack-allocated buffer that can be used for writing formatted strings.
///
/// This allows us to leverage existing `fmt` implementations on integer types
/// without requiring heap allocations (i.e. writing to a `String` buffer).
struct FormatBuffer<const N: usize> {
    offset: usize,
    buffer: [MaybeUninit<u8>; N],
}

impl<const N: usize> FormatBuffer<N> {
    /// Creates a new formatting buffer.
    fn new() -> Self {
        Self {
            offset: 0,
            buffer: [MaybeUninit::uninit(); N],
        }
    }

    /// Returns a `str` to the currently written data.
    fn as_str(&self) -> &str {
        // SAFETY: We only ever write valid UTF-8 strings to the buffer, so the
        // resulting string will always be valid.
        unsafe {
            let buffer = slice::from_raw_parts(self.buffer[0].as_ptr(), self.offset);
            str::from_utf8_unchecked(buffer)
        }
    }
}

impl FormatBuffer<78> {
    /// Allocates a formatting buffer large enough to hold any possible decimal
    /// encoded 256-bit value.
    fn decimal() -> Self {
        Self::new()
    }
}

impl<const N: usize> Write for FormatBuffer<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.offset.checked_add(s.len()).ok_or(fmt::Error)?;

        // Make sure there is enough space in the buffer.
        if end > N {
            return Err(fmt::Error);
        }

        // SAFETY: We checked that there is enough space in the buffer to fit
        // the string `s` starting from `offset`, and the pointers cannot be
        // overlapping because of Rust ownership semantics (i.e. `s` cannot
        // overlap with `buffer` because we have a mutable reference to `self`
        // and by extension `buffer`).
        unsafe {
            let buffer = self.buffer[0].as_mut_ptr().add(self.offset);
            ptr::copy_nonoverlapping(s.as_ptr(), buffer, s.len());
        }
        self.offset = end;

        Ok(())
    }
}
