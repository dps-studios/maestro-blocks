import { Show } from 'solid-js';
import { scoreStore, editorStore } from '../stores/score';
import type { WorksheetSection } from '../types/score';

export default function Sidebar() {
  // Reactive store accessors
  const showAnswers = () => scoreStore.state.showAnswers;
  const sections = () => scoreStore.state.sections;
  const selectedChordId = () => editorStore.state.selectedChordId;

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

  function countChords(sectionList: WorksheetSection[]): number {
    return sectionList.reduce((sum, s) => 
      sum + s.staff.measures.reduce((mSum, m) => 
        mSum + m.elements.filter(e => e.type === 'chord').length, 0
      ), 0
    );
  }

  // Get display name of currently selected chord
  function getSelectedChordName(): string {
    const chordId = selectedChordId();
    if (!chordId) return '';
    const chord = scoreStore.getChordElement(chordId);
    return chord?.displayName ?? '';
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

      {/* Instructions */}
      <section class="sidebar-section">
        <h3 class="section-header">How to Use</h3>
        <div class="instructions">
          <p class="instruction-item">
            <strong>Click</strong> on a measure to place a chord and open the editor
          </p>
          <p class="instruction-item">
            <strong>Shift+Click</strong> for quick placement using last settings
          </p>
          <p class="instruction-item">
            <strong>Escape</strong> to close the chord editor
          </p>
        </div>
      </section>

      {/* Selected Chord Info */}
      <Show when={selectedChordId()}>
        <section class="sidebar-section">
          <h3 class="section-header">Selected Chord</h3>
          <div class="selected-chord-info">
            <span class="chord-name">{getSelectedChordName()}</span>
          </div>
        </section>
      </Show>

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
