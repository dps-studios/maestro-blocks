// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod music;
mod types;

use commands::lilypond::render_lilypond;
use commands::worksheet::{generate_worksheet, generate_chord_naming_template};
use commands::music::{generate_chord_pitches, get_chord_qualities};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            render_lilypond,
            generate_worksheet,
            generate_chord_naming_template,
            generate_chord_pitches,
            get_chord_qualities
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}