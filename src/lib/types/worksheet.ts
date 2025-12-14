// Worksheet-focused data structures for document-based LilyPond generation

export type WorksheetType = 
  | 'chord-naming'
  | 'interval-recognition'
  | 'scale-building'
  | 'rhythm-exercise'
  | 'note-identification';

export type EditableElementType = 
  | 'chord'
  | 'note'
  | 'rest'
  | 'text'
  | 'time-signature'
  | 'key-signature';

export interface EditableElement {
  id: string;
  type: EditableElementType;
  position: {
    measure: number;
    beat: number;
    voice?: number;
  };
  content: string; // LilyPond notation or text
  isAnswer: boolean; // Whether this is a question or answer
  isInteractive: boolean; // Whether user can edit this element
}

export interface WorksheetSection {
  id: string;
  title: string;
  instructions?: string;
  elements: EditableElement[];
  layout: {
    measuresPerSystem: number;
    systemsPerPage: number;
    clef: 'treble' | 'bass' | 'both';
    timeSignature?: string;
    keySignature?: string;
  };
}

export interface WorksheetConfig {
  id: string;
  title: string;
  subtitle?: string;
  type: WorksheetType;
  sections: WorksheetSection[];
  globalSettings: {
    paperSize: 'letter' | 'a4';
    orientation: 'portrait' | 'landscape';
    showAnswers: boolean;
    fontSize: number;
  };
}

// Template definitions for different worksheet types
export interface WorksheetTemplate {
  id: string;
  name: string;
  description: string;
  type: WorksheetType;
  defaultSections: Omit<WorksheetSection, 'id' | 'elements'>[];
  createSections: (params: any) => WorksheetSection[];
}

// Chord naming worksheet specific types
export interface ChordNamingParams {
  chords: Array<{
    root: string; // e.g., "C", "F#", "Bb"
    quality: 'major' | 'minor' | 'diminished' | 'augmented' | 'dominant7' | 'major7' | 'minor7';
    position: {
      measure: number;
      beat: number;
    };
    showAnswer: boolean;
  }>;
  instructions?: string;
  layout: {
    chordsPerLine: number;
    showStaffLines: boolean;
  };
}

export interface InteractiveElement {
  id: string;
  element_type: string;
  bounds: ElementBounds;
  data: any;
}

export interface ElementBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface LilyPondDocument {
  version: string;
  content: string;
  metadata: {
    title: string;
    composer?: string;
    tagline?: string;
  };
}