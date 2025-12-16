// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod music;
mod audio;
mod types;

use std::sync::Mutex;
use commands::audio::{AudioState, init_audio, play_chord, play_notes, stop_audio, set_volume, reset_voicing, play_one_shot};
use commands::export::{export_pdf, export_png};
use commands::lilypond::render_lilypond;
use commands::music::{generate_chord_pitches, get_chord_qualities};
use commands::worksheet::{generate_worksheet, generate_chord_naming_template};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AudioState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            // LilyPond commands
            render_lilypond,
            // Worksheet generation commands
            generate_worksheet,
            generate_chord_naming_template,
            // Music theory commands
            generate_chord_pitches,
            get_chord_qualities,
            // Audio playback commands
            init_audio,
            play_chord,
            play_notes,
            stop_audio,
            set_volume,
            reset_voicing,
            play_one_shot,
            // Export commands
            export_pdf,
            export_png,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}