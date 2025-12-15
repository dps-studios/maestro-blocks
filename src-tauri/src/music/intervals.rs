// Chord quality to interval mappings and chord decomposition
// This module handles converting chord suffixes to note intervals

use super::types::{MusicError, MusicResult};
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Interval specification: (semitones, scale_degree)
/// This encodes both the chromatic distance AND the diatonic degree for proper spelling
type IntervalSpec = (u8, u8);

/// Map of chord suffixes to interval specifications with explicit scale degrees
/// Each chord maps to (semitones, degree) pairs where:
/// - semitones: chromatic distance from root (0-23 for compound intervals)
/// - degree: diatonic scale degree (1-7, where extensions use base degree: 9→2, 11→4, 13→6)
pub static CHORD_INTERVAL_SPECS: Lazy<HashMap<&'static str, Vec<IntervalSpec>>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Major triads - 1, M3, P5
    m.insert("", vec![(0,1), (4,3), (7,5)]);
    m.insert("M", vec![(0,1), (4,3), (7,5)]);
    m.insert("maj", vec![(0,1), (4,3), (7,5)]);
    m.insert("major", vec![(0,1), (4,3), (7,5)]);

    // Minor triads - 1, m3, P5
    m.insert("m", vec![(0,1), (3,3), (7,5)]);
    m.insert("min", vec![(0,1), (3,3), (7,5)]);
    m.insert("minor", vec![(0,1), (3,3), (7,5)]);
    m.insert("-", vec![(0,1), (3,3), (7,5)]);

    // Diminished - 1, m3, d5
    m.insert("dim", vec![(0,1), (3,3), (6,5)]);
    m.insert("°", vec![(0,1), (3,3), (6,5)]);
    m.insert("o", vec![(0,1), (3,3), (6,5)]);

    // Augmented - 1, M3, A5
    m.insert("aug", vec![(0,1), (4,3), (8,5)]);
    m.insert("+", vec![(0,1), (4,3), (8,5)]);

    // Suspended - 1, M2/P4, P5
    m.insert("sus2", vec![(0,1), (2,2), (7,5)]);
    m.insert("sus4", vec![(0,1), (5,4), (7,5)]);
    m.insert("sus", vec![(0,1), (5,4), (7,5)]);

    // Seventh chords - add degree 7
    // Dominant 7th - 1, M3, P5, m7
    m.insert("7", vec![(0,1), (4,3), (7,5), (10,7)]);
    // Major 7th - 1, M3, P5, M7
    m.insert("maj7", vec![(0,1), (4,3), (7,5), (11,7)]);
    m.insert("M7", vec![(0,1), (4,3), (7,5), (11,7)]);
    m.insert("Maj7", vec![(0,1), (4,3), (7,5), (11,7)]);
    // Minor 7th - 1, m3, P5, m7
    m.insert("m7", vec![(0,1), (3,3), (7,5), (10,7)]);
    m.insert("min7", vec![(0,1), (3,3), (7,5), (10,7)]);
    // Minor-major 7th - 1, m3, P5, M7
    m.insert("mM7", vec![(0,1), (3,3), (7,5), (11,7)]);
    m.insert("mmaj7", vec![(0,1), (3,3), (7,5), (11,7)]);
    m.insert("mMaj7", vec![(0,1), (3,3), (7,5), (11,7)]);
    // Diminished 7th - 1, m3, d5, d7 (bb7)
    m.insert("dim7", vec![(0,1), (3,3), (6,5), (9,7)]);
    m.insert("°7", vec![(0,1), (3,3), (6,5), (9,7)]);
    // Half-diminished 7th - 1, m3, d5, m7
    m.insert("m7b5", vec![(0,1), (3,3), (6,5), (10,7)]);
    m.insert("ø", vec![(0,1), (3,3), (6,5), (10,7)]);
    m.insert("ø7", vec![(0,1), (3,3), (6,5), (10,7)]);
    // Augmented 7th - 1, M3, A5, m7
    m.insert("aug7", vec![(0,1), (4,3), (8,5), (10,7)]);
    m.insert("+7", vec![(0,1), (4,3), (8,5), (10,7)]);
    // 7 no 5 - 1, M3, m7
    m.insert("7no5", vec![(0,1), (4,3), (10,7)]);

    // Altered seventh chords - modify degree 5
    // 7b5 - 1, M3, d5, m7
    m.insert("7b5", vec![(0,1), (4,3), (6,5), (10,7)]);
    // 7#5 - 1, M3, A5, m7
    m.insert("7#5", vec![(0,1), (4,3), (8,5), (10,7)]);
    // maj7b5 - 1, M3, d5, M7
    m.insert("maj7b5", vec![(0,1), (4,3), (6,5), (11,7)]);
    // maj7#5 - 1, M3, A5, M7
    m.insert("maj7#5", vec![(0,1), (4,3), (8,5), (11,7)]);
    
    // Extended chords with altered 9ths - degree 2 (compound 2nd)
    // 7b9 - 1, M3, P5, m7, b9 (degree 2)
    m.insert("7b9", vec![(0,1), (4,3), (7,5), (10,7), (13,2)]);
    // 7#9 - 1, M3, P5, m7, #9 (degree 2)
    m.insert("7#9", vec![(0,1), (4,3), (7,5), (10,7), (15,2)]);
    // m7b9 - 1, m3, P5, m7, b9
    m.insert("m7b9", vec![(0,1), (3,3), (7,5), (10,7), (13,2)]);
    // 7alt - 1, M3, d5, m7, b9
    m.insert("7alt", vec![(0,1), (4,3), (6,5), (10,7), (13,2)]);
    
    // Extended chords with altered 11ths/13ths
    // maj7#11 - 1, M3, P5, M7, #11 (degree 4)
    m.insert("maj7#11", vec![(0,1), (4,3), (7,5), (11,7), (18,4)]);
    // 7b13 - 1, M3, P5, m7, b13 (degree 6)
    m.insert("7b13", vec![(0,1), (4,3), (7,5), (10,7), (20,6)]);

    // Suspended seventh chords
    // 7sus4 - 1, P4, P5, m7
    m.insert("7sus", vec![(0,1), (5,4), (7,5), (10,7)]);
    m.insert("7sus4", vec![(0,1), (5,4), (7,5), (10,7)]);
    // 7sus2 - 1, M2, P5, m7
    m.insert("7sus2", vec![(0,1), (2,2), (7,5), (10,7)]);
    // m7sus4 - 1, P4, P5, m7
    m.insert("m7sus4", vec![(0,1), (5,4), (7,5), (10,7)]);
    // 9sus4 - 1, P4, P5, m7, M9
    m.insert("9sus", vec![(0,1), (5,4), (7,5), (10,7), (14,2)]);
    m.insert("9sus4", vec![(0,1), (5,4), (7,5), (10,7), (14,2)]);
    // 7b9sus4 - 1, P4, P5, m7, b9
    m.insert("7b9sus4", vec![(0,1), (5,4), (7,5), (10,7), (13,2)]);

    // Sixth chords - use degree 6 instead of 7
    // 6 - 1, M3, P5, M6
    m.insert("6", vec![(0,1), (4,3), (7,5), (9,6)]);
    // m6 - 1, m3, P5, M6
    m.insert("m6", vec![(0,1), (3,3), (7,5), (9,6)]);
    m.insert("min6", vec![(0,1), (3,3), (7,5), (9,6)]);
    // 6/9 - 1, M3, P5, M6, M9
    m.insert("6/9", vec![(0,1), (4,3), (7,5), (9,6), (14,2)]);
    m.insert("69", vec![(0,1), (4,3), (7,5), (9,6), (14,2)]);
    // m6/9 - 1, m3, P5, M6, M9
    m.insert("m6/9", vec![(0,1), (3,3), (7,5), (9,6), (14,2)]);
    m.insert("m69", vec![(0,1), (3,3), (7,5), (9,6), (14,2)]);
    // 6sus2 - 1, M2, P5, M6
    m.insert("6sus2", vec![(0,1), (2,2), (7,5), (9,6)]);
    // 6sus4 - 1, P4, P5, M6
    m.insert("6sus4", vec![(0,1), (5,4), (7,5), (9,6)]);
    // 6add9 - 1, M3, P5, M6, M9
    m.insert("6add9", vec![(0,1), (4,3), (7,5), (9,6), (14,2)]);

    // Add chords - major triad + added tone
    // add2 - 1, M2, M3, P5 (note: 2nd before 3rd)
    m.insert("add2", vec![(0,1), (2,2), (4,3), (7,5)]);
    // add4 - 1, M3, P4, P5
    m.insert("add4", vec![(0,1), (4,3), (5,4), (7,5)]);
    // add9 - 1, M3, P5, M9
    m.insert("add9", vec![(0,1), (4,3), (7,5), (14,2)]);
    // add11 - 1, M3, P5, P11
    m.insert("add11", vec![(0,1), (4,3), (7,5), (17,4)]);
    // add13 - 1, M3, P5, M13
    m.insert("add13", vec![(0,1), (4,3), (7,5), (21,6)]);
    // madd2 - 1, M2, m3, P5
    m.insert("madd2", vec![(0,1), (2,2), (3,3), (7,5)]);
    // madd4 - 1, m3, P4, P5
    m.insert("madd4", vec![(0,1), (3,3), (5,4), (7,5)]);
    // madd9 - 1, m3, P5, M9
    m.insert("madd9", vec![(0,1), (3,3), (7,5), (14,2)]);
    // madd11 - 1, m3, P5, P11
    m.insert("madd11", vec![(0,1), (3,3), (7,5), (17,4)]);
    // add6 - 1, M3, P5, M6
    m.insert("add6", vec![(0,1), (4,3), (7,5), (9,6)]);
    // madd6 - 1, m3, P5, M6
    m.insert("madd6", vec![(0,1), (3,3), (7,5), (9,6)]);

    // 9th chords - 7th chord + degree 2 (M9)
    // 9 - 1, M3, P5, m7, M9
    m.insert("9", vec![(0,1), (4,3), (7,5), (10,7), (14,2)]);
    // maj9 - 1, M3, P5, M7, M9
    m.insert("maj9", vec![(0,1), (4,3), (7,5), (11,7), (14,2)]);
    m.insert("M9", vec![(0,1), (4,3), (7,5), (11,7), (14,2)]);
    m.insert("Maj9", vec![(0,1), (4,3), (7,5), (11,7), (14,2)]);
    // m9 - 1, m3, P5, m7, M9
    m.insert("m9", vec![(0,1), (3,3), (7,5), (10,7), (14,2)]);
    m.insert("min9", vec![(0,1), (3,3), (7,5), (10,7), (14,2)]);
    // 9b5 - 1, M3, d5, m7, M9
    m.insert("9b5", vec![(0,1), (4,3), (6,5), (10,7), (14,2)]);
    // 9#5 - 1, M3, A5, m7, M9
    m.insert("9#5", vec![(0,1), (4,3), (8,5), (10,7), (14,2)]);
    // m9b5 - 1, m3, d5, m7, M9
    m.insert("m9b5", vec![(0,1), (3,3), (6,5), (10,7), (14,2)]);
    // 9#11 - 1, M3, P5, m7, M9, #11
    m.insert("9#11", vec![(0,1), (4,3), (7,5), (10,7), (14,2), (18,4)]);
    // maj9#11 - 1, M3, P5, M7, M9, #11
    m.insert("maj9#11", vec![(0,1), (4,3), (7,5), (11,7), (14,2), (18,4)]);

    // 11th chords - 7th chord + degree 2 (M9) + degree 4 (P11)
    // 11 - 1, M3, P5, m7, M9, P11
    m.insert("11", vec![(0,1), (4,3), (7,5), (10,7), (14,2), (17,4)]);
    // maj11 - 1, M3, P5, M7, M9, P11
    m.insert("maj11", vec![(0,1), (4,3), (7,5), (11,7), (14,2), (17,4)]);
    // m11 - 1, m3, P5, m7, M9, P11
    m.insert("m11", vec![(0,1), (3,3), (7,5), (10,7), (14,2), (17,4)]);
    m.insert("min11", vec![(0,1), (3,3), (7,5), (10,7), (14,2), (17,4)]);
    // #11 - 1, M3, P5, m7, #11
    m.insert("#11", vec![(0,1), (4,3), (7,5), (10,7), (18,4)]);

    // 13th chords - 7th chord + degree 2 (M9) + degree 6 (M13)
    // 13 - 1, M3, P5, m7, M9, M13
    m.insert("13", vec![(0,1), (4,3), (7,5), (10,7), (14,2), (21,6)]);
    // maj13 - 1, M3, P5, M7, M9, M13
    m.insert("maj13", vec![(0,1), (4,3), (7,5), (11,7), (14,2), (21,6)]);
    // m13 - 1, m3, P5, m7, M9, M13
    m.insert("m13", vec![(0,1), (3,3), (7,5), (10,7), (14,2), (21,6)]);
    m.insert("min13", vec![(0,1), (3,3), (7,5), (10,7), (14,2), (21,6)]);

    // Power chords - 1, P5
    m.insert("5", vec![(0,1), (7,5)]);

    m
});

/// Legacy interval map (semitones only) - kept for backward compatibility
/// New code should use CHORD_INTERVAL_SPECS instead
pub static CHORD_INTERVALS: Lazy<HashMap<&'static str, Vec<u8>>> = Lazy::new(|| {
    CHORD_INTERVAL_SPECS.iter().map(|(k, v)| {
        (*k, v.iter().map(|(semitones, _degree)| *semitones).collect())
    }).collect()
});

/// Default chord intervals (major triad)
pub const DEFAULT_CHORD_INTERVALS: &[u8] = &[0, 4, 7];
pub const DEFAULT_CHORD_INTERVAL_SPECS: &[IntervalSpec] = &[(0,1), (4,3), (7,5)];

/// Parse chord suffix to get interval specifications with explicit scale degrees
pub fn parse_chord_with_interval_specs(suffix: &str) -> MusicResult<Vec<IntervalSpec>> {
    // Remove common separators
    let cleaned = suffix.replace(['-', '_', ' '], "");
    
    // Handle common variations
    let normalized = match cleaned.to_lowercase().as_str() {
        "maj" | "major" => "M",
        "minor" => "m",
        "diminished" => "dim",
        "augmented" => "aug",
        "suspended" => "sus4",
        _ => cleaned.as_str(),
    };
    
    // Try exact match first
    if let Some(specs) = CHORD_INTERVAL_SPECS.get(normalized) {
        return Ok(specs.clone());
    }
    
    // Try with original case (for things like "M7" vs "m7")
    if let Some(specs) = CHORD_INTERVAL_SPECS.get(&cleaned.as_str()) {
        return Ok(specs.clone());
    }
    
    // Return default major triad if not found
    Ok(DEFAULT_CHORD_INTERVAL_SPECS.to_vec())
}

/// Legacy function - Parse chord with intervals to get interval pattern (semitones only)
/// Kept for backward compatibility. New code should use parse_chord_with_interval_specs()
pub fn parse_chord_with_intervals(suffix: &str) -> MusicResult<Vec<u8>> {
    let specs = parse_chord_with_interval_specs(suffix)?;
    Ok(specs.iter().map(|(semitones, _degree)| *semitones).collect())
}

/// Map semitone intervals to diatonic scale degrees (1-7)
/// This determines which letter name to use for spelling
/// 
/// Examples:
/// - 0 semitones → 1st (unison/root)
/// - 3 semitones → 3rd (minor 3rd)
/// - 6 semitones → 4th (tritone, spelled as #4 per music theory)
/// - 10 semitones → 7th (minor 7th)
fn interval_to_scale_degree(semitones: u8) -> u8 {
    let normalized = semitones % 12;
    match normalized {
        0 => 1,        // Unison
        1 | 2 => 2,    // Minor/Major 2nd
        3 | 4 => 3,    // Minor/Major 3rd
        5 | 6 => 4,    // Perfect 4th / Augmented 4th (tritone)
        7 | 8 => 5,    // Perfect 5th / Augmented 5th / Minor 6th
        9 => 6,        // Major 6th
        10 | 11 => 7,  // Minor/Major 7th
        _ => 1,
    }
}

/// Get the letter name at a given scale degree from a root letter
/// 
/// Examples:
/// - get_letter_at_degree('F', 1) → 'F'
/// - get_letter_at_degree('F', 3) → 'A' (F→G→A)
/// - get_letter_at_degree('F', 5) → 'C' (F→G→A→B→C)
/// - get_letter_at_degree('F', 7) → 'E' (F→G→A→B→C→D→E)
fn get_letter_at_degree(root_letter: char, degree: u8) -> char {
    const LETTERS: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
    
    // Find index of root letter
    let root_idx = LETTERS.iter()
        .position(|&l| l == root_letter)
        .expect("Invalid root letter");
    
    // Calculate target index (degree - 1 because degree is 1-indexed)
    let target_idx = (root_idx + (degree as usize - 1)) % 7;
    
    LETTERS[target_idx]
}

/// Spell an interval from a root note using explicit scale degree
/// 
/// This is the core spelling function that uses both semitone distance AND
/// the intended scale degree to produce correct musical spelling.
/// 
/// Examples:
/// - spell_interval_with_degree("F", 3, 3) → "Ab" (minor 3rd: F to A-flat)
/// - spell_interval_with_degree("F", 10, 7) → "Eb" (minor 7th: F to E-flat)
/// - spell_interval_with_degree("C", 6, 5) → "Gb" (diminished 5th: C to G-flat)
/// - spell_interval_with_degree("C", 9, 7) → "Bbb" (diminished 7th: C to B-double-flat)
pub fn spell_interval_with_degree(
    root: &str,
    interval_semitones: u8,
    scale_degree: u8
) -> MusicResult<String> {
    use super::notes::note_index;
    
    // Get root letter (first character)
    let root_letter = root.chars().next()
        .ok_or_else(|| MusicError::ParseError("Empty root".to_string()))?;
    
    // Get target letter at the specified degree
    let target_letter = get_letter_at_degree(root_letter, scale_degree);
    
    // Calculate target pitch (absolute semitone)
    let root_pitch = note_index(root)?;
    let target_pitch = (root_pitch + interval_semitones) % 12;
    
    // Calculate target letter's natural pitch
    let target_letter_str = target_letter.to_string();
    let target_natural_pitch = note_index(&target_letter_str)?;
    
    // Determine accidental needed (difference between target and natural)
    let diff = (target_pitch + 12 - target_natural_pitch) as i8 % 12;
    let accidental = if diff > 6 { diff - 12 } else { diff };
    
    // Format result with appropriate accidental
    let result = match accidental {
        0 => target_letter_str,
        1 => format!("{}#", target_letter),
        -1 => format!("{}b", target_letter),
        2 => format!("{}##", target_letter),
        -2 => format!("{}bb", target_letter),
        _ => return Err(MusicError::ParseError(
            format!("Invalid accidental adjustment: {}", accidental)
        )),
    };
    
    Ok(result)
}

/// Legacy spelling function - kept for backward compatibility
/// New code should use spell_interval_with_degree() or intervals_to_notes()
#[allow(dead_code)]
pub fn spell_interval_diatonically(
    root: &str,
    interval_semitones: u8
) -> MusicResult<String> {
    // Infer degree from semitones (old buggy behavior)
    let degree = interval_to_scale_degree(interval_semitones);
    spell_interval_with_degree(root, interval_semitones, degree)
}

/// Convert interval specifications to absolute note names given a root
/// Uses explicit scale degrees for correct musical spelling
pub fn interval_specs_to_notes(root: &str, specs: &[IntervalSpec]) -> MusicResult<Vec<String>> {
    specs
        .iter()
        .map(|(semitones, degree)| spell_interval_with_degree(root, *semitones, *degree))
        .collect()
}

/// Legacy function - Convert intervals to absolute note names given a root
/// Kept for backward compatibility. New code should use interval_specs_to_notes()
pub fn intervals_to_notes(root: &str, intervals: &[u8]) -> MusicResult<Vec<String>> {
    intervals
        .iter()
        .map(|interval| spell_interval_diatonically(root, *interval))
        .collect()
}

/// High-level API: convert chord name to array of note names with correct spelling
/// Uses explicit scale degrees for proper enharmonic spelling
/// Example: "Cmaj7" → ["C", "E", "G", "B"]
/// Example: "Fm7" → ["F", "Ab", "C", "Eb"]
/// Example: "Cdim7" → ["C", "Eb", "Gb", "Bbb"]
/// Example: "C/E" → ["E", "C", "E", "G"] (bass note prepended)
pub fn chord_to_notes(chord: &str) -> MusicResult<Vec<String>> {
    use super::chords::parse_chord;

    let parsed = parse_chord(chord)?;
    if parsed.root.is_empty() {
        return Err(MusicError::ParseError("Empty chord".to_string()));
    }

    // Get interval specifications with explicit degrees
    let specs = parse_chord_with_interval_specs(&parsed.suffix)?;

    // Convert interval specs to note names with correct spelling
    let mut notes = interval_specs_to_notes(&parsed.root, &specs)?;

    // Prepend bass note for slash chords (e.g., "C/E" puts E first)
    if let Some(bass) = parsed.bass {
        notes.insert(0, bass);
    }

    Ok(notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chord_intervals_lookup() {
        assert_eq!(CHORD_INTERVALS.get("").unwrap(), &vec![0, 4, 7]);
        assert_eq!(CHORD_INTERVALS.get("m").unwrap(), &vec![0, 3, 7]);
        assert_eq!(CHORD_INTERVALS.get("7").unwrap(), &vec![0, 4, 7, 10]);
    }

    // ========================================================================
    // Diatonic Spelling Tests
    // ========================================================================

    #[test]
    fn test_interval_to_scale_degree() {
        assert_eq!(interval_to_scale_degree(0), 1);   // Unison
        assert_eq!(interval_to_scale_degree(1), 2);   // Minor 2nd
        assert_eq!(interval_to_scale_degree(2), 2);   // Major 2nd
        assert_eq!(interval_to_scale_degree(3), 3);   // Minor 3rd
        assert_eq!(interval_to_scale_degree(4), 3);   // Major 3rd
        assert_eq!(interval_to_scale_degree(5), 4);   // Perfect 4th
        assert_eq!(interval_to_scale_degree(6), 4);   // Tritone (#4)
        assert_eq!(interval_to_scale_degree(7), 5);   // Perfect 5th
        assert_eq!(interval_to_scale_degree(8), 5);   // Aug 5th / Min 6th
        assert_eq!(interval_to_scale_degree(9), 6);   // Major 6th
        assert_eq!(interval_to_scale_degree(10), 7);  // Minor 7th
        assert_eq!(interval_to_scale_degree(11), 7);  // Major 7th
    }

    #[test]
    fn test_get_letter_at_degree() {
        // From F
        assert_eq!(get_letter_at_degree('F', 1), 'F');
        assert_eq!(get_letter_at_degree('F', 2), 'G');
        assert_eq!(get_letter_at_degree('F', 3), 'A');
        assert_eq!(get_letter_at_degree('F', 4), 'B');
        assert_eq!(get_letter_at_degree('F', 5), 'C');
        assert_eq!(get_letter_at_degree('F', 6), 'D');
        assert_eq!(get_letter_at_degree('F', 7), 'E');

        // From C
        assert_eq!(get_letter_at_degree('C', 3), 'E');
        assert_eq!(get_letter_at_degree('C', 5), 'G');
        
        // From Bb (uses 'B' letter)
        assert_eq!(get_letter_at_degree('B', 3), 'D');
    }

    #[test]
    fn test_spell_interval_diatonically_basic() {
        // Perfect intervals from C
        assert_eq!(spell_interval_diatonically("C", 0).unwrap(), "C");
        assert_eq!(spell_interval_diatonically("C", 7).unwrap(), "G");
        
        // Major 3rd from C
        assert_eq!(spell_interval_diatonically("C", 4).unwrap(), "E");
        
        // Minor 3rd from C (needs flat)
        assert_eq!(spell_interval_diatonically("C", 3).unwrap(), "Eb");
    }

    #[test]
    fn test_fm7_spelling_critical() {
        // THE CRITICAL TEST CASE
        let fm7_notes = chord_to_notes("Fm7").unwrap();
        assert_eq!(fm7_notes, vec!["F", "Ab", "C", "Eb"]);
        
        // Verify it's NOT using sharps
        assert_ne!(fm7_notes[1], "G#");
        assert_ne!(fm7_notes[3], "D#");
    }

    #[test]
    fn test_minor_chord_spelling() {
        // F minor triad
        assert_eq!(chord_to_notes("Fm").unwrap(), vec!["F", "Ab", "C"]);
        
        // C minor
        assert_eq!(chord_to_notes("Cm").unwrap(), vec!["C", "Eb", "G"]);
        
        // Bb minor
        assert_eq!(chord_to_notes("Bbm").unwrap(), vec!["Bb", "Db", "F"]);
    }

    #[test]
    fn test_seventh_chords() {
        // Dominant 7ths
        assert_eq!(chord_to_notes("C7").unwrap(), vec!["C", "E", "G", "Bb"]);
        assert_eq!(chord_to_notes("F7").unwrap(), vec!["F", "A", "C", "Eb"]);
        
        // Major 7ths
        assert_eq!(chord_to_notes("Cmaj7").unwrap(), vec!["C", "E", "G", "B"]);
        assert_eq!(chord_to_notes("Fmaj7").unwrap(), vec!["F", "A", "C", "E"]);
        
        // Minor 7ths
        assert_eq!(chord_to_notes("Cm7").unwrap(), vec!["C", "Eb", "G", "Bb"]);
    }

    #[test]
    fn test_augmented_with_flat_roots() {
        // User spec: augmented 5th with flat root should naturalize, not sharp
        
        // Db augmented: Db, F, A (NOT Ab#)
        let dbaug = chord_to_notes("Dbaug").unwrap();
        assert_eq!(dbaug, vec!["Db", "F", "A"]);
        assert_eq!(dbaug[2], "A");  // Naturalized, not Ab#
        
        // Eb augmented: Eb, G, B (NOT Bb#)
        let ebaug = chord_to_notes("Ebaug").unwrap();
        assert_eq!(ebaug, vec!["Eb", "G", "B"]);
        assert_eq!(ebaug[2], "B");  // Naturalized
        
        // Ab augmented
        let abaug = chord_to_notes("Abaug").unwrap();
        assert_eq!(abaug, vec!["Ab", "C", "E"]);
        assert_eq!(abaug[2], "E");
    }

    #[test]
    fn test_tritone_spelling() {
        // Tritone (6 semitones) should spell as #4 (augmented 4th)
        assert_eq!(spell_interval_diatonically("C", 6).unwrap(), "F#");
        assert_eq!(spell_interval_diatonically("F", 6).unwrap(), "B");
        assert_eq!(spell_interval_diatonically("G", 6).unwrap(), "C#");
    }

    #[test]
    fn test_diminished_chords() {
        // Diminished triad
        assert_eq!(chord_to_notes("Cdim").unwrap(), vec!["C", "Eb", "Gb"]);
        assert_eq!(chord_to_notes("Fdim").unwrap(), vec!["F", "Ab", "Cb"]);
        
        // Half-diminished 7th (m7b5)
        assert_eq!(chord_to_notes("Cm7b5").unwrap(), vec!["C", "Eb", "Gb", "Bb"]);
    }

    #[test]
    fn test_diminished_7th() {
        // Diminished 7th has double-flat for the 7th degree
        let cdim7 = chord_to_notes("Cdim7").unwrap();
        assert_eq!(cdim7, vec!["C", "Eb", "Gb", "Bbb"]);
        
        // The 7th note should be Bbb (double flat), not A
        assert_eq!(cdim7[3], "Bbb");
    }

    #[test]
    fn test_altered_chords() {
        // 7b9 chord
        let c7b9 = chord_to_notes("C7b9").unwrap();
        assert_eq!(c7b9, vec!["C", "E", "G", "Bb", "Db"]);
        assert_eq!(c7b9[4], "Db");  // Flat 9th, not C#
        
        // 7#9 chord
        let c7sharp9 = chord_to_notes("C7#9").unwrap();
        assert_eq!(c7sharp9, vec!["C", "E", "G", "Bb", "D#"]);
        assert_eq!(c7sharp9[4], "D#");  // Sharp 9th
    }

    #[test]
    fn test_suspended_chords() {
        assert_eq!(chord_to_notes("Csus2").unwrap(), vec!["C", "D", "G"]);
        assert_eq!(chord_to_notes("Csus4").unwrap(), vec!["C", "F", "G"]);
        assert_eq!(chord_to_notes("C7sus4").unwrap(), vec!["C", "F", "G", "Bb"]);
    }

    #[test]
    fn test_extended_chords() {
        // 9th chords
        assert_eq!(chord_to_notes("C9").unwrap(), vec!["C", "E", "G", "Bb", "D"]);
        assert_eq!(chord_to_notes("Cmaj9").unwrap(), vec!["C", "E", "G", "B", "D"]);
        assert_eq!(chord_to_notes("Cm9").unwrap(), vec!["C", "Eb", "G", "Bb", "D"]);
    }

    #[test]
    fn test_complex_roots() {
        // Sharp roots
        let fsharp_maj = chord_to_notes("F#").unwrap();
        assert_eq!(fsharp_maj, vec!["F#", "A#", "C#"]);
        
        // Flat roots  
        let ab_maj = chord_to_notes("Ab").unwrap();
        assert_eq!(ab_maj, vec!["Ab", "C", "Eb"]);
        
        let db_maj = chord_to_notes("Db").unwrap();
        assert_eq!(db_maj, vec!["Db", "F", "Ab"]);
    }

    #[test]
    fn test_chord_to_notes() {
        assert_eq!(chord_to_notes("C").unwrap(), vec!["C", "E", "G"]);
        assert_eq!(chord_to_notes("Am").unwrap(), vec!["A", "C", "E"]);
        assert_eq!(chord_to_notes("Dm7").unwrap(), vec!["D", "F", "A", "C"]);
        assert_eq!(chord_to_notes("G7").unwrap(), vec!["G", "B", "D", "F"]);
        assert_eq!(chord_to_notes("Fmaj7").unwrap(), vec!["F", "A", "C", "E"]);
    }

    #[test]
    fn test_add6_chords() {
        assert_eq!(chord_to_notes("Cadd6").unwrap(), vec!["C", "E", "G", "A"]);
        assert_eq!(chord_to_notes("Dmadd6").unwrap(), vec!["D", "F", "A", "B"]);
    }

    #[test]
    fn test_slash_chords() {
        // Bass note prepended - may duplicate if bass is in chord
        assert_eq!(chord_to_notes("C/E").unwrap(), vec!["E", "C", "E", "G"]);
        assert_eq!(chord_to_notes("Am/G").unwrap(), vec!["G", "A", "C", "E"]);
        // Non-chord-tone bass
        assert_eq!(chord_to_notes("C/B").unwrap(), vec!["B", "C", "E", "G"]);

        // Extended slash chords (7ths, maj7, m7)
        assert_eq!(chord_to_notes("Cmaj7/E").unwrap(), vec!["E", "C", "E", "G", "B"]);
        assert_eq!(chord_to_notes("Dm7/A").unwrap(), vec!["A", "D", "F", "A", "C"]);
        assert_eq!(chord_to_notes("G7/F").unwrap(), vec!["F", "G", "B", "D", "F"]);
    }

    #[test]
    fn test_sixth_chord_variants() {
        assert_eq!(chord_to_notes("C6sus2").unwrap(), vec!["C", "D", "G", "A"]);
        assert_eq!(chord_to_notes("C6sus4").unwrap(), vec!["C", "F", "G", "A"]);
        assert_eq!(chord_to_notes("C6add9").unwrap(), vec!["C", "E", "G", "A", "D"]);
    }

    #[test]
    fn test_7no5_chord() {
        // C7no5: C (root), E (major 3rd), Bb (minor 7th)
        assert_eq!(chord_to_notes("C7no5").unwrap(), vec!["C", "E", "Bb"]);
    }

    #[test]
    fn test_parse_chord_with_intervals() {
        // Test basic chords
        assert_eq!(parse_chord_with_intervals("").unwrap(), vec![0, 4, 7]);
        assert_eq!(parse_chord_with_intervals("m").unwrap(), vec![0, 3, 7]);
        assert_eq!(parse_chord_with_intervals("7").unwrap(), vec![0, 4, 7, 10]);
        
        // Test added chord qualities
        assert_eq!(parse_chord_with_intervals("Maj7").unwrap(), vec![0, 4, 7, 11]);
        assert_eq!(parse_chord_with_intervals("mM7").unwrap(), vec![0, 3, 7, 11]);
        assert_eq!(parse_chord_with_intervals("°7").unwrap(), vec![0, 3, 6, 9]);
        assert_eq!(parse_chord_with_intervals("aug7").unwrap(), vec![0, 4, 8, 10]);
        
        // Test extended chords
        assert_eq!(parse_chord_with_intervals("9").unwrap(), vec![0, 4, 7, 10, 14]);
        assert_eq!(parse_chord_with_intervals("maj9").unwrap(), vec![0, 4, 7, 11, 14]);
        assert_eq!(parse_chord_with_intervals("Maj9").unwrap(), vec![0, 4, 7, 11, 14]);
        
        // Test add chords
        assert_eq!(parse_chord_with_intervals("add9").unwrap(), vec![0, 4, 7, 14]);
        assert_eq!(parse_chord_with_intervals("madd9").unwrap(), vec![0, 3, 7, 14]);
        
        // Test normalization
        assert_eq!(parse_chord_with_intervals("maj").unwrap(), vec![0, 4, 7]);
        assert_eq!(parse_chord_with_intervals("major").unwrap(), vec![0, 4, 7]);
        assert_eq!(parse_chord_with_intervals("minor").unwrap(), vec![0, 3, 7]);
        
        // Test unknown chord falls back to major triad
        assert_eq!(parse_chord_with_intervals("unknown").unwrap(), vec![0, 4, 7]);
    }
}
