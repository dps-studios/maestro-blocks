# Maestro Blocks Architecture Documentation

## Overview

Maestro Blocks is a cross-platform desktop music theory worksheet builder built with Tauri (Rust backend) and Svelte + Konva (Frontend). The application allows users to create, drag, and edit music notation blocks using LilyPond for rendering.

## Backend (Rust) Responsibilities

### Core Processing
- **LilyPond execution** — Spawn process, handle stdin/stdout, error parsing
- **File I/O operations** — Read/write `.ly` files, manage temp directories  
- **SVG/PDF generation** — Convert LilyPond output to final formats
- **Image optimization** — PNG compression for social media exports
- **File system operations** — Save/load project files, template management

### System Integration
- **Native dialogs** — File picker, save dialogs (via Tauri)
- **Clipboard operations** — Copy rendered notation
- **Print queue management** — Direct-to-printer functionality
- **License validation** — Check/verify purchase keys (if implementing DRM)

### Performance-Critical
- **Caching layer** — Store rendered SVGs to avoid re-rendering unchanged blocks
- **Batch processing** — Queue multiple LilyPond renders efficiently
- **Background jobs** — Non-blocking export operations

### Current Implementation
```rust
// src-tauri/src/commands/lilypond.rs
#[tauri::command]
pub async fn render_lilypond(notation: String) -> Result<String, String> {
    // Creates temp directory
    // Writes .ly file
    // Executes lilypond --svg
    // Returns SVG string
}
```

### Future Enhancement: Embedded LilyPond
To address LilyPond installation dependency, consider:
1. **Bundle LilyPond binary** with Tauri app
2. **Use LilyPond WebAssembly** compilation
3. **Implement LilyPond.js** JavaScript port
4. **Cross-platform package managers** integration

---

## Frontend (Svelte + Konva) Responsibilities

### UI/UX Layer
- **Canvas rendering** — All Konva.js drag-and-drop interactions
- **Block positioning** — Grid snapping to staff lines, alignment guides, z-index management
- **Property panels** — Input fields for text, transposition, clef changes
- **Undo/redo stack** — Command pattern for user actions
- **Selection state** — Multi-select, copy/paste, delete operations

### Application State
- **Block data structure** — Store positions, properties, types in Svelte stores
- **Document state** — Page size, margins, title metadata
- **UI state** — Active tool, zoom level, ruler visibility
- **Template library** — Browse/preview/apply pre-made layouts

### User Interactions
- **Debounced updates** — Wait 300ms after text input before calling Rust
- **Preview rendering** — Show low-res placeholder while LilyPond processes
- **Validation** — Check for invalid notation syntax before sending to backend
- **Keyboard shortcuts** — Ctrl+Z, Ctrl+C, arrow keys for nudging

### Current Implementation

#### Store Architecture
```typescript
// src/lib/stores/canvas.ts
export const canvasStore = {
  subscribe,     // Reactive block array
  addBlock,       // Create new blocks with unique IDs
  updateBlock,    // Modify block properties
  removeBlock,    // Delete blocks
  getBlock        // Retrieve specific block
};
```

#### Component Hierarchy
```
App.svelte
├── Canvas.svelte (Konva Stage/Layer)
│   ├── Background Grid (staff line snapping)
│   └── MusicBlock.svelte (draggable notation blocks)
│       ├── SVG Rendering (via Rust backend)
│       ├── Loading States
│       └── Drag Handle Overlay
└── Toolbar.svelte (Add/Remove blocks)
```

#### Music Block Architecture
```typescript
interface MusicBlock {
  id: string;           // UUID for unique identification
  type: BlockType;      // 'note' | 'chord' | 'staff' | etc.
  x: number;           // Canvas X position (snapped to grid)
  y: number;           // Canvas Y position (snapped to grid)
  width: number;       // Auto-calculated from SVG dimensions
  height: number;      // Auto-calculated from SVG dimensions  
  notation: string;    // LilyPond notation string
  svgContent?: string; // Cached SVG from Rust backend
  isRendered: boolean; // Track render state
}
```

---

## Communication Pattern

### Core Principle
**Frontend owns layout; Backend owns rendering.**

- Frontend never parses LilyPond syntax
- Backend never knows about canvas coordinates  
- Communication happens only when notation content changes, not on drag/resize

### Data Flow
```typescript
// Frontend stores block data
const block = {
  id: 'block-1',
  type: 'staff',
  x: 100, y: 200,           // Frontend responsibility
  lilypondCode: '\\relative c\' { c4 d e f }',
  cachedSvg: null           // Filled by Rust response
}

// Frontend calls Rust when needed
const svg = await invoke('render_lilypond', { 
  notation: block.lilypondCode  // Send notation only
})

block.cachedSvg = svg  // Store result, render in Konva
```

### Event Architecture
```typescript
// MusicBlock Component Events
on:rendered    → Updates store with cached SVG
on:moved       → Updates store with snapped position  
on:selected    → Highlights block for property editing

// Canvas Events  
on:canvasclick → Deselects blocks
on:drag        → Updates position in real-time
```

---

## File Structure

```
maestro-blocks/
├── src/
│   ├── lib/
│   │   ├── components/           # Svelte UI components
│   │   │   ├── Canvas.svelte     # Main workspace (1200x800)
│   │   │   ├── MusicBlock.svelte # Draggable notation blocks
│   │   │   └── Toolbar.svelte    # Add/remove controls
│   │   ├── stores/              # Svelte state management
│   │   │   └── canvas.ts        # Block CRUD operations
│   │   ├── types/               # TypeScript interfaces
│   │   │   └── blocks.ts        # MusicBlock, BlockType, etc.
│   │   └── utils/               # Helper functions
│   │       ├── svg-to-image.ts  # SVG → Konva Image conversion
│   │       └── debounce.ts      # Prevent excessive Rust calls
│   ├── main.ts                  # App entry point
│   ├── app.css                  # Global styles
│   └── App.svelte               # Main component
├── src-tauri/
│   ├── src/
│   │   ├── commands/            # Rust commands  
│   │   │   ├── mod.rs
│   │   │   └── lilypond.rs      # SVG rendering logic
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
bun run check         # Svelte type checking
```

### Tauri Development  
```bash
bun run tauri:dev     # Start desktop app with hot reload
bun run tauri:build   # Build desktop binary
```

### Testing
```bash
bun run test          # Run unit tests
bun run lint          # Code linting
```

---

## Performance Considerations

### Frontend Optimizations
1. **Block Caching** - Store rendered SVGs to prevent re-renders
2. **Virtual Scrolling** - Only render visible blocks for large canvases
3. **Debounced Updates** - 300ms delay before calling Rust backend
4. **Lazy Loading** - Render SVGs only when blocks come into view

### Backend Optimizations  
1. **Background Processing** - Don't block UI during LilyPond execution
2. **Batch Operations** - Queue multiple renders together
3. **Temp File Management** - Clean up temporary files promptly
4. **Error Caching** - Cache failed renders to prevent repeated attempts

### Memory Management
1. **Blob URL Cleanup** - Revoke object URLs after image loading
2. **Component Lifecycle** - Proper cleanup in onDestroy
3. **Store Subscriptions** - Unsubscribe to prevent memory leaks
4. **Canvas State** - Clear unused Konva nodes

---

## Known Limitations & Future Work

### Current Limitations
1. **LilyPond Dependency** - Requires external LilyPond installation
2. **Single Selection** - No multi-select functionality yet
3. **No Property Panel** - Can't edit notation after creation
4. **No Export** - Can't save/print worksheets yet
5. **Limited Block Types** - Only basic notation blocks

### Day 3 Priorities
1. **Property Panel** - Click blocks to edit notation
2. **Real-time Preview** - Show notation changes as you type
3. **Selection System** - Visual feedback for active blocks
4. **Keyboard Shortcuts** - Delete, copy, paste operations
5. **Basic Export** - Save canvas as SVG/PNG

### Future Enhancements  
1. **Multi-select** - Shift+click for multiple blocks
2. **Advanced Notation** - Dynamics, lyrics, chord symbols
3. **Template Library** - Pre-made worksheet layouts
4. **Audio Playback** - MIDI generation from notation
5. **Collaboration** - Real-time multi-user editing
6. **Cloud Sync** - Save projects to cloud storage

---

## Development Guidelines

### Code Style
- **TypeScript First** - All new code must be fully typed
- **Svelte Patterns** - Use reactive statements, proper lifecycle
- **Component Architecture** - Single responsibility, clear props/events
- **Error Handling** - User-friendly error messages, fallback states

### Testing Strategy
- **Unit Tests** - Store operations, utility functions
- **Component Tests** - MusicBlock rendering, drag behavior
- **Integration Tests** - Full canvas workflows
- **E2E Tests** - User journeys from toolbar to export

### Performance Targets
- **Canvas FPS** - Maintain 60fps with 50+ blocks
- **Render Time** - LilyPond calls < 2 seconds average
- **Memory Usage** - < 200MB with 100 blocks
- **Startup Time** - Desktop app ready < 3 seconds

This architecture supports the core requirement of a drag-and-drop music notation builder while maintaining clean separation between frontend layout management and backend rendering responsibilities.