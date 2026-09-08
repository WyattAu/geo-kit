# Changelog

All notable changes to this project are documented here. Format: [Keep a
Changelog](https://keepachangelog.com/) — versions follow [semver](https://semver.org).

## [Unreleased]

## [1.1.0] - 2026-09-09

### Added

- International postcode support: `CaPostcode` (Canada `A1A 1A1`, space
  optional, uppercase normalized), `DePlz` (Germany, 5 digits),
  `FrCp` (France, 5 digits), `JpPostal` (Japan `NNN-NNNN`, hyphen optional,
  normalized), `AuPostcode` (Australia, 4 digits), and `InPin` (India,
  6 digits with non-zero first digit). Each implements `TryFrom`/
  `FromStr`/`Display`/`Deref`/`AsRef<str>`, optional transparent `serde`,
  and the same `regex`-feature pattern as the existing types (hand-rolled
  equivalent without the feature).
- `Postcode` enum extended with `Ca`, `De`, `Fr`, `Jp`, `Au`, and `In`
  variants; `Postcode::parse` tries each country format (documented order;
  5-digit strings resolve to US first, matching pre-1.1 behaviour).
- Roundtrip property tests for all new types, plus unit tests covering
  valid/invalid inputs and normalization per country.

### Fixed

- `UkPostcode::parse` panicked on multibyte input (e.g. `"ééé"`) because
  the normalizer split on a non-char boundary. Such input is now rejected
  with a validation error.

### Semver notes

- Adding `Postcode`/`GeoError`-adjacent enum variants is a breaking change
  for downstream exhaustive matches; new `Postcode` variants were appended
  after all existing variants so existing discriminant values are
  unchanged. The `Address`/`PostcodeChoice` API still models UK postcodes
  and is unchanged; international address support is on the roadmap.

## [1.0.0] - 2026-09-05

### Added

- API declared stable; semver contract enforced via cargo-semver-checks CI gate.
- Typed newtypes for validated geo primitives: postcodes (incl. UK GIR 0AA
  special case), country codes, coordinates, addresses.
- Optional `serde` and `regex` integrations; `no_std`-compatible core.

## [0.1.1] - 2026-09-05

### Added
- Initial public release.
