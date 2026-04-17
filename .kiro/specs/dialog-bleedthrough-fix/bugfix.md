# Bugfix Requirements Document

## Introduction

When modal-style egui dialog windows are open (Load Tileset, Spritesheet Import, New Map, Error, Unsaved Changes, Remove Spritesheet Confirmation), mouse clicks pass through the dialog and interact with the underlying canvas. This causes unintended tile painting, erasing, flood filling, panning, zooming, and attribute placement while the user is interacting with a dialog. The root cause is that the Bevy `Update` systems (`painting_system`, `zoom_system`, `pan_system`, `attribute_click_system`) and the `update_cursor_state` system do not check whether egui is currently consuming pointer input via `ctx.wants_pointer_input()` or `ctx.is_pointer_over_area()`.

## Bug Analysis

### Current Behavior (Defect)

1.1 WHEN the Load Tileset dialog is open AND the user clicks within the dialog area THEN the system allows the click to also reach the painting system, causing unintended tile placement or erasure on the canvas behind the dialog

1.2 WHEN the Spritesheet Import/Manager dialog is open AND the user clicks within the dialog area THEN the system allows the click to also reach the painting system, causing unintended tile placement or erasure on the canvas behind the dialog

1.3 WHEN any egui dialog window is open (New Map, Error, Unsaved Changes, Remove Spritesheet Confirmation) AND the user clicks within the dialog area THEN the system allows the click to also reach the painting system, causing unintended canvas interactions behind the dialog

1.4 WHEN any egui dialog window is open AND the user scrolls the mouse wheel over the dialog THEN the system allows the scroll event to reach the zoom system, causing unintended camera zoom changes

1.5 WHEN any egui dialog window is open AND the user performs a middle-mouse drag or left-click drag (in Pan mode) over the dialog THEN the system allows the drag to reach the pan system, causing unintended camera panning

1.6 WHEN any egui dialog window is open AND the editor is in Attribute mode AND the user clicks within the dialog area THEN the system allows the click to reach the attribute click system, causing unintended opacity toggles, event trigger openings, spawn point placements, or NPC placements on the canvas behind the dialog

### Expected Behavior (Correct)

2.1 WHEN the Load Tileset dialog is open AND the user clicks within the dialog area THEN the system SHALL consume the click in egui and NOT pass it through to the painting system or any canvas interaction system

2.2 WHEN the Spritesheet Import/Manager dialog is open AND the user clicks within the dialog area THEN the system SHALL consume the click in egui and NOT pass it through to the painting system or any canvas interaction system

2.3 WHEN any egui dialog window is open AND the user clicks within the dialog area THEN the system SHALL consume the click in egui and NOT pass it through to the painting system or any canvas interaction system

2.4 WHEN any egui dialog window is open AND the user scrolls the mouse wheel over the dialog THEN the system SHALL consume the scroll event in egui and NOT pass it through to the zoom system

2.5 WHEN any egui dialog window is open AND the user performs a middle-mouse drag or left-click drag over the dialog THEN the system SHALL consume the input in egui and NOT pass it through to the pan system

2.6 WHEN any egui dialog window is open AND the editor is in Attribute mode AND the user clicks within the dialog area THEN the system SHALL consume the click in egui and NOT pass it through to the attribute click system

### Unchanged Behavior (Regression Prevention)

3.1 WHEN no egui dialog window is open AND the user clicks on the canvas THEN the system SHALL CONTINUE TO perform tile painting, erasing, flood filling, or stamp brushing as expected based on the active tool

3.2 WHEN no egui dialog window is open AND the user scrolls the mouse wheel over the canvas THEN the system SHALL CONTINUE TO zoom the camera as expected

3.3 WHEN no egui dialog window is open AND the user middle-mouse drags or left-click drags (in Pan mode) on the canvas THEN the system SHALL CONTINUE TO pan the camera as expected

3.4 WHEN no egui dialog window is open AND the user clicks on egui side panels (tile palette, layer panel) THEN the system SHALL CONTINUE TO NOT pass those clicks through to the canvas (existing CanvasRect gating behavior is preserved)

3.5 WHEN no egui dialog window is open AND the editor is in Attribute mode AND the user clicks on the canvas THEN the system SHALL CONTINUE TO toggle opacity, open event trigger dialogs, place spawn points, or place NPCs as expected

3.6 WHEN any egui dialog window is open AND the user clicks OUTSIDE the dialog but within the canvas area THEN the system SHALL CONTINUE TO allow canvas interactions (the guard only blocks input when egui is consuming pointer input, not merely when a dialog resource is flagged open)
