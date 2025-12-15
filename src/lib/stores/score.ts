/**
 * Score store for MuseScore-style worksheet editing
 * Manages the score state, editor state, and rendering
 */

import { writable, derived, get } from 'svelte/store';
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

function createScoreStore() {
  const { subscribe, set, update } = writable<Score>(createEmptyScore());

  return {
    subscribe,
    set,
    update,

    /** Reset to empty score */
    reset() {
      set(createEmptyScore());
    },

    /** Update score metadata */
    updateMetadata(metadata: Partial<Score['metadata']>) {
      update((s) => ({
        ...s,
        metadata: { ...s.metadata, ...metadata },
      }));
    },

    /** Add a new worksheet section */
    addSection(type: WorksheetSection['type'] = 'chord-naming', measureCount = 4) {
      update((s) => {
        const section = createChordNamingSection(measureCount);
        section.type = type;
        return {
          ...s,
          sections: [...s.sections, section],
        };
      });
    },

    /** Remove a section by ID */
    removeSection(sectionId: string) {
      update((s) => ({
        ...s,
        sections: s.sections.filter((sec) => sec.id !== sectionId),
      }));
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

      update((s) => {
        const sections = s.sections.map((section) => {
          if (section.id !== sectionId) return section;

          // Find the measure index for auto-expand check
          const measureIndex = section.staff.measures.findIndex(m => m.id === measureId);
          const isLastMeasure = measureIndex === section.staff.measures.length - 1;

          const measures = section.staff.measures.map((measure) => {
            if (measure.id !== measureId) return measure;

            const chordElement: ChordElement = {
              id: elementId,
              type: 'chord',
              pitches,
              duration,
              chordDef,
              displayName,
            };

            // REPLACE existing elements instead of appending (one chord per measure)
            return {
              ...measure,
              elements: [chordElement],
            };
          });

          // REPLACE answer box for this measure (remove old one, add new one)
          const answerBox: ChordNameBox = {
            id: crypto.randomUUID(),
            measureId: measureId,
            chordElementId: elementId,
            answer: '',
            correctAnswer: displayName,
            showAnswer: s.showAnswers,
          };

          // Filter out any existing answer box for this measure, then add new one
          const answerBoxes = section.answerBoxes
            .filter((box) => box.measureId !== measureId)
            .concat([answerBox]);

          // AUTO-EXPAND: If placing in last measure and below max, add more measures
          let finalMeasures = measures;
          if (isLastMeasure && section.autoExpand && measures.length < section.maxMeasures) {
            const currentCount = measures.length;
            const toAdd = Math.min(section.autoExpandCount, section.maxMeasures - currentCount);
            const newMeasures = Array.from({ length: toAdd }, (_, i) =>
              createEmptyMeasure(currentCount + i + 1)
            );
            finalMeasures = [...measures, ...newMeasures];
          }

          return {
            ...section,
            staff: { ...section.staff, measures: finalMeasures },
            answerBoxes,
          };
        });

        return { ...s, sections };
      });

      return elementId;
    },

    /** Remove an element from a measure */
    removeElement(sectionId: string, elementId: string) {
      update((s) => {
        const sections = s.sections.map((section) => {
          if (section.id !== sectionId) return section;

          const measures = section.staff.measures.map((measure) => ({
            ...measure,
            elements: measure.elements.filter((el) => el.id !== elementId),
          }));

          // Remove associated answer box
          const answerBoxes = section.answerBoxes.filter(
            (box) => box.chordElementId !== elementId
          );

          return {
            ...section,
            staff: { ...section.staff, measures },
            answerBoxes,
          };
        });

        return { ...s, sections };
      });
    },

    /** Update an element's properties */
    updateElement(sectionId: string, elementId: string, updates: Partial<MusicElement>) {
      update((s) => {
        const sections = s.sections.map((section): WorksheetSection => {
          if (section.id !== sectionId) return section;

          const measures: Measure[] = section.staff.measures.map((measure): Measure => ({
            ...measure,
            elements: measure.elements.map((el): MusicElement => {
              if (el.id !== elementId) return el;
              // Preserve the discriminated union type
              if (el.type === 'chord') {
                return { ...el, ...updates } as ChordElement;
              } else if (el.type === 'note') {
                return { ...el, ...updates } as MusicElement;
              } else {
                return { ...el, ...updates } as MusicElement;
              }
            }),
          }));

          return {
            ...section,
            staff: { ...section.staff, measures },
          };
        });

        return { ...s, sections };
      });
    },

    /** Toggle answer visibility */
    toggleAnswers() {
      update((s) => {
        const showAnswers = !s.showAnswers;

        // Update all answer boxes
        const sections = s.sections.map((section) => ({
          ...section,
          answerBoxes: section.answerBoxes.map((box) => ({
            ...box,
            showAnswer: showAnswers,
          })),
        }));

        return { ...s, showAnswers, sections };
      });
    },

    /** Set staff clef for a section */
    setClef(sectionId: string, clef: ClefType) {
      update((s) => ({
        ...s,
        sections: s.sections.map((section) =>
          section.id === sectionId
            ? { ...section, staff: { ...section.staff, clef } }
            : section
        ),
      }));
    },

    /** Add measures to a section */
    addMeasures(sectionId: string, count: number = 1) {
      update((s) => ({
        ...s,
        sections: s.sections.map((section) => {
          if (section.id !== sectionId) return section;

          const currentCount = section.staff.measures.length;
          const newMeasures: Measure[] = Array.from({ length: count }, (_, i) => ({
            id: crypto.randomUUID(),
            number: currentCount + i + 1,
            elements: [],
          }));

          return {
            ...section,
            staff: {
              ...section.staff,
              measures: [...section.staff.measures, ...newMeasures],
            },
          };
        }),
      }));
    },

    /** Get a specific section */
    getSection(sectionId: string): WorksheetSection | undefined {
      return get({ subscribe }).sections.find((s) => s.id === sectionId);
    },

    /** Get a specific measure */
    getMeasure(sectionId: string, measureId: string): Measure | undefined {
      const section = get({ subscribe }).sections.find((s) => s.id === sectionId);
      return section?.staff.measures.find((m) => m.id === measureId);
    },
  };
}

export const scoreStore = createScoreStore();

// ============================================================================
// EDITOR STATE STORE
// ============================================================================

function createEditorStore() {
  const { subscribe, set, update } = writable<EditorState>({
    cursor: null,
    selection: { type: 'none', elementIds: [] },
    activeTool: { type: 'chord', quality: 'major' },
    hoverPitch: null,
  });

  return {
    subscribe,
    set,
    update,

    /** Set the active tool */
    setTool(tool: EditorTool) {
      update((s) => ({ ...s, activeTool: tool }));
    },

    /** Set chord quality for chord tool */
    setChordQuality(quality: ChordQuality) {
      update((s) => ({
        ...s,
        activeTool: { type: 'chord', quality },
      }));
    },

    /** Select an element */
    selectElement(elementId: string) {
      update((s) => ({
        ...s,
        selection: { type: 'element', elementIds: [elementId] },
      }));
    },

    /** Clear selection */
    clearSelection() {
      update((s) => ({
        ...s,
        selection: { type: 'none', elementIds: [] },
      }));
    },

    /** Set hover pitch (for preview) */
    setHoverPitch(pitch: Pitch | null) {
      update((s) => ({ ...s, hoverPitch: pitch }));
    },

    /** Move cursor to position */
    setCursor(sectionId: string, measureNumber: number, beat: number = 0) {
      update((s) => ({
        ...s,
        cursor: { sectionId, measureNumber, beat },
      }));
    },
  };
}

export const editorStore = createEditorStore();

// ============================================================================
// DERIVED STORES
// ============================================================================

/** Currently selected section */
export const currentSection = derived(
  [scoreStore, editorStore],
  ([$score, $editor]) => {
    if (!$editor.cursor) {
      return $score.sections[0] ?? null;
    }
    return $score.sections.find((s) => s.id === $editor.cursor?.sectionId) ?? null;
  }
);

/** Currently selected elements */
export const selectedElements = derived(
  [scoreStore, editorStore],
  ([$score, $editor]) => {
    if ($editor.selection.type === 'none') return [];

    const elements: MusicElement[] = [];
    for (const section of $score.sections) {
      for (const measure of section.staff.measures) {
        for (const element of measure.elements) {
          if ($editor.selection.elementIds.includes(element.id)) {
            elements.push(element);
          }
        }
      }
    }
    return elements;
  }
);

// ============================================================================
// NOTE: Rendering is now handled directly in ScoreCanvas using VexFlow
// The old Verovio render cache has been removed
// ============================================================================
