# Maestro Blocks - Music Theory Worksheet Builder

## Tech Stack
- Tauri (Rust backend)
- SolidJS + TypeScript (frontend)
- VexFlow (music notation rendering)
- Bun (package manager)

## Architecture
- VexFlow for music notation rendering
- SolidJS for reactive UI with fine-grained updates
- Local-first: Zero backend servers
- Rust commands handle music theory computations

## Key Files
- `src-tauri/src/commands/music.rs` - Chord generation logic
- `src/lib/types/score.ts` - TypeScript interfaces
- `src/lib/components/` - SolidJS TSX components
- `src/lib/stores/` - SolidJS state management

## Development Commands
- `bun tauri dev` - Run dev server
- `bun tauri build` - Build production binary

## Project Structure
```
maestro-blocks/
├── src/
│   ├── lib/
│   │   ├── components/     # SolidJS TSX components
│   │   ├── stores/         # SolidJS stores
│   │   ├── hooks/          # Custom hooks (useTheme)
│   │   ├── services/       # VexFlow, Tauri bridge
│   │   ├── types/          # TypeScript interfaces
│   │   └── utils/          # Utility functions
│   ├── index.tsx           # App entry point
│   ├── App.tsx             # Main component
│   └── app.css             # Global styles
├── src-tauri/
│   ├── src/
│   │   ├── commands/       # Rust commands
│   │   ├── music/          # Music theory logic
│   │   └── main.rs         # Tauri main
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
└── package.json            # Node dependencies
```

## Features
- Music notation rendering via VexFlow
- Chord identification worksheets
- Export to SVG/PDF
- Cross-platform desktop app
- Dark/light mode support
