use bevy_egui::egui;

/// Filters items by a search query using case-insensitive substring matching.
///
/// - If `query` is empty, returns all items sorted alphabetically by display_label (case-insensitive).
/// - If `query` is non-empty, returns items whose display_label contains the query as a
///   case-insensitive substring, sorted alphabetically by display_label (case-insensitive).
pub fn filter_items<'a>(items: &'a [(String, String)], query: &str) -> Vec<&'a (String, String)> {
    let query_lower = query.to_lowercase();
    let mut filtered: Vec<&'a (String, String)> = if query_lower.is_empty() {
        items.iter().collect()
    } else {
        items
            .iter()
            .filter(|(_, label)| label.to_lowercase().contains(&query_lower))
            .collect()
    };
    filtered
        .sort_by(|(_, a_label), (_, b_label)| a_label.to_lowercase().cmp(&b_label.to_lowercase()));
    filtered
}

/// Renders a searchable combobox dropdown using egui.
///
/// - `id_salt`: unique identifier for the combobox widget
/// - `current_label`: text shown as the currently selected value
/// - `items`: slice of `(id, display_label)` pairs
/// - `search_buffer`: mutable reference to the search text state
///
/// Returns `Some(id)` when the user selects an item, `None` otherwise.
#[allow(dead_code)]
pub fn searchable_combobox(
    ui: &mut egui::Ui,
    id_salt: &str,
    current_label: &str,
    items: &[(String, String)],
    search_buffer: &mut String,
) -> Option<String> {
    let mut selected_id: Option<String> = None;

    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(current_label)
        .show_ui(ui, |ui| {
            // Search input at the top
            ui.add(
                egui::TextEdit::singleline(search_buffer)
                    .hint_text("Search…")
                    .desired_width(f32::INFINITY),
            );

            ui.separator();

            // Filter and display items
            let filtered = filter_items(items, search_buffer);

            if filtered.is_empty() {
                ui.label("No results");
            } else {
                for (id, label) in filtered {
                    if ui.selectable_label(false, label).clicked() {
                        selected_id = Some(id.clone());
                    }
                }
            }
        });

    selected_id
}
