//! Address newtype with validated components.

extern crate alloc;

use alloc::string::{String, ToString};

use crate::coords::Coords;
use crate::country::CountryCode;
use crate::error::GeoError;
use crate::postcode::{UkPostcode, UsZipCode};

/// Which postcode type is used in the address.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum PostcodeChoice {
    /// UK postcode.
    Uk(UkPostcode),
    /// US ZIP code.
    Us(UsZipCode),
}

impl core::fmt::Display for PostcodeChoice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PostcodeChoice::Uk(x) => f.write_str(x.as_str()),
            PostcodeChoice::Us(x) => f.write_str(x.as_str()),
        }
    }
}

/// A validated postal address.
///
/// Validation:
/// - `line1` must be non-empty after trimming
/// - `city` must be non-empty after trimming
/// - `postcode` must be valid (already validated by its type)
/// - `country` must be valid
/// - `line2` and `county` if `Some` must be non-empty after trimming
/// - optional `coords` if `Some` must be valid
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Address {
    /// First address line (required, non-empty).
    pub line1: String,
    /// Optional second address line.
    pub line2: Option<String>,
    /// City / town (required, non-empty).
    pub city: String,
    /// Optional county / state / province.
    pub county: Option<String>,
    /// Postcode / ZIP.
    pub postcode: UkPostcode,
    /// ISO 3166-1 alpha-2 country code.
    pub country: CountryCode,
    /// Optional coordinates.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub coords: Option<Coords>,
}

impl Address {
    /// Create a validated address.
    ///
    /// Validates that required string fields are non-empty after trimming and that optional string
    /// fields, if present, are non-empty after trimming.
    ///
    /// # Errors
    ///
    /// Returns [`GeoError::InvalidAddress`] if validation fails.
    pub fn new(
        line1: String,
        line2: Option<String>,
        city: String,
        county: Option<String>,
        postcode: UkPostcode,
        country: CountryCode,
    ) -> Result<Self, GeoError> {
        Self::new_with_coords(line1, line2, city, county, postcode, country, None)
    }

    /// Create a validated address with optional coordinates.
    ///
    /// # Errors
    ///
    /// Returns [`GeoError::InvalidAddress`] if validation fails or [`GeoError::InvalidCoords`] via
    /// address error conversion if coords are invalid (coords are already validated on construction).
    pub fn new_with_coords(
        line1: String,
        line2: Option<String>,
        city: String,
        county: Option<String>,
        postcode: UkPostcode,
        country: CountryCode,
        coords: Option<Coords>,
    ) -> Result<Self, GeoError> {
        validate_address_fields(&line1, &line2, &city, &county)?;
        Ok(Address {
            line1: line1.trim().to_string(),
            line2: line2.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            city: city.trim().to_string(),
            county: county.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            postcode,
            country,
            coords,
        })
    }

    /// Return line1.
    #[must_use]
    pub fn line1(&self) -> &str {
        &self.line1
    }

    /// Return city.
    #[must_use]
    pub fn city(&self) -> &str {
        &self.city
    }

    /// Return postcode.
    #[must_use]
    pub fn postcode(&self) -> &UkPostcode {
        &self.postcode
    }

    /// Return country.
    #[must_use]
    pub fn country(&self) -> &CountryCode {
        &self.country
    }
}

fn validate_address_fields(
    line1: &str,
    line2: &Option<String>,
    city: &str,
    county: &Option<String>,
) -> Result<(), GeoError> {
    if line1.trim().is_empty() {
        return Err(GeoError::InvalidAddress(
            "line1 must be non-empty".to_string(),
        ));
    }
    if city.trim().is_empty() {
        return Err(GeoError::InvalidAddress(
            "city must be non-empty".to_string(),
        ));
    }
    if line1.contains('\r') || line1.contains('\n') {
        return Err(GeoError::InvalidAddress(
            "line1 must not contain CR or LF".to_string(),
        ));
    }
    if city.contains('\r') || city.contains('\n') {
        return Err(GeoError::InvalidAddress(
            "city must not contain CR or LF".to_string(),
        ));
    }
    if let Some(l2) = line2 {
        if l2.trim().is_empty() {
            return Err(GeoError::InvalidAddress(
                "line2 must be non-empty if provided".to_string(),
            ));
        }
        if l2.contains('\r') || l2.contains('\n') {
            return Err(GeoError::InvalidAddress(
                "line2 must not contain CR or LF".to_string(),
            ));
        }
    }
    if let Some(c) = county {
        if c.trim().is_empty() {
            return Err(GeoError::InvalidAddress(
                "county must be non-empty if provided".to_string(),
            ));
        }
        if c.contains('\r') || c.contains('\n') {
            return Err(GeoError::InvalidAddress(
                "county must not contain CR or LF".to_string(),
            ));
        }
    }
    Ok(())
}

/// Address builder for ergonomic construction.
///
/// Validates on [`AddressBuilder::build`].
#[derive(Debug, Clone, Default)]
pub struct AddressBuilder {
    line1: Option<String>,
    line2: Option<String>,
    city: Option<String>,
    county: Option<String>,
    postcode: Option<UkPostcode>,
    country: Option<CountryCode>,
    coords: Option<Coords>,
}

impl AddressBuilder {
    /// Create a new empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set line1.
    #[must_use]
    pub fn line1(mut self, s: impl Into<String>) -> Self {
        self.line1 = Some(s.into());
        self
    }

    /// Set line2.
    #[must_use]
    pub fn line2(mut self, s: impl Into<String>) -> Self {
        self.line2 = Some(s.into());
        self
    }

    /// Set city.
    #[must_use]
    pub fn city(mut self, s: impl Into<String>) -> Self {
        self.city = Some(s.into());
        self
    }

    /// Set county.
    #[must_use]
    pub fn county(mut self, s: impl Into<String>) -> Self {
        self.county = Some(s.into());
        self
    }

    /// Set postcode.
    #[must_use]
    pub fn postcode(mut self, p: UkPostcode) -> Self {
        self.postcode = Some(p);
        self
    }

    /// Set country.
    #[must_use]
    pub fn country(mut self, c: CountryCode) -> Self {
        self.country = Some(c);
        self
    }

    /// Set coordinates.
    #[must_use]
    pub fn coords(mut self, c: Coords) -> Self {
        self.coords = Some(c);
        self
    }

    /// Build the address, validating required fields.
    ///
    /// # Errors
    ///
    /// Returns [`GeoError::InvalidAddress`] if required fields are missing or invalid.
    pub fn build(self) -> Result<Address, GeoError> {
        let line1 = self.line1.ok_or_else(|| {
            GeoError::InvalidAddress("line1 is required".to_string())
        })?;
        let city = self.city.ok_or_else(|| {
            GeoError::InvalidAddress("city is required".to_string())
        })?;
        let postcode = self.postcode.ok_or_else(|| {
            GeoError::InvalidAddress("postcode is required".to_string())
        })?;
        let country = self.country.ok_or_else(|| {
            GeoError::InvalidAddress("country is required".to_string())
        })?;
        Address::new_with_coords(line1, self.line2, city, self.county, postcode, country, self.coords)
    }
}

impl core::fmt::Display for Address {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}, {}", self.line1, self.city)?;
        if let Some(l2) = &self.line2 {
            write!(f, " ({})", l2)?;
        }
        write!(f, ", {}", self.postcode)?;
        write!(f, ", {}", self.country)?;
        if let Some(coords) = &self.coords {
            write!(f, " @ {}", coords)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::country::CountryCode;
    use crate::postcode::UkPostcode;

    fn uk(s: &str) -> UkPostcode {
        UkPostcode::parse(s).expect("valid uk postcode")
    }
    fn cc(s: &str) -> CountryCode {
        CountryCode::parse(s).expect("valid country")
    }

    #[test]
    fn valid_address() {
        let a = Address::new(
            "10 Downing St".to_string(),
            None,
            "London".to_string(),
            None,
            uk("SW1A 2AA"),
            cc("GB"),
        )
        .expect("valid");
        assert_eq!(a.line1(), "10 Downing St");
        assert_eq!(a.city(), "London");
    }

    #[test]
    fn valid_with_all_fields() {
        let a = AddressBuilder::new()
            .line1("221B Baker St")
            .line2("Flat B")
            .city("London")
            .county("Greater London")
            .postcode(uk("NW1 6XE"))
            .country(cc("GB"))
            .build()
            .expect("valid");
        assert_eq!(a.line2.as_deref(), Some("Flat B"));
        assert_eq!(a.county.as_deref(), Some("Greater London"));
    }

    #[test]
    fn valid_with_coords() {
        let coords = Coords::new(51.5, -0.12).expect("valid coords");
        let a = Address::new_with_coords(
            "10 Downing St".to_string(),
            None,
            "London".to_string(),
            None,
            uk("SW1A 2AA"),
            cc("GB"),
            Some(coords),
        )
        .expect("valid");
        assert_eq!(a.coords, Some(coords));
    }

    #[test]
    fn invalid_empty_line1() {
        let res = Address::new(
            "   ".to_string(),
            None,
            "London".to_string(),
            None,
            uk("SW1A 2AA"),
            cc("GB"),
        );
        assert!(res.is_err());
    }

    #[test]
    fn invalid_empty_city() {
        let res = Address::new(
            "10 Downing St".to_string(),
            None,
            "".to_string(),
            None,
            uk("SW1A 2AA"),
            cc("GB"),
        );
        assert!(res.is_err());
    }

    #[test]
    fn invalid_line2_empty() {
        let res = Address::new(
            "10 Downing St".to_string(),
            Some("   ".to_string()),
            "London".to_string(),
            None,
            uk("SW1A 2AA"),
            cc("GB"),
        );
        assert!(res.is_err());
    }

    #[test]
    fn builder_missing_field() {
        let res = AddressBuilder::new().line1("10 Downing St").city("London").build();
        assert!(res.is_err());
    }
}
