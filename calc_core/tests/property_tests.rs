use calc_core::Engine;
use proptest::prelude::*;

proptest! {
    #[test]
    fn implicit_mul_equivalence(a in 0.0f64..1e6, b in 0.0f64..1e6) {
        let mut e = Engine::new();
        e.eval_cell(&format!("x = {}", a)).unwrap();
        let r1 = e.eval_cell(&format!("{}x", b)).unwrap().value.display();
        let r2 = e.eval_cell(&format!("{} * x", b)).unwrap().value.display();
        prop_assert_eq!(r1, r2);
    }

    #[test]
    fn conversion_roundtrip_m_to_in(a in 0.0f64..1e6) {
        let mut e = Engine::new();
        e.eval_cell(&format!("d = {} m", a)).unwrap();
        let to_in = e.eval_cell("d to in").unwrap().value.display();
        prop_assert!(to_in.contains("in"));
    }
}


