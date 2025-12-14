# Maestro Blocks - Music Theory Worksheet Builder

## Tech Stack
- Tauri (Rust backend)
- Svelte + TypeScript (frontend)
- Konva.js (canvas rendering)
- LilyPond (music notation CLI)
- Bun (package manager)

## Architecture
- Hybrid rendering: Konva for drag-and-drop UI, LilyPond for vector generation
- Local-first: Zero backend servers
- Rust commands handle LilyPond execution and file I/O

## Key Files
- `src-tauri/src/commands/lilypond.rs` - SVG rendering logic
- `src/lib/types/blocks.ts` - TypeScript interfaces
- `src/lib/components/` - Svelte UI components

## Development Commands
- `bun tauri dev` - Run dev server
- `bun tauri build` - Build production binary

## Project Structure
```
maestro-blocks/
├── src/
│   ├── lib/
│   │   ├── components/     # Svelte components
│   │   ├── stores/         # Svelte stores
│   │   ├── types/          # TypeScript interfaces
│   │   └── utils/          # Utility functions
│   ├── main.ts            # App entry point
│   ├── app.css            # Global styles
│   └── App.svelte         # Main component
├── src-tauri/
│   ├── src/
│   │   ├── commands/      # Rust commands
│   │   │   ├── mod.rs
│   │   │   └── lilypond.rs
│   │   └── main.rs        # Tauri main
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
└── package.json           # Node dependencies
```

## Current Status
✅ Day 1 Complete - Scaffold & Rust Bridge Working
- Tauri + Svelte initialized with Bun
- `render_lilypond` Rust command functional (tested)
- TypeScript types in place
- Directory structure ready

## Next: Day 2 - Canvas Implementation
- Setup svelte-konva Stage/Layer
- Create draggable MusicBlock component
- Integrate SVG rendering from Rust
- Implement basic drag-and-drop with position state

## Features
- Music block drag-and-drop interface
- LilyPond notation rendering
- Export to SVG/PDF
- Cross-platform desktop app