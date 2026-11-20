# Pattern adoption

Geographic primitives: postcodes, countries, coordinates, addresses.

| Pattern | Status | Evidence |
|---|---|---|
| T1 | adopted | src/postcode.rs:261 (TryFrom<String> for UkPostcode; the parse-at-boundary convention is stated in src/lib.rs:51 for all postcode and country types) |
| P64 | adopted | tests/proptest.rs:74 (proptest suites over the geo types) |
