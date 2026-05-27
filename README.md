# RPG Toolkit

A Rust / Bevy toolkit for creating and playing retro-style, story-driven adventure games. It includes a visual map editor for building worlds and a runtime engine for playing them — with dialog, event triggers, NPC behavior, and more.

## Features

### Editor
- Multi-layer tilemap painting with a tileset palette
- Tools: paint, erase, flood fill, stamp brush, and line-drag (Ctrl+click)
- Tile attributes: opacity, event triggers, elevation, and elevation transitions
- NPC placement and patrol configuration
- Spawn point assignment
- Event trigger editor: show dialog, jump maps, screen shake, fade transitions, set state flags
- Multi-map project management with undo/redo
- Dialog text panel for managing in-game text entries

### Renderer
- Player movement with grid-to-world animation
- Camera with zoom-to-fit and pixel-perfect scaling
- Multi-layer tile rendering with z-sorting
- NPC patrol behavior (loop and random wander)
- Interaction-triggered and collision-triggered event systems
- Dialog system with typewriter effect, configurable position, and text speed
- Screen shake and fade transition effects
- Map transitions with coordinate and elevation support
- Game state flags for conditional storytelling

### Project Format
- Human-readable JSON with full validation on load
- Multi-map, multi-tileset projects in a single file
- Referenced dialog text entries (by ID)
- Spritesheet registry with dimension validation

## Getting Started

### Prerequisites
- Rust toolchain (edition 2024, workspace resolver 3)
- Bevy 0.18, egui via `bevy_egui`

### Build

```bash
cargo build
```

### Run the Editor

```bash
cargo run -p rpg-toolkit-editor
```

### Run a Project

```bash
cargo run -p rpg-toolkit-launcher -- <path-to-project.rpg> [--scale <N|fit>]
```

The `--scale` flag overrides the default zoom-to-fit behavior. Use `fit` for pixel-perfect scaling or an integer for a fixed scale factor.

## Architecture

```
crates/
  rpg-toolkit-common     Shared data types and serialization
  rpg-toolkit-editor     Visual map editor (egui + Bevy)
  rpg-toolkit-renderer   Game runtime engine
  rpg-toolkit-launcher   CLI entry point for playing projects
  rpg-toolkit-asset-gen  Asset generation tool
```

The project is designed to be extensible: the renderer is a Bevy plugin that can be augmented with custom systems for game-specific logic.
