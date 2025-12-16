use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, FilePath};
use usvg::fontdb;

/// Create a font database with the bundled Bravura music font loaded.
/// Also loads system fonts as fallbacks.
fn create_fontdb_with_bravura(app: &tauri::AppHandle) -> Result<fontdb::Database, String> {
    let mut db = fontdb::Database::new();
    
    // Load system fonts as fallback for text elements
    db.load_system_fonts();
    
    // Try multiple paths to find Bravura font:
    // 1. Production: resource_dir/fonts/Bravura.otf (bundled with app)
    // 2. Development: src-tauri/resources/fonts/Bravura.otf (source location)
    let mut font_paths = Vec::new();
    
    // Production path (bundled resources)
    if let Ok(resource_dir) = app.path().resource_dir() {
        font_paths.push(resource_dir.join("fonts").join("Bravura.otf"));
    }
    
    // Development fallback paths
    font_paths.push(std::path::PathBuf::from("resources/fonts/Bravura.otf"));
    font_paths.push(std::path::PathBuf::from("src-tauri/resources/fonts/Bravura.otf"));
    
    // Try each path until one works
    let mut font_loaded = false;
    for font_path in &font_paths {
        if font_path.exists() {
            match db.load_font_file(font_path) {
                Ok(_) => {
                    println!("[export] Loaded Bravura font from {:?}", font_path);
                    font_loaded = true;
                    break;
                }
                Err(e) => {
                    println!("[export] Failed to load font from {:?}: {}", font_path, e);
                }
            }
        }
    }
    
    if !font_loaded {
        println!("[export] Warning: Bravura font not found in any of: {:?}", font_paths);
    }
    
    Ok(db)
}

/// Export the canvas SVG content to a PDF file.
/// 
/// The SVG is converted to PDF using svg2pdf. The PDF page size is determined
/// by the SVG's viewBox dimensions at 72 DPI (1 SVG unit = 1 PDF point).
/// For 8.5x11 inch output, the SVG should have viewBox="0 0 612 792".
/// 
/// The Bravura music font is loaded into the font database for proper
/// rendering of music notation symbols (noteheads, clefs, etc.).
#[tauri::command]
pub async fn export_pdf(
    app: tauri::AppHandle,
    svg_content: String,
    default_filename: String,
) -> Result<bool, String> {
    // Show native save dialog
    let file_path = app
        .dialog()
        .file()
        .add_filter("PDF Document", &["pdf"])
        .set_file_name(&default_filename)
        .set_title("Export as PDF")
        .blocking_save_file();

    let file_path = match file_path {
        Some(path) => path,
        None => return Ok(false), // User cancelled
    };

    // Convert FilePath to std::path::PathBuf
    let path = match file_path {
        FilePath::Path(p) => p,
        _ => return Err("Invalid file path".to_string()),
    };

    // Create font database with Bravura loaded
    let fontdb = create_fontdb_with_bravura(&app)?;

    // Parse SVG with usvg using our custom font database
    let options = usvg::Options {
        fontdb: Arc::new(fontdb),
        ..Default::default()
    };
    
    let tree = usvg::Tree::from_str(&svg_content, &options)
        .map_err(|e| format!("Failed to parse SVG: {}", e))?;

    // Convert SVG to PDF (page size determined by SVG viewBox at 72 DPI)
    let pdf = svg2pdf::to_pdf(
        &tree,
        svg2pdf::ConversionOptions::default(),
        svg2pdf::PageOptions::default(),
    )
    .map_err(|e| format!("Failed to convert to PDF: {}", e))?;

    // Write PDF to file
    std::fs::write(&path, pdf)
        .map_err(|e| format!("Failed to write PDF: {}", e))?;

    Ok(true)
}

/// Export the canvas SVG content to a PNG file.
/// 
/// The SVG is rendered to PNG using resvg at 300 DPI for print quality.
#[tauri::command]
pub async fn export_png(
    app: tauri::AppHandle,
    svg_content: String,
    default_filename: String,
) -> Result<bool, String> {
    // Show native save dialog
    let file_path = app
        .dialog()
        .file()
        .add_filter("PNG Image", &["png"])
        .set_file_name(&default_filename)
        .set_title("Export as PNG")
        .blocking_save_file();

    let file_path = match file_path {
        Some(path) => path,
        None => return Ok(false), // User cancelled
    };

    // Convert FilePath to std::path::PathBuf
    let path = match file_path {
        FilePath::Path(p) => p,
        _ => return Err("Invalid file path".to_string()),
    };

    // Create font database with Bravura loaded
    let fontdb = create_fontdb_with_bravura(&app)?;

    // Parse SVG with usvg using our custom font database
    let options = usvg::Options {
        fontdb: Arc::new(fontdb),
        ..Default::default()
    };
    
    let tree = usvg::Tree::from_str(&svg_content, &options)
        .map_err(|e| format!("Failed to parse SVG: {}", e))?;

    // Render at 300 DPI (4.16x scale from 72 DPI)
    // 8.5 x 11 inches at 300 DPI = 2550 x 3300 pixels
    let scale = 300.0 / 72.0;
    let width = (8.5 * 300.0) as u32;  // 2550 pixels
    let height = (11.0 * 300.0) as u32; // 3300 pixels

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or("Failed to create pixmap")?;
    
    // Fill with white background
    pixmap.fill(resvg::tiny_skia::Color::WHITE);

    // Render SVG to pixmap
    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Encode as PNG and write to file
    let png_data = pixmap.encode_png()
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    
    std::fs::write(&path, png_data)
        .map_err(|e| format!("Failed to write PNG: {}", e))?;

    Ok(true)
}
