/**
 * Theme State Management
 * 
 * Provides reactive dark/light mode toggling with:
 * - System preference detection
 * - LocalStorage persistence
 * - Automatic HTML class application
 * 
 * SolidJS implementation using a lazy-initialized signal pattern
 * to avoid "computations created outside createRoot" warning.
 */

import { createSignal, createRoot, type Accessor } from 'solid-js';

// Check if we're in a browser environment
const isBrowser = typeof window !== 'undefined';

// Lazy-initialized theme state
// We use createRoot to properly own the signal and avoid disposal warnings
let isDarkSignal: Accessor<boolean> | null = null;
let setIsDarkFn: ((v: boolean) => void) | null = null;
let initialized = false;

/**
 * Get or create the theme signal within a proper reactive root
 */
function getThemeSignal(): { isDark: Accessor<boolean>; setIsDark: (v: boolean) => void } {
  if (!isDarkSignal) {
    // Create the signal inside a root so it's properly owned
    createRoot(() => {
      const [dark, setDark] = createSignal(false);
      isDarkSignal = dark;
      setIsDarkFn = setDark;
    });
  }
  return { isDark: isDarkSignal!, setIsDark: setIsDarkFn! };
}

/**
 * Initialize theme from stored preference or system preference
 */
function initTheme(): void {
  if (!isBrowser || initialized) return;
  initialized = true;

  const { isDark, setIsDark } = getThemeSignal();

  // Check localStorage first
  const stored = localStorage.getItem('theme');
  if (stored) {
    setIsDark(stored === 'dark');
  } else {
    // Fall back to system preference
    setIsDark(window.matchMedia('(prefers-color-scheme: dark)').matches);
  }

  applyTheme();

  // Listen for system preference changes
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
    // Only auto-switch if no stored preference
    if (!localStorage.getItem('theme')) {
      setIsDark(e.matches);
      applyTheme();
    }
  });
}

/**
 * Apply the current theme to the document
 */
function applyTheme(): void {
  if (!isBrowser) return;

  const { isDark } = getThemeSignal();
  if (isDark()) {
    document.documentElement.classList.add('dark');
  } else {
    document.documentElement.classList.remove('dark');
  }
}

/**
 * Toggle between dark and light mode
 */
function toggle(): void {
  const { isDark, setIsDark } = getThemeSignal();
  setIsDark(!isDark());
  if (isBrowser) {
    localStorage.setItem('theme', isDark() ? 'dark' : 'light');
  }
  applyTheme();
}

/**
 * Set theme explicitly
 */
function setTheme(dark: boolean): void {
  const { setIsDark } = getThemeSignal();
  setIsDark(dark);
  if (isBrowser) {
    localStorage.setItem('theme', dark ? 'dark' : 'light');
  }
  applyTheme();
}

/**
 * Clear stored preference and revert to system preference
 */
function useSystemPreference(): void {
  if (!isBrowser) return;
  
  const { setIsDark } = getThemeSignal();
  localStorage.removeItem('theme');
  setIsDark(window.matchMedia('(prefers-color-scheme: dark)').matches);
  applyTheme();
}

// Export reactive theme object
export const theme = {
  get isDark() {
    return getThemeSignal().isDark();
  },
  toggle,
  setTheme,
  useSystemPreference,
  init: initTheme,
};

/**
 * Reactive accessor for use in components
 * Returns a function that returns the current dark mode state
 */
export function isDark(): boolean {
  return getThemeSignal().isDark();
}
