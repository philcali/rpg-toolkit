// Feature: graphics-database-editor, Property 1: AssetManager resolution round-trip

use std::path::Path;

use proptest::prelude::*;

use rpg_toolkit_common::asset::AssetManager;

/// Strategy for generating valid path segments (non-empty, no `.` or `..`, alphanumeric + underscore/hyphen).
fn arb_path_segment() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_\\-]{1,20}"
}

/// Strategy for generating valid relative paths:
/// - Non-empty
/// - No `.` or `..` components
/// - Forward-slash separators
/// - ≤260 characters total
fn arb_valid_relative_path() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_path_segment(), 1..=5)
        .prop_map(|segments| segments.join("/"))
        .prop_filter("path must be ≤260 characters", |p| p.len() <= 260)
}

/// Strategy for generating valid root paths (absolute path, non-empty).
fn arb_root_path() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_path_segment(), 1..=4)
        .prop_map(|segments| format!("/{}", segments.join("/")))
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 10.1, 10.4**
    ///
    /// Property 1: AssetManager resolution round-trip — For any valid relative file path
    /// (non-empty, no `.` or `..` components, ≤260 characters, forward-slash separators)
    /// and any project root path, resolving through `AssetManager::resolve_path` and then
    /// stripping the project root prefix from the result SHALL reproduce the original
    /// relative path exactly.
    #[test]
    fn asset_resolution_round_trip(
        root in arb_root_path(),
        relative in arb_valid_relative_path(),
    ) {
        let root_path = Path::new(&root);
        let resolved = AssetManager::resolve_path(root_path, &relative)
            .expect("resolve_path should succeed for valid inputs");

        // Strip the root prefix from the resolved path
        let stripped = resolved
            .strip_prefix(root_path)
            .expect("resolved path should start with root");

        // Convert back to a string with forward slashes for comparison
        let stripped_str = stripped.to_str()
            .expect("stripped path should be valid UTF-8");

        prop_assert_eq!(
            stripped_str, &relative,
            "Stripping root from resolved path should reproduce the original relative path. \
             Root: {:?}, Relative: {:?}, Resolved: {:?}, Stripped: {:?}",
            root, relative, resolved, stripped_str
        );
    }

    /// **Validates: Requirements 10.4**
    ///
    /// For different root paths, the relative suffix extracted by stripping each respective
    /// root prefix SHALL remain equal to the original relative path.
    #[test]
    fn asset_resolution_round_trip_different_roots(
        root1 in arb_root_path(),
        root2 in arb_root_path(),
        relative in arb_valid_relative_path(),
    ) {
        let root_path1 = Path::new(&root1);
        let root_path2 = Path::new(&root2);

        let resolved1 = AssetManager::resolve_path(root_path1, &relative)
            .expect("resolve_path should succeed for valid inputs with root1");
        let resolved2 = AssetManager::resolve_path(root_path2, &relative)
            .expect("resolve_path should succeed for valid inputs with root2");

        let stripped1 = resolved1
            .strip_prefix(root_path1)
            .expect("resolved1 should start with root1");
        let stripped2 = resolved2
            .strip_prefix(root_path2)
            .expect("resolved2 should start with root2");

        let stripped_str1 = stripped1.to_str()
            .expect("stripped path 1 should be valid UTF-8");
        let stripped_str2 = stripped2.to_str()
            .expect("stripped path 2 should be valid UTF-8");

        prop_assert_eq!(
            stripped_str1, &relative,
            "Round-trip with root1 should reproduce relative path"
        );
        prop_assert_eq!(
            stripped_str2, &relative,
            "Round-trip with root2 should reproduce relative path"
        );
    }
}
