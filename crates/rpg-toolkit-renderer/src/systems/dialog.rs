use bevy::prelude::*;

use crate::dialog::{
    DialogBox, DialogConfig, DialogPanel, DialogPosition, DialogState, DialogText, DialogTextNode,
    DialogTextRegistry, FacePortrait, OverflowIndicator, compute_visible_chars,
};
use crate::events::ShowDialog;
use crate::markup::{TextStyle, parse_markup};
use crate::resources::RendererProjectData;

/// Reads `ShowDialog` messages, resolves text, spawns dialog UI, and inserts `DialogState`.
pub fn handle_dialog_event(
    mut show_dialog: MessageReader<ShowDialog>,
    dialog_state: Option<Res<DialogState>>,
    registry: Option<Res<DialogTextRegistry>>,
    project_data: Option<Res<RendererProjectData>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for event in show_dialog.read() {
        // If a dialog is already active, ignore new events
        if dialog_state.is_some() {
            debug!("Dialog already active, ignoring ShowDialog event");
            return;
        }

        // Resolve the text content
        let resolved_text = match &event.text {
            DialogText::Inline(text) => text.clone(),
            DialogText::Id(id) => {
                let Some(reg) = &registry else {
                    warn!(
                        "ShowDialog with text ID '{}' but no DialogTextRegistry is present; ignoring",
                        id
                    );
                    continue;
                };
                let Some(text) = reg.get(id) else {
                    warn!(
                        "ShowDialog text ID '{}' not found in DialogTextRegistry; ignoring",
                        id
                    );
                    continue;
                };
                text.to_string()
            }
        };

        let total_chars = resolved_text.chars().count();
        let text_speed = event.config.text_speed;
        let fully_revealed = text_speed <= 0.0 || total_chars == 0;

        // Spawn the dialog UI, resolving face portrait ID to file path
        let mut resolved_config = event.config.clone();
        if let Some(ref portrait_id) = resolved_config.face_portrait {
            // Look up the portrait ID in the project's face_portraits registry
            let resolved_path = project_data
                .as_ref()
                .and_then(|pd| pd.project_file.face_portraits.get(portrait_id))
                .cloned();
            if resolved_path.is_none() && !portrait_id.is_empty() {
                warn!(
                    "Face portrait ID '{}' not found in project face_portraits registry; skipping portrait",
                    portrait_id
                );
            }
            resolved_config.face_portrait = resolved_path;
        }
        spawn_dialog_ui(
            &mut commands,
            &resolved_text,
            &resolved_config,
            &asset_server,
        );

        // Insert the DialogState resource
        commands.insert_resource(DialogState {
            full_text: resolved_text,
            total_chars,
            chars_revealed: if fully_revealed { total_chars } else { 0 },
            fully_revealed,
            elapsed: 0.0,
            text_speed,
            movement_blocked: event.config.movement_block,
        });

        // Only handle the first event
        return;
    }
}

/// Spawns the dialog box UI entities.
fn spawn_dialog_ui(
    commands: &mut Commands,
    text: &str,
    config: &DialogConfig,
    asset_server: &AssetServer,
) {
    let (justify_content, align_items) = match config.position {
        DialogPosition::Top => (JustifyContent::FlexStart, AlignItems::Center),
        DialogPosition::Center => (JustifyContent::Center, AlignItems::Center),
        DialogPosition::Bottom => (JustifyContent::FlexEnd, AlignItems::Center),
    };

    let padding = match config.position {
        DialogPosition::Top => UiRect::top(Val::Px(20.0)),
        DialogPosition::Center => UiRect::DEFAULT,
        DialogPosition::Bottom => UiRect::bottom(Val::Px(20.0)),
    };

    // Determine initial visible text based on text_speed
    let instant_reveal = config.text_speed <= 0.0 || text.is_empty();

    // Determine panel styling based on attribute_dialog mode
    let (background_color, border, border_color) = if config.attribute_dialog {
        // Attribute dialog: no background, no border
        (
            BackgroundColor(Color::NONE),
            UiRect::ZERO,
            BorderColor::all(Color::NONE),
        )
    } else {
        // Standard dialog: semi-transparent background with visible border
        (
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            UiRect::all(Val::Px(2.0)),
            BorderColor::all(Color::srgba(0.85, 0.85, 0.85, 1.0)),
        )
    };

    // Parse markup segments from the text
    let segments = parse_markup(text);

    // Spawn root dialog container with DialogBox marker
    // flex_direction: Column makes justify_content control the vertical axis,
    // so Top/Center/Bottom position the dialog box vertically as intended.
    commands
        .spawn((
            DialogBox,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content,
                align_items,
                padding,
                ..default()
            },
            GlobalZIndex(100),
        ))
        .with_children(|parent| {
            // Inner dialog panel with fixed height, overflow clipping, and border
            parent
                .spawn((
                    DialogPanel,
                    Node {
                        width: Val::Percent(80.0),
                        height: Val::Px(120.0),
                        padding: UiRect::all(Val::Px(16.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        border,
                        overflow: Overflow::clip(),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::FlexStart,
                        column_gap: Val::Px(12.0),
                        ..default()
                    },
                    background_color,
                    border_color,
                ))
                .with_children(|panel| {
                    // Face portrait (if configured)
                    if let Some(ref portrait_path) = config.face_portrait {
                        let portrait_handle: Handle<Image> =
                            asset_server.load(portrait_path.clone());
                        panel.spawn((
                            FacePortrait,
                            ImageNode {
                                image: portrait_handle,
                                ..default()
                            },
                            Node {
                                width: Val::Px(64.0),
                                height: Val::Px(64.0),
                                flex_shrink: 0.0,
                                ..default()
                            },
                        ));
                    }

                    // Text container (takes remaining space)
                    panel
                        .spawn(Node {
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        })
                        .with_children(|text_container| {
                            // Spawn parent Text entity with styled TextSpan children
                            let initial_text = if instant_reveal {
                                // For instant reveal, show the first segment's text
                                segments.first().map(|s| s.text.clone()).unwrap_or_default()
                            } else {
                                String::new()
                            };

                            let first_font = text_font_for_style(
                                segments
                                    .first()
                                    .map(|s| &s.style)
                                    .unwrap_or(&TextStyle::Plain),
                            );

                            text_container
                                .spawn((
                                    DialogTextNode,
                                    Text::new(initial_text),
                                    TextColor(Color::WHITE),
                                    first_font,
                                ))
                                .with_children(|text_parent| {
                                    // Spawn remaining segments as TextSpan children
                                    for segment in segments.iter().skip(1) {
                                        let span_text = if instant_reveal {
                                            segment.text.clone()
                                        } else {
                                            String::new()
                                        };

                                        text_parent.spawn((
                                            TextSpan(span_text),
                                            TextColor(Color::WHITE),
                                            text_font_for_style(&segment.style),
                                        ));
                                    }
                                });
                        });
                });
        });
}

/// Returns the appropriate `TextFont` for a given `TextStyle`.
fn text_font_for_style(style: &TextStyle) -> TextFont {
    match style {
        TextStyle::Plain => TextFont {
            font_size: 20.0,
            ..default()
        },
        TextStyle::Bold => TextFont {
            font_size: 20.0,
            weight: FontWeight::BOLD,
            ..default()
        },
        // Note: True italic requires a separate italic font asset.
        // Using default weight as a fallback until an italic font is loaded.
        TextStyle::Italic => TextFont {
            font_size: 20.0,
            ..default()
        },
        TextStyle::BoldItalic => TextFont {
            font_size: 20.0,
            weight: FontWeight::BOLD,
            ..default()
        },
    }
}

/// Reads Space/Enter to advance or dismiss the dialog.
///
/// If the typewriter is still animating, completes it instantly.
/// If the text is fully revealed, dismisses the dialog by despawning all
/// `DialogBox` entities and removing the `DialogState` resource.
pub fn handle_dialog_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut dialog_state: Option<ResMut<DialogState>>,
    dialog_entities: Query<Entity, With<DialogBox>>,
    mut text_query: Query<&mut Text, With<DialogTextNode>>,
    mut span_query: Query<&mut TextSpan>,
    children_query: Query<&Children, With<DialogTextNode>>,
    mut commands: Commands,
) {
    // Only respond to Space or Enter
    if !keyboard.just_pressed(KeyCode::Space) && !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }

    let Some(ref mut state) = dialog_state else {
        return;
    };

    if !state.fully_revealed {
        // Complete the typewriter effect instantly
        state.chars_revealed = state.total_chars;
        state.fully_revealed = true;

        // Reveal all text across spans using markup segments
        let segments = parse_markup(&state.full_text);
        reveal_text_spans(
            state.total_chars,
            &segments,
            &mut text_query,
            &mut span_query,
            &children_query,
        );
    } else {
        // Dismiss the dialog: despawn all DialogBox entities and remove DialogState
        for entity in dialog_entities.iter() {
            commands.entity(entity).despawn();
        }
        commands.remove_resource::<DialogState>();
    }
}

/// Advances the typewriter effect each frame.
///
/// Each frame, increments elapsed time, recomputes the number of visible characters,
/// and updates the `Text` component and `TextSpan` children on the `DialogTextNode`
/// entity to show only the revealed portion of the full text.
pub fn update_dialog_typewriter(
    time: Res<Time>,
    mut dialog_state: Option<ResMut<DialogState>>,
    mut text_query: Query<&mut Text, With<DialogTextNode>>,
    mut span_query: Query<&mut TextSpan>,
    children_query: Query<&Children, With<DialogTextNode>>,
) {
    let Some(ref mut state) = dialog_state else {
        return;
    };

    // Handle instant reveal when text_speed <= 0
    if state.text_speed <= 0.0 {
        state.chars_revealed = state.total_chars;
        state.fully_revealed = true;
    } else {
        // Advance elapsed time
        state.elapsed += time.delta_secs();

        // Compute visible characters using the pure function
        state.chars_revealed =
            compute_visible_chars(state.elapsed, state.text_speed, state.total_chars);
        state.fully_revealed = state.chars_revealed >= state.total_chars;
    }

    // Parse markup and distribute visible characters across spans
    let segments = parse_markup(&state.full_text);
    reveal_text_spans(
        state.chars_revealed,
        &segments,
        &mut text_query,
        &mut span_query,
        &children_query,
    );
}

/// Distributes `chars_revealed` across the parent `Text` and child `TextSpan` entities.
///
/// The first segment maps to the `Text` component on the `DialogTextNode` entity.
/// Subsequent segments map to `TextSpan` children in order.
fn reveal_text_spans(
    chars_revealed: usize,
    segments: &[crate::markup::TextSegment],
    text_query: &mut Query<&mut Text, With<DialogTextNode>>,
    span_query: &mut Query<&mut TextSpan>,
    children_query: &Query<&Children, With<DialogTextNode>>,
) {
    let mut remaining = chars_revealed;

    for mut text in text_query.iter_mut() {
        // First segment goes into the parent Text component
        if let Some(first_seg) = segments.first() {
            let seg_chars = first_seg.text.chars().count();
            let visible = remaining.min(seg_chars);
            let visible_text: String = first_seg.text.chars().take(visible).collect();
            **text = visible_text;
            remaining = remaining.saturating_sub(seg_chars);
        } else {
            **text = String::new();
        }
    }

    // Update TextSpan children for remaining segments
    for children in children_query.iter() {
        for (i, child) in children.iter().enumerate() {
            let seg_index = i + 1; // segments[0] is the parent Text
            if let Some(segment) = segments.get(seg_index)
                && let Ok(mut span) = span_query.get_mut(child)
            {
                let seg_chars = segment.text.chars().count();
                let visible = remaining.min(seg_chars);
                let visible_text: String = segment.text.chars().take(visible).collect();
                span.0 = visible_text;
                remaining = remaining.saturating_sub(seg_chars);
            }
        }
    }
}

/// Detects whether the dialog text overflows the visible area of the `DialogPanel`
/// and spawns or despawns an `OverflowIndicator` entity accordingly.
///
/// Uses a character-count heuristic: estimates characters per line based on panel width
/// and font size, then estimates visible lines based on panel height and font size.
/// If the total character count exceeds the estimated capacity, an overflow indicator
/// ("▼") is spawned at the bottom-right of the panel.
pub fn detect_overflow(
    dialog_state: Option<Res<DialogState>>,
    panel_query: Query<Entity, With<DialogPanel>>,
    indicator_query: Query<Entity, With<OverflowIndicator>>,
    mut commands: Commands,
) {
    let Some(state) = dialog_state else {
        // No active dialog — despawn any leftover indicators
        for entity in indicator_query.iter() {
            commands.entity(entity).despawn();
        }
        return;
    };

    let Ok(panel_entity) = panel_query.single() else {
        return;
    };

    // Heuristic constants based on the dialog panel configuration:
    // - Panel height: 120px
    // - Padding: 16px top + 16px bottom = 32px
    // - Usable height: 120 - 32 = 88px
    // - Font size: 20px, line height ~1.2 → ~24px per line
    // - Estimated visible lines: 88 / 24 ≈ 3 lines
    // - Panel width: 80% of screen, minus padding (32px horizontal)
    //   Assume ~800px effective width at a typical resolution
    //   At font size 20px, average character width ~10px
    //   Estimated chars per line: 800 / 10 = ~80 chars
    const ESTIMATED_CHARS_PER_LINE: usize = 80;
    const ESTIMATED_VISIBLE_LINES: usize = 3;
    const ESTIMATED_CAPACITY: usize = ESTIMATED_CHARS_PER_LINE * ESTIMATED_VISIBLE_LINES;

    let text_length = state.full_text.chars().count();
    let overflows = text_length > ESTIMATED_CAPACITY;

    if overflows {
        // Only spawn if no indicator already exists
        if indicator_query.is_empty() {
            // Spawn the overflow indicator as a child of the dialog panel
            commands.entity(panel_entity).with_children(|panel| {
                panel.spawn((
                    OverflowIndicator,
                    Text::new("▼".to_string()),
                    TextColor(Color::WHITE),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(4.0),
                        right: Val::Px(8.0),
                        ..default()
                    },
                ));
            });
        }
    } else {
        // Text fits — despawn any existing overflow indicator
        for entity in indicator_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog::{
        DialogConfig, DialogPanel, DialogText, DialogTextRegistry, FacePortrait, OverflowIndicator,
    };
    use crate::events::ShowDialog;
    use bevy::app::App;
    use bevy::asset::AssetPlugin;
    use bevy::ecs::system::RunSystemOnce;

    /// Creates a minimal Bevy app configured for dialog system testing.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.init_asset::<Image>();
        app.add_message::<ShowDialog>();
        app.init_resource::<DialogTextRegistry>();
        app.add_systems(
            Update,
            (
                handle_dialog_event,
                detect_overflow.after(handle_dialog_event),
            ),
        );
        app
    }

    /// Helper: sends a ShowDialog message via a one-shot system and runs one update.
    fn send_show_dialog(app: &mut App, text: &str, config: DialogConfig) {
        let text_owned = text.to_string();
        let system = move |mut writer: MessageWriter<ShowDialog>| {
            writer.write(ShowDialog {
                text: DialogText::Inline(text_owned.clone()),
                config: config.clone(),
            });
        };
        app.world_mut().run_system_once(system).unwrap();
        app.update();
    }

    // =========================================================================
    // 9.1 Test fixed height and border on standard dialog
    // =========================================================================

    #[test]
    fn standard_dialog_panel_has_fixed_height() {
        let mut app = test_app();
        send_show_dialog(&mut app, "Hello world", DialogConfig::default());

        let mut query = app.world_mut().query_filtered::<&Node, With<DialogPanel>>();
        let panels: Vec<&Node> = query.iter(app.world()).collect();
        assert_eq!(panels.len(), 1, "Expected exactly one DialogPanel");

        let node = panels[0];
        assert_eq!(
            node.height,
            Val::Px(120.0),
            "DialogPanel should have fixed height of 120px"
        );
    }

    #[test]
    fn standard_dialog_panel_has_border() {
        let mut app = test_app();
        send_show_dialog(&mut app, "Hello world", DialogConfig::default());

        let mut query = app.world_mut().query_filtered::<&Node, With<DialogPanel>>();
        let panels: Vec<&Node> = query.iter(app.world()).collect();
        assert_eq!(panels.len(), 1);

        let node = panels[0];
        assert_eq!(
            node.border,
            UiRect::all(Val::Px(2.0)),
            "DialogPanel should have 2px border on all sides"
        );
    }

    #[test]
    fn standard_dialog_panel_has_overflow_clip() {
        let mut app = test_app();
        send_show_dialog(&mut app, "Hello world", DialogConfig::default());

        let mut query = app.world_mut().query_filtered::<&Node, With<DialogPanel>>();
        let panels: Vec<&Node> = query.iter(app.world()).collect();
        assert_eq!(panels.len(), 1);

        let node = panels[0];
        assert_eq!(
            node.overflow,
            Overflow::clip(),
            "DialogPanel should clip overflow"
        );
    }

    #[test]
    fn standard_dialog_panel_has_background_and_border_color() {
        let mut app = test_app();
        send_show_dialog(&mut app, "Hello world", DialogConfig::default());

        let mut query = app
            .world_mut()
            .query_filtered::<(&BackgroundColor, &BorderColor), With<DialogPanel>>();
        let results: Vec<(&BackgroundColor, &BorderColor)> = query.iter(app.world()).collect();
        assert_eq!(results.len(), 1);

        let (bg, border_color) = results[0];
        // Standard dialog has semi-transparent black background
        assert_eq!(bg.0, Color::srgba(0.0, 0.0, 0.0, 0.75));
        // Standard dialog has light gray border on all sides
        let expected_border_color = Color::srgba(0.85, 0.85, 0.85, 1.0);
        assert_eq!(border_color.top, expected_border_color);
        assert_eq!(border_color.right, expected_border_color);
        assert_eq!(border_color.bottom, expected_border_color);
        assert_eq!(border_color.left, expected_border_color);
    }

    // =========================================================================
    // 9.2 Test attribute dialog mode
    // =========================================================================

    #[test]
    fn attribute_dialog_has_transparent_background() {
        let mut app = test_app();
        let config = DialogConfig {
            attribute_dialog: true,
            ..Default::default()
        };
        send_show_dialog(&mut app, "Attribute text", config);

        let mut query = app
            .world_mut()
            .query_filtered::<&BackgroundColor, With<DialogPanel>>();
        let results: Vec<&BackgroundColor> = query.iter(app.world()).collect();
        assert_eq!(results.len(), 1);

        let bg = results[0];
        assert_eq!(
            bg.0,
            Color::NONE,
            "Attribute dialog should have transparent background"
        );
    }

    #[test]
    fn attribute_dialog_has_zero_border() {
        let mut app = test_app();
        let config = DialogConfig {
            attribute_dialog: true,
            ..Default::default()
        };
        send_show_dialog(&mut app, "Attribute text", config);

        let mut query = app
            .world_mut()
            .query_filtered::<(&Node, &BorderColor), With<DialogPanel>>();
        let results: Vec<(&Node, &BorderColor)> = query.iter(app.world()).collect();
        assert_eq!(results.len(), 1);

        let (node, border_color) = results[0];
        assert_eq!(
            node.border,
            UiRect::ZERO,
            "Attribute dialog should have zero border"
        );
        assert_eq!(
            border_color.top,
            Color::NONE,
            "Attribute dialog border top should be transparent"
        );
        assert_eq!(
            border_color.right,
            Color::NONE,
            "Attribute dialog border right should be transparent"
        );
        assert_eq!(
            border_color.bottom,
            Color::NONE,
            "Attribute dialog border bottom should be transparent"
        );
        assert_eq!(
            border_color.left,
            Color::NONE,
            "Attribute dialog border left should be transparent"
        );
    }

    // =========================================================================
    // 9.3 Test face portrait spawning
    // =========================================================================

    #[test]
    fn face_portrait_spawned_when_configured() {
        let mut app = test_app();

        // Insert a RendererProjectData with the portrait ID registered
        let mut face_portraits = std::collections::HashMap::new();
        face_portraits.insert(
            "portraits/hero.png".to_string(),
            "portraits/hero.png".to_string(),
        );
        let project_file = rpg_toolkit_common::ProjectFile::new(
            std::collections::HashMap::new(),
            std::collections::HashMap::new(),
            None,
            std::collections::HashMap::new(),
            None,
            std::collections::HashMap::new(),
            face_portraits,
            rpg_toolkit_common::CharacterRegistry::default(),
        );
        app.insert_resource(crate::resources::RendererProjectData {
            project_file,
            tileset_textures: std::collections::HashMap::new(),
            tileset_atlas_layouts: std::collections::HashMap::new(),
            spritesheet_textures: std::collections::HashMap::new(),
            spritesheet_atlas_layouts: std::collections::HashMap::new(),
        });

        let config = DialogConfig {
            face_portrait: Some("portraits/hero.png".to_string()),
            ..Default::default()
        };
        send_show_dialog(&mut app, "Hello hero", config);

        let mut query = app
            .world_mut()
            .query_filtered::<&Node, With<FacePortrait>>();
        let portraits: Vec<&Node> = query.iter(app.world()).collect();
        assert_eq!(
            portraits.len(),
            1,
            "FacePortrait entity should be spawned when face_portrait is Some"
        );

        let node = portraits[0];
        assert_eq!(node.width, Val::Px(64.0), "Portrait should be 64px wide");
        assert_eq!(node.height, Val::Px(64.0), "Portrait should be 64px tall");
    }

    #[test]
    fn no_face_portrait_when_none() {
        let mut app = test_app();
        let config = DialogConfig {
            face_portrait: None,
            ..Default::default()
        };
        send_show_dialog(&mut app, "Hello world", config);

        let mut query = app
            .world_mut()
            .query_filtered::<&Node, With<FacePortrait>>();
        let portraits: Vec<&Node> = query.iter(app.world()).collect();
        assert_eq!(
            portraits.len(),
            0,
            "No FacePortrait entity should exist when face_portrait is None"
        );
    }

    // =========================================================================
    // 9.4 Test overflow indicator logic
    // =========================================================================

    #[test]
    fn overflow_indicator_appears_for_long_text() {
        let mut app = test_app();
        // ESTIMATED_CAPACITY = 80 * 3 = 240 chars. Create text longer than that.
        let long_text = "a".repeat(300);
        send_show_dialog(&mut app, &long_text, DialogConfig::default());

        // Run another update to let detect_overflow process the state
        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<&Text, With<OverflowIndicator>>();
        let indicators: Vec<&Text> = query.iter(app.world()).collect();
        assert_eq!(
            indicators.len(),
            1,
            "OverflowIndicator should be spawned for text exceeding capacity"
        );
    }

    #[test]
    fn no_overflow_indicator_for_short_text() {
        let mut app = test_app();
        // Short text well under 240 chars
        send_show_dialog(&mut app, "Short text", DialogConfig::default());

        // Run another update to let detect_overflow process the state
        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<&Text, With<OverflowIndicator>>();
        let indicators: Vec<&Text> = query.iter(app.world()).collect();
        assert_eq!(
            indicators.len(),
            0,
            "No OverflowIndicator should exist for short text"
        );
    }
}
