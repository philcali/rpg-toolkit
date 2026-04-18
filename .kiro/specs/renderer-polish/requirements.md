# Requirements Document

## Introduction

This specification captures a set of visual and stylistic improvements to the RPG toolkit's game renderer. The current renderer displays character sprites at their raw 24×32 pixel size and renders the game world at a 1:1 pixel-to-screen-pixel ratio, which makes everything appear tiny on modern high-resolution displays. Walk animations cycle through all three frames continuously without a proper step pattern, and there is no idle breathing animation. These polish items address sprite scaling, animation quality, and screen scaling to produce a visually coherent, retro-styled RPG experience before new major features are tackled.

## Glossary

- **Renderer**: The rpg-toolkit-renderer crate that renders the project as a playable game world using Bevy ECS.
- **Character_Sprite**: A sprite entity rendered from a Character_Spritesheet, used for the player or NPC characters.
- **Pixel_Scale**: An integer multiplier applied to the camera projection, determining how many screen pixels represent one game pixel (e.g., a Pixel_Scale of 3 means each game pixel occupies a 3×3 block of screen pixels). Can be set to a fixed value or computed automatically via zoom-to-fit.
- **Sprite_Scale**: A uniform scale factor applied to Character_Sprite transforms so that sprites visually occupy the correct tile area relative to the tile grid.
- **Walk_Cycle**: The ordered sequence of Animation_Frames displayed when a character moves between tiles, following a left-step, center, right-step, center pattern.
- **Idle_Pose**: The stationary appearance of a character when not moving, displayed using the center (second) Animation_Frame for the current Facing_Direction.
- **Animation_Frame**: One of three sequential images in a spritesheet row for a given Facing_Direction (frame 0: left step, frame 1: center/idle, frame 2: right step).
- **Frame_Duration**: The time in seconds each Animation_Frame is displayed before advancing to the next frame in the Walk_Cycle.
- **Camera_Projection**: The Bevy OrthographicProjection component on the game camera that controls the visible area and effective zoom level.
- **Viewport**: The visible area of the game world displayed in the application window.

## Requirements

### Requirement 1: Game Screen Pixel Scaling

**User Story:** As a game creator, I want the game world to be scaled up so that pixel art is clearly visible on modern displays, so that the retro art style reads well at any window size.

#### Acceptance Criteria

1. THE Renderer SHALL provide a configurable Pixel_Scale resource containing a scale mode (either a fixed integer value or zoom-to-fit) defaulting to zoom-to-fit.
2. WHEN the scale mode is set to zoom-to-fit, THE Renderer SHALL compute the largest integer Pixel_Scale at which the entire active map fits within the window, ensuring the full map is visible without scrolling.
3. WHEN the scale mode is set to zoom-to-fit, THE Renderer SHALL recompute the Pixel_Scale each time the window is resized or the active map changes.
4. WHEN the scale mode is set to a fixed integer value of N, THE Renderer SHALL set the Camera_Projection scale to 1.0 divided by N, causing each game pixel to occupy N × N screen pixels.
5. WHEN Pixel_Scale is applied, THE Viewport SHALL display an area of (window_width / Pixel_Scale) × (window_height / Pixel_Scale) game pixels.
6. WHILE the camera follows the player, THE Renderer SHALL continue to clamp the camera position to map bounds using the scaled Viewport dimensions.
7. IF a fixed Pixel_Scale value is less than 1, THEN THE Renderer SHALL clamp the Pixel_Scale to a minimum value of 1.
8. IF the zoom-to-fit calculation results in a scale less than 1 (map larger than window at 1:1), THEN THE Renderer SHALL use a Pixel_Scale of 1 and allow the camera to scroll across the map.

### Requirement 2: Character Sprite Scaling

**User Story:** As a game creator, I want character sprites to be sized proportionally to the tile grid, so that characters look correct relative to the map tiles.

#### Acceptance Criteria

1. WHEN a player Character_Sprite is spawned from a Character_Spritesheet, THE Renderer SHALL apply a Sprite_Scale to the sprite transform so the sprite's width matches the map tile width.
2. WHEN an NPC Character_Sprite is spawned from a Character_Spritesheet, THE Renderer SHALL apply the same Sprite_Scale as the player so all characters appear at a consistent size.
3. THE Sprite_Scale SHALL be computed as the map tile width divided by the spritesheet sprite width (e.g., for 16-pixel tiles and 24-pixel-wide sprites, the scale is 16 / 24 = 0.667).
4. WHILE a Character_Sprite is rendered with Sprite_Scale applied, THE Character_Sprite SHALL maintain its original aspect ratio (both X and Y axes use the same scale factor).

### Requirement 3: Walk Animation Step Pattern

**User Story:** As a game creator, I want the character walk animation to follow a natural stepping pattern, so that movement looks like actual walking rather than a simple frame loop.

#### Acceptance Criteria

1. WHILE the player is moving between tiles, THE Renderer SHALL cycle through Animation_Frames in the sequence [0, 1, 2, 1] (left step, center, right step, center) rather than [0, 1, 2].
2. WHEN the player begins a tile-to-tile move, THE Renderer SHALL start the Walk_Cycle from frame 0 (left step).
3. WHEN the player completes a tile-to-tile move, THE Renderer SHALL return to frame 1 (center/Idle_Pose) regardless of where in the Walk_Cycle the animation was.
4. THE walk_animation_frame function SHALL accept elapsed time and Frame_Duration and return the correct frame index from the [0, 1, 2, 1] pattern, cycling continuously.
5. FOR ALL non-negative elapsed times and positive Frame_Duration values, THE walk_animation_frame function SHALL return a frame index that is one of 0, 1, or 2.

### Requirement 4: Idle Pose Display

**User Story:** As a game creator, I want characters to display a clear idle stance when not moving, so that the player can distinguish between moving and standing still.

#### Acceptance Criteria

1. WHILE the player character is stationary, THE Renderer SHALL display the center Animation_Frame (frame 1) for the player's current Facing_Direction.
2. WHILE an NPC is stationary, THE Renderer SHALL display the center Animation_Frame (frame 1) for the NPC's Facing_Direction.
3. WHEN the player transitions from moving to stationary, THE Renderer SHALL reset the animation timer to 0 and set the sprite to the Idle_Pose within a single frame.

### Requirement 5: Animation Speed Configuration

**User Story:** As a game creator, I want to configure the speed of walk animations, so that I can tune the visual feel of character movement.

#### Acceptance Criteria

1. THE Renderer SHALL provide a configurable AnimationConfig resource containing a Frame_Duration value defaulting to 0.15 seconds.
2. WHILE a Walk_Cycle is playing, THE Renderer SHALL advance to the next Animation_Frame after each Frame_Duration interval elapses.
3. WHEN the Frame_Duration is decreased, THE Walk_Cycle SHALL play faster (more frames per second).
4. IF the Frame_Duration is set to zero or a negative value, THEN THE Renderer SHALL clamp the Frame_Duration to a minimum positive value of 0.01 seconds.

### Requirement 6: Camera Bounds with Scaled Viewport

**User Story:** As a game creator, I want the camera to correctly stay within map bounds when pixel scaling is active, so that the viewport never shows empty space outside the map.

#### Acceptance Criteria

1. WHILE the Pixel_Scale is active and the map is larger than the scaled Viewport, THE Renderer SHALL clamp the camera so the Viewport edges do not extend beyond the map boundaries.
2. WHILE the Pixel_Scale is active and the map is smaller than the scaled Viewport in one or both axes, THE Renderer SHALL center the map within the Viewport along the smaller axis.
3. THE camera bounds calculation SHALL use the effective Viewport size (window_size / Pixel_Scale) rather than the raw window size.
4. WHEN zoom-to-fit mode is active, THE camera bounds calculation SHALL use the auto-computed Pixel_Scale value for determining the effective Viewport size.
