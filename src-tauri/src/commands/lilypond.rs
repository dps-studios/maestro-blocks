use std::fs;
use std::process::Command;
use tempfile::TempDir;
use uuid::Uuid;

#[tauri::command]
pub async fn render_lilypond(notation: String) -> Result<String, String> {
    // Create temporary directory
    let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let temp_path = temp_dir.path();
    
    // Generate unique filename
    let file_id = Uuid::new_v4().to_string();
    let input_file = temp_path.join(format!("{}.ly", file_id));
    let output_dir = temp_path.join("output");
    
    // Create output directory
    fs::create_dir_all(&output_dir).map_err(|e| format!("Failed to create output directory: {}", e))?;
    
    // Write LilyPond notation to file
    fs::write(&input_file, &notation).map_err(|e| format!("Failed to write input file: {}", e))?;
    
    // Execute LilyPond command
    let output = Command::new("lilypond")
        .arg("--svg")
        .arg("-o")
        .arg(&output_dir)
        .arg(&input_file)
        .output()
        .map_err(|e| format!("Failed to execute lilypond: {}. Make sure LilyPond is installed and in PATH.", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("LilyPond execution failed: {}", stderr));
    }
    
    // Read the generated SVG file
    let svg_file = output_dir.join(format!("{}.svg", file_id));
    let svg_content = fs::read_to_string(&svg_file)
        .map_err(|e| format!("Failed to read SVG output: {}", e))?;
    
    Ok(svg_content)
}