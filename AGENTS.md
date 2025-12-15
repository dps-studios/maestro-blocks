# Maestro Blocks Architecture Documentation

## Overview

Maestro Blocks is a cross-platform desktop music theory worksheet builder built with Tauri (Rust backend) and SolidJS + VexFlow (Frontend). The application allows users to create and edit music notation worksheets with a focus on chord identification exercises.

## Backend (Rust) Responsibilities

### Core Processing
- **Music theory computations** — Chord generation, interval calculations, voice leading
- **File I/O operations** — Read/write project files, manage temp directories  
- **SVG/PDF generation** — Convert notation to final formats
- **Image optimization** — PNG compression for social media exports
- **File system operations** — Save/load project files, template management

### System Integration
- **Native dialogs** — File picker, save dialogs (via Tauri)
- **Clipboard operations** — Copy rendered notation
- **Print queue management** — Direct-to-printer functionality

### Performance-Critical
- **Caching layer** — Store rendered SVGs to avoid re-rendering unchanged blocks
- **Batch processing** — Queue multiple renders efficiently
- **Background jobs** — Non-blocking export operations

### Current Implementation
```rust
// src-tauri/src/commands/music.rs
#[tauri::command]
pub async fn generate_chord_pitches(request: ChordRequest) -> Result<ChordResponse, String> {
    // Generates chord pitches based on root, quality, inversion
    // Returns pitches and display name
}
```

---

## Frontend (SolidJS + VexFlow) Responsibilities

### UI/UX Layer
- **VexFlow rendering** — Music notation via VexFlow library
- **Reactive state** — Fine-grained reactivity with SolidJS stores
- **Property panels** — Input fields for chord selection, clef changes
- **Selection state** — Element selection and editing

### Application State
- **Score store** — Sections, measures, elements using SolidJS `createStore`
- **Editor store** — Active tool, cursor position, selection state
- **Document state** — Page size, margins, title metadata
- **Theme state** — Dark/light mode via signals

### User Interactions
- **Ghost note preview** — Shows chord placement position before clicking
- **Debounced updates** — Prevents excessive re-renders during user input
- **Keyboard shortcuts** — Standard editing operations

### Current Implementation

#### Store Architecture
```typescript
// src/lib/stores/score.ts
import { createStore } from 'solid-js/store';

const [score, setScore] = createStore<Score>(initialScore);

export const scoreStore = {
  get state() { return score; },
  addSection: () => setScore('sections', sections => [...sections, newSection]),
  toggleAnswers: () => setScore('showAnswers', show => !show)
};
```

#### Component Hierarchy
```
App.tsx
├── Sidebar.tsx (tool palette, controls)
└── ScoreCanvas.tsx (VexFlow rendering)
    ├── Section containers
    ├── VexFlow SVG output
    └── Ghost note overlay
```

#### Music Element Architecture
```typescript
interface ChordElement {
  id: string;
  type: 'chord';
  pitches: Pitch[];
  duration: Duration;
  chordDef: ChordDefinition;
  displayName: string;
}
```

---

## Communication Pattern

### Core Principle
**Frontend owns layout; Backend owns music theory.**

- Frontend handles all UI state and rendering
- Backend handles chord generation and music theory computations
- Communication happens only when music theory computation is needed

### Data Flow
```typescript
// Frontend requests chord pitches from Rust
const { pitches, displayName } = await generateChordPitchesRust(
  chordDef,
  rootOctave
);

// Frontend renders using VexFlow
renderSection(section, showAnswers, timeSignature, keySignature);
```

---

## File Structure

```
maestro-blocks/
├── src/
│   ├── lib/
│   │   ├── components/           # SolidJS TSX components
│   │   │   ├── ScoreCanvas.tsx   # Main VexFlow canvas
│   │   │   ├── Sidebar.tsx       # Tool palette, controls
│   │   │   ├── WorksheetViewer.tsx
│   │   │   ├── ChordNamingGenerator.tsx
│   │   │   └── ChordPalette.tsx
│   │   ├── stores/              # SolidJS state management
│   │   │   ├── score.ts         # Score and editor state
│   │   │   └── worksheet.ts     # Worksheet generation
│   │   ├── hooks/               # Custom hooks
│   │   │   └── useTheme.ts      # Dark/light mode
│   │   ├── services/            # External integrations
│   │   │   ├── music.ts         # Rust backend bridge
│   │   │   └── vexflow.ts       # VexFlow rendering
│   │   ├── types/               # TypeScript interfaces
│   │   │   ├── score.ts         # Music types
│   │   │   └── worksheet.ts     # Worksheet types
│   │   ├── utils/               # Helper functions
│   │   │   ├── debounce.ts
│   │   │   ├── real-time-updates.ts
│   │   │   ├── notation-templates.ts
│   │   │   └── tauri-api.ts
│   │   └── styles/
│   │       └── design-system.css
│   ├── index.tsx                # App entry point
│   ├── App.tsx                  # Main component
│   └── app.css                  # Global styles
├── src-tauri/
│   ├── src/
│   │   ├── commands/            # Rust commands  
│   │   │   ├── mod.rs
│   │   │   ├── music.rs         # Chord generation
│   │   │   └── worksheet.rs     # Worksheet generation
│   │   ├── music/               # Music theory logic
│   │   │   ├── chords.rs
│   │   │   ├── intervals.rs
│   │   │   └── notes.rs
│   │   └── main.rs              # Tauri main
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri configuration
└── package.json                 # Node dependencies
```

---

## Development Commands

### Frontend Development
```bash
bun run dev           # Start Vite dev server
bun run build         # Build for production  
```

### Tauri Development  
```bash
bun run tauri:dev     # Start desktop app with hot reload
bun run tauri:build   # Build desktop binary
```

---

## Performance Considerations

### Frontend Optimizations
1. **Fine-grained reactivity** - SolidJS only updates what changes
2. **Debounced Updates** - 300ms delay before calling Rust backend
3. **VexFlow caching** - Rendered SVGs cached per section

### Backend Optimizations  
1. **Background Processing** - Don't block UI during computation
2. **Batch Operations** - Queue multiple renders together

### Memory Management
1. **Component Lifecycle** - Proper cleanup with `onCleanup`
2. **Store efficiency** - SolidJS stores use proxies for minimal updates

---

## Development Guidelines

### SolidJS Reactivity Pitfalls

**IMPORTANT: Deep store changes don't trigger `createEffect` with `on()` dependencies**

When using `createStore` with `produce()` for nested updates, the top-level array/object reference stays the same. This means:

```typescript
// THIS WON'T WORK - sections() reference doesn't change on deep updates
createEffect(
  on(
    () => [sections(), showAnswers()] as const,
    () => renderAllSections()
  )
);

// After this, the effect does NOT fire because sections array ref is unchanged:
setScore(produce((s) => {
  s.sections[0].staff.measures[0].elements = [newChord];
}));
```

**Solution: Explicitly call render functions after store mutations**

```typescript
// In the store method or after awaiting it:
await scoreStore.addChordToMeasure(...);
renderAllSections(); // Explicitly trigger re-render

// Or use a render version signal:
const [renderVersion, setRenderVersion] = createSignal(0);
// Include renderVersion() in effect dependencies
// Increment after mutations: setRenderVersion(v => v + 1)
```

**Other SolidJS patterns to follow:**
- Wrap module-level signals in `createRoot()` to avoid "computations created outside createRoot" warnings
- Use `onCleanup` for cleanup in effects
- Prefer explicit re-render calls over complex dependency tracking for imperative operations like canvas rendering

### Code Style
- **TypeScript First** - All new code must be fully typed
- **SolidJS Patterns** - Use signals, stores, effects appropriately
- **Component Architecture** - Single responsibility, clear props
- **Error Handling** - User-friendly error messages, fallback states

### Performance Targets
- **Render Time** - VexFlow rendering < 100ms per section
- **Memory Usage** - < 200MB with large worksheets
- **Startup Time** - Desktop app ready < 3 seconds

This architecture supports the core requirement of a music theory worksheet builder while maintaining clean separation between frontend rendering and backend music theory computations.
