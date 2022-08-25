//! This crate implements 256-bit integer types.
//!
//! The implementation tries to follow as closely as possible to primitive
//! integer types, and should implement all the common methods and traits as the
//! primitive integer types.

#![deny(missing_docs)]
#![no_std]

#[cfg(test)]
extern crate alloc;

#[macro_use]
mod macros {
    #[macro_use]
    pub mod cmp;
    #[macro_use]
    pub mod fmt;
    #[macro_use]
    pub mod ops;
    #[macro_use]
    pub mod iter;
}

mod error;
mod fmt;
mod int;
pub mod intrinsics;
#[cfg(feature = "original-serde")]
pub mod serde;
mod uint;

/// Convenience re-export of 256-integer types and as- conversion traits.
pub mod prelude {
    pub use crate::{AsI256, AsU256, I256, U256, DecimalU256};
}

pub use crate::{
    int::{AsI256, I256},
    uint::{AsU256, U256, DecimalU256},
};

/// A 256-bit signed integer type.
#[allow(non_camel_case_types)]
pub type i256 = I256;

/// A 256-bit unsigned integer type.
#[allow(non_camel_case_types)]
pub type u256 = U256;
