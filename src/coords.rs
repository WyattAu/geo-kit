//! Geographic coordinates newtype.

extern crate alloc;

use alloc::string::ToString;
use core::fmt;

use crate::error::GeoError;

/// Geographic coordinates in decimal degrees.
///
/// Validation:
/// - `lat` must be finite and in `[-90.0, 90.0]`
/// - `lon` must be finite and in `[-180.0, 180.0]`
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Coords {
    /// Latitude in decimal degrees, `-90..=90`.
    pub lat: f64,
    /// Longitude in decimal degrees, `-180..=180`.
    pub lon: f64,
}

impl Coords {
    /// Create validated coordinates.
    ///
    /// # Errors
    ///
    /// Returns [`GeoError::InvalidCoords`] if validation fails.
    pub fn new(lat: f64, lon: f64) -> Result<Self, GeoError> {
        validate_coords(lat, lon)
    }

    /// Parse from a string slice in the form `"lat,lon"` or `"lat lon"`.
    ///
    /// Whitespace around numbers and separator is allowed.
    ///
    /// # Errors
    ///
    /// Returns [`GeoError::InvalidCoords`] if parsing or validation fails.
    pub fn parse(s: &str) -> Result<Self, GeoError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(GeoError::InvalidCoords("coords string is empty".to_string()));
        }
        // Support ',' or whitespace separated, but prefer comma.
        let (lat_str, lon_str) = if let Some(idx) = trimmed.find(',') {
            let (a, b) = trimmed.split_at(idx);
            (a.trim(), b[1..].trim())
        } else {
            // split on whitespace
            let mut parts = trimmed.split_whitespace();
            let a = parts.next();
            let b = parts.next();
            let extra = parts.next();
            match (a, b, extra) {
                (Some(a), Some(b), None) => (a, b),
                _ => {
                    return Err(GeoError::InvalidCoords(alloc::format!(
                        "coords '{}' must be 'lat,lon' or 'lat lon'",
                        s
                    )))
                }
            }
        };
        let lat: f64 = lat_str.parse().map_err(|_| {
            GeoError::InvalidCoords(alloc::format!("invalid latitude '{}'", lat_str))
        })?;
        let lon: f64 = lon_str.parse().map_err(|_| {
            GeoError::InvalidCoords(alloc::format!("invalid longitude '{}'", lon_str))
        })?;
        validate_coords(lat, lon)
    }

    /// Return latitude.
    #[must_use]
    pub fn lat(&self) -> f64 {
        self.lat
    }

    /// Return longitude.
    #[must_use]
    pub fn lon(&self) -> f64 {
        self.lon
    }
}

fn validate_coords(lat: f64, lon: f64) -> Result<Coords, GeoError> {
    if !lat.is_finite() {
        return Err(GeoError::InvalidCoords(alloc::format!(
            "latitude {} is not finite",
            lat
        )));
    }
    if !lon.is_finite() {
        return Err(GeoError::InvalidCoords(alloc::format!(
            "longitude {} is not finite",
            lon
        )));
    }
    if !(-90.0..=90.0).contains(&lat) {
        return Err(GeoError::InvalidCoords(alloc::format!(
            "latitude {} out of range -90..90",
            lat
        )));
    }
    if !(-180.0..=180.0).contains(&lon) {
        return Err(GeoError::InvalidCoords(alloc::format!(
            "longitude {} out of range -180..180",
            lon
        )));
    }
    Ok(Coords { lat, lon })
}

/// Returns `true` if `lat` and `lon` are valid coordinates.
#[must_use]
pub fn is_valid_coords(lat: f64, lon: f64) -> bool {
    validate_coords(lat, lon).is_ok()
}

impl fmt::Display for Coords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.lat, self.lon)
    }
}

impl core::str::FromStr for Coords {
    type Err = GeoError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Coords::parse(s)
    }
}

impl TryFrom<(f64, f64)> for Coords {
    type Error = GeoError;
    fn try_from(value: (f64, f64)) -> Result<Self, Self::Error> {
        Coords::new(value.0, value.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_coords() {
        assert!(Coords::new(51.5074, -0.1278).is_ok());
        assert!(Coords::new(90.0, 180.0).is_ok());
        assert!(Coords::new(-90.0, -180.0).is_ok());
        assert!(Coords::new(0.0, 0.0).is_ok());
    }

    #[test]
    fn invalid_lat() {
        assert!(Coords::new(91.0, 0.0).is_err());
        assert!(Coords::new(-91.0, 0.0).is_err());
        assert!(Coords::new(f64::NAN, 0.0).is_err());
        assert!(Coords::new(f64::INFINITY, 0.0).is_err());
    }

    #[test]
    fn invalid_lon() {
        assert!(Coords::new(0.0, 181.0).is_err());
        assert!(Coords::new(0.0, -181.0).is_err());
        assert!(Coords::new(0.0, f64::NAN).is_err());
    }

    #[test]
    fn parse_comma() {
        let c = Coords::parse("51.5074,-0.1278").expect("valid");
        assert!((c.lat - 51.5074).abs() < 1e-9);
        assert!((c.lon - -0.1278).abs() < 1e-9);
    }

    #[test]
    fn parse_space() {
        let c = Coords::parse("51.5074 -0.1278").expect("valid");
        assert!((c.lat - 51.5074).abs() < 1e-9);
    }

    #[test]
    fn parse_invalid() {
        assert!(Coords::parse("").is_err());
        assert!(Coords::parse("91,0").is_err());
        assert!(Coords::parse("0,181").is_err());
        assert!(Coords::parse("not a coord").is_err());
        assert!(Coords::parse("51.5").is_err());
    }

    #[test]
    fn is_valid_helper() {
        assert!(is_valid_coords(0.0, 0.0));
        assert!(!is_valid_coords(91.0, 0.0));
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_roundtrip() {
        let coords = Coords::new(51.5, -0.12).expect("valid");
        let json = serde_json::to_string(&coords).expect("serialize");
        let de: Coords = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(coords, de);
    }
}
