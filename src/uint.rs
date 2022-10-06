//! Root module for 256-bit unsigned integer type.

mod api;
mod cmp;
mod convert;
mod fmt;
mod iter;
mod ops;
mod parse;

pub use self::convert::AsU256;
use crate::I256;
use borsh::{BorshDeserialize, BorshSerialize};
use core::num::ParseIntError;
use cosmwasm_std::Uint128;
#[cfg(feature = "cosmwasm")]
use cosmwasm_std::{Decimal256, Uint256};
use serde::{Deserialize, Serialize};

/// A 256-bit unsigned integer type.
#[derive(
    Clone,
    Copy,
    Default,
    Eq,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    BorshDeserialize,
    BorshSerialize,
)]
#[repr(transparent)]
pub struct U256(pub [u128; 2]);

/// Type alias for U256.
pub type DecimalU256 = U256;

#[cfg(feature = "cosmwasm")]
impl From<Uint128> for U256 {
    fn from(u: Uint128) -> Self {
        U256::new(u.u128())
    }
}

#[cfg(feature = "cosmwasm")]
#[allow(clippy::from_over_into)]
impl Into<Uint128> for U256 {
    fn into(self) -> Uint128 {
        Uint128::new(self.as_u128())
    }
}

#[cfg(feature = "cosmwasm")]
#[allow(clippy::from_over_into)]
impl Into<Uint256> for U256 {
    fn into(self) -> Uint256 {
        Uint256::from_be_bytes(self.to_be_bytes())
    }
}

#[cfg(feature = "cosmwasm")]
impl From<Uint256> for U256 {
    fn from(u: Uint256) -> Self {
        U256::from_be_bytes(u.to_be_bytes())
    }
}

#[cfg(feature = "cosmwasm")]
#[allow(clippy::from_over_into)]
impl Into<Decimal256> for U256 {
    fn into(self) -> Decimal256 {
        Decimal256::new(Uint256::from_be_bytes(self.to_be_bytes()))
    }
}

#[cfg(feature = "cosmwasm")]
impl From<Decimal256> for U256 {
    fn from(u: Decimal256) -> Self {
        U256::from_be_bytes(u.atomics().to_be_bytes())
    }
}

impl U256 {
    /// The additive identity for this integer type, i.e. `0`.
    pub const ZERO: Self = U256([0; 2]);

    /// The multiplicative identity for this integer type, i.e. `1`.
    pub const ONE: Self = U256::new(1);

    /// Creates a new 256-bit integer value from a primitive `u128` integer.
    #[inline]
    pub const fn new(value: u128) -> Self {
        U256::from_words(0, value)
    }

    /// Creates a new 256-bit integer value from high and low words.
    #[inline]
    pub const fn from_words(hi: u128, lo: u128) -> Self {
        #[cfg(target_endian = "little")]
        {
            U256([lo, hi])
        }
        #[cfg(target_endian = "big")]
        {
            U256([hi, lo])
        }
    }

    /// Splits a 256-bit integer into high and low words.
    #[inline]
    pub const fn into_words(self) -> (u128, u128) {
        #[cfg(target_endian = "little")]
        {
            let U256([lo, hi]) = self;
            (hi, lo)
        }
        #[cfg(target_endian = "big")]
        {
            let U256([hi, lo]) = self;
            (hi, lo)
        }
    }

    /// Get the low 128-bit word for this unsigned integer.
    #[inline]
    pub fn low(&self) -> &u128 {
        #[cfg(target_endian = "little")]
        {
            &self.0[0]
        }
        #[cfg(target_endian = "big")]
        {
            &self.0[1]
        }
    }

    /// Get the low 128-bit word for this unsigned integer as a mutable
    /// reference.
    #[inline]
    pub fn low_mut(&mut self) -> &mut u128 {
        #[cfg(target_endian = "little")]
        {
            &mut self.0[0]
        }
        #[cfg(target_endian = "big")]
        {
            &mut self.0[1]
        }
    }

    /// Get the high 128-bit word for this unsigned integer.
    #[inline]
    pub fn high(&self) -> &u128 {
        #[cfg(target_endian = "little")]
        {
            &self.0[1]
        }
        #[cfg(target_endian = "big")]
        {
            &self.0[0]
        }
    }

    /// Get the high 128-bit word for this unsigned integer as a mutable
    /// reference.
    #[inline]
    pub fn high_mut(&mut self) -> &mut u128 {
        #[cfg(target_endian = "little")]
        {
            &mut self.0[1]
        }
        #[cfg(target_endian = "big")]
        {
            &mut self.0[0]
        }
    }

    /// Converts a prefixed string slice in base 16 to an integer.
    ///
    /// The string is expected to be an optional `+` sign followed by the `0x`
    /// prefix and finally the digits. Leading and trailing whitespace represent
    /// an error.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use ethnum::U256;
    /// assert_eq!(U256::from_str_hex("0x2A"), Ok(U256::new(42)));
    /// ```
    pub fn from_str_hex(src: &str) -> Result<Self, ParseIntError> {
        crate::parse::from_str_radix(src, 16, Some("0x"))
    }

    /// Converts a prefixed string slice in a base determined by the prefix to
    /// an integer.
    ///
    /// The string is expected to be an optional `+` sign followed by the one of
    /// the supported prefixes and finally the digits. Leading and trailing
    /// whitespace represent an error. The base is dertermined based on the
    /// prefix:
    ///
    /// * `0x`: base `16`
    /// * no prefix: base `10`
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use ethnum::U256;
    /// assert_eq!(U256::from_str_prefixed("42"), Ok(U256::new(42)));
    /// assert_eq!(U256::from_str_prefixed("0xa"), Ok(U256::new(10)));
    /// ```
    pub fn from_str_prefixed(src: &str) -> Result<Self, ParseIntError> {
        crate::parse::from_str_prefixed(src)
    }

    /// Cast to a primitive `i8`.
    pub const fn as_i8(self) -> i8 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `i16`.
    pub const fn as_i16(self) -> i16 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `i32`.
    pub const fn as_i32(self) -> i32 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `i64`.
    pub const fn as_i64(self) -> i64 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `i128`.
    pub const fn as_i128(self) -> i128 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a `I256`.
    pub const fn as_i256(self) -> I256 {
        let Self([a, b]) = self;
        I256([a as _, b as _])
    }

    /// Cast to a primitive `u8`.
    pub const fn as_u8(self) -> u8 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `u16`.
    pub const fn as_u16(self) -> u16 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `u32`.
    pub const fn as_u32(self) -> u32 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `u64`.
    pub const fn as_u64(self) -> u64 {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `u128`.
    pub const fn as_u128(self) -> u128 {
        let (_, lo) = self.into_words();
        lo
    }

    /// Cast to a primitive `isize`.
    pub const fn as_isize(self) -> isize {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `usize`.
    pub const fn as_usize(self) -> usize {
        let (_, lo) = self.into_words();
        lo as _
    }

    /// Cast to a primitive `f32`.
    pub fn as_f32(self) -> f32 {
        match self.into_words() {
            (0, lo) => lo as _,
            _ => f32::INFINITY,
        }
    }

    /// Cast to a primitive `f64`.
    pub fn as_f64(self) -> f64 {
        // NOTE: Binary representation of 2**128. This is used because `powi` is
        // neither `const` nor `no_std`.
        const HI: u64 = 0x47f0000000000000;
        let (hi, lo) = self.into_words();
        (hi as f64) * f64::from_bits(HI) + (lo as f64)
    }
}

#[cfg(test)]
mod tests {
    use crate::uint::U256;

    #[test]
    #[allow(clippy::float_cmp)]
    fn converts_to_f64() {
        assert_eq!(U256::from_words(1, 0).as_f64(), 2.0f64.powi(128))
    }
}
