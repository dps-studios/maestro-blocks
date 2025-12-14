// Roman numeral conversion system
// Handles conversion between chord names and Roman numeral notation

use super::types::{RomanNumeralParts, Accidental, MusicError, MusicResult};
use super::notes::{note_index, get_preferred_note_name};
use super::chords::transpose_chord;
use std::collections::HashMap;

// Map roman numeral to degree (1-7)
static ROMAN_TO_DEGREE: &[(&str, u8)] = &[
    ("i", 1), ("I", 1),
    ("ii", 2), ("II", 2),
    ("iii", 3), ("III", 3),
    ("iv", 4), ("IV", 4),
    ("v", 5), ("V", 5),
    ("vi", 6), ("VI", 6),
    ("vii", 7), ("VII", 7),
];

// Degree (1-7) to semitone interval in major scale
static DEGREE_SEMITONES: &[u8] = &[0, 0, 2, 4, 5, 7, 9, 11]; // Index 0 unused, 1-7 are degrees

/// Parse a roman numeral string into its components
/// Handles: I, ii, bVII, #iv, V7, vii°7, III/4, Imaj7, etc.
pub fn parse_roman_numeral(numeral: &str) -> MusicResult<RomanNumeralParts> {
    if numeral.is_empty() {
        return Err(MusicError::ParseError("Empty roman numeral".to_string()));
    }

    let trimmed = numeral.trim();
    if trimmed.is_empty() {
        return Err(MusicError::ParseError("Empty roman numeral".to_string()));
    }

    // Handle slash chords - strip off bass notation for parsing
    let (main_part, bass_notation) = if let Some(slash_pos) = trimmed.find('/') {
        let main = &trimmed[..slash_pos];
        let bass = &trimmed[slash_pos + 1..];
        (main, Some(bass.to_string()))
    } else {
        (trimmed, None)
    };

    // Extract accidental prefix (b or #)
    let (accidental, after_accidental) = if main_part.starts_with('b') && main_part.len() > 1 {
        let after_b = &main_part[1..2];
        if after_b == "I" || after_b == "V" || after_b == "i" || after_b == "v" {
            (Some(Accidental::Flat), &main_part[1..])
        } else {
            (None, main_part)
        }
    } else if main_part.starts_with('#') {
        (Some(Accidental::Sharp), &main_part[1..])
    } else {
        (None, main_part)
    };

    // Extract roman numeral base (case-sensitive for major/minor)
    // Order matters: longer/more specific patterns must come first
    let roman_match = after_accidental
        .find(|c: char| c.is_alphabetic())
        .and_then(|_| {
            // Try to match roman numerals in order of length
            ["VII", "VI", "IV", "III", "II", "V", "I", "vii", "vi", "iv", "iii", "ii", "v", "i"]
                .iter()
                .find(|&roman| after_accidental.starts_with(roman))
                .map(|&roman| {
                    let len = roman.len();
                    (roman, &after_accidental[len..])
                })
        });

    let (roman_part, suffix) = roman_match.ok_or_else(|| {
        MusicError::ParseError(format!("No valid roman numeral found in: {}", numeral))
    })?;

    // Normalize roman numeral part for lookup
    let roman_upper = roman_part.to_uppercase();
    let degree = ROMAN_TO_DEGREE
        .iter()
        .find(|(roman, _)| *roman == roman_upper)
        .map(|(_, degree)| *degree)
        .ok_or_else(|| MusicError::ParseError(format!("Invalid roman numeral: {}", roman_part)))?;

    // Check if minor based on case (lowercase = minor)
    let is_minor = roman_part == roman_part.to_lowercase();

    // Normalize special symbols in suffix
    let mut normalized_suffix = suffix.to_string();

    // ø7, ø → m7b5
    if normalized_suffix.starts_with("ø7") || normalized_suffix.starts_with("ø") {
        normalized_suffix = normalized_suffix.replacen("ø", "m7b5", 1);
    }

    // + → aug
    if normalized_suffix.starts_with("+7") {
        normalized_suffix = format!("aug7{}", &normalized_suffix[2..]);
    } else if normalized_suffix.starts_with('+') {
        normalized_suffix = format!("aug{}", &normalized_suffix[1..]);
    }

    // maj → M (e.g., maj7 → M7, maj9 → M9)
    normalized_suffix = normalized_suffix.replace("maj", "M");

    // dim → ° (e.g., dim7 → °7, dim → °)
    normalized_suffix = normalized_suffix.replace("dim", "°");

    // Add bass notation back to suffix if present (for reconstruction)
    let final_suffix = if let Some(ref bass) = bass_notation {
        format!("{}/{}", normalized_suffix, bass)
    } else {
        normalized_suffix
    };

    Ok(RomanNumeralParts {
        degree,
        accidental,
        is_minor,
        suffix: final_suffix,
        bass: bass_notation,
    })
}

/// Get note name for a scale degree in a given key
/// Builds major scale and picks appropriate degree
fn get_scale_degree_note(degree: u8, key: &str, use_flats: bool) -> MusicResult<String> {
    // C major scale notes (with letters): C, D, E, F, G, A, B
    let c_major_scale_notes = ["C", "D", "E", "F", "G", "A", "B"];

    // Transpose each note from C to target key
    let major_scale: Result<Vec<String>, _> = c_major_scale_notes
        .iter()
        .map(|note| transpose_chord(note, "C", key, use_flats))
        .collect();

    let major_scale = major_scale?;

    // Return note for requested degree (1-indexed)
    if degree >= 1 && degree <= 7 {
        Ok(major_scale[(degree - 1) as usize].clone())
    } else {
        Err(MusicError::ParseError(format!("Invalid scale degree: {}", degree)))
    }
}

/// Apply an accidental to a note, preserving letter name
fn apply_accidental_to_note(note: &str, accidental: Option<Accidental>) -> MusicResult<String> {
    let accidental = match accidental {
        Some(acc) => acc,
        None => return Ok(note.to_string()),
    };

    if note.is_empty() {
        return Err(MusicError::ParseError("Empty note".to_string()));
    }

    let letter = note.chars().next().unwrap();
    let existing_accidental: String = note.chars().skip(1).collect();

    let result = match accidental {
        Accidental::Flat => {
            // Flatten note
            if existing_accidental == "#" {
                // F# + flat = F (natural)
                letter.to_string()
            } else if existing_accidental == "bb" {
                // Ebb + flat = Ebbb (triple flat, rare but valid)
                format!("{}bbb", letter)
            } else if existing_accidental == "b" {
                // Eb + flat = Ebb (double flat, rare but valid)
                format!("{}bb", letter)
            } else {
                // E + flat = Eb
                format!("{}b", letter)
            }
        }
        Accidental::Sharp => {
            // Sharpen note
            if existing_accidental == "b" {
                // Bb + sharp = B (natural)
                letter.to_string()
            } else if existing_accidental == "##" {
                // F## + sharp = F### (triple sharp, rare but valid)
                format!("{}###", letter)
            } else if existing_accidental == "#" {
                // F# + sharp = F## (double sharp, rare but valid)
                format!("{}##", letter)
            } else {
                // F + sharp = F#
                format!("{}#", letter)
            }
        }
        Accidental::DoubleFlat => {
            // Double flat - add another flat
            if existing_accidental == "b" {
                format!("{}bb", letter)
            } else if existing_accidental == "#" {
                format!("{}b", letter)
            } else {
                format!("{}bb", letter)
            }
        }
        Accidental::DoubleSharp => {
            // Double sharp - add another sharp
            if existing_accidental == "#" {
                format!("{}##", letter)
            } else if existing_accidental == "b" {
                format!("{}#", letter)
            } else {
                format!("{}##", letter)
            }
        }
    };

    Ok(result)
}

/// Convert a roman numeral to a chord name in a given key
/// Handles: I, ii, bVII, V7, vii°7, III/4, etc.
pub fn roman_numeral_to_chord(numeral: &str, key: &str, use_flats: bool) -> MusicResult<String> {
    let parts = parse_roman_numeral(numeral)?;

    let RomanNumeralParts {
        degree,
        accidental,
        is_minor,
        suffix,
        bass: _,
    } = parts;

    // Get scale degree note (e.g., III in C = "E", IV in Eb = "Ab")
    let scale_degree_note = get_scale_degree_note(degree, key, use_flats)?;

    // Apply Roman numeral's accidental (b or #) to that note
    let root = apply_accidental_to_note(&scale_degree_note, accidental)?;

    // Handle slash chord notation separately
    let (main_suffix, bass_notation) = if let Some(slash_pos) = suffix.find('/') {
        let main = &suffix[..slash_pos];
        let bass = &suffix[slash_pos + 1..];
        (main, Some(bass))
    } else {
        (&suffix[..], None)
    };

    // Build chord suffix
    let mut chord_suffix = String::new();

    // Add 'm' prefix for minor chords (unless it's dim, aug, °, or m7b5)
    let needs_minor_prefix = is_minor &&
        !main_suffix.contains("dim") &&
        !main_suffix.contains("aug") &&
        !main_suffix.contains("°") &&
        !main_suffix.starts_with("m7b5");

    if needs_minor_prefix {
        chord_suffix.push('m');
    }

    // Add quality suffix
    if !main_suffix.is_empty() {
        // Handle diminished: both "dim" and "°" symbols
        if main_suffix == "dim" || main_suffix.starts_with("dim") ||
           main_suffix == "°" || main_suffix.starts_with("°") {
            // Convert ° to dim for chord name (e.g., vii° → Bdim, vii°7 → Bdim7)
            let normalized = main_suffix.replace("°", "dim");
            chord_suffix = normalized; // dim replaces 'm'
        } else if main_suffix == "aug" || main_suffix.starts_with("aug") {
            chord_suffix = main_suffix.to_string(); // aug replaces 'm'
        } else if main_suffix.starts_with("m7b5") {
            chord_suffix = main_suffix.to_string(); // m7b5 is complete
        } else {
            chord_suffix.push_str(main_suffix);
        }
    }

    let mut chord = format!("{}{}", root, chord_suffix);

    // Convert bass notation (chord tone number) back to actual note
    if let Some(bass_notation) = bass_notation {
        // Map chord tone notation to interval from root
        let tone_to_interval: HashMap<&str, u8> = [
            ("1", 0),
            ("2", 2),
            ("b3", 3),
            ("3", if is_minor { 3 } else { 4 }), // Minor 3rd for minor chords, major 3rd otherwise
            ("4", 5),
            ("b5", 6),
            ("5", 7),
            ("#5", 8),
            ("6", 9),
            ("b7", 10),
            ("7", if is_minor { 10 } else { 11 }), // Minor 7th for dominant, major 7th otherwise
            ("9", 2),  // 9th = 2nd
            ("11", 5), // 11th = 4th
            ("13", 9), // 13th = 6th
        ].iter().cloned().collect();

        if let Some(&bass_interval) = tone_to_interval.get(bass_notation) {
            // Calculate root note's semitone index for bass note calculation
            let root_idx = note_index(&root)?;
            let bass_note_idx = (root_idx + bass_interval) % 12;
            let bass_note = get_preferred_note_name(bass_note_idx, key, use_flats);
            chord = format!("{}/{}", chord, bass_note);
        } else {
            // If we can't parse bass notation, just append it as-is
            chord = format!("{}/{}", chord, bass_notation);
        }
    }

    Ok(chord)
}

/// Get the Roman numeral for a chord in a given key
/// Uses letter-based scale degrees to preserve chord spelling:
/// F#m in C → #iv (F is 4th degree, sharp applied)
/// Gbm in C → bv (G is 5th degree, flat applied)
pub fn get_chord_numeral(chord: &str, key: &str) -> MusicResult<String> {
    use super::chords::parse_chord;

    if chord.is_empty() {
        return Err(MusicError::ParseError("Chord cannot be empty".to_string()));
    }

    // Handle slash chords - parse bass note separately
    let (main_chord, bass_note) = if let Some(slash_pos) = chord.find('/') {
        let main = &chord[..slash_pos];
        let bass = &chord[slash_pos + 1..];
        (main, Some(bass.to_string()))
    } else {
        (chord, None)
    };

    // Parse the chord to get root, suffix, and is_minor
    let parsed_chord = parse_chord(main_chord)?;

    if parsed_chord.root.is_empty() {
        return Err(MusicError::ParseError("Invalid chord format".to_string()));
    }

    // Extract base letter and accidental from chord root
    let root = &parsed_chord.root;
    let chord_letter = root.chars().next().unwrap().to_ascii_uppercase();
    let chord_accidental = if root.contains('#') {
        "#"
    } else if root.contains('b') {
        "b"
    } else {
        ""
    };

    // Extract base letter from key
    let key_letter = key.chars().next()
        .ok_or_else(|| MusicError::ParseError("Key cannot be empty".to_string()))?
        .to_ascii_uppercase();

    // Map letters to scale degree indices (0-6)
    fn letter_to_degree(letter: char) -> Option<usize> {
        match letter {
            'C' => Some(0),
            'D' => Some(1),
            'E' => Some(2),
            'F' => Some(3),
            'G' => Some(4),
            'A' => Some(5),
            'B' => Some(6),
            _ => None,
        }
    }

    let chord_degree = letter_to_degree(chord_letter)
        .ok_or_else(|| MusicError::ParseError(format!("Invalid note letter: {}", chord_letter)))?;
    let key_degree = letter_to_degree(key_letter)
        .ok_or_else(|| MusicError::ParseError(format!("Invalid key letter: {}", key_letter)))?;

    // Calculate scale degree (1-7) relative to key
    // If key is C (degree 0), then F (degree 3) is the 4th scale degree
    // If key is G (degree 4), then C (degree 0) is the 4th scale degree
    let relative_degree = ((chord_degree as i32 - key_degree as i32).rem_euclid(7)) as usize;

    // Calculate expected semitone for this scale degree to determine display accidental
    // DEGREE_SEMITONES is 1-indexed, relative_degree is 0-indexed
    let degree_semitone_offset = DEGREE_SEMITONES[relative_degree + 1];
    let chord_semitone = note_index(&parsed_chord.root)?;
    let key_semitone = note_index(key)?;
    let expected_semitone = (key_semitone + degree_semitone_offset) % 12;

    // Determine accidental for Roman numeral display
    // Only show accidental if chord deviates from expected semitone for this degree
    let display_accidental = {
        let diff = (chord_semitone as i32 - expected_semitone as i32).rem_euclid(12);
        match diff {
            0 => "",     // Matches expected - no accidental
            11 => "b",   // One semitone below (11 = -1 mod 12)
            1 => "#",    // One semitone above
            _ => chord_accidental,  // Larger deviation - preserve chord spelling
        }
    };

    // Roman numeral bases (0-indexed: I, II, III, IV, V, VI, VII)
    const MAJOR_BASES: &[&str] = &["I", "II", "III", "IV", "V", "VI", "VII"];
    const MINOR_BASES: &[&str] = &["i", "ii", "iii", "iv", "v", "vi", "vii"];

    // Determine if chord is minor (m, min, but not maj)
    let is_minor = parsed_chord.suffix.starts_with('m') && !parsed_chord.suffix.starts_with("maj");

    // Build numeral: accidental + base
    let base = if is_minor {
        MINOR_BASES[relative_degree]
    } else {
        MAJOR_BASES[relative_degree]
    };

    let mut numeral = format!("{}{}", display_accidental, base);
    
    // Handle chord suffix (quality)
    let mut suffix = parsed_chord.suffix.clone();
    
    // Remove 'm' prefix for minor chords since it's indicated by lowercase
    if is_minor && suffix.starts_with('m') && !suffix.starts_with("maj") && !suffix.starts_with("m7b5") {
        suffix = suffix.chars().skip(1).collect();
    }
    
    // Add quality indicators for extended chords
    if suffix.contains("7") && !suffix.contains("maj7") {
        numeral.push('7');
    } else if suffix.contains("maj7") {
        numeral.push_str("maj7");
    } else if suffix.contains("dim") {
        // Convert to lowercase and add degree symbol
        numeral = numeral.to_lowercase();
        numeral.push('°');
    } else if suffix.contains("aug") {
        numeral.push('+');
    } else if suffix.contains("sus") {
        numeral.push_str("sus");
    } else if !suffix.is_empty() {
        numeral.push_str(&suffix);
    }
    
    // Add bass note if present
    if let Some(ref bass) = bass_note {
        numeral.push('/');
        numeral.push_str(bass);
    }
    
    Ok(numeral)
}

/// Semitone-based Roman numeral for database lookups
/// Matches TypeScript's chordToRoman() in convert-to-roman.ts
/// Both F#m and Gbm → #iv (semitone 6 maps to #IV)
///
/// This is necessary because the database was generated using semitone-based
/// Roman numerals, so lookups must use the same logic.
#[allow(dead_code)]
pub fn get_chord_numeral_for_lookup(chord: &str, key: &str) -> MusicResult<String> {
    use super::chords::parse_chord;

    // Semitone-based lookup tables (match TypeScript's MAJOR_NUMERALS/MINOR_NUMERALS)
    const LOOKUP_MAJOR: &[&str] = &[
        "I", "bII", "II", "bIII", "III", "IV", "#IV", "V", "bVI", "VI", "bVII", "VII"
    ];
    const LOOKUP_MINOR: &[&str] = &[
        "i", "bii", "ii", "biii", "iii", "iv", "#iv", "v", "bvi", "vi", "bvii", "vii"
    ];

    if chord.is_empty() {
        return Err(MusicError::ParseError("Chord cannot be empty".to_string()));
    }

    // Handle slash chords
    let (main_chord, bass_note) = if let Some(slash_pos) = chord.find('/') {
        let main = &chord[..slash_pos];
        let bass = &chord[slash_pos + 1..];
        (main, Some(bass.to_string()))
    } else {
        (chord, None)
    };

    let parsed = parse_chord(main_chord)?;
    if parsed.root.is_empty() {
        return Err(MusicError::ParseError("Invalid chord format".to_string()));
    }

    let root_idx = note_index(&parsed.root)?;
    let key_idx = note_index(key)?;
    let interval = ((root_idx as i32 - key_idx as i32).rem_euclid(12)) as usize;

    let suffix = &parsed.suffix;
    let is_minor = suffix.starts_with('m') && !suffix.starts_with("maj");
    let is_dim = suffix.to_lowercase().contains("dim") || suffix.contains('°');
    let is_aug = suffix.to_lowercase().contains("aug") || suffix.contains('+');

    // Get base numeral based on quality (matches TypeScript logic)
    let base = if is_dim {
        // Diminished uses lowercase
        LOOKUP_MINOR[interval]
    } else if is_aug {
        // Augmented uses uppercase
        LOOKUP_MAJOR[interval]
    } else if is_minor {
        LOOKUP_MINOR[interval]
    } else {
        LOOKUP_MAJOR[interval]
    };

    let mut numeral = base.to_string();

    // Format suffix to match TypeScript's formatSuffixForRoman()
    let formatted_suffix = format_suffix_for_lookup(suffix, is_dim, is_aug, is_minor);
    if !formatted_suffix.is_empty() {
        numeral.push_str(&formatted_suffix);
    }

    // Handle slash chord bass note
    if let Some(ref bass) = bass_note {
        // Convert bass note to chord tone notation (matches TypeScript's getBassChordTone)
        let bass_chord_tone = get_bass_chord_tone_for_lookup(&parsed.root, bass, suffix, is_minor)?;
        numeral.push('/');
        numeral.push_str(&bass_chord_tone);
    }

    Ok(numeral)
}

/// Format suffix for Roman numeral lookup (matches TypeScript's formatSuffixForRoman)
#[allow(dead_code)]
fn format_suffix_for_lookup(suffix: &str, is_dim: bool, is_aug: bool, is_minor: bool) -> String {
    // Handle diminished
    if is_dim {
        if suffix.to_lowercase().contains("dim7") {
            return "°7".to_string();
        } else if suffix.to_lowercase().contains("dim") || suffix.contains('°') {
            return "°".to_string();
        }
    }

    // Handle augmented
    if is_aug {
        if suffix.to_lowercase().contains("aug7") {
            return "+7".to_string();
        } else if suffix.to_lowercase().contains("aug") || suffix.contains('+') {
            return "+".to_string();
        }
    }

    // Handle half-diminished (m7b5)
    if suffix == "m7b5" || suffix == "ø7" || suffix == "ø" {
        return "ø7".to_string();
    }

    // Handle minor-major 7th
    let lower = suffix.to_lowercase();
    if lower == "mm7" || lower == "mmaj7" {
        return "M7".to_string();
    }

    let mut formatted = suffix.to_string();

    // Strip 'm' prefix for minor chords (encoded in numeral case)
    if is_minor && formatted.starts_with('m') && !formatted.starts_with("maj") {
        formatted = formatted.chars().skip(1).collect();
    }

    // Simplify major interval notation
    formatted = formatted.replace("maj7", "M7");
    formatted = formatted.replace("maj9", "M9");
    formatted = formatted.replace("maj11", "M11");
    formatted = formatted.replace("maj13", "M13");

    // Remove standalone 'maj'
    if formatted == "maj" {
        formatted = String::new();
    }

    // Remove parentheses
    formatted = formatted.replace(['(', ')'], "");

    formatted
}

/// Get bass chord tone notation for lookup (matches TypeScript's getBassChordTone)
#[allow(dead_code)]
fn get_bass_chord_tone_for_lookup(root: &str, bass: &str, _suffix: &str, is_minor: bool) -> MusicResult<String> {
    let root_idx = note_index(root)?;
    let bass_idx = note_index(bass)?;
    let interval = ((bass_idx as i32 - root_idx as i32).rem_euclid(12)) as usize;

    // Map interval to chord tone or interval name
    let tone = match interval {
        0 => "1",
        1 => "b2",
        2 => "2",
        3 => if is_minor { "3" } else { "b3" },
        4 => if is_minor { "#3" } else { "3" },
        5 => "4",
        6 => "b5",
        7 => "5",
        8 => "#5",
        9 => "6",
        10 => "b7",
        11 => "7",
        _ => "1",
    };

    Ok(tone.to_string())
}

/// Get display-formatted Roman numeral for UI
/// Preserves the chord spelling - F#m → #iv, Gbm → bv
pub fn get_display_numeral(chord: &str, key: &str) -> MusicResult<String> {
    // Use chord as-is - the numeral accidental is derived from the chord's spelling
    let raw_numeral = get_chord_numeral(chord, key)?;

    // Apply display formatting (handles slash chords)
    roman_numeral_for_display(&raw_numeral, chord)
}

/// Convert Roman numeral slash notation to use note names for display
/// Transforms "bVII/5" + "Bb/F" → "bVII/F"
fn roman_numeral_for_display(numeral: &str, chord: &str) -> MusicResult<String> {

    
    // If numeral doesn't have a slash, return as-is
    if !numeral.contains('/') {
        return Ok(numeral.to_string());
    }
    
    // If chord doesn't have a slash, something is wrong - return original
    if !chord.contains('/') {
        return Ok(numeral.to_string());
    }
    
    // Extract the Roman numeral base (before the slash)
    let numeral_base = if let Some(slash_pos) = numeral.find('/') {
        &numeral[..slash_pos]
    } else {
        numeral
    };
    
    // Extract the bass note from the chord (after the slash)
    let bass_note = if let Some(slash_pos) = chord.find('/') {
        &chord[slash_pos + 1..].trim()
    } else {
        chord
    };
    
    // Return Roman numeral with note name
    Ok(format!("{}/{}", numeral_base, bass_note))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_roman_numeral_simple() {
        let result = parse_roman_numeral("I").unwrap();
        assert_eq!(result.degree, 1);
        assert_eq!(result.accidental, None);
        assert!(!result.is_minor);
        assert_eq!(result.suffix, "");
    }

    #[test]
    fn test_parse_roman_numeral_minor() {
        let result = parse_roman_numeral("ii").unwrap();
        assert_eq!(result.degree, 2);
        assert_eq!(result.accidental, None);
        assert!(result.is_minor);
        assert_eq!(result.suffix, "");
    }

    #[test]
    fn test_parse_roman_numeral_with_accidental() {
        let result = parse_roman_numeral("bVII").unwrap();
        assert_eq!(result.degree, 7);
        assert_eq!(result.accidental, Some(Accidental::Flat));
        assert!(!result.is_minor);
        assert_eq!(result.suffix, "");
    }

    #[test]
    fn test_parse_roman_numeral_with_suffix() {
        let result = parse_roman_numeral("V7").unwrap();
        assert_eq!(result.degree, 5);
        assert_eq!(result.accidental, None);
        assert!(!result.is_minor);
        assert_eq!(result.suffix, "7");
    }

    #[test]
    fn test_parse_roman_numeral_slash_chord() {
        let result = parse_roman_numeral("III/4").unwrap();
        assert_eq!(result.degree, 3);
        assert_eq!(result.accidental, None);
        assert!(!result.is_minor);
        assert_eq!(result.suffix, "/4");
    }

    #[test]
    fn test_parse_roman_numeral_complex() {
        let result = parse_roman_numeral("bVII7").unwrap();
        assert_eq!(result.degree, 7);
        assert_eq!(result.accidental, Some(Accidental::Flat));
        assert!(!result.is_minor);
        assert_eq!(result.suffix, "7");
    }

    #[test]
    fn test_roman_numeral_to_chord_simple() {
        let result = roman_numeral_to_chord("I", "C", true).unwrap();
        assert_eq!(result, "C");
    }

    #[test]
    fn test_roman_numeral_to_chord_minor() {
        let result = roman_numeral_to_chord("ii", "C", true).unwrap();
        assert_eq!(result, "Dm");
    }

    #[test]
    fn test_roman_numeral_to_chord_with_accidental() {
        let result = roman_numeral_to_chord("bVII", "C", true).unwrap();
        assert_eq!(result, "Bb");
    }

    #[test]
    fn test_roman_numeral_to_chord_with_suffix() {
        let result = roman_numeral_to_chord("V7", "C", true).unwrap();
        assert_eq!(result, "G7");
    }

    #[test]
    fn test_apply_accidental_to_note() {
        assert_eq!(apply_accidental_to_note("E", Some(Accidental::Flat)).unwrap(), "Eb");
        assert_eq!(apply_accidental_to_note("F", Some(Accidental::Sharp)).unwrap(), "F#");
        assert_eq!(apply_accidental_to_note("Bb", Some(Accidental::Sharp)).unwrap(), "B");
        assert_eq!(apply_accidental_to_note("F#", Some(Accidental::Flat)).unwrap(), "F");
    }

    #[test]
    fn test_get_scale_degree_note() {
        assert_eq!(get_scale_degree_note(1, "C", true).unwrap(), "C");
        assert_eq!(get_scale_degree_note(3, "C", true).unwrap(), "E");
        assert_eq!(get_scale_degree_note(5, "C", true).unwrap(), "G");
        assert_eq!(get_scale_degree_note(1, "G", true).unwrap(), "G");
        assert_eq!(get_scale_degree_note(4, "G", true).unwrap(), "C");
    }
}
#[cfg(test)]
mod numeral_tests {
    use super::*;

    #[test]
    fn test_sharp_fourth_vs_flat_fifth() {
        // F#m in C should be #iv (F is 4th degree, with sharp)
        let result = get_chord_numeral("F#m", "C").unwrap();
        assert_eq!(result, "#iv", "F#m in C should be #iv");

        // Gbm in C should be bv (G is 5th degree, with flat)  
        let result = get_chord_numeral("Gbm", "C").unwrap();
        assert_eq!(result, "bv", "Gbm in C should be bv");
    }

    #[test]
    fn test_natural_degrees() {
        // C in C should be I
        assert_eq!(get_chord_numeral("C", "C").unwrap(), "I");
        // Am in C should be vi
        assert_eq!(get_chord_numeral("Am", "C").unwrap(), "vi");
        // G in C should be V
        assert_eq!(get_chord_numeral("G", "C").unwrap(), "V");
    }

    #[test]
    fn test_flat_key_degrees() {
        // In Gb major, the tonic Gb should be "I" (not "bI")
        assert_eq!(get_chord_numeral("Gb", "Gb").unwrap(), "I", "Gb in key Gb should be I");

        // Ab is the natural II in Gb major
        assert_eq!(get_chord_numeral("Ab", "Gb").unwrap(), "II", "Ab in key Gb should be II");

        // Bb is the natural III in Gb major
        assert_eq!(get_chord_numeral("Bbm", "Gb").unwrap(), "iii", "Bbm in key Gb should be iii");

        // F is the natural VII in Gb major (no flat)
        assert_eq!(get_chord_numeral("F", "Gb").unwrap(), "VII", "F in key Gb should be VII");

        // Db is the natural V in Gb major
        assert_eq!(get_chord_numeral("Db", "Gb").unwrap(), "V", "Db in key Gb should be V");

        // G (natural) in key Gb should be #I (raised tonic)
        assert_eq!(get_chord_numeral("G", "Gb").unwrap(), "#I", "G in key Gb should be #I");
    }

    #[test]
    fn test_lookup_enharmonic_equivalents() {
        // For database lookups, F#m and Gbm should BOTH map to #iv
        // (because database was generated with semitone-based logic)
        let f_sharp = get_chord_numeral_for_lookup("F#m", "C").unwrap();
        let g_flat = get_chord_numeral_for_lookup("Gbm", "C").unwrap();

        assert_eq!(f_sharp, "#iv", "F#m lookup should be #iv");
        assert_eq!(g_flat, "#iv", "Gbm lookup should also be #iv (same semitone)");
        assert_eq!(f_sharp, g_flat, "Enharmonic equivalents should have same lookup key");
    }

    #[test]
    fn test_lookup_basic_numerals() {
        // Basic diatonic chords
        assert_eq!(get_chord_numeral_for_lookup("C", "C").unwrap(), "I");
        assert_eq!(get_chord_numeral_for_lookup("Dm", "C").unwrap(), "ii");
        assert_eq!(get_chord_numeral_for_lookup("Em", "C").unwrap(), "iii");
        assert_eq!(get_chord_numeral_for_lookup("F", "C").unwrap(), "IV");
        assert_eq!(get_chord_numeral_for_lookup("G", "C").unwrap(), "V");
        assert_eq!(get_chord_numeral_for_lookup("Am", "C").unwrap(), "vi");

        // Non-diatonic (chromatic) chords
        assert_eq!(get_chord_numeral_for_lookup("Bb", "C").unwrap(), "bVII");
        assert_eq!(get_chord_numeral_for_lookup("Eb", "C").unwrap(), "bIII");
        assert_eq!(get_chord_numeral_for_lookup("Ab", "C").unwrap(), "bVI");
    }

    #[test]
    fn test_lookup_with_quality() {
        // 7th chords
        assert_eq!(get_chord_numeral_for_lookup("Dm7", "C").unwrap(), "ii7");
        assert_eq!(get_chord_numeral_for_lookup("G7", "C").unwrap(), "V7");

        // Major 7th
        assert_eq!(get_chord_numeral_for_lookup("Cmaj7", "C").unwrap(), "IM7");
        assert_eq!(get_chord_numeral_for_lookup("Fmaj7", "C").unwrap(), "IVM7");
    }
}
