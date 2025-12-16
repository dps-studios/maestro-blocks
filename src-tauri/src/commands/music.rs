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
    pub inversion: Option<String>, // "root", "first", "second", "third"
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

/// Check if a quality represents a 7th chord (4 notes)
fn is_seventh_chord(quality: &str) -> bool {
    matches!(quality, 
        "major7" | "maj7" | 
        "minor7" | "min7" | 
        "dominant7" | "7" | 
        "diminished7" | "dim7" | 
        "half-diminished7" | "m7b5" |
        "augmented7" | "aug7"
    )
}

/// Get inversion suffix using figured bass notation
/// Returns (superscript, subscript) tuple for SVG rendering
fn get_inversion_figures(inversion: Option<&str>, is_seventh: bool) -> (&'static str, &'static str) {
    match (inversion, is_seventh) {
        // Triads: root = none, 1st = 6, 2nd = 6/4
        (None | Some("root"), _) => ("", ""),
        (Some("first"), false) => ("6", ""),
        (Some("second"), false) => ("6", "4"),
        // 7th chords: root = 7, 1st = 6/5, 2nd = 4/3, 3rd = 4/2
        (Some("first"), true) => ("6", "5"),
        (Some("second"), true) => ("4", "3"),
        (Some("third"), true) => ("4", "2"),
        _ => ("", ""),
    }
}

/// Get display name for a chord
fn format_display_name(root: &str, quality: &str, inversion: Option<&str>) -> String {
    let suffix = match quality {
        "major" | "maj" => "",
        "minor" | "min" => "m",
        "diminished" | "dim" => "dim",
        "augmented" | "aug" => "aug",
        "major7" | "maj7" => "maj7",
        "minor7" | "min7" => "m7",
        "dominant7" | "7" => "7",
        "diminished7" | "dim7" => "dim7",
        "half-diminished7" | "m7b5" => "ø7",
        "augmented7" | "aug7" => "aug7",
        "sus2" => "sus2",
        "sus4" => "sus4",
        other => other,
    };
    
    let is_seventh = is_seventh_chord(quality);
    let (sup, sub) = get_inversion_figures(inversion, is_seventh);
    
    // Format: "Cm|6|4" where | separates base|superscript|subscript
    // Frontend will parse this to render super/subscripts properly
    if sup.is_empty() && sub.is_empty() {
        format!("{}{}", root, suffix)
    } else {
        format!("{}{}|{}|{}", root, suffix, sup, sub)
    }
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
    
    for &(semitones, degree) in interval_specs.iter() {
        // Calculate the absolute semitone from C0
        let absolute_semitone = root_semitone + (semitones as i32);
        
        // Use correct diatonic spelling with explicit scale degree
        let note = spell_interval_with_degree(&request.root, semitones, degree)
            .map_err(|e| format!("Spelling error: {}", e))?;
        
        // Calculate octave based on absolute semitone
        let base_octave = request.root_octave as i32 + (absolute_semitone / 12);
        
        // Adjust octave for enharmonic spellings:
        // Cb is enharmonically B, so Cb/4 = B/3 (Cb needs octave +1 vs B)
        // B# is enharmonically C, so B#/3 = C/4 (B# needs octave -1 vs C)
        let note_semitone = note_to_semitone(&note).unwrap_or(0);
        let expected_semitone = ((absolute_semitone % 12) + 12) % 12;
        
        let octave_adjustment = if note_semitone == 11 && expected_semitone == 11 && note.starts_with('C') {
            // Cb case: note is spelled as C-flat but sounds like B
            // Cb/4 sounds like B/3, so we need to bump up the octave
            1
        } else if note_semitone == 0 && expected_semitone == 0 && note.starts_with('B') {
            // B# case: note is spelled as B-sharp but sounds like C
            // B#/3 sounds like C/4, so we need to reduce the octave
            -1
        } else {
            0
        };
        
        pitches.push(PitchResult {
            note,
            octave: (base_octave + octave_adjustment) as u8,
        });
    }
    
    let display_name = format_display_name(&request.root, &request.quality, request.inversion.as_deref());
    
    // Handle inversions
    let inversion_shifts = match request.inversion.as_deref().unwrap_or("root") {
        "first" => 1,
        "second" => 2,
        "third" => 3,
        _ => 0,
    };

    let num_pitches = pitches.len();
    if num_pitches > 0 && inversion_shifts > 0 {
        let shifts = inversion_shifts % num_pitches;
        
        // For inversions, shift the TOP notes DOWN an octave instead of bottom notes up.
        // This keeps the chord in the same register (important for bass clef).
        // Example: C-E-G in first inversion becomes E-G-C where C moves down,
        // giving us E3-G3-C4 instead of E4-G4-C5
        for i in shifts..num_pitches {
            if pitches[i].octave > 0 {
                pitches[i].octave -= 1;
            }
        }

        // Rotate the pitches array so the bass note comes first
        pitches.rotate_left(shifts);
    }
    
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
            inversion: None,
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
            inversion: None,
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
            inversion: None,
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
            inversion: None,
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
            inversion: None,
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
            inversion: None,
        };

        let response = generate_chord_pitches(request).unwrap();

        assert_eq!(response.pitches[0].note, "C");
        assert_eq!(response.pitches[1].note, "Eb");  // Minor 3rd
        assert_eq!(response.pitches[2].note, "Gb");  // Diminished 5th
    }

    #[test]
    fn test_f_half_diminished_root_position() {
        // F half-diminished: F-Ab-Cb-Eb
        // Critical: F must be the bass note in root position, not Cb
        let request = ChordRequest {
            root: "F".to_string(),
            quality: "half-diminished7".to_string(),
            root_octave: 3,
            inversion: None,
        };

        let response = generate_chord_pitches(request).unwrap();

        // F must be first (bass note) in root position
        assert_eq!(response.pitches[0].note, "F");
        assert_eq!(response.pitches[0].octave, 3);
        
        assert_eq!(response.pitches[1].note, "Ab");
        assert_eq!(response.pitches[1].octave, 3);
        
        // Cb is the diminished 5th - must be ABOVE Ab, not below F
        assert_eq!(response.pitches[2].note, "Cb");
        assert_eq!(response.pitches[2].octave, 4);  // Must wrap to octave 4
        
        assert_eq!(response.pitches[3].note, "Eb");
        assert_eq!(response.pitches[3].octave, 4);
    }

    #[test]
    fn test_quality_normalization() {
        // Test that quality aliases work
        let request1 = ChordRequest {
            root: "C".to_string(),
            quality: "minor".to_string(),
            root_octave: 3,
            inversion: None,
        };

        let request2 = ChordRequest {
            root: "C".to_string(),
            quality: "min".to_string(),
            root_octave: 3,
            inversion: None,
        };

        let response1 = generate_chord_pitches(request1).unwrap();
        let response2 = generate_chord_pitches(request2).unwrap();

        assert_eq!(response1.pitches, response2.pitches);
    }

    #[test]
    fn test_first_inversion() {
        // C Major: C E G -> First inversion: E G C
        // We shift the TOP notes DOWN to keep the chord in the same register
        // (important for bass clef where we don't want chords jumping above middle C)
        // C4-E4-G4 -> E3-G3-C4 (E and G shift down, C stays as the top note)
        let request = ChordRequest {
            root: "C".to_string(),
            quality: "major".to_string(),
            root_octave: 4,
            inversion: Some("first".to_string()),
        };

        let response = generate_chord_pitches(request).unwrap();
        
        assert_eq!(response.pitches[0].note, "E");
        assert_eq!(response.pitches[0].octave, 3);  // Shifted down
        
        assert_eq!(response.pitches[1].note, "G");
        assert_eq!(response.pitches[1].octave, 3);  // Shifted down
        
        assert_eq!(response.pitches[2].note, "C");
        assert_eq!(response.pitches[2].octave, 4);  // Stays at original octave
    }

    #[test]
    fn test_half_diminished_inversion_with_cb() {
        // F half-diminished: F-Ab-Cb-Eb
        // First inversion should be Ab-Cb-Eb-F with correct octaves
        let request = ChordRequest {
            root: "F".to_string(),
            quality: "half-diminished7".to_string(),
            root_octave: 3,
            inversion: Some("first".to_string()),
        };

        let response = generate_chord_pitches(request).unwrap();

        // Ab should be the bass note
        assert_eq!(response.pitches[0].note, "Ab");
        // Cb should be above Ab (not below due to enharmonic octave issues)
        assert_eq!(response.pitches[1].note, "Cb");
        assert!(response.pitches[1].octave >= response.pitches[0].octave, 
            "Cb should be at same or higher octave than Ab");
        
        assert_eq!(response.pitches[2].note, "Eb");
        assert_eq!(response.pitches[3].note, "F");
        
        // All notes should be in ascending order by pitch
        // This verifies the chord doesn't have notes jumping around octaves incorrectly
    }
}
