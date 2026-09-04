//! Postcode newtypes — UK and US.

extern crate alloc;

use alloc::string::{String, ToString};
use core::fmt;
use core::ops::Deref;
use core::str::FromStr;

use crate::error::GeoError;

/// A validated UK postcode, e.g. `SW1A 1AA`.
///
/// Validation:
/// - uppercase normalized
/// - space optional on input, stored with single space separating outward and inward
/// - regex `^[A-Z]{1,2}[0-9][A-Z0-9]? [0-9][A-Z]{2}$` when `regex` feature enabled
/// - otherwise hand-rolled equivalent
/// - outward 1–4 alphanum (1–2 letters + digit + optional alphanum), inward exactly `digit + 2 letters`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct UkPostcode(String);

impl UkPostcode {
    /// Parse and validate a UK postcode.
    ///
    /// # Errors
    ///
    /// Returns [`GeoError::InvalidPostcode`] if validation fails.
    pub fn parse(s: &str) -> Result<Self, GeoError> {
        validate_uk(s)
    }

    /// Create from an owned string.
    ///
    /// # Errors
    ///
    /// Returns [`GeoError::InvalidPostcode`] if validation fails.
    pub fn new(s: String) -> Result<Self, GeoError> {
        validate_uk(&s)
    }

    /// Return as string slice (normalized with single space).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return inner string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Returns `true` if `s` is a valid UK postcode.
#[must_use]
pub fn is_valid_uk_postcode(s: &str) -> bool {
    validate_uk(s).is_ok()
}

/// Normalize UK postcode: trim, uppercase, remove all spaces, then re-insert single space before last 3 chars.
fn normalize_uk(input: &str) -> String {
    let upper = input.trim().to_ascii_uppercase();
    // Remove all spaces
    let compact: String = upper.chars().filter(|c| *c != ' ').collect();
    if compact.len() <= 3 {
        return compact;
    }
    let split_at = compact.len() - 3;
    let (outward, inward) = compact.split_at(split_at);
    alloc::format!("{} {}", outward, inward)
}

fn validate_uk(input: &str) -> Result<UkPostcode, GeoError> {
    if input.is_empty() {
        return Err(GeoError::InvalidPostcode("postcode is empty".to_string()));
    }
    if input.contains('\r') || input.contains('\n') || input.contains('\t') {
        return Err(GeoError::InvalidPostcode(
            "postcode contains control character".to_string(),
        ));
    }
    // Normalized form with single space.
    let normalized = normalize_uk(input);

    // GIR 0AA is the special-case UK postcode ( fertile Giro bank ) that
    // predates the standard outward/inward pattern and does not match the
    // regular expression below.
    if normalized == "GIR 0AA" {
        return Ok(UkPostcode(normalized));
    }

    #[cfg(feature = "regex")]
    {
        #[cfg(feature = "std")]
        {
            use std::sync::OnceLock;
            static RE: OnceLock<regex::Regex> = OnceLock::new();
            let re = match RE.get() {
                Some(r) => r,
                None => {
                    let init = match regex::Regex::new(r"^[A-Z]{1,2}[0-9][A-Z0-9]? [0-9][A-Z]{2}$") {
                        Ok(r) => r,
                        Err(_) => {
                            return Err(GeoError::InvalidPostcode(
                                "internal regex error".to_string(),
                            ))
                        }
                    };
                    let _ = RE.set(init);
                    match RE.get() {
                        Some(r) => r,
                        None => {
                            return Err(GeoError::InvalidPostcode(
                                "internal regex error".to_string(),
                            ))
                        }
                    }
                }
            };
            if !re.is_match(&normalized) {
                return Err(GeoError::InvalidPostcode(alloc::format!(
                    "postcode '{}' does not match UK pattern",
                    input
                )));
            }
        }
        #[cfg(not(feature = "std"))]
        {
            let re = match regex::Regex::new(r"^[A-Z]{1,2}[0-9][A-Z0-9]? [0-9][A-Z]{2}$") {
                Ok(r) => r,
                Err(_) => {
                    return Err(GeoError::InvalidPostcode(
                        "internal regex error".to_string(),
                    ))
                }
            };
            if !re.is_match(&normalized) {
                return Err(GeoError::InvalidPostcode(alloc::format!(
                    "postcode '{}' does not match UK pattern",
                    input
                )));
            }
        }
        Ok(UkPostcode(normalized))
    }

    #[cfg(not(feature = "regex"))]
    {
        // Hand-rolled: outward 1-2 letters + digit + optional alnum, inward digit + 2 letters
        let parts: alloc::vec::Vec<&str> = normalized.split(' ').collect();
        if parts.len() != 2 {
            return Err(GeoError::InvalidPostcode(alloc::format!(
                "postcode '{}' must have outward and inward parts",
                input
            )));
        }
        let outward = parts[0];
        let inward = parts[1];

        // Inward: 3 chars, ^[0-9][A-Z]{2}$
        if inward.len() != 3 {
            return Err(GeoError::InvalidPostcode(alloc::format!(
                "inward '{}' must be 3 chars (digit + 2 letters)",
                inward
            )));
        }
        let ib = inward.as_bytes();
        if !ib[0].is_ascii_digit() || !ib[1].is_ascii_uppercase() || !ib[2].is_ascii_uppercase() {
            return Err(GeoError::InvalidPostcode(alloc::format!(
                "inward '{}' must be digit + 2 letters",
                inward
            )));
        }

        // Outward: 2-4 chars, ^[A-Z]{1,2}[0-9][A-Z0-9]?$
        if outward.len() < 2 || outward.len() > 4 {
            return Err(GeoError::InvalidPostcode(alloc::format!(
                "outward '{}' must be 2-4 chars",
                outward
            )));
        }
        let ob = outward.as_bytes();
        // Count leading letters: 1 or 2
        let mut letter_count = 0;
        for &b in ob {
            if b.is_ascii_uppercase() {
                letter_count += 1;
            } else {
                break;
            }
        }
        if letter_count < 1 || letter_count > 2 {
            return Err(GeoError::InvalidPostcode(alloc::format!(
                "outward '{}' must start with 1-2 letters",
                outward
            )));
        }
        // After letters must be digit
        if !ob[letter_count].is_ascii_digit() {
            return Err(GeoError::InvalidPostcode(alloc::format!(
                "outward '{}' must have digit after letters",
                outward
            )));
        }
        // Optional trailing alnum
        let remaining = ob.len() - (letter_count + 1);
        if remaining > 1 {
            return Err(GeoError::InvalidPostcode(alloc::format!(
                "outward '{}' too long",
                outward
            )));
        }
        if remaining == 1 {
            let last = ob[ob.len() - 1];
            if !last.is_ascii_alphanumeric() {
                return Err(GeoError::InvalidPostcode(alloc::format!(
                    "outward '{}' last char must be alphanumeric",
                    outward
                )));
            }
            // Must be uppercase letter or digit (already uppercased, so check not lowercase)
            if last.is_ascii_lowercase() {
                return Err(GeoError::InvalidPostcode(alloc::format!(
                    "outward '{}' must be uppercase alphanumeric",
                    outward
                )));
            }
        }
        // Ensure all chars are alphanumeric
        for &b in ob {
            if !b.is_ascii_alphanumeric() {
                return Err(GeoError::InvalidPostcode(alloc::format!(
                    "outward '{}' must be alphanumeric",
                    outward
                )));
            }
        }
        Ok(UkPostcode(normalized))
    }
}

impl Deref for UkPostcode {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for UkPostcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for UkPostcode {
    type Error = GeoError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        UkPostcode::new(value)
    }
}

impl TryFrom<&str> for UkPostcode {
    type Error = GeoError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        UkPostcode::parse(value)
    }
}

impl FromStr for UkPostcode {
    type Err = GeoError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        UkPostcode::parse(s)
    }
}

impl AsRef<str> for UkPostcode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A validated US ZIP code, e.g. `90210` or `90210-1234`.
///
/// Validation: `^\d{5}(-\d{4})?$`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct UsZipCode(String);

impl UsZipCode {
    /// Parse and validate a US ZIP code.
    ///
    /// # Errors
    ///
    /// Returns [`GeoError::InvalidUsZip`] if validation fails.
    pub fn parse(s: &str) -> Result<Self, GeoError> {
        validate_us(s)
    }

    /// Create from an owned string.
    ///
    /// # Errors
    ///
    /// Returns [`GeoError::InvalidUsZip`] if validation fails.
    pub fn new(s: String) -> Result<Self, GeoError> {
        validate_us(&s)
    }

    /// Return as string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return inner string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Returns `true` if `s` is a valid US ZIP code.
#[must_use]
pub fn is_valid_us_zip(s: &str) -> bool {
    validate_us(s).is_ok()
}

fn validate_us(input: &str) -> Result<UsZipCode, GeoError> {
    if input.is_empty() {
        return Err(GeoError::InvalidUsZip("zip is empty".to_string()));
    }
    if input.contains('\r') || input.contains('\n') || input.contains(' ') || input.contains('\t') {
        return Err(GeoError::InvalidUsZip(
            "zip must not contain whitespace or control".to_string(),
        ));
    }

    #[cfg(feature = "regex")]
    {
        #[cfg(feature = "std")]
        {
            use std::sync::OnceLock;
            static RE: OnceLock<regex::Regex> = OnceLock::new();
            let re = match RE.get() {
                Some(r) => r,
                None => {
                    let init = match regex::Regex::new(r"^\d{5}(-\d{4})?$") {
                        Ok(r) => r,
                        Err(_) => {
                            return Err(GeoError::InvalidUsZip("internal regex error".to_string()))
                        }
                    };
                    let _ = RE.set(init);
                    match RE.get() {
                        Some(r) => r,
                        None => {
                            return Err(GeoError::InvalidUsZip("internal regex error".to_string()))
                        }
                    }
                }
            };
            if !re.is_match(input) {
                return Err(GeoError::InvalidUsZip(alloc::format!(
                    "zip '{}' must match XXX or XXXXX-XXXX",
                    input
                )));
            }
        }
        #[cfg(not(feature = "std"))]
        {
            let re = match regex::Regex::new(r"^\d{5}(-\d{4})?$") {
                Ok(r) => r,
                Err(_) => return Err(GeoError::InvalidUsZip("internal regex error".to_string())),
            };
            if !re.is_match(input) {
                return Err(GeoError::InvalidUsZip(alloc::format!(
                    "zip '{}' must match XXX or XXXXX-XXXX",
                    input
                )));
            }
        }
        Ok(UsZipCode(input.to_string()))
    }

    #[cfg(not(feature = "regex"))]
    {
        let bytes = input.as_bytes();
        if bytes.len() == 5 {
            if bytes.iter().all(|b| b.is_ascii_digit()) {
                return Ok(UsZipCode(input.to_string()));
            }
            return Err(GeoError::InvalidUsZip(alloc::format!(
                "zip '{}' must be 5 digits",
                input
            )));
        }
        if bytes.len() == 10 {
            // XXXXX-XXXX
            if bytes[5] != b'-' {
                return Err(GeoError::InvalidUsZip(alloc::format!(
                    "zip '{}' must have '-' at position 6",
                    input
                )));
            }
            if bytes[..5].iter().all(|b| b.is_ascii_digit())
                && bytes[6..].iter().all(|b| b.is_ascii_digit())
            {
                return Ok(UsZipCode(input.to_string()));
            }
            return Err(GeoError::InvalidUsZip(alloc::format!(
                "zip '{}' must be 5 digits, hyphen, 4 digits",
                input
            )));
        }
        Err(GeoError::InvalidUsZip(alloc::format!(
            "zip '{}' must be 5 digits or 5-4 extended",
            input
        )))
    }
}

impl Deref for UsZipCode {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for UsZipCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for UsZipCode {
    type Error = GeoError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        UsZipCode::new(value)
    }
}

impl TryFrom<&str> for UsZipCode {
    type Error = GeoError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        UsZipCode::parse(value)
    }
}

impl FromStr for UsZipCode {
    type Err = GeoError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        UsZipCode::parse(s)
    }
}

impl AsRef<str> for UsZipCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Generic postcode covering UK and US variants.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum Postcode {
    /// UK postcode.
    Uk(UkPostcode),
    /// US ZIP code.
    Us(UsZipCode),
}

impl Postcode {
    /// Parse as UK postcode first, then US ZIP.
    ///
    /// # Errors
    ///
    /// Returns [`GeoError::InvalidPostcode`] if neither matches.
    pub fn parse(s: &str) -> Result<Self, GeoError> {
        if let Ok(uk) = UkPostcode::parse(s) {
            return Ok(Postcode::Uk(uk));
        }
        if let Ok(us) = UsZipCode::parse(s) {
            return Ok(Postcode::Us(us));
        }
        Err(GeoError::InvalidPostcode(alloc::format!(
            "postcode '{}' is neither valid UK nor US",
            s
        )))
    }

    /// Return as string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Postcode::Uk(x) => x.as_str(),
            Postcode::Us(x) => x.as_str(),
        }
    }
}

impl fmt::Display for Postcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Postcode {
    type Err = GeoError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Postcode::parse(s)
    }
}

impl TryFrom<String> for Postcode {
    type Error = GeoError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Postcode::parse(&value)
    }
}

impl TryFrom<&str> for Postcode {
    type Error = GeoError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Postcode::parse(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_uk_with_space() {
        assert!(UkPostcode::parse("SW1A 1AA").is_ok());
        assert!(UkPostcode::parse("EC1A 1BB").is_ok());
        assert!(UkPostcode::parse("M1 1AE").is_ok());
        assert!(UkPostcode::parse("B33 8TH").is_ok());
        assert!(UkPostcode::parse("CR2 6XH").is_ok());
        assert!(UkPostcode::parse("DN55 1PT").is_ok());
    }

    #[test]
    fn valid_uk_gir_special_case() {
        // GIR 0AA is the historic Giro bank postcode — valid despite not
        // matching the standard outward/inward pattern.
        let pc = UkPostcode::parse("GIR 0AA").expect("GIR 0AA must be valid");
        assert_eq!(pc.as_str(), "GIR 0AA");
        assert!(UkPostcode::parse("gir0aa").is_ok()); // case/space variants
        assert!(is_valid_uk_postcode("GIR 0AA"));
    }

    #[test]
    fn valid_uk_without_space_normalizes() {
        let pc = UkPostcode::parse("SW1A1AA").expect("valid");
        assert_eq!(pc.as_str(), "SW1A 1AA");
        let pc2 = UkPostcode::parse("m11ae").expect("valid"); // lowercase
        assert_eq!(pc2.as_str(), "M1 1AE");
    }

    #[test]
    fn valid_uk_lowercase_normalized() {
        let pc = UkPostcode::parse("sw1a 1aa").expect("valid");
        assert_eq!(pc.as_str(), "SW1A 1AA");
    }

    #[test]
    fn invalid_uk() {
        assert!(UkPostcode::parse("").is_err());
        assert!(UkPostcode::parse("SW1A1A").is_err()); // inward too short
        assert!(UkPostcode::parse("12345").is_err());
        assert!(UkPostcode::parse("SW1A 1A").is_err());
        assert!(UkPostcode::parse("ZZZ 1AA").is_err()); // too many letters outward
        assert!(UkPostcode::parse("SW1A 1AAA").is_err());
        assert!(UkPostcode::parse("SWA 1AA").is_err()); // letter after digit where digit expected
    }

    #[test]
    fn valid_us() {
        assert!(UsZipCode::parse("90210").is_ok());
        assert!(UsZipCode::parse("12345-6789").is_ok());
        assert!(UsZipCode::parse("00501").is_ok());
    }

    #[test]
    fn invalid_us() {
        assert!(UsZipCode::parse("").is_err());
        assert!(UsZipCode::parse("9021").is_err());
        assert!(UsZipCode::parse("902101").is_err());
        assert!(UsZipCode::parse("9021A").is_err());
        assert!(UsZipCode::parse("1234-5678").is_err());
        assert!(UsZipCode::parse("12345-678").is_err());
        assert!(UsZipCode::parse(" 90210").is_err());
    }

    #[test]
    fn generic_postcode() {
        assert!(matches!(Postcode::parse("SW1A 1AA").unwrap(), Postcode::Uk(_)));
        assert!(matches!(Postcode::parse("90210").unwrap(), Postcode::Us(_)));
        assert!(Postcode::parse("INVALID").is_err());
    }
}
