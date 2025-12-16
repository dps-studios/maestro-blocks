/**
 * WorksheetModeSidebar
 * Sidebar for worksheet generation mode with randomizer controls
 */

import { Component, For, Show, createMemo } from 'solid-js';
import { 
  worksheetSettingsStore, 
  ALL_ROOTS, 
  TRIAD_QUALITIES, 
  SEVENTH_QUALITIES,
  PROBLEM_COUNT_OPTIONS,
  PROBLEM_COUNT_CONFIG,
  type RootOption,
  type RootPreset,
  type ProblemCountOption,
  type InversionWeight
} from '../stores/worksheetSettings';
import type { ChordQuality } from '../types/score';

// Quality display labels
const QUALITY_LABELS: Record<ChordQuality, string> = {
  'major': 'Major',
  'minor': 'Minor',
  'diminished': '°',
  'augmented': 'aug',
  'major7': 'Major 7',
  'minor7': 'Minor 7',
  'dominant7': 'Dom 7',
  'diminished7': '°7',
  'half-diminished7': 'ø7',
  'augmented7': 'aug7',
  'sus2': 'sus2',
  'sus4': 'sus4',
};

interface WorksheetModeSidebarProps {
  onGenerateWorksheet: () => void;
  onRandomizeSelected?: () => void;
  onClearAll?: () => void;
  onToggleAnswers?: () => void;
  hasSelection?: boolean;
}

const WorksheetModeSidebar: Component<WorksheetModeSidebarProps> = (props) => {
  const settings = () => worksheetSettingsStore.state;

  // Check if a root is selected
  const isRootSelected = (root: RootOption) => {
    return settings().allowedRoots.some(
      r => r.note === root.note && r.accidental === root.accidental
    );
  };

  // Check if a quality is selected
  const isQualitySelected = (quality: ChordQuality) => {
    return settings().allowedQualities.includes(quality);
  };

  // Check if all triads are selected
  const allTriadsSelected = createMemo(() => {
    return TRIAD_QUALITIES.every(q => settings().allowedQualities.includes(q));
  });

  // Check if all sevenths are selected
  const allSeventhsSelected = createMemo(() => {
    return SEVENTH_QUALITIES.every(q => settings().allowedQualities.includes(q));
  });

  // Get density info for display
  const densityInfo = createMemo(() => {
    const config = PROBLEM_COUNT_CONFIG[settings().problemCount];
    const durationLabel = config.duration === 1 ? 'whole' : config.duration === 2 ? 'half' : 'quarter';
    return `${durationLabel} notes, ${config.staves} staves`;
  });

  return (
    <div class="worksheet-sidebar">
      <div class="sidebar-section">
        <h3 class="sidebar-section-title">Worksheet Settings</h3>
        
        {/* Clef Selection */}
        <div class="sidebar-field">
          <label class="sidebar-label">Clef</label>
          <div class="button-group">
            <button
              class={`button-group-item ${settings().clef === 'treble' ? 'selected' : ''}`}
              onClick={() => worksheetSettingsStore.setClef('treble')}
            >
              Treble
            </button>
            <button
              class={`button-group-item ${settings().clef === 'bass' ? 'selected' : ''}`}
              onClick={() => worksheetSettingsStore.setClef('bass')}
            >
              Bass
            </button>
            <button
              class={`button-group-item ${settings().clef === 'both' ? 'selected' : ''}`}
              onClick={() => worksheetSettingsStore.setClef('both')}
            >
              Both
            </button>
          </div>
        </div>

        {/* Problem Count */}
        <div class="sidebar-field">
          <label class="sidebar-label">Problems</label>
          <div class="button-group">
            <For each={PROBLEM_COUNT_OPTIONS}>
              {(count) => (
                <button
                  class={`button-group-item ${settings().problemCount === count ? 'selected' : ''}`}
                  onClick={() => worksheetSettingsStore.setProblemCount(count)}
                >
                  {count}
                </button>
              )}
            </For>
          </div>
          <span class="sidebar-hint">{densityInfo()}</span>
        </div>
      </div>

      {/* Root Selection */}
      <div class="sidebar-section">
        <h3 class="sidebar-section-title">Allowed Roots</h3>
        
        {/* Presets */}
        <div class="sidebar-field">
          <select
            class="sidebar-select"
            value={settings().rootPreset}
            onChange={(e) => worksheetSettingsStore.setRootPreset(e.target.value as RootPreset)}
          >
            <option value="natural">Natural Notes</option>
            <option value="common-keys">Common Keys</option>
            <option value="all">All Chromatic</option>
            <option value="sharps">Sharps Only</option>
            <option value="flats">Flats Only</option>
            <option value="custom">Custom</option>
          </select>
        </div>

        {/* Root Grid */}
        <div class="root-grid">
          <For each={ALL_ROOTS}>
            {(root) => (
              <button
                class={`root-button ${isRootSelected(root) ? 'selected' : ''}`}
                onClick={() => worksheetSettingsStore.toggleRoot(root)}
                title={root.label}
              >
                {root.label}
              </button>
            )}
          </For>
        </div>
      </div>

      {/* Quality Selection */}
      <div class="sidebar-section">
        <h3 class="sidebar-section-title">Allowed Qualities</h3>
        
        {/* Quick toggles */}
        <div class="quality-toggles">
          <button
            class={`toggle-button ${allTriadsSelected() ? 'selected' : ''}`}
            onClick={() => worksheetSettingsStore.setTriads(!allTriadsSelected())}
          >
            All Triads
          </button>
          <button
            class={`toggle-button ${allSeventhsSelected() ? 'selected' : ''}`}
            onClick={() => worksheetSettingsStore.setSevenths(!allSeventhsSelected())}
          >
            All 7ths
          </button>
        </div>

        {/* Triads */}
        <div class="quality-group">
          <span class="quality-group-label">Triads</span>
          <div class="quality-grid">
            <For each={TRIAD_QUALITIES}>
              {(quality) => (
                <button
                  class={`quality-button ${isQualitySelected(quality) ? 'selected' : ''}`}
                  onClick={() => worksheetSettingsStore.toggleQuality(quality)}
                >
                  {QUALITY_LABELS[quality]}
                </button>
              )}
            </For>
          </div>
        </div>

        {/* Sevenths */}
        <div class="quality-group">
          <span class="quality-group-label">Sevenths</span>
          <div class="quality-grid">
            <For each={SEVENTH_QUALITIES}>
              {(quality) => (
                <button
                  class={`quality-button ${isQualitySelected(quality) ? 'selected' : ''}`}
                  onClick={() => worksheetSettingsStore.toggleQuality(quality)}
                >
                  {QUALITY_LABELS[quality]}
                </button>
              )}
            </For>
          </div>
        </div>
      </div>

      {/* Inversion Settings */}
      <div class="sidebar-section">
        <h3 class="sidebar-section-title">Inversions</h3>
        
        <div class="button-group vertical">
          <button
            class={`button-group-item ${settings().inversionWeight === 'root-only' ? 'selected' : ''}`}
            onClick={() => worksheetSettingsStore.setInversionWeight('root-only')}
          >
            Root position only
          </button>
          <button
            class={`button-group-item ${settings().inversionWeight === 'balanced' ? 'selected' : ''}`}
            onClick={() => worksheetSettingsStore.setInversionWeight('balanced')}
          >
            Balanced
          </button>
          <button
            class={`button-group-item ${settings().inversionWeight === 'favor-root' ? 'selected' : ''}`}
            onClick={() => worksheetSettingsStore.setInversionWeight('favor-root')}
          >
            Favor root (50%)
          </button>
        </div>
      </div>

      {/* Actions */}
      <div class="sidebar-section sidebar-actions">
        <button 
          class="action-button primary"
          onClick={props.onGenerateWorksheet}
        >
          Generate Worksheet
        </button>
        
        <Show when={props.hasSelection && props.onRandomizeSelected}>
          <button 
            class="action-button secondary"
            onClick={props.onRandomizeSelected}
          >
            Randomize Selected
          </button>
        </Show>
        
        <Show when={props.onClearAll}>
          <button 
            class="action-button secondary"
            onClick={props.onClearAll}
          >
            Clear All
          </button>
        </Show>
        
        <Show when={props.onToggleAnswers}>
          <button 
            class="action-button secondary"
            onClick={props.onToggleAnswers}
          >
            Toggle Answers
          </button>
        </Show>
      </div>
    </div>
  );
};

export default WorksheetModeSidebar;
