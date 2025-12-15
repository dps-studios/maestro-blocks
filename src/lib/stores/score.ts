/**
 * Score store for MuseScore-style worksheet editing
 * Manages the score state, editor state, and rendering
 * 
 * SolidJS implementation using createStore for fine-grained reactivity
 */

import { createStore, produce } from 'solid-js/store';
import { createMemo } from 'solid-js';
import type {
  Score,
  WorksheetSection,
  Measure,
  MusicElement,
  ChordElement,
  ChordDefinition,
  ChordQuality,
  EditorState,
  EditorTool,
  Pitch,
  Duration,
  ClefType,
  ChordNameBox,
} from '../types/score';
import {
  createEmptyScore,
  createChordNamingSection,
  createEmptyMeasure,
} from '../types/score';
import { generateChordPitchesRust } from '../services/music';

// ============================================================================
// SCORE STORE
// ============================================================================

const [score, setScore] = createStore<Score>(createEmptyScore());

export const scoreStore = {
  /** Get the current score state (reactive) */
  get state() {
    return score;
  },

  /** Reset to empty score */
  reset() {
    setScore(createEmptyScore());
  },

  /** Update score metadata */
  updateMetadata(metadata: Partial<Score['metadata']>) {
    setScore('metadata', (prev) => ({ ...prev, ...metadata }));
  },

  /** Add a new worksheet section */
  addSection(type: WorksheetSection['type'] = 'chord-naming', measureCount = 4) {
    const section = createChordNamingSection(measureCount);
    section.type = type;
    setScore('sections', (sections) => [...sections, section]);
  },

  /** Remove a section by ID */
  removeSection(sectionId: string) {
    setScore('sections', (sections) => sections.filter((sec) => sec.id !== sectionId));
  },

  /** Add a chord to a specific measure (replaces existing chord if any) */
  async addChordToMeasure(
    sectionId: string,
    measureId: string,
    chordDef: ChordDefinition,
    rootOctave: number = 4,
    duration: Duration = { value: 1, dots: 0 }
  ): Promise<string> {
    const elementId = crypto.randomUUID();

    console.log('[ScoreStore] addChordToMeasure called:', {
      sectionId,
      measureId,
      chordDef,
      rootOctave,
    });

    // Generate chord pitches using Rust backend
    const { pitches, displayName } = await generateChordPitchesRust(
      chordDef,
      rootOctave as 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8
    );

    console.log('[ScoreStore] Rust backend returned:', {
      pitches,
      displayName,
    });

    setScore(
      produce((s) => {
        const sectionIndex = s.sections.findIndex((sec) => sec.id === sectionId);
        if (sectionIndex === -1) return;

        const section = s.sections[sectionIndex];
        const measureIndex = section.staff.measures.findIndex((m) => m.id === measureId);
        if (measureIndex === -1) return;

        const isLastMeasure = measureIndex === section.staff.measures.length - 1;

        // Create chord element
        const chordElement: ChordElement = {
          id: elementId,
          type: 'chord',
          pitches,
          duration,
          chordDef,
          displayName,
        };

        // REPLACE existing elements instead of appending (one chord per measure)
        section.staff.measures[measureIndex].elements = [chordElement];

        // REPLACE answer box for this measure
        const answerBox: ChordNameBox = {
          id: crypto.randomUUID(),
          measureId: measureId,
          chordElementId: elementId,
          answer: '',
          correctAnswer: displayName,
          showAnswer: s.showAnswers,
        };

        // Filter out any existing answer box for this measure, then add new one
        section.answerBoxes = section.answerBoxes
          .filter((box) => box.measureId !== measureId)
          .concat([answerBox]);

        // AUTO-EXPAND: If placing in last measure and below max, add more measures
        if (isLastMeasure && section.autoExpand && section.staff.measures.length < section.maxMeasures) {
          const currentCount = section.staff.measures.length;
          const toAdd = Math.min(section.autoExpandCount, section.maxMeasures - currentCount);
          const newMeasures = Array.from({ length: toAdd }, (_, i) =>
            createEmptyMeasure(currentCount + i + 1)
          );
          section.staff.measures.push(...newMeasures);
        }
      })
    );

    return elementId;
  },

  /** Remove an element from a measure */
  removeElement(sectionId: string, elementId: string) {
    setScore(
      produce((s) => {
        const section = s.sections.find((sec) => sec.id === sectionId);
        if (!section) return;

        for (const measure of section.staff.measures) {
          measure.elements = measure.elements.filter((el) => el.id !== elementId);
        }

        // Remove associated answer box
        section.answerBoxes = section.answerBoxes.filter(
          (box) => box.chordElementId !== elementId
        );
      })
    );
  },

  /** Update an element's properties */
  updateElement(sectionId: string, elementId: string, updates: Partial<MusicElement>) {
    setScore(
      produce((s) => {
        const section = s.sections.find((sec) => sec.id === sectionId);
        if (!section) return;

        for (const measure of section.staff.measures) {
          const element = measure.elements.find((el) => el.id === elementId);
          if (element) {
            Object.assign(element, updates);
            break;
          }
        }
      })
    );
  },

  /** Toggle answer visibility */
  toggleAnswers() {
    setScore(
      produce((s) => {
        s.showAnswers = !s.showAnswers;

        // Update all answer boxes
        for (const section of s.sections) {
          for (const box of section.answerBoxes) {
            box.showAnswer = s.showAnswers;
          }
        }
      })
    );
  },

  /** Set staff clef for a section */
  setClef(sectionId: string, clef: ClefType) {
    setScore(
      produce((s) => {
        const section = s.sections.find((sec) => sec.id === sectionId);
        if (section) {
          section.staff.clef = clef;
        }
      })
    );
  },

  /** Add measures to a section */
  addMeasures(sectionId: string, count: number = 1) {
    setScore(
      produce((s) => {
        const section = s.sections.find((sec) => sec.id === sectionId);
        if (!section) return;

        const currentCount = section.staff.measures.length;
        const newMeasures: Measure[] = Array.from({ length: count }, (_, i) => ({
          id: crypto.randomUUID(),
          number: currentCount + i + 1,
          elements: [],
        }));

        section.staff.measures.push(...newMeasures);
      })
    );
  },

  /** Get a specific section */
  getSection(sectionId: string): WorksheetSection | undefined {
    return score.sections.find((s) => s.id === sectionId);
  },

  /** Get a specific measure */
  getMeasure(sectionId: string, measureId: string): Measure | undefined {
    const section = score.sections.find((s) => s.id === sectionId);
    return section?.staff.measures.find((m) => m.id === measureId);
  },
};

// ============================================================================
// EDITOR STATE STORE
// ============================================================================

const [editor, setEditor] = createStore<EditorState>({
  cursor: null,
  selection: { type: 'none', elementIds: [] },
  activeTool: { type: 'chord', quality: 'major' },
  hoverPitch: null,
});

export const editorStore = {
  /** Get the current editor state (reactive) */
  get state() {
    return editor;
  },

  /** Set the active tool */
  setTool(tool: EditorTool) {
    setEditor('activeTool', tool);
  },

  /** Set chord quality for chord tool */
  setChordQuality(quality: ChordQuality) {
    setEditor('activeTool', { type: 'chord', quality });
  },

  /** Select an element */
  selectElement(elementId: string) {
    setEditor('selection', { type: 'element', elementIds: [elementId] });
  },

  /** Clear selection */
  clearSelection() {
    setEditor('selection', { type: 'none', elementIds: [] });
  },

  /** Set hover pitch (for preview) */
  setHoverPitch(pitch: Pitch | null) {
    setEditor('hoverPitch', pitch);
  },

  /** Move cursor to position */
  setCursor(sectionId: string, measureNumber: number, beat: number = 0) {
    setEditor('cursor', { sectionId, measureNumber, beat });
  },
};

// ============================================================================
// DERIVED/COMPUTED VALUES
// ============================================================================

/** Currently selected section */
export const currentSection = createMemo(() => {
  if (!editor.cursor) {
    return score.sections[0] ?? null;
  }
  return score.sections.find((s) => s.id === editor.cursor?.sectionId) ?? null;
});

/** Currently selected elements */
export const selectedElements = createMemo(() => {
  if (editor.selection.type === 'none') return [];

  const elements: MusicElement[] = [];
  for (const section of score.sections) {
    for (const measure of section.staff.measures) {
      for (const element of measure.elements) {
        if (editor.selection.elementIds.includes(element.id)) {
          elements.push(element);
        }
      }
    }
  }
  return elements;
});

// ============================================================================
// NOTE: Rendering is now handled directly in ScoreCanvas using VexFlow
// The old Verovio render cache has been removed
// ============================================================================
