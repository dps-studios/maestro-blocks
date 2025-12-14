// Note and semitone conversion utilities
// This module handles chromatic scale operations, enharmonic equivalents,
// and key signature logic

use super::types::{MusicError, MusicResult};

/// Key signature type (sharp, flat, or neutral)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    Sharp,
    Flat,
    Neutral,
}
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Chromatic scale with sharps
pub const CHROMATIC: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F",
    "F#", "G", "G#", "A", "A#", "B",
];

/// Chromatic scale with flats
pub const CHROMATIC_FLAT: [&str; 12] = [
    "C", "Db", "D", "Eb", "E", "F",
    "Gb", "G", "Ab", "A", "Bb", "B",
];

/// Valid key names with flats
pub const KEYS_FLAT: [&str; 12] = [
    "C", "Db", "D", "Eb", "E", "F",
    "Gb", "G", "Ab", "A", "Bb", "B",
];

/// Map of note names to semitone indices (includes enharmonic equivalents)
pub static NOTE_TO_SEMITONE: Lazy<HashMap<&'static str, u8>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Natural notes and sharps
    m.insert("C", 0);
    m.insert("C#", 1);
    m.insert("Db", 1);
    m.insert("D", 2);
    m.insert("D#", 3);
    m.insert("Eb", 3);
    m.insert("E", 4);
    m.insert("E#", 5);
    m.insert("Fb", 4);
    m.insert("F", 5);
    m.insert("F#", 6);
    m.insert("Gb", 6);
    m.insert("G", 7);
    m.insert("G#", 8);
    m.insert("Ab", 8);
    m.insert("A", 9);
    m.insert("A#", 10);
    m.insert("Bb", 10);
    m.insert("B", 11);
    m.insert("B#", 0);
    m.insert("Cb", 11);

    // Double sharps and flats (for comprehensive support)
    m.insert("C##", 2);
    m.insert("D##", 4);
    m.insert("E##", 6);
    m.insert("F##", 7);
    m.insert("G##", 9);
    m.insert("A##", 11);
    m.insert("B##", 1);

    m.insert("Cbb", 10);
    m.insert("Dbb", 0);
    m.insert("Ebb", 2);
    m.insert("Fbb", 3);
    m.insert("Gbb", 5);
    m.insert("Abb", 7);
    m.insert("Bbb", 9);

    m
});

/// Get semitone index for a note name
pub fn note_index(note: &str) -> MusicResult<u8> {
    NOTE_TO_SEMITONE
        .get(note)
        .copied()
        .ok_or_else(|| MusicError::InvalidKey(note.to_string()))
}

/// Determine if a key uses sharps, flats, or is neutral
pub fn get_key_signature_type(key: &str) -> KeyType {
    // Sharp keys: G, D, A, E, B, F#, C#
    if matches!(key, "G" | "D" | "A" | "E" | "B" | "F#" | "C#" | "G#" | "D#" | "A#") {
        return KeyType::Sharp;
    }

    // Flat keys: F, Bb, Eb, Ab, Db, Gb, Cb
    if matches!(key, "F" | "Bb" | "Eb" | "Ab" | "Db" | "Gb" | "Cb") {
        return KeyType::Flat;
    }

    // C and Am are neutral (can use either)
    KeyType::Neutral
}

/// Get preferred note name based on key signature and user preference
pub fn get_preferred_note_name(
    semitone: u8,
    key: &str,
    use_flats: bool,
) -> &'static str {
    let key_type = get_key_signature_type(key);

    // Determine whether to use sharps or flats
    let prefer_flats = match key_type {
        KeyType::Flat => true,
        KeyType::Sharp => false,
        KeyType::Neutral => use_flats,
    };

    let index = (semitone % 12) as usize;

    if prefer_flats {
        CHROMATIC_FLAT[index]
    } else {
        CHROMATIC[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_index() {
        assert_eq!(note_index("C").unwrap(), 0);
        assert_eq!(note_index("C#").unwrap(), 1);
        assert_eq!(note_index("Db").unwrap(), 1);
        assert_eq!(note_index("B").unwrap(), 11);
    }

    #[test]
    fn test_enharmonic_equivalents() {
        assert_eq!(note_index("C#").unwrap(), note_index("Db").unwrap());
        assert_eq!(note_index("D#").unwrap(), note_index("Eb").unwrap());
        assert_eq!(note_index("E#").unwrap(), note_index("F").unwrap());
        assert_eq!(note_index("B#").unwrap(), note_index("C").unwrap());
    }

    #[test]
    fn test_key_signature_type() {
        assert_eq!(get_key_signature_type("G"), KeyType::Sharp);
        assert_eq!(get_key_signature_type("D"), KeyType::Sharp);
        assert_eq!(get_key_signature_type("F"), KeyType::Flat);
        assert_eq!(get_key_signature_type("Bb"), KeyType::Flat);
        assert_eq!(get_key_signature_type("C"), KeyType::Neutral);
    }

    #[test]
    fn test_preferred_note_name() {
        assert_eq!(get_preferred_note_name(1, "G", false), "C#");
        assert_eq!(get_preferred_note_name(1, "F", false), "Db");
        assert_eq!(get_preferred_note_name(1, "C", true), "Db");
        assert_eq!(get_preferred_note_name(1, "C", false), "C#");
    }
}
