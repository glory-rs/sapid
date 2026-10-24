#![cfg(feature = "json")]
use sapid::{
    JsonError, JsonErrorKind, JsonFormat, JsonLimits, JsonValidity, format_json, validate_json,
};

#[test]
fn exact_tokens_and_duplicate_keys_survive_formatting() {
    let original = r#"{"z":1e999,"a":9007199254740993123456789,"z":"\u0041\\\"","emoji":"😀"}"#;
    let pretty = format_json(original, JsonFormat::PrettyFour, JsonLimits::default()).unwrap();
    assert_eq!(
        format_json(&pretty, JsonFormat::Compact, JsonLimits::default()).unwrap(),
        original
    );
    assert_eq!(
        format_json(&pretty, JsonFormat::PrettyFour, JsonLimits::default()).unwrap(),
        pretty
    );
}

#[test]
fn empty_containers_and_escaped_punctuation_have_correct_indentation() {
    let original =
        r#"{"empty":{},"list":[],"escaped":"[,\\\"]","items":[1,true,null,{"a":2}]}"#;
    let formatted =
        format_json(original, JsonFormat::PrettyTwo, JsonLimits::default()).unwrap();
    assert!(formatted.contains("\n  \"items\": [\n    1,"));
    assert!(formatted.contains("\"empty\": {}"));
    assert_eq!(
        format_json(&formatted, JsonFormat::Compact, JsonLimits::default()).unwrap(),
        original
    );
}

#[test]
fn byte_depth_output_and_configuration_budgets_reject_before_unbounded_work() {
    let nested = format!("{}0{}", "[".repeat(30000), "]".repeat(30000));
    assert!(matches!(
        validate_json(&nested, JsonLimits::default()),
        JsonValidity::Invalid(JsonError {
            kind: JsonErrorKind::DepthBudget,
            ..
        })
    ));
    assert!(matches!(
        validate_json(&" ".repeat(65537), JsonLimits::default()),
        JsonValidity::Invalid(JsonError {
            kind: JsonErrorKind::InputBudget,
            ..
        })
    ));
    let limits = JsonLimits { bytes: 8, depth: 8 };
    assert_eq!(
        format_json("[1,2,3]", JsonFormat::PrettyFour, limits)
            .unwrap_err()
            .kind,
        JsonErrorKind::OutputBudget
    );
    assert!(JsonLimits { bytes: 0, depth: 1 }.validate().is_err());
}

#[test]
fn malformed_drafts_and_non_json_whitespace_are_distinct_from_empty() {
    assert_eq!(
        validate_json(" \t\r\n", JsonLimits::default()),
        JsonValidity::Empty
    );
    for draft in ["{", "\u{a0}1\u{a0}", "{\"x\":}", "true false", "\u{b}"] {
        assert!(
            matches!(
                validate_json(draft, JsonLimits::default()),
                JsonValidity::Invalid(JsonError {
                    kind: JsonErrorKind::Syntax,
                    ..
                })
            ),
            "{draft:?}"
        );
    }
}
