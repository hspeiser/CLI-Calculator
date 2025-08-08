use calc_core::registry::default_registry;
use calc_core::types::Value;

#[test]
fn trig_and_degree_variants() {
    let r = default_registry();
    // sin(pi/2)
    let pi_over_2 = Value::Number(std::f64::consts::PI / 2.0);
    let v = (r.get("sin").unwrap().func)(&[pi_over_2]).unwrap();
    match v { Value::Number(n) => assert!((n - 1.0).abs() < 1e-12), _ => panic!() }

    // cos_deg(60)
    let v = (r.get("cos_deg").unwrap().func)(&[Value::Number(60.0)]).unwrap();
    match v { Value::Number(n) => assert!((n - 0.5).abs() < 1e-12), _ => panic!() }
}


