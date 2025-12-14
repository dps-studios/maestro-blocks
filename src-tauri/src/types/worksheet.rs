use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorksheetType {
    ChordNaming,
    IntervalRecognition,
    ScaleBuilding,
    RhythmExercise,
    NoteIdentification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EditableElementType {
    Chord,
    Note,
    Rest,
    Text,
    TimeSignature,
    KeySignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditableElement {
    pub id: String,
    pub element_type: EditableElementType,
    pub position: ElementPosition,
    pub content: String,
    pub is_answer: bool,
    pub is_interactive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementPosition {
    pub measure: u32,
    pub beat: u32,
    pub voice: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorksheetSection {
    pub id: String,
    pub title: String,
    pub instructions: Option<String>,
    pub elements: Vec<EditableElement>,
    pub layout: WorksheetSectionLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorksheetSectionLayout {
    pub measures_per_system: u32,
    pub systems_per_page: u32,
    pub clef: Clef,
    pub time_signature: Option<String>,
    pub key_signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Clef {
    Treble,
    Bass,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorksheetConfig {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    #[serde(rename = "worksheetType")]
    pub worksheet_type: WorksheetType,
    pub sections: Vec<WorksheetSection>,
    pub global_settings: WorksheetGlobalSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorksheetGlobalSettings {
    #[serde(rename = "paperSize")]
    pub paper_size: PaperSize,
    pub orientation: Orientation,
    #[serde(rename = "showAnswers")]
    pub show_answers: bool,
    #[serde(rename = "fontSize")]
    pub font_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaperSize {
    Letter,
    A4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordNamingParams {
    pub chords: Vec<ChordDefinition>,
    pub instructions: Option<String>,
    pub layout: ChordLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordDefinition {
    pub root: String,
    pub quality: ChordQuality,
    pub position: ElementPosition,
    #[serde(rename = "showAnswer")]
    pub show_answer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    #[serde(rename = "dominant7")]
    Dominant7,
    #[serde(rename = "major7")]
    Major7,
    #[serde(rename = "minor7")]
    Minor7,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordLayout {
    #[serde(rename = "chordsPerLine")]
    pub chords_per_line: u32,
    #[serde(rename = "showStaffLines")]
    pub show_staff_lines: bool,
}