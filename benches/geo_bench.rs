use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geo_kit::{Coords, CountryCode, UkPostcode, UsZipCode};

fn bench_uk(c: &mut Criterion) {
    c.bench_function("parse_uk_valid", |b| {
        b.iter(|| UkPostcode::parse(black_box("SW1A 1AA")).unwrap())
    });
    c.bench_function("parse_uk_without_space", |b| {
        b.iter(|| UkPostcode::parse(black_box("SW1A1AA")).unwrap())
    });
    c.bench_function("parse_uk_invalid", |b| {
        b.iter(|| UkPostcode::parse(black_box("INVALID")))
    });
    c.bench_function("is_valid_uk", |b| {
        b.iter(|| geo_kit::is_valid_uk_postcode(black_box("SW1A 1AA")))
    });
}

fn bench_us(c: &mut Criterion) {
    c.bench_function("parse_us_valid", |b| {
        b.iter(|| UsZipCode::parse(black_box("90210")).unwrap())
    });
    c.bench_function("parse_us_extended", |b| {
        b.iter(|| UsZipCode::parse(black_box("90210-1234")).unwrap())
    });
    c.bench_function("parse_us_invalid", |b| {
        b.iter(|| UsZipCode::parse(black_box("ABCDE")))
    });
}

fn bench_country(c: &mut Criterion) {
    c.bench_function("parse_country_valid", |b| {
        b.iter(|| CountryCode::parse(black_box("GB")).unwrap())
    });
    c.bench_function("parse_country_invalid", |b| {
        b.iter(|| CountryCode::parse(black_box("gb")))
    });
}

fn bench_coords(c: &mut Criterion) {
    c.bench_function("parse_coords_valid", |b| {
        b.iter(|| Coords::new(black_box(51.5074), black_box(-0.1278)).unwrap())
    });
    c.bench_function("parse_coords_invalid", |b| {
        b.iter(|| Coords::new(black_box(91.0), black_box(0.0)))
    });
    c.bench_function("coords_parse_str", |b| {
        b.iter(|| Coords::parse(black_box("51.5074,-0.1278")).unwrap())
    });
}

criterion_group!(benches, bench_uk, bench_us, bench_country, bench_coords);
criterion_main!(benches);
