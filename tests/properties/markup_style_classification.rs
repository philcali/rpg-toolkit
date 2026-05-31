// Feature: dialog-rendering-polish, Property 2: Markup style classification correctness
//
// For generated inputs with properly fenced styled spans, `parse_markup` assigns
// correct styles: Italic for `_text_`, Bold for `__text__`, BoldItalic for `___text___`.
//
// Strategy: Generate structured markup strings by interleaving plain text segments
// with properly fenced styled spans, then verify that `parse_markup` assigns the
// expected style to each span.
//
// Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5

use proptest::prelude::*;
use rpg_toolkit_renderer::markup::{TextSegment, TextStyle, parse_markup};

/// Generate a non-empty string that does not contain underscores.
/// This ensures the generated content won't accidentally form delimiters.
fn plain_text_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ,.!?]{1,20}".prop_filter("no underscores", |s| !s.contains('_'))
}

/// Generate a non-empty string that does not contain underscores (for styled inner text).
fn inner_text_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ,.!?]{1,15}".prop_filter("no underscores", |s| !s.contains('_'))
}

/// Represents a segment in our generated test input.
#[derive(Clone, Debug)]
enum GeneratedSegment {
    Plain(String),
    Styled { text: String, delim_len: usize },
}

impl GeneratedSegment {
    fn to_markup_string(&self) -> String {
        match self {
            GeneratedSegment::Plain(s) => s.clone(),
            GeneratedSegment::Styled { text, delim_len } => {
                let delim = "_".repeat(*delim_len);
                format!("{}{}{}", delim, text, delim)
            }
        }
    }

    fn expected_style(&self) -> TextStyle {
        match self {
            GeneratedSegment::Plain(_) => TextStyle::Plain,
            GeneratedSegment::Styled { delim_len, .. } => match delim_len {
                1 => TextStyle::Italic,
                2 => TextStyle::Bold,
                3 => TextStyle::BoldItalic,
                _ => unreachable!(),
            },
        }
    }

    fn expected_text(&self) -> &str {
        match self {
            GeneratedSegment::Plain(s) => s,
            GeneratedSegment::Styled { text, .. } => text,
        }
    }
}

/// Strategy that generates a sequence of segments (plain and styled interleaved).
fn segments_strategy() -> impl Strategy<Value = Vec<GeneratedSegment>> {
    // Generate 1-5 segments, alternating between plain and styled
    prop::collection::vec(
        prop_oneof![
            plain_text_strategy().prop_map(GeneratedSegment::Plain),
            (inner_text_strategy(), 1usize..=3)
                .prop_map(|(text, delim_len)| { GeneratedSegment::Styled { text, delim_len } }),
        ],
        1..=6,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 2: For generated inputs with properly fenced styled spans,
    /// parse_markup assigns the correct style to each span.
    #[test]
    fn style_classification_is_correct(segments in segments_strategy()) {
        // Build the markup string from generated segments
        let markup: String = segments.iter().map(|s| s.to_markup_string()).collect();

        // Parse it
        let parsed = parse_markup(&markup);

        // Build expected segments (filtering empty and merging adjacent same-style)
        let mut expected: Vec<TextSegment> = Vec::new();
        for seg in &segments {
            let text = seg.expected_text();
            if text.is_empty() {
                continue;
            }
            let style = seg.expected_style();
            if let Some(last) = expected.last_mut()
                && last.style == style {
                    last.text.push_str(text);
                    continue;
                }
            expected.push(TextSegment {
                text: text.to_string(),
                style,
            });
        }

        prop_assert_eq!(
            &parsed,
            &expected,
            "Markup: {:?}\nExpected segments: {:?}\nActual segments: {:?}",
            markup,
            expected,
            parsed
        );
    }

    /// Property 2b: Single styled span with each delimiter length is correctly classified.
    #[test]
    fn single_span_style_correct(
        inner in inner_text_strategy(),
        delim_len in 1usize..=3,
    ) {
        let delim = "_".repeat(delim_len);
        let markup = format!("{}{}{}", delim, inner, delim);

        let parsed = parse_markup(&markup);

        let expected_style = match delim_len {
            1 => TextStyle::Italic,
            2 => TextStyle::Bold,
            3 => TextStyle::BoldItalic,
            _ => unreachable!(),
        };

        prop_assert_eq!(parsed.len(), 1, "Expected exactly one segment for {:?}", markup);
        prop_assert_eq!(&parsed[0].text, &inner, "Text content mismatch for {:?}", markup);
        prop_assert_eq!(&parsed[0].style, &expected_style, "Style mismatch for {:?}", markup);
    }

    /// Property 2c: Plain text with no underscores is always classified as Plain.
    #[test]
    fn plain_text_always_plain(text in plain_text_strategy()) {
        let parsed = parse_markup(&text);

        prop_assert_eq!(parsed.len(), 1, "Expected exactly one segment for plain text {:?}", text);
        prop_assert_eq!(&parsed[0].text, &text);
        prop_assert_eq!(&parsed[0].style, &TextStyle::Plain);
    }
}
