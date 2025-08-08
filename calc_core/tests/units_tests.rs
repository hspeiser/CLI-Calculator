use calc_core::units::{lookup_unit, resolve_prefixed_unit, canonical_unit_for_dim};

#[test]
fn lookup_basic_units() {
    let m = lookup_unit("m").unwrap();
    assert_eq!(m.name, "m");
    let ohm = lookup_unit("ohm").unwrap();
    assert_eq!(ohm.name, "Ω");
}

#[test]
fn prefixed_units() {
    let (_d, scale, name) = resolve_prefixed_unit("kΩ").unwrap();
    assert_eq!(name, "Ω");
    assert!((scale - 1000.0).abs() < 1e-12);
}

#[test]
fn angle_units_scale() {
    let deg = lookup_unit("deg").unwrap();
    let rad = lookup_unit("rad").unwrap();
    assert_eq!(rad.scale, 1.0);
    assert!((deg.scale - std::f64::consts::PI / 180.0).abs() < 1e-12);
}

#[test]
fn canonical_for_dimension() {
    let meter = lookup_unit("m").unwrap();
    let canon = canonical_unit_for_dim(&meter.dim).unwrap();
    assert_eq!(canon, "m");
}


