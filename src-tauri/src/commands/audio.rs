// Audio playback commands for Tauri
// These expose the Rust audio engine to the frontend

use std::sync::Mutex;
use tauri::State;

use crate::audio::AudioEngineHandle;
use crate::music::types::AudioNote;
use crate::music::voice_leading;
use crate::music::types::VoicingStyle;
use crate::music::intervals;

/// Managed state wrapper for audio engine handle
/// The handle is Send + Sync as it only contains a channel sender
pub struct AudioState(pub Mutex<Option<AudioEngineHandle>>);

/// Initialize audio engine (lazy initialization on first play if not called)
#[tauri::command]
pub fn init_audio(state: State<'_, AudioState>) -> Result<bool, String> {
    let mut guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;

    if guard.is_none() {
        *guard = Some(AudioEngineHandle::new()?);
    }

    Ok(true)
}

/// Play a chord with voice leading
/// Set is_final to true for the last chord of a progression (applies fade-out)
#[tauri::command]
pub fn play_chord(
    state: State<'_, AudioState>,
    chord: String,
    voicing_style: String,
    base_octave: i8,
    is_final: bool,
) -> Result<(), String> {
    // Validate input
    if chord.is_empty() {
        return Err("Chord cannot be empty".to_string());
    }

    // Get notes from chord
    let notes = intervals::chord_to_notes(&chord)
        .map_err(|e| format!("Failed to parse chord: {}", e))?;

    if notes.is_empty() {
        return Err("Chord has no notes".to_string());
    }

    // Get bass note (first note of chord)
    let bass_note = notes.first().cloned().unwrap_or_default();

    // Voice the chord based on style
    let audio_notes = match voicing_style.as_str() {
        "close" => voice_leading::voice_chord(&notes, &bass_note, base_octave, VoicingStyle::Close),
        "wide" => voice_leading::voice_chord(&notes, &bass_note, base_octave, VoicingStyle::Wide),
        "lead" | _ => voice_leading::voice_chord_with_leading(&notes, &bass_note, base_octave),
    }
    .map_err(|e| format!("Voice leading failed: {}", e))?;

    // Play the notes
    play_notes_internal(state, audio_notes, is_final)
}

/// Play raw notes (for direct note playback)
/// Set is_final to true for the last chord of a progression (applies fade-out)
#[tauri::command]
pub fn play_notes(
    state: State<'_, AudioState>,
    notes: Vec<AudioNote>,
    is_final: bool,
) -> Result<(), String> {
    play_notes_internal(state, notes, is_final)
}

/// Internal helper to play notes
fn play_notes_internal(
    state: State<'_, AudioState>,
    notes: Vec<AudioNote>,
    is_final: bool,
) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;

    // Lazy initialize if needed
    if guard.is_none() {
        *guard = Some(AudioEngineHandle::new()?);
    }

    if let Some(ref engine) = *guard {
        engine.play_notes(notes, is_final)?;
    }

    Ok(())
}

/// Stop all currently playing audio
#[tauri::command]
pub fn stop_audio(
    state: State<'_, AudioState>,
    immediate: bool,
) -> Result<(), String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;

    if let Some(ref engine) = *guard {
        engine.stop(immediate)?;
    }

    Ok(())
}

/// Set master volume (0.0 to 1.0)
#[tauri::command]
pub fn set_volume(
    state: State<'_, AudioState>,
    volume: f32,
) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;

    // Lazy initialize if needed
    if guard.is_none() {
        *guard = Some(AudioEngineHandle::new()?);
    }

    if let Some(ref engine) = *guard {
        engine.set_volume(volume)?;
    }

    Ok(())
}

/// Reset voice leading state (for starting new progression)
#[tauri::command]
pub fn reset_voicing() -> Result<(), String> {
    voice_leading::reset_voicing();
    Ok(())
}

/// Play a one-shot sound effect by name (e.g., "swoosh")
#[tauri::command]
pub fn play_one_shot(
    state: State<'_, AudioState>,
    sample_name: String,
) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;

    // Lazy initialize if needed
    if guard.is_none() {
        *guard = Some(AudioEngineHandle::new()?);
    }

    if let Some(ref engine) = *guard {
        engine.play_one_shot(&sample_name)?;
    }

    Ok(())
}
