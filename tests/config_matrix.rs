//! Config-knob behavior matrix for geo-kit's `AddressBuilder`.
//!
//! Each builder knob must observably change the built `Address`:
//! optional knobs default to `None` and become `Some` when set;
//! required knobs fail the build when omitted and appear in the output
//! when set. A knob with no observable effect is a bug.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use geo_kit::{AddressBuilder, Coords, CountryCode, UkPostcode};

fn uk(s: &str) -> UkPostcode {
    UkPostcode::parse(s).expect("valid uk postcode")
}
fn cc(s: &str) -> CountryCode {
    CountryCode::parse(s).expect("valid country")
}

/// Minimal valid baseline: every required knob set, no optional knobs.
fn baseline() -> AddressBuilder {
    AddressBuilder::new()
        .line1("10 Downing St")
        .city("London")
        .postcode(uk("SW1A 2AA"))
        .country(cc("GB"))
}

// --- Required knobs: omitted -> error, set -> observable ------------------

#[test]
fn required_knobs_omitted_fail_build() {
    let cases: &[(&str, AddressBuilder)] = &[
        (
            "line1",
            AddressBuilder::new()
                .city("London")
                .postcode(uk("SW1A 2AA"))
                .country(cc("GB")),
        ),
        (
            "city",
            AddressBuilder::new()
                .line1("1 A Rd")
                .postcode(uk("SW1A 2AA"))
                .country(cc("GB")),
        ),
        (
            "postcode",
            AddressBuilder::new()
                .line1("1 A Rd")
                .city("London")
                .country(cc("GB")),
        ),
        (
            "country",
            AddressBuilder::new()
                .line1("1 A Rd")
                .city("London")
                .postcode(uk("SW1A 2AA")),
        ),
    ];
    for (knob, builder) in cases {
        let err = builder.clone().build().expect_err(knob);
        assert!(
            err.to_string().contains(knob),
            "omitting `{knob}` must name it in the error, got: {err}"
        );
    }
}

#[test]
fn required_knobs_set_are_observable() {
    let a = baseline().build().unwrap();
    assert_eq!(a.line1(), "10 Downing St");
    assert_eq!(a.city(), "London");
    assert_eq!(a.postcode().as_str(), "SW1A 2AA");
    assert_eq!(a.country().as_str(), "GB");

    // Changing a required knob changes the output.
    let b = baseline()
        .line1("221B Baker St")
        .city("Manchester")
        .postcode(uk("NW1 6XE"))
        .country(cc("US"))
        .build()
        .unwrap();
    assert_ne!(a.line1(), b.line1());
    assert_ne!(a.city(), b.city());
    assert_ne!(a.postcode().as_str(), b.postcode().as_str());
}

// --- Optional knobs: default None vs set Some ------------------------------

#[test]
fn knob_line2_default_none_vs_set() {
    let without = baseline().build().unwrap();
    assert_eq!(without.line2, None, "line2 must default to None");

    let with = baseline().line2("Flat B").build().unwrap();
    assert_eq!(with.line2.as_deref(), Some("Flat B"));
}

#[test]
fn knob_county_default_none_vs_set() {
    let without = baseline().build().unwrap();
    assert_eq!(without.county, None, "county must default to None");

    let with = baseline().county("Greater London").build().unwrap();
    assert_eq!(with.county.as_deref(), Some("Greater London"));
}

#[test]
fn knob_coords_default_none_vs_set() {
    let without = baseline().build().unwrap();
    assert_eq!(without.coords, None, "coords must default to None");

    let coords = Coords::new(51.5014, -0.1419).expect("valid coords");
    let with = baseline().coords(coords).build().unwrap();
    assert_eq!(with.coords, Some(coords));
    assert!(
        (with.coords.expect("some").lat() - 51.5014).abs() < 1e-9,
        "coords knob must carry the exact value through the builder"
    );
}

// --- Optional-but-validated: empty optional input is rejected -------------

#[test]
fn optional_knobs_reject_whitespace_only() {
    assert!(baseline().line2("   ").build().is_err());
    assert!(baseline().county("  ").build().is_err());
}

// --- Required knobs are trimmed, not passed through raw -------------------

#[test]
fn required_knob_values_are_trimmed() {
    let a = baseline().line1("  10 Downing St  ").build().unwrap();
    assert_eq!(a.line1(), "10 Downing St");
    let b = baseline().city("  London ").build().unwrap();
    assert_eq!(b.city(), "London");
}
