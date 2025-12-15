<script lang="ts">
  import { scoreStore, editorStore } from '../stores/score';
  import type { ChordQuality, WorksheetSection } from '../types/score';

  // Reactive store subscriptions using $derived
  let showAnswers = $derived($scoreStore.showAnswers);
  let sections = $derived($scoreStore.sections);
  let activeTool = $derived($editorStore.activeTool);

  // Chord qualities grouped by type with tier assignments
  // Tier 0 (Safe/Green): Basic triads - foundational, most common
  // Tier 1 (Colorful/Amber): Seventh chords - adds color, jazz vocabulary
  // Tier 2 (Bold/Rose): Extended/Altered - advanced, distinctive sounds
  const triadQualities: { value: ChordQuality; label: string; tier: number }[] = [
    { value: 'major', label: 'Major', tier: 0 },
    { value: 'minor', label: 'Minor', tier: 0 },
    { value: 'diminished', label: 'Dim', tier: 2 },
    { value: 'augmented', label: 'Aug', tier: 2 },
  ];

  const seventhQualities: { value: ChordQuality; label: string; tier: number }[] = [
    { value: 'major7', label: 'Maj7', tier: 1 },
    { value: 'minor7', label: 'Min7', tier: 1 },
    { value: 'dominant7', label: 'Dom7', tier: 1 },
    { value: 'diminished7', label: 'Dim7', tier: 2 },
    { value: 'half-diminished7', label: 'Half-dim7', tier: 2 },
  ];

  const suspendedQualities: { value: ChordQuality; label: string; tier: number }[] = [
    { value: 'sus2', label: 'Sus2', tier: 1 },
    { value: 'sus4', label: 'Sus4', tier: 1 },
  ];

  function selectChordQuality(quality: ChordQuality) {
    editorStore.setChordQuality(quality);
  }

  function addStaffLine() {
    // If no section exists, create one with 4 measures
    // If a section exists, add 4 more measures to it (like adding a new staff line)
    if (sections.length === 0) {
      console.log('[Sidebar] Creating first section...');
      scoreStore.addSection('chord-naming', 4);
    } else {
      // Add measures to the first (and typically only) section
      const sectionId = sections[0].id;
      console.log('[Sidebar] Adding 4 measures to existing section...');
      scoreStore.addMeasures(sectionId, 4);
    }
  }

  function toggleAnswers() {
    scoreStore.toggleAnswers();
  }

  function isActiveQuality(quality: ChordQuality): boolean {
    return activeTool.type === 'chord' && activeTool.quality === quality;
  }

  function countChords(sectionList: WorksheetSection[]): number {
    return sectionList.reduce((sum, s) => 
      sum + s.staff.measures.reduce((mSum, m) => 
        mSum + m.elements.filter(e => e.type === 'chord').length, 0
      ), 0
    );
  }

  // Get tier class for chord quality button
  function getTierClass(tier: number, isActive: boolean): string {
    if (isActive) {
      switch (tier) {
        case 0: return 'tier-safe-active';
        case 1: return 'tier-colorful-active';
        case 2: return 'tier-bold-active';
        default: return 'tier-neutral-active';
      }
    }
    switch (tier) {
      case 0: return 'tier-safe';
      case 1: return 'tier-colorful';
      case 2: return 'tier-bold';
      default: return 'tier-neutral';
    }
  }
</script>

<aside class="sidebar">
  <!-- Worksheet Actions -->
  <section class="sidebar-section">
    <h3 class="section-header">Worksheet</h3>
    <div class="action-buttons">
      <button class="action-btn primary" onclick={addStaffLine}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
        Add Staff Line
      </button>
      <button class="action-btn" onclick={toggleAnswers}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          {#if showAnswers}
            <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
            <line x1="1" y1="1" x2="23" y2="23"/>
          {:else}
            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
            <circle cx="12" cy="12" r="3"/>
          {/if}
        </svg>
        {showAnswers ? 'Hide Answers' : 'Show Answers'}
      </button>
    </div>
  </section>

  <!-- Chord Quality Selection -->
  <section class="sidebar-section">
    <h3 class="section-header">Chord Quality</h3>
    <p class="section-hint">Select a quality, then click on the staff to place a chord.</p>

    <div class="quality-group">
      <h4 class="group-label">Triads</h4>
      <div class="quality-buttons">
        {#each triadQualities as q}
          <button
            class="quality-btn {getTierClass(q.tier, isActiveQuality(q.value))}"
            class:active={isActiveQuality(q.value)}
            onclick={() => selectChordQuality(q.value)}
          >
            {q.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="quality-group">
      <h4 class="group-label">Seventh Chords</h4>
      <div class="quality-buttons">
        {#each seventhQualities as q}
          <button
            class="quality-btn {getTierClass(q.tier, isActiveQuality(q.value))}"
            class:active={isActiveQuality(q.value)}
            onclick={() => selectChordQuality(q.value)}
          >
            {q.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="quality-group">
      <h4 class="group-label">Suspended</h4>
      <div class="quality-buttons">
        {#each suspendedQualities as q}
          <button
            class="quality-btn {getTierClass(q.tier, isActiveQuality(q.value))}"
            class:active={isActiveQuality(q.value)}
            onclick={() => selectChordQuality(q.value)}
          >
            {q.label}
          </button>
        {/each}
      </div>
    </div>
  </section>

  <!-- Current Tool Info -->
  <section class="sidebar-section">
    <h3 class="section-header">Current Tool</h3>
    <div class="tool-info">
      {#if activeTool.type === 'chord'}
        <div class="tool-badge chord">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 18V5l12-2v13"/>
            <circle cx="6" cy="18" r="3"/>
            <circle cx="18" cy="16" r="3"/>
          </svg>
          <span>{activeTool.quality} chord</span>
        </div>
        <p class="tool-hint">Click on a measure to place this chord</p>
      {:else if activeTool.type === 'select'}
        <div class="tool-badge select">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z"/>
          </svg>
          <span>Select</span>
        </div>
        <p class="tool-hint">Click elements to select them</p>
      {/if}
    </div>
  </section>

  <!-- Score Info -->
  {#if sections.length > 0}
    <section class="sidebar-section">
      <h3 class="section-header">Score Info</h3>
      <div class="score-info">
        <div class="info-row">
          <span class="info-label">Sections:</span>
          <span class="info-value">{sections.length}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Total Chords:</span>
          <span class="info-value">{countChords(sections)}</span>
        </div>
      </div>
    </section>
  {/if}
</aside>

<style>
  .sidebar {
    width: 260px;
    background-color: var(--color-paper);
    border-right: 1px solid var(--color-accent-line);
    padding: var(--space-4);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    flex-shrink: 0;
  }

  .sidebar-section {
    background-color: var(--color-paper);
    border: 1px solid var(--color-accent-line);
    border-radius: var(--radius-lg);
    padding: var(--space-3);
  }

  .section-header {
    margin: 0 0 var(--space-3);
    font-family: var(--font-sans);
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: var(--color-ink-muted);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
  }

  .section-hint {
    margin: 0 0 var(--space-3);
    font-size: var(--text-xs);
    color: var(--color-ink-muted);
    line-height: var(--leading-relaxed);
  }

  .action-buttons {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .action-btn {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-accent-line);
    border-radius: var(--radius-md);
    background-color: var(--color-paper);
    color: var(--color-ink);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    cursor: pointer;
    transition: var(--transition-colors), transform var(--transition-fast);
  }

  .action-btn:hover {
    background-color: var(--color-paper-dark);
    transform: scale(1.01);
  }

  .action-btn.primary {
    background-color: var(--color-ink);
    border-color: var(--color-ink);
    color: var(--color-paper);
  }

  .action-btn.primary:hover {
    background-color: var(--color-ink-light);
    border-color: var(--color-ink-light);
  }

  .quality-group {
    margin-bottom: var(--space-3);
  }

  .quality-group:last-child {
    margin-bottom: 0;
  }

  .group-label {
    margin: 0 0 var(--space-2);
    font-family: var(--font-sans);
    font-size: 10px;
    font-weight: var(--font-medium);
    color: var(--color-ink-muted);
    text-transform: uppercase;
    letter-spacing: var(--tracking-widest);
  }

  .quality-buttons {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-1-5);
  }

  .quality-btn {
    padding: var(--space-1-5) var(--space-2);
    border: 1px solid var(--color-accent-line);
    border-radius: var(--radius-sm);
    background-color: var(--color-paper);
    font-family: var(--font-serif);
    font-size: var(--text-sm);
    cursor: pointer;
    transition: var(--transition-colors), transform var(--transition-fast);
    text-align: center;
  }

  .quality-btn:hover {
    transform: scale(1.02);
  }

  /* Tier Color System */
  .quality-btn.tier-safe {
    color: var(--color-tier-safe);
  }
  .quality-btn.tier-safe:hover {
    background-color: var(--color-tier-safe-bg);
    border-color: var(--color-tier-safe-border);
  }
  .quality-btn.tier-safe-active {
    background-color: var(--color-tier-safe);
    border-color: var(--color-tier-safe);
    color: var(--color-paper);
  }

  .quality-btn.tier-colorful {
    color: var(--color-tier-colorful);
  }
  .quality-btn.tier-colorful:hover {
    background-color: var(--color-tier-colorful-bg);
    border-color: var(--color-tier-colorful-border);
  }
  .quality-btn.tier-colorful-active {
    background-color: var(--color-tier-colorful);
    border-color: var(--color-tier-colorful);
    color: var(--color-paper);
  }

  .quality-btn.tier-bold {
    color: var(--color-tier-bold);
  }
  .quality-btn.tier-bold:hover {
    background-color: var(--color-tier-bold-bg);
    border-color: var(--color-tier-bold-border);
  }
  .quality-btn.tier-bold-active {
    background-color: var(--color-tier-bold);
    border-color: var(--color-tier-bold);
    color: var(--color-paper);
  }

  .quality-btn.tier-neutral {
    color: var(--color-tier-neutral);
  }
  .quality-btn.tier-neutral:hover {
    background-color: var(--color-tier-neutral-bg);
    border-color: var(--color-tier-neutral-border);
  }
  .quality-btn.tier-neutral-active {
    background-color: var(--color-tier-neutral);
    border-color: var(--color-tier-neutral);
    color: var(--color-paper);
  }

  .tool-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .tool-badge {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    font-family: var(--font-serif);
    font-size: var(--text-sm);
    text-transform: capitalize;
  }

  .tool-badge.chord {
    background-color: var(--color-tier-colorful-bg);
    color: var(--color-tier-colorful);
    border: 1px solid var(--color-tier-colorful-border);
  }

  .tool-badge.select {
    background-color: var(--color-paper-dark);
    color: var(--color-ink-light);
    border: 1px solid var(--color-accent-line);
  }

  .tool-hint {
    margin: 0;
    font-size: var(--text-xs);
    color: var(--color-ink-muted);
  }

  .score-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    font-size: var(--text-sm);
  }

  .info-label {
    color: var(--color-ink-muted);
  }

  .info-value {
    font-weight: var(--font-medium);
    color: var(--color-ink);
    font-family: var(--font-serif);
  }
</style>
