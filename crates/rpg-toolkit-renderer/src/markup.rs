/// Inline text style markup parser for dialog text.
///
/// Supports underscore-fenced styling:
/// - `___text___` → BoldItalic
/// - `__text__` → Bold
/// - `_text_` → Italic
///
/// Unclosed delimiters are emitted as plain text.
/// The visual style applied to a text segment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextStyle {
    Plain,
    Bold,
    Italic,
    BoldItalic,
}

/// A segment of parsed dialog text with an associated style.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextSegment {
    pub text: String,
    pub style: TextStyle,
}

/// Parse underscore-fenced markup into styled text segments.
///
/// The parser scans left-to-right, greedily matching the longest delimiter
/// first (3 underscores, then 2, then 1). When an opening delimiter is found,
/// it searches for the matching closing delimiter. If not found before end-of-string,
/// the opening underscores are emitted as plain text.
///
/// # Examples
///
/// ```
/// use rpg_toolkit_renderer::markup::{parse_markup, TextSegment, TextStyle};
///
/// let segments = parse_markup("Hello __world__!");
/// assert_eq!(segments, vec![
///     TextSegment { text: "Hello ".to_string(), style: TextStyle::Plain },
///     TextSegment { text: "world".to_string(), style: TextStyle::Bold },
///     TextSegment { text: "!".to_string(), style: TextStyle::Plain },
/// ]);
/// ```
pub fn parse_markup(input: &str) -> Vec<TextSegment> {
    let mut segments: Vec<TextSegment> = Vec::new();
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut pos = 0;
    let mut plain_start = 0;

    while pos < len {
        if bytes[pos] == b'_' {
            // Count consecutive underscores
            let underscore_start = pos;
            let mut count = 0;
            while pos < len && bytes[pos] == b'_' {
                count += 1;
                pos += 1;
            }

            // Try to match a delimiter (greedy: 3, then 2, then 1)
            let matched = try_match_delimiter(bytes, underscore_start, count, len);

            if let Some((style, inner_start, inner_end, after_close)) = matched {
                // Flush any accumulated plain text before this delimiter
                if underscore_start > plain_start {
                    push_segment(
                        &mut segments,
                        &input[plain_start..underscore_start],
                        TextStyle::Plain,
                    );
                }

                // Emit the styled segment
                push_segment(&mut segments, &input[inner_start..inner_end], style);

                pos = after_close;
                plain_start = pos;
            } else {
                // No closing delimiter found — underscores are plain text, continue scanning
                // pos is already advanced past the underscores
            }
        } else {
            pos += 1;
        }
    }

    // Flush remaining plain text
    if plain_start < len {
        push_segment(&mut segments, &input[plain_start..len], TextStyle::Plain);
    }

    segments
}

/// Try to match a delimiter starting at `underscore_start` with `count` underscores.
/// Tries the longest delimiter first (min of count and 3), then shorter ones.
/// Returns `Some((style, inner_start, inner_end, position_after_closing_delimiter))` on success.
fn try_match_delimiter(
    bytes: &[u8],
    underscore_start: usize,
    count: usize,
    len: usize,
) -> Option<(TextStyle, usize, usize, usize)> {
    // Try delimiter lengths from longest (capped at 3) down to 1
    let max_delim = count.min(3);

    for delim_len in (1..=max_delim).rev() {
        let style = match delim_len {
            3 => TextStyle::BoldItalic,
            2 => TextStyle::Bold,
            1 => TextStyle::Italic,
            _ => unreachable!(),
        };

        // Inner content starts after ALL counted underscores, not just delim_len.
        // This prevents the "extra" underscores from being matched as closing delimiters.
        let inner_start = underscore_start + count;

        // Search for the closing delimiter
        if let Some(close_pos) = find_closing_delimiter(bytes, inner_start, delim_len, len) {
            let inner_end = close_pos;
            let after_close = close_pos + delim_len;
            return Some((style, inner_start, inner_end, after_close));
        }
    }

    None
}

/// Search for a closing delimiter of `delim_len` underscores starting from `search_start`.
/// Returns the byte position where the closing delimiter begins, or None.
fn find_closing_delimiter(
    bytes: &[u8],
    search_start: usize,
    delim_len: usize,
    len: usize,
) -> Option<usize> {
    let mut i = search_start;

    while i + delim_len <= len {
        if bytes[i] == b'_' {
            // Count consecutive underscores at this position
            let mut consecutive = 0;
            let start = i;
            while i < len && bytes[i] == b'_' {
                consecutive += 1;
                i += 1;
            }

            // Check if we have an exact match for the closing delimiter.
            // We require exactly `delim_len` underscores (not more) to avoid
            // ambiguity with nested/adjacent delimiters.
            if consecutive == delim_len {
                return Some(start);
            }

            // If we found more underscores than needed, we can still match
            // the first `delim_len` of them as the closing delimiter.
            if consecutive >= delim_len {
                return Some(start);
            }

            // Fewer underscores than needed — not a match, continue scanning
            // (i is already advanced past the underscores)
        } else {
            i += 1;
        }
    }

    None
}

/// Push a segment onto the list, merging with the previous segment if styles match
/// and avoiding empty segments.
fn push_segment(segments: &mut Vec<TextSegment>, text: &str, style: TextStyle) {
    if text.is_empty() {
        return;
    }

    // Merge with previous segment if same style
    if let Some(last) = segments.last_mut()
        && last.style == style
    {
        last.text.push_str(text);
        return;
    }

    segments.push(TextSegment {
        text: text.to_string(),
        style,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_no_markup() {
        let result = parse_markup("Hello world");
        assert_eq!(
            result,
            vec![TextSegment {
                text: "Hello world".to_string(),
                style: TextStyle::Plain,
            }]
        );
    }

    #[test]
    fn single_bold() {
        let result = parse_markup("Hello __world__!");
        assert_eq!(
            result,
            vec![
                TextSegment {
                    text: "Hello ".to_string(),
                    style: TextStyle::Plain,
                },
                TextSegment {
                    text: "world".to_string(),
                    style: TextStyle::Bold,
                },
                TextSegment {
                    text: "!".to_string(),
                    style: TextStyle::Plain,
                },
            ]
        );
    }

    #[test]
    fn single_italic() {
        let result = parse_markup("Hello _world_!");
        assert_eq!(
            result,
            vec![
                TextSegment {
                    text: "Hello ".to_string(),
                    style: TextStyle::Plain,
                },
                TextSegment {
                    text: "world".to_string(),
                    style: TextStyle::Italic,
                },
                TextSegment {
                    text: "!".to_string(),
                    style: TextStyle::Plain,
                },
            ]
        );
    }

    #[test]
    fn single_bold_italic() {
        let result = parse_markup("Hello ___world___!");
        assert_eq!(
            result,
            vec![
                TextSegment {
                    text: "Hello ".to_string(),
                    style: TextStyle::Plain,
                },
                TextSegment {
                    text: "world".to_string(),
                    style: TextStyle::BoldItalic,
                },
                TextSegment {
                    text: "!".to_string(),
                    style: TextStyle::Plain,
                },
            ]
        );
    }

    #[test]
    fn multiple_styles() {
        let result = parse_markup("A _B_ C __D__ E ___F___ G");
        assert_eq!(
            result,
            vec![
                TextSegment {
                    text: "A ".to_string(),
                    style: TextStyle::Plain,
                },
                TextSegment {
                    text: "B".to_string(),
                    style: TextStyle::Italic,
                },
                TextSegment {
                    text: " C ".to_string(),
                    style: TextStyle::Plain,
                },
                TextSegment {
                    text: "D".to_string(),
                    style: TextStyle::Bold,
                },
                TextSegment {
                    text: " E ".to_string(),
                    style: TextStyle::Plain,
                },
                TextSegment {
                    text: "F".to_string(),
                    style: TextStyle::BoldItalic,
                },
                TextSegment {
                    text: " G".to_string(),
                    style: TextStyle::Plain,
                },
            ]
        );
    }

    #[test]
    fn unclosed_delimiter_is_plain() {
        let result = parse_markup("Hello __world");
        assert_eq!(
            result,
            vec![TextSegment {
                text: "Hello __world".to_string(),
                style: TextStyle::Plain,
            }]
        );
    }

    #[test]
    fn empty_input() {
        let result = parse_markup("");
        assert_eq!(result, vec![]);
    }

    #[test]
    fn only_underscores_unclosed() {
        let result = parse_markup("___");
        assert_eq!(
            result,
            vec![TextSegment {
                text: "___".to_string(),
                style: TextStyle::Plain,
            }]
        );
    }

    #[test]
    fn only_underscores_no_content() {
        // 4 underscores with no content after them — no closing delimiter can be found,
        // so all underscores are emitted as plain text.
        let result = parse_markup("____");
        assert_eq!(
            result,
            vec![TextSegment {
                text: "____".to_string(),
                style: TextStyle::Plain,
            }]
        );
    }

    #[test]
    fn adjacent_styled_spans() {
        let result = parse_markup("__bold___italic_");
        assert_eq!(
            result,
            vec![
                TextSegment {
                    text: "bold".to_string(),
                    style: TextStyle::Bold,
                },
                TextSegment {
                    text: "italic".to_string(),
                    style: TextStyle::Italic,
                },
            ]
        );
    }
}
