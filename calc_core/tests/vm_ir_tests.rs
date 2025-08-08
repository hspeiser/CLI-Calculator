use calc_core::ir::{lower_expr};
use calc_core::parser::parse_cell;
use calc_core::registry::default_registry;
use calc_core::vm::{eval_chunk, VmState};
use calc_core::types::Value;

#[test]
fn vm_add_mul_div_pow_parallel() {
    let e = parse_cell("(10 + 5) * 2 / 3");
    let chunk = lower_expr(&e.expr);
    let v = eval_chunk(&chunk, &default_registry(), &mut VmState::default()).unwrap();
    match v { Value::Number(n) => assert!((n - 10.0).abs() < 1e-12), _ => panic!() }

    let e2 = parse_cell("100 // 200");
    let chunk2 = lower_expr(&e2.expr);
    let v2 = eval_chunk(&chunk2, &default_registry(), &mut VmState::default()).unwrap();
    match v2 { Value::Number(n) => assert!((n - (100.0*200.0/(100.0+200.0))).abs() < 1e-12), _ => panic!() }
}

#[test]
fn vm_quantities_and_conversion() {
    let e = parse_cell("d = 10 m");
    let chunk = lower_expr(&e.expr);
    let mut st = VmState::default();
    let _ = eval_chunk(&chunk, &default_registry(), &mut st).unwrap();
    let e2 = parse_cell("d to in");
    let chunk2 = lower_expr(&e2.expr);
    let v = eval_chunk(&chunk2, &default_registry(), &mut st).unwrap();
    match v { Value::Quantity{ unit, .. } => assert_eq!(unit, "in"), _ => panic!() }
}


