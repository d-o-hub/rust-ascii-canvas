use ascii_canvas::utils::math::signum;

#[test]
fn test_signum_i32() {
    assert_eq!(signum(10i32), 1, "Positive i32 should return 1");
    assert_eq!(signum(-10i32), -1, "Negative i32 should return -1");
    assert_eq!(signum(0i32), 0, "Zero i32 should return 0");
    assert_eq!(signum(i32::MAX), 1, "Max i32 should return 1");
    assert_eq!(signum(i32::MIN), -1, "Min i32 should return -1");
}

#[test]
fn test_signum_f64() {
    assert_eq!(signum(10.5f64), 1, "Positive f64 should return 1");
    assert_eq!(signum(-10.5f64), -1, "Negative f64 should return -1");
    assert_eq!(signum(0.0f64), 0, "Zero f64 should return 0");
    assert_eq!(signum(-0.0f64), 0, "Negative zero f64 should return 0");
    assert_eq!(
        signum(f64::INFINITY),
        1,
        "Positive infinity should return 1"
    );
    assert_eq!(
        signum(f64::NEG_INFINITY),
        -1,
        "Negative infinity should return -1"
    );
    assert_eq!(
        signum(f64::NAN),
        0,
        "NaN should return 0 based on standard PartialOrd behavior"
    );
}
