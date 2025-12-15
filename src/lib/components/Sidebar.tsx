import { Show, For } from 'solid-js';
import { scoreStore, editorStore } from '../stores/score';
import type { ChordQuality, WorksheetSection } from '../types/score';

export default function Sidebar() {
  // Reactive store accessors
  const showAnswers = () => scoreStore.state.showAnswers;
  const sections = () => scoreStore.state.sections;
  const activeTool = () => editorStore.state.activeTool;

  // Chord qualities grouped by type with tier assignments
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
    if (sections().length === 0) {
      console.log('[Sidebar] Creating first section...');
      scoreStore.addSection('chord-naming', 4);
    } else {
      const sectionId = sections()[0].id;
      console.log('[Sidebar] Adding 4 measures to existing section...');
      scoreStore.addMeasures(sectionId, 4);
    }
  }

  function toggleAnswers() {
    scoreStore.toggleAnswers();
  }

  function isActiveQuality(quality: ChordQuality): boolean {
    return activeTool().type === 'chord' && (activeTool() as { type: 'chord'; quality: ChordQuality }).quality === quality;
  }

  function countChords(sectionList: WorksheetSection[]): number {
    return sectionList.reduce((sum, s) => 
      sum + s.staff.measures.reduce((mSum, m) => 
        mSum + m.elements.filter(e => e.type === 'chord').length, 0
      ), 0
    );
  }

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

  return (
    <aside class="sidebar">
      {/* Worksheet Actions */}
      <section class="sidebar-section">
        <h3 class="section-header">Worksheet</h3>
        <div class="action-buttons">
          <button class="action-btn primary" onClick={addStaffLine}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="12" y1="5" x2="12" y2="19"/>
              <line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
            Add Staff Line
          </button>
          <button class="action-btn" onClick={toggleAnswers}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <Show 
                when={showAnswers()}
                fallback={
                  <>
                    <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                    <circle cx="12" cy="12" r="3"/>
                  </>
                }
              >
                <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
                <line x1="1" y1="1" x2="23" y2="23"/>
              </Show>
            </svg>
            {showAnswers() ? 'Hide Answers' : 'Show Answers'}
          </button>
        </div>
      </section>

      {/* Chord Quality Selection */}
      <section class="sidebar-section">
        <h3 class="section-header">Chord Quality</h3>
        <p class="section-hint">Select a quality, then click on the staff to place a chord.</p>

        <div class="quality-group">
          <h4 class="group-label">Triads</h4>
          <div class="quality-buttons">
            <For each={triadQualities}>
              {(q) => (
                <button
                  class={`quality-btn ${getTierClass(q.tier, isActiveQuality(q.value))} ${isActiveQuality(q.value) ? 'active' : ''}`}
                  onClick={() => selectChordQuality(q.value)}
                >
                  {q.label}
                </button>
              )}
            </For>
          </div>
        </div>

        <div class="quality-group">
          <h4 class="group-label">Seventh Chords</h4>
          <div class="quality-buttons">
            <For each={seventhQualities}>
              {(q) => (
                <button
                  class={`quality-btn ${getTierClass(q.tier, isActiveQuality(q.value))} ${isActiveQuality(q.value) ? 'active' : ''}`}
                  onClick={() => selectChordQuality(q.value)}
                >
                  {q.label}
                </button>
              )}
            </For>
          </div>
        </div>

        <div class="quality-group">
          <h4 class="group-label">Suspended</h4>
          <div class="quality-buttons">
            <For each={suspendedQualities}>
              {(q) => (
                <button
                  class={`quality-btn ${getTierClass(q.tier, isActiveQuality(q.value))} ${isActiveQuality(q.value) ? 'active' : ''}`}
                  onClick={() => selectChordQuality(q.value)}
                >
                  {q.label}
                </button>
              )}
            </For>
          </div>
        </div>
      </section>

      {/* Current Tool Info */}
      <section class="sidebar-section">
        <h3 class="section-header">Current Tool</h3>
        <div class="tool-info">
          <Show 
            when={activeTool().type === 'chord'}
            fallback={
              <>
                <div class="tool-badge select">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z"/>
                  </svg>
                  <span>Select</span>
                </div>
                <p class="tool-hint">Click elements to select them</p>
              </>
            }
          >
            <div class="tool-badge chord">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M9 18V5l12-2v13"/>
                <circle cx="6" cy="18" r="3"/>
                <circle cx="18" cy="16" r="3"/>
              </svg>
              <span>{(activeTool() as { type: 'chord'; quality: string }).quality} chord</span>
            </div>
            <p class="tool-hint">Click on a measure to place this chord</p>
          </Show>
        </div>
      </section>

      {/* Score Info */}
      <Show when={sections().length > 0}>
        <section class="sidebar-section">
          <h3 class="section-header">Score Info</h3>
          <div class="score-info">
            <div class="info-row">
              <span class="info-label">Sections:</span>
              <span class="info-value">{sections().length}</span>
            </div>
            <div class="info-row">
              <span class="info-label">Total Chords:</span>
              <span class="info-value">{countChords(sections())}</span>
            </div>
          </div>
        </section>
      </Show>
    </aside>
  );
}
