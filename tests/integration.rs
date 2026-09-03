use geo_kit::{Address, AddressBuilder, Coords, CountryCode, Postcode, UkPostcode, UsZipCode};
use std::str::FromStr;

fn roundtrip<T>(s: &str, parse: impl Fn(&str) -> Result<T, geo_kit::GeoError>) -> bool
where
    T: ToString + FromStr<Err = geo_kit::GeoError>,
    T: AsRef<str>,
{
    let parsed = match parse(s) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let displayed = parsed.to_string();
    match T::from_str(&displayed) {
        Ok(reparsed) => reparsed.as_ref() == parsed.as_ref(),
        Err(_) => false,
    }
}

// UkPostcode ---------------------------------------------------------

#[test]
fn uk_valid_cases() {
    let valid = [
        "SW1A 1AA",
        "SW1A1AA",
        "sw1a 1aa",
        "EC1A 1BB",
        "M1 1AE",
        "B33 8TH",
        "CR2 6XH",
        "DN55 1PT",
        "W1A 0AX",
        "NW1 6XE",
    ];
    for case in valid {
        assert!(
            UkPostcode::parse(case).is_ok(),
            "should be valid: {}",
            case
        );
        // After normalization, should be valid again.
        let normalized = UkPostcode::parse(case).unwrap().to_string();
        assert!(
            UkPostcode::parse(&normalized).is_ok(),
            "normalized should be valid: {} -> {}",
            case,
            normalized
        );
    }
}

#[test]
fn uk_valid_normalization() {
    let pc = UkPostcode::parse("sw1a1aa").expect("valid");
    assert_eq!(pc.as_str(), "SW1A 1AA");
    let pc2 = UkPostcode::parse("M1 1AE").expect("valid");
    assert_eq!(pc2.as_str(), "M1 1AE");
    let pc3 = UkPostcode::parse("m11ae").expect("valid");
    assert_eq!(pc3.as_str(), "M1 1AE");
}

#[test]
fn uk_invalid_cases() {
    let invalid = [
        "",
        "SW1A1A",
        "SW1A 1AAA",
        "12345",
        "SWA 1AA",
        "ZZZ 1AA",
        "SW1A  1AA ", // actually we normalize spaces so double space may be considered valid after normalization? Our normalize removes all spaces then reinserts, so this would be valid. So we keep alternative invalid.
        "SW1A 1A",
        "N0T A PC",
        "   ",
        "SW1A\t1AA",
    ];
    for case in invalid {
        // Note: "SW1A  1AA " will be normalized to "SW1A 1AA" and thus valid; skip it if our logic says valid.
        if case == "SW1A  1AA " {
            continue;
        }
        assert!(
            UkPostcode::parse(case).is_err(),
            "should be invalid: {:?}",
            case
        );
        assert!(!geo_kit::is_valid_uk_postcode(case));
    }
}

#[test]
fn uk_try_from_and_from_str() {
    let p: UkPostcode = "SW1A 1AA".parse().expect("from_str");
    assert_eq!(p.as_str(), "SW1A 1AA");
    let p2 = UkPostcode::try_from("SW1A 1AA".to_string()).expect("try_from String");
    assert_eq!(p2.as_str(), "SW1A 1AA");
    let p3 = UkPostcode::try_from("SW1A 1AA").expect("try_from &str");
    assert_eq!(p3.as_str(), "SW1A 1AA");
    assert!(roundtrip("SW1A 1AA", UkPostcode::parse));
}

#[test]
fn uk_serde_roundtrip() {
    #[cfg(feature = "serde")]
    {
        let pc = UkPostcode::parse("SW1A 1AA").expect("valid");
        let json = serde_json::to_string(&pc).expect("serialize");
        let de: UkPostcode = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(pc, de);
    }
}

// UsZipCode -----------------------------------------------------------

#[test]
fn us_valid_cases() {
    let valid = ["90210", "00501", "12345-6789", "99999-9999", "00000"];
    for case in valid {
        assert!(UsZipCode::parse(case).is_ok(), "should be valid: {}", case);
        assert!(roundtrip(case, UsZipCode::parse), "roundtrip failed: {}", case);
    }
}

#[test]
fn us_invalid_cases() {
    let invalid = [
        "",
        "9021",
        "902101",
        "9021A",
        "1234-5678",
        "12345-678",
        "12345-67890",
        " 90210",
        "90210 ",
        "12 345",
        "abcde",
        "12345--6789",
    ];
    for case in invalid {
        assert!(
            UsZipCode::parse(case).is_err(),
            "should be invalid: {:?}",
            case
        );
        assert!(!geo_kit::is_valid_us_zip(case));
    }
}

#[test]
fn us_try_from() {
    let z: UsZipCode = "90210".parse().expect("from_str");
    assert_eq!(z.as_str(), "90210");
    let z2 = UsZipCode::try_from("90210-1234".to_string()).expect("try_from");
    assert_eq!(z2.as_str(), "90210-1234");
}

// Postcode generic ----------------------------------------------------

#[test]
fn postcode_generic() {
    let uk: Postcode = "SW1A 1AA".parse().expect("uk postcode");
    assert!(matches!(uk, Postcode::Uk(_)));
    let us: Postcode = "90210".parse().expect("us postcode");
    assert!(matches!(us, Postcode::Us(_)));
    assert!(Postcode::parse("INVALID").is_err());
    assert_eq!(uk.to_string(), "SW1A 1AA");
}

// CountryCode ---------------------------------------------------------

#[test]
fn country_valid() {
    let valid = ["GB", "US", "DE", "FR", "JP", "CN", "BR", "ZZ"];
    for case in valid {
        assert!(CountryCode::parse(case).is_ok(), "should be valid: {}", case);
        assert!(roundtrip(case, CountryCode::parse), "roundtrip failed: {}", case);
    }
}

#[test]
fn country_name_mapping() {
    assert_eq!(CountryCode::parse("GB").unwrap().country_name(), "United Kingdom");
    assert_eq!(
        CountryCode::parse("US").unwrap().country_name(),
        "United States of America"
    );
    assert_eq!(CountryCode::parse("DE").unwrap().country_name(), "Germany");
    assert_eq!(CountryCode::parse("FR").unwrap().country_name(), "France");
    assert_eq!(CountryCode::parse("ZZ").unwrap().country_name(), "Unknown");
}

#[test]
fn country_invalid() {
    assert!(CountryCode::parse("").is_err());
    assert!(CountryCode::parse("gb").is_err());
    assert!(CountryCode::parse("G").is_err());
    assert!(CountryCode::parse("GBR").is_err());
    assert!(CountryCode::parse("G1").is_err());
    assert!(CountryCode::parse("12").is_err());
    assert!(CountryCode::parse("G ").is_err());
    assert!(!geo_kit::is_valid_country_code("gb"));
}

#[test]
fn country_try_from() {
    let c: CountryCode = "GB".parse().expect("from_str");
    assert_eq!(c.as_str(), "GB");
    let c2 = CountryCode::try_from("GB".to_string()).expect("try_from");
    assert_eq!(c2.as_str(), "GB");
}

// Coords --------------------------------------------------------------

#[test]
fn coords_valid() {
    let valid = [
        (51.5074, -0.1278),
        (0.0, 0.0),
        (90.0, 180.0),
        (-90.0, -180.0),
        (-33.8688, 151.2093),
    ];
    for (lat, lon) in valid {
        assert!(Coords::new(lat, lon).is_ok(), "should be valid: {},{}", lat, lon);
    }
}

#[test]
fn coords_invalid() {
    assert!(Coords::new(91.0, 0.0).is_err());
    assert!(Coords::new(-91.0, 0.0).is_err());
    assert!(Coords::new(0.0, 181.0).is_err());
    assert!(Coords::new(0.0, -181.0).is_err());
    assert!(Coords::new(f64::NAN, 0.0).is_err());
    assert!(Coords::new(0.0, f64::INFINITY).is_err());
    assert!(!geo_kit::is_valid_coords(91.0, 0.0));
}

#[test]
fn coords_parse() {
    let c = Coords::parse("51.5074,-0.1278").expect("valid");
    assert!((c.lat - 51.5074).abs() < 1e-9);
    let c2 = Coords::parse("51.5074 -0.1278").expect("valid");
    assert!((c2.lon - -0.1278).abs() < 1e-9);
    assert!(Coords::parse("91,0").is_err());
    assert!(Coords::parse("0,181").is_err());
    assert!(Coords::parse("").is_err());
}

#[test]
fn coords_try_from_tuple() {
    let c = Coords::try_from((51.5, -0.12)).expect("valid");
    assert_eq!(c.lat, 51.5);
}

#[test]
#[cfg(feature = "serde")]
fn coords_serde_roundtrip() {
    let c = Coords::new(51.5, -0.12).expect("valid");
    let json = serde_json::to_string(&c).expect("serialize");
    let de: Coords = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(c, de);
}

// Address -------------------------------------------------------------

#[test]
fn address_valid() {
    let addr = Address::new(
        "10 Downing St".to_string(),
        None,
        "London".to_string(),
        None,
        UkPostcode::parse("SW1A 2AA").expect("valid"),
        CountryCode::parse("GB").expect("valid"),
    )
    .expect("valid");
    assert_eq!(addr.line1(), "10 Downing St");
    assert_eq!(addr.city(), "London");
    assert_eq!(addr.postcode().as_str(), "SW1A 2AA");
    assert_eq!(addr.country().as_str(), "GB");
    assert_eq!(addr.to_string(), "10 Downing St, London, SW1A 2AA, GB");
}

#[test]
fn address_with_all_fields() {
    let addr = AddressBuilder::new()
        .line1("221B Baker St")
        .line2("Flat B")
        .city("London")
        .county("Greater London")
        .postcode(UkPostcode::parse("NW1 6XE").expect("valid"))
        .country(CountryCode::parse("GB").expect("valid"))
        .build()
        .expect("valid");
    assert_eq!(addr.line2.as_deref(), Some("Flat B"));
    assert_eq!(addr.county.as_deref(), Some("Greater London"));
}

#[test]
fn address_with_coords() {
    let coords = Coords::new(51.5, -0.12).expect("valid");
    let addr = Address::new_with_coords(
        "10 Downing St".to_string(),
        None,
        "London".to_string(),
        None,
        UkPostcode::parse("SW1A 2AA").expect("valid"),
        CountryCode::parse("GB").expect("valid"),
        Some(coords),
    )
    .expect("valid");
    assert_eq!(addr.coords, Some(coords));
}

#[test]
fn address_invalid_empty() {
    assert!(Address::new(
        "   ".to_string(),
        None,
        "London".to_string(),
        None,
        UkPostcode::parse("SW1A 2AA").expect("valid"),
        CountryCode::parse("GB").expect("valid"),
    )
    .is_err());
    assert!(Address::new(
        "10 Downing St".to_string(),
        None,
        "".to_string(),
        None,
        UkPostcode::parse("SW1A 2AA").expect("valid"),
        CountryCode::parse("GB").expect("valid"),
    )
    .is_err());
    assert!(Address::new(
        "10 Downing St".to_string(),
        Some("   ".to_string()),
        "London".to_string(),
        None,
        UkPostcode::parse("SW1A 2AA").expect("valid"),
        CountryCode::parse("GB").expect("valid"),
    )
    .is_err());
}

#[test]
fn address_builder_missing() {
    let res = AddressBuilder::new()
        .line1("10 Downing St")
        .city("London")
        .build();
    assert!(res.is_err());
}

#[test]
#[cfg(feature = "serde")]
fn address_serde_roundtrip() {
    let addr = Address::new(
        "10 Downing St".to_string(),
        Some("Second line".to_string()),
        "London".to_string(),
        None,
        UkPostcode::parse("SW1A 2AA").expect("valid"),
        CountryCode::parse("GB").expect("valid"),
    )
    .expect("valid");
    let json = serde_json::to_string(&addr).expect("serialize");
    let de: Address = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(addr, de);
}

#[test]
fn display_parse_idempotent() {
    let pc = UkPostcode::parse("SW1A 1AA").expect("valid");
    assert_eq!(pc.to_string(), "SW1A 1AA");
    let cc = CountryCode::parse("GB").expect("valid");
    assert_eq!(cc.to_string(), "GB");
    let coords = Coords::new(51.5, -0.12).expect("valid");
    let s = coords.to_string();
    let reparsed = Coords::parse(&s).expect("reparse");
    assert_eq!(coords, reparsed);
}
