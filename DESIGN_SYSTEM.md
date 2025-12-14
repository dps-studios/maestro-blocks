# Maestro Blocks Design System

A paper-and-ink manuscript aesthetic inspired by aged manuscript paper, leather-bound music books, and traditional music notation.

## Overview

This design system prioritizes:
- **Elegant typography** with serif fonts for musical content
- **Muted, scholarly color palettes** over modern saturated UI
- **Warm paper tones** and deep ink colors
- **Subtle animations** for tactile feedback
- **Full dark mode support** with appropriate contrast

---

## File Structure

```
src/
├── lib/
│   ├── styles/
│   │   └── design-system.css    # All design tokens and utility classes
│   └── theme.svelte.ts          # Dark/light mode state management
└── app.css                       # Global styles importing design system
```

---

## Typography

### Font Families

| Token | Value | Usage |
|-------|-------|-------|
| `--font-serif` | Libre Baskerville, Georgia, serif | Musical notation, titles, chord names |
| `--font-sans` | Inter, system-ui, sans-serif | UI elements, labels, hints |

### Usage Guidelines

```css
/* Musical content - always use serif */
.chord-name {
  font-family: var(--font-serif);
}

/* UI elements - use sans-serif */
.button-label {
  font-family: var(--font-sans);
}
```

### Font Scale

| Token | Size | Usage |
|-------|------|-------|
| `--text-xs` | 12px | Hints, status bar |
| `--text-sm` | 14px | Buttons, secondary text |
| `--text-base` | 16px | Body text |
| `--text-lg` | 18px | Section titles |
| `--text-xl` | 20px | Page titles |
| `--text-2xl` | 24px | Large headings |

### Font Weights

| Token | Value | Usage |
|-------|-------|-------|
| `--font-normal` | 400 | Body text, serif headings |
| `--font-medium` | 500 | Emphasized text |
| `--font-semibold` | 600 | Section headers, labels |
| `--font-bold` | 700 | Strong emphasis |

---

## Color System

### Paper & Ink Tones

The foundation of the design system. Use these for all primary surfaces and text.

#### Light Mode

| Token | Hex | Usage |
|-------|-----|-------|
| `--color-paper` | #FAF8F3 | Primary background |
| `--color-paper-dark` | #F5F2EA | Secondary background, sidebar |
| `--color-paper-darker` | #EBE7DC | Tertiary background |
| `--color-ink` | #2C2416 | Primary text |
| `--color-ink-light` | #5A4D3A | Secondary text |
| `--color-ink-muted` | #8B7D6B | Disabled text, hints |
| `--color-accent-line` | #C4B8A4 | Borders, dividers |
| `--color-accent-gold` | #B8956C | Focus rings, highlights |

#### Dark Mode

| Token | Hex | Usage |
|-------|-----|-------|
| `--color-paper` | #1C1914 | Primary background |
| `--color-paper-dark` | #141210 | Secondary background |
| `--color-ink` | #E8E4DC | Primary text |
| `--color-ink-light` | #B8B0A0 | Secondary text |
| `--color-accent-gold` | #C9A86C | Focus rings, highlights |

### Tier Colors

Used for categorizing chord qualities by complexity/rarity.

| Tier | Name | Light Mode | Dark Mode | Usage |
|------|------|------------|-----------|-------|
| 0 | Safe | #5B7C5D | #6B9C6D | Major/minor triads - foundational |
| 1 | Colorful | #9C6B3C | #C88B4C | 7th chords, suspended - adds color |
| 2 | Bold | #8B3A4C | #B85A6C | Diminished, augmented - distinctive |
| 3 | Neutral | #6B6B6B | #8A8A8A | Uncolored/default |

Each tier includes background and border variants:
```css
--color-tier-safe-bg: rgba(91, 124, 93, 0.08);
--color-tier-safe-border: rgba(91, 124, 93, 0.25);
```

### Using Colors

Always use CSS custom properties, never hardcoded values:

```css
/* Correct */
.card {
  background-color: var(--color-paper);
  color: var(--color-ink);
  border: 1px solid var(--color-accent-line);
}

/* Incorrect - breaks dark mode */
.card {
  background-color: #FAF8F3;
  color: #2C2416;
}
```

---

## Spacing System

Base unit: **4px**

| Token | Value | Common Usage |
|-------|-------|--------------|
| `--space-0-5` | 2px | Tight gaps |
| `--space-1` | 4px | Icon padding |
| `--space-1-5` | 6px | Button vertical padding |
| `--space-2` | 8px | Button horizontal padding, small gaps |
| `--space-3` | 12px | Card padding, section gaps |
| `--space-4` | 16px | Container padding |
| `--space-6` | 24px | Large section gaps |
| `--space-8` | 32px | Page margins |

### Usage Pattern

```css
.card {
  padding: var(--space-3);
  margin-bottom: var(--space-4);
}

.button {
  padding: var(--space-1-5) var(--space-3);
  gap: var(--space-2);
}
```

---

## Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| `--radius-sm` | 4px | Small buttons, tags |
| `--radius-md` | 6px | Buttons, inputs |
| `--radius-lg` | 8px | Cards, panels |
| `--radius-xl` | 12px | Modal dialogs |
| `--radius-full` | 9999px | Pills, circular elements |

---

## Shadows

| Token | Usage |
|-------|-------|
| `--shadow-sm` | Subtle elevation |
| `--shadow-md` | Cards, dropdowns |
| `--shadow-lg` | Modals, overlays |
| `--shadow-card` | Standard card elevation |

---

## Transitions

| Token | Duration | Usage |
|-------|----------|-------|
| `--transition-fast` | 100ms | Hover states |
| `--transition-base` | 150ms | Most interactions |
| `--transition-slow` | 300ms | Page transitions |
| `--transition-colors` | Combined | Color-only transitions |

### Usage Pattern

```css
.button {
  transition: var(--transition-colors), transform var(--transition-fast);
}

.button:hover {
  transform: scale(1.01);
}
```

---

## Component Patterns

### Buttons

Three button variants are available:

```html
<!-- Primary: Ink background, paper text -->
<button class="btn btn-primary">Add Section</button>

<!-- Secondary: Paper background, ink text, border -->
<button class="btn btn-secondary">Cancel</button>

<!-- Ghost: Transparent, no border -->
<button class="btn btn-ghost">Settings</button>
```

### Cards

```html
<div class="card">
  <h3 class="section-header">Card Title</h3>
  <!-- Content -->
</div>
```

### Section Headers

```html
<h3 class="section-header">WORKSHEET</h3>
```

Renders as: uppercase, small text, wide letter-spacing, muted color.

### Paper Texture

Add the `.paper-texture` class to apply a subtle grain overlay:

```html
<div class="main-content paper-texture">
  <!-- Content with paper grain effect -->
</div>
```

---

## Interactive States

### Hover

```css
.interactive-element:hover {
  transform: scale(1.01);
  background-color: var(--color-hover-overlay);
}
```

### Focus

```css
.interactive-element:focus-visible {
  outline: 2px solid var(--color-accent-gold);
  outline-offset: 2px;
}
```

### Disabled

```css
.button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}
```

---

## Dark Mode

### Theme Toggle

Use the theme store for reactive dark/light mode:

```typescript
import { theme } from '$lib/theme.svelte';

// Toggle theme
theme.toggle();

// Check current theme
if (theme.isDark) {
  // Dark mode active
}

// Set explicitly
theme.setTheme(true);  // Dark
theme.setTheme(false); // Light

// Revert to system preference
theme.useSystemPreference();
```

### CSS Implementation

The dark mode is applied via a `.dark` class on the `<html>` element. All color tokens automatically switch values:

```css
:root {
  --color-paper: #FAF8F3;  /* Light mode */
}

:root.dark {
  --color-paper: #1C1914;  /* Dark mode */
}
```

### Preventing Flash

The `index.html` includes a blocking script that applies the theme before first paint:

```html
<script>
  var theme = localStorage.getItem('theme');
  if (theme === 'dark' || (!theme && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
    document.documentElement.classList.add('dark');
  }
</script>
```

---

## Accessibility

### Touch Targets

Minimum touch target size: **32x32px** (`w-8 h-8`)

```css
.icon-button {
  min-width: 32px;
  min-height: 32px;
}
```

### Focus Visibility

Always use `focus-visible` for keyboard-only focus states:

```css
button:focus-visible {
  outline: 2px solid var(--color-accent-gold);
  outline-offset: 2px;
}
```

### Reduced Motion

Animations are disabled for users who prefer reduced motion:

```css
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

---

## Animations

### Available Keyframes

| Animation | Usage |
|-----------|-------|
| `spin` | Loading spinners |
| `fadeIn` | Content appearing |
| `scaleIn` | Modal dialogs |
| `shimmer` | Active/playing states (light mode) |
| `shimmerDark` | Active/playing states (dark mode) |
| `pulse` | Loading indicators |

### Spinner Component

```html
<div class="spinner"></div>
<div class="spinner spinner-sm"></div>
<div class="spinner spinner-lg"></div>
```

---

## Icons

All icons use inline SVG with consistent specifications:

```html
<svg 
  width="18" 
  height="18" 
  viewBox="0 0 24 24" 
  fill="none" 
  stroke="currentColor" 
  stroke-width="2" 
  stroke-linecap="round" 
  stroke-linejoin="round"
>
  <!-- paths -->
</svg>
```

### Icon Sizes

| Context | Size |
|---------|------|
| Header controls | 18px |
| Button icons | 16px |
| Status bar | 12px |
| Empty states | 48-64px |

---

## Best Practices

### Do

- Use CSS custom properties for all colors
- Apply `paper-texture` to main content areas
- Use serif fonts for musical content
- Include hover and focus states on all interactive elements
- Test in both light and dark modes

### Don't

- Hardcode color hex values
- Use bright, saturated colors
- Skip focus states on interactive elements
- Forget to test responsive behavior
- Use emojis (prefer SVG icons)

---

## Quick Reference

### Common Patterns

```css
/* Card with standard styling */
.my-card {
  background-color: var(--color-paper);
  border: 1px solid var(--color-accent-line);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  box-shadow: var(--shadow-card);
}

/* Interactive element */
.my-button {
  padding: var(--space-2) var(--space-3);
  background-color: var(--color-ink);
  color: var(--color-paper);
  border: none;
  border-radius: var(--radius-md);
  font-family: var(--font-sans);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: var(--transition-colors), transform var(--transition-fast);
}

.my-button:hover {
  background-color: var(--color-ink-light);
  transform: scale(1.01);
}

.my-button:focus-visible {
  outline: 2px solid var(--color-accent-gold);
  outline-offset: 2px;
}

/* Tier-colored element */
.chord-badge.tier-safe {
  color: var(--color-tier-safe);
  background-color: var(--color-tier-safe-bg);
  border: 1px solid var(--color-tier-safe-border);
}
```

### File Imports

```css
/* In component styles */
/* Tokens are globally available via app.css import */

/* Use tokens directly */
.component {
  color: var(--color-ink);
}
```

```typescript
// In Svelte components
import { theme } from '$lib/theme.svelte';
```
