/**
 * Worksheet Settings Store
 * Manages randomizer configuration for worksheet generation
 */

import { createStore } from 'solid-js/store';
import { createRoot } from 'solid-js';
import type { ChordQuality, ChordDefinition, ChordInversion, NoteName, Accidental, ClefType } from '../types/score';

// ============================================================================
// TYPES
// ============================================================================

export type RootPreset = 'all' | 'natural' | 'common-keys' | 'sharps' | 'flats' | 'custom';

export type InversionWeight = 'root-only' | 'balanced' | 'favor-root' | 'custom';

export interface RootOption {
  note: NoteName;
  accidental: Accidental;
  label: string;
}

/** Fixed problem count options with predefined density mappings */
export type ProblemCountOption = 4 | 8 | 12 | 16 | 24 | 32;

/** Density configuration for each problem count option */
export interface DensityConfig {
  duration: 1 | 2 | 4;  // 1=whole, 2=half, 4=quarter
  chordsPerMeasure: number;
  measures: number;
  staves: number;
}

/** Predefined density mappings for each problem count */
export const PROBLEM_COUNT_CONFIG: Record<ProblemCountOption, DensityConfig> = {
  4:  { duration: 1, chordsPerMeasure: 1, measures: 4,  staves: 1 },
  8:  { duration: 1, chordsPerMeasure: 1, measures: 8,  staves: 2 },
  12: { duration: 1, chordsPerMeasure: 1, measures: 12, staves: 3 },
  16: { duration: 1, chordsPerMeasure: 1, measures: 16, staves: 4 },
  24: { duration: 2, chordsPerMeasure: 2, measures: 12, staves: 3 },
  32: { duration: 2, chordsPerMeasure: 2, measures: 16, staves: 4 },
};

export const PROBLEM_COUNT_OPTIONS: ProblemCountOption[] = [4, 8, 12, 16, 24, 32];

export interface WorksheetSettings {
  // Clef & Layout
  clef: ClefType;
  problemCount: ProblemCountOption;
  
  // Root Selection
  rootPreset: RootPreset;
  allowedRoots: RootOption[];
  
  // Quality Selection
  allowedQualities: ChordQuality[];
  
  // Inversion Settings
  inversionWeight: InversionWeight;
  /** Custom weights: [root, first, second] as percentages (should sum to 100) */
  customInversionWeights: [number, number, number];
}

// ============================================================================
// CONSTANTS
// ============================================================================

export const ALL_ROOTS: RootOption[] = [
  { note: 'c', accidental: null, label: 'C' },
  { note: 'c', accidental: 'sharp', label: 'C#' },
  { note: 'd', accidental: 'flat', label: 'Db' },
  { note: 'd', accidental: null, label: 'D' },
  { note: 'd', accidental: 'sharp', label: 'D#' },
  { note: 'e', accidental: 'flat', label: 'Eb' },
  { note: 'e', accidental: null, label: 'E' },
  { note: 'f', accidental: null, label: 'F' },
  { note: 'f', accidental: 'sharp', label: 'F#' },
  { note: 'g', accidental: 'flat', label: 'Gb' },
  { note: 'g', accidental: null, label: 'G' },
  { note: 'g', accidental: 'sharp', label: 'G#' },
  { note: 'a', accidental: 'flat', label: 'Ab' },
  { note: 'a', accidental: null, label: 'A' },
  { note: 'a', accidental: 'sharp', label: 'A#' },
  { note: 'b', accidental: 'flat', label: 'Bb' },
  { note: 'b', accidental: null, label: 'B' },
];

export const NATURAL_ROOTS: RootOption[] = ALL_ROOTS.filter(r => r.accidental === null);

export const COMMON_KEY_ROOTS: RootOption[] = [
  { note: 'c', accidental: null, label: 'C' },
  { note: 'd', accidental: null, label: 'D' },
  { note: 'e', accidental: null, label: 'E' },
  { note: 'f', accidental: null, label: 'F' },
  { note: 'g', accidental: null, label: 'G' },
  { note: 'a', accidental: null, label: 'A' },
  { note: 'b', accidental: 'flat', label: 'Bb' },
  { note: 'e', accidental: 'flat', label: 'Eb' },
];

export const SHARP_ROOTS: RootOption[] = ALL_ROOTS.filter(
  r => r.accidental === 'sharp' || r.accidental === null
);

export const FLAT_ROOTS: RootOption[] = ALL_ROOTS.filter(
  r => r.accidental === 'flat' || r.accidental === null
);

export const TRIAD_QUALITIES: ChordQuality[] = ['major', 'minor', 'diminished', 'augmented'];

export const SEVENTH_QUALITIES: ChordQuality[] = [
  'major7', 'minor7', 'dominant7', 'diminished7', 'half-diminished7', 'augmented7'
];

export const ALL_QUALITIES: ChordQuality[] = [...TRIAD_QUALITIES, ...SEVENTH_QUALITIES, 'sus2', 'sus4'];

// ============================================================================
// DEFAULT SETTINGS
// ============================================================================

const DEFAULT_SETTINGS: WorksheetSettings = {
  clef: 'treble',
  problemCount: 8,
  rootPreset: 'natural',
  allowedRoots: [...NATURAL_ROOTS],
  allowedQualities: ['major', 'minor'],
  inversionWeight: 'root-only',
  customInversionWeights: [50, 25, 25],
};

// ============================================================================
// STORE
// ============================================================================

// Wrap in createRoot to avoid "computations created outside createRoot" warning
const { worksheetSettingsStore } = createRoot(() => {
  const [settings, setSettings] = createStore<WorksheetSettings>({ ...DEFAULT_SETTINGS });

  return {
    worksheetSettingsStore: {
      /** Get the current settings (reactive) */
      get state() {
        return settings;
      },

      /** Update clef */
      setClef(clef: ClefType) {
        setSettings('clef', clef);
      },

      /** Update problem count */
      setProblemCount(count: ProblemCountOption) {
        setSettings('problemCount', count);
      },

      /** Set root preset and update allowed roots accordingly */
      setRootPreset(preset: RootPreset) {
        setSettings('rootPreset', preset);
        
        switch (preset) {
          case 'all':
            setSettings('allowedRoots', [...ALL_ROOTS]);
            break;
          case 'natural':
            setSettings('allowedRoots', [...NATURAL_ROOTS]);
            break;
          case 'common-keys':
            setSettings('allowedRoots', [...COMMON_KEY_ROOTS]);
            break;
          case 'sharps':
            setSettings('allowedRoots', [...SHARP_ROOTS]);
            break;
          case 'flats':
            setSettings('allowedRoots', [...FLAT_ROOTS]);
            break;
          // 'custom' keeps current selection
        }
      },

      /** Toggle a specific root on/off */
      toggleRoot(root: RootOption) {
        const current = settings.allowedRoots;
        const exists = current.some(r => r.note === root.note && r.accidental === root.accidental);
        
        if (exists) {
          // Don't allow removing last root
          if (current.length > 1) {
            setSettings('allowedRoots', current.filter(
              r => !(r.note === root.note && r.accidental === root.accidental)
            ));
          }
        } else {
          setSettings('allowedRoots', [...current, root]);
        }
        
        // Switch to custom preset when manually toggling
        setSettings('rootPreset', 'custom');
      },

      /** Toggle a quality on/off */
      toggleQuality(quality: ChordQuality) {
        const current = settings.allowedQualities;
        const exists = current.includes(quality);
        
        if (exists) {
          // Don't allow removing last quality
          if (current.length > 1) {
            setSettings('allowedQualities', current.filter(q => q !== quality));
          }
        } else {
          setSettings('allowedQualities', [...current, quality]);
        }
      },

      /** Set all triads on/off */
      setTriads(enabled: boolean) {
        const current = settings.allowedQualities;
        if (enabled) {
          const newQualities = [...new Set([...current, ...TRIAD_QUALITIES])];
          setSettings('allowedQualities', newQualities);
        } else {
          const filtered = current.filter(q => !TRIAD_QUALITIES.includes(q));
          // Ensure at least one quality remains
          setSettings('allowedQualities', filtered.length > 0 ? filtered : ['major']);
        }
      },

      /** Set all sevenths on/off */
      setSevenths(enabled: boolean) {
        const current = settings.allowedQualities;
        if (enabled) {
          const newQualities = [...new Set([...current, ...SEVENTH_QUALITIES])];
          setSettings('allowedQualities', newQualities);
        } else {
          const filtered = current.filter(q => !SEVENTH_QUALITIES.includes(q));
          // Ensure at least one quality remains
          setSettings('allowedQualities', filtered.length > 0 ? filtered : ['major']);
        }
      },

      /** Set inversion weight mode */
      setInversionWeight(weight: InversionWeight) {
        setSettings('inversionWeight', weight);
      },

      /** Set custom inversion weights */
      setCustomInversionWeights(weights: [number, number, number]) {
        setSettings('customInversionWeights', weights);
        setSettings('inversionWeight', 'custom');
      },

      /** Reset to defaults */
      reset() {
        setSettings({ ...DEFAULT_SETTINGS });
      },

      // ========== RANDOMIZATION ==========

      /** Generate a single random chord based on current settings */
      generateRandomChord(): ChordDefinition {
        const { allowedRoots, allowedQualities, inversionWeight, customInversionWeights } = settings;
        
        // Pick random root
        const root = allowedRoots[Math.floor(Math.random() * allowedRoots.length)];
        
        // Pick random quality
        const quality = allowedQualities[Math.floor(Math.random() * allowedQualities.length)];
        
        // Pick inversion based on weight
        const inversion = this.pickInversion(quality, inversionWeight, customInversionWeights);
        
        return {
          root: root.note,
          rootAccidental: root.accidental,
          quality,
          inversion,
        };
      },

      /** Pick an inversion based on weight settings */
      pickInversion(
        quality: ChordQuality,
        weight: InversionWeight,
        customWeights: [number, number, number]
      ): ChordInversion {
        if (weight === 'root-only') {
          return 'root';
        }
        
        // Determine max inversion (triads have 2, 7ths have 3)
        const isSeventh = quality.includes('7');
        const maxInversion = isSeventh ? 3 : 2;
        
        let weights: number[];
        switch (weight) {
          case 'balanced':
            weights = isSeventh ? [25, 25, 25, 25] : [33, 33, 34];
            break;
          case 'favor-root':
            weights = isSeventh ? [50, 17, 17, 16] : [50, 25, 25];
            break;
          case 'custom':
            weights = isSeventh 
              ? [...customWeights, 100 - customWeights[0] - customWeights[1] - customWeights[2]]
              : customWeights;
            break;
          default:
            return 'root';
        }
        
        // Weighted random selection
        const rand = Math.random() * 100;
        let cumulative = 0;
        
        for (let i = 0; i <= maxInversion; i++) {
          cumulative += weights[i];
          if (rand < cumulative) {
            return (['root', 'first', 'second', 'third'] as ChordInversion[])[i];
          }
        }
        
        return 'root';
      },

      /** Get the appropriate octave for the current clef (centered around D3 for bass, D4 for treble) */
      getOctaveForClef(): 3 | 4 {
        // Bass clef: center around D3 (uses octave 3)
        // Treble clef: center around D4 (uses octave 4)
        return settings.clef === 'bass' ? 3 : 4;
      },

      /** Get density configuration for current problem count */
      getDensityConfig(): DensityConfig {
        return PROBLEM_COUNT_CONFIG[settings.problemCount];
      },
    },
  };
});

export { worksheetSettingsStore };
