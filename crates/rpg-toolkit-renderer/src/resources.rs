use bevy::prelude::*;
use rpg_toolkit_common::{
    AbilityId, CharacterId, EntityTarget, EventAction, ItemId, MapId, MusicLoopId, ProjectFile,
    ScreenShakeMode, SpritesheetId, TilesetId, TransitionDirection, TransitionKind,
};
use std::collections::{HashMap, VecDeque};

/// Input resource: consumers insert this before adding the plugin.
/// Contains the deserialized project data and tileset texture handles.
#[derive(Resource)]
pub struct RendererProjectData {
    pub project_file: ProjectFile,
    pub tileset_textures: HashMap<TilesetId, Handle<Image>>,
    pub tileset_atlas_layouts: HashMap<TilesetId, Handle<TextureAtlasLayout>>,
    pub spritesheet_textures: HashMap<SpritesheetId, Handle<Image>>,
    pub spritesheet_atlas_layouts: HashMap<SpritesheetId, Handle<TextureAtlasLayout>>,
}

/// Runtime state managed by the plugin.
#[derive(Resource, Default)]
pub struct RendererState {
    pub active_map_id: Option<MapId>,
    /// Set to `Some(map_id)` when a map transition is requested.
    pub pending_map_change: Option<MapId>,
    /// Target coordinates for the pending map change (from JumpTo action).
    pub pending_target_coords: Option<(u32, u32)>,
    /// Target elevation for pending map change (from JumpTo).
    pub pending_target_elevation: Option<u32>,
}

/// Configuration for player movement animation.
#[derive(Resource)]
pub struct MovementConfig {
    /// Duration of tile-to-tile animation in seconds.
    pub move_duration: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            move_duration: 0.15,
        }
    }
}

/// The player's visual representation.
#[derive(Resource)]
pub struct PlayerVisual {
    pub color: Color,
}

impl Default for PlayerVisual {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.2, 0.6, 1.0),
        }
    }
}

/// Configuration for sprite walk animation timing.
#[derive(Resource)]
pub struct AnimationConfig {
    /// Duration of each animation frame in seconds.
    pub frame_duration: f32,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            frame_duration: 0.15,
        }
    }
}

impl AnimationConfig {
    /// Returns the frame duration, clamped to a minimum of 0.01 seconds.
    pub fn clamped_frame_duration(&self) -> f32 {
        self.frame_duration.max(0.01)
    }
}

/// What the ActionQueue is currently waiting for before advancing.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum WaitingFor {
    #[default]
    Nothing,
    Dialog,
    Selection,
    ScreenShake,
    Transition,
    EntityMove,
    CameraPan,
    Wait,
    Jump,
}

/// Tracks the remaining EventActions in the current trigger sequence.
/// Present only while a sequence is being processed.
#[derive(Resource)]
pub struct ActionQueue {
    /// The remaining actions to process (front = next action).
    pub actions: VecDeque<EventAction>,
    /// What blocking action the queue is currently waiting for.
    pub waiting_for: WaitingFor,
}

/// Tracks an active screen shake effect.
#[derive(Resource)]
pub struct ScreenShakeState {
    pub intensity: f32,
    pub mode: ScreenShakeMode,
    pub duration: f32,
    pub elapsed: f32,
}

/// Tracks an active shader-driven screen transition.
#[derive(Resource)]
pub struct TransitionState {
    /// Visual style of the transition (fade, mosaic, distortion wave, …).
    pub kind: TransitionKind,
    /// Whether the scene is being revealed (`In`) or obscured (`Out`).
    pub direction: TransitionDirection,
    /// Total animation duration in seconds (> 0 while a transition is active).
    pub duration: f32,
    /// Seconds elapsed since the transition started.
    pub elapsed: f32,
    /// RGBA cover color used by the effect.
    pub color: [f32; 4],
}

/// Persistent game state flags (key-value store).
#[derive(Resource, Default)]
pub struct GameState {
    pub flags: HashMap<String, String>,
}

/// Tracks the player's original spritesheet for restoration.
#[derive(Resource)]
pub struct PlayerAppearanceState {
    pub original_spritesheet_id: Option<SpritesheetId>,
}

/// Determines how the game world is scaled on screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PixelScaleMode {
    /// Automatically compute the largest integer scale where the
    /// entire map fits in the window.
    ZoomToFit,
    /// Use a fixed integer scale (clamped to >= 1).
    Fixed(u32),
}

/// Resource controlling pixel scaling of the game world.
#[derive(Resource)]
pub struct PixelScaleConfig {
    /// The scaling mode: zoom-to-fit or fixed integer.
    pub mode: PixelScaleMode,
    /// The currently computed effective integer scale (always >= 1).
    /// Updated each frame by `apply_pixel_scale`.
    pub effective_scale: u32,
}

impl Default for PixelScaleConfig {
    fn default() -> Self {
        Self {
            mode: PixelScaleMode::ZoomToFit,
            effective_scale: 1,
        }
    }
}

/// Runtime grid positions for all NPCs on the active map.
/// Updated each frame as NPCs move, used for dynamic collision checks.
#[derive(Resource, Default)]
pub struct NpcPositions {
    /// Maps npc_index → current grid position and elevation (x, y, elevation).
    pub positions: Vec<(u32, u32, u32)>,
}

impl NpcPositions {
    /// Returns `true` if any NPC occupies the tile at `(x, y)` regardless of elevation.
    pub fn is_occupied(&self, x: u32, y: u32) -> bool {
        self.positions.iter().any(|&(px, py, _)| px == x && py == y)
    }

    /// Returns `true` if any NPC at the given elevation occupies the tile at `(x, y)`.
    pub fn is_occupied_at_elevation(&self, x: u32, y: u32, elevation: u32) -> bool {
        self.positions
            .iter()
            .any(|&(px, py, pe)| px == x && py == y && pe == elevation)
    }

    /// Returns `true` if any NPC *other than* `exclude_index` occupies `(x, y)`.
    pub fn is_occupied_by_other(&self, x: u32, y: u32, exclude_index: usize) -> bool {
        self.positions
            .iter()
            .enumerate()
            .any(|(i, &(px, py, _))| i != exclude_index && px == x && py == y)
    }
}

/// Signals that the player pressed the action key (Space/Enter) this frame.
#[derive(Resource, Default)]
pub struct InteractionIntent {
    pub pressed: bool,
}

/// Signals that the player attempted to move onto a tile occupied by an NPC.
/// Populated by `player_movement` and consumed by `npc_trigger_system`.
/// Uses an Option field so it can be written via ResMut (immediate visibility)
/// rather than Commands (deferred until end of stage).
#[derive(Resource, Default)]
pub struct NpcCollisionEvent {
    /// The index of the NPC the player collided with (index into `map.npcs`),
    /// or None if no collision occurred this frame.
    pub npc_index: Option<usize>,
}

/// The path to the on-disk save file.
/// Inserted by the launcher before adding the renderer plugin.
#[derive(Resource)]
pub struct SavePath {
    pub path: std::path::PathBuf,
}

/// Player's current currency balance.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct CurrencyState {
    pub balance: u64,
}

/// Player's inventory: item_id → quantity held.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct InventoryState {
    pub items: HashMap<ItemId, u32>,
}

/// Per-character experience and learned abilities.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CharacterProgress {
    pub experience: u64,
    pub learned_abilities: Vec<AbilityId>,
}

/// Progress state for all characters.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct CharacterProgressState {
    pub characters: HashMap<CharacterId, CharacterProgress>,
}

/// Active party members (ordered list).
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct PartyState {
    pub members: Vec<CharacterId>,
}

// Re-export ActiveShopId from rpg-toolkit-common for backward compatibility.
pub use rpg_toolkit_common::ActiveShopId;

/// Tracks an active entity forced-move in progress.
#[derive(Resource)]
pub struct EntityMoveState {
    pub target: EntityTarget,
    pub target_x: u32,
    pub target_y: u32,
    pub speed: f32,
    pub current_x: f32,
    pub current_y: f32,
    pub complete: bool,
}

/// Tracks the current camera follow target.
#[derive(Resource)]
pub struct CameraFollowTarget {
    pub target: EntityTarget,
}

/// Tracks an active camera pan in progress.
#[derive(Resource)]
pub struct CameraPanState {
    pub start_x: f32,
    pub start_y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub duration: f32,
    pub elapsed: f32,
}

/// Tracks a Wait action in progress.
#[derive(Resource)]
pub struct WaitState {
    pub duration: f32,
    pub elapsed: f32,
}

/// Tracks an active jump animation in progress.
#[derive(Resource)]
pub struct JumpAnimState {
    pub start_x: u32,
    pub start_y: u32,
    pub landing_x: u32,
    pub landing_y: u32,
    pub distance: u32,
    pub duration: f32,
    pub elapsed: f32,
}

/// Speed scaling factor applied to player movement.
/// Default value is 1.0 (normal walk speed).
#[derive(Resource)]
pub struct SpeedMultiplier {
    pub value: f32,
}

impl Default for SpeedMultiplier {
    fn default() -> Self {
        Self { value: 1.0 }
    }
}

/// Marker resource indicating that intro events are currently playing.
#[derive(Resource)]
pub struct IntroEventsActive;

/// Tracks the previous frame's camera position for computing parallax deltas.
#[derive(Resource, Default)]
pub struct PreviousCameraPosition {
    pub position: Vec2,
}

// ---------------------------------------------------------------------------
// Audio playback
// ---------------------------------------------------------------------------

/// Marker component for the single active music-channel audio entity.
///
/// At most one entity carrying this marker represents the currently playing
/// music loop; outgoing cross-fade tracks are tracked separately in
/// [`MusicChannelState::fading_out`].
#[derive(Component)]
pub struct MusicChannel;

/// Marker component for a one-shot sound-effect audio entity.
///
/// Any number of these may exist concurrently; Bevy plays them in parallel and
/// despawns them when they finish.
#[derive(Component)]
pub struct SoundEffectChannel;

/// Describes an in-progress volume ramp applied to a music-channel sink.
///
/// A ramp linearly interpolates the sink volume from `start_volume` to
/// `end_volume` over `duration` seconds, tracked by `elapsed`. Fades are
/// advanced by `update_music_fades` each frame independently of the
/// `ActionQueue`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FadeRamp {
    /// Starting linear volume (0.0..=1.0).
    pub start_volume: f32,
    /// Target linear volume (0.0..=1.0).
    pub end_volume: f32,
    /// Total fade duration in seconds (> 0.0).
    pub duration: f32,
    /// Seconds elapsed so far.
    pub elapsed: f32,
}

impl FadeRamp {
    /// Creates a new ramp from `start_volume` to `end_volume` over `duration`.
    pub fn new(start_volume: f32, end_volume: f32, duration: f32) -> Self {
        Self {
            start_volume,
            end_volume,
            duration,
            elapsed: 0.0,
        }
    }

    /// Returns the interpolated linear volume at the current `elapsed`.
    pub fn current_volume(&self) -> f32 {
        if self.duration <= 0.0 {
            return self.end_volume;
        }
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        self.start_volume + (self.end_volume - self.start_volume) * t
    }

    /// Returns `true` once the ramp has run for at least its full duration.
    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// An outgoing track being cross-faded out and then despawned.
///
/// Holds the entity of the outgoing music sink and the ramp driving its volume
/// to zero.
pub struct FadingTrack {
    /// The outgoing music-channel entity.
    pub entity: Entity,
    /// The fade-out ramp driving this track's volume to zero.
    pub ramp: FadeRamp,
}

/// The currently playing music loop on the music channel.
pub struct MusicPlayback {
    /// Identifier of the music loop that is playing.
    pub music_loop_id: MusicLoopId,
    /// The audio entity carrying the `MusicChannel` marker.
    pub entity: Entity,
    /// A fade-in / cross-fade-in ramp in progress, if any.
    pub fade_in: Option<FadeRamp>,
    /// Number of times to play before stopping. `None` = infinite.
    pub loop_count: Option<u32>,
    /// Number of loop repetitions completed so far.
    pub loops_completed: u32,
    /// Fade-out duration in seconds applied at the end of playback.
    pub fade_out_duration: f32,
    /// The end-of-playback fade-out ramp, once it has begun.
    pub fade_out: Option<FadeRamp>,
    /// Last observed sink playback position (seconds), used to detect loop
    /// wrap-around for finite loop counting.
    pub last_position: f32,
    /// Best-known loop duration in seconds, learned from the peak playback
    /// position observed just before a wrap-around. `0.0` until the first
    /// wrap is observed. Used to begin a finite-loop fade-out early enough that
    /// it completes as the final repetition ends (Req 6.9).
    pub loop_duration: f32,
}

/// Runtime state of the single music channel.
///
/// Registered by the renderer plugin. Tracks the current playback (if any) and
/// any outgoing tracks that are still fading out during a cross-fade.
#[derive(Resource, Default)]
pub struct MusicChannelState {
    /// The currently playing music loop, if any.
    pub current: Option<MusicPlayback>,
    /// Outgoing tracks still fading out from a cross-fade.
    pub fading_out: Vec<FadingTrack>,
}

/// The decision produced by [`next_music_command`] for a `PlayMusic` request.
///
/// This is a pure classification of what the music channel should do; the
/// action-queue system carries it out against the live Bevy sinks.
#[derive(Clone, Debug, PartialEq)]
pub enum MusicCommand {
    /// Leave the music channel untouched (same id already playing/fading in,
    /// or the requested id is not registered).
    NoChange,
    /// Start the requested loop immediately at full volume, stopping any
    /// previous track in the same step (no fade).
    StartImmediate,
    /// Fade the new loop in from zero to full while fading the previous track
    /// out to zero over `fade_duration` seconds.
    CrossFade,
    /// Fade the new loop in from zero to full over `fade_duration` seconds
    /// (no previous track playing).
    FadeIn,
}

/// Pure decision function for a `PlayMusic` request against the music channel.
///
/// Returns [`MusicCommand::NoChange`] when the requested `music_loop_id`:
/// - is not registered in the project (`registered == false`), or
/// - equals the id currently playing or currently fading in on the channel.
///
/// Otherwise returns the appropriate start decision based on `fade_duration`
/// and whether a track is already playing:
/// - `fade_duration == 0.0` → [`MusicCommand::StartImmediate`]
/// - `fade_duration > 0.0` with a previous track → [`MusicCommand::CrossFade`]
/// - `fade_duration > 0.0` with no previous track → [`MusicCommand::FadeIn`]
///
/// This function performs no I/O and mutates nothing, so it is exercised
/// directly by property tests.
pub fn next_music_command(
    state: &MusicChannelState,
    requested_id: &str,
    fade_duration: f32,
    registered: bool,
) -> MusicCommand {
    if !registered {
        return MusicCommand::NoChange;
    }

    // A request matching the currently playing (or currently fading-in) id is a
    // no-op regardless of fade_duration.
    if let Some(current) = &state.current
        && current.music_loop_id == requested_id
    {
        return MusicCommand::NoChange;
    }

    if fade_duration > 0.0 {
        if state.current.is_some() {
            MusicCommand::CrossFade
        } else {
            MusicCommand::FadeIn
        }
    } else {
        MusicCommand::StartImmediate
    }
}
