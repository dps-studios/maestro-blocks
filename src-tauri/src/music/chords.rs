// Chord parsing, transposition, and validation
// This module contains the core chord manipulation logic

use super::types::{Chord, ChordNotation, ChordValidationResult, MusicError, MusicResult};
use super::notes::{note_index, get_preferred_note_name, get_key_signature_type, KeyType};
use super::roman;

/// Diatonic chords in C major (I, ii, iii, IV, V, vi, vii)
const DIATONIC_IN_C: &[(&str, u8)] = &[
    ("C", 0),    // I
    ("Dm", 2),   // ii
    ("Em", 4),   // iii
    ("F", 5),    // IV
    ("G", 7),    // V
    ("Am", 9),   // vi
    ("Bm", 11),  // vii (using m instead of dim for simplicity)
];

/// Diatonic chords in C minor (i, ii°, III, iv, v, VI, VII)
const DIATONIC_MINOR_IN_C: &[(&str, u8)] = &[
    ("Cm", 0),   // i
    ("Ddim", 2), // ii°
    ("Eb", 3),   // III
    ("Fm", 5),   // iv
    ("Gm", 7),   // v
    ("Ab", 8),   // VI
    ("Bb", 10),  // VII
];

/// Parse a chord string into root, suffix, and isMinor flag
/// Examples: "C" → (C, "", false), "Dm7" → (D, "m7", true), "C/E" → (C, "", false) with bass
pub fn parse_chord(chord: &str) -> MusicResult<Chord> {

    if chord.is_empty() {
        return Ok(Chord {
            root: String::new(),
            suffix: String::new(),
            bass: None,
        });
    }

    // Handle slash chords (e.g., "C/E")
    let (main_chord, bass) = if let Some(slash_pos) = chord.find('/') {
        let main = &chord[..slash_pos];
        let bass_note = &chord[slash_pos + 1..].trim();
        (main, Some(bass_note.to_string()))
    } else {
        (chord, None)
    };

    // Parse root note (first character)
    let mut chars = main_chord.chars();
    let first = chars.next().ok_or_else(|| MusicError::InvalidChord(chord.to_string()))?;

    if !first.is_ascii_uppercase() {
        return Err(MusicError::InvalidChord(format!("Root must be uppercase: {}", chord)));
    }

    let mut root = first.to_string();
    let remainder: String = chars.collect();

    // Check for accidental (# or b) immediately after root
    let suffix = if let Some(first_char) = remainder.chars().next() {
        if first_char == '#' || first_char == 'b' {
            root.push(first_char);
            remainder.chars().skip(1).collect()
        } else {
            remainder
        }
    } else {
        String::new()
    };


    
    Ok(Chord { root, suffix, bass })
}

/// Transpose a chord from one key to another with proper enharmonic spelling
/// This is the most frequently called function in the engine
pub fn transpose_chord(
    chord: &str,
    from_key: &str,
    to_key: &str,
    use_flats: bool,
) -> MusicResult<String> {
    if chord.is_empty() {
        return Ok(chord.to_string());
    }

    // Calculate transposition interval
    let from_idx = note_index(from_key)?;
    let to_idx = note_index(to_key)?;
    let interval = ((to_idx as i16 - from_idx as i16).rem_euclid(12)) as u8;

    // Parse chord to extract root, suffix, and optional bass
    let parsed = parse_chord(chord)?;

    // Transpose the main chord root
    let root_idx = note_index(&parsed.root)?;
    let new_root_idx = (root_idx + interval) % 12;
    let new_root = get_preferred_note_name(new_root_idx, to_key, use_flats);

    // Transpose bass note if present (slash chord)
    let result = if let Some(bass_note) = parsed.bass {
        let bass_idx = note_index(&bass_note)?;
        let new_bass_idx = (bass_idx + interval) % 12;
        let new_bass = get_preferred_note_name(new_bass_idx, to_key, use_flats);
        format!("{}{}/{}", new_root, parsed.suffix, new_bass)
    } else {
        format!("{}{}", new_root, parsed.suffix)
    };

    Ok(result)
}

/// Normalize chord notation to match the key signature
/// E.g., "D#m" in Eb major → "Ebm", "Fb" in C major → "E"
pub fn normalize_chord_to_key(chord: &str, key: &str) -> MusicResult<String> {
    let parsed = parse_chord(chord)?;
    
    if parsed.root.is_empty() {
        return Ok(chord.to_string());
    }

    // Get key signature type (sharp or flat)
    let key_type = get_key_signature_type(key);

    // Convert root to semitone index
    let root_idx = note_index(&parsed.root)?;

    // Get preferred note name for this key
    let use_flats = key_type == KeyType::Flat;
    let normalized_root = get_preferred_note_name(root_idx, key, use_flats);

    // Rebuild chord with normalized root - append everything after root as-is
    // This matches TypeScript: normalizedRoot + chord.slice(parsed.root.length)
    // Don't try to normalize bass note since it might be interval notation like "/b2"
    let suffix_and_rest = &chord[parsed.root.len()..];
    let result = format!("{}{}", normalized_root, suffix_and_rest);



    Ok(result)
}

/// Get diatonic chords for a major key
/// Returns: I, ii, iii, IV, V, vi, vii chords
pub fn get_diatonic_chords(
    key: &str,
    use_flats: bool,
) -> MusicResult<Vec<String>> {
    DIATONIC_IN_C
        .iter()
        .map(|(chord, _interval)| transpose_chord(chord, "C", key, use_flats))
        .collect()
}

/// Get diatonic chords for a minor key
/// Returns: i, ii°, III, iv, v, VI, VII chords
pub fn get_minor_diatonic_chords(
    key: &str,
    use_flats: bool,
) -> MusicResult<Vec<String>> {
    DIATONIC_MINOR_IN_C
        .iter()
        .map(|(chord, _interval)| transpose_chord(chord, "C", key, use_flats))
        .collect()
}

/// Validate chord input (accepts chord names or Roman numerals)
pub fn validate_chord_input(
    input: &str,
    key: &str,
    use_flats: bool,
) -> MusicResult<ChordValidationResult> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Ok(ChordValidationResult {
            valid: false,
            chord: None,
            normalized_chord: None,
            message: None,
            input_type: None,
        });
    }

    // First, try to parse as a chord name (starts with A-G)
    if let Some(first_char) = trimmed.chars().next() {
        if first_char.is_ascii_uppercase() && "ABCDEFG".contains(first_char) {
            if let Ok(parsed) = parse_chord(trimmed) {
                if !parsed.root.is_empty() {
                    return Ok(ChordValidationResult {
                        valid: true,
                        chord: Some(trimmed.to_string()),
                        normalized_chord: Some(trimmed.to_string()),
                        message: None,
                        input_type: Some("chord".to_string()),
                    });
                }
            }
        }
    }

    // Try to parse as a Roman numeral
    if let Ok(chord) = roman::roman_numeral_to_chord(trimmed, key, use_flats) {
        return Ok(ChordValidationResult {
            valid: true,
            chord: Some(chord.clone()),
            normalized_chord: Some(chord),
            message: None,
            input_type: Some("numeral".to_string()),
        });
    }

    // Neither valid chord nor valid Roman numeral
    Ok(ChordValidationResult {
        valid: false,
        chord: None,
        normalized_chord: None,
        message: Some(format!("Invalid chord or numeral: {}", trimmed)),
        input_type: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chord_simple() {
        let chord = parse_chord("C").unwrap();
        assert_eq!(chord.root, "C");
        assert_eq!(chord.suffix, "");
        assert!(chord.bass.is_none());
    }

    #[test]
    fn test_parse_chord_with_suffix() {
        let chord = parse_chord("Dm7").unwrap();
        assert_eq!(chord.root, "D");
        assert_eq!(chord.suffix, "m7");
        assert!(chord.bass.is_none());
    }

    #[test]
    fn test_parse_chord_with_accidental() {
        let chord = parse_chord("F#m").unwrap();
        assert_eq!(chord.root, "F#");
        assert_eq!(chord.suffix, "m");

        let chord2 = parse_chord("Bbmaj7").unwrap();
        assert_eq!(chord2.root, "Bb");
        assert_eq!(chord2.suffix, "maj7");
    }

    #[test]
    fn test_parse_slash_chord() {
        let chord = parse_chord("C/E").unwrap();
        assert_eq!(chord.root, "C");
        assert_eq!(chord.suffix, "");
        assert_eq!(chord.bass, Some("E".to_string()));
    }

    #[test]
    fn test_transpose_c_to_d() {
        assert_eq!(transpose_chord("C", "C", "D", true).unwrap(), "D");
    }

    #[test]
    fn test_transpose_with_suffix() {
        assert_eq!(transpose_chord("Dm7", "C", "G", true).unwrap(), "Am7");
    }

    #[test]
    fn test_transpose_slash_chord() {
        assert_eq!(transpose_chord("C/E", "C", "G", true).unwrap(), "G/B");
    }

    #[test]
    fn test_transpose_enharmonic() {
        // Transposing to Db (flat key) should use flats
        assert_eq!(transpose_chord("C", "C", "Db", true).unwrap(), "Db");

        // Transposing to C# (sharp key) should use sharps
        assert_eq!(transpose_chord("C", "C", "C#", false).unwrap(), "C#");
    }

    #[test]
    fn test_transpose_round_trip() {
        let original = "Dm7";
        let transposed = transpose_chord(original, "C", "G", true).unwrap();
        let back = transpose_chord(&transposed, "G", "C", true).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn test_normalize_chord_to_key() {
        // D# in Eb major should become Eb
        let result = normalize_chord_to_key("D#m", "Eb").unwrap();
        assert_eq!(result, "Ebm");

        // C# in G major should stay C# (sharp key)
        let result2 = normalize_chord_to_key("C#", "G").unwrap();
        assert_eq!(result2, "C#");
    }

    #[test]
    fn test_get_diatonic_chords() {
        let chords = get_diatonic_chords("C", true).unwrap();
        assert_eq!(chords, vec!["C", "Dm", "Em", "F", "G", "Am", "Bm"]);

        let g_chords = get_diatonic_chords("G", true).unwrap();
        assert_eq!(g_chords, vec!["G", "Am", "Bm", "C", "D", "Em", "F#m"]);
    }

    #[test]
    fn test_get_minor_diatonic_chords() {
        let chords = get_minor_diatonic_chords("C", true).unwrap();
        assert_eq!(chords, vec!["Cm", "Ddim", "Eb", "Fm", "Gm", "Ab", "Bb"]);
    }

    #[test]
    fn test_validate_chord() {
        let result = validate_chord_input("C", "C", true).unwrap();
        assert!(result.valid);

        let result2 = validate_chord_input("Dm7", "C", true).unwrap();
        assert!(result2.valid);
    }

    #[test]
    fn test_get_initial_chords() {
        let chords = get_initial_chords("C", true).unwrap();
        assert!(!chords.is_empty());
        assert!(chords.contains(&"C".to_string()));
    }
}

/// Get initial chords for a key (diatonic in center, non-diatonic on edges)
pub fn get_initial_chords(key: &str, use_flats: bool) -> MusicResult<Vec<String>> {
    let diatonic = get_diatonic_chords(key, use_flats)?;
    let mut initial = Vec::new();
    
    // Add some common non-diatonic chords
    let common_non_diatonic = vec![
        "bVII", "bIII", "bVI", "ii7", "vi7", "iii7"
    ];
    
    for numeral in common_non_diatonic {
        if let Ok(chord) = roman::roman_numeral_to_chord(numeral, key, use_flats) {
            initial.push(chord);
        }
    }
    
    // Combine diatonic and non-diatonic
    initial.extend(diatonic);
    Ok(initial)
}

/// Prepare a chord for display by computing its roman numeral
/// This preserves the chord spelling as given - no re-normalization
/// The caller (recommendations, user input) has already chosen the correct enharmonic spelling
pub fn prepare_chord_display(chord: &str, key: &str) -> MusicResult<ChordNotation> {
    // Compute roman numeral for display - preserve chord as-is
    let numeral = roman::get_display_numeral(chord, key)?;

    Ok(ChordNotation {
        chord: chord.to_string(),
        numeral,
    })
}
