import { Component, Show, createEffect, createSignal, onCleanup } from 'solid-js';
import { editorStore, scoreStore } from '../stores/score';
import type {
  NoteName,
  Accidental,
  ChordQuality,
  ChordInversion,
  ChordDefinition,
} from '../types/score';

// ============================================================================
// SELECT OPTIONS
// ============================================================================

interface NoteOption {
  label: string;
  note: NoteName;
  accidental: Accidental;
}

const NOTE_OPTIONS: NoteOption[] = [
  { label: 'C', note: 'c', accidental: null },
  { label: 'C#', note: 'c', accidental: 'sharp' },
  { label: 'Db', note: 'd', accidental: 'flat' },
  { label: 'D', note: 'd', accidental: null },
  { label: 'D#', note: 'd', accidental: 'sharp' },
  { label: 'Eb', note: 'e', accidental: 'flat' },
  { label: 'E', note: 'e', accidental: null },
  { label: 'F', note: 'f', accidental: null },
  { label: 'F#', note: 'f', accidental: 'sharp' },
  { label: 'Gb', note: 'g', accidental: 'flat' },
  { label: 'G', note: 'g', accidental: null },
  { label: 'G#', note: 'g', accidental: 'sharp' },
  { label: 'Ab', note: 'a', accidental: 'flat' },
  { label: 'A', note: 'a', accidental: null },
  { label: 'A#', note: 'a', accidental: 'sharp' },
  { label: 'Bb', note: 'b', accidental: 'flat' },
  { label: 'B', note: 'b', accidental: null },
];

interface QualityOption {
  label: string;
  value: ChordQuality;
  isSeventh: boolean;
}

const QUALITY_OPTIONS: QualityOption[] = [
  { label: 'Maj', value: 'major', isSeventh: false },
  { label: 'min', value: 'minor', isSeventh: false },
  { label: 'dim', value: 'diminished', isSeventh: false },
  { label: 'aug', value: 'augmented', isSeventh: false },
  { label: 'Maj7', value: 'major7', isSeventh: true },
  { label: 'min7', value: 'minor7', isSeventh: true },
  { label: 'dom7', value: 'dominant7', isSeventh: true },
  { label: 'dim7', value: 'diminished7', isSeventh: true },
  { label: 'ø7', value: 'half-diminished7', isSeventh: true },
  { label: 'sus2', value: 'sus2', isSeventh: false },
  { label: 'sus4', value: 'sus4', isSeventh: false },
];

interface InversionOption {
  label: string;
  value: ChordInversion;
  forSeventh: boolean; // true = only for 7th chords, false = for all
}

const INVERSION_OPTIONS: InversionOption[] = [
  { label: 'Root', value: 'root', forSeventh: false },
  { label: '6', value: 'first', forSeventh: false },
  { label: '6/4', value: 'second', forSeventh: false },
  { label: '4/2', value: 'third', forSeventh: true },
];

// Map internal inversion to display format for 7th chords
const SEVENTH_INVERSION_LABELS: Record<ChordInversion, string> = {
  root: 'Root',
  first: '6/5',
  second: '4/3',
  third: '4/2',
};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function isSeventhChord(quality: ChordQuality): boolean {
  return QUALITY_OPTIONS.find((q) => q.value === quality)?.isSeventh ?? false;
}

function getNoteOptionKey(note: NoteName, accidental: Accidental): string {
  const opt = NOTE_OPTIONS.find(
    (n) => n.note === note && n.accidental === accidental
  );
  return opt?.label ?? 'C';
}

function parseNoteOption(label: string): { note: NoteName; accidental: Accidental } {
  const opt = NOTE_OPTIONS.find((n) => n.label === label);
  return opt ?? { note: 'c', accidental: null };
}

// ============================================================================
// TRASH ICON SVG
// ============================================================================

const TrashIcon: Component = () => (
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
    <polyline points="3 6 5 6 21 6" />
    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
    <line x1="10" y1="11" x2="10" y2="17" />
    <line x1="14" y1="11" x2="14" y2="17" />
  </svg>
);

// ============================================================================
// COMPONENT
// ============================================================================

export const ChordEditorPopup: Component = () => {
  let popupRef: HTMLDivElement | undefined;

  // Local state for the selects (synced with selected chord)
  const [noteLabel, setNoteLabel] = createSignal('C');
  const [quality, setQuality] = createSignal<ChordQuality>('major');
  const [inversion, setInversion] = createSignal<ChordInversion>('root');

  // Sync local state when a chord is selected
  createEffect(() => {
    const chordId = editorStore.state.selectedChordId;
    if (!chordId) return;

    const chord = scoreStore.getChordElement(chordId);
    if (chord?.chordDef) {
      const def = chord.chordDef;
      setNoteLabel(getNoteOptionKey(def.root, def.rootAccidental));
      setQuality(def.quality);
      setInversion(def.inversion);
    }
  });

  // Live update chord when any select changes
  const updateChord = async () => {
    const chordId = editorStore.state.selectedChordId;
    const anchor = editorStore.state.popupAnchor;
    if (!chordId || !anchor) return;

    const { note, accidental } = parseNoteOption(noteLabel());
    const chordDef: ChordDefinition = {
      root: note,
      rootAccidental: accidental,
      quality: quality(),
      inversion: inversion(),
    };

    // Determine octave based on clef (treble = 4, bass = 3)
    const section = scoreStore.state.sections.find((s) => s.id === anchor.sectionId);
    const rootOctave = section?.staff.clef === 'bass' ? 3 : 4;

    await scoreStore.updateChord(anchor.sectionId, chordId, chordDef, rootOctave);

    // Persist quality and inversion for Shift+click
    // Map ChordInversion to the short format used by editorStore
    const inversionMap: Record<ChordInversion, 'root' | '1' | '2' | '3'> = {
      root: 'root',
      first: '1',
      second: '2',
      third: '3',
    };
    editorStore.setLastUsed(quality(), inversionMap[inversion()]);
  };

  // Handle note change
  const handleNoteChange = (e: Event) => {
    const value = (e.target as HTMLSelectElement).value;
    setNoteLabel(value);
    updateChord();
  };

  // Handle quality change
  const handleQualityChange = (e: Event) => {
    const value = (e.target as HTMLSelectElement).value as ChordQuality;
    setQuality(value);

    // If switching from 7th to triad and inversion is 'third', reset to root
    if (!isSeventhChord(value) && inversion() === 'third') {
      setInversion('root');
    }

    updateChord();
  };

  // Handle inversion change
  const handleInversionChange = (e: Event) => {
    const value = (e.target as HTMLSelectElement).value as ChordInversion;
    setInversion(value);
    updateChord();
  };

  // Handle delete
  const handleDelete = () => {
    const chordId = editorStore.state.selectedChordId;
    const anchor = editorStore.state.popupAnchor;
    if (!chordId || !anchor) return;

    scoreStore.removeElement(anchor.sectionId, chordId);
    editorStore.deselectChord();
  };

  // Handle click outside to close
  const handleClickOutside = (e: MouseEvent) => {
    if (popupRef && !popupRef.contains(e.target as Node)) {
      // Check if click is on the score canvas (allow re-selection)
      const target = e.target as HTMLElement;
      if (!target.closest('.score-canvas')) {
        editorStore.deselectChord();
      }
    }
  };

  // Handle Escape to close
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      editorStore.deselectChord();
    }
  };

  // Setup event listeners
  createEffect(() => {
    if (editorStore.state.selectedChordId) {
      document.addEventListener('mousedown', handleClickOutside);
      document.addEventListener('keydown', handleKeyDown);
    }

    onCleanup(() => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    });
  });

  // Get available inversions based on quality
  const availableInversions = () => {
    const isSeventh = isSeventhChord(quality());
    return INVERSION_OPTIONS.filter((inv) => !inv.forSeventh || isSeventh);
  };

  // Get inversion label based on quality type
  const getInversionLabel = (inv: InversionOption) => {
    if (isSeventhChord(quality())) {
      return SEVENTH_INVERSION_LABELS[inv.value];
    }
    return inv.label;
  };

  return (
    <Show when={editorStore.state.selectedChordId && editorStore.state.popupAnchor}>
      <div
        ref={popupRef}
        class="chord-editor-popup"
        style={{
          left: `${editorStore.state.popupAnchor!.x}px`,
          top: `${editorStore.state.popupAnchor!.y}px`,
        }}
      >
        <div class="chord-editor-popup__arrow" />
        <div class="chord-editor-popup__content">
          {/* Note Select */}
          <select
            class="chord-editor-popup__select"
            value={noteLabel()}
            onChange={handleNoteChange}
          >
            {NOTE_OPTIONS.map((opt) => (
              <option value={opt.label}>{opt.label}</option>
            ))}
          </select>

          {/* Quality Select */}
          <select
            class="chord-editor-popup__select chord-editor-popup__select--quality"
            value={quality()}
            onChange={handleQualityChange}
          >
            {QUALITY_OPTIONS.map((opt) => (
              <option value={opt.value}>{opt.label}</option>
            ))}
          </select>

          {/* Inversion Select */}
          <select
            class="chord-editor-popup__select"
            value={inversion()}
            onChange={handleInversionChange}
          >
            {availableInversions().map((inv) => (
              <option value={inv.value}>{getInversionLabel(inv)}</option>
            ))}
          </select>

          {/* Delete Button */}
          <button
            class="chord-editor-popup__delete"
            onClick={handleDelete}
            title="Delete chord"
          >
            <TrashIcon />
          </button>
        </div>
      </div>
    </Show>
  );
};

export default ChordEditorPopup;
