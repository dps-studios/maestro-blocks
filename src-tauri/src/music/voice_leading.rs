// Voice leading calculations for smooth chord transitions
// Converts chords to MIDI notes with minimal movement between voicings

use super::types::{AudioNote, VoicingStyle, MusicError, MusicResult};

// Note to semitone mapping (same as in notes.rs)
static NOTE_TO_SEMITONE: &[(&str, u8)] = &[
    ("C", 0), ("C#", 1), ("Db", 1), ("D", 2), ("D#", 3), ("Eb", 3),
    ("E", 4), ("F", 5), ("F#", 6), ("Gb", 6), ("G", 7), ("G#", 8),
    ("Ab", 8), ("A", 9), ("A#", 10), ("Bb", 10), ("B", 11),
];

// Semitone to note mapping (sharp notation)
static SEMITONE_TO_SHARP: &[&str; 12] = &[
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Convert note name to MIDI number
/// Example: "C4" → 60, "A3" → 57
pub fn note_to_midi(note: &str, octave: i8) -> MusicResult<u8> {
    let note_idx = NOTE_TO_SEMITONE
        .iter()
        .find(|(n, _)| *n == note)
        .map(|(_, idx)| *idx)
        .ok_or_else(|| MusicError::ParseError(format!("Invalid note: {}", note)))?;

    let midi = (octave as u8 + 1) * 12 + note_idx;
    
    // Clamp to valid MIDI range
    if midi > 127 {
        return Err(MusicError::ParseError("MIDI note out of range".to_string()));
    }
    
    Ok(midi)
}

/// Convert MIDI number to note name
pub fn midi_to_note(midi: u8) -> MusicResult<String> {
    if midi > 127 {
        return Err(MusicError::ParseError("MIDI note out of range".to_string()));
    }
    
    let octave = (midi as i8 / 12) - 1;
    let semitone = midi % 12;
    let note = SEMITONE_TO_SHARP[semitone as usize];
    
    Ok(format!("{}{}", note, octave))
}

/// Calculate octave for close voicing ensuring ascending pitch
fn calc_close_voicing_octave(
    note: &str,
    base_octave: i8,
    prev_note: Option<&AudioNote>,
) -> MusicResult<i8> {
    let Some(prev) = prev_note else {
        return Ok(base_octave);
    };

    let prev_midi = note_to_midi(&prev.note, prev.octave)?;
    let curr_midi_at_base = note_to_midi(note, base_octave)?;

    // If current note at base octave is higher than previous, use base
    if curr_midi_at_base > prev_midi {
        return Ok(base_octave);
    }

    // Try previous note's octave
    let curr_midi_at_prev = note_to_midi(note, prev.octave)?;
    if curr_midi_at_prev > prev_midi {
        return Ok(prev.octave);
    }

    // Otherwise bump to next octave
    Ok(prev.octave + 1)
}

/// Apply close voicing: keep all notes within a small range
fn apply_close_voicing(notes: &[String], base_octave: i8) -> MusicResult<Vec<AudioNote>> {
    let mut result = Vec::with_capacity(notes.len());

    for (i, note) in notes.iter().enumerate() {
        let prev = if i > 0 { result.get(i - 1) } else { None };
        let octave = calc_close_voicing_octave(note, base_octave, prev)?;
        result.push(AudioNote { note: note.clone(), octave });
    }

    Ok(result)
}

/// Apply wide voicing: spread notes across ~2 octaves
fn apply_wide_voicing(notes: &[String], base_octave: i8) -> Vec<AudioNote> {
    notes
        .iter()
        .enumerate()
        .map(|(i, note)| AudioNote {
            note: note.clone(),
            octave: base_octave + (i / 2) as i8,
        })
        .collect()
}

/// Clamp audio note to valid MIDI range (A1-C5)
fn clamp_audio_note(audio_note: &mut AudioNote, min_midi: u8, max_midi: u8) -> MusicResult<()> {
    let midi = note_to_midi(&audio_note.note, audio_note.octave)?;
    let clamped = midi.max(min_midi).min(max_midi);

    if clamped != midi {
        let note_str = midi_to_note(clamped)?;
        let note_len = note_str.len().saturating_sub(1);
        if note_len > 0 {
            audio_note.note = note_str[..note_len].to_string();
            audio_note.octave = note_str[note_len..]
                .parse()
                .map_err(|_| MusicError::ParseError("Invalid octave".to_string()))?;
        }
    }

    Ok(())
}

/// Apply voicing strategy to a set of note names
/// Returns notes with octave assignments based on voicing configuration
pub fn voice_chord(
    notes: &[String],
    _bass_note: &str,
    base_octave: i8,
    style: VoicingStyle,
) -> MusicResult<Vec<AudioNote>> {
    let mut audio_notes = match style {
        VoicingStyle::Close => apply_close_voicing(notes, base_octave)?,
        VoicingStyle::Wide => apply_wide_voicing(notes, base_octave),
    };

    // Clamp all notes to available range (A1 = MIDI 21 to C5 = MIDI 72)
    for audio_note in &mut audio_notes {
        clamp_audio_note(audio_note, 21, 72)?;
    }

    Ok(audio_notes)
}

// Voice leading constants
const BASS_OCTAVE: i8 = 2;
const MIN_MIDI: u8 = 21;  // A1
const MAX_MIDI: u8 = 72;  // C5

thread_local! {
    static PREVIOUS_VOICING: std::cell::RefCell<Option<Vec<AudioNote>>> = std::cell::RefCell::new(None);
}

/// Reset voice leading state (called when starting new progression or changing key)
pub fn reset_voicing() {
    PREVIOUS_VOICING.with(|v| *v.borrow_mut() = None);
}

/// Convert MIDI number to AudioNote, clamping to valid range
fn midi_to_audio_note(midi: u8) -> MusicResult<AudioNote> {
    let clamped = midi.max(MIN_MIDI).min(MAX_MIDI);
    let note_str = midi_to_note(clamped)?;
    let note_len = note_str.len().saturating_sub(1);
    
    Ok(AudioNote {
        note: note_str[..note_len].to_string(),
        octave: note_str[note_len..].parse()
            .map_err(|_| MusicError::ParseError("Invalid octave".to_string()))?,
    })
}

/// Get previous upper voices (excluding bass) from thread-local state
fn get_previous_upper_voices() -> Option<Vec<AudioNote>> {
    PREVIOUS_VOICING.with(|v| {
        v.borrow()
            .as_ref()
            .filter(|prev| prev.len() > 1)
            .map(|prev| prev[1..].to_vec())
    })
}

/// Build initial voicing for first chord (no previous chord to reference)
fn build_initial_voicing(
    upper_notes: &[&String],
    bass_note: &str,
) -> MusicResult<Vec<AudioNote>> {
    let mut result = Vec::new();
    let mut current_octave: i8 = 2;
    let bass_midi = note_to_midi(bass_note, BASS_OCTAVE)?;

    for note in upper_notes {
        let octave = find_octave_above_bass(note, current_octave, bass_midi)?;
        let midi = note_to_midi(note, octave)?;
        result.push(midi_to_audio_note(midi)?);
        current_octave = octave;
    }

    Ok(result)
}

/// Find the lowest octave where note is above bass
fn find_octave_above_bass(note: &str, start_octave: i8, bass_midi: u8) -> MusicResult<i8> {
    let mut octave = start_octave;
    let mut midi = note_to_midi(note, octave)?;
    
    while midi <= bass_midi && octave < 5 {
        octave += 1;
        midi = note_to_midi(note, octave)?;
    }
    
    Ok(octave)
}

/// Build voicing using voice leading from previous chord
fn build_voice_led_voicing(
    upper_notes: &[&String],
    bass_note: &str,
    base_octave: i8,
    previous_upper: &[AudioNote],
) -> MusicResult<Vec<AudioNote>> {
    let mut result = Vec::new();
    let bass_midi = note_to_midi(bass_note, BASS_OCTAVE)?;

    for note in upper_notes {
        let best_octave = find_closest_octave(note, bass_midi, base_octave, previous_upper)?;
        let midi = note_to_midi(note, best_octave)?;
        result.push(midi_to_audio_note(midi)?);
    }

    Ok(result)
}

/// Find octave that minimizes distance to previous upper voices
fn find_closest_octave(
    note: &str,
    bass_midi: u8,
    default_octave: i8,
    previous_upper: &[AudioNote],
) -> MusicResult<i8> {
    let mut best_octave = default_octave;
    let mut min_distance = u8::MAX;

    for octave in 2..=3 {
        let midi = note_to_midi(note, octave)?;
        
        if midi < MIN_MIDI || midi > MAX_MIDI || midi <= bass_midi {
            continue;
        }

        for prev_note in previous_upper {
            let prev_midi = note_to_midi(&prev_note.note, prev_note.octave)?;
            let distance = midi.abs_diff(prev_midi);
            
            if distance < min_distance {
                min_distance = distance;
                best_octave = octave;
            }
        }
    }

    Ok(best_octave)
}

/// Sort upper voices by ascending pitch
fn sort_upper_voices_by_pitch(mut voices: Vec<AudioNote>) -> Vec<AudioNote> {
    voices.sort_by(|a, b| {
        let midi_a = note_to_midi(&a.note, a.octave).unwrap_or(0);
        let midi_b = note_to_midi(&b.note, b.octave).unwrap_or(0);
        midi_a.cmp(&midi_b)
    });
    voices
}

/// Voice a chord using voice leading principles
/// Bass note stays at octave 2, upper voices move to closest positions from previous chord
pub fn voice_chord_with_leading(
    notes: &[String],
    bass_note: &str,
    base_octave: i8,
) -> MusicResult<Vec<AudioNote>> {
    // 1. Bass voice - always at low octave
    let bass = AudioNote {
        note: bass_note.to_string(),
        octave: BASS_OCTAVE,
    };

    // 2. Upper voices - exclude bass note
    let upper_notes: Vec<&String> = notes
        .iter()
        .filter(|note| note.as_str() != bass_note)
        .collect();

    if upper_notes.is_empty() {
        return Ok(vec![bass]);
    }

    // 3. Build upper voices based on whether we have previous voicing
    let previous_upper = get_previous_upper_voices();
    let upper_voices = match previous_upper {
        None => build_initial_voicing(&upper_notes, bass_note)?,
        Some(ref prev) => build_voice_led_voicing(&upper_notes, bass_note, base_octave, prev)?,
    };

    // 4. Combine bass with sorted upper voices
    let mut result = vec![bass];
    result.extend(sort_upper_voices_by_pitch(upper_voices));

    // 5. Store for next chord
    PREVIOUS_VOICING.with(|v| *v.borrow_mut() = Some(result.clone()));

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_to_midi() {
        assert_eq!(note_to_midi("C", 4).unwrap(), 60);
        assert_eq!(note_to_midi("A", 3).unwrap(), 57);
        assert_eq!(note_to_midi("F#", 2).unwrap(), 42);
    }

    #[test]
    fn test_midi_to_note() {
        assert_eq!(midi_to_note(60).unwrap(), "C4");
        assert_eq!(midi_to_note(57).unwrap(), "A3");
        assert_eq!(midi_to_note(42).unwrap(), "F#2");
    }

    #[test]
    fn test_voice_chord_close() {
        let notes = vec!["C".to_string(), "E".to_string(), "G".to_string()];
        let result = voice_chord(&notes, "C", 3, VoicingStyle::Close).unwrap();
        
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].note, "C");
        assert_eq!(result[0].octave, 3);
        assert_eq!(result[1].note, "E");
        assert_eq!(result[2].note, "G");
    }

    #[test]
    fn test_voice_chord_wide() {
        let notes = vec!["C".to_string(), "E".to_string(), "G".to_string()];
        let result = voice_chord(&notes, "C", 3, VoicingStyle::Wide).unwrap();
        
        assert_eq!(result.len(), 3);
        // In wide voicing, notes should be spread across octaves
        assert!(result[0].octave <= result[1].octave);
        assert!(result[1].octave <= result[2].octave);
    }

    #[test]
    fn test_voice_chord_with_leading_first() {
        reset_voicing(); // Ensure clean state
        let notes = vec!["C".to_string(), "E".to_string(), "G".to_string()];
        let result = voice_chord_with_leading(&notes, "C", 3).unwrap();
        
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].note, "C");
        assert_eq!(result[0].octave, 2); // Bass always at octave 2
    }

    #[test]
    fn test_voice_chord_with_leading_bass_excluded_from_upper() {
        reset_voicing();
        // C major: C E G with C as bass
        let notes = vec!["C".to_string(), "E".to_string(), "G".to_string()];
        let result = voice_chord_with_leading(&notes, "C", 3).unwrap();
        
        // Bass is C, upper voices should be E and G only (C excluded)
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].note, "C"); // Bass
        // Upper voices should not duplicate C
        let upper_notes: Vec<&str> = result[1..].iter().map(|n| n.note.as_str()).collect();
        assert!(upper_notes.contains(&"E"));
        assert!(upper_notes.contains(&"G"));
    }

    #[test]
    fn test_voice_chord_with_leading_sequence() {
        reset_voicing();
        
        // First chord: C major
        let c_major = vec!["C".to_string(), "E".to_string(), "G".to_string()];
        let result1 = voice_chord_with_leading(&c_major, "C", 3).unwrap();
        assert_eq!(result1.len(), 3);
        
        // Second chord: G major - should use voice leading from previous
        let g_major = vec!["G".to_string(), "B".to_string(), "D".to_string()];
        let result2 = voice_chord_with_leading(&g_major, "G", 3).unwrap();
        assert_eq!(result2.len(), 3);
        assert_eq!(result2[0].note, "G"); // Bass
        assert_eq!(result2[0].octave, 2); // Bass always at octave 2
        
        // Upper voices should be close to previous chord's upper voices
        // Previous upper: E3, G3 (approximately)
        // New upper: B, D - should be placed close to minimize movement
    }

    #[test]
    fn test_voice_chord_with_leading_upper_voices_sorted() {
        reset_voicing();
        
        // Notes in non-ascending order
        let notes = vec!["C".to_string(), "G".to_string(), "E".to_string()];
        let result = voice_chord_with_leading(&notes, "C", 3).unwrap();
        
        // Upper voices (excluding bass) should be sorted by pitch
        if result.len() > 2 {
            let upper_midi_1 = note_to_midi(&result[1].note, result[1].octave).unwrap();
            let upper_midi_2 = note_to_midi(&result[2].note, result[2].octave).unwrap();
            assert!(upper_midi_1 <= upper_midi_2, "Upper voices should be sorted ascending");
        }
    }

    #[test]
    fn test_voice_chord_with_leading_range_limits() {
        reset_voicing();
        
        let notes = vec!["C".to_string(), "E".to_string(), "G".to_string()];
        let result = voice_chord_with_leading(&notes, "C", 3).unwrap();
        
        // All notes should be within MIDI 21 (A1) to 72 (C5)
        for audio_note in &result {
            let midi = note_to_midi(&audio_note.note, audio_note.octave).unwrap();
            assert!(midi >= 21, "Note {} should be >= A1 (MIDI 21), got MIDI {}", audio_note.note, midi);
            assert!(midi <= 72, "Note {} should be <= C5 (MIDI 72), got MIDI {}", audio_note.note, midi);
        }
    }

    #[test]
    fn test_voice_chord_with_leading_single_note() {
        reset_voicing();
        
        // Edge case: chord with only root note
        let notes = vec!["C".to_string()];
        let result = voice_chord_with_leading(&notes, "C", 3).unwrap();
        
        // Should just have bass note
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].note, "C");
        assert_eq!(result[0].octave, 2);
    }

    #[test]
    fn test_reset_voicing_clears_state() {
        // First chord sets state
        let notes = vec!["C".to_string(), "E".to_string(), "G".to_string()];
        let _ = voice_chord_with_leading(&notes, "C", 3).unwrap();
        
        // Reset should clear state
        reset_voicing();
        
        // Next chord should behave like first chord (no voice leading)
        let g_major = vec!["G".to_string(), "B".to_string(), "D".to_string()];
        let result = voice_chord_with_leading(&g_major, "G", 3).unwrap();
        
        // Should use initial voicing logic, not voice leading
        assert_eq!(result[0].note, "G");
        assert_eq!(result[0].octave, 2);
    }

    #[test]
    fn test_voice_chord_range_clamping() {
        // Test very low notes get clamped to A1 (MIDI 21)
        let notes = vec!["C".to_string(), "E".to_string()];
        let result = voice_chord(&notes, "C", 0, VoicingStyle::Close).unwrap();
        
        // Should clamp to minimum range
        assert!(result.iter().all(|note| {
            note_to_midi(&note.note, note.octave).unwrap_or(0) >= 21
        }));
    }
}