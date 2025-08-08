use std::collections::HashMap;

use serde::{Serialize};

use crate::types::{Value, Dim};
use crate::units::lookup_unit;

#[derive(Debug, Clone, Serialize)]
pub struct FunctionMeta {
    pub name: String,
    pub docs: String,
    #[serde(skip)]
    pub func: fn(&[Value]) -> anyhow::Result<Value>,
}

#[derive(Default)]
pub struct FunctionRegistry {
    map: HashMap<String, FunctionMeta>,
}

impl FunctionRegistry {
    pub fn new() -> Self { Self { map: HashMap::new() } }
    pub fn register(&mut self, name: &str, docs: &str, func: fn(&[Value]) -> anyhow::Result<Value>) {
        self.map.insert(name.to_string(), FunctionMeta { name: name.to_string(), docs: docs.to_string(), func });
    }
    pub fn get(&self, name: &str) -> Option<&FunctionMeta> { self.map.get(name) }
}

pub fn default_registry() -> FunctionRegistry {
    let mut r = FunctionRegistry::new();
    r.register("pow", "Power function pow(x, y)", |args| {
        if let [Value::Number(x), Value::Number(y)] = args { Ok(Value::Number(x.powf(*y))) } else { anyhow::bail!("pow expects two numbers") }
    });
    r.register("pi", "Constant pi", |_| Ok(Value::Number(std::f64::consts::PI)));
    r.register("i", "Imaginary unit i", |_| Ok(Value::Complex(num_complex::Complex64::new(0.0, 1.0))));
    // Trig (expects radians)
    fn angle_to_rad(arg: &Value) -> anyhow::Result<f64> {
        match arg {
            Value::Number(x) => Ok(*x),
            Value::Quantity { value, dim, unit } => {
                // Require dimensionless; use unit scale to radians if available
                if dim.exponents == [0;7] {
                    if let Some(u) = lookup_unit(unit) { Ok(value * u.scale) } else { Ok(*value) }
                } else { anyhow::bail!("angle must be dimensionless or angle unit") }
            }
            _ => anyhow::bail!("expected number or angle quantity"),
        }
    }
    r.register("sin", "Sine (rad)", |args| { if let [a] = args { Ok(Value::Number(angle_to_rad(a)?.sin())) } else { anyhow::bail!("sin expects 1 arg") } });
    r.register("cos", "Cosine (rad)", |args| { if let [a] = args { Ok(Value::Number(angle_to_rad(a)?.cos())) } else { anyhow::bail!("cos expects 1 arg") } });
    r.register("tan", "Tangent (rad)", |args| { if let [a] = args { Ok(Value::Number(angle_to_rad(a)?.tan())) } else { anyhow::bail!("tan expects 1 arg") } });
    r.register("asin", "Arcsine (rad)", |args| { if let [Value::Number(x)] = args { Ok(Value::Number(x.asin())) } else { anyhow::bail!("asin expects number") } });
    r.register("acos", "Arccos (rad)", |args| { if let [Value::Number(x)] = args { Ok(Value::Number(x.acos())) } else { anyhow::bail!("acos expects number") } });
    r.register("atan", "Arctan (rad)", |args| { if let [Value::Number(x)] = args { Ok(Value::Number(x.atan())) } else { anyhow::bail!("atan expects number") } });
    // Degree variants: sin_deg(x) where x is in degrees
    r.register("sin_deg", "Sine of degrees", |args| { if let [Value::Number(x)] = args { Ok(Value::Number((x.to_radians()).sin())) } else { anyhow::bail!("sin_deg expects number") } });
    r.register("cos_deg", "Cosine of degrees", |args| { if let [Value::Number(x)] = args { Ok(Value::Number((x.to_radians()).cos())) } else { anyhow::bail!("cos_deg expects number") } });
    r.register("tan_deg", "Tangent of degrees", |args| { if let [Value::Number(x)] = args { Ok(Value::Number((x.to_radians()).tan())) } else { anyhow::bail!("tan_deg expects number") } });
    r
}

