use geo_kit::{
    AuPostcode, CaPostcode, Coords, CountryCode, DePlz, FrCp, InPin, JpPostal, Postcode,
    UkPostcode, UsZipCode,
};
use proptest::prelude::*;
use std::str::FromStr;

// Strategies -----------------------------------------------------------

fn arb_uk_postcode() -> impl Strategy<Value = String> {
    // Generate valid UK postcodes: [A-Z]{1,2}[0-9][A-Z0-9]? [0-9][A-Z]{2}
    // We'll generate via parts.
    let outward = prop_oneof![
        // 1 letter + digit
        ("[A-Z]", "[0-9]").prop_map(|(a, b): (String, String)| format!("{}{}", a, b)),
        // 2 letters + digit
        ("[A-Z][A-Z]", "[0-9]").prop_map(|(a, b): (String, String)| format!("{}{}", a, b)),
        // 1 letter + digit + letter
        ("[A-Z]", "[0-9]", "[A-Z0-9]")
            .prop_map(|(a, b, c): (String, String, String)| format!("{}{}{}", a, b, c)),
        // 2 letters + digit + alnum
        ("[A-Z][A-Z]", "[0-9]", "[A-Z0-9]")
            .prop_map(|(a, b, c): (String, String, String)| format!("{}{}{}", a, b, c)),
    ];
    let inward = ("[0-9]", "[A-Z][A-Z]").prop_map(|(a, b)| format!("{}{}", a, b));
    (outward, inward).prop_map(|(o, i)| format!("{} {}", o, i))
}

fn arb_us_zip() -> impl Strategy<Value = String> {
    prop_oneof![
        "[0-9]{5}",
        ("[0-9]{5}", "[0-9]{4}").prop_map(|(a, b)| format!("{}-{}", a, b)),
    ]
}

fn arb_country() -> impl Strategy<Value = String> {
    "[A-Z]{2}"
}

fn arb_ca_postcode() -> impl Strategy<Value = String> {
    ("[A-Z][0-9][A-Z]", "[0-9][A-Z][0-9]").prop_map(|(a, b): (String, String)| format!("{a} {b}"))
}

fn arb_de_plz() -> impl Strategy<Value = String> {
    "[0-9]{5}"
}

fn arb_fr_cp() -> impl Strategy<Value = String> {
    "[0-9]{5}"
}

fn arb_jp_postal() -> impl Strategy<Value = String> {
    ("[0-9]{3}", "[0-9]{4}").prop_map(|(a, b)| format!("{a}-{b}"))
}

fn arb_au_postcode() -> impl Strategy<Value = String> {
    "[0-9]{4}"
}

fn arb_in_pin() -> impl Strategy<Value = String> {
    "[1-9][0-9]{5}"
}

fn arb_coords() -> impl Strategy<Value = (f64, f64)> {
    // lat -90..90, lon -180..180
    (-90.0f64..=90.0f64, -180.0f64..=180.0f64)
}

// Tests ---------------------------------------------------------------

proptest! {
    #[test]
    fn uk_roundtrip(s in arb_uk_postcode()) {
        let parsed = UkPostcode::parse(&s).unwrap();
        let displayed = parsed.to_string();
        let reparsed = UkPostcode::from_str(&displayed).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
        // With regex, should still be valid
        prop_assert!(geo_kit::is_valid_uk_postcode(&s));
    }

    #[test]
    fn uk_normalises_lowercase(s in arb_uk_postcode()) {
        let lower = s.to_ascii_lowercase();
        let parsed = UkPostcode::parse(&lower).unwrap();
        prop_assert_eq!(parsed.as_str(), s);
    }

    #[test]
    fn us_roundtrip(s in arb_us_zip()) {
        let parsed = UsZipCode::parse(&s).unwrap();
        let displayed = parsed.to_string();
        let reparsed = UsZipCode::from_str(&displayed).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
    }

    #[test]
    fn country_roundtrip(s in arb_country()) {
        let parsed = CountryCode::parse(&s).unwrap();
        let displayed = parsed.to_string();
        let reparsed = CountryCode::from_str(&displayed).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
        prop_assert!(geo_kit::is_valid_country_code(&s));
    }

    #[test]
    fn coords_roundtrip(pair in arb_coords()) {
        let (lat, lon) = pair;
        // filter out NaN not needed because range generates finite
        let coords = Coords::new(lat, lon).unwrap();
        let displayed = coords.to_string();
        let reparsed = Coords::from_str(&displayed).unwrap();
        // Allow small floating error due to display precision? We compare with tolerance.
        prop_assert!((coords.lat - reparsed.lat).abs() < 1e-9);
        prop_assert!((coords.lon - reparsed.lon).abs() < 1e-9);
    }

    #[test]
    fn coords_valid_range(pair in arb_coords()) {
        let (lat, lon) = pair;
        prop_assert!(geo_kit::is_valid_coords(lat, lon));
    }

    #[test]
    fn ca_roundtrip(s in arb_ca_postcode()) {
        let parsed = CaPostcode::parse(&s).unwrap();
        let reparsed = CaPostcode::from_str(parsed.as_ref()).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
        prop_assert!(geo_kit::is_valid_ca_postcode(&s));
    }

    #[test]
    fn ca_normalises_unspaced(s in arb_ca_postcode()) {
        let compact: String = s.chars().filter(|c| *c != ' ').collect();
        let parsed = CaPostcode::parse(&compact).unwrap();
        prop_assert_eq!(parsed.as_str(), s);
    }

    #[test]
    fn de_roundtrip(s in arb_de_plz()) {
        let parsed = DePlz::parse(&s).unwrap();
        let reparsed = DePlz::from_str(parsed.as_ref()).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
        prop_assert!(geo_kit::is_valid_de_plz(&s));
    }

    #[test]
    fn fr_roundtrip(s in arb_fr_cp()) {
        let parsed = FrCp::parse(&s).unwrap();
        let reparsed = FrCp::from_str(parsed.as_ref()).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
        prop_assert!(geo_kit::is_valid_fr_cp(&s));
    }

    #[test]
    fn jp_roundtrip(s in arb_jp_postal()) {
        let parsed = JpPostal::parse(&s).unwrap();
        let reparsed = JpPostal::from_str(parsed.as_ref()).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
        prop_assert!(geo_kit::is_valid_jp_postal(&s));
    }

    #[test]
    fn jp_normalises_unhyphenated(s in arb_jp_postal()) {
        let compact: String = s.chars().filter(|c| *c != '-').collect();
        let parsed = JpPostal::parse(&compact).unwrap();
        prop_assert_eq!(parsed.as_str(), s);
    }

    #[test]
    fn au_roundtrip(s in arb_au_postcode()) {
        let parsed = AuPostcode::parse(&s).unwrap();
        let reparsed = AuPostcode::from_str(parsed.as_ref()).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
        prop_assert!(geo_kit::is_valid_au_postcode(&s));
    }

    #[test]
    fn in_roundtrip(s in arb_in_pin()) {
        let parsed = InPin::parse(&s).unwrap();
        let reparsed = InPin::from_str(parsed.as_ref()).unwrap();
        prop_assert_eq!(parsed.as_str(), reparsed.as_str());
        prop_assert!(geo_kit::is_valid_in_pin(&s));
    }

    #[test]
    fn postcode_generic_roundtrip(s in arb_jp_postal()) {
        let parsed = Postcode::parse(&s).unwrap();
        prop_assert!(matches!(parsed, Postcode::Jp(_)));
        let displayed = parsed.to_string();
        let reparsed = Postcode::from_str(&displayed).unwrap();
        prop_assert_eq!(displayed, reparsed.to_string());
    }
}

// Negative property tests ---------------------------------------------

proptest! {
    #[test]
    fn uk_rejects_lowercase_country_mismatch(s in "[a-z]{2} [0-9][A-Z]{2}") {
        // This pattern looks like postcode but lowercase outward should be accepted via normalization,
        // but we test that country codes reject lowercase.
        prop_assert!(CountryCode::parse(&s[..2]).is_err());
    }

    #[test]
    fn us_rejects_invalid_len(s in "[0-9]{4}") {
        prop_assert!(UsZipCode::parse(&s).is_err());
    }

    #[test]
    fn us_rejects_alpha(s in "[A-Z]{5}") {
        prop_assert!(UsZipCode::parse(&s).is_err());
    }

    #[test]
    fn country_rejects_lowercase(s in "[a-z]{2}") {
        prop_assert!(CountryCode::parse(&s).is_err());
    }

    #[test]
    fn country_rejects_too_long(s in "[A-Z]{3,5}") {
        prop_assert!(CountryCode::parse(&s).is_err());
    }

    #[test]
    fn coords_rejects_out_of_range_lat(lat in 91.0..1000.0f64, lon in -180.0..180.0f64) {
        prop_assert!(Coords::new(lat, lon).is_err());
        prop_assert!(Coords::new(-lat, lon).is_err());
    }

    #[test]
    fn coords_rejects_out_of_range_lon(lat in -90.0..90.0f64, lon in 181.0..1000.0f64) {
        prop_assert!(Coords::new(lat, lon).is_err());
        prop_assert!(Coords::new(lat, -lon).is_err());
    }
}
