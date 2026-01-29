use super::*;

/// Ensures heat scaling is capped at the maximum value.
#[test]
fn heat_scaling_is_capped() {
    let high_events = (MAX_HEAT_SCALING / HEAT_SCALING_FACTOR + 10) as usize;
    assert_eq!(heat_scaling(high_events), MAX_HEAT_SCALING);
}

/// Ensures base heat includes the scaling factor.
#[test]
fn heat_base_includes_scaling() {
    let base = heat_base(0);
    assert_eq!(base, BASE_HEAT);
    let scaled = heat_base(10);
    assert!(scaled >= base);
}

/// Ensures injection count grows with event count.
#[test]
fn num_injections_increases_with_events() {
    let width = 80;
    let low = num_injections(width, 0);
    let high = num_injections(width, 80);
    assert!(high >= low);
}

/// Ensures cooling never drops below 1.
#[test]
fn cooling_value_is_at_least_one() {
    let cooling = cooling_value(1, 10_000);
    assert_eq!(cooling, 1);
}

/// Ensures color index matches heat thresholds.
#[test]
fn color_idx_thresholds() {
    assert_eq!(color_idx(0), 0);
    assert_eq!(color_idx(1), 1);
    assert_eq!(color_idx(6), 2);
    assert_eq!(color_idx(11), 3);
    assert_eq!(color_idx(16), 4);
    assert_eq!(color_idx(21), 5);
}

/// Ensures character index stays within bounds.
#[test]
fn char_idx_is_bounded() {
    let len = 11;
    assert_eq!(char_idx(0, len), 0);
    assert_eq!(char_idx(25, len), len - 1);
}
