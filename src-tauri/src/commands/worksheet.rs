use std::fs;
use std::process::Command;
use tempfile::TempDir;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::types::worksheet::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorksheetRequest {
    pub config: WorksheetConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorksheetResponse {
    pub svg_content: String,
    pub interactive_elements: Vec<InteractiveElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveElement {
    pub id: String,
    pub element_type: String,
    pub bounds: ElementBounds,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Generate a complete worksheet document using LilyPond
#[tauri::command]
pub async fn generate_worksheet(request: WorksheetRequest) -> Result<WorksheetResponse, String> {
    let lilypond_source = build_lilypond_document(&request.config)?;
    let svg_content = render_lilypond_document(lilypond_source)?;
    let interactive_elements = extract_interactive_elements(&svg_content)?;

    Ok(WorksheetResponse {
        svg_content,
        interactive_elements,
    })
}

/// Build a complete LilyPond document from worksheet configuration
fn build_lilypond_document(config: &WorksheetConfig) -> Result<String, String> {
    let paper_size = match config.global_settings.paper_size {
        PaperSize::Letter => "letter",
        PaperSize::A4 => "a4",
    };

    let orientation = match config.global_settings.orientation {
        Orientation::Portrait => "portrait",
        Orientation::Landscape => "landscape",
    };

    let mut document = format!(
        r#"\version "2.24.0"

#(set-paper-size "{}{}")

\paper {{
  indent = 0\mm
  line-width = 180\mm
  top-margin = 20\mm
  bottom-margin = 20\mm
  left-margin = 15\mm
  right-margin = 15\mm
  ragged-last-bottom = ##f
  print-all-headers = ##f
}}

\header {{
  title = "{}"
  subtitle = "{}"
  tagline = ##f
  composer = ##f
}}

"#,
        paper_size,
        if orientation == "landscape" { "-landscape" } else { "" },
        config.title,
        config.subtitle.as_deref().unwrap_or("")
    );

    // Add each section as a separate score
    for (index, section) in config.sections.iter().enumerate() {
        if index > 0 {
            document.push_str("\n\\pageBreak\n\n");
        }

        if !section.title.is_empty() {
            document.push_str(&format!(r#"\markup {{ \column {{
  \vspace #2
  \fill-line {{ \fontsize #2 \bold {{ "{}" }} }}
  {}
}}}}

"#, &section.title, 
            if let Some(inst) = &section.instructions {
                format!(r#"
  \fill-line {{ \italic {{ "{}" }} }}"#, inst)
            } else {
                String::new()
            }));
        }

        document.push_str(&build_section_lilypond(section, &config.global_settings)?);
    }

    Ok(document)
}

/// Build LilyPond code for a specific worksheet section
fn build_section_lilypond(
    section: &WorksheetSection, 
    global_settings: &WorksheetGlobalSettings
) -> Result<String, String> {
    let clef = match section.layout.clef {
        Clef::Treble => "treble",
        Clef::Bass => "bass",
        Clef::Both => "treble", // Handle both clefs with separate staves
    };

    let time_signature = section.layout.time_signature
        .as_deref()
        .unwrap_or("4/4");

    let key_signature = section.layout.key_signature
        .as_deref()
        .unwrap_or("c");

    // Generate music content and chord symbols from elements
    let (music_content, chord_symbols) = build_music_and_chords_from_elements(&section.elements, global_settings.show_answers)?;

    let score = format!(
        r#"\score {{
  <<
    \new ChordNames {{
      {}
    }}
    \new Staff {{
      \clef "{}"
      \key "{}" \major
      \time "{}"
      {}
    }}
  >>
  \layout {{
    \context {{
      \Staff
      \override NoteHead.output-attributes = #'((class . "interactive-note"))
      \override Rest.output-attributes = #'((class . "interactive-rest"))
    }}
    \context {{
      \ChordNames
      \override ChordName.output-attributes = #'((class . "interactive-chord"))
    }}
  }}
}}
"#,
        chord_symbols, clef, key_signature, time_signature, music_content
    );

    Ok(score)
}

/// Build LilyPond music notation and chord symbols from worksheet elements
fn build_music_and_chords_from_elements(
    elements: &[EditableElement], 
    show_answers: bool
) -> Result<(String, String), String> {
    let mut music = String::new();
    let mut chords = String::new();
    let mut current_measure = 1;
    let mut current_beat = 1;

    // Sort elements by position
    let mut sorted_elements = elements.to_vec();
    sorted_elements.sort_by(|a, b| {
        a.position.measure.cmp(&b.position.measure)
            .then_with(|| a.position.beat.cmp(&b.position.beat))
    });

    for element in &sorted_elements {
        // Add bar lines if needed
        while current_measure < element.position.measure {
            music.push_str(" | ");
            chords.push_str(" | ");
            current_measure += 1;
            current_beat = 1;
        }

        // Add rests for spacing if needed
        while current_beat < element.position.beat {
            music.push_str("r4 ");
            chords.push_str("s4 "); // Spacer in chord context
            current_beat += 1;
        }

        // Add the element
        match element.element_type {
            EditableElementType::Chord => {
                if show_answers || !element.is_answer {
                    // Add chord symbol
                    chords.push_str(&format!("{}4 ", element.content));
                    // Add simple chord notes (root position)
                    let root_note = get_chord_root_note(&element.content);
                    music.push_str(&format!("<{} {} {}>4 ", root_note, get_chord_third(&element.content), get_chord_fifth(&element.content)));
                } else {
                    // Show question mark for hidden answers
                    chords.push_str("r4 ");
                    music.push_str("r4 ");
                }
            }
            EditableElementType::Note => {
                if show_answers || !element.is_answer {
                    music.push_str(&format!("{}4 ", element.content));
                } else {
                    music.push_str("r4 ");
                }
                chords.push_str("s4 "); // Spacer for non-chord elements
            }
            EditableElementType::Rest => {
                music.push_str(&format!("{} ", element.content));
                chords.push_str("s4 ");
            }
            _ => {
                music.push_str("r4 "); // Default to rest
                chords.push_str("s4 ");
            }
        }

        current_beat += 1;
    }

    Ok((music, chords))
}

/// Extract root note from chord notation
fn get_chord_root_note(chord: &str) -> String {
    // Simple extraction - take first character(s) before any chord quality
    if chord.len() >= 2 && &chord[1..2] == "#" {
        format!("{}is", chord.chars().next().unwrap())
    } else if chord.len() >= 2 && &chord[1..2] == "b" {
        format!("{}es", chord.chars().next().unwrap())
    } else {
        chord.chars().next().unwrap().to_string()
    }
}

/// Get third of chord (simplified)
fn get_chord_third(chord: &str) -> String {
    let root = chord.chars().next().unwrap();
    if chord.contains("m") || chord.contains("dim") {
        // Minor third
        match root {
            'C' => "ees".to_string(),
            'D' => "f".to_string(),
            'E' => "g".to_string(),
            'F' => "aes".to_string(),
            'G' => "bes".to_string(),
            'A' => "c".to_string(),
            'B' => "d".to_string(),
            _ => "ees".to_string(),
        }
    } else {
        // Major third
        match root {
            'C' => "e".to_string(),
            'D' => "fis".to_string(),
            'E' => "gis".to_string(),
            'F' => "a".to_string(),
            'G' => "b".to_string(),
            'A' => "cis".to_string(),
            'B' => "dis".to_string(),
            _ => "e".to_string(),
        }
    }
}

/// Get fifth of chord (simplified - always perfect fifth except diminished)
fn get_chord_fifth(chord: &str) -> String {
    let root = chord.chars().next().unwrap();
    if chord.contains("dim") {
        // Diminished fifth
        match root {
            'C' => "ges".to_string(),
            'D' => "aes".to_string(),
            'E' => "bes".to_string(),
            'F' => "ces".to_string(),
            'G' => "des".to_string(),
            'A' => "ees".to_string(),
            'B' => "f".to_string(),
            _ => "ges".to_string(),
        }
    } else {
        // Perfect fifth
        match root {
            'C' => "g".to_string(),
            'D' => "a".to_string(),
            'E' => "b".to_string(),
            'F' => "c".to_string(),
            'G' => "d".to_string(),
            'A' => "e".to_string(),
            'B' => "fis".to_string(),
            _ => "g".to_string(),
        }
    }
}

/// Render LilyPond document to SVG
fn render_lilypond_document(lilypond_source: String) -> Result<String, String> {
    let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let temp_path = temp_dir.path();
    
    let file_id = Uuid::new_v4().to_string();
    let input_file = temp_path.join(format!("{}.ly", file_id));
    let output_dir = temp_path.join("output");
    
    fs::create_dir_all(&output_dir).map_err(|e| format!("Failed to create output directory: {}", e))?;
    
    fs::write(&input_file, lilypond_source).map_err(|e| format!("Failed to write input file: {}", e))?;
    
    let output = Command::new("lilypond")
        .arg("--svg")
        .arg("-dno-point-and-click") // Disable for now, we'll add our own interactivity
        .arg("-o")
        .arg(&output_dir)
        .arg(&input_file)
        .output()
        .map_err(|e| format!("Failed to execute lilypond: {}. Make sure LilyPond is installed and in PATH.", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("LilyPond execution failed: {}", stderr));
    }
    
    let svg_file = output_dir.join(format!("{}.svg", file_id));
    fs::read_to_string(&svg_file)
        .map_err(|e| format!("Failed to read SVG output: {}", e))
}

/// Extract interactive elements from SVG (placeholder for now)
fn extract_interactive_elements(_svg_content: &str) -> Result<Vec<InteractiveElement>, String> {
    // TODO: Parse SVG and extract elements with output-attributes
    // For now, return empty list
    Ok(vec![])
}

/// Generate chord naming worksheet template
#[tauri::command]
pub async fn generate_chord_naming_template(params: ChordNamingParams) -> Result<WorksheetConfig, String> {
    let mut elements = Vec::new();

    for (index, chord) in params.chords.iter().enumerate() {
        let element = EditableElement {
            id: format!("chord-{}", index),
            element_type: EditableElementType::Chord,
            position: chord.position.clone(),
            content: format_chord_notation(&chord.root, &chord.quality),
            is_answer: chord.show_answer,
            is_interactive: true,
        };
        elements.push(element);
    }

    let section = WorksheetSection {
        id: "chord-naming-section".to_string(),
        title: "Chord Identification".to_string(),
        instructions: Some(params.instructions.unwrap_or_else(|| "Identify the following chords".to_string())),
        elements,
        layout: WorksheetSectionLayout {
            measures_per_system: params.layout.chords_per_line,
            systems_per_page: 4,
            clef: Clef::Treble,
            time_signature: Some("4/4".to_string()),
            key_signature: Some("c".to_string()),
        },
    };

    let config = WorksheetConfig {
        id: Uuid::new_v4().to_string(),
        title: "Chord Naming Worksheet".to_string(),
        subtitle: None,
        worksheet_type: WorksheetType::ChordNaming,
        sections: vec![section],
        global_settings: WorksheetGlobalSettings {
            paper_size: PaperSize::Letter,
            orientation: Orientation::Portrait,
            show_answers: false,
            font_size: 14,
        },
    };

    Ok(config)
}

/// Format chord notation for LilyPond
fn format_chord_notation(root: &str, quality: &ChordQuality) -> String {
    let lilypond_root = match root {
        "C#" => "cis",
        "Db" => "des",
        "D#" => "dis",
        "Eb" => "ees",
        "F#" => "fis",
        "Gb" => "ges",
        "G#" => "gis",
        "Ab" => "aes",
        "A#" => "ais",
        "Bb" => "bes",
        _ => root,
    };

    let chord_extension = match quality {
        ChordQuality::Major => "",
        ChordQuality::Minor => "m",
        ChordQuality::Diminished => "dim",
        ChordQuality::Augmented => "aug",
        ChordQuality::Dominant7 => "7",
        ChordQuality::Major7 => "maj7",
        ChordQuality::Minor7 => "m7",
    };

    format!("{}{}", lilypond_root, chord_extension)
}