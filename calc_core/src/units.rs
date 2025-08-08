use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::types::Dim;

#[derive(Debug, Clone)]
pub struct UnitInfo {
    pub name: &'static str,
    pub dim: Dim,
    pub scale: f64, // to canonical base unit
}

pub static UNIT_REGISTRY: Lazy<HashMap<&'static str, UnitInfo>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // Base units: m (L), kg (M), s (T), A (I), K (Θ), mol (N), cd (J)
    fn d(l:i8,m_:i8,t:i8,i:i8,th:i8,n:i8,j:i8)->Dim{ Dim{exponents:[l,m_,t,i,th,n,j]} }
    m.insert("m", UnitInfo{ name:"m", dim: d(1,0,0,0,0,0,0), scale: 1.0 });
    m.insert("kg", UnitInfo{ name:"kg", dim: d(0,1,0,0,0,0,0), scale: 1.0 });
    m.insert("s", UnitInfo{ name:"s", dim: d(0,0,1,0,0,0,0), scale: 1.0 });
    m.insert("A", UnitInfo{ name:"A", dim: d(0,0,0,1,0,0,0), scale: 1.0 });
    m.insert("K", UnitInfo{ name:"K", dim: d(0,0,0,0,1,0,0), scale: 1.0 });
    m.insert("mol", UnitInfo{ name:"mol", dim: d(0,0,0,0,0,1,0), scale: 1.0 });
    m.insert("cd", UnitInfo{ name:"cd", dim: d(0,0,0,0,0,0,1), scale: 1.0 });
    // Derived: V, Ω, F, H, S, Hz
    // For simplicity, treat them as canonical names with dimensions:
    // V = kg·m^2·s^-3·A^-1; Ω = kg·m^2·s^-3·A^-2; A: current
    m.insert("V", UnitInfo{ name:"V", dim: d(2,1,-3,-1,0,0,0), scale: 1.0 });
    m.insert("Ω", UnitInfo{ name:"Ω", dim: d(2,1,-3,-2,0,0,0), scale: 1.0 });
    m.insert("ohm", UnitInfo{ name:"Ω", dim: d(2,1,-3,-2,0,0,0), scale: 1.0 });
    m.insert("A", UnitInfo{ name:"A", dim: d(0,0,0,1,0,0,0), scale: 1.0 });
    m.insert("F", UnitInfo{ name:"F", dim: d(-2,-1,4,2,0,0,0), scale: 1.0 });
    m.insert("H", UnitInfo{ name:"H", dim: d(2,1,-2,-2,0,0,0), scale: 1.0 });
    m.insert("S", UnitInfo{ name:"S", dim: d(-2,-1,3,2,0,0,0), scale: 1.0 });
    m.insert("Hz", UnitInfo{ name:"Hz", dim: d(0,0,-1,0,0,0,0), scale: 1.0 });
     // Angles (dimensionless): rad canonical, deg and ° map to rad with PI/180
    m.insert("rad", UnitInfo{ name:"rad", dim: d(0,0,0,0,0,0,0), scale: 1.0 });
    // For conversion target readability, keep name as "deg"/"°" and use dim zero with scale to rad
    m.insert("deg", UnitInfo{ name:"deg", dim: d(0,0,0,0,0,0,0), scale: std::f64::consts::PI/180.0 });
    m.insert("°", UnitInfo{ name:"°", dim: d(0,0,0,0,0,0,0), scale: std::f64::consts::PI/180.0 });
    // Imperial length examples
    m.insert("in", UnitInfo{ name:"in", dim: d(1,0,0,0,0,0,0), scale: 0.0254 });
    m.insert("ft", UnitInfo{ name:"ft", dim: d(1,0,0,0,0,0,0), scale: 0.3048 });
    m.insert("cm", UnitInfo{ name:"m", dim: d(1,0,0,0,0,0,0), scale: 0.01 });
    m.insert("mm", UnitInfo{ name:"m", dim: d(1,0,0,0,0,0,0), scale: 0.001 });
    m.insert("km", UnitInfo{ name:"m", dim: d(1,0,0,0,0,0,0), scale: 1000.0 });
    m.insert("yd", UnitInfo{ name:"yd", dim: d(1,0,0,0,0,0,0), scale: 0.9144 });
    m.insert("mi", UnitInfo{ name:"mi", dim: d(1,0,0,0,0,0,0), scale: 1609.344 });
    // time
    m.insert("ms", UnitInfo{ name:"s", dim: d(0,0,1,0,0,0,0), scale: 1e-3 });
    m.insert("min", UnitInfo{ name:"s", dim: d(0,0,1,0,0,0,0), scale: 60.0 });
    m.insert("hr", UnitInfo{ name:"s", dim: d(0,0,1,0,0,0,0), scale: 3600.0 });
    // mass
    m.insert("g", UnitInfo{ name:"kg", dim: d(0,1,0,0,0,0,0), scale: 1e-3 });
    m.insert("lb", UnitInfo{ name:"lb", dim: d(0,1,0,0,0,0,0), scale: 0.453_592_37 });
    m.insert("oz", UnitInfo{ name:"oz", dim: d(0,1,0,0,0,0,0), scale: 0.028_349_523_125 });
    // Metric prefixes example: k, m (milli), u (micro), μ
    // We will handle scaling externally when parsing quantities.
    m
});

pub fn lookup_unit(name: &str) -> Option<&'static UnitInfo> {
    UNIT_REGISTRY.get(name).map(|u| u)
}

pub fn metric_scale(prefix: &str) -> Option<f64> {
    Some(match prefix {
        "Y" => 1e24,
        "Z" => 1e21,
        "E" => 1e18,
        "P" => 1e15,
        "T" => 1e12,
        "G" => 1e9,
        "M" => 1e6,
        "k" => 1e3,
        "h" => 1e2,
        "da" => 1e1,
        "d" => 1e-1,
        "c" => 1e-2,
        "m" => 1e-3,
        "u" => 1e-6,
        "μ" => 1e-6,
        "n" => 1e-9,
        "p" => 1e-12,
        "f" => 1e-15,
        "a" => 1e-18,
        "z" => 1e-21,
        "y" => 1e-24,
        _ => return None,
    })
}

pub fn resolve_prefixed_unit(token: &str) -> Option<(Dim, f64, &'static str)> {
    // Try to split token into prefix + base
    // Prefer longest prefixes like "da" before single-letter
    let prefixes = [
        "da", "Y", "Z", "E", "P", "T", "G", "M", "k", "h", "d", "c", "m", "u", "μ", "n", "p", "f", "a", "z", "y",
    ];
    if let Some(u) = lookup_unit(token) {
        return Some((u.dim.clone(), u.scale, u.name));
    }
    for &p in &prefixes {
        if token.starts_with(p) {
            let base = &token[p.len()..];
            if let Some(u) = lookup_unit(base) {
                if let Some(s) = metric_scale(p) {
                    return Some((u.dim.clone(), u.scale * s, u.name));
                }
            }
        }
    }
    None
}

pub fn canonical_unit_for_dim(dim: &Dim) -> Option<&'static str> {
    // Return a single-symbol canonical name if any unit has exactly this dimension and scale 1
    for (_k, u) in UNIT_REGISTRY.iter() {
        if &u.dim == dim && (u.scale - 1.0).abs() < 1e-12 {
            return Some(u.name);
        }
    }
    None
}

pub fn try_canonicalize(unit: &str, dim: &Dim) -> String {
    if let Some(canon) = canonical_unit_for_dim(dim) {
        canon.to_string()
    } else {
        unit.to_string()
    }
}

