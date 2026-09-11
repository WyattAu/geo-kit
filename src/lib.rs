#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))] // tests assert invariants directly
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
//! | [`CaPostcode`] | Canada `^[A-Z][0-9][A-Z] [0-9][A-Z][0-9]$`, space optional, uppercase normalized |
//! | [`DePlz`] | Germany `^\d{5}$` |
//! | [`FrCp`] | France `^\d{5}$` |
//! | [`JpPostal`] | Japan `^\d{3}-\d{4}$`, hyphen optional, normalized |
//! | [`AuPostcode`] | Australia `^\d{4}$` |
//! | [`InPin`] | India `^[1-9]\d{5}$` |
//! | [`Postcode`] | Generic postcode (any supported country) |
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
pub use coords::{is_valid_coords, Coords};
pub use country::{is_valid_country_code, CountryCode};
pub use error::GeoError;
pub use postcode::{
    is_valid_au_postcode, is_valid_ca_postcode, is_valid_de_plz, is_valid_fr_cp, is_valid_in_pin,
    is_valid_jp_postal, is_valid_uk_postcode, is_valid_us_zip, AuPostcode, CaPostcode, DePlz, FrCp,
    InPin, JpPostal, Postcode, UkPostcode, UsZipCode,
};

#[cfg(test)]
mod smoke {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    #[test]
    fn reexports_work() {
        let _ = UkPostcode::parse("SW1A 1AA").expect("valid");
        let _ = UsZipCode::parse("90210").expect("valid");
        let _ = CaPostcode::parse("K1A 0B1").expect("valid");
        let _ = DePlz::parse("10115").expect("valid");
        let _ = FrCp::parse("75001").expect("valid");
        let _ = JpPostal::parse("100-0001").expect("valid");
        let _ = AuPostcode::parse("2000").expect("valid");
        let _ = InPin::parse("110001").expect("valid");
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
