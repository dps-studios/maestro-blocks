/**
 * Theme State Management
 * 
 * Provides reactive dark/light mode toggling with:
 * - System preference detection
 * - LocalStorage persistence
 * - Automatic HTML class application
 */

// Check if we're in a browser environment
const isBrowser = typeof window !== 'undefined';

// Theme state using Svelte 5 runes
let isDark = $state(false);

/**
 * Initialize theme from stored preference or system preference
 */
function initTheme(): void {
  if (!isBrowser) return;

  // Check localStorage first
  const stored = localStorage.getItem('theme');
  if (stored) {
    isDark = stored === 'dark';
  } else {
    // Fall back to system preference
    isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  }

  applyTheme();

  // Listen for system preference changes
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
    // Only auto-switch if no stored preference
    if (!localStorage.getItem('theme')) {
      isDark = e.matches;
      applyTheme();
    }
  });
}

/**
 * Apply the current theme to the document
 */
function applyTheme(): void {
  if (!isBrowser) return;

  if (isDark) {
    document.documentElement.classList.add('dark');
  } else {
    document.documentElement.classList.remove('dark');
  }
}

/**
 * Toggle between dark and light mode
 */
function toggle(): void {
  isDark = !isDark;
  if (isBrowser) {
    localStorage.setItem('theme', isDark ? 'dark' : 'light');
  }
  applyTheme();
}

/**
 * Set theme explicitly
 */
function setTheme(dark: boolean): void {
  isDark = dark;
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
  
  localStorage.removeItem('theme');
  isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  applyTheme();
}

// Initialize on module load (browser only)
if (isBrowser) {
  initTheme();
}

// Export reactive theme object
export const theme = {
  get isDark() {
    return isDark;
  },
  toggle,
  setTheme,
  useSystemPreference,
  init: initTheme,
};
