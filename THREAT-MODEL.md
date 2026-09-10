# Threat Model — geo-kit

Reference: STRIDE. Scope: the crate's public API surface (typed newtypes
`UkPostcode`, `UsZipCode`, `Coords`, `Address`, `CountryCode`, and the
`is_valid_*` helpers). Trust boundaries: (1) strings entering `parse` /
`FromStr` / `TryFrom<String>`, (2) floats entering `Coords::new`, (3) the
dependency tree (regex-lite, serde).

This crate is an input-validation library — validation **is** its security
surface. It holds no secrets, performs no I/O, and exposes no network or
filesystem API.

## Assets

| ID | Asset | Example |
|----|-------|---------|
| A1 | Soundness of downstream logic relying on validated types | An out-of-range latitude propagating into a mapping/geo query |
| A2 | Robustness of the parser (no abort on hostile input) | Multibyte or malformed input panicking the caller |

## STRIDE Analysis

| # | Threat | Category | Surface | Mitigation | Verifying test |
|---|--------|----------|---------|------------|----------------|
| T1 | Panic on malformed/multibyte input | DoS | all `parse`/`FromStr` impls | `#![forbid(unsafe_code)]`; parsing is regex/range based and returns `GeoError`; property test asserts multibyte input is rejected without panicking | `uk_rejects_multibyte_input_without_panicking` (`tests/proptest.rs`), `uk_invalid_cases`, `us_invalid_cases` |
| T2 | Out-of-range coordinates accepted (NaN/∞/off-globe) | Tampering | `Coords::new`, `is_valid_coords` | Requires finite lat ∈ [-90, 90], lon ∈ [-180, 180]; `is_valid_coords` mirrors the same logic for callers avoiding newtypes | `coords_rejects_out_of_range_lat`, `coords_rejects_out_of_range_lon`, `coords_valid_range`, `parse_invalid`, `parse_comma` (`src/coords.rs`) |
| T3 | Country/region confusion (lowercase, wrong length) | Spoofing | `CountryCode::parse`, all postcodes | Strict patterns: ISO alpha-2 uppercase-only (lowercase rejected, not normalized), country-specific postcode regexes with normalization only where documented (UK/CA uppercase, JP hyphen) | `country_rejects_lowercase`, `country_rejects_too_long`, `us_rejects_alpha`, `us_rejects_invalid_len`, `invalid_uk`, `invalid_de` |
| T4 | Partially validated `Address` (empty lines, mismatched postcode/country) | Tampering | `Address::new`, `AddressBuilder` | Constructor validates non-empty lines and requires an already-validated postcode + country pair; builder rejects empty city/line1 | `invalid_empty_line1`, `invalid_empty_city`, `invalid_line2_empty`, `reexports_work` (valid path) |
| T5 | Deserialization bypass of validation (serde) | Elevation | serde transparent (de)serialization | Serde impls go **through** the same parse path (transparent), so a forged JSON string cannot produce an invalid instance — deserialization fails with `GeoError` | `display_parse_idempotent`, roundtrip properties `uk_roundtrip` … `us_roundtrip` (`tests/proptest.rs`) |

## Repudiation

Not applicable — the crate is stateless and keeps no records.

## Out of Scope

- **Syntactic vs. semantic validity:** `SW1A 1AA` being format-valid does
  not mean the postcode exists or is deliverable; `CountryCode::parse("GB")`
  checks the ISO pattern, not postal-service reality. Downstream address
  verification is a separate concern.
- Legitimate-value coverage: the postcode regexes accept some
  syntactically-valid-but-nonexistent codes and reject a few exotic real
  ones (overseas territories etc.); that is a correctness trade, not a
  security control.
- Caller-side logging of `Address` values (PII) — `Display` renders the
  full address by design.

## Residual Risks

- **R1 (Low, accepted):** Regex-based validation on adversarially long
  input allocates proportionally to input length; no length pre-cap exists.
  Bounded impact (pure allocation, `forbid(unsafe_code)`, error return), but
  callers streaming unbounded input should cap length first.
- **R2 (Low, accepted):** `Coords` accepts values like `(-90.0, 180.0)`
  that are technically in-range corners; geofencing logic downstream must
  treat the boundaries deliberately.
- **R3 (Low, accepted):** Dependency risk in the regex engine; no in-repo
  `cargo audit` gate (org-level Dependabot only).
