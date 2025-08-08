use calc_core::Engine;

#[test]
fn implicit_multiply_and_trig() {
    let mut e = Engine::new();
    e.eval_cell("x = 10").unwrap();
    let out = e.eval_cell("10x").unwrap();
    assert_eq!(out.value.display(), "100");

    let out = e.eval_cell("sin(pi()/2)").unwrap();
    assert!(out.value.display().starts_with("1"));
}

#[test]
fn quantity_compute_and_convert() {
    let mut e = Engine::new();
    e.eval_cell("r = 10kΩ").unwrap();
    e.eval_cell("v = 5 V").unwrap();
    let out = e.eval_cell("i = v / r").unwrap();
    // normalized to A by engine post-processing
    assert!(out.value.display().contains("A") || out.value.display().contains("V/Ω"));

    e.eval_cell("d = 1 m").unwrap();
    let out = e.eval_cell("d to in").unwrap();
    assert!(out.value.display().contains("in"));
}


