# Changelog

All notable changes to this project are documented here. Format: [Keep a
Changelog](https://keepachangelog.com/) — versions follow [semver](https://semver.org).

## [Unreleased]

## [1.0.0] - 2026-09-05

### Added

- API declared stable; semver contract enforced via cargo-semver-checks CI gate.
- Typed newtypes for validated geo primitives: postcodes (incl. UK GIR 0AA
  special case), country codes, coordinates, addresses.
- Optional `serde` and `regex` integrations; `no_std`-compatible core.

## [0.1.1] - 2026-09-05

### Added
- Initial public release.
