use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Represents a musical chord with root, suffix, and optional bass note
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Chord {
    pub root: String,
    pub suffix: String,
    pub bass: Option<String>,
}

/// Display-ready chord notation with chord name and roman numeral
/// This is the authoritative representation for rendering chords to users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordNotation {
    pub chord: String,
    pub numeral: String,
}

/// Chord recommendation result with probability and numeral representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordRecommendation {
    pub chord: String,
    pub probability: f32,
    pub numeral: String,
}

/// Audio note with octave for voice leading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioNote {
    pub note: String,
    pub octave: i8,
}

/// Voicing style for chord arrangement
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VoicingStyle {
    Close,
    Wide,
}

/// Chord recommendation tier classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Safe,
    Colorful,
    Bold,
}

// KeyType moved to notes.rs since it's specific to note logic

/// Accidental type (sharp or flat)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Accidental {
    Flat,
    Sharp,
    DoubleFlat,
    DoubleSharp,
}

/// Roman numeral components
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RomanNumeralParts {
    pub degree: u8,              // 1-7
    pub accidental: Option<Accidental>,
    pub is_minor: bool,
    pub suffix: String,
    pub bass: Option<String>,    // For slash chords
}

/// Custom error type for music theory operations
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum MusicError {
    #[error("Invalid chord: {0}")]
    InvalidChord(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Invalid roman numeral: {0}")]
    InvalidRomanNumeral(String),

    #[error("Unknown chord quality: {0}")]
    UnknownQuality(String),

    #[error("Data lookup failed for key: {0}")]
    DataLookupFailed(String),

    #[error("Voice leading failed: {0}")]
    VoiceLeadingError(String),

    #[error("Parsing error: {0}")]
    ParseError(String),
}

/// Result type alias for music operations
pub type MusicResult<T> = Result<T, MusicError>;

/// Chord validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChordValidationResult {
    #[serde(rename = "isValid")]
    pub valid: bool,
    pub chord: Option<String>,
    pub normalized_chord: Option<String>,
    #[serde(rename = "error")]
    pub message: Option<String>,
    pub input_type: Option<String>,
}
