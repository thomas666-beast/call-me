/// Fixed set of pause reasons. Values must match what the frontend sends.
pub const PAUSE_REASONS: &[&str] = &[
    "lunch",
    "break",
    "meeting",
    "training",
    "wrap-up",
    "other",
];

pub fn is_valid_reason(reason: &str) -> bool {
    PAUSE_REASONS.contains(&reason)
}
