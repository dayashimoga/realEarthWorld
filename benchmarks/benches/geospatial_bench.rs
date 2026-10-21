use criterion::{black_box, criterion_group, criterion_main, Criterion};
use realearth_geospatial::transforms::{ecef_to_wgs84, wgs84_to_ecef, EnuConverter};
use realearth_geospatial::utm::wgs84_to_utm;
use realearth_shared::coordinates::Wgs84Coord;

fn bench_geospatial_transforms(c: &mut Criterion) {
    let vidhana_soudha = Wgs84Coord::new(12.9797, 77.5907, 920.0);
    let ecef = wgs84_to_ecef(vidhana_soudha);
    let enu_converter = EnuConverter::new(vidhana_soudha);

    let mut group = c.benchmark_group("geospatial_transforms");

    group.bench_function("wgs84_to_ecef", |b| {
        b.iter(|| {
            let _ = wgs84_to_ecef(black_box(vidhana_soudha));
        });
    });

    group.bench_function("ecef_to_wgs84_bowring", |b| {
        b.iter(|| {
            let _ = ecef_to_wgs84(black_box(ecef));
        });
    });

    group.bench_function("wgs84_to_local_enu", |b| {
        let test_point = Wgs84Coord::new(12.9800, 77.5920, 922.0);
        b.iter(|| {
            let _ = enu_converter.wgs84_to_enu(black_box(test_point));
        });
    });

    group.bench_function("wgs84_to_utm_zone43n", |b| {
        b.iter(|| {
            let _ = wgs84_to_utm(black_box(vidhana_soudha));
        });
    });

    let transamerica = Wgs84Coord::new(37.7952, -122.4028, 10.0);
    group.bench_function("wgs84_to_utm_zone10n_sf", |b| {
        b.iter(|| {
            let _ = wgs84_to_utm(black_box(transamerica));
        });
    });

    let navd88 = realearth_geospatial::datum::VerticalDatumMetadata::san_francisco_navd88();
    group.bench_function("ellipsoidal_to_navd88_sf", |b| {
        b.iter(|| {
            let _ = navd88.ellipsoidal_to_orthometric(black_box(10.0));
        });
    });

    group.finish();
}

criterion_group!(benches, bench_geospatial_transforms);
criterion_main!(benches);
