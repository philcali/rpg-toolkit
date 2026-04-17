# Dialog Bleedthrough Fix — Bugfix Design

## Overview

Mouse clicks, scrolls, and drags pass through egui dialog windows (New Map, Load Tileset, Error, Unsaved Changes, Spritesheet Manager, Remove Spritesheet, Event Trigger Editor, Spawn Point Confirm, NPC Placement) to the Bevy `Update` schedule systems that handle canvas interactions. The fix adds early-return guards in five systems (`painting_system`, `zoom_system`, `pan_system`, `attribute_click_system`, `update_cursor_state`) that query `egui::Context::wants_pointer_input()` before processing mouse input. This is a minimal, centralized change that leverages egui's own input-consumption tracking rather than manually checking dialog resource flags.

## Glossary

- **Bug_Condition (C)**: The condition that triggers the bug — when egui reports `wants_pointer_input() == true` (a dialog/popup/combo is consuming pointer input) and a Bevy `Update` system processes mouse input anyway
- **Property (P)**: The desired behavior — when egui wants pointer input, the five canvas interaction systems must skip all mouse processing
- **Preservation**: Existing canvas behavior when no dialog is consuming pointer input must remain unchanged: painting, erasing, flood fill, stamp brush, zoom, pan, attribute clicks, and cursor state all continue to work
- **`painting_system`**: The system in `plugins/painting.rs` that handles tile painting, erasing, flood fill, and stamp brush via left-click
- **`zoom_system`**: The system in `systems/camera.rs` that handles mouse-wheel zoom centered on cursor
- **`pan_system`**: The system in `systems/camera.rs` that handles middle-mouse and left-click (Pan tool) camera dragging
- **`attribute_click_system`**: The system in `plugins/attribute.rs` that handles left-click in attribute mode for opacity toggle, event trigger, spawn point, and NPC placement
- **`update_cursor_state`**: The system in `systems/input.rs` that projects screen cursor to world/tile coordinates each frame
- **`wants_pointer_input()`**: `egui::Context` method that returns `true` when egui is actively consuming pointer input (dialog windows, popups, combo boxes, etc.)

## Bug Details

### Bug Condition

The bug manifests when any egui dialog window is open and consuming pointer input, and the user performs mouse interactions (clicks, scrolls, drags) within the dialog area. The five Bevy `Update` systems process these mouse events without checking whether egui has claimed them, causing unintended canvas side effects.

**Formal Specification:**
```
FUNCTION isBugCondition(input)
  INPUT: input of type MouseEvent (click, scroll, or drag)
  OUTPUT: boolean

  RETURN egui_context.wants_pointer_input() == true
         AND input IS processed by any of:
             [painting_system, zoom_system, pan_system,
              attribute_click_system, update_cursor_state]
END FUNCTION
```

### Examples

- User opens "New Map" dialog, clicks "Create" button → click bleeds through to `painting_system`, placing a tile on the canvas behind the dialog
- User opens "Spritesheet Manager" window, scrolls through spritesheet list → scroll bleeds through to `zoom_system`, zooming the camera
- User opens "Error" dialog, middle-mouse drags to dismiss → drag bleeds through to `pan_system`, panning the camera
- User opens "Event Trigger Editor" in attribute mode, clicks "Save" → click bleeds through to `attribute_click_system`, toggling opacity on the tile behind the dialog
- User opens "Load Tileset" dialog, moves cursor over it → `update_cursor_state` computes tile_pos under the dialog, enabling other systems to act on stale cursor state

## Expected Behavior

### Preservation Requirements

**Unchanged Behaviors:**
- When no dialog is consuming pointer input, painting (left-click, left-drag) continues to place/erase tiles as expected
- When no dialog is consuming pointer input, mouse-wheel zoom continues to work centered on cursor
- When no dialog is consuming pointer input, middle-mouse drag and left-click drag (Pan mode) continue to pan the camera
- When no dialog is consuming pointer input, attribute mode clicks continue to toggle opacity, open event trigger dialog, place spawn points, and place NPCs
- When no dialog is consuming pointer input, cursor state continues to track world and tile positions
- Existing `CanvasRect` gating in `update_cursor_state` (blocks clicks on egui side panels) remains functional
- Clicking outside a dialog but within the canvas area continues to allow canvas interactions even when a dialog is open (the guard is based on `wants_pointer_input()`, not on dialog `.open` flags)

**Scope:**
All inputs where `egui_context.wants_pointer_input() == false` should be completely unaffected by this fix. This includes:
- All mouse interactions when no dialog is open
- All mouse interactions on the canvas area when a dialog is open but the pointer is not over the dialog
- Keyboard shortcuts (tool hotkeys, undo/redo) — these are unaffected since the guard only checks pointer input

## Hypothesized Root Cause

Based on the code analysis, the root cause is clear and confirmed:

1. **No egui pointer check in `painting_system`** (`plugins/painting.rs`): The system reads `ButtonInput<MouseButton>` and `CursorWorldState` directly with no guard against egui consuming the pointer. It has an `EditorMode::Attribute` early return and a `Pan` early return but nothing for egui dialogs.

2. **No egui pointer check in `zoom_system`** (`systems/camera.rs`): The system reads `MouseWheel` scroll events directly with no guard. Scroll events from interacting with egui combo boxes or scrollable dialog content pass straight through.

3. **No egui pointer check in `pan_system`** (`systems/camera.rs`): The system reads `ButtonInput<MouseButton>` for middle-mouse and left-click drag with no guard. Drags that start on a dialog bleed through.

4. **Partial guard in `attribute_click_system`** (`plugins/attribute.rs`): The system has an existing guard but it only checks `event_trigger_dialog.open || spawn_confirm_dialog.open || npc_placement_dialog.open` combined with `ctx.is_pointer_over_area()`. This misses dialogs from other plugins (New Map, Load Tileset, Error, Unsaved Changes, Spritesheet Manager, Remove Spritesheet Confirmation). It should use `wants_pointer_input()` unconditionally.

5. **No egui pointer check in `update_cursor_state`** (`systems/input.rs`): The system updates `tile_pos` based on screen cursor position. Although it has `CanvasRect` gating for side panels, it doesn't check whether egui is consuming the pointer, which means other systems can act on stale `tile_pos` values computed while a dialog is active.

## Correctness Properties

Property 1: Bug Condition — Dialog blocks canvas mouse input

_For any_ mouse event (click, scroll, or drag) where `egui_context.wants_pointer_input()` returns true, the five canvas interaction systems (`painting_system`, `zoom_system`, `pan_system`, `attribute_click_system`, `update_cursor_state`) SHALL return early without processing the mouse event, preventing any canvas side effects (tile placement, zoom changes, camera panning, attribute modifications, tile_pos updates).

**Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6**

Property 2: Preservation — Normal canvas interactions unchanged

_For any_ mouse event where `egui_context.wants_pointer_input()` returns false, the five canvas interaction systems SHALL produce exactly the same behavior as the original unfixed code, preserving all painting, zooming, panning, attribute clicking, and cursor state functionality.

**Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6**

## Fix Implementation

### Changes Required

Assuming our root cause analysis is correct:

**File**: `crates/rpg-toolkit-editor/src/plugins/painting.rs`

**Function**: `painting_system`

**Specific Changes**:
1. **Add `EguiContexts` parameter** to the system function signature
2. **Add early return guard** at the top of the function: query `ctx.wants_pointer_input()` and return early if true. This must come before any `ButtonInput<MouseButton>` reads.

---

**File**: `crates/rpg-toolkit-editor/src/systems/camera.rs`

**Function**: `zoom_system`

**Specific Changes**:
1. **Add `EguiContexts` parameter** to the system function signature
2. **Add early return guard** at the top of the function (before reading `MouseWheel` events): query `ctx.wants_pointer_input()` and return early if true

**Function**: `pan_system`

**Specific Changes**:
1. **Add `EguiContexts` parameter** to the system function signature
2. **Add early return guard** at the top of the function (before reading `ButtonInput<MouseButton>`): query `ctx.wants_pointer_input()` and return early if true
3. **Cancel active pans** when egui takes pointer: if a pan is in progress and egui starts consuming pointer input, reset `PanState` to prevent stuck pan state

---

**File**: `crates/rpg-toolkit-editor/src/plugins/attribute.rs`

**Function**: `attribute_click_system`

**Specific Changes**:
1. **Replace the existing partial guard** (which checks specific dialog `.open` flags) with a single `ctx.wants_pointer_input()` check at the top. This covers all current and future dialogs without needing to enumerate them.

---

**File**: `crates/rpg-toolkit-editor/src/systems/input.rs`

**Function**: `update_cursor_state`

**Specific Changes**:
1. **Add `EguiContexts` parameter** to the system function signature
2. **Add early return guard** after resetting `cursor_state` fields: query `ctx.wants_pointer_input()` and return early if true. This prevents `tile_pos` from being set when egui is consuming the pointer, which downstream systems rely on.

---

**Note on `EguiContexts` usage**: Since `zoom_system`, `pan_system`, and `update_cursor_state` run in the `Update` schedule (not `EguiPrimaryContextPass`), they need `EguiContexts` as a system parameter. Bevy_egui supports this — `EguiContexts` is a `SystemParam` that can be used in any schedule. The `ctx_mut()` call may return `Err` if no egui context exists, so the guard should use `if let Ok(ctx) = contexts.ctx_mut() && ctx.wants_pointer_input() { return; }`.

## Testing Strategy

### Validation Approach

The testing strategy follows a two-phase approach: first, surface counterexamples that demonstrate the bug on unfixed code, then verify the fix works correctly and preserves existing behavior.

### Exploratory Bug Condition Checking

**Goal**: Surface counterexamples that demonstrate the bug BEFORE implementing the fix. Confirm that the five systems process mouse input when egui is consuming it.

**Test Plan**: Write integration tests that simulate egui dialog state + mouse input and assert that canvas state changes. Run on UNFIXED code to observe the bug.

**Test Cases**:
1. **Painting Bleedthrough Test**: Set up a painting scenario with an active brush and tile_pos, simulate left-click while `wants_pointer_input()` is true → observe tile placement (will fail on unfixed code — tile is placed)
2. **Zoom Bleedthrough Test**: Fire a `MouseWheel` event while `wants_pointer_input()` is true → observe zoom_level change (will fail on unfixed code — zoom changes)
3. **Pan Bleedthrough Test**: Simulate `MouseButton::Middle` press while `wants_pointer_input()` is true → observe `PanState.middle_panning` set to true (will fail on unfixed code — panning starts)
4. **Attribute Bleedthrough Test**: In attribute mode, simulate left-click with a dialog from another plugin open → observe opacity toggle (will fail on unfixed code for non-attribute dialogs)

**Expected Counterexamples**:
- Canvas state mutates (tiles placed, zoom changed, panning started, opacity toggled) even though egui is consuming the pointer
- Root cause confirmed: no `wants_pointer_input()` check in four of the five systems, partial check in `attribute_click_system`

### Fix Checking

**Goal**: Verify that for all inputs where the bug condition holds, the fixed systems do not mutate canvas state.

**Pseudocode:**
```
FOR ALL mouseEvent WHERE isBugCondition(mouseEvent) DO
  canvasStateBefore := snapshot(project, editorState, panState)
  run_fixed_system(mouseEvent)
  canvasStateAfter := snapshot(project, editorState, panState)
  ASSERT canvasStateBefore == canvasStateAfter
END FOR
```

### Preservation Checking

**Goal**: Verify that for all inputs where the bug condition does NOT hold, the fixed systems produce the same result as the original systems.

**Pseudocode:**
```
FOR ALL mouseEvent WHERE NOT isBugCondition(mouseEvent) DO
  ASSERT run_original_system(mouseEvent) == run_fixed_system(mouseEvent)
END FOR
```

**Testing Approach**: Property-based testing is recommended for preservation checking because:
- It generates many random mouse events, tool states, and editor configurations automatically
- It catches edge cases where the guard might accidentally block valid canvas interactions
- It provides strong guarantees that behavior is unchanged for all non-dialog-active scenarios

**Test Plan**: Observe behavior on UNFIXED code first for normal canvas interactions (no dialog active), then write property-based tests capturing that behavior.

**Test Cases**:
1. **Painting Preservation**: Generate random tool/brush/tile_pos combinations with `wants_pointer_input() == false`, verify tiles are placed/erased identically to unfixed code
2. **Zoom Preservation**: Generate random scroll amounts with no dialog active, verify zoom_level matches unfixed code
3. **Pan Preservation**: Generate random middle-mouse drags with no dialog active, verify camera_offset matches unfixed code
4. **Attribute Preservation**: Generate random attribute clicks with no dialog active, verify opacity/event trigger/spawn/NPC behavior matches unfixed code

### Unit Tests

- Test that `painting_system` returns early and does not place tiles when `wants_pointer_input()` returns true
- Test that `zoom_system` returns early and does not change zoom when `wants_pointer_input()` returns true
- Test that `pan_system` returns early, does not start panning, and cancels active pan when `wants_pointer_input()` returns true
- Test that `attribute_click_system` returns early for ALL dialog types (not just attribute-specific ones) when `wants_pointer_input()` returns true
- Test that `update_cursor_state` does not set `tile_pos` when `wants_pointer_input()` returns true
- Test edge case: clicking outside dialog area but within canvas while a dialog is open (should still allow canvas interaction since `wants_pointer_input()` returns false for that pointer position)

### Property-Based Tests

- Generate random `(EditorTool, Option<TileRef>, Option<(u32,u32)>, bool)` tuples where the bool represents `wants_pointer_input()`, and verify the painting system only mutates state when the bool is false
- Generate random `(f32_scroll_delta, f32_zoom_level, bool)` tuples and verify zoom only changes when the bool is false
- Generate random `(MouseButton, bool_panning, bool_wants_pointer)` tuples and verify panning only activates/continues when `wants_pointer` is false

### Integration Tests

- Test full flow: open "New Map" dialog → click within it → verify no tile placement
- Test full flow: open "Spritesheet Manager" → scroll within it → verify no camera zoom
- Test full flow: open "Event Trigger Editor" → click Save → verify no opacity toggle on underlying tile
- Test that closing a dialog immediately restores normal canvas interaction
