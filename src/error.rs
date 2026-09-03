//! Error types for `geo-kit`.

extern crate alloc;

use alloc::string::String;
use thiserror::Error;

/// Validation error covering all newtypes in this crate.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GeoError {
    /// Invalid UK postcode.
    #[error("invalid postcode: {0}")]
    InvalidPostcode(String),

    /// Invalid US ZIP code.
    #[error("invalid US ZIP code: {0}")]
    InvalidUsZip(String),

    /// Invalid country code.
    #[error("invalid country code: {0}")]
    InvalidCountry(String),

    /// Invalid coordinates.
    #[error("invalid coordinates: {0}")]
    InvalidCoords(String),

    /// Invalid address.
    #[error("invalid address: {0}")]
    InvalidAddress(String),
}
