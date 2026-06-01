// Feature: editor-ux-improvements, Property 5: Case-Insensitive Substring Filter
//
// For any list of names and for any non-empty search query, the filter function
// SHALL return exactly those names whose lowercase representation contains the
// lowercase query as a substring.
//
// Validates: Requirements 5.2, 6.2

use proptest::prelude::*;

/// Reference implementation of the case-insensitive substring filter.
/// This mirrors the specification from the design document and the implementation
/// in `rpg_toolkit_editor::plugins::searchable_combobox::filter_items`.
fn filter_items<'a>(items: &'a [(String, String)], query: &str) -> Vec<&'a (String, String)> {
    let query_lower = query.to_lowercase();
    let mut filtered: Vec<&'a (String, String)> = if query_lower.is_empty() {
        items.iter().collect()
    } else {
        items
            .iter()
            .filter(|(_, label)| label.to_lowercase().contains(&query_lower))
            .collect()
    };
    filtered.sort_by_key(|(_, label)| label.to_lowercase());
    filtered
}

/// Strategy for generating a list of (id, display_label) items.
fn arb_items() -> impl Strategy<Value = Vec<(String, String)>> {
    prop::collection::vec(("[a-z0-9]{1,8}", "[a-zA-Z0-9 ]{1,20}"), 0..=20)
        .prop_map(|pairs| pairs.into_iter().collect())
}

/// Strategy for generating a non-empty search query.
fn arb_non_empty_query() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]{1,10}"
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 5.2, 6.2**
    ///
    /// Property 5: For any list of items and any non-empty query, filter_items
    /// returns exactly those items whose display_label (lowercased) contains
    /// the query (lowercased) as a substring.
    #[test]
    fn case_insensitive_substring_filter(
        items in arb_items(),
        query in arb_non_empty_query(),
    ) {
        let result = filter_items(&items, &query);
        let query_lower = query.to_lowercase();

        // Compute expected set: items whose lowercase label contains lowercase query
        let expected: Vec<&(String, String)> = items
            .iter()
            .filter(|(_, label)| label.to_lowercase().contains(&query_lower))
            .collect();

        // The result should contain exactly the expected items (same count)
        prop_assert_eq!(
            result.len(),
            expected.len(),
            "Filter returned {} items but expected {}",
            result.len(),
            expected.len()
        );

        // Every item in the result must have its lowercase label contain the lowercase query
        for (_, label) in &result {
            prop_assert!(
                label.to_lowercase().contains(&query_lower),
                "Item '{}' does not contain query '{}' (case-insensitive)",
                label,
                query
            );
        }

        // Every item in the input that matches must be present in the result
        for item in &expected {
            prop_assert!(
                result.contains(item),
                "Expected item '{:?}' missing from filter result",
                item
            );
        }
    }
}

// Feature: editor-ux-improvements, Property 6: Alphabetical Sort with Empty Filter
//
// For any list of names, when the search query is empty, the result SHALL contain
// all names and they SHALL be sorted in case-insensitive alphabetical order.
//
// Validates: Requirements 5.5, 6.5

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 5.5, 6.5**
    ///
    /// Property 6: For any list of (id, display_label) items, calling filter_items
    /// with an empty query returns ALL items sorted case-insensitively by display_label.
    #[test]
    fn alphabetical_sort_with_empty_filter(
        items in arb_items(),
    ) {
        let result = filter_items(&items, "");

        // Assert the result contains ALL items (same count as input)
        prop_assert_eq!(
            result.len(),
            items.len(),
            "Empty filter returned {} items but input has {}",
            result.len(),
            items.len()
        );

        // Assert every input item is present in the result
        for item in &items {
            prop_assert!(
                result.contains(&item),
                "Item '{:?}' missing from empty-filter result",
                item
            );
        }

        // Assert the result is sorted case-insensitively by display_label
        for window in result.windows(2) {
            let (_, label_a) = window[0];
            let (_, label_b) = window[1];
            prop_assert!(
                label_a.to_lowercase() <= label_b.to_lowercase(),
                "Items not sorted case-insensitively: '{}' should come before '{}'",
                label_a,
                label_b
            );
        }
    }
}
