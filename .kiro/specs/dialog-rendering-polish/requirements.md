# Requirements Document

## Introduction

This feature enhances the dialog rendering system in the RPG toolkit renderer. The current dialog box has a fixed width (80%) but no fixed height, no visible border, no overflow indicator, and no distinction between regular dialogs and attribute-triggered dialogs. This spec addresses all four gaps: fixed height, a visible border, a "more content" visual cue when text overflows, and a borderless/backgroundless rendering mode for attribute dialogs.

## Glossary

- **Dialog_Box**: The UI panel spawned by the renderer to display dialog text to the player during gameplay. Composed of a root container entity and an inner panel with text.
- **Dialog_Panel**: The inner node of the Dialog_Box that holds the background color, border, and text content.
- **Overflow_Indicator**: A visual cue rendered inside the Dialog_Panel when the text content exceeds the visible area, signaling the player that more content exists.
- **Attribute_Dialog**: A ShowDialog event triggered from a tile attribute event trigger. Renders text without the Dialog_Panel background or border.
- **DialogConfig**: The configuration struct controlling dialog behavior (text speed, position, movement block).
- **Renderer**: The `rpg-toolkit-renderer` crate responsible for spawning and managing dialog UI entities.
- **Text_Style_Markup**: Inline formatting syntax within dialog text strings that uses underscore fencing to denote styled spans. Two underscores (`__text__`) produce bold, one underscore (`_text_`) produces emphasis (italic), and three underscores (`___text___`) produce bold emphasis (bold + italic).
- **Face_Portrait**: A close-up image of the speaking character displayed alongside the dialog text inside the Dialog_Panel, used to visually identify who is speaking.

## Requirements

### Requirement 1: Fixed Dialog Box Height

**User Story:** As a game designer, I want the dialog box to have a consistent fixed height, so that the dialog presentation looks uniform regardless of text length.

#### Acceptance Criteria

1. THE Dialog_Panel SHALL have a fixed height that does not change based on text content length.
2. WHEN the Dialog_Panel is rendered, THE Renderer SHALL apply the fixed height independently of the existing fixed width (80% of screen width).
3. THE Dialog_Panel SHALL clip or hide text content that exceeds the fixed height boundary.

### Requirement 2: Dialog Box Border

**User Story:** As a game designer, I want a visible border around the dialog box, so that the dialog is clearly distinguished from the game world.

#### Acceptance Criteria

1. WHEN a Dialog_Box is spawned, THE Renderer SHALL render a visible border around the Dialog_Panel.
2. THE Dialog_Panel border SHALL be visually distinct from both the panel background and the surrounding game content.
3. THE Dialog_Panel border SHALL be rendered on all four sides of the panel.

### Requirement 3: Overflow Content Indicator

**User Story:** As a player, I want a visual cue when there is more dialog text than fits in the box, so that I know additional content is available.

#### Acceptance Criteria

1. WHEN the dialog text content exceeds the visible area of the fixed-height Dialog_Panel, THE Renderer SHALL display an Overflow_Indicator.
2. WHEN the dialog text content fits entirely within the Dialog_Panel, THE Renderer SHALL NOT display an Overflow_Indicator.
3. THE Overflow_Indicator SHALL be positioned within the Dialog_Panel in a location that does not obscure the readable text.
4. THE Overflow_Indicator SHALL be visually recognizable as a "more content" signal (e.g., a downward arrow, ellipsis, or similar cue).

### Requirement 4: Attribute Dialog Renders Without Box

**User Story:** As a game designer, I want attribute-triggered dialogs to render text without the dialog box background and border, so that attribute text appears as floating text in the game world.

#### Acceptance Criteria

1. WHEN a ShowDialog event is configured as an Attribute_Dialog, THE Renderer SHALL render the dialog text without a background color on the Dialog_Panel.
2. WHEN a ShowDialog event is configured as an Attribute_Dialog, THE Renderer SHALL render the dialog text without a border on the Dialog_Panel.
3. WHEN a ShowDialog event is configured as an Attribute_Dialog, THE Renderer SHALL still render the dialog text content and respect the text speed and position settings from DialogConfig.
4. THE DialogConfig SHALL include a field that distinguishes an Attribute_Dialog from a standard dialog.


### Requirement 5: Inline Text Style Markup

**User Story:** As a game designer, I want to apply bold, emphasis, and bold-emphasis styling to portions of dialog text using underscore fencing, so that I can add visual emphasis to important words and phrases within dialog.

#### Acceptance Criteria

1. WHEN dialog text contains a span enclosed in two underscores (`__text__`), THE Renderer SHALL render that span in bold.
2. WHEN dialog text contains a span enclosed in one underscore (`_text_`), THE Renderer SHALL render that span in italic (emphasis).
3. WHEN dialog text contains a span enclosed in three underscores (`___text___`), THE Renderer SHALL render that span in bold and italic (bold emphasis).
4. WHEN dialog text contains no Text_Style_Markup delimiters, THE Renderer SHALL render the entire text in the default (unstyled) font.
5. THE Renderer SHALL support multiple styled spans within a single dialog text string.
6. IF a Text_Style_Markup delimiter is opened but not closed before the end of the text, THEN THE Renderer SHALL render the remaining text in the default style without producing an error.

### Requirement 6: Face Portrait Display

**User Story:** As a game designer, I want some dialogs to display a face portrait of the speaking character alongside the text, so that the player can visually identify who is speaking.

#### Acceptance Criteria

1. WHEN a ShowDialog event includes a Face_Portrait reference, THE Renderer SHALL display the portrait image inside the Dialog_Panel alongside the dialog text.
2. WHEN a ShowDialog event does not include a Face_Portrait reference, THE Renderer SHALL render the Dialog_Panel with text only and no portrait space reserved.
3. THE Face_Portrait SHALL be positioned within the Dialog_Panel so that it does not overlap or obscure the dialog text.
4. THE DialogConfig SHALL include an optional field for specifying a Face_Portrait image reference.
5. THE Face_Portrait image SHALL maintain its original aspect ratio when rendered inside the Dialog_Panel.
