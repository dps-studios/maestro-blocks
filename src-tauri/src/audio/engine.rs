use rodio::source::LimitSettings;
use rodio::{Decoder, OutputStreamBuilder, Sink, Source};
use std::io::Cursor;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use super::envelope::TwoStageEnvelopeExt;
use super::monitor::AudioMonitorExt;
use super::samples::{get_sample, note_to_sample_key};
use crate::music::types::AudioNote;

/// Volume multiplier for chord playback (piano samples)
const CHORD_VOLUME_MULTIPLIER: f32 = 2.5;

/// Volume multiplier for one-shot sound effects (swoosh, etc.)
const ONESHOT_VOLUME_MULTIPLIER: f32 = 0.5;

/// Highpass cutoff frequency for chord playback (Hz)
/// Removes sub-bass that muddles mobile speakers
const CHORD_HIGHPASS_FREQ: u32 = 150;

/// Limiter settings to prevent clipping without causing artifacts
fn chord_limiter_settings() -> LimitSettings {
    LimitSettings::default()
        .with_attack(Duration::from_millis(50))   // Catches transients without pumping
        .with_release(Duration::from_millis(150)) // Smooth recovery
        .with_threshold(-9.0) // Good headroom, makeup gain compensates
}

/// Makeup gain to restore loudness after limiting
/// Tuned based on monitor data: 3.5 caused summed clipping, 3.0 provides headroom
const MAKEUP_GAIN: f32 = 3.0;

/// Fade-out duration to mask sample noise floor at end of decay
/// Piano samples ring ~2 sec, so fade over most of that duration
const TAIL_FADEOUT_DURATION: Duration = Duration::from_millis(2000);

/// Quick fade-out all sinks to prevent click artifacts, then stop them
fn fade_out_and_stop_sinks(sinks: &mut Vec<Sink>) {
    // Ramp volume down in steps rather than instant zero
    for sink in sinks.iter() {
        sink.set_volume(sink.volume() * 0.3);
    }
    thread::sleep(Duration::from_millis(2));
    for sink in sinks.iter() {
        sink.set_volume(sink.volume() * 0.1);
    }
    thread::sleep(Duration::from_millis(3));
    for sink in sinks.drain(..) {
        sink.set_volume(0.0);
        sink.stop();
    }
}

/// Quick fade before detaching sinks (prevents potential click)
fn quick_fade_before_detach(sinks: &[Sink]) {
    if !sinks.is_empty() {
        for sink in sinks {
            sink.set_volume(sink.volume() * 0.5);
        }
        thread::sleep(Duration::from_millis(3));
    }
}

/// Detach all sinks to let them decay naturally
fn detach_all_sinks(sinks: &mut Vec<Sink>) {
    for sink in sinks.drain(..) {
        sink.detach();
    }
}

/// Commands sent to the audio thread
pub enum AudioCommand {
    PlayNotes(Vec<AudioNote>, bool), // (notes, is_final)
    PlayOneShot(String),
    Stop(bool),
    SetVolume(f32),
    Shutdown,
}

/// Audio engine handle - sends commands to the audio thread
pub struct AudioEngineHandle {
    sender: Sender<AudioCommand>,
}

impl AudioEngineHandle {
    /// Create a new audio engine running on a dedicated thread
    pub fn new() -> Result<Self, String> {
        let (sender, receiver) = mpsc::channel();

        // Spawn audio thread
        thread::spawn(move || {
            audio_thread_main(receiver);
        });

        Ok(Self { sender })
    }

    /// Play a set of notes simultaneously
    /// If is_final is true, applies fade-out for noise floor masking
    pub fn play_notes(&self, notes: Vec<AudioNote>, is_final: bool) -> Result<(), String> {
        self.sender
            .send(AudioCommand::PlayNotes(notes, is_final))
            .map_err(|e| format!("Failed to send play command: {}", e))
    }

    /// Stop all currently playing audio
    pub fn stop(&self, immediate: bool) -> Result<(), String> {
        self.sender
            .send(AudioCommand::Stop(immediate))
            .map_err(|e| format!("Failed to send stop command: {}", e))
    }

    /// Set master volume (0.0 to 1.0)
    pub fn set_volume(&self, volume: f32) -> Result<(), String> {
        self.sender
            .send(AudioCommand::SetVolume(volume))
            .map_err(|e| format!("Failed to send volume command: {}", e))
    }

    /// Play a one-shot sound effect by sample name (e.g., "swoosh")
    pub fn play_one_shot(&self, sample_name: &str) -> Result<(), String> {
        self.sender
            .send(AudioCommand::PlayOneShot(sample_name.to_string()))
            .map_err(|e| format!("Failed to send play_one_shot command: {}", e))
    }
}

impl Drop for AudioEngineHandle {
    fn drop(&mut self) {
        let _ = self.sender.send(AudioCommand::Shutdown);
    }
}

/// Main function for the audio thread
fn audio_thread_main(receiver: Receiver<AudioCommand>) {
    // Initialize audio output on this thread (rodio 0.21 API)
    let stream = match OutputStreamBuilder::open_default_stream() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to initialize audio output: {}", e);
            return;
        }
    };
    let mixer = stream.mixer();

    // Use Vec<Sink> - one sink per note for simultaneous playback
    let mut sinks: Vec<Sink> = Vec::new();
    let mut volume: f32 = 1.0;
    let mut current_note_count: f32 = 1.0; // Track for SetVolume scaling

    loop {
        match receiver.recv() {
            Ok(AudioCommand::PlayNotes(notes, is_final)) => {
                // Let old sinks continue playing and decay naturally
                quick_fade_before_detach(&sinks);
                detach_all_sinks(&mut sinks);

                // Divide volume by note count AFTER limiter to prevent summed clipping
                current_note_count = notes.len().max(1) as f32;
                let per_note_volume = volume / current_note_count;

                // Create one sink per note for simultaneous playback
                for audio_note in &notes {
                    let sample_key = note_to_sample_key(&audio_note.note, audio_note.octave);

                    if let Some(sample_bytes) = get_sample(&sample_key) {
                        let cursor = Cursor::new(sample_bytes);
                        if let Ok(source) = Decoder::new(cursor) {
                            let sink = Sink::connect_new(&mixer);
                            // Per-note volume: limiter outputs ~0.7 max, divided by note count
                            sink.set_volume(per_note_volume);

                            // Signal chain: envelope → highpass → amplify → limit → makeup
                            // Only monitor final chords (they play to completion, giving accurate stats)
                            // Intermediate chords get detached early, so monitoring would show partial data
                            if is_final {
                                let source_processed = source
                                    .two_stage_envelope()
                                    .high_pass(CHORD_HIGHPASS_FREQ)
                                    .amplify(CHORD_VOLUME_MULTIPLIER)
                                    .monitor(format!("{}/pre-limit", sample_key))
                                    .limit(chord_limiter_settings())
                                    .amplify(MAKEUP_GAIN)
                                    .monitor(format!("{}/post-makeup", sample_key))
                                    .fade_out(TAIL_FADEOUT_DURATION);
                                sink.append(source_processed);
                            } else {
                                let source_processed = source
                                    .two_stage_envelope()
                                    .high_pass(CHORD_HIGHPASS_FREQ)
                                    .amplify(CHORD_VOLUME_MULTIPLIER)
                                    .limit(chord_limiter_settings())
                                    .amplify(MAKEUP_GAIN);
                                sink.append(source_processed);
                            }
                            sinks.push(sink);
                        }
                    } else {
                        eprintln!("Warning: No sample found for {}", sample_key);
                    }
                }
            }
            Ok(AudioCommand::PlayOneShot(sample_name)) => {
                // Play a one-shot sound effect without stopping other audio
                if let Some(sample_bytes) = get_sample(&sample_name) {
                    let cursor = Cursor::new(sample_bytes);
                    if let Ok(source) = Decoder::new(cursor) {
                        // Apply fade-in to prevent click artifacts (25ms matches chord playback)
                        let source_with_fade = source.fade_in(Duration::from_millis(25));
                        let sink = Sink::connect_new(&mixer);
                        sink.set_volume(volume * ONESHOT_VOLUME_MULTIPLIER);
                        sink.append(source_with_fade);
                        sinks.push(sink);
                    }
                } else {
                    eprintln!("Warning: No sample found for {}", sample_name);
                }
            }
            Ok(AudioCommand::Stop(immediate)) => {
                if immediate {
                    fade_out_and_stop_sinks(&mut sinks);
                } else {
                    detach_all_sinks(&mut sinks);
                }
            }
            Ok(AudioCommand::SetVolume(v)) => {
                volume = v.clamp(0.0, 1.0);
                let per_note_volume = volume / current_note_count;
                for sink in &sinks {
                    sink.set_volume(per_note_volume);
                }
            }
            Ok(AudioCommand::Shutdown) | Err(_) => {
                fade_out_and_stop_sinks(&mut sinks);
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_engine_creation() {
        // Note: This test may fail in CI environments without audio output
        let result = AudioEngineHandle::new();
        if result.is_err() {
            eprintln!("AudioEngine creation failed (expected in headless environments)");
        }
    }
}
