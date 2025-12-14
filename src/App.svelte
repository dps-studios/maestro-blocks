<script lang="ts">
  import { onMount } from 'svelte';
  import ScoreCanvas from './lib/components/ScoreCanvas.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import { scoreStore } from './lib/stores/score';
  import { theme } from './lib/theme.svelte';

  let score = $state($scoreStore);

  $effect(() => {
    score = $scoreStore;
  });

  onMount(() => {
    console.log('Maestro Blocks - Paper & Ink Edition loaded');
    theme.init();
  });

  function toggleTheme() {
    theme.toggle();
  }

  // Detect macOS for traffic light spacer
  const isMacOS = typeof navigator !== 'undefined' && navigator.platform.toLowerCase().includes('mac');
</script>

<div class="app paper-texture">
  <!-- Header -->
  <header class="app-header">
    {#if isMacOS}
      <div class="traffic-light-spacer"></div>
    {/if}
    
    <div class="logo">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M9 18V5l12-2v13"/>
        <circle cx="6" cy="18" r="3"/>
        <circle cx="18" cy="16" r="3"/>
      </svg>
      <h1>Maestro Blocks</h1>
    </div>
    
    <div class="header-center">
      <input 
        type="text" 
        class="title-input"
        value={score.metadata.title}
        oninput={(e) => scoreStore.updateMetadata({ title: (e.target as HTMLInputElement).value })}
        placeholder="Untitled Worksheet"
      />
    </div>
    
    <div class="header-actions">
      <button class="header-btn" onclick={toggleTheme} title="Toggle theme">
        {#if theme.isDark}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="5"/>
            <line x1="12" y1="1" x2="12" y2="3"/>
            <line x1="12" y1="21" x2="12" y2="23"/>
            <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
            <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
            <line x1="1" y1="12" x2="3" y2="12"/>
            <line x1="21" y1="12" x2="23" y2="12"/>
            <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
            <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
          </svg>
        {:else}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
          </svg>
        {/if}
      </button>
      <button class="header-btn" title="Export PDF (coming soon)" disabled>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
          <line x1="12" y1="18" x2="12" y2="12"/>
          <line x1="9" y1="15" x2="15" y2="15"/>
        </svg>
        <span class="btn-label">Export</span>
      </button>
    </div>
  </header>

  <!-- Main Layout -->
  <div class="app-body">
    <Sidebar />
    <main class="main-content paper-texture">
      <ScoreCanvas />
    </main>
  </div>

  <!-- Status Bar -->
  <footer class="status-bar">
    <div class="status-left">
      <span class="status-item">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M9 18V5l12-2v13"/>
          <circle cx="6" cy="18" r="3"/>
          <circle cx="18" cy="16" r="3"/>
        </svg>
        {score.sections.length} section{score.sections.length !== 1 ? 's' : ''}
      </span>
      <span class="status-divider"></span>
      <span class="status-item">
        {score.keySignature.fifths === 0 ? 'C Major' : 
         score.keySignature.fifths > 0 ? `${score.keySignature.fifths} sharp${score.keySignature.fifths !== 1 ? 's' : ''}` :
         `${Math.abs(score.keySignature.fifths)} flat${Math.abs(score.keySignature.fifths) !== 1 ? 's' : ''}`}
      </span>
      <span class="status-divider"></span>
      <span class="status-item">
        {score.timeSignature.beats}/{score.timeSignature.beatType}
      </span>
    </div>
    <div class="status-right">
      <span class="status-item muted">
        Verovio rendering
      </span>
    </div>
  </footer>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    background-color: var(--color-paper-dark);
  }

  /* Header */
  .app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 44px;
    padding: 0 var(--space-4);
    background-color: var(--color-paper);
    border-bottom: 1px solid var(--color-accent-line);
    gap: var(--space-4);
    flex-shrink: 0;
  }

  .traffic-light-spacer {
    width: 70px;
    flex-shrink: 0;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--color-ink);
  }

  .logo svg {
    color: var(--color-accent-gold);
  }

  .logo h1 {
    margin: 0;
    font-family: var(--font-serif);
    font-size: var(--text-lg);
    font-weight: var(--font-normal);
    letter-spacing: var(--tracking-wide);
  }

  .header-center {
    flex: 1;
    display: flex;
    justify-content: center;
    max-width: 400px;
  }

  .title-input {
    width: 100%;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    padding: var(--space-1-5) var(--space-3);
    font-family: var(--font-serif);
    font-size: var(--text-base);
    text-align: center;
    color: var(--color-ink);
    transition: var(--transition-colors), border-color var(--transition-base);
  }

  .title-input::placeholder {
    color: var(--color-ink-muted);
    font-style: italic;
  }

  .title-input:hover {
    border-color: var(--color-accent-line);
  }

  .title-input:focus {
    outline: none;
    border-color: var(--color-accent-gold);
    background-color: var(--color-paper);
  }

  .header-actions {
    display: flex;
    gap: var(--space-2);
    align-items: center;
  }

  .header-btn {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1-5) var(--space-2);
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    color: var(--color-ink);
    font-size: var(--text-sm);
    transition: var(--transition-colors), border-color var(--transition-base);
  }

  .header-btn:hover:not(:disabled) {
    background-color: var(--color-hover-overlay);
    border-color: var(--color-accent-line);
  }

  .header-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-label {
    display: none;
  }

  @media (min-width: 640px) {
    .btn-label {
      display: inline;
    }
  }

  /* Main Body */
  .app-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    display: flex;
    padding: var(--space-4);
    overflow: hidden;
    background-color: var(--color-paper-dark);
  }

  /* Status Bar */
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 28px;
    padding: 0 var(--space-4);
    background-color: var(--color-paper);
    border-top: 1px solid var(--color-accent-line);
    font-family: var(--font-sans);
    font-size: var(--text-xs);
    color: var(--color-ink-light);
    flex-shrink: 0;
  }

  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  .status-item.muted {
    color: var(--color-ink-muted);
  }

  .status-divider {
    width: 1px;
    height: 12px;
    background-color: var(--color-accent-line);
  }
</style>
