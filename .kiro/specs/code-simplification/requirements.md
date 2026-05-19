# Requirements Document

## Introduction

This spec covers a code review and simplification pass across the `rpg-toolkit-editor` crate. The primary goal is to reduce complexity, improve maintainability, and establish clearer module boundaries before embarking on significant editor improvements. The main target is `attribute.rs` (2546 lines — 35% of all editor plugin code), but the effort extends to other modules with structural issues such as duplicated patterns, mixed concerns, and oversized resource structs.

## Glossary

- **Editor**: The `rpg-toolkit-editor` crate, a Bevy-based map editor application
- **Plugin**: A Bevy plugin struct that registers systems, resources, and events for a specific editor feature
- **Module**: A Rust module (file or directory) within the editor crate
- **Resource**: A Bevy `Resource` struct that holds shared mutable state accessible by systems
- **System**: A Bevy system function that runs each frame or on specific schedules
- **Dialog_Resource**: A Bevy Resource struct that holds the transient UI state for a modal dialog window
- **Attribute_Plugin**: The plugin in `attribute.rs` handling opacity, event triggers, spawn points, and NPC placement
- **Event_Trigger_Dialog**: The modal dialog for editing event trigger actions on a tile
- **NPC_Placement_Dialog**: The modal dialog for placing and editing NPCs
- **Common_Crate**: The `rpg-toolkit-common` crate containing shared types used by both editor and renderer

## Requirements

### Requirement 1: Decompose attribute.rs into sub-modules

**User Story:** As a developer, I want the attribute plugin split into focused sub-modules, so that I can navigate and modify individual attribute features without scrolling through 2500+ lines.

#### Acceptance Criteria

1. WHEN the Attribute_Plugin is loaded, THE Editor SHALL register all attribute systems and resources identically to the current monolithic implementation
2. THE Editor SHALL organize the Attribute_Plugin code into separate sub-modules for each concern: overlay rendering, click handling, event trigger dialog UI, spawn point dialog UI, and NPC placement dialog UI
3. THE Editor SHALL ensure no individual attribute sub-module exceeds 500 lines of code
4. WHEN the attribute sub-modules are compiled, THE Editor SHALL produce identical public API surface as the current single-file implementation

### Requirement 2: Extract shared dialog action editing into a reusable module

**User Story:** As a developer, I want the duplicated action-editing UI code (shared between Event_Trigger_Dialog and NPC_Placement_Dialog) extracted into a reusable component, so that adding new action types requires changes in only one place.

#### Acceptance Criteria

1. THE Editor SHALL provide a shared action editor module that renders UI for adding and editing EventAction variants
2. WHEN a new EventAction variant is added to the Common_Crate, THE Editor SHALL require action editor changes in only the shared module rather than in multiple dialog implementations
3. THE Editor SHALL use the shared action editor in both the Event_Trigger_Dialog and the NPC_Placement_Dialog without duplicating field definitions or rendering logic

### Requirement 3: Consolidate duplicated dialog resource fields

**User Story:** As a developer, I want the duplicated field patterns across Dialog_Resource structs consolidated, so that dialog state initialization is consistent and less error-prone.

#### Acceptance Criteria

1. THE Editor SHALL define a shared ActionEditorState struct containing the fields common to event action editing (action type, target coordinates, dialog text fields, shake fields, fade fields, state fields, appearance fields)
2. WHEN a Dialog_Resource is initialized, THE Editor SHALL use the shared ActionEditorState default rather than manually initializing each field
3. THE Editor SHALL reduce the total field count in NPC_Placement_Dialog by extracting action-editing fields into the shared struct

### Requirement 4: Separate editor_state.rs concerns

**User Story:** As a developer, I want editor state, edit commands, and undo history in separate modules, so that each concern can evolve independently.

#### Acceptance Criteria

1. THE Editor SHALL place EditorState and related enums (EditorTool, EditorMode, AttributeTool) in a dedicated state module
2. THE Editor SHALL place EditCommand, EditCommandKind, and their apply/apply_inverse implementations in a dedicated commands module
3. THE Editor SHALL place UndoHistory in a dedicated undo history module
4. WHEN the data modules are compiled, THE Editor SHALL maintain the same public re-exports from the data module root

### Requirement 5: Reduce system function parameter counts

**User Story:** As a developer, I want system functions with excessive parameters refactored to use SystemParam bundles, so that system signatures are readable and Bevy's borrow-checking is clearer.

#### Acceptance Criteria

1. WHEN a system function has more than 6 parameters, THE Editor SHALL group related parameters into a custom SystemParam struct
2. THE Editor SHALL apply SystemParam extraction to at minimum the attribute_click_system and painting_system functions
3. WHEN SystemParam structs are introduced, THE Editor SHALL name them descriptively to indicate their grouped concern (e.g., AttributeClickParams, PaintingContext)

### Requirement 6: Eliminate redundant NPC dialog initialization code

**User Story:** As a developer, I want NPC dialog initialization consolidated, so that the two near-identical initialization blocks (new placement vs. edit existing) share a common reset path.

#### Acceptance Criteria

1. THE Editor SHALL provide a single initialization method on NPC_Placement_Dialog that resets all action-editing fields to defaults
2. WHEN opening the NPC dialog for a new placement, THE Editor SHALL call the shared initialization method and then set placement-specific fields
3. WHEN opening the NPC dialog for editing an existing NPC, THE Editor SHALL call the shared initialization method and then populate fields from the existing NPC data
4. THE Editor SHALL eliminate the duplicated field-by-field initialization blocks currently present in attribute_click_system

### Requirement 7: Ensure compilation correctness after refactoring

**User Story:** As a developer, I want confidence that the refactoring preserves correctness, so that no regressions are introduced.

#### Acceptance Criteria

1. WHEN all simplification changes are applied, THE Editor SHALL compile without errors or new warnings
2. WHEN all simplification changes are applied, THE Editor SHALL pass all existing property-based tests in the tests/properties directory
3. THE Editor SHALL maintain identical runtime behavior for all user-facing features (painting, attribute editing, undo/redo, serialization)

### Requirement 8: Document module structure

**User Story:** As a developer, I want a brief module-level documentation comment at the top of each new sub-module, so that the purpose of each file is immediately clear.

#### Acceptance Criteria

1. THE Editor SHALL include a module-level doc comment (`//!`) at the top of every newly created sub-module file explaining its responsibility
2. WHEN a developer opens any attribute sub-module, THE Editor SHALL present a doc comment that identifies which attribute feature the module implements
