# Requirements Document

## Introduction

This feature adds audio capability to the RPG toolkit, which until now has focused on narrative events, inventory, and the project database. The audio system introduces two new asset types: **music loops** (continuously repeating background tracks) and **sound effects** (short, one-shot clips such as a door opening or a clash of battle). Music loops and sound effects are registered as project assets through the existing `AssetRegistry` conventions, using two new well-known asset categories.

Each map gains an association to a **default music loop** that plays when the player enters that map. Two new `EventAction` variants extend the existing event trigger system: **PlayMusic** overrides the currently playing music loop with a different one, and **PlaySoundEffect** triggers a one-shot sound effect tied to an action or event. Because nearly every scene requires audio, audio decoding and loading is defined as a shared **AudioLoader** utility in `rpg-toolkit-common`, mirroring the role that the `AssetManager` plays for image assets.

This requirements document defines the data models, validation rules, serialization behavior, map association, event actions, and the shared loading utility. Runtime playback behavior in the renderer and editor UI support are also specified so the feature integrates with existing systems consistently.

## Glossary

- **MusicLoop**: A project asset representing a continuously repeating background audio track, identified by a unique `MusicLoopId` and referencing an audio file by relative path.
- **SoundEffect**: A project asset representing a short, one-shot audio clip (for example a door opening or a battle clash), identified by a unique `SoundEffectId` and referencing an audio file by relative path.
- **MusicLoopId**: A `String` type alias identifying a music loop asset within the project. Format matches existing asset identifiers (1–128 characters).
- **SoundEffectId**: A `String` type alias identifying a sound effect asset within the project. Format matches existing asset identifiers (1–128 characters).
- **AssetRegistry**: The existing registry in `rpg-toolkit-common` that stores `AssetReference` records keyed by unique identifier and enforces the 1–128 character identifier rule.
- **AssetReference**: The existing record associating a logical asset identifier with a relative file path and an `AssetCategory`.
- **AssetCategory**: The existing open string set of asset classifications. This feature adds two well-known category constants: `music_loop` and `sound_effect`.
- **AssetManager**: The existing utility that resolves, validates, loads, and saves project assets and maps categories to subdirectories.
- **AudioLoader**: A new shared utility in `rpg-toolkit-common` that resolves an audio asset's relative path against a project root, validates the file, and loads its raw bytes for use by any scene.
- **EventAction**: The existing `#[serde(tag = "type")]` enum in `rpg-toolkit-common` representing a single step in a trigger sequence. This feature adds the `PlayMusic` and `PlaySoundEffect` variants.
- **PlayMusic**: A new `EventAction` variant that overrides the currently playing music loop with a specified music loop. It supports an optional `loop_count` (how many times the loop plays before stopping, defaulting to infinite repetition) and an optional `fade_out_duration` (a fade applied at the end of playback once the loop count is reached, useful for pairing with a visual fade to close a chapter).
- **PlaySoundEffect**: A new `EventAction` variant that plays a specified one-shot sound effect.
- **MapData**: The existing complete map data structure in `rpg-toolkit-common`. This feature adds an optional default music loop association.
- **ProjectManifest**: The existing on-disk manifest summarizing project contents. This feature adds registries for music loops and sound effects.
- **Renderer**: The `rpg-toolkit-renderer` crate responsible for running the game world, processing triggers, and producing audio and visual output.
- **Editor**: The `rpg-toolkit-editor` crate providing the map editing UI, including the Event Trigger Editor dialog and map property configuration.
- **AudioChannel**: The logical playback slot in the Renderer. This feature defines two channels: a single **music channel** that plays at most one music loop at a time, and a **sound effect channel** that plays one-shot effects.
- **Audio Page**: A dedicated top-level Editor page (a peer of the Character, Item, Ability, Enemy, and Shop pages, selected via `AppEditorMode`) for managing the project's registered music loops and sound effects, including registration, listing, and in-editor preview playback.

## Requirements

### Requirement 1: MusicLoop Asset Data Model

**User Story:** As a game designer, I want to register music loops as project assets, so that maps and events can reference background tracks by identifier.

#### Acceptance Criteria

1. THE MusicLoop data model SHALL include an `id` field of type `MusicLoopId` and a `relative_path` field of type `String`, where a `MusicLoopId` is a string of 1 to 128 characters inclusive.
2. WHEN a MusicLoop is registered in the AssetRegistry, THE AssetRegistry SHALL store the entry with `category` set to the well-known constant `music_loop`.
3. IF a MusicLoop is registered in the AssetRegistry with an `id` that is already present in the AssetRegistry, THEN THE AssetRegistry SHALL reject the registration and return an error indicating that the identifier is already registered, leaving the existing entry unchanged.
4. WHEN the `id` field is deserialized, THE MusicLoop parser SHALL accept string values whose character count is greater than or equal to 1 and less than or equal to 128.
5. IF the `id` field is empty or its character count exceeds 128 during deserialization, THEN THE MusicLoop parser SHALL return a deserialization error indicating that the identifier length is invalid, and SHALL NOT produce a MusicLoop value.
6. WHEN the `relative_path` field is deserialized, THE MusicLoop parser SHALL accept string values whose character count is greater than or equal to 1 after leading and trailing whitespace is removed.
7. IF the `relative_path` field is empty or contains only whitespace characters during deserialization, THEN THE MusicLoop parser SHALL return a deserialization error indicating that the relative path must not be empty, and SHALL NOT produce a MusicLoop value.
8. THE MusicLoop data model SHALL serialize to and deserialize from JSON using the serde format consistent with existing asset records.
9. FOR ALL MusicLoop values whose `id` is 1 to 128 characters inclusive and whose `relative_path` is a non-empty, non-whitespace-only string, serializing the value to JSON and then deserializing the result SHALL produce a value equal to the original (round-trip property).

### Requirement 2: SoundEffect Asset Data Model

**User Story:** As a game designer, I want to register sound effects as project assets, so that events can trigger discrete sounds by identifier.

#### Acceptance Criteria

1. THE SoundEffect data model SHALL include an `id` field of type `SoundEffectId` and a `relative_path` field of type `String`.
2. WHEN a SoundEffect with an `id` that is not already present in the AssetRegistry is registered, THE AssetRegistry SHALL store the entry with `category` set to the well-known constant `sound_effect` and SHALL return a success result.
3. IF a SoundEffect is registered with an `id` that is already present in the AssetRegistry, THEN THE AssetRegistry SHALL leave the existing entry unchanged and SHALL return an error indicating that the identifier is already registered.
4. WHEN the `id` field is deserialized, THE SoundEffect parser SHALL accept string values whose length in characters is between 1 and 128 inclusive.
5. IF the `id` field has a length of 0 characters or greater than 128 characters during deserialization, THEN THE SoundEffect parser SHALL reject the input and return a deserialization error indicating that the identifier length is invalid.
6. WHEN the `relative_path` field is deserialized, THE SoundEffect parser SHALL accept string values whose length in characters is between 1 and 4096 inclusive.
7. IF the `relative_path` field has a length of 0 characters or greater than 4096 characters during deserialization, THEN THE SoundEffect parser SHALL reject the input and return a deserialization error indicating that the relative path length is invalid.
8. THE SoundEffect data model SHALL serialize to and deserialize from JSON using serde, producing the `id`, `relative_path`, and `category` fields using the same field names and encoding as the existing AssetReference record.
9. WHERE a SoundEffect value passes the `id` and `relative_path` validation rules, serializing it to JSON with serde and then deserializing the result SHALL produce a value whose `id`, `relative_path`, and `category` fields are each equal to the original value's corresponding fields.

### Requirement 3: Audio Asset Categories and AssetManager Integration

**User Story:** As a game designer, I want audio assets to load and save through the existing asset pipeline, so that music loops and sound effects are packaged with the project like other assets.

#### Acceptance Criteria

1. THE `rpg-toolkit-common` crate SHALL define a public constant `CATEGORY_MUSIC_LOOP` whose string value is exactly `music_loop`.
2. THE `rpg-toolkit-common` crate SHALL define a public constant `CATEGORY_SOUND_EFFECT` whose string value is exactly `sound_effect`.
3. WHEN an AssetManager is created via its default constructor, THE AssetManager SHALL include a category mapping from `music_loop` to the subdirectory string `audio/music/`.
4. WHEN an AssetManager is created via its default constructor, THE AssetManager SHALL include a category mapping from `sound_effect` to the subdirectory string `audio/sfx/`.
5. WHEN an AssetManager saves a project to a directory target and a registered music loop or sound effect asset's source file exists, THE AssetManager SHALL copy that audio file into the target under its category-mapped subdirectory, preserving the source file name.
6. WHEN an AssetManager saves a project to a ZIP target and a registered music loop or sound effect asset's source file exists, THE AssetManager SHALL write that audio file into the archive at its normalized relative path.
7. IF a registered audio asset's source file does not exist during save, THEN THE AssetManager SHALL emit one `AssetWarning` identifying the asset by its identifier and category, SHALL leave any previously written audio files in place, and SHALL continue processing the remaining registered assets.
8. WHEN an AssetManager validates a project, THE AssetManager SHALL produce exactly one `AssetValidationError` carrying the asset identifier, category, and resolved path for each registered audio asset whose resolved file path does not exist, and SHALL skip any registered audio asset whose relative path is empty.

### Requirement 4: Map Default Music Loop Association

**User Story:** As a game designer, I want each map to have a default music loop, so that entering a map automatically plays its intended background music.

#### Acceptance Criteria

1. THE MapData structure SHALL include an optional `default_music_loop` field of type `Option<MusicLoopId>`.
2. WHEN the `default_music_loop` field is absent from a map's JSON, THE MapData parser SHALL default the field to `None`.
3. WHEN the `default_music_loop` field is present in JSON, THE MapData parser SHALL accept a string value containing 1 to 128 characters inclusive as a valid `MusicLoopId`.
4. IF the `default_music_loop` field is present but contains an empty string, THEN THE MapData parser SHALL return a deserialization error indicating that the default music loop identifier must not be empty.
5. IF the `default_music_loop` field is present but exceeds 128 characters, THEN THE MapData parser SHALL return a deserialization error indicating that the default music loop identifier length is invalid.
6. THE MapData structure SHALL serialize to and deserialize from JSON using the existing serde format for maps.
7. FOR ALL valid MapData values, serializing then deserializing SHALL produce an equivalent value (round-trip property).
8. WHEN a project is loaded and a map's `default_music_loop` references a `MusicLoopId` that is not registered in the project, THE project loader SHALL log a warning identifying the map by its name and the missing music loop identifier, and SHALL complete the load with all remaining maps and registries retained.

### Requirement 5: PlayMusic EventAction Data Model

**User Story:** As a game designer, I want an event action that overrides the current music loop, so that story moments can change the background music independent of the map default.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `PlayMusic` variant with a `music_loop_id` field of type `MusicLoopId`.
2. WHEN the `music_loop_id` field is deserialized, THE EventAction parser SHALL accept string values containing 1 to 128 characters inclusive.
3. IF the `music_loop_id` field is an empty string during deserialization, THEN THE EventAction parser SHALL return a deserialization error indicating that music_loop_id must not be empty.
4. IF the `music_loop_id` field contains more than 128 characters during deserialization, THEN THE EventAction parser SHALL return a deserialization error indicating that music_loop_id must contain 1 to 128 characters, and SHALL NOT produce a `PlayMusic` value.
5. THE `PlayMusic` variant SHALL include a `fade_duration` field of type `f32` expressed in seconds that defaults to `0.0` when absent from JSON.
6. WHEN the `fade_duration` field is deserialized, THE EventAction parser SHALL accept finite values in the range 0.0 to 10.0 seconds inclusive.
7. IF the `fade_duration` field value is non-finite (NaN or infinity), less than 0.0, or greater than 10.0 during deserialization, THEN THE EventAction parser SHALL return a deserialization error indicating that fade_duration must be a finite value between 0.0 and 10.0 seconds inclusive, and SHALL NOT produce a `PlayMusic` value.
8. THE `PlayMusic` variant SHALL include a `loop_count` field of type `Option<u32>` where `None` represents infinite repetition, and WHEN the `loop_count` field is absent from JSON, THE EventAction parser SHALL default it to `None`.
9. WHEN the `loop_count` field is present in JSON, THE EventAction parser SHALL accept integer values greater than or equal to 1.
10. IF the `loop_count` field is present and its value is 0, THEN THE EventAction parser SHALL return a deserialization error indicating that loop_count must be at least 1 when specified, and SHALL NOT produce a `PlayMusic` value.
11. THE `PlayMusic` variant SHALL include a `fade_out_duration` field of type `f32` expressed in seconds that defaults to `0.0` when absent from JSON, representing a fade applied at the end of playback after the `loop_count` is reached.
12. WHEN the `fade_out_duration` field is deserialized, THE EventAction parser SHALL accept finite values in the range 0.0 to 10.0 seconds inclusive.
13. IF the `fade_out_duration` field value is non-finite (NaN or infinity), less than 0.0, or greater than 10.0 during deserialization, THEN THE EventAction parser SHALL return a deserialization error indicating that fade_out_duration must be a finite value between 0.0 and 10.0 seconds inclusive, and SHALL NOT produce a `PlayMusic` value.
14. THE EventAction `PlayMusic` variant SHALL serialize to and deserialize from JSON using the existing `#[serde(tag = "type")]` format, producing a JSON object containing a `"type"` field with value `"PlayMusic"`.
15. FOR ALL valid PlayMusic EventAction values, serializing then deserializing SHALL produce an equivalent value (round-trip property).

### Requirement 6: PlayMusic Runtime Effect

**User Story:** As a player, I want the background music to change during specific events, so that key moments feel distinct from the map's default music.

#### Acceptance Criteria

1. WHEN the ActionQueue advances to a `PlayMusic` action whose `loop_count` is `None` and the specified `music_loop_id` differs from the `music_loop_id` currently playing or currently cross-fading in on the music channel, THE Renderer SHALL begin playing the specified music loop on the music channel and SHALL repeat that music loop from its start each time it reaches its end until the music channel is next changed or stopped.
2. WHEN a `PlayMusic` action begins a new music loop and the `fade_duration` field is 0.0, THE Renderer SHALL stop the previously playing music loop on the music channel within the same processing step in which the new music loop begins.
3. WHEN the ActionQueue advances to a `PlayMusic` action whose `music_loop_id` matches the `music_loop_id` currently playing or currently cross-fading in on the music channel, THE Renderer SHALL continue the current playback without restarting it and without altering its current volume or fade progress, regardless of the `fade_duration` value.
4. WHERE the `fade_duration` field is greater than 0.0 and a previous music loop is playing on the music channel, THE Renderer SHALL cross-fade from the previous music loop to the new music loop over a duration equal to the `fade_duration` value in seconds (within the range 0.0 to 10.0 inclusive), reaching full volume on the new music loop and zero volume on the previous music loop at the end of that duration.
5. WHERE the `fade_duration` field is greater than 0.0 and no music loop is playing on the music channel, THE Renderer SHALL fade the new music loop in from zero volume to full volume over a duration equal to the `fade_duration` value in seconds.
6. IF the specified `music_loop_id` is not registered in the project, THEN THE Renderer SHALL log a warning identifying the missing `music_loop_id` and SHALL leave the music channel playback unchanged.
7. WHEN the ActionQueue advances to a `PlayMusic` action whose `loop_count` is a value N greater than or equal to 1 and the specified `music_loop_id` is registered, THE Renderer SHALL begin playing the specified music loop on the music channel and SHALL play it exactly N times from start to end.
8. WHEN a music loop started by a `PlayMusic` action with a finite `loop_count` completes its final repetition and the action's `fade_out_duration` is 0.0, THE Renderer SHALL stop playback and leave the music channel silent, playing no further audio on the music channel until a subsequent action changes it.
9. WHERE a `PlayMusic` action's `fade_out_duration` is greater than 0.0 and the action's `loop_count` is a finite value, THE Renderer SHALL fade the music loop's volume from its current level to zero over a duration equal to the `fade_out_duration` value in seconds, beginning such that the fade completes as the final repetition ends, and SHALL leave the music channel silent when the fade completes.
10. WHEN a music loop started by a `PlayMusic` action reaches the end of its `fade_out_duration` fade, THE Renderer SHALL NOT resume the map's default music loop and SHALL leave the music channel silent until a subsequent action changes it.
11. WHEN a `PlayMusic` action is processed, THE ActionQueue SHALL advance to the next action within the same processing step in which the action is processed, without waiting for playback, any cross-fade, the configured loop count, or any fade-out to complete.
12. WHILE a cross-fade, fade-in, or end-of-playback fade-out initiated by a `PlayMusic` action is in progress, THE Renderer SHALL continue that fade to completion independently of ActionQueue advancement.

### Requirement 7: PlaySoundEffect EventAction Data Model

**User Story:** As a game designer, I want an event action that triggers a sound effect, so that actions such as opening a door or a battle clash produce the intended sound.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `PlaySoundEffect` variant with a `sound_effect_id` field of type `SoundEffectId`.
2. WHEN the `sound_effect_id` field is deserialized, THE EventAction parser SHALL accept string values containing 1 to 128 characters inclusive.
3. IF the `sound_effect_id` field is an empty string during deserialization, THEN THE EventAction parser SHALL return a deserialization error indicating that sound_effect_id must not be empty.
4. IF the `sound_effect_id` field contains more than 128 characters during deserialization, THEN THE EventAction parser SHALL return a deserialization error indicating that sound_effect_id must contain 1 to 128 characters, and SHALL NOT produce a `PlaySoundEffect` value.
5. WHEN the `sound_effect_id` field is absent from JSON, THE EventAction parser SHALL return a deserialization error indicating that sound_effect_id is required.
6. THE `PlaySoundEffect` variant SHALL include a `volume` field of type `f32` that defaults to `1.0` when absent from JSON.
7. WHEN the `volume` field is deserialized, THE EventAction parser SHALL accept finite values in the range 0.0 to 1.0 inclusive.
8. IF the `volume` field value is non-finite (NaN or infinity), less than 0.0, or greater than 1.0 during deserialization, THEN THE EventAction parser SHALL return a deserialization error indicating that volume must be a finite value between 0.0 and 1.0 inclusive, and SHALL NOT produce a `PlaySoundEffect` value.
9. THE EventAction `PlaySoundEffect` variant SHALL serialize to and deserialize from JSON using the existing `#[serde(tag = "type")]` format, producing a JSON object containing a `"type"` field with value `"PlaySoundEffect"`.
10. FOR ALL valid PlaySoundEffect EventAction values, serializing then deserializing SHALL produce an equivalent value (round-trip property).

### Requirement 8: PlaySoundEffect Runtime Effect

**User Story:** As a player, I want to hear a sound effect when an event triggers, so that in-game actions such as doors and battles feel responsive.

#### Acceptance Criteria

1. WHEN the ActionQueue advances to a `PlaySoundEffect` action whose `sound_effect_id` is registered in the project, THE Renderer SHALL play the specified sound effect exactly once on the sound effect channel at the action's `volume` value in the range 0.0 to 1.0 inclusive.
2. WHEN a `PlaySoundEffect` action plays a sound effect, THE Renderer SHALL continue any music loop currently playing on the music channel without stopping, restarting, or altering its volume.
3. IF the specified `sound_effect_id` is not registered in the project, THEN THE Renderer SHALL log a warning identifying the missing sound effect and advance the ActionQueue without playing any audio and without altering the sound effect channel or the music channel.
4. WHEN a `PlaySoundEffect` action is processed, THE ActionQueue SHALL advance to the next action within the same update cycle without waiting for the sound effect to finish playing (non-blocking action).
5. IF the specified sound effect fails to load or decode, THEN THE Renderer SHALL log a warning identifying the sound effect and advance the ActionQueue without playing any audio and without altering the music channel.
6. WHEN a `PlaySoundEffect` action plays a sound effect while one or more previously triggered sound effects are still playing on the sound effect channel, THE Renderer SHALL play the new sound effect concurrently without stopping or interrupting the sound effects already playing.

### Requirement 9: Shared AudioLoader Utility

**User Story:** As a developer, I want a shared audio loading utility, so that every scene loads music loops and sound effects through one consistent path.

#### Acceptance Criteria

1. THE AudioLoader utility SHALL provide an operation that accepts a project root path and an audio asset relative path and returns the raw bytes of the audio file.
2. WHEN the AudioLoader is given a relative path that resolves to an existing regular file within the project root, THE AudioLoader SHALL return the complete raw byte contents of that file.
3. IF the AudioLoader is given a relative path containing a `..` component that resolves to a location outside the project root, THEN THE AudioLoader SHALL return an error indicating path traversal is not permitted, and SHALL NOT read any file.
4. IF the AudioLoader is given a relative path that is empty or contains only whitespace characters, THEN THE AudioLoader SHALL return an error indicating the path is empty, and SHALL NOT read any file.
5. IF the AudioLoader is given a relative path that does not resolve to an existing filesystem entry within the project root, THEN THE AudioLoader SHALL return an error indicating the file does not exist.
6. IF the AudioLoader is given a relative path that resolves to a directory rather than a regular file, THEN THE AudioLoader SHALL return an error indicating the path is not a regular file.
7. WHEN the AudioLoader classifies a loaded audio file's format by its file extension, THE AudioLoader SHALL recognize `ogg`, `wav`, and `mp3` as supported formats, matching the extension case-insensitively.
8. IF the AudioLoader is given a path whose file extension is present but is not one of the supported formats `ogg`, `wav`, or `mp3`, THEN THE AudioLoader SHALL return an error identifying the unsupported extension.
9. IF the AudioLoader is given a path that has no file extension, THEN THE AudioLoader SHALL return an error indicating the audio format could not be determined.
10. IF the AudioLoader resolves a path to an existing regular file with a supported extension but the file cannot be read, THEN THE AudioLoader SHALL return an error indicating the file could not be read.

### Requirement 10: ProjectManifest Audio Registries

**User Story:** As a game designer, I want music loops and sound effects stored in the project manifest, so that they persist with the project and are available across sessions.

#### Acceptance Criteria

1. THE ProjectManifest SHALL include a `music_loops` field that maps each `MusicLoopId` to its MusicLoop record, and WHEN the `music_loops` field is absent from the JSON, THE ProjectManifest parser SHALL default it to an empty mapping containing zero entries.
2. THE ProjectManifest SHALL include a `sound_effects` field that maps each `SoundEffectId` to its SoundEffect record, and WHEN the `sound_effects` field is absent from the JSON, THE ProjectManifest parser SHALL default it to an empty mapping containing zero entries.
3. WHEN a project manifest JSON containing neither a `music_loops` field nor a `sound_effects` field is loaded, THE ProjectManifest parser SHALL complete deserialization without returning an error and SHALL produce a manifest whose `music_loops` mapping and `sound_effects` mapping each contain zero entries.
4. IF a loaded ProjectManifest contains a `music_loops` or `sound_effects` registry entry whose map key differs from the `id` field of the record it maps to, THEN THE ProjectManifest parser SHALL return a validation error that identifies the offending registry name and map key, SHALL abort loading of the manifest, and SHALL NOT return a partially populated manifest.
5. WHILE a ProjectManifest value contains `music_loops` and `sound_effects` registries in which every map key equals the `id` field of its record, THE ProjectManifest parser SHALL guarantee that serializing that value and then deserializing the produced output yields a manifest value that compares equal (via PartialEq) to the original, independent of registry entry ordering.

### Requirement 11: Serialization Compatibility for Audio Additions

**User Story:** As a game designer, I want my existing project files to keep loading after audio support is added, so that I do not lose any prior work.

#### Acceptance Criteria

1. WHEN a project file created before the audio feature is loaded, THE project loader SHALL deserialize it without returning a deserialization error and SHALL preserve all pre-existing project data unchanged.
2. WHEN a project file created before the audio feature is loaded and its JSON contains no music loop or sound effect registry entries, THE project loader SHALL initialize both the music loop registry and the sound effect registry as empty collections containing zero entries.
3. WHEN a project file created before the audio feature is loaded and a map's JSON omits the `default_music_loop` field, THE MapData parser SHALL set that map's `default_music_loop` to `None`.
4. WHEN a project file containing only EventAction variants whose `type` tag values existed before the audio feature is loaded, THE EventAction parser SHALL deserialize every action in the file without returning a deserialization error.
5. IF a project file containing an EventAction whose `type` tag value is `PlayMusic` or `PlaySoundEffect` is loaded by a toolkit version whose EventAction parser does not define that tag value, THEN THE EventAction parser SHALL return a deserialization error whose message includes the unrecognized `type` tag value, and SHALL NOT partially apply the file's contents to the in-memory project.
6. IF a project file cannot be parsed as valid JSON, THEN THE project loader SHALL return a deserialization error indicating the input is not valid JSON and SHALL leave any previously loaded project unchanged.
7. FOR ALL valid project files containing any combination of EventAction variants (including `PlayMusic` with any valid combination of `fade_duration`, `loop_count`, and `fade_out_duration`, and `PlaySoundEffect`), a `default_music_loop` value of `None` or a registered `MusicLoopId`, and any combination of music loop and sound effect registry entries, serializing the project and then deserializing the result SHALL produce a project that compares equal by value to the original (round-trip property).

### Requirement 12: Editor Support for Audio Assets and Actions

**User Story:** As a game designer, I want to manage audio assets and audio event actions in the editor, so that I can configure music and sound effects without editing JSON by hand.

#### Acceptance Criteria

1. WHEN a game designer selects an audio file and assigns a `MusicLoopId` of 1 to 128 characters inclusive using the Editor's music loop registration control, THE Editor SHALL register the music loop as a project asset and display it in the project's registered music loops list.
2. WHEN a game designer selects an audio file and assigns a `SoundEffectId` of 1 to 128 characters inclusive using the Editor's sound effect registration control, THE Editor SHALL register the sound effect as a project asset and display it in the project's registered sound effects list.
3. WHEN a map's properties are edited, THE Editor SHALL display a music loop selector for the map's `default_music_loop` field populated with one entry per registered music loop plus a "None" option, where "None" is preselected if the field is `None`.
4. THE Editor Event Trigger Editor dialog SHALL include `PlayMusic` and `PlaySoundEffect` as selectable action types alongside existing action options.
5. WHEN `PlayMusic` is selected, THE Editor SHALL display a music loop selector populated with one entry per registered music loop, a numeric input for `fade_duration` initialized to 0.0 and accepting values from 0.0 to 10.0 inclusive, a control for `loop_count` that defaults to "infinite" (representing `None`) and otherwise accepts integer values greater than or equal to 1, and a numeric input for `fade_out_duration` initialized to 0.0 and accepting values from 0.0 to 10.0 inclusive.
6. WHEN `PlaySoundEffect` is selected, THE Editor SHALL display a sound effect selector populated with one entry per registered sound effect and a numeric input for `volume` initialized to 1.0 and accepting values from 0.0 to 1.0 inclusive.
7. IF a required audio identifier field (`music_loop_id` or `sound_effect_id`) is empty, THEN THE Editor SHALL disable the Add/Update button until a non-empty selection is made.
8. IF the `fade_duration` or `fade_out_duration` value is less than 0.0 or greater than 10.0 when a `PlayMusic` action is saved, THEN THE Editor SHALL clamp that value to the nearest bound of the range 0.0 to 10.0 inclusive.
9. IF a finite `loop_count` value less than 1 is entered when a `PlayMusic` action is saved, THEN THE Editor SHALL clamp the value to 1.
10. IF the `volume` value is less than 0.0 or greater than 1.0 when a `PlaySoundEffect` action is saved, THEN THE Editor SHALL clamp the value to the nearest bound of the range 0.0 to 1.0 inclusive.
11. IF a game designer attempts to register an audio asset with an identifier that is empty, exceeds 128 characters, or matches an already registered `MusicLoopId` or `SoundEffectId`, THEN THE Editor SHALL reject the registration, display an error indication describing the invalid or duplicate identifier, and leave the existing registered assets unchanged.
### Requirement 13: Audio Database Page

**User Story:** As a game designer, I want a dedicated Audio page in the editor alongside the other database pages, so that I can manage and audition music loops and sound effects in one place and reference them from other entities.

#### Acceptance Criteria

1. THE Editor SHALL provide an Audio page selectable from the same navigation used to switch between the Map, Character, Item, Ability, Enemy, and Shop pages, backed by a dedicated `AppEditorMode` variant.
2. WHEN the Audio page is active, THE Editor SHALL display the project's registered music loops and registered sound effects as two selectable lists, each entry showing its identifier and relative path.
3. WHEN the Audio page is active and the project has no registered music loops or no registered sound effects, THE Editor SHALL display an empty-state indication for the corresponding list rather than an error.
4. WHEN a game designer selects an audio file and assigns a `MusicLoopId` of 1 to 128 characters inclusive on the Audio page, THE Editor SHALL register the music loop as a project asset and display it in the registered music loops list.
5. WHEN a game designer selects an audio file and assigns a `SoundEffectId` of 1 to 128 characters inclusive on the Audio page, THE Editor SHALL register the sound effect as a project asset and display it in the registered sound effects list.
6. IF a game designer attempts to register an audio asset on the Audio page with an identifier that is empty, exceeds 128 characters, or matches an already registered `MusicLoopId` or `SoundEffectId`, THEN THE Editor SHALL reject the registration, display an error indication describing the invalid or duplicate identifier, and leave the existing registered assets unchanged.
7. WHEN a game designer selects a registered music loop or sound effect on the Audio page, THE Editor SHALL display a preview control offering a play action for that asset.
8. WHEN a game designer activates the play action for a selected asset whose source file resolves and loads successfully, THE Editor SHALL play that asset's audio once through the editor's audio output.
9. WHILE an asset preview is playing, THE Editor SHALL offer a stop action that halts the preview playback when activated.
10. WHEN a game designer activates the play action for a different asset while a preview is already playing, THE Editor SHALL stop the currently playing preview before starting the newly selected asset's preview, so that at most one preview plays at a time.
11. IF the selected asset's source file cannot be resolved, loaded, or decoded when the play action is activated, THEN THE Editor SHALL display an error indication identifying the asset and SHALL NOT alter the registered assets.
12. WHEN registration on the Audio page succeeds, THE Editor SHALL mark the project as having unsaved audio changes consistent with the existing `has_unsaved_audio_changes` tracking.
