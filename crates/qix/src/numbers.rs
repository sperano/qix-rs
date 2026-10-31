//! Numbers as the player reads them.

/// Digits between two thousands separators.
const GROUP: usize = 3;

/// Between two groups of digits.
const SEPARATOR: char = ',';

/// `n` in decimal with its digits grouped by thousands: `1234567` reads
/// `1,234,567`.
pub fn thousands(n: u32) -> String {
    let digits = n.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / GROUP);
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(GROUP) {
            grouped.push(SEPARATOR);
        }
        grouped.push(digit);
    }
    grouped
}
