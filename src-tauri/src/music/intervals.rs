// Chord quality to interval mappings and chord decomposition
// This module handles converting chord suffixes to note intervals

use super::types::{MusicError, MusicResult};
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Map of chord suffixes to interval patterns (in semitones from root)
/// This will be populated with 146 chord quality definitions
pub static CHORD_INTERVALS: Lazy<HashMap<&'static str, Vec<u8>>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Major triads
    m.insert("", vec![0, 4, 7]);
    m.insert("M", vec![0, 4, 7]);
    m.insert("maj", vec![0, 4, 7]);
    m.insert("major", vec![0, 4, 7]);

    // Minor triads
    m.insert("m", vec![0, 3, 7]);
    m.insert("min", vec![0, 3, 7]);
    m.insert("minor", vec![0, 3, 7]);
    m.insert("-", vec![0, 3, 7]);

    // Diminished
    m.insert("dim", vec![0, 3, 6]);
    m.insert("°", vec![0, 3, 6]);
    m.insert("o", vec![0, 3, 6]);

    // Augmented
    m.insert("aug", vec![0, 4, 8]);
    m.insert("+", vec![0, 4, 8]);

    // Suspended
    m.insert("sus2", vec![0, 2, 7]);
    m.insert("sus4", vec![0, 5, 7]);
    m.insert("sus", vec![0, 5, 7]);

    // Seventh chords
    m.insert("7", vec![0, 4, 7, 10]);
    m.insert("maj7", vec![0, 4, 7, 11]);
    m.insert("M7", vec![0, 4, 7, 11]);
    m.insert("Maj7", vec![0, 4, 7, 11]);
    m.insert("m7", vec![0, 3, 7, 10]);
    m.insert("min7", vec![0, 3, 7, 10]);
    m.insert("mM7", vec![0, 3, 7, 11]);
    m.insert("mmaj7", vec![0, 3, 7, 11]);
    m.insert("mMaj7", vec![0, 3, 7, 11]);
    m.insert("dim7", vec![0, 3, 6, 9]);
    m.insert("°7", vec![0, 3, 6, 9]);
    m.insert("m7b5", vec![0, 3, 6, 10]);
    m.insert("ø", vec![0, 3, 6, 10]);
    m.insert("ø7", vec![0, 3, 6, 10]);
    m.insert("aug7", vec![0, 4, 8, 10]);
    m.insert("+7", vec![0, 4, 8, 10]);
    m.insert("7no5", vec![0, 4, 10]);

    // Extended chords (9th, 11th, 13th)
    // Altered seventh chords
    m.insert("7b5", vec![0, 4, 6, 10]);
    m.insert("7#5", vec![0, 4, 8, 10]);
    m.insert("7b9", vec![0, 4, 7, 10, 13]);
    m.insert("7#9", vec![0, 4, 7, 10, 15]);
    m.insert("7alt", vec![0, 4, 6, 10, 13]);
    m.insert("7b13", vec![0, 4, 7, 10, 20]);
    m.insert("m7b9", vec![0, 3, 7, 10, 13]);
    m.insert("maj7b5", vec![0, 4, 6, 11]);
    m.insert("maj7#5", vec![0, 4, 8, 11]);
    m.insert("maj7#11", vec![0, 4, 7, 11, 18]);

    // Suspended seventh chords
    m.insert("7sus", vec![0, 5, 7, 10]);
    m.insert("7sus4", vec![0, 5, 7, 10]);
    m.insert("7sus2", vec![0, 2, 7, 10]);
    m.insert("m7sus4", vec![0, 5, 7, 10]);
    m.insert("9sus", vec![0, 5, 7, 10, 14]);
    m.insert("9sus4", vec![0, 5, 7, 10, 14]);
    m.insert("7b9sus4", vec![0, 5, 7, 10, 13]);

    // Sixth chords
    m.insert("6", vec![0, 4, 7, 9]);
    m.insert("m6", vec![0, 3, 7, 9]);
    m.insert("min6", vec![0, 3, 7, 9]);
    m.insert("6/9", vec![0, 4, 7, 9, 14]);
    m.insert("69", vec![0, 4, 7, 9, 14]);
    m.insert("m6/9", vec![0, 3, 7, 9, 14]);
    m.insert("m69", vec![0, 3, 7, 9, 14]);
    m.insert("6sus2", vec![0, 2, 7, 9]);
    m.insert("6sus4", vec![0, 5, 7, 9]);
    m.insert("6add9", vec![0, 4, 7, 9, 14]);

    // Add chords
    m.insert("add2", vec![0, 2, 4, 7]);
    m.insert("add4", vec![0, 4, 5, 7]);
    m.insert("add9", vec![0, 4, 7, 14]);
    m.insert("add11", vec![0, 4, 7, 17]);
    m.insert("add13", vec![0, 4, 7, 21]);
    m.insert("madd2", vec![0, 2, 3, 7]);
    m.insert("madd4", vec![0, 3, 5, 7]);
    m.insert("madd9", vec![0, 3, 7, 14]);
    m.insert("madd11", vec![0, 3, 7, 17]);
    m.insert("add6", vec![0, 4, 7, 9]);
    m.insert("madd6", vec![0, 3, 7, 9]);

    // Extended chords (9th, 11th, 13th)
    m.insert("9", vec![0, 4, 7, 10, 14]);
    m.insert("maj9", vec![0, 4, 7, 11, 14]);
    m.insert("M9", vec![0, 4, 7, 11, 14]);
    m.insert("Maj9", vec![0, 4, 7, 11, 14]);
    m.insert("m9", vec![0, 3, 7, 10, 14]);
    m.insert("min9", vec![0, 3, 7, 10, 14]);
    m.insert("9b5", vec![0, 4, 6, 10, 14]);
    m.insert("9#5", vec![0, 4, 8, 10, 14]);
    m.insert("m9b5", vec![0, 3, 6, 10, 14]);
    m.insert("9#11", vec![0, 4, 7, 10, 14, 18]);
    m.insert("maj9#11", vec![0, 4, 7, 11, 14, 18]);

    // 11th chords
    m.insert("11", vec![0, 4, 7, 10, 14, 17]);
    m.insert("maj11", vec![0, 4, 7, 11, 14, 17]);
    m.insert("m11", vec![0, 3, 7, 10, 14, 17]);
    m.insert("min11", vec![0, 3, 7, 10, 14, 17]);
    m.insert("#11", vec![0, 4, 7, 10, 18]);

    // 13th chords
    m.insert("13", vec![0, 4, 7, 10, 14, 21]);
    m.insert("maj13", vec![0, 4, 7, 11, 14, 21]);
    m.insert("m13", vec![0, 3, 7, 10, 14, 21]);
    m.insert("min13", vec![0, 3, 7, 10, 14, 21]);

    // Power chords
    m.insert("5", vec![0, 7]);

    m
});

/// Default chord intervals (major triad)
pub const DEFAULT_CHORD_INTERVALS: &[u8] = &[0, 4, 7];

/// Parse chord with intervals to get interval pattern
pub fn parse_chord_with_intervals(suffix: &str) -> MusicResult<Vec<u8>> {
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
    if let Some(intervals) = CHORD_INTERVALS.get(normalized) {
        return Ok(intervals.clone());
    }
    
    // Try with original case (for things like "M7" vs "m7")
    if let Some(intervals) = CHORD_INTERVALS.get(&cleaned.as_str()) {
        return Ok(intervals.clone());
    }
    
    // Return default major triad if not found
    Ok(DEFAULT_CHORD_INTERVALS.to_vec())
}

/// Convert intervals to absolute note names given a root
/// Uses sharp notation for output (like TypeScript's SEMITONE_TO_SHARP)
pub fn intervals_to_notes(root: &str, intervals: &[u8]) -> MusicResult<Vec<String>> {
    use super::notes::{note_index, CHROMATIC};

    let root_semitone = note_index(root)?;
    let notes: Vec<String> = intervals
        .iter()
        .map(|interval| {
            let abs_semitone = (root_semitone + *interval as u8) % 12;
            CHROMATIC[abs_semitone as usize].to_string()
        })
        .collect();
    Ok(notes)
}

/// High-level API: convert chord name to array of note names
/// Example: "Cmaj7" → ["C", "E", "G", "B"]
/// Example: "C/E" → ["E", "C", "E", "G"] (bass note prepended)
pub fn chord_to_notes(chord: &str) -> MusicResult<Vec<String>> {
    use super::chords::parse_chord;

    let parsed = parse_chord(chord)?;
    if parsed.root.is_empty() {
        return Err(MusicError::ParseError("Empty chord".to_string()));
    }

    // Get intervals for the suffix (or default major triad)
    let intervals = parse_chord_with_intervals(&parsed.suffix)?;

    // Convert intervals to note names
    let mut notes = intervals_to_notes(&parsed.root, &intervals)?;

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
        // Note: system uses sharps, so Bb is represented as A#
        assert_eq!(chord_to_notes("C7no5").unwrap(), vec!["C", "E", "A#"]);
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
