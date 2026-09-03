#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! # geo-kit
//!
//! Typed newtypes for validated geo primitives — replaces hand-rolled `is_valid_*` checks
//! with parse-once, use-everywhere strong types.
//!
//! | Type | Validation |
//! |------|-----------|
//! | [`UkPostcode`] | UK postcode `^[A-Z]{1,2}[0-9][A-Z0-9]? [0-9][A-Z]{2}$`, space optional, uppercase normalized |
//! | [`UsZipCode`] | US ZIP `^\d{5}(-\d{4})?$` |
//! | [`Postcode`] | Generic postcode (UK or US) |
//! | [`CountryCode`] | ISO 3166-1 alpha-2 `^[A-Z]{2}$`, with `country_name()` mapping |
//! | [`Coords`] | Latitude `-90..=90`, longitude `-180..=180`, finite |
//! | [`Address`] | Validated address with non-empty lines, postcode, country, optional coords |
//!
//! ## Example
//!
//! ```rust
//! use geo_kit::{UkPostcode, CountryCode, Coords, Address};
//!
//! let pc = UkPostcode::parse("SW1A 1AA").expect("valid");
//! assert_eq!(pc.as_str(), "SW1A 1AA");
//!
//! let cc = CountryCode::parse("GB").expect("valid");
//! assert_eq!(cc.country_name(), "United Kingdom");
//!
//! let coords = Coords::new(51.5074, -0.1278).expect("valid");
//! assert!(coords.lat > 51.0);
//!
//! let addr = Address::new(
//!     "10 Downing St".to_string(),
//!     None,
//!     "London".to_string(),
//!     None,
//!     pc,
//!     cc,
//! ).expect("valid");
//! assert_eq!(addr.city(), "London");
//! ```
//!
//! All postcode/country types implement `TryFrom<String>`, `FromStr`, `Display`, `Deref<Target=str>`,
//! `AsRef<str>`, and optional `serde` transparent (de)serialization.

extern crate alloc;

pub mod address;
pub mod coords;
pub mod country;
pub mod error;
pub mod postcode;

// Re-exports
pub use address::{Address, AddressBuilder, PostcodeChoice};
pub use coords::{Coords, is_valid_coords};
pub use country::{CountryCode, is_valid_country_code};
pub use error::GeoError;
pub use postcode::{Postcode, UkPostcode, UsZipCode, is_valid_uk_postcode, is_valid_us_zip};

#[cfg(test)]
mod smoke {
    use super::*;

    #[test]
    fn reexports_work() {
        let _ = UkPostcode::parse("SW1A 1AA").expect("valid");
        let _ = UsZipCode::parse("90210").expect("valid");
        let _ = Postcode::parse("SW1A 1AA").expect("valid");
        let _ = CountryCode::parse("GB").expect("valid");
        let _ = Coords::new(51.5, -0.12).expect("valid");
        let _ = Address::new(
            "10 Downing St".to_string(),
            None,
            "London".to_string(),
            None,
            UkPostcode::parse("SW1A 1AA").expect("valid"),
            CountryCode::parse("GB").expect("valid"),
        )
        .expect("valid");
    }
}
