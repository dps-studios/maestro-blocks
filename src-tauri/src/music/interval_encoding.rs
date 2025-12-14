// Interval-based chord encoding system
// Provides key-agnostic chord recommendations from ANY chord
//
// Format: quality_interval_quality_interval_...
// Example: C Am F G -> "M_3_m_4_M_2_M"
//   - First chord: just quality (M)
//   - Interval 3: semitones from C to A (normalized 0-6)
//   - m: Am quality
//   - Interval 4: semitones from A to F (normalized)
//   - M: F quality
//   - Interval 2: semitones from F to G
//   - M: G quality
//
// Slash chords encode bass as interval from root:
//   C/E -> "M/4" (bass is 4 semitones from root)

use std::collections::HashMap;
use std::sync::LazyLock;

use super::notes::{note_index, get_preferred_note_name};
use super::chords::parse_chord;
use super::types::{MusicError, MusicResult};

/// Static lookup table for suffix normalization (case-insensitive keys)
static SUFFIX_MAPPINGS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        // Typos
        ("dimm", "dim"),
        ("aug5", "aug"),
        // Wrong order
        ("7m", "maj7"),
        // Sus without number
        ("sus", "sus4"),
        ("7sus", "7sus4"),
        ("9sus", "9sus4"),
        // Bare numbers
        ("2", "sus2"),
        ("4", "sus4"),
        // Half-diminished (both hyphen and flat variants, since M7-5 -> M7b5 before lookup)
        ("m7-5", "m7b5"),
        ("m7b5", "m7b5"),
        // O as diminished
        ("o", "dim"),
        ("o7", "dim7"),
        ("om", "dim"),
        // Spanish notation
        ("mi", "m"),
        // Reversed sus notation
        ("sus7", "7sus4"),
        ("sus9", "9sus4"),
        // ma7 shorthand
        ("ma7", "maj7"),
        // Minor-major 7th variants
        ("mm7", "mmaj7"),
        ("mmaj7", "mmaj7"),
        // Minor augmented
        ("maug7", "m7#5"),
        ("maug5", "m#5"),
        ("maug", "m#5"),
        // min variants
        ("min", "m"),
        ("min7", "m7"),
    ])
});

/// Normalize interval to 0-6 range (direction-agnostic)
/// 7 semitones = 5 the other way, 8 = 4, etc.
pub fn normalize_interval(semitones: i32) -> u8 {
    let modulo = ((semitones % 12) + 12) % 12;
    if modulo <= 6 { modulo as u8 } else { (12 - modulo) as u8 }
}

/// Suffix whitelist - matches TypeScript VALID_SUFFIXES
static VALID_SUFFIXES: &[&str] = &[
    // Basic qualities
    "", "m",
    // Diminished/Augmented
    "dim", "dim7", "aug", "aug7",
    // Sevenths
    "7", "maj7", "m7", "mmaj7",
    // Extensions
    "9", "maj9", "m9", "add9", "madd9",
    "11", "maj11", "m11", "add11", "madd11",
    "13", "maj13", "m13", "add13",
    // Suspensions
    "sus2", "sus4", "7sus2", "7sus4", "9sus4",
    // Alterations
    "#5", "b5", "m#5", "m7#5",
    "7#5", "7b5", "7#9", "7b9", "7#11", "7b13", "7alt",
    // Sixths
    "6", "m6", "6/9", "6sus2", "6sus4", "6add9",
    // Power chord
    "5",
    // Added tones
    "add2", "add4", "add6",
    // No-fifth
    "7no5",
];

/// Check if suffix is valid
fn is_valid_suffix(suffix: &str) -> bool {
    VALID_SUFFIXES.contains(&suffix)
}

/// Normalize chord suffix to standard form (matches TypeScript normalizeSuffix)
fn normalize_suffix(suffix: &str) -> String {
    // Step 1: Strip parentheses
    let s = suffix.replace(['(', ')'], "");
    
    // Step 2: Apply hyphen-flat notation replacements
    let s = s.replace("-5", "b5").replace("-9", "b9");
    
    // Step 3: Lookup in mapping table (case-insensitive)
    let lower = s.to_lowercase();
    if let Some(&normalized) = SUFFIX_MAPPINGS.get(lower.as_str()) {
        return normalized.to_string();
    }
    
    // Step 4: Case normalization for known prefixes
    normalize_suffix_case(&s, &lower)
}

/// Normalize case for known prefix patterns (sus, dim, aug, maj, add)
fn normalize_suffix_case(s: &str, lower: &str) -> String {
    const PREFIXES: [&str; 5] = ["sus", "dim", "aug", "maj", "add"];
    
    for prefix in PREFIXES {
        if lower.starts_with(prefix) {
            return format!("{}{}", prefix, &s[prefix.len()..]);
        }
    }
    
    s.to_string()
}

/// Parse a chord into root semitone and quality (with bass interval encoding)
pub fn parse_chord_for_interval(chord: &str) -> MusicResult<(u8, String)> {
    let parsed = parse_chord(chord)?;
    if parsed.root.is_empty() {
        return Err(MusicError::ParseError("Empty chord root".to_string()));
    }

    let root_semitone = note_index(&parsed.root)?;
    let suffix = parsed.suffix.clone();

    // Handle bass note from parsed.bass (parse_chord already extracts it)
    // Bass intervals use full 0-11 semitone range (NOT normalized to 0-6)
    // because bass notes are specific pitch positions, not directional relationships
    let bass_interval: Option<u8> = if let Some(ref bass_note) = parsed.bass {
        match note_index(bass_note) {
            Ok(bass_semitone) => {
                // Use modulo 12 to get semitone distance, but don't normalize to 0-6
                let interval = ((bass_semitone as i32 - root_semitone as i32).rem_euclid(12)) as u8;
                Some(interval)
            }
            Err(_) => return Err(MusicError::ParseError(format!("Invalid bass note: {}", bass_note))),
        }
    } else {
        None
    };

    // Normalize suffix
    let normalized = normalize_suffix(&suffix);

    // Validate suffix
    if !normalized.is_empty() && !is_valid_suffix(&normalized) {
        return Err(MusicError::ParseError(format!("Invalid chord suffix: {}", normalized)));
    }

    // Map empty suffix to "M" for clarity
    let mut quality = if normalized.is_empty() { "M".to_string() } else { normalized };

    // Append bass interval if present
    if let Some(interval) = bass_interval {
        quality = format!("{}/{}", quality, interval);
    }

    Ok((root_semitone, quality))
}

/// Convert chord history to interval encoding key
/// ["C", "Am", "F"] -> "M_3_m_4_M"
pub fn history_to_interval_key(history: &[String]) -> MusicResult<String> {
    if history.is_empty() {
        return Err(MusicError::ParseError("Empty history".to_string()));
    }

    let mut parts: Vec<String> = Vec::new();
    let mut prev_root: Option<u8> = None;

    for chord in history {
        let (root, quality) = parse_chord_for_interval(chord)?;

        if let Some(prev) = prev_root {
            let interval = normalize_interval(root as i32 - prev as i32);
            parts.push(interval.to_string());
        }

        parts.push(quality);
        prev_root = Some(root);
    }

    Ok(parts.join("_"))
}

/// Parse an interval key back into parts
/// "3_m" -> (interval: 3, quality: "m")
#[allow(dead_code)]
pub fn parse_interval_key(key: &str) -> MusicResult<(u8, String)> {
    let parts: Vec<&str> = key.split('_').collect();
    if parts.len() != 2 {
        return Err(MusicError::ParseError(format!("Invalid interval key format: {}", key)));
    }

    let interval: u8 = parts[0].parse()
        .map_err(|_| MusicError::ParseError(format!("Invalid interval: {}", parts[0])))?;
    let quality = parts[1].to_string();

    Ok((interval, quality))
}

/// Parse quality string to extract main quality and optional bass interval
/// "M/4" -> ("M", Some(4))
/// "m7" -> ("m7", None)
pub fn parse_quality_with_bass(quality: &str) -> MusicResult<(String, Option<u8>)> {
    if let Some(slash_pos) = quality.find('/') {
        let main = &quality[..slash_pos];
        let bass_str = &quality[slash_pos + 1..];
        let bass: u8 = bass_str.parse()
            .map_err(|_| MusicError::ParseError(format!("Invalid bass interval: {}", bass_str)))?;
        Ok((main.to_string(), Some(bass)))
    } else {
        Ok((quality.to_string(), None))
    }
}

/// Calculate the absolute semitone of a bass note given root semitone and bass interval
/// root=0 (C), bass_interval=4 -> 4 (E)
/// root=5 (F), bass_interval=4 -> 9 (A)
pub fn calculate_bass_semitone(root_semitone: u8, bass_interval: u8) -> u8 {
    (root_semitone + bass_interval) % 12
}

/// Convert semitone to note name with proper spelling for the key
/// This ensures we get the EXACT note (e.g., F# vs Gb) based on context
pub fn semitone_to_note_in_key(semitone: u8, key: &str, use_flats: bool) -> String {
    get_preferred_note_name(semitone, key, use_flats).to_string()
}

/// Build a slash chord string from components
/// root="F", suffix="", bass_note="A" -> "F/A"
/// root="C", suffix="m7", bass_note="G" -> "Cm7/G"
pub fn build_slash_chord(root: &str, suffix: &str, bass_note: &str) -> String {
    if suffix.is_empty() {
        format!("{}/{}", root, bass_note)
    } else {
        format!("{}{}/{}", root, suffix, bass_note)
    }
}

/// Convert interval recommendation back to absolute chord
///
/// # Arguments
/// * `interval` - Semitone distance from previous chord root (0-6, normalized)
/// * `quality` - Chord quality including optional bass interval (e.g., "m7", "M/4")
/// * `from_root` - Semitone of the previous chord's root (0-11)
/// * `use_flats` - Whether to prefer flat notation
/// * `key` - The musical key for determining note spelling
///
/// # Example
/// ```ignore
/// // From E (semitone 4), interval 5 with quality "m" -> Am
/// interval_to_chord(5, "m", 4, true, "C") // -> "Am"
///
/// // From C (semitone 0), interval 0 with quality "M/4" -> C/E
/// // Bass calculation: C (0) + bass_interval (4) = E (4)
/// interval_to_chord(0, "M/4", 0, true, "C") // -> "C/E"
/// ```
pub fn interval_to_chord(interval: u8, quality: &str, from_root: u8, use_flats: bool, key: &str) -> MusicResult<String> {
    // Step 1: Calculate new chord root semitone
    let new_root_semitone = (from_root + interval) % 12;

    // Step 2: Get note name for the root with proper spelling
    let root_note = semitone_to_note_in_key(new_root_semitone, key, use_flats);

    // Step 3: Parse quality to separate main quality from bass interval
    let (main_quality, bass_interval) = parse_quality_with_bass(quality)?;

    // Step 4: Build chord suffix (M -> empty string for major)
    let suffix = if main_quality == "M" { String::new() } else { main_quality };

    // Step 5: Handle bass note if present
    if let Some(bass_int) = bass_interval {
        // Calculate bass note semitone relative to the NEW chord root
        let bass_semitone = calculate_bass_semitone(new_root_semitone, bass_int);

        // Get the exact bass note name with proper spelling
        let bass_note = semitone_to_note_in_key(bass_semitone, key, use_flats);

        // Build the slash chord
        Ok(build_slash_chord(&root_note, &suffix, &bass_note))
    } else {
        // No bass note - just root + suffix
        Ok(format!("{}{}", root_note, suffix))
    }
}

/// Get a display-friendly representation of an interval recommendation
/// "3_m" -> "3rd minor" or similar
#[allow(dead_code)]
pub fn interval_key_to_display(key: &str) -> String {
    // For now, just return the key as-is
    // Could be enhanced to show "down 3 semitones to minor" etc.
    key.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_interval() {
        assert_eq!(normalize_interval(0), 0);
        assert_eq!(normalize_interval(1), 1);
        assert_eq!(normalize_interval(6), 6);
        assert_eq!(normalize_interval(7), 5);  // 7 -> 12-7 = 5
        assert_eq!(normalize_interval(11), 1); // 11 -> 12-11 = 1
        assert_eq!(normalize_interval(-3), 3); // -3 mod 12 = 9 -> 12-9 = 3
    }

    #[test]
    fn test_parse_chord_for_interval_basic() {
        let (root, quality) = parse_chord_for_interval("C").unwrap();
        assert_eq!(root, 0);
        assert_eq!(quality, "M");

        let (root, quality) = parse_chord_for_interval("Am").unwrap();
        assert_eq!(root, 9);
        assert_eq!(quality, "m");

        let (root, quality) = parse_chord_for_interval("F#m7").unwrap();
        assert_eq!(root, 6);
        assert_eq!(quality, "m7");
    }

    #[test]
    fn test_parse_chord_for_interval_slash() {
        // Bass intervals use full 0-11 range (not normalized to 0-6)
        let (root, quality) = parse_chord_for_interval("C/E").unwrap();
        assert_eq!(root, 0);
        assert_eq!(quality, "M/4"); // E is 4 semitones from C

        let (root, quality) = parse_chord_for_interval("Am/G").unwrap();
        assert_eq!(root, 9);
        assert_eq!(quality, "m/10"); // G is 10 semitones from A (NOT normalized)

        let (root, quality) = parse_chord_for_interval("C/G").unwrap();
        assert_eq!(root, 0);
        assert_eq!(quality, "M/7"); // G is 7 semitones from C (NOT normalized)

        // Verify different bass notes produce different encodings
        let (_, quality_e) = parse_chord_for_interval("C/E").unwrap();
        let (_, quality_ab) = parse_chord_for_interval("C/Ab").unwrap();
        assert_eq!(quality_e, "M/4");   // E = 4 semitones
        assert_eq!(quality_ab, "M/8");  // Ab = 8 semitones
        assert_ne!(quality_e, quality_ab, "C/E and C/Ab should have different encodings");
    }

    #[test]
    fn test_history_to_interval_key() {
        let history = vec!["C".to_string(), "Am".to_string(), "F".to_string()];
        let key = history_to_interval_key(&history).unwrap();
        // C=M, interval C->A = 9 semitones -> normalized 3, Am=m, interval A->F = 8 -> normalized 4, F=M
        assert_eq!(key, "M_3_m_4_M");

        let history = vec!["C".to_string()];
        let key = history_to_interval_key(&history).unwrap();
        assert_eq!(key, "M"); // Just quality, no interval prefix
    }

    #[test]
    fn test_interval_to_chord() {
        // From E (semitone 4), interval 5 with quality "m" -> Am
        let chord = interval_to_chord(5, "m", 4, true, "C").unwrap();
        assert_eq!(chord, "Am");

        // From E (semitone 4), interval 0 with quality "M" -> E
        let chord = interval_to_chord(0, "M", 4, true, "C").unwrap();
        assert_eq!(chord, "E");

        // From C (semitone 0), interval 5 with quality "M" -> F
        let chord = interval_to_chord(5, "M", 0, true, "C").unwrap();
        assert_eq!(chord, "F");
    }

    #[test]
    fn test_interval_to_chord_with_bass() {
        // From C (0), interval 0, quality "M/4" -> C/E
        let chord = interval_to_chord(0, "M/4", 0, true, "C").unwrap();
        assert_eq!(chord, "C/E");
    }

    #[test]
    fn test_normalize_suffix_basic() {
        // Existing cases
        assert_eq!(normalize_suffix("sus"), "sus4");
        assert_eq!(normalize_suffix("7M"), "maj7");
        assert_eq!(normalize_suffix("dimm"), "dim");
        assert_eq!(normalize_suffix("min"), "m");
        assert_eq!(normalize_suffix("min7"), "m7");
    }

    #[test]
    fn test_normalize_suffix_parentheses() {
        assert_eq!(normalize_suffix("(maj7)"), "maj7");
        assert_eq!(normalize_suffix("m(7)"), "m7");
        assert_eq!(normalize_suffix("(sus4)"), "sus4");
    }

    #[test]
    fn test_normalize_suffix_typos() {
        assert_eq!(normalize_suffix("dimm"), "dim");
        assert_eq!(normalize_suffix("aug5"), "aug");
        assert_eq!(normalize_suffix("DIMM"), "dim");
        assert_eq!(normalize_suffix("AUG5"), "aug");
    }

    #[test]
    fn test_normalize_suffix_sus_variants() {
        // Basic sus normalization
        assert_eq!(normalize_suffix("sus"), "sus4");
        assert_eq!(normalize_suffix("SUS"), "sus4");
        assert_eq!(normalize_suffix("Sus"), "sus4");
        
        // 7sus variants
        assert_eq!(normalize_suffix("7sus"), "7sus4");
        assert_eq!(normalize_suffix("7SUS"), "7sus4");
        
        // 9sus variants
        assert_eq!(normalize_suffix("9sus"), "9sus4");
        
        // Reversed notation: sus7/sus9 -> 7sus4/9sus4
        assert_eq!(normalize_suffix("sus7"), "7sus4");
        assert_eq!(normalize_suffix("sus9"), "9sus4");
        
        // Bare numbers -> sus
        assert_eq!(normalize_suffix("2"), "sus2");
        assert_eq!(normalize_suffix("4"), "sus4");
    }

    #[test]
    fn test_normalize_suffix_diminished_variants() {
        // O notation for diminished
        assert_eq!(normalize_suffix("O"), "dim");
        assert_eq!(normalize_suffix("o"), "dim");
        assert_eq!(normalize_suffix("O7"), "dim7");
        assert_eq!(normalize_suffix("o7"), "dim7");
        assert_eq!(normalize_suffix("Om"), "dim");
        assert_eq!(normalize_suffix("om"), "dim");
        
        // Half-diminished
        assert_eq!(normalize_suffix("m7-5"), "m7b5");
        assert_eq!(normalize_suffix("M7-5"), "m7b5");
    }

    #[test]
    fn test_normalize_suffix_hyphen_flat_notation() {
        assert_eq!(normalize_suffix("7-5"), "7b5");
        assert_eq!(normalize_suffix("7-9"), "7b9");
        assert_eq!(normalize_suffix("m7-5"), "m7b5");
    }

    #[test]
    fn test_normalize_suffix_spanish_notation() {
        assert_eq!(normalize_suffix("mi"), "m");
        assert_eq!(normalize_suffix("MI"), "m");
        assert_eq!(normalize_suffix("Mi"), "m");
    }

    #[test]
    fn test_normalize_suffix_major_seventh_variants() {
        assert_eq!(normalize_suffix("ma7"), "maj7");
        assert_eq!(normalize_suffix("MA7"), "maj7");
        assert_eq!(normalize_suffix("7M"), "maj7");
        assert_eq!(normalize_suffix("7m"), "maj7"); // Note: case insensitive match
    }

    #[test]
    fn test_normalize_suffix_minor_major_seventh() {
        assert_eq!(normalize_suffix("mM7"), "mmaj7");
        assert_eq!(normalize_suffix("mMaj7"), "mmaj7");
        assert_eq!(normalize_suffix("MM7"), "mmaj7");
        assert_eq!(normalize_suffix("MMAJ7"), "mmaj7");
    }

    #[test]
    fn test_normalize_suffix_minor_augmented() {
        assert_eq!(normalize_suffix("maug"), "m#5");
        assert_eq!(normalize_suffix("maug5"), "m#5");
        assert_eq!(normalize_suffix("maug7"), "m7#5");
        assert_eq!(normalize_suffix("MAUG"), "m#5");
        assert_eq!(normalize_suffix("MAUG5"), "m#5");
        assert_eq!(normalize_suffix("MAUG7"), "m7#5");
    }

    #[test]
    fn test_normalize_suffix_case_normalization() {
        // These should preserve case for non-prefix parts but normalize prefix
        assert_eq!(normalize_suffix("SUS2"), "sus2");
        assert_eq!(normalize_suffix("SUS4"), "sus4");
        assert_eq!(normalize_suffix("DIM7"), "dim7");
        assert_eq!(normalize_suffix("AUG7"), "aug7");
        assert_eq!(normalize_suffix("MAJ7"), "maj7");
        assert_eq!(normalize_suffix("MAJ9"), "maj9");
        assert_eq!(normalize_suffix("ADD9"), "add9");
        assert_eq!(normalize_suffix("ADD11"), "add11");
    }

    #[test]
    fn test_normalize_suffix_passthrough() {
        // These should pass through unchanged (already normalized)
        assert_eq!(normalize_suffix("m"), "m");
        assert_eq!(normalize_suffix("m7"), "m7");
        assert_eq!(normalize_suffix("maj7"), "maj7");
        assert_eq!(normalize_suffix("dim"), "dim");
        assert_eq!(normalize_suffix("dim7"), "dim7");
        assert_eq!(normalize_suffix("aug"), "aug");
        assert_eq!(normalize_suffix("7"), "7");
        assert_eq!(normalize_suffix("9"), "9");
        assert_eq!(normalize_suffix("sus2"), "sus2");
        assert_eq!(normalize_suffix("sus4"), "sus4");
        assert_eq!(normalize_suffix("6"), "6");
        assert_eq!(normalize_suffix("m6"), "m6");
        assert_eq!(normalize_suffix(""), "");
    }

    /// E2E test: verify interval encoding preserves chord qualities
    /// Since intervals are direction-agnostic (0-6), we verify qualities are preserved
    #[test]
    fn test_e2e_qualities_preserved() {
        // Real progression from dataset: A A/Ab C#m (uses slash chord)
        let progression = vec![
            "A".to_string(),
            "A/G#".to_string(),  // A with G# bass (enharmonic to Ab)
            "C#m".to_string(),
        ];

        let interval_key = history_to_interval_key(&progression).unwrap();

        // Verify the encoding captures the qualities
        assert!(interval_key.starts_with("M"), "First chord A should be Major");
        assert!(interval_key.contains("m"), "C#m should encode as minor");

        // The slash chord should include bass interval
        // A to G# is 11 semitones (NOT normalized for bass notes)
        assert!(interval_key.contains("M/11"), "A/G# should encode as M/11");
    }

    /// E2E test: verify same intervals produce same pattern from any root
    #[test]
    fn test_e2e_interval_pattern_consistency() {
        // C to G is interval 5 (up 7 semitones, normalized)
        // F to C is also interval 5 (up 7 semitones, normalized)
        let prog1 = vec!["C".to_string(), "G".to_string()];
        let prog2 = vec!["F".to_string(), "C".to_string()];

        let key1 = history_to_interval_key(&prog1).unwrap();
        let key2 = history_to_interval_key(&prog2).unwrap();

        // Both should have same pattern: M_5_M (major, 5 semitones, major)
        assert_eq!(key1, key2, "Same interval relationship should produce same key");
        assert_eq!(key1, "M_5_M");
    }

    /// E2E test with slash chords - verifies bass note encoding
    #[test]
    fn test_e2e_slash_chord_encoding() {
        // Bass intervals use full 0-11 range (not normalized)

        // C/E: C major with E in bass (E is 4 semitones from C)
        let (root, quality) = parse_chord_for_interval("C/E").unwrap();
        assert_eq!(root, 0, "Root should be C (semitone 0)");
        assert_eq!(quality, "M/4", "C/E should encode as M/4 (bass at interval 4)");

        // Am/G: A minor with G in bass (G is 10 semitones from A)
        let (root, quality) = parse_chord_for_interval("Am/G").unwrap();
        assert_eq!(root, 9, "Root should be A (semitone 9)");
        assert_eq!(quality, "m/10", "Am/G should encode as m/10 (bass at interval 10)");

        // G/B: G major with B in bass (B is 4 semitones from G)
        let (root, quality) = parse_chord_for_interval("G/B").unwrap();
        assert_eq!(root, 7, "Root should be G (semitone 7)");
        assert_eq!(quality, "M/4", "G/B should encode as M/4 (bass at interval 4)");

        // C/G: C major with G in bass (G is 7 semitones from C)
        let (root, quality) = parse_chord_for_interval("C/G").unwrap();
        assert_eq!(root, 0, "Root should be C (semitone 0)");
        assert_eq!(quality, "M/7", "C/G should encode as M/7 (bass at interval 7)");
    }

    /// E2E test: verify interval_to_chord produces correct output
    #[test]
    fn test_e2e_interval_to_chord_reconstruction() {
        // From C (semitone 0), go up 5 semitones with major quality
        let chord = interval_to_chord(5, "M", 0, true, "C").unwrap();
        assert_eq!(chord, "F", "0 + 5 semitones with M should be F");

        // From G (semitone 7), go up 5 semitones with minor quality
        let chord = interval_to_chord(5, "m", 7, true, "G").unwrap();
        assert_eq!(chord, "Cm", "7 + 5 semitones with m should be Cm");

        // From E (semitone 4), go 0 with 7 quality
        let chord = interval_to_chord(0, "7", 4, true, "C").unwrap();
        assert_eq!(chord, "E7", "E with 7 quality should be E7");
    }

    /// E2E test: slash chord reconstruction
    #[test]
    fn test_e2e_slash_chord_reconstruction() {
        // From C (0), quality "M/4" should produce C/E
        let chord = interval_to_chord(0, "M/4", 0, true, "C").unwrap();
        assert_eq!(chord, "C/E");

        // From G (7), quality "m/10" should produce Gm with bass at 7+10=17%12=5 (F)
        let chord = interval_to_chord(0, "m/10", 7, true, "G").unwrap();
        assert_eq!(chord, "Gm/F");

        // From F (5), quality "M/4" should produce F/A
        let chord = interval_to_chord(0, "M/4", 5, true, "C").unwrap();
        assert_eq!(chord, "F/A");

        // From C (0), quality "M/7" should produce C/G
        let chord = interval_to_chord(0, "M/7", 0, true, "C").unwrap();
        assert_eq!(chord, "C/G");
    }

    /// E2E test: verify complex qualities encode and decode correctly
    #[test]
    fn test_e2e_complex_qualities() {
        let test_cases = vec![
            ("Cmaj7", 0, "maj7"),
            ("Dm7", 2, "m7"),
            ("G7", 7, "7"),
            ("Bdim", 11, "dim"),
            ("Fmaj9", 5, "maj9"),
            ("Am7", 9, "m7"),
        ];

        for (chord, expected_root, expected_quality) in test_cases {
            let (root, quality) = parse_chord_for_interval(chord).unwrap();
            assert_eq!(root, expected_root, "Root of {} should be {}", chord, expected_root);
            assert_eq!(quality, expected_quality, "Quality of {} should be {}", chord, expected_quality);
        }
    }

    /// E2E test using real data patterns
    #[test]
    fn test_e2e_real_data_patterns() {
        // Common progressions from the dataset
        let progressions = vec![
            // I-V-vi-IV in C
            vec!["C".to_string(), "G".to_string(), "Am".to_string(), "F".to_string()],
            // ii-V-I in C
            vec!["Dm".to_string(), "G".to_string(), "C".to_string()],
            // I-IV-V in C
            vec!["C".to_string(), "F".to_string(), "G".to_string()],
        ];

        for prog in progressions {
            let key = history_to_interval_key(&prog).unwrap();

            // Verify key is not empty
            assert!(!key.is_empty(), "Interval key should not be empty for {:?}", prog);

            // Verify all qualities are present
            let parts: Vec<&str> = key.split('_').collect();
            let qualities: Vec<&str> = parts.iter().step_by(2).cloned().collect();

            assert_eq!(qualities.len(), prog.len(), "Should have {} qualities for {:?}", prog.len(), prog);
        }
    }
}
