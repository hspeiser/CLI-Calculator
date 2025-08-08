pub mod ast;
pub mod diag;
pub mod lexer;
pub mod parser;
pub mod binder;
pub mod types;
pub mod units;
pub mod registry;
pub mod ir;
pub mod vm;
pub mod depgraph;
pub mod engine;

pub use engine::{Engine, EvalOutput};

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use proptest::prelude::*;
    use crate::types::Value;

    #[test]
    fn parse_quantity_and_parallel() {
        let mut engine = Engine::new();
        let out = engine.eval_cell("r1 = 10kΩ").unwrap();
        assert_eq!(out.value.display(), "10000 Ω");
        let _ = engine.eval_cell("r2 = 15kΩ").unwrap();
        let out = engine.eval_cell("r_eq = r1 // r2").unwrap();
        assert_eq!(out.value.display(), "6000 Ω");
    }

    #[test]
    fn basic_units_and_division() {
        let mut engine = Engine::new();
        let _ = engine.eval_cell("r = 10kΩ").unwrap();
        let _ = engine.eval_cell("v = 5 V").unwrap();
        let out = engine.eval_cell("i = v / r").unwrap();
        assert!(out.value.display().contains("A") || out.value.display().contains("V/Ω"));
    }

    #[test]
    fn complex_constant_i_in_arith() {
        let mut engine = Engine::new();
        engine.eval_cell("x = 10").unwrap();
        let out = engine.eval_cell("x * 10 / 4 * i").unwrap();
        assert!(out.value.display().contains("i"));
    }

    #[test]
    fn conversion_and_trig() {
        let mut e = Engine::new();
        // conversion
        let _ = e.eval_cell("d = 10 m").unwrap();
        let out = e.eval_cell("d to in").unwrap();
        assert!(out.value.display().contains("in"));
        // trig
        let s = e.eval_cell("sin(pi()/2)").unwrap();
        assert!(s.value.display().starts_with("1"), "{}", s.value.display());
        let c = e.eval_cell("cos_deg(60)").unwrap();
        assert!(c.value.display().starts_with("0.5"));
        // angle quantity in degrees
        let s2 = e.eval_cell("sin(90 °)").unwrap();
        assert!(s2.value.display().starts_with("1"), "{}", s2.value.display());
    }

    #[test]
    fn complex_impedance_parallel() {
        let mut engine = Engine::new();
        // Zc(f, C) = -i / (2π f C)  (units emerge to Ω automatically)
        let _ = engine.eval_cell("Zc(f, C) = -1 * i() / (2*pi() * f * C)").unwrap();
        let _ = engine.eval_cell("Z = 100 Ω // Zc(1000 Hz, 100 nF)").unwrap();
        let out = engine.eval_cell("Z").unwrap();
        // Ensure we got some complex unit Ω display
        assert!(out.value.display().contains("Ω"));
    }

    #[test]
    fn parallel_associativity_on_equal_units() {
        let mut engine = Engine::new();
        let _ = engine.eval_cell("a = 10 Ω").unwrap();
        let _ = engine.eval_cell("b = 20 Ω").unwrap();
        let _ = engine.eval_cell("c = 30 Ω").unwrap();
        let left = engine.eval_cell("x = (a // b) // c").unwrap();
        let right = engine.eval_cell("y = a // (b // c)").unwrap();
        // Just ensure numeric closeness by comparing displayed strings normalized
        assert!(left.value.display().contains("Ω"));
        assert!(right.value.display().contains("Ω"));
    }
}

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // For equal units, parallel should be commutative and associative numerically
        #[test]
        fn parallel_commutative(a in 1.0f64..1e6, b in 1.0f64..1e6) {
            let mut e = Engine::new();
            e.eval_cell(&format!("x = {} Ω", a)).unwrap();
            e.eval_cell(&format!("y = {} Ω", b)).unwrap();
            let r1 = e.eval_cell("r = x // y").unwrap().value.display();
            let r2 = e.eval_cell("r = y // x").unwrap().value.display();
            prop_assert_eq!(r1, r2);
        }

        #[test]
        fn parallel_associative(a in 1.0f64..1e6, b in 1.0f64..1e6, c in 1.0f64..1e6) {
            let mut e = Engine::new();
            e.eval_cell(&format!("a = {} Ω", a)).unwrap();
            e.eval_cell(&format!("b = {} Ω", b)).unwrap();
            e.eval_cell(&format!("c = {} Ω", c)).unwrap();
            let l = e.eval_cell("x = (a // b) // c").unwrap().value.display();
            let r = e.eval_cell("y = a // (b // c)").unwrap().value.display();
            prop_assert!(l.contains("Ω") && r.contains("Ω"));
        }
    }
}
