// Include the generated audio samples at compile time
include!(concat!(env!("OUT_DIR"), "/audio_samples.rs"));

/// Get embedded audio sample bytes for a note key (e.g., "C4", "Cs3", "A2")
/// Returns None if the note is out of range
pub fn get_sample(key: &str) -> Option<&'static [u8]> {
    SAMPLES.get(key).copied()
}

/// Convert a note name with sharp notation to sample key format
/// e.g., "C#" -> "Cs", "D" -> "D"
pub fn note_to_sample_key(note: &str, octave: i8) -> String {
    let normalized = note
        .replace('#', "s")
        .replace('b', ""); // Flats need to be converted to their enharmonic sharp

    // Handle enharmonic equivalents for flats
    let key = match note {
        "Db" => "Cs",
        "Eb" => "Ds",
        "Gb" => "Fs",
        "Ab" => "Gs",
        "Bb" => "As",
        _ => &normalized,
    };

    format!("{}{}", key, octave)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_to_sample_key() {
        assert_eq!(note_to_sample_key("C", 4), "C4");
        assert_eq!(note_to_sample_key("C#", 4), "Cs4");
        assert_eq!(note_to_sample_key("Db", 3), "Cs3");
        assert_eq!(note_to_sample_key("F#", 2), "Fs2");
        assert_eq!(note_to_sample_key("Bb", 3), "As3");
    }

    #[test]
    fn test_get_sample_exists() {
        // These should exist after build
        assert!(get_sample("C4").is_some());
        assert!(get_sample("A2").is_some());
        assert!(get_sample("C5").is_some());
    }

    #[test]
    fn test_get_sample_missing() {
        // Out of range
        assert!(get_sample("C6").is_none());
        assert!(get_sample("A0").is_none());
    }
}
