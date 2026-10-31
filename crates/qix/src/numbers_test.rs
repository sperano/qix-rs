use crate::numbers::thousands;

#[test]
fn numbers_below_a_thousand_have_no_separator() {
    assert_eq!(thousands(0), "0");
    assert_eq!(thousands(7), "7");
    assert_eq!(thousands(999), "999");
}

#[test]
fn every_three_digits_from_the_right_are_separated() {
    assert_eq!(thousands(1_000), "1,000");
    assert_eq!(thousands(12_345), "12,345");
    assert_eq!(thousands(123_456), "123,456");
    assert_eq!(thousands(1_234_567), "1,234,567");
}

#[test]
fn the_largest_score_is_grouped_too() {
    assert_eq!(thousands(u32::MAX), "4,294,967,295");
}
