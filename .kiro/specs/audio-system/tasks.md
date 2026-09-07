# Implementation Plan: Audio System

## Overview

This plan implements the audio system across the three workspace crates in dependency
order: `rpg-toolkit-common` data models and validation first, then AssetManager /
manifest / project integration, then the `AudioLoader` utility, then renderer playback,
and finally editor UI. Each task builds on the previous ones and ends by wiring the new
capability into an existing integration point (registry, manifest, project round-trip,
action queue, or editor panel).

Property-based tests (proptest, minimum 100 cases) cover the 22 correctness properties
defined in the design. They live in `crates/rpg-toolkit-common/tests/properties/` (with
matching `[[test]]` entries in `Cargo.toml`), in `crates/rpg-toolkit-renderer/tests/properties/`
for the queue-classification and music-decision properties, and with the editor crate's
tests for the clamp property. Time-based renderer playback (cross-fades, loop counting,
concurrent one-shots) is covered by integration tests per the design's Testing Strategy.

Language: Rust (per design). Tests use `proptest`, tagged with
`// Feature: audio-system, Property {n}: {text}` and `**Validates: Requirements X.Y**`.

## Tasks

- [x] 1. Add audio asset data models and category constants (rpg-toolkit-common)
  - [x] 1.1 Define `MusicLoop`, `SoundEffect`, id type aliases, and category constants
    - In `crates/rpg-toolkit-common/src/asset.rs`, add `pub type MusicLoopId = String;` and `pub type SoundEffectId = String;`
    - Add `pub const CATEGORY_MUSIC_LOOP: &str = "music_loop";` and `pub const CATEGORY_SOUND_EFFECT: &str = "sound_effect";`
    - Define `MusicLoop` and `SoundEffect` structs with `id`, `relative_path`, `category` fields, deriving `Serialize` and using a `RawAudioAsset` helper with `#[serde(try_from = "RawAudioAsset")]`
    - Implement `TryFrom<RawAudioAsset>` validation following the `ParallaxLayer`/`ChoiceData` pattern: `id` 1..=128 chars; `MusicLoop.relative_path` non-empty after trim; `SoundEffect.relative_path` 1..=4096 chars; coerce `category` to the correct constant
    - Add `impl From<&MusicLoop> for AssetReference` and `impl From<&SoundEffect> for AssetReference` producing the registry record with the correct category
    - _Requirements: 1.1, 1.4, 1.5, 1.6, 1.7, 1.8, 2.1, 2.4, 2.5, 2.6, 2.7, 2.8, 3.1, 3.2_

  - [ ]* 1.2 Write property test: audio asset registration stores correct category
    - **Property 1: Audio asset registration stores the correct category and succeeds**
    - **Validates: Requirements 1.2, 2.2**
    - File `tests/properties/audio_asset_registration.rs` with a `[[test]]` entry in Cargo.toml

  - [ ]* 1.3 Write property test: duplicate audio identifiers rejected, existing entry unchanged
    - **Property 2: Duplicate audio identifiers are rejected and leave the existing entry unchanged**
    - **Validates: Requirements 1.3, 2.3**
    - File `tests/properties/audio_asset_duplicate_id.rs` with a `[[test]]` entry in Cargo.toml

  - [ ]* 1.4 Write property test: MusicLoop validation accept/reject
    - **Property 3: MusicLoop validation accepts valid values and rejects invalid ones**
    - **Validates: Requirements 1.4, 1.5, 1.6, 1.7**
    - Boundary-focused id strategy (lengths 0, 1, 128, 129) and empty/whitespace paths; file `tests/properties/music_loop_validation.rs` with a `[[test]]` entry in Cargo.toml

  - [ ]* 1.5 Write property test: SoundEffect validation accept/reject
    - **Property 4: SoundEffect validation accepts valid values and rejects invalid ones**
    - **Validates: Requirements 2.4, 2.5, 2.6, 2.7**
    - Boundary-focused strategies for id (0/1/128/129) and path (0/1/4096/4097); file `tests/properties/sound_effect_validation.rs` with a `[[test]]` entry in Cargo.toml

  - [x] 1.6 Write property test: audio asset serialization round-trip
    - **Property 5: Audio asset serialization round-trip**
    - **Validates: Requirements 1.8, 1.9, 2.8, 2.9**
    - File `tests/properties/audio_asset_round_trip.rs` with a `[[test]]` entry in Cargo.toml

- [x] 2. Wire audio categories into AssetManager (rpg-toolkit-common)
  - [x] 2.1 Add default category→subdirectory mappings for audio
    - In `AssetManager::new()` in `asset.rs`, insert `CATEGORY_MUSIC_LOOP → "audio/music/"` and `CATEGORY_SOUND_EFFECT → "audio/sfx/"` into `category_dirs`
    - Add unit tests asserting the two default mappings and the two category constant string values
    - _Requirements: 3.3, 3.4_

  - [ ]* 2.2 Write property test: audio files placed under category subdirectory on save
    - **Property 10: AssetManager places audio files under their category subdirectory on save**
    - **Validates: Requirements 3.5**
    - Registers audio assets with existing temp source files, saves to a directory target, asserts destination path; file `tests/properties/audio_save_category_dir.rs` with a `[[test]]` entry in Cargo.toml

  - [ ]* 2.3 Write property test: validation reports exactly the missing audio assets
    - **Property 11: AssetManager validation reports exactly the missing audio assets**
    - **Validates: Requirements 3.8**
    - File `tests/properties/audio_validation_missing.rs` with a `[[test]]` entry in Cargo.toml

  - [x] 2.4 Write unit tests for save-to-ZIP audio placement and missing-source warning
    - Assert audio entries written at normalized relative path in ZIP; assert one `AssetWarning` per missing source with remaining assets still processed
    - _Requirements: 3.6, 3.7_

- [x] 3. Add `default_music_loop` to MapData (rpg-toolkit-common)
  - [x] 3.1 Add the field and its optional validating deserializer
    - In `crates/rpg-toolkit-common/src/map.rs`, add `#[serde(default, deserialize_with = "deserialize_optional_music_loop_id")] pub default_music_loop: Option<MusicLoopId>` to `MapData`
    - Implement `deserialize_optional_music_loop_id` (None when absent; when present validate 1..=128 chars, reject empty and >128)
    - Set `default_music_loop: None` in `MapData::new`
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

  - [x] 3.2 Write property test: MapData default_music_loop round-trip
    - **Property 6: MapData default_music_loop round-trip**
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.6, 4.7**
    - File `tests/properties/map_default_music_loop_round_trip.rs` with a `[[test]]` entry in Cargo.toml

  - [ ]* 3.3 Write property test: invalid default_music_loop rejected
    - **Property 7: Invalid default_music_loop is rejected**
    - **Validates: Requirements 4.4, 4.5**
    - Generates map JSON with empty or >128-char `default_music_loop`; file `tests/properties/map_default_music_loop_invalid.rs` with a `[[test]]` entry in Cargo.toml

- [x] 4. Add PlayMusic and PlaySoundEffect EventAction variants (rpg-toolkit-common)
  - [x] 4.1 Add the two variants and their field validators
    - In `map.rs`, add `PlayMusic { music_loop_id, fade_duration, loop_count, fade_out_duration }` and `PlaySoundEffect { sound_effect_id, volume }` to the `#[serde(tag = "type")]` `EventAction` enum
    - Add `deserialize_music_loop_id` / `deserialize_sound_effect_id` (1..=128, reject empty/>128); `deserialize_fade_seconds` (finite 0.0..=10.0, default 0.0); `deserialize_optional_loop_count` (None when absent, reject 0, accept >=1); `deserialize_volume` + `default_volume` (finite 0.0..=1.0, default 1.0)
    - Ensure `sound_effect_id` has no default so an absent field yields a missing-field error
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9, 5.10, 5.11, 5.12, 5.13, 5.14, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8, 7.9_

  - [x] 4.2 Write property test: audio EventAction round-trip and correct type tag
    - **Property 8: Audio EventAction round-trip**
    - **Validates: Requirements 5.1, 5.2, 5.5, 5.6, 5.8, 5.9, 5.11, 5.12, 5.14, 5.15, 7.1, 7.2, 7.6, 7.7, 7.9, 7.10**
    - File `tests/properties/audio_event_action_round_trip.rs` with a `[[test]]` entry in Cargo.toml

  - [ ]* 4.3 Write property test: invalid audio EventAction fields rejected
    - **Property 9: Invalid audio EventAction fields are rejected**
    - **Validates: Requirements 5.3, 5.4, 5.7, 5.10, 5.13, 7.3, 7.4, 7.8**
    - Boundary strategies: id 0/129 chars; floats NaN/±inf and out of range; `loop_count` 0; file `tests/properties/audio_event_action_invalid.rs` with a `[[test]]` entry in Cargo.toml

  - [ ]* 4.4 Write property test: unknown EventAction type tag error names the tag
    - **Property 18: Unknown EventAction type tag produces an error naming the tag**
    - **Validates: Requirements 11.5**
    - File `tests/properties/event_action_unknown_tag.rs` with a `[[test]]` entry in Cargo.toml

  - [ ]* 4.5 Write unit tests for absent-field defaults
    - Assert `fade_duration`/`fade_out_duration` default `0.0`, `loop_count` default `None`, `volume` default `1.0`, missing `sound_effect_id` errors
    - _Requirements: 5.5, 5.8, 5.11, 7.5, 7.6_

- [x] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Add audio registries to ProjectManifest and ProjectFile (rpg-toolkit-common)
  - [x] 6.1 Add `music_loops` / `sound_effects` fields and thread them through construction
    - In `manifest.rs` and `project.rs`, add `#[serde(default)] pub music_loops: HashMap<MusicLoopId, MusicLoop>` and `#[serde(default)] pub sound_effects: HashMap<SoundEffectId, SoundEffect>`
    - Update `ProjectFile::new`, `to_manifest`, `ProjectManifest::into_project_file`, and any construction sites to carry both registries
    - _Requirements: 10.1, 10.2, 10.3, 11.1, 11.2, 11.7_

  - [x] 6.2 Add key/id-mismatch validation for audio registries
    - In `ProjectManifest::into_project_file` (and `ProjectFile` deserialization validation), alongside existing registry checks, return `CommonError::ProjectValidationError` naming the registry and offending key when a `music_loops`/`sound_effects` key differs from the record's `id`, aborting the load
    - _Requirements: 10.4_

  - [x] 6.3 Add unregistered `default_music_loop` load warning
    - After maps and registries load, iterate maps whose `default_music_loop` is `Some(id)` not present in `music_loops`; `warn!`/`eprintln!` naming the map and missing id (mirroring the existing `JumpTo` unknown-map warning), then continue loading
    - _Requirements: 4.8_

  - [ ]* 6.4 Write property test: manifest rejects audio-registry key/id mismatches
    - **Property 16: Manifest rejects audio-registry key/id mismatches**
    - **Validates: Requirements 10.4**
    - File `tests/properties/manifest_audio_key_mismatch.rs` with a `[[test]]` entry in Cargo.toml

  - [ ]* 6.5 Write property test: manifest audio-registry round-trip is order-independent
    - **Property 17: Manifest audio-registry round-trip is order-independent**
    - **Validates: Requirements 10.1, 10.2, 10.5**
    - File `tests/properties/manifest_audio_round_trip.rs` with a `[[test]]` entry in Cargo.toml

  - [ ]* 6.6 Write property test: comprehensive project round-trip
    - **Property 19: Comprehensive project round-trip**
    - **Validates: Requirements 11.7**
    - Generates `ProjectFile`s with audio EventActions, `default_music_loop` None/registered, and audio registry entries; file `tests/properties/project_audio_round_trip.rs` with a `[[test]]` entry in Cargo.toml

  - [x] 6.7 Write unit tests for backward compatibility
    - Pre-audio manifest/project loads with empty audio registries and `None` map default; file with only pre-existing action tags loads without error; malformed JSON errors
    - _Requirements: 10.3, 11.1, 11.2, 11.3, 11.4, 11.6_

- [x] 7. Implement the AudioLoader utility (rpg-toolkit-common)
  - [x] 7.1 Create the `audio` module with `AudioFormat` and `AudioLoader`
    - Create `crates/rpg-toolkit-common/src/audio.rs`; declare it in `lib.rs`
    - Define `pub enum AudioFormat { Ogg, Wav, Mp3 }` and `pub struct AudioLoader`
    - Implement `classify_format` (case-insensitive extension; no extension → "could not determine audio format"; unsupported → error naming the extension)
    - Implement `load` sequencing: trim/empty guard → `AssetManager::resolve_path` (rejects `..` without reading) → classify → `AssetManager::load_file_bytes` (non-existent / directory / read-failure), returning `(AudioFormat, Vec<u8>)` on success
    - Add `AudioLoadError(String)` to `CommonError` (or reuse `AssetPathError`) for format messages
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 9.8, 9.9, 9.10_

  - [ ]* 7.2 Write property test: AudioLoader byte round-trip
    - **Property 12: AudioLoader byte round-trip**
    - **Validates: Requirements 9.1, 9.2**
    - File `tests/properties/audio_loader_bytes.rs` with a `[[test]]` entry in Cargo.toml

  - [ ]* 7.3 Write property test: AudioLoader recognizes supported formats case-insensitively
    - **Property 13: AudioLoader recognizes supported formats case-insensitively**
    - **Validates: Requirements 9.7**
    - Mixed-case `ogg`/`wav`/`mp3` extensions; file `tests/properties/audio_loader_formats.rs` with a `[[test]]` entry in Cargo.toml

  - [ ]* 7.4 Write property test: AudioLoader rejects unrecognizable/unsupported extensions
    - **Property 14: AudioLoader rejects unrecognizable or unsupported extensions**
    - **Validates: Requirements 9.8, 9.9**
    - File `tests/properties/audio_loader_unsupported_ext.rs` with a `[[test]]` entry in Cargo.toml

  - [ ]* 7.5 Write property test: AudioLoader rejects empty and traversing paths without reading
    - **Property 15: AudioLoader rejects empty and traversing paths without reading a file**
    - **Validates: Requirements 9.3, 9.4**
    - File `tests/properties/audio_loader_bad_paths.rs` with a `[[test]]` entry in Cargo.toml

  - [x] 7.6 Write unit tests for AudioLoader directory and unreadable-file errors
    - Directory path → not-a-regular-file error; unreadable existing file → read error (where platform-supported)
    - _Requirements: 9.6, 9.10_

- [x] 8. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 9. Implement renderer audio playback (rpg-toolkit-renderer)
  - [x] 9.1 Add audio resources, components, and the pure music-decision function
    - In `resources.rs`, add `MusicChannelState`, `MusicPlayback`, `FadeRamp`, `FadingTrack`, and marker components `MusicChannel` / `SoundEffectChannel`
    - Add a pure `next_music_command(state, request, registered) -> MusicCommand` function returning "no change" when the requested id equals the current playing/fading-in id or is unregistered, otherwise the start/cross-fade/fade-in decision
    - _Requirements: 6.1, 6.3, 6.6_

  - [x] 9.2 Implement PlayMusic / PlaySoundEffect handling in the action queue (non-blocking)
    - Create `systems/audio.rs`; handle `PlayMusic` in `advance_action_queue`: look up id in `project_file.music_loops` (warn + unchanged if missing), apply `next_music_command`, start/stop/cross-fade/fade-in per `fade_duration`, record `loop_count`/`fade_out_duration` bookkeeping; pop and continue (non-blocking)
    - Handle `PlaySoundEffect`: look up id (warn + advance if missing), load/decode via the audio asset path through `AudioLoader` (warn + advance on failure), spawn a one-shot `SoundEffectChannel` at `volume` playing once concurrently; never touch the music channel; pop and continue
    - _Requirements: 6.2, 6.4, 6.5, 6.6, 6.7, 6.11, 8.1, 8.2, 8.3, 8.4, 8.5, 8.6_

  - [x] 9.3 Implement the fade-advancement system and loop counting
    - Add `update_music_fades` advancing `FadeRamp`s each frame via `AudioSink::set_volume`, completing cross-fades/fade-ins/fade-outs independently of the queue; count loop completions for finite `loop_count`, and on the Nth completion stop (0.0 fade-out) or fade to zero over `fade_out_duration`, leaving the channel silent without resuming the map default
    - _Requirements: 6.8, 6.9, 6.10, 6.12_

  - [x] 9.4 Wire map-entry default music and register systems in the renderer plugin
    - On `MapChanged`/map-load, issue the internal default-music play equivalent to `PlayMusic { default, 0.0, None, 0.0 }` when `default_music_loop` is `Some`
    - Register `MusicChannelState`, the audio systems, and startup in the renderer plugin
    - _Requirements: 4.8 (runtime), 6.1, 6.2_

  - [ ]* 9.5 Write property test: music channel unchanged for same-id/unregistered request
    - **Property 20: Music channel is unchanged for a same-id or unregistered PlayMusic request**
    - **Validates: Requirements 6.3, 6.6**
    - Tests the pure `next_music_command` decision function; file `crates/rpg-toolkit-renderer/tests/properties/music_channel_no_change.rs` with a `[[test]]` entry in the renderer Cargo.toml

  - [ ]* 9.6 Write property test: audio event actions are non-blocking
    - **Property 21: Audio event actions are non-blocking**
    - **Validates: Requirements 6.11, 8.4**
    - Queue-classification property alongside `intro_blocking_classification.rs`; file `crates/rpg-toolkit-renderer/tests/properties/audio_action_non_blocking.rs` with a `[[test]]` entry in the renderer Cargo.toml

  - [ ]* 9.7 Write integration tests for time-based playback
    - Differing `PlayMusic` starts/cross-fades/fades-in; finite `loop_count` plays exactly N times then goes silent (with/without `fade_out_duration`) without resuming map default; fades complete independent of the queue; sound effects play once, concurrently, at requested volume without disturbing the music channel
    - _Requirements: 6.1, 6.2, 6.4, 6.5, 6.7, 6.8, 6.9, 6.10, 6.12, 8.1, 8.2, 8.6_

- [x] 10. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 11. Implement editor audio support (rpg-toolkit-editor)
  - [x] 11.1 Add audio asset registration UI
    - In `project_settings_panel.rs` (or a new `audio_panel.rs`): file picker + id text field; on submit validate id length/non-empty and duplicate against the registry, register a `MusicLoop`/`SoundEffect`, add to the manifest registry, and display in a list; reject invalid/duplicate ids with an error indication leaving existing assets unchanged
    - _Requirements: 12.1, 12.2, 12.11_

  - [x] 11.2 Add the map default music loop selector
    - In the map properties UI, add a combobox (reusing `searchable_combobox`) populated with each registered music loop plus a "None" option, preselected to "None" when `default_music_loop` is `None`
    - _Requirements: 12.3_

  - [x] 11.3 Add PlayMusic / PlaySoundEffect to the Event Trigger Editor with clamping
    - In `attribute/action_editor_forms.rs` + `action_editor_ui.rs`: add both variants to the action list; `PlayMusic` form (music loop selector; `fade_duration` init 0.0 range 0.0..=10.0; `loop_count` defaulting to infinite/None else integer >=1; `fade_out_duration` init 0.0 range 0.0..=10.0); `PlaySoundEffect` form (sound effect selector; `volume` init 1.0 range 0.0..=1.0)
    - Disable Add/Update while the required id selection is empty
    - Implement save-time clamping: `fade_duration`/`fade_out_duration` to `[0.0, 10.0]`, finite `loop_count < 1` to `1`, `volume` to `[0.0, 1.0]`
    - _Requirements: 12.4, 12.5, 12.6, 12.7, 12.8, 12.9, 12.10_

  - [ ]* 11.4 Write property test: editor clamps out-of-range numeric inputs
    - **Property 22: Editor clamps out-of-range numeric inputs**
    - **Validates: Requirements 12.8, 12.9, 12.10**
    - Tests the pure clamp function for `fade_duration`/`fade_out_duration`/`volume`/`loop_count`; file placed with the editor crate's tests with a matching `[[test]]` entry in the editor Cargo.toml

- [x] 12. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 13. Implement the Audio database page (rpg-toolkit-editor)
  - [x] 13.1 Add the `Audio` app editor mode and navigation entry
    - In `crates/rpg-toolkit-editor/src/data/state.rs`, add an `Audio` variant to `AppEditorMode`
    - Add the Audio entry to the mode-switching UI (wherever Map/Character/Item/Ability/Enemy/Shop are selected) so the page is reachable
    - _Requirements: 13.1_

  - [x] 13.2 Create the Audio page plugin, state, and two-list layout
    - Add `crates/rpg-toolkit-editor/src/plugins/audio_page.rs` with `AudioPagePlugin`, an `AudioPageState` resource, and an egui system in `EguiPrimaryContextPass` under `EditorUiSet::Panels` gated on `resource_equals(AppEditorMode::Audio)` (mirror `AbilityPanelPlugin`); register the plugin in the editor app and declare the module in `plugins/mod.rs`
    - Render music-loop and sound-effect lists (`id — relative_path`) with selection, and an empty-state label when a registry is empty
    - _Requirements: 13.1, 13.2, 13.3_

  - [x] 13.3 Add registration controls to the Audio page
    - Reuse the file-picker + id-field + Register flow and the `music_relative_path` / `sfx_relative_path` / validation helpers from `audio_panel.rs` (extract shared helpers if needed); on success register the asset, refresh the list, and set `has_unsaved_audio_changes`; on invalid/duplicate id show an inline error and leave assets unchanged
    - _Requirements: 13.4, 13.5, 13.6, 13.12_

  - [x] 13.4 Add preview playback via bevy_audio
    - On selecting an asset, show Play/Stop controls; Play resolves the asset's `relative_path` against the current project root and plays it through Bevy audio (spawn an `AudioPlayer` entity with `PlaybackSettings::DESPAWN`), storing the `Entity` in `AudioPageState::preview_entity`; Stop and switching assets despawn the prior preview so at most one plays at a time; surface an inline error and leave assets unchanged when resolve/load/decode fails
    - _Requirements: 13.7, 13.8, 13.9, 13.10, 13.11_

## Notes

- Tasks marked with `*` are optional (tests) and can be skipped for a faster MVP.
- Each task references specific requirements for traceability.
- Property tests use `proptest` with a minimum of 100 cases, tagged `// Feature: audio-system, Property {n}: {text}` and annotated with `**Validates: Requirements X.Y**`, matching the existing conventions in `event_reward_actions.rs` / `asset_registry_round_trip.rs`.
- Common-crate property tests live under `crates/rpg-toolkit-common/tests/properties/` with matching `[[test]]` entries in that crate's `Cargo.toml`; renderer properties live under `crates/rpg-toolkit-renderer/tests/properties/`; the editor clamp property lives with the editor crate's tests.
- Time-based renderer playback is covered by integration tests (task 9.7) rather than property tests, per the design's Testing Strategy.
- Properties 1–19 attach to the common crate, Properties 20–21 to the renderer, and Property 22 to the editor.

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1"] },
    { "id": 1, "tasks": ["1.2", "1.3", "1.4", "1.5", "1.6", "2.1", "3.1", "4.1"] },
    { "id": 2, "tasks": ["2.2", "2.3", "2.4", "3.2", "3.3", "4.2", "4.3", "4.4", "4.5", "6.1"] },
    { "id": 3, "tasks": ["6.2", "6.3", "7.1"] },
    { "id": 4, "tasks": ["6.4", "6.5", "6.6", "6.7", "7.2", "7.3", "7.4", "7.5", "7.6", "9.1"] },
    { "id": 5, "tasks": ["9.2", "9.3", "9.5"] },
    { "id": 6, "tasks": ["9.4", "9.6", "9.7", "11.1", "11.2", "11.3"] },
    { "id": 7, "tasks": ["11.4"] },
    { "id": 8, "tasks": ["13.1"] },
    { "id": 9, "tasks": ["13.2", "13.3", "13.4"] }
  ]
}
```
