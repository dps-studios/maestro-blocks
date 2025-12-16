/**
 * Score data model for MuseScore-style sheet music editing
 * Designed for music theory worksheet generation
 */

// ============================================================================
// PITCH & DURATION
// ============================================================================

export type NoteName = 'c' | 'd' | 'e' | 'f' | 'g' | 'a' | 'b';
export type Accidental = 'sharp' | 'flat' | 'natural' | 'double-sharp' | 'double-flat' | null;
export type Octave = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8;

export interface Pitch {
  note: NoteName;
  accidental: Accidental;
  octave: Octave;
}

/** Duration as fraction of whole note: 1 = whole, 2 = half, 4 = quarter, etc. */
export type DurationValue = 1 | 2 | 4 | 8 | 16 | 32;

export interface Duration {
  value: DurationValue;
  dots: 0 | 1 | 2;
}

// ============================================================================
// CHORD QUALITIES (for worksheet generation)
// ============================================================================

export type ChordQuality =
  | 'major'
  | 'minor'
  | 'diminished'
  | 'augmented'
  | 'major7'
  | 'minor7'
  | 'dominant7'
  | 'diminished7'
  | 'half-diminished7'
  | 'augmented7'
  | 'sus2'
  | 'sus4';

export type ChordInversion = 'root' | 'first' | 'second' | 'third';

export interface ChordDefinition {
  root: NoteName;
  rootAccidental: Accidental;
  quality: ChordQuality;
  inversion: ChordInversion;
}

// ============================================================================
// MUSIC ELEMENTS
// ============================================================================

export interface BaseElement {
  id: string;
  type: string;
}

export interface NoteElement extends BaseElement {
  type: 'note';
  pitch: Pitch;
  duration: Duration;
}

export interface RestElement extends BaseElement {
  type: 'rest';
  duration: Duration;
}

export interface ChordElement extends BaseElement {
  type: 'chord';
  pitches: Pitch[];
  duration: Duration;
  /** For worksheet mode: the chord definition used to generate this */
  chordDef?: ChordDefinition;
  /** Display name (e.g., "Cmaj7") - hidden in student mode */
  displayName?: string;
  /** Clef override for "both" mode - determines which clef this chord uses */
  clefOverride?: 'treble' | 'bass';
}

export type MusicElement = NoteElement | RestElement | ChordElement;

// ============================================================================
// MEASURES & STAVES
// ============================================================================

export type ClefType = 'treble' | 'bass' | 'alto' | 'tenor' | 'both';

export interface TimeSignature {
  beats: number;
  beatType: DurationValue;
}

export interface KeySignature {
  /** Number of sharps (positive) or flats (negative) */
  fifths: number;
  /** Major or minor mode */
  mode: 'major' | 'minor';
}

export interface Measure {
  id: string;
  number: number;
  elements: MusicElement[];
  /** Override time signature for this measure (null = inherit from previous) */
  timeSignature?: TimeSignature;
  /** Override key signature for this measure */
  keySignature?: KeySignature;
}

export interface Staff {
  id: string;
  clef: ClefType;
  measures: Measure[];
}

// ============================================================================
// WORKSHEET-SPECIFIC ELEMENTS
// ============================================================================

export interface ChordNameBox {
  id: string;
  /** The measure this answer box belongs to */
  measureId: string;
  /** The chord element this box labels */
  chordElementId: string;
  /** Student's answer (empty string = blank box) */
  answer: string;
  /** Correct answer (shown in answer key mode) */
  correctAnswer: string;
  /** Whether to show the answer */
  showAnswer: boolean;
}

export interface WorksheetSection {
  id: string;
  type: 'chord-naming' | 'interval-recognition' | 'scale-building';
  title?: string;
  instructions?: string;
  staff: Staff;
  /** Answer boxes positioned below staff */
  answerBoxes: ChordNameBox[];
  /** Auto-expand measures when last one is filled */
  autoExpand: boolean;
  /** How many measures to add when auto-expanding */
  autoExpandCount: number;
  /** Maximum number of measures allowed */
  maxMeasures: number;
}

// ============================================================================
// SCORE (TOP-LEVEL DOCUMENT)
// ============================================================================

export type HeaderAlignment = 'start' | 'center' | 'end';

export interface ScoreMetadata {
  /** Markdown content for the header */
  headerContent: string;
  /** Text alignment for the rendered header */
  alignment: HeaderAlignment;
}

export interface Score {
  id: string;
  metadata: ScoreMetadata;
  /** Global settings */
  timeSignature: TimeSignature;
  keySignature: KeySignature;
  /** Worksheet sections */
  sections: WorksheetSection[];
  /** UI state: show answers or not */
  showAnswers: boolean;
}

// ============================================================================
// EDITOR STATE
// ============================================================================

export interface EditorCursor {
  sectionId: string;
  measureNumber: number;
  /** Position within measure (0 = start) */
  beat: number;
}

export interface EditorSelection {
  type: 'none' | 'element' | 'range';
  elementIds: string[];
}

export type EditorTool =
  | { type: 'select' }
  | { type: 'note'; duration: Duration }
  | { type: 'rest'; duration: Duration }
  | { type: 'chord'; quality: ChordQuality };

export interface EditorState {
  cursor: EditorCursor | null;
  selection: EditorSelection;
  activeTool: EditorTool;
  /** Currently hovered pitch on staff (for preview) */
  hoverPitch: Pitch | null;
}

// ============================================================================
// UTILITY TYPES
// ============================================================================

/** Chord symbol display format */
export function formatChordName(def: ChordDefinition): string {
  const rootName = def.root.toUpperCase();
  const accidental = def.rootAccidental === 'sharp' ? '#' : def.rootAccidental === 'flat' ? 'b' : '';
  
  const qualityMap: Record<ChordQuality, string> = {
    'major': ' Major',
    'minor': ' Minor',
    'diminished': '°',
    'augmented': 'aug',
    'major7': ' Major 7',
    'minor7': ' Minor 7',
    'dominant7': '7',
    'diminished7': '°7',
    'half-diminished7': 'ø7',
    'augmented7': 'aug7',
    'sus2': 'sus2',
    'sus4': 'sus4',
  };
  
  return `${rootName}${accidental}${qualityMap[def.quality]}`;
}

/** Generate pitches for a chord based on root and quality 
 * @param def - Chord definition with root note and quality
 * @param rootOctave - Octave for the root (bottom) note of the chord
 */
export function generateChordPitches(def: ChordDefinition, rootOctave: Octave = 4): Pitch[] {
  // Root note at the specified octave (this is the BOTTOM note of the chord)
  const root: Pitch = {
    note: def.root,
    accidental: def.rootAccidental,
    octave: rootOctave,
  };
  
  // Interval patterns (in semitones from root)
  const patterns: Record<ChordQuality, number[]> = {
    'major': [0, 4, 7],
    'minor': [0, 3, 7],
    'diminished': [0, 3, 6],
    'augmented': [0, 4, 8],
    'major7': [0, 4, 7, 11],
    'minor7': [0, 3, 7, 10],
    'dominant7': [0, 4, 7, 10],
    'diminished7': [0, 3, 6, 9],
    'half-diminished7': [0, 3, 6, 10],
    'augmented7': [0, 4, 8, 10],
    'sus2': [0, 2, 7],
    'sus4': [0, 5, 7],
  };
  
  // Calculate pitches based on intervals
  const semitones = patterns[def.quality];
  const pitches: Pitch[] = semitones.map((semitone) => {
    return addSemitones(root, semitone);
  });
  
  return pitches;
}

/** Add semitones to a pitch, returning a new pitch */
function addSemitones(base: Pitch, semitones: number): Pitch {
  const noteOrder: NoteName[] = ['c', 'd', 'e', 'f', 'g', 'a', 'b'];
  const semitonesFromC: Record<NoteName, number> = {
    'c': 0, 'd': 2, 'e': 4, 'f': 5, 'g': 7, 'a': 9, 'b': 11
  };
  
  // Get base pitch in absolute semitones
  let baseSemitone = semitonesFromC[base.note] + (base.octave * 12);
  if (base.accidental === 'sharp') baseSemitone += 1;
  if (base.accidental === 'flat') baseSemitone -= 1;
  if (base.accidental === 'double-sharp') baseSemitone += 2;
  if (base.accidental === 'double-flat') baseSemitone -= 2;
  
  // Add the interval
  const targetSemitone = baseSemitone + semitones;
  const targetOctave = Math.floor(targetSemitone / 12) as Octave;
  const targetInOctave = ((targetSemitone % 12) + 12) % 12;
  
  // Find the closest natural note and determine accidental
  let closestNote: NoteName = 'c';
  let minDiff = 12;
  
  for (const note of noteOrder) {
    const noteSemitone = semitonesFromC[note];
    const diff = Math.abs(noteSemitone - targetInOctave);
    if (diff < minDiff || (diff === minDiff && noteSemitone <= targetInOctave)) {
      minDiff = diff;
      closestNote = note;
    }
  }
  
  const naturalSemitone = semitonesFromC[closestNote];
  const accidentalDiff = targetInOctave - naturalSemitone;
  
  let accidental: Accidental = null;
  if (accidentalDiff === 1) accidental = 'sharp';
  else if (accidentalDiff === -1 || accidentalDiff === 11) accidental = 'flat';
  else if (accidentalDiff === 2) accidental = 'double-sharp';
  else if (accidentalDiff === -2 || accidentalDiff === 10) accidental = 'double-flat';
  
  return {
    note: closestNote,
    accidental,
    octave: Math.min(8, Math.max(0, targetOctave)) as Octave,
  };
}

/** Create a new empty score */
export function createEmptyScore(): Score {
  return {
    id: crypto.randomUUID(),
    metadata: {
      headerContent: '',
      alignment: 'start',
    },
    timeSignature: { beats: 4, beatType: 4 },
    keySignature: { fifths: 0, mode: 'major' },
    sections: [createChordNamingSection()],
    showAnswers: false,
  };
}

/** Create a new empty measure */
export function createEmptyMeasure(number: number): Measure {
  return {
    id: crypto.randomUUID(),
    number,
    elements: [],
  };
}

/** Create a new worksheet section with empty measures */
export function createChordNamingSection(measureCount: number = 4): WorksheetSection {
  const measures: Measure[] = Array.from({ length: measureCount }, (_, i) => 
    createEmptyMeasure(i + 1)
  );
  
  return {
    id: crypto.randomUUID(),
    type: 'chord-naming',
    title: 'Name the Chords',
    instructions: 'Write the chord symbol for each chord shown below.',
    staff: {
      id: crypto.randomUUID(),
      clef: 'treble',
      measures,
    },
    answerBoxes: [],
    autoExpand: true,
    autoExpandCount: 2,
    maxMeasures: 16,
  };
}
