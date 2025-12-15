use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Run the standard Tauri build
    tauri_build::build();

    // Get the output directory for generated code
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir);

    // Generate embedded audio samples
    println!("cargo:rerun-if-changed=resources/samples");
    generate_audio_samples(dest_path);
}

fn generate_audio_samples(out_dir: &Path) {
    println!("cargo:warning=Generating embedded audio samples...");

    // Get CARGO_MANIFEST_DIR for absolute paths in include_bytes!
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    // Define all sample keys: notes A-G# for octaves 1-4, plus C5
    let notes = ["C", "Cs", "D", "Ds", "E", "F", "Fs", "G", "Gs", "A", "As", "B"];
    let mut samples: Vec<(String, String)> = Vec::new();

    // Octaves 1-4 for all notes
    for octave in 1..=4 {
        for note in &notes {
            let key = format!("{}{}", note, octave);
            let filename = format!("{}_bip.ogg", key);
            samples.push((key, filename));
        }
    }

    // Add C5 (top of range)
    samples.push(("C5".to_string(), "C5_bip.ogg".to_string()));

    // Add sound effects
    samples.push(("swoosh".to_string(), "swoosh.ogg".to_string()));

    // Check if samples exist before generating code
    let samples_dir = Path::new(&manifest_dir).join("resources/samples");
    if !samples_dir.exists() {
        println!("cargo:warning=WARNING: resources/samples/ directory not found");
        println!("cargo:warning=Audio samples will not be embedded. See resources/samples/README.md");
        
        // Generate empty samples map
        let mut code = String::new();
        code.push_str("// Auto-generated audio sample embeddings\n");
        code.push_str("// WARNING: No samples directory found - empty map generated\n\n");
        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use once_cell::sync::Lazy;\n\n");
        code.push_str("pub static SAMPLES: Lazy<HashMap<&'static str, &'static [u8]>> = Lazy::new(|| {\n");
        code.push_str("    HashMap::new()\n");
        code.push_str("});\n");

        let dest_file = out_dir.join("audio_samples.rs");
        fs::write(&dest_file, code).expect("Failed to write audio_samples.rs");
        return;
    }

    // Generate Rust code with absolute paths for include_bytes!
    let mut code = String::new();
    code.push_str("// Auto-generated audio sample embeddings\n");
    code.push_str("// Do not edit manually\n\n");
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use once_cell::sync::Lazy;\n\n");

    code.push_str("pub static SAMPLES: Lazy<HashMap<&'static str, &'static [u8]>> = Lazy::new(|| {\n");
    code.push_str("    let mut m = HashMap::new();\n");

    let mut embedded_count = 0;
    for (key, filename) in &samples {
        let full_path = format!("{}/resources/samples/{}", manifest_dir, filename);
        let path_obj = Path::new(&full_path);
        
        if path_obj.exists() {
            code.push_str(&format!(
                "    m.insert(\"{}\", include_bytes!(\"{}\").as_slice());\n",
                key, full_path
            ));
            embedded_count += 1;
        } else {
            println!("cargo:warning=Missing sample file: {}", filename);
        }
    }

    code.push_str("    m\n");
    code.push_str("});\n");

    // Write the generated code
    let dest_file = out_dir.join("audio_samples.rs");
    fs::write(&dest_file, code).expect("Failed to write audio_samples.rs");

    println!("cargo:warning=Generated {} embedded audio samples", embedded_count);
    
    if embedded_count == 0 {
        println!("cargo:warning=WARNING: No audio sample files were found!");
        println!("cargo:warning=See resources/samples/README.md for instructions");
    }
}
