use serde::{Deserialize, Serialize};
use crate::music::CHORD_INTERVALS;

/// A note with octave for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Convert semitone to note name with preferred accidental
fn semitone_to_note(semitone: i32, prefer_sharp: bool) -> String {
    let s = ((semitone % 12) + 12) % 12;
    match s {
        0 => "C".to_string(),
        1 => if prefer_sharp { "C#" } else { "Db" }.to_string(),
        2 => "D".to_string(),
        3 => if prefer_sharp { "D#" } else { "Eb" }.to_string(),
        4 => "E".to_string(),
        5 => "F".to_string(),
        6 => if prefer_sharp { "F#" } else { "Gb" }.to_string(),
        7 => "G".to_string(),
        8 => if prefer_sharp { "G#" } else { "Ab" }.to_string(),
        9 => "A".to_string(),
        10 => if prefer_sharp { "A#" } else { "Bb" }.to_string(),
        11 => "B".to_string(),
        _ => "C".to_string(),
    }
}

/// Generate chord pitches from root, quality, and octave
#[tauri::command]
pub fn generate_chord_pitches(request: ChordRequest) -> Result<ChordResponse, String> {
    let quality = normalize_quality(&request.quality);
    
    // Get intervals for this chord quality
    let intervals = CHORD_INTERVALS.get(quality)
        .ok_or_else(|| format!("Unknown chord quality: {}", request.quality))?;
    
    // Get root semitone
    let root_semitone = note_to_semitone(&request.root)
        .ok_or_else(|| format!("Invalid root note: {}", request.root))?;
    
    // Determine if we should prefer sharps or flats based on root
    let prefer_sharp = !request.root.contains('b');
    
    // Generate pitches from intervals
    let mut pitches: Vec<PitchResult> = Vec::new();
    let mut current_octave = request.root_octave;
    let mut prev_semitone = root_semitone;
    
    for (i, &interval) in intervals.iter().enumerate() {
        let absolute_semitone = root_semitone + (interval as i32);
        let note_semitone = ((absolute_semitone % 12) + 12) % 12;
        
        // Determine octave - if semitone is lower than previous, we've wrapped
        if i > 0 && note_semitone < prev_semitone {
            current_octave += 1;
        }
        prev_semitone = note_semitone;
        
        let note = semitone_to_note(note_semitone, prefer_sharp);
        
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
    CHORD_INTERVALS.keys().map(|s| s.to_string()).collect()
}
