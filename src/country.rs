//! ISO 3166-1 alpha-2 country code newtype.

extern crate alloc;

use alloc::string::{String, ToString};
use core::fmt;
use core::ops::Deref;
use core::str::FromStr;

use crate::error::GeoError;

/// A validated ISO 3166-1 alpha-2 country code, e.g. `GB`, `US`.
///
/// Validation:
/// - exactly 2 characters
/// - `^[A-Z]{2}$` (uppercase ASCII)
/// - when `regex` feature is enabled, regex is used; otherwise hand-rolled check
///
/// The type accepts any `A-Z` pair as syntactically valid; [`CountryCode::country_name`]
/// returns a human-readable name for known codes and `"Unknown"` otherwise.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct CountryCode(String);

impl CountryCode {
    /// Parse and validate a country code.
    ///
    /// # Errors
    ///
    /// Returns [`GeoError::InvalidCountry`] if validation fails.
    pub fn parse(s: &str) -> Result<Self, GeoError> {
        validate_country(s)
    }

    /// Create from an owned string.
    ///
    /// # Errors
    ///
    /// Returns [`GeoError::InvalidCountry`] if validation fails.
    pub fn new(s: String) -> Result<Self, GeoError> {
        validate_country(&s)
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

    /// Return the English short name for this country.
    ///
    /// Returns `"Unknown"` if the code is syntactically valid but not in the known mapping.
    #[must_use]
    pub fn country_name(&self) -> &str {
        match self.0.as_str() {
            "AD" => "Andorra",
            "AE" => "United Arab Emirates",
            "AF" => "Afghanistan",
            "AG" => "Antigua and Barbuda",
            "AI" => "Anguilla",
            "AL" => "Albania",
            "AM" => "Armenia",
            "AO" => "Angola",
            "AQ" => "Antarctica",
            "AR" => "Argentina",
            "AS" => "American Samoa",
            "AT" => "Austria",
            "AU" => "Australia",
            "AW" => "Aruba",
            "AX" => "Åland Islands",
            "AZ" => "Azerbaijan",
            "BA" => "Bosnia and Herzegovina",
            "BB" => "Barbados",
            "BD" => "Bangladesh",
            "BE" => "Belgium",
            "BF" => "Burkina Faso",
            "BG" => "Bulgaria",
            "BH" => "Bahrain",
            "BI" => "Burundi",
            "BJ" => "Benin",
            "BM" => "Bermuda",
            "BN" => "Brunei",
            "BO" => "Bolivia",
            "BQ" => "Bonaire, Sint Eustatius and Saba",
            "BR" => "Brazil",
            "BS" => "Bahamas",
            "BT" => "Bhutan",
            "BV" => "Bouvet Island",
            "BW" => "Botswana",
            "BY" => "Belarus",
            "BZ" => "Belize",
            "CA" => "Canada",
            "CC" => "Cocos (Keeling) Islands",
            "CD" => "Congo, Democratic Republic of the",
            "CF" => "Central African Republic",
            "CG" => "Congo",
            "CH" => "Switzerland",
            "CI" => "Côte d'Ivoire",
            "CK" => "Cook Islands",
            "CL" => "Chile",
            "CM" => "Cameroon",
            "CN" => "China",
            "CO" => "Colombia",
            "CR" => "Costa Rica",
            "CU" => "Cuba",
            "CV" => "Cabo Verde",
            "CW" => "Curaçao",
            "CX" => "Christmas Island",
            "CY" => "Cyprus",
            "CZ" => "Czechia",
            "DE" => "Germany",
            "DJ" => "Djibouti",
            "DK" => "Denmark",
            "DM" => "Dominica",
            "DO" => "Dominican Republic",
            "DZ" => "Algeria",
            "EC" => "Ecuador",
            "EE" => "Estonia",
            "EG" => "Egypt",
            "EH" => "Western Sahara",
            "ER" => "Eritrea",
            "ES" => "Spain",
            "ET" => "Ethiopia",
            "FI" => "Finland",
            "FJ" => "Fiji",
            "FK" => "Falkland Islands (Malvinas)",
            "FM" => "Micronesia (Federated States of)",
            "FO" => "Faroe Islands",
            "FR" => "France",
            "GA" => "Gabon",
            "GB" => "United Kingdom",
            "GD" => "Grenada",
            "GE" => "Georgia",
            "GF" => "French Guiana",
            "GG" => "Guernsey",
            "GH" => "Ghana",
            "GI" => "Gibraltar",
            "GL" => "Greenland",
            "GM" => "Gambia",
            "GN" => "Guinea",
            "GP" => "Guadeloupe",
            "GQ" => "Equatorial Guinea",
            "GR" => "Greece",
            "GS" => "South Georgia and the South Sandwich Islands",
            "GT" => "Guatemala",
            "GU" => "Guam",
            "GW" => "Guinea-Bissau",
            "GY" => "Guyana",
            "HK" => "Hong Kong",
            "HM" => "Heard Island and McDonald Islands",
            "HN" => "Honduras",
            "HR" => "Croatia",
            "HT" => "Haiti",
            "HU" => "Hungary",
            "ID" => "Indonesia",
            "IE" => "Ireland",
            "IL" => "Israel",
            "IM" => "Isle of Man",
            "IN" => "India",
            "IO" => "British Indian Ocean Territory",
            "IQ" => "Iraq",
            "IR" => "Iran",
            "IS" => "Iceland",
            "IT" => "Italy",
            "JE" => "Jersey",
            "JM" => "Jamaica",
            "JO" => "Jordan",
            "JP" => "Japan",
            "KE" => "Kenya",
            "KG" => "Kyrgyzstan",
            "KH" => "Cambodia",
            "KI" => "Kiribati",
            "KM" => "Comoros",
            "KN" => "Saint Kitts and Nevis",
            "KP" => "Korea (Democratic People's Republic of)",
            "KR" => "Korea, Republic of",
            "KW" => "Kuwait",
            "KY" => "Cayman Islands",
            "KZ" => "Kazakhstan",
            "LA" => "Lao People's Democratic Republic",
            "LB" => "Lebanon",
            "LC" => "Saint Lucia",
            "LI" => "Liechtenstein",
            "LK" => "Sri Lanka",
            "LR" => "Liberia",
            "LS" => "Lesotho",
            "LT" => "Lithuania",
            "LU" => "Luxembourg",
            "LV" => "Latvia",
            "LY" => "Libya",
            "MA" => "Morocco",
            "MC" => "Monaco",
            "MD" => "Moldova",
            "ME" => "Montenegro",
            "MF" => "Saint Martin (French part)",
            "MG" => "Madagascar",
            "MH" => "Marshall Islands",
            "MK" => "North Macedonia",
            "ML" => "Mali",
            "MM" => "Myanmar",
            "MN" => "Mongolia",
            "MO" => "Macao",
            "MP" => "Northern Mariana Islands",
            "MQ" => "Martinique",
            "MR" => "Mauritania",
            "MS" => "Montserrat",
            "MT" => "Malta",
            "MU" => "Mauritius",
            "MV" => "Maldives",
            "MW" => "Malawi",
            "MX" => "Mexico",
            "MY" => "Malaysia",
            "MZ" => "Mozambique",
            "NA" => "Namibia",
            "NC" => "New Caledonia",
            "NE" => "Niger",
            "NF" => "Norfolk Island",
            "NG" => "Nigeria",
            "NI" => "Nicaragua",
            "NL" => "Netherlands",
            "NO" => "Norway",
            "NP" => "Nepal",
            "NR" => "Nauru",
            "NU" => "Niue",
            "NZ" => "New Zealand",
            "OM" => "Oman",
            "PA" => "Panama",
            "PE" => "Peru",
            "PF" => "French Polynesia",
            "PG" => "Papua New Guinea",
            "PH" => "Philippines",
            "PK" => "Pakistan",
            "PL" => "Poland",
            "PM" => "Saint Pierre and Miquelon",
            "PN" => "Pitcairn",
            "PR" => "Puerto Rico",
            "PS" => "Palestine, State of",
            "PT" => "Portugal",
            "PW" => "Palau",
            "PY" => "Paraguay",
            "QA" => "Qatar",
            "RE" => "Réunion",
            "RO" => "Romania",
            "RS" => "Serbia",
            "RU" => "Russian Federation",
            "RW" => "Rwanda",
            "SA" => "Saudi Arabia",
            "SB" => "Solomon Islands",
            "SC" => "Seychelles",
            "SD" => "Sudan",
            "SE" => "Sweden",
            "SG" => "Singapore",
            "SH" => "Saint Helena, Ascension and Tristan da Cunha",
            "SI" => "Slovenia",
            "SJ" => "Svalbard and Jan Mayen",
            "SK" => "Slovakia",
            "SL" => "Sierra Leone",
            "SM" => "San Marino",
            "SN" => "Senegal",
            "SO" => "Somalia",
            "SR" => "Suriname",
            "SS" => "South Sudan",
            "ST" => "Sao Tome and Principe",
            "SV" => "El Salvador",
            "SX" => "Sint Maarten (Dutch part)",
            "SY" => "Syrian Arab Republic",
            "SZ" => "Eswatini",
            "TC" => "Turks and Caicos Islands",
            "TD" => "Chad",
            "TF" => "French Southern Territories",
            "TG" => "Togo",
            "TH" => "Thailand",
            "TJ" => "Tajikistan",
            "TK" => "Tokelau",
            "TL" => "Timor-Leste",
            "TM" => "Turkmenistan",
            "TN" => "Tunisia",
            "TO" => "Tonga",
            "TR" => "Turkey",
            "TT" => "Trinidad and Tobago",
            "TV" => "Tuvalu",
            "TW" => "Taiwan",
            "TZ" => "Tanzania",
            "UA" => "Ukraine",
            "UG" => "Uganda",
            "UM" => "United States Minor Outlying Islands",
            "US" => "United States of America",
            "UY" => "Uruguay",
            "UZ" => "Uzbekistan",
            "VA" => "Holy See",
            "VC" => "Saint Vincent and the Grenadines",
            "VE" => "Venezuela",
            "VG" => "Virgin Islands (British)",
            "VI" => "Virgin Islands (U.S.)",
            "VN" => "Viet Nam",
            "VU" => "Vanuatu",
            "WF" => "Wallis and Futuna",
            "WS" => "Samoa",
            "YE" => "Yemen",
            "YT" => "Mayotte",
            "ZA" => "South Africa",
            "ZM" => "Zambia",
            "ZW" => "Zimbabwe",
            _ => "Unknown",
        }
    }
}

fn validate_country(input: &str) -> Result<CountryCode, GeoError> {
    if input.is_empty() {
        return Err(GeoError::InvalidCountry("country code is empty".to_string()));
    }
    if input.contains('\r') || input.contains('\n') || input.contains(' ') || input.contains('\t') {
        return Err(GeoError::InvalidCountry(
            "country code must not contain whitespace or control".to_string(),
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
                    let init = match regex::Regex::new(r"^[A-Z]{2}$") {
                        Ok(r) => r,
                        Err(_) => {
                            return Err(GeoError::InvalidCountry("internal regex error".to_string()))
                        }
                    };
                    let _ = RE.set(init);
                    match RE.get() {
                        Some(r) => r,
                        None => return Err(GeoError::InvalidCountry("internal regex error".to_string())),
                    }
                }
            };
            if !re.is_match(input) {
                return Err(GeoError::InvalidCountry(alloc::format!(
                    "country code '{}' must match ISO 3166-1 alpha-2 ^[A-Z]{{2}}$",
                    input
                )));
            }
        }
        #[cfg(not(feature = "std"))]
        {
            let re = match regex::Regex::new(r"^[A-Z]{2}$") {
                Ok(r) => r,
                Err(_) => return Err(GeoError::InvalidCountry("internal regex error".to_string())),
            };
            if !re.is_match(input) {
                return Err(GeoError::InvalidCountry(alloc::format!(
                    "country code '{}' must match ISO 3166-1 alpha-2 ^[A-Z]{{2}}$",
                    input
                )));
            }
        }
        Ok(CountryCode(input.to_string()))
    }

    #[cfg(not(feature = "regex"))]
    {
        if input.len() != 2 {
            return Err(GeoError::InvalidCountry(alloc::format!(
                "country code '{}' must be exactly 2 characters",
                input
            )));
        }
        for ch in input.chars() {
            if !ch.is_ascii_uppercase() {
                return Err(GeoError::InvalidCountry(alloc::format!(
                    "country code '{}' must be 2 uppercase A-Z letters",
                    input
                )));
            }
        }
        Ok(CountryCode(input.to_string()))
    }
}

/// Returns `true` if `s` is a valid ISO 3166-1 alpha-2 country code.
#[must_use]
pub fn is_valid_country_code(s: &str) -> bool {
    validate_country(s).is_ok()
}

impl Deref for CountryCode {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for CountryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for CountryCode {
    type Error = GeoError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        CountryCode::new(value)
    }
}

impl TryFrom<&str> for CountryCode {
    type Error = GeoError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        CountryCode::parse(value)
    }
}

impl FromStr for CountryCode {
    type Err = GeoError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CountryCode::parse(s)
    }
}

impl AsRef<str> for CountryCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_codes() {
        assert!(CountryCode::parse("GB").is_ok());
        assert_eq!(CountryCode::parse("GB").unwrap().country_name(), "United Kingdom");
        assert!(CountryCode::parse("US").is_ok());
        assert_eq!(CountryCode::parse("US").unwrap().country_name(), "United States of America");
        assert!(CountryCode::parse("DE").is_ok());
        assert_eq!(CountryCode::parse("FR").unwrap().country_name(), "France");
        assert!(CountryCode::parse("JP").is_ok());
        assert!(CountryCode::parse("ZZ").is_ok()); // syntactically valid, unknown name
        assert_eq!(CountryCode::parse("ZZ").unwrap().country_name(), "Unknown");
    }

    #[test]
    fn invalid_lowercase() {
        assert!(CountryCode::parse("gb").is_err());
        assert!(CountryCode::parse("Gb").is_err());
    }

    #[test]
    fn invalid_length() {
        assert!(CountryCode::parse("").is_err());
        assert!(CountryCode::parse("G").is_err());
        assert!(CountryCode::parse("GBR").is_err());
        assert!(CountryCode::parse("USA").is_err());
    }

    #[test]
    fn invalid_chars() {
        assert!(CountryCode::parse("12").is_err());
        assert!(CountryCode::parse("G1").is_err());
        assert!(CountryCode::parse("G ").is_err());
        assert!(CountryCode::parse("G\n").is_err());
    }

    #[test]
    fn is_valid_helper() {
        assert!(is_valid_country_code("GB"));
        assert!(!is_valid_country_code("gb"));
    }
}
