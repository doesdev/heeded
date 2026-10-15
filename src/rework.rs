pub const OUTCOMES: [&str; 3] = ["kept", "reworked", "discarded"];

pub const CLASSES: [&str; 6] = [
    "none",
    "wrong-context",
    "wrong-assumption",
    "missing-test",
    "style-mismatch",
    "spec-gap",
];

/// What is wrong with a rework line. Both sets are closed on purpose: a
/// free-text class cannot be counted.
pub fn check_rework(task: &str, outcome: &str, class: &str) -> Vec<&'static str> {
    let checks = [
        ("task", !task.trim().is_empty()),
        ("outcome", OUTCOMES.contains(&outcome)),
        ("class", CLASSES.contains(&class)),
        (
            "class-for-outcome",
            (outcome == "kept") == (class == "none"),
        ),
    ];

    checks
        .into_iter()
        .filter(|(_, ok)| !ok)
        .map(|(name, _)| name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holds_a_line_to_the_closed_sets() {
        let cases: [(&str, &str, &str, &[&str]); 7] = [
            ("x", "kept", "none", &[]),
            ("x", "reworked", "spec-gap", &[]),
            ("x", "discarded", "wrong-context", &[]),
            ("x", "kept", "spec-gap", &["class-for-outcome"]),
            ("x", "reworked", "none", &["class-for-outcome"]),
            (" ", "fine", "vibes", &["task", "outcome", "class"]),
            ("", "", "", &["task", "outcome", "class"]),
        ];

        for (task, outcome, class, problems) in cases {
            assert_eq!(
                check_rework(task, outcome, class),
                problems,
                "{outcome} {class}"
            );
        }
    }
}
