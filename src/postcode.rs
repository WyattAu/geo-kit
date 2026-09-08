//! Postcode newtypes — UK, US, CA, DE, FR, JP, AU, and IN.

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
    // Guard against multibyte input: only split on a char boundary, otherwise
    // hand the unsplit string to the validator, which will reject it.
    if compact.len() <= 3 || !compact.is_char_boundary(compact.len() - 3) {
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
                    let init = match regex::Regex::new(r"^[A-Z]{1,2}[0-9][A-Z0-9]? [0-9][A-Z]{2}$")
                    {
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

/// Generic postcode covering UK, US, CA, DE, FR, JP, AU, and IN variants.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum Postcode {
    /// UK postcode.
    Uk(UkPostcode),
    /// US ZIP code.
    Us(UsZipCode),
    /// Canadian postcode.
    Ca(CaPostcode),
    /// German Postleitzahl.
    De(DePlz),
    /// French code postal.
    Fr(FrCp),
    /// Japanese postal code.
    Jp(JpPostal),
    /// Australian postcode.
    Au(AuPostcode),
    /// Indian PIN code.
    In(InPin),
}

impl Postcode {
    /// Parse against each country's format, in order: UK, CA, US, JP, DE,
    /// FR, AU, IN.
    ///
    /// Note that purely numeric formats are ambiguous between countries:
    /// a 5-digit string parses as US (matching pre-1.1 behaviour) and only
    /// otherwise as DE, then FR. Callers that need a specific country
    /// should parse the country-specific type directly.
    ///
    /// # Errors
    ///
    /// Returns [`GeoError::InvalidPostcode`] if no country format matches.
    pub fn parse(s: &str) -> Result<Self, GeoError> {
        if let Ok(uk) = UkPostcode::parse(s) {
            return Ok(Postcode::Uk(uk));
        }
        if let Ok(ca) = CaPostcode::parse(s) {
            return Ok(Postcode::Ca(ca));
        }
        if let Ok(us) = UsZipCode::parse(s) {
            return Ok(Postcode::Us(us));
        }
        if let Ok(jp) = JpPostal::parse(s) {
            return Ok(Postcode::Jp(jp));
        }
        if let Ok(de) = DePlz::parse(s) {
            return Ok(Postcode::De(de));
        }
        if let Ok(fr) = FrCp::parse(s) {
            return Ok(Postcode::Fr(fr));
        }
        if let Ok(au) = AuPostcode::parse(s) {
            return Ok(Postcode::Au(au));
        }
        if let Ok(pin) = InPin::parse(s) {
            return Ok(Postcode::In(pin));
        }
        Err(GeoError::InvalidPostcode(alloc::format!(
            "postcode '{s}' matches no supported country format"
        )))
    }

    /// Return as string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Postcode::Uk(x) => x.as_str(),
            Postcode::Us(x) => x.as_str(),
            Postcode::Ca(x) => x.as_str(),
            Postcode::De(x) => x.as_str(),
            Postcode::Fr(x) => x.as_str(),
            Postcode::Jp(x) => x.as_str(),
            Postcode::Au(x) => x.as_str(),
            Postcode::In(x) => x.as_str(),
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

/// Normalize a Canadian postcode: trim, uppercase, remove all whitespace,
/// then re-insert a single space before the last 3 characters.
fn normalize_ca(input: &str) -> String {
    let compact: String = input
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_uppercase();
    if compact.len() <= 3 || !compact.is_char_boundary(compact.len() - 3) {
        return compact;
    }
    let split_at = compact.len() - 3;
    let (outward, inward) = compact.split_at(split_at);
    alloc::format!("{outward} {inward}")
}

/// Normalize a Japanese postal code: trim, remove all whitespace and
/// hyphens, then re-insert the hyphen after the first 3 digits.
fn normalize_jp(input: &str) -> String {
    let compact: String = input
        .trim()
        .chars()
        .filter(|c| *c != '-' && !c.is_whitespace())
        .collect();
    if compact.len() < 4 || !compact.is_char_boundary(3) {
        return compact;
    }
    let (head, tail) = compact.split_at(3);
    alloc::format!("{head}-{tail}")
}

/// Digits-only fixed-length check (shared by DE, FR, AU).
#[cfg(not(feature = "regex"))]
fn check_digits_exact(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|b| b.is_ascii_digit())
}

#[cfg(not(feature = "regex"))]
fn check_de(s: &str) -> bool {
    check_digits_exact(s, 5)
}

#[cfg(not(feature = "regex"))]
fn check_fr(s: &str) -> bool {
    check_digits_exact(s, 5)
}

#[cfg(not(feature = "regex"))]
fn check_au(s: &str) -> bool {
    check_digits_exact(s, 4)
}

/// India PIN: 6 digits, first digit 1-9.
#[cfg(not(feature = "regex"))]
fn check_in(s: &str) -> bool {
    s.len() == 6
        && s.bytes().all(|b| b.is_ascii_digit())
        && s.starts_with(['1', '2', '3', '4', '5', '6', '7', '8', '9'])
}

/// Canada: `A1A 1A1` (normalized with a single space).
#[cfg(not(feature = "regex"))]
fn check_ca(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 7 || bytes[3] != b' ' {
        return false;
    }
    let pattern = [
        (b'A', b'Z'),
        (b'0', b'9'),
        (b'A', b'Z'),
        (b' ', b' '),
        (b'0', b'9'),
        (b'A', b'Z'),
        (b'0', b'9'),
    ];
    bytes
        .iter()
        .zip(pattern.iter())
        .all(|(&b, &(lo, hi))| b >= lo && b <= hi)
}

/// Japan: `NNN-NNNN` (normalized with a hyphen).
#[cfg(not(feature = "regex"))]
fn check_jp(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 8 || bytes[3] != b'-' {
        return false;
    }
    bytes.iter().enumerate().all(|(i, &b)| {
        if i == 3 {
            b == b'-'
        } else {
            b.is_ascii_digit()
        }
    })
}

macro_rules! define_fixed_postcode {
    (
        $(#[$doc:meta])*
        $name:ident, $label:literal, $is_valid:ident {
            pattern: $pattern:literal,
            normalize: $normalize:expr,
            check: $check:expr,
        }
    ) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(transparent))]
        pub struct $name(String);

        impl $name {
            /// Parse and validate.
            ///
            /// Input is normalized (whitespace/case per the country's
            /// convention) before validation and storage.
            ///
            /// # Errors
            ///
            /// Returns [`GeoError::InvalidPostcode`] if validation fails.
            pub fn parse(s: &str) -> Result<Self, GeoError> {
                if s.is_empty() {
                    return Err(GeoError::InvalidPostcode(concat!($label, " is empty").to_string()));
                }
                if s.contains('\r') || s.contains('\n') || s.contains('\t') {
                    return Err(GeoError::InvalidPostcode(
                        concat!($label, " contains control character").to_string(),
                    ));
                }
                let normalized = $normalize(s);

                #[cfg(all(feature = "regex", feature = "std"))]
                {
                    static RE_CACHE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
                    let re = match RE_CACHE.get() {
                        Some(r) => r,
                        None => {
                            let compiled =
                                regex::Regex::new($pattern).map_err(|_| {
                                    GeoError::InvalidPostcode(
                                        "internal regex error".to_string(),
                                    )
                                })?;
                            RE_CACHE.get_or_init(move || compiled)
                        }
                    };
                    if !re.is_match(&normalized) {
                        return Err(GeoError::InvalidPostcode(alloc::format!(
                            concat!($label, " '{}' does not match expected pattern"),
                            s
                        )));
                    }
                }

                #[cfg(all(feature = "regex", not(feature = "std")))]
                {
                    let re = regex::Regex::new($pattern).map_err(|_| {
                        GeoError::InvalidPostcode("internal regex error".to_string())
                    })?;
                    if !re.is_match(&normalized) {
                        return Err(GeoError::InvalidPostcode(alloc::format!(
                            concat!($label, " '{}' does not match expected pattern"),
                            s
                        )));
                    }
                }

                #[cfg(not(feature = "regex"))]
                {
                    if !($check)(&normalized) {
                        return Err(GeoError::InvalidPostcode(alloc::format!(
                            concat!($label, " '{}' does not match expected pattern"),
                            s
                        )));
                    }
                }

                Ok($name(normalized))
            }

            /// Create from an owned string.
            ///
            /// # Errors
            ///
            /// Returns [`GeoError::InvalidPostcode`] if validation fails.
            pub fn new(s: String) -> Result<Self, GeoError> {
                Self::parse(&s)
            }

            /// Return as string slice (normalized form).
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

        #[must_use]
        #[doc = concat!("Returns `true` if `s` is a valid ", $label, ".")]
        pub fn $is_valid(s: &str) -> bool {
            $name::parse(s).is_ok()
        }

        impl Deref for $name {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl TryFrom<String> for $name {
            type Error = GeoError;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                $name::new(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = GeoError;
            fn try_from(value: &str) -> Result<Self, Self::Error> {
                $name::parse(value)
            }
        }

        impl FromStr for $name {
            type Err = GeoError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $name::parse(s)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

define_fixed_postcode!(
    /// A validated Canadian postcode, e.g. `K1A 0B1`.
    ///
    /// Validation:
    /// - uppercase normalized, whitespace removed, single space re-inserted
    /// - regex `^[A-Z][0-9][A-Z] [0-9][A-Z][0-9]$` when `regex` feature enabled,
    ///   otherwise hand-rolled equivalent
    CaPostcode, "Canadian postcode", is_valid_ca_postcode {
        pattern: "^[A-Z][0-9][A-Z] [0-9][A-Z][0-9]$",
        normalize: normalize_ca,
        check: check_ca,
    }
);

define_fixed_postcode!(
    /// A validated German Postleitzahl, e.g. `10115`.
    ///
    /// Validation: exactly 5 ASCII digits (`^[0-9]{5}$`); leading zeros are
    /// valid. Whitespace/control input is rejected.
    DePlz, "German PLZ", is_valid_de_plz {
        pattern: "^[0-9]{5}$",
        normalize: alloc::string::String::from,
        check: check_de,
    }
);

define_fixed_postcode!(
    /// A validated French code postal, e.g. `75001`.
    ///
    /// Validation: exactly 5 ASCII digits (`^[0-9]{5}$`); leading zeros are
    /// valid. Whitespace/control input is rejected.
    FrCp, "French code postal", is_valid_fr_cp {
        pattern: "^[0-9]{5}$",
        normalize: alloc::string::String::from,
        check: check_fr,
    }
);

define_fixed_postcode!(
    /// A validated Japanese postal code, e.g. `100-0001`.
    ///
    /// Validation:
    /// - hyphen and whitespace removed on input, hyphen re-inserted after
    ///   the first 3 digits
    /// - regex `^[0-9]{3}-[0-9]{4}$` when `regex` feature enabled, otherwise
    ///   hand-rolled equivalent
    JpPostal, "Japanese postal code", is_valid_jp_postal {
        pattern: "^[0-9]{3}-[0-9]{4}$",
        normalize: normalize_jp,
        check: check_jp,
    }
);

define_fixed_postcode!(
    /// A validated Australian postcode, e.g. `2000`.
    ///
    /// Validation: exactly 4 ASCII digits (`^[0-9]{4}$`); leading zeros are
    /// valid (e.g. NT `0800`). Whitespace/control input is rejected.
    AuPostcode, "Australian postcode", is_valid_au_postcode {
        pattern: "^[0-9]{4}$",
        normalize: alloc::string::String::from,
        check: check_au,
    }
);

define_fixed_postcode!(
    /// A validated Indian PIN code, e.g. `110001`.
    ///
    /// Validation: 6 ASCII digits with a non-zero first digit
    /// (`^[1-9][0-9]{5}$`). Whitespace/control input is rejected.
    InPin, "Indian PIN code", is_valid_in_pin {
        pattern: "^[1-9][0-9]{5}$",
        normalize: alloc::string::String::from,
        check: check_in,
    }
);

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
        assert!(matches!(
            Postcode::parse("SW1A 1AA").unwrap(),
            Postcode::Uk(_)
        ));
        assert!(matches!(Postcode::parse("90210").unwrap(), Postcode::Us(_)));
        assert!(Postcode::parse("INVALID").is_err());
    }

    #[test]
    fn uk_rejects_multibyte_input_without_panicking() {
        // Regression: normalize_uk used split_at on a non-char boundary,
        // which panicked on multibyte input. Must now be a validation error.
        for evil in ["ééé", "日本語", "é1 1AA", "SW1A éAA"] {
            assert!(UkPostcode::parse(evil).is_err(), "must reject {evil}");
            assert!(Postcode::parse(evil).is_err(), "must reject {evil}");
        }
    }

    // Canada ------------------------------------------------------------

    #[test]
    fn valid_ca() {
        for (input, expected) in [
            ("K1A 0B1", "K1A 0B1"),
            ("k1a0b1", "K1A 0B1"),
            ("M5V 2T6", "M5V 2T6"),
            ("  h0h 0h0  ", "H0H 0H0"),
        ] {
            let pc = CaPostcode::parse(input).unwrap_or_else(|e| panic!("{input}: {e}"));
            assert_eq!(pc.as_str(), expected);
            assert!(is_valid_ca_postcode(input));
        }
    }

    #[test]
    fn invalid_ca() {
        for evil in [
            "",
            "K1A 0B",
            "K1A0B",
            "K1A  O B1",
            "11A 0B1",
            "K1A OB1", // letter instead of digit in last position
            "K1A 0B11",
            "K1A-0B1",
            "K1A 0B1\r",
        ] {
            assert!(CaPostcode::parse(evil).is_err(), "must reject {evil}");
        }
    }

    // Germany -----------------------------------------------------------

    #[test]
    fn valid_de() {
        for input in ["10115", "01067", "80331"] {
            assert!(DePlz::parse(input).is_ok(), "must accept {input}");
            assert!(is_valid_de_plz(input));
        }
        let pc = DePlz::parse("10115").unwrap();
        assert_eq!(pc.as_str(), "10115");
    }

    #[test]
    fn invalid_de() {
        for evil in [
            "",
            "1011",
            "101156",
            "10115A",
            "101 15",
            "۱۰۱۱۵",
            "-1011",
            "  20095",
        ] {
            assert!(DePlz::parse(evil).is_err(), "must reject {evil}");
        }
    }

    // France ------------------------------------------------------------

    #[test]
    fn valid_fr() {
        for input in ["75001", "01000", "20090", "97100"] {
            assert!(FrCp::parse(input).is_ok(), "must accept {input}");
            assert!(is_valid_fr_cp(input));
        }
    }

    #[test]
    fn invalid_fr() {
        for evil in ["", "7500", "750011", "7500A", "75 001", "75001\n"] {
            assert!(FrCp::parse(evil).is_err(), "must reject {evil}");
        }
    }

    // Japan -------------------------------------------------------------

    #[test]
    fn valid_jp_normalizes() {
        for (input, expected) in [
            ("100-0001", "100-0001"),
            ("1000001", "100-0001"),
            ("605-0017", "605-0017"),
            (" 6050017 ", "605-0017"),
        ] {
            let pc = JpPostal::parse(input).unwrap_or_else(|e| panic!("{input}: {e}"));
            assert_eq!(pc.as_str(), expected);
            assert!(is_valid_jp_postal(input));
        }
    }

    #[test]
    fn invalid_jp() {
        for evil in [
            "",
            "100-000",
            "100-00011",
            "1O0-0001",
            "100_0001",
            "100 0001\n",
        ] {
            assert!(JpPostal::parse(evil).is_err(), "must reject {evil}");
        }
    }

    // Australia ---------------------------------------------------------

    #[test]
    fn valid_au() {
        for input in ["2000", "0800", "3000"] {
            assert!(AuPostcode::parse(input).is_ok(), "must accept {input}");
            assert!(is_valid_au_postcode(input));
        }
    }

    #[test]
    fn invalid_au() {
        for evil in ["", "200", "20000", "200A", "20 00", "-2000", " 4000"] {
            assert!(AuPostcode::parse(evil).is_err(), "must reject {evil}");
        }
    }

    // India -------------------------------------------------------------

    #[test]
    fn valid_in() {
        for input in ["110001", "400001", "560001"] {
            assert!(InPin::parse(input).is_ok(), "must accept {input}");
            assert!(is_valid_in_pin(input));
        }
    }

    #[test]
    fn invalid_in() {
        for evil in [
            "", "11000", "1100011", "010001", "11000A", "11 0001", " 700001",
        ] {
            assert!(InPin::parse(evil).is_err(), "must reject {evil}");
        }
    }

    // Generic dispatch ----------------------------------------------------

    #[test]
    fn generic_postcode_new_variants() {
        assert!(matches!(
            Postcode::parse("K1A 0B1").unwrap(),
            Postcode::Ca(_)
        ));
        assert!(matches!(Postcode::parse("10115").unwrap(), Postcode::Us(_)));
        assert!(matches!(
            Postcode::parse("605-0017").unwrap(),
            Postcode::Jp(_)
        ));
        assert!(matches!(Postcode::parse("2000").unwrap(), Postcode::Au(_)));
        assert!(matches!(
            Postcode::parse("110001").unwrap(),
            Postcode::In(_)
        ));
        // 5-digit strings that fail US (reserved, all digits pass US...) —
        // US accepts any 5 digits, so DE/FR are only reachable via their
        // own types; documented in Postcode::parse.
        assert!(Postcode::parse("@@@@").is_err());
    }

    #[test]
    fn generic_postcode_tryfrom_and_display() {
        let pc: Postcode = "605-0017".parse().unwrap();
        assert_eq!(pc.to_string(), "605-0017");
        let pc2 = Postcode::try_from(String::from("K1A 0B1")).unwrap();
        assert_eq!(pc2.to_string(), "K1A 0B1");
        let pc3 = Postcode::try_from("2000").unwrap();
        assert_eq!(pc3.as_str(), "2000");
    }

    #[test]
    fn fixed_types_tryfrom_fromstr_asref() {
        let de = DePlz::try_from(String::from("10115")).unwrap();
        let parsed: DePlz = "10115".parse().unwrap();
        assert_eq!(de, parsed);
        assert_eq!(de.as_ref(), "10115");
        let s: &str = de.as_str();
        assert_eq!(s, "10115");

        let au = AuPostcode::try_from("0800").unwrap();
        assert_eq!(au.into_inner(), "0800");
    }
}
