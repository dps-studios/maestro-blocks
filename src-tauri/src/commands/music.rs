use serde::{Deserialize, Serialize};

/// A note with octave for rendering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PitchResult {
    pub note: String,      // "C", "D#", "Bb", etc.
    pub octave: u8,        // 0-8
}

/// Request to generate chord pitches
#[derive(Debug, Clone, Deserialize)]
pub struct ChordRequest {
    pub root: String,           // "C", "F#", "Bb"
    pub quality: String,        // "maj", "min", "dim", "aug", "maj7", etc.
    pub root_octave: u8,        // Octave for the root (bottom) note
}

/// Response with generated chord pitches
#[derive(Debug, Clone, Serialize)]
pub struct ChordResponse {
    pub pitches: Vec<PitchResult>,
    pub display_name: String,   // "Cmaj7", "Dm", etc.
}

/// Map our UI quality names to Canon's suffix format
fn normalize_quality(quality: &str) -> &str {
    match quality {
        "major" => "maj",
        "minor" => "min",
        "diminished" => "dim",
        "augmented" => "aug",
        "major7" => "maj7",
        "minor7" => "min7",
        "dominant7" => "7",
        "diminished7" => "dim7",
        "half-diminished7" => "m7b5",
        "augmented7" => "aug7",
        "sus2" => "sus2",
        "sus4" => "sus4",
        other => other,
    }
}

/// Get display name for a chord
fn format_display_name(root: &str, quality: &str) -> String {
    let suffix = match quality {
        "major" | "maj" => "",
        "minor" | "min" => "m",
        "diminished" | "dim" => "dim",
        "augmented" | "aug" => "aug",
        "major7" | "maj7" => "maj7",
        "minor7" | "min7" => "m7",
        "dominant7" | "7" => "7",
        "diminished7" | "dim7" => "dim7",
        "half-diminished7" | "m7b5" => "m7b5",
        "augmented7" | "aug7" => "aug7",
        "sus2" => "sus2",
        "sus4" => "sus4",
        other => other,
    };
    format!("{}{}", root, suffix)
}

/// Convert note name to semitone offset from C
fn note_to_semitone(note: &str) -> Option<i32> {
    let base = match note.chars().next()? {
        'C' | 'c' => 0,
        'D' | 'd' => 2,
        'E' | 'e' => 4,
        'F' | 'f' => 5,
        'G' | 'g' => 7,
        'A' | 'a' => 9,
        'B' | 'b' => 11,
        _ => return None,
    };
    
    let modifier: i32 = note.chars().skip(1).map(|c| match c {
        '#' => 1,
        'b' => -1,
        _ => 0,
    }).sum();
    
    Some((base + modifier + 12) % 12)
}

// Removed obsolete semitone_to_note function - now using diatonic spelling from music::intervals

/// Generate chord pitches from root, quality, and octave
#[tauri::command]
pub fn generate_chord_pitches(request: ChordRequest) -> Result<ChordResponse, String> {
    use crate::music::intervals::{parse_chord_with_interval_specs, spell_interval_with_degree};
    
    let quality = normalize_quality(&request.quality);
    
    // Get interval specifications (with explicit degrees) for this chord quality
    let interval_specs = parse_chord_with_interval_specs(quality)
        .map_err(|e| format!("Failed to parse chord quality: {}", e))?;
    
    // Get root semitone (for octave calculation)
    let root_semitone = note_to_semitone(&request.root)
        .ok_or_else(|| format!("Invalid root note: {}", request.root))?;
    
    // Generate pitches from interval specifications with correct spelling
    let mut pitches: Vec<PitchResult> = Vec::new();
    let mut current_octave = request.root_octave;
    let mut prev_semitone = root_semitone;
    
    for (i, &(semitones, degree)) in interval_specs.iter().enumerate() {
        let absolute_semitone = root_semitone + (semitones as i32);
        let note_semitone = ((absolute_semitone % 12) + 12) % 12;
        
        // Determine octave: if this note's semitone is lower than the previous,
        // it has wrapped around the octave, so increment
        if i > 0 && note_semitone < prev_semitone {
            current_octave += 1;
        }
        prev_semitone = note_semitone;
        
        // Use correct diatonic spelling with explicit scale degree
        let note = spell_interval_with_degree(&request.root, semitones, degree)
            .map_err(|e| format!("Spelling error: {}", e))?;
        
        pitches.push(PitchResult {
            note,
            octave: current_octave,
        });
    }
    
    let display_name = format_display_name(&request.root, &request.quality);
    
    Ok(ChordResponse {
        pitches,
        display_name,
    })
}

/// Get all available chord qualities
#[tauri::command]
pub fn get_chord_qualities() -> Vec<String> {
    use crate::music::intervals::CHORD_INTERVAL_SPECS;
    CHORD_INTERVAL_SPECS.keys().map(|s| s.to_string()).collect()
}

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_fm7_pitches() {
        // THE CRITICAL TEST CASE
        let request = ChordRequest {
            root: "F".to_string(),
            quality: "minor7".to_string(),
            root_octave: 3,
        };

        let response = generate_chord_pitches(request).unwrap();

        assert_eq!(response.pitches.len(), 4);
        assert_eq!(response.pitches[0].note, "F");
        assert_eq!(response.pitches[1].note, "Ab");  // Not G#!
        assert_eq!(response.pitches[2].note, "C");
        assert_eq!(response.pitches[3].note, "Eb");  // Not D#!

        // Verify octaves (F=5, Ab=8 stay in oct 3; C=0, Eb=3 wrap to oct 4)
        assert_eq!(response.pitches[0].octave, 3);  // F
        assert_eq!(response.pitches[1].octave, 3);  // Ab
        assert_eq!(response.pitches[2].octave, 4);  // C (wrapped)
        assert_eq!(response.pitches[3].octave, 4);  // Eb
    }

    #[test]
    fn test_generate_dbaug_pitches() {
        // User spec: augmented with flat root should naturalize
        let request = ChordRequest {
            root: "Db".to_string(),
            quality: "augmented".to_string(),
            root_octave: 4,
        };

        let response = generate_chord_pitches(request).unwrap();

        assert_eq!(response.pitches[0].note, "Db");
        assert_eq!(response.pitches[1].note, "F");
        assert_eq!(response.pitches[2].note, "A");  // Natural, not Ab#
    }

    #[test]
    fn test_generate_cm7_pitches() {
        let request = ChordRequest {
            root: "C".to_string(),
            quality: "minor7".to_string(),
            root_octave: 3,
        };

        let response = generate_chord_pitches(request).unwrap();

        assert_eq!(response.pitches.len(), 4);
        assert_eq!(response.pitches[0].note, "C");
        assert_eq!(response.pitches[1].note, "Eb");
        assert_eq!(response.pitches[2].note, "G");
        assert_eq!(response.pitches[3].note, "Bb");
    }

    #[test]
    fn test_generate_major7_pitches() {
        let request = ChordRequest {
            root: "F".to_string(),
            quality: "major7".to_string(),
            root_octave: 3,
        };

        let response = generate_chord_pitches(request).unwrap();

        assert_eq!(response.pitches.len(), 4);
        assert_eq!(response.pitches[0].note, "F");
        assert_eq!(response.pitches[1].note, "A");
        assert_eq!(response.pitches[2].note, "C");
        assert_eq!(response.pitches[3].note, "E");
    }

    #[test]
    fn test_octave_wrapping() {
        // Test that octaves increment correctly when notes wrap
        let request = ChordRequest {
            root: "B".to_string(),
            quality: "major".to_string(),
            root_octave: 3,
        };

        let response = generate_chord_pitches(request).unwrap();

        assert_eq!(response.pitches[0].note, "B");
        assert_eq!(response.pitches[0].octave, 3);
        
        // D# (semitone 3) is lower than B (semitone 11), so it wraps to octave 4
        assert_eq!(response.pitches[1].note, "D#");
        assert_eq!(response.pitches[1].octave, 4);
        
        // F# (semitone 6) is higher than D# (semitone 3), stays in octave 4
        assert_eq!(response.pitches[2].note, "F#");
        assert_eq!(response.pitches[2].octave, 4);
    }

    #[test]
    fn test_diminished_spelling() {
        let request = ChordRequest {
            root: "C".to_string(),
            quality: "diminished".to_string(),
            root_octave: 4,
        };

        let response = generate_chord_pitches(request).unwrap();

        assert_eq!(response.pitches[0].note, "C");
        assert_eq!(response.pitches[1].note, "Eb");  // Minor 3rd
        assert_eq!(response.pitches[2].note, "Gb");  // Diminished 5th
    }

    #[test]
    fn test_quality_normalization() {
        // Test that quality aliases work
        let request1 = ChordRequest {
            root: "C".to_string(),
            quality: "minor".to_string(),
            root_octave: 3,
        };

        let request2 = ChordRequest {
            root: "C".to_string(),
            quality: "min".to_string(),
            root_octave: 3,
        };

        let response1 = generate_chord_pitches(request1).unwrap();
        let response2 = generate_chord_pitches(request2).unwrap();

        assert_eq!(response1.pitches, response2.pitches);
    }
}
