# Requirements — geo-kit

Numbered, testable requirements. Every requirement maps to at least one named
test or doc-comment contract; security-relevant items cite THREAT-MODEL.md rows.

Scope: Geospatial primitives — haversine distance, bounding boxes, postcode parsing/validation

## Functional

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-GK-001 | Haversine distance matches reference values to documented tolerance (< 1 m at city scales) | MUST |
| REQ-GK-002 | Bounding-box containment is inclusive of edges; antimeridian handling documented | MUST |
| REQ-GK-003 | Postcode parsing validates format per region table; unknown regions error | MUST |

## Security

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-GK-100 | No panics on hostile coordinate input (NaN/inf return errors or `None`) | MUST |
| REQ-GK-101 | Parsing is allocation-bounded by input length | MUST |

## Observability & API hygiene

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-GK-900 | All fallible public APIs return typed errors; production `unwrap`/`expect` is denied or explicitly justified with an invariant comment | MUST |
| REQ-GK-901 | Public items carry doc comments with runnable examples where practical | SHOULD |

Reviewed: 2026-09-11
