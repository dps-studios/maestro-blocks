/**
 * Music theory service - interfaces with Rust backend
 */

import { invoke } from '@tauri-apps/api/core';
import type { Pitch, ChordDefinition, Octave, NoteName, Accidental } from '../types/score';

// Types matching Rust structs
interface PitchResult {
  note: string;
  octave: number;
}

interface ChordResponse {
  pitches: PitchResult[];
  display_name: string;
}

interface ChordRequest {
  root: string;
  quality: string;
  root_octave: number;
}

/**
 * Convert a PitchResult from Rust to our Pitch type
 */
function pitchResultToPitch(result: PitchResult): Pitch {
  // Parse note name and accidental from string like "C#", "Bb", "D"
  const noteChar = result.note.charAt(0).toLowerCase() as NoteName;
  let accidental: Accidental = null;
  
  if (result.note.length > 1) {
    const modifier = result.note.substring(1);
    if (modifier === '#') accidental = 'sharp';
    else if (modifier === 'b') accidental = 'flat';
    else if (modifier === '##') accidental = 'double-sharp';
    else if (modifier === 'bb') accidental = 'double-flat';
  }
  
  return {
    note: noteChar,
    accidental,
    octave: result.octave as Octave,
  };
}

/**
 * Format root note for Rust (e.g., "C", "F#", "Bb")
 */
function formatRootForRust(note: NoteName, accidental: Accidental): string {
  const noteUpper = note.toUpperCase();
  if (accidental === 'sharp') return `${noteUpper}#`;
  if (accidental === 'flat') return `${noteUpper}b`;
  if (accidental === 'double-sharp') return `${noteUpper}##`;
  if (accidental === 'double-flat') return `${noteUpper}bb`;
  return noteUpper;
}

/**
 * Generate chord pitches using Rust backend
 */
export async function generateChordPitchesRust(
  def: ChordDefinition,
  rootOctave: Octave = 4
): Promise<{ pitches: Pitch[]; displayName: string }> {
  const request: ChordRequest = {
    root: formatRootForRust(def.root, def.rootAccidental),
    quality: def.quality,
    root_octave: rootOctave,
  };
  
  console.log('[Music] Sending chord request to Rust:', request);
  
  try {
    const response = await invoke<ChordResponse>('generate_chord_pitches', { request });
    
    console.log('[Music] Rust response (raw):', response);
    
    const pitches = response.pitches.map(pitchResultToPitch);
    
    console.log('[Music] Converted pitches:', pitches);
    console.log('[Music] Display name:', response.display_name);
    
    return {
      pitches,
      displayName: response.display_name,
    };
  } catch (error) {
    console.error('[Music] Rust chord generation failed:', error);
    // Fallback to a simple root-only response
    return {
      pitches: [{
        note: def.root,
        accidental: def.rootAccidental,
        octave: rootOctave,
      }],
      displayName: `${def.root.toUpperCase()}${def.rootAccidental === 'sharp' ? '#' : def.rootAccidental === 'flat' ? 'b' : ''}`,
    };
  }
}

/**
 * Get all available chord qualities from Rust
 */
export async function getChordQualities(): Promise<string[]> {
  try {
    return await invoke<string[]>('get_chord_qualities');
  } catch (error) {
    console.error('[Music] Failed to get chord qualities:', error);
    // Return default qualities
    return ['maj', 'min', 'dim', 'aug', 'maj7', 'min7', '7', 'dim7'];
  }
}
