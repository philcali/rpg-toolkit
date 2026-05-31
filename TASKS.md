# RPG Toolkit State Feature — Task List

## Phase 1: Save/Load (DONE)
- [x] `SaveFile` type in `crates/rpg-toolkit-renderer/src/save.rs` with `load`/`save` methods
- [x] `SavePath` resource in `crates/rpg-toolkit-renderer/src/resources.rs`
- [x] `add_systems(Last, save_shutdown)` in renderer plugin — persists `GameState.flags` to disk on app exit
- [x] `--save <path>` flag in launcher — default save path is `save.json` next to project
- [x] Launcher loads `save.json` on startup, populates `GameState` resource
- [x] All 21 tests pass

## Phase 2: StateCheck + required_state (DONE)
- [x] `EventAction::StateCheck` — checks a game state key/value, dispatches to `on_true`/`on_false` branches
  - `value: None` = key existence check (key present = true)
  - `value: Some(v)` = key == value check
- [x] `NpcInstance.required_state: Option<(String, String)>` — NPC invisible when condition fails
- [x] `TileAttributes.required_state: Option<(String, String)>` — tile invisible when condition fails
- [x] Renderer checks `required_state` in: `spawn_npc_sprites`, `sync_map_sprites`, `init_npc_positions`, `npc_trigger_system`, `npc_patrol_movement`
- [x] `advance_action_queue` handles `StateCheck` — evaluates `GameState.flags`, pushes matching branch to front of queue
- [x] Editor: `StateCheck` action type in action editor UI + form
- [x] Editor: `required_state` field in NPC placement dialog
- [x] All 21 tests pass

## Phase 3: Editor State Panel (DONE)
- [x] New egui panel in editor showing current `GameState.flags`
- [x] Ability to manually add/edit/delete state keys for testing
- [x] Display which NPCs are conditionally hidden on the current map (by showing `required_state` column in NPC list)
- [x] Display which tiles are conditionally hidden on the current layer

## Future / Out of Scope
- [ ] Entity spawning (adding entities back)
- [ ] Complex state types (arrays, nested objects) — strings are fine for flags
- [ ] Multiple save slots
- [ ] State persistence across project file saves (project file stays the template)
- [ ] `StateCheck` on tile triggers (e.g., only show dialog if key matches)
