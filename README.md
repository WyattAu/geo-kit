# geo-kit

Typed newtypes for validated geo primitives — postcodes, country codes, coordinates, addresses.

[![Crates.io](https://img.shields.io/crates/v/geo-kit.svg)](https://crates.io/crates/geo-kit)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](./LICENSE-MIT)

## Purpose

`geo-kit` is an L0 leaf crate providing validated newtypes for UK postcodes, US ZIP codes, ISO 3166-1 alpha-2 country codes, geographic coordinates, and postal addresses. Each type guarantees its invariants at construction time.

## Types

| Type | Validation |
|------|-----------|
| `UkPostcode` | UK postcode `^[A-Z]{1,2}[0-9][A-Z0-9]? [0-9][A-Z]{2}$`, space optional, uppercase normalized |
| `UsZipCode` | US ZIP `^\d{5}(-\d{4})?$` |
| `Postcode` | Generic `Uk | Us` enum |
| `CountryCode` | ISO 3166-1 alpha-2 `^[A-Z]{2}$`, with `country_name()` mapping |
| `Coords` | `lat` `-90..=90`, `lon` `-180..=180`, finite |
| `Address` | `line1`/`city` non-empty, `postcode`/`country` valid, optional `coords` |

## Features

- `std` (default) — enables `std` support
- `serde` — `Serialize`/`Deserialize` transparent for newtypes, derived for `Coords`/`Address`
- `regex` — regex-backed validation (more precise where applicable)
- `no_std` — `no_std` compatible (`extern crate alloc`)

No `unsafe` code (`#![forbid(unsafe_code)]`), `#![deny(missing_docs)]`.

## Usage

```rust
use geo_kit::{UkPostcode, CountryCode, Coords, Address};

let pc = UkPostcode::parse("SW1A 1AA").expect("valid");
assert_eq!(pc.as_str(), "SW1A 1AA");

// Space optional, lowercase normalized
let pc2 = UkPostcode::parse("sw1a1aa").expect("valid");
assert_eq!(pc2.as_str(), "SW1A 1AA");

let cc = CountryCode::parse("GB").expect("valid");
assert_eq!(cc.country_name(), "United Kingdom");

let coords = Coords::new(51.5074, -0.1278).expect("valid");
assert!(coords.lat > 51.0);

let addr = Address::new(
    "10 Downing St".to_string(),
    None,
    "London".to_string(),
    None,
    pc,
    cc,
).expect("valid");
assert_eq!(addr.city(), "London");

// All postcode/country types impl TryFrom<String>, FromStr, Display, Deref<Target=str>, AsRef<str>
let zip: geo_kit::UsZipCode = "90210".parse().expect("valid");
let generic: geo_kit::Postcode = "90210-1234".parse().expect("valid");
```

## No_Std

```toml
geo-kit = { version = "0.1", default-features = false }
```

## License

MIT OR Apache-2.0
