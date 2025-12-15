# Audio Engine Integration Guide

This document explains the audio playback system in Maestro Blocks, which is adapted from the Canon project's proven audio architecture.

## Architecture Overview

The audio engine provides native audio playback using **Rodio** (Rust audio library) for reliable cross-platform support without Web Audio API limitations.

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (TypeScript)                    │
│  Audio wrapper functions call Tauri commands                │
└───────────────────────┬─────────────────────────────────────┘
                        │ invoke() via Tauri IPC
┌───────────────────────▼─────────────────────────────────────┐
│                  Rust Backend (src-tauri/)                   │
│  src/commands/audio.rs → src/audio/*.rs                     │
│  - Dedicated audio thread                                    │
│  - Rodio-based playback engine                              │
│  - Voice leading integration                                 │
└─────────────────────────────────────────────────────────────┘
```

## Backend Components

### Audio Modules (`src-tauri/src/audio/`)

| Module | Purpose |
|--------|---------|
| `engine.rs` | Core playback engine with command queue pattern |
| `samples.rs` | Embedded audio sample loading (build-time generation) |
| `envelope.rs` | Two-stage attack envelope (prevents clicks) |
| `monitor.rs` | Audio level monitoring for debugging |
| `mod.rs` | Module exports |

### Command Handlers (`src-tauri/src/commands/audio.rs`)

Exposes audio functionality to the frontend via Tauri commands:

| Command | Purpose |
|---------|---------|
| `init_audio()` | Initialize audio engine (lazy, optional) |
| `play_chord(chord, voicing_style, base_octave, is_final)` | Play chord with voice leading |
| `play_notes(notes, is_final)` | Play raw notes directly |
| `stop_audio(immediate)` | Stop all audio |
| `set_volume(volume)` | Set master volume (0.0-1.0) |
| `reset_voicing()` | Reset voice leading memory |
| `play_one_shot(sample_name)` | Play sound effect |

## Audio Samples

### Required Files

The engine expects **49 piano samples** in OGG Vorbis format, covering the range **A1 to C5**:

- Location: `src-tauri/resources/samples/`
- Format: `{Note}{Octave}_bip.ogg`
- Examples: `C3_bip.ogg`, `Cs4_bip.ogg`, `As2_bip.ogg`
- Sharp notation: C# = Cs, F# = Fs, etc.

See `src-tauri/resources/samples/README.md` for the complete list.

### Build-Time Embedding

Audio samples are embedded into the binary at compile time via `build.rs`:

1. `build.rs` scans `resources/samples/` directory
2. Generates `audio_samples.rs` with `include_bytes!()` calls
3. Creates static `HashMap<&str, &[u8]>` for fast lookup
4. If samples are missing, build warns but continues with empty map

**Note**: You must obtain piano samples separately. Canon project samples can be copied from `/tmp/canon-analysis/src-tauri/resources/samples/` if available.

## DSP Chain

Each played note goes through this processing chain:

```
Sample → TwoStageEnvelope → HighpassFilter(150Hz) 
      → Limiter(threshold:-9dB) → MakeupGain(3.0x) → Output
```

### Why This Chain?

1. **Two-Stage Envelope**: Prevents click at note start (3ms linear + 12ms exponential attack)
2. **Highpass Filter (150Hz)**: Removes sub-bass that muddles mobile speakers
3. **Limiter**: Prevents clipping when multiple notes play simultaneously
4. **Makeup Gain**: Restores loudness after limiting

## Click/Pop Prevention

The engine uses several techniques to avoid audio artifacts:

1. **Detach Instead of Stop**
   ```rust
   // BAD - causes clicks
   sink.stop();
   
   // GOOD - lets audio decay naturally
   sink.detach();
   ```

2. **Always Apply Fade-In**
   ```rust
   source.two_stage_envelope()  // 3ms + 12ms attack
   ```

3. **Fade-Out on Final Chord**
   ```rust
   play_chord(chord, config, is_final: true)  // 2-second fade
   ```

## Voice Leading Integration

The audio engine works seamlessly with the music theory engine:

```typescript
// Frontend calls
await invoke('play_chord', {
  chord: 'Cmaj7',
  voicingStyle: 'lead',  // 'lead' | 'close' | 'wide'
  baseOctave: 2,
  isFinal: false
});
```

Backend flow:
1. Parse chord → `['C', 'E', 'G', 'B']`
2. Apply voice leading → `[(C,2), (E,3), (G,3), (B,3)]`
3. Load samples → `[C2.ogg, E3.ogg, G3.ogg, B3.ogg]`
4. Apply DSP chain → Envelope + Filter + Limiter
5. Play simultaneously → One sink per note

## Thread Architecture

```
Main Thread                 Audio Thread
────────────                ────────────
invoke('play_chord')  ──►   Receive Command
                            ↓
                       Load Samples
                            ↓
                       Apply DSP Chain
                            ↓
                       Create Sinks
                            ↓
                       Play & Detach
```

- **Non-blocking**: Audio thread handles all playback
- **Command Queue**: Uses `mpsc::channel` for thread communication
- **Lazy Init**: Engine spawns thread on first use

## State Management

The audio engine uses Tauri managed state:

```rust
// In main.rs
.manage(AudioState(Mutex<Option<AudioEngineHandle>>))

// Lazy initialization in commands
if guard.is_none() {
    *guard = Some(AudioEngineHandle::new()?);
}
```

Voice leading state is separate (thread-local):

```rust
// In voice_leading.rs
static VOICE_LEADING_MEMORY: LazyLock<Mutex<Option<Vec<u8>>>> = ...
```

## Frontend Integration

### TypeScript Wrapper (To Be Created)

```typescript
// src/lib/audio/chordPlayer.ts
import { invoke } from '@tauri-apps/api/core';

export async function playChord(
  chord: string,
  config: { style: 'lead' | 'close' | 'wide', baseOctave: number } = 
    { style: 'lead', baseOctave: 2 },
  isFinal: boolean = false
): Promise<void> {
  await invoke('play_chord', {
    chord,
    voicingStyle: config.style,
    baseOctave: config.baseOctave,
    isFinal
  });
}

export async function stopAllAudio(): Promise<void> {
  await invoke('stop_audio', { immediate: true });
}

export async function resetVoicing(): Promise<void> {
  await invoke('reset_voicing');
}
```

### Usage Example

```typescript
import { playChord, resetVoicing } from '$lib/audio/chordPlayer';

// Play a progression
await resetVoicing();  // Start fresh
await playChord('C', { style: 'lead', baseOctave: 2 });
await new Promise(r => setTimeout(r, 800));  // Timing
await playChord('Am', { style: 'lead', baseOctave: 2 });
await new Promise(r => setTimeout(r, 800));
await playChord('F', { style: 'lead', baseOctave: 2 }, true);  // Final chord
```

## Testing

### Manual Testing

1. **Copy samples** from Canon project:
   ```bash
   cp -r /tmp/canon-analysis/src-tauri/resources/samples/* \
         src-tauri/resources/samples/
   ```

2. **Build the project**:
   ```bash
   cd src-tauri
   cargo build
   ```
   
   Check for warnings about embedded samples.

3. **Test audio commands** via Tauri dev tools:
   ```javascript
   // In browser console
   await __TAURI__.core.invoke('init_audio');
   await __TAURI__.core.invoke('play_chord', {
     chord: 'C',
     voicingStyle: 'lead',
     baseOctave: 2,
     isFinal: false
   });
   ```

### Troubleshooting

**No sound plays**:
- Check: Are samples in `src-tauri/resources/samples/`?
- Check: Did build show "Generated X embedded audio samples" (X > 0)?
- Check: Console for audio initialization errors

**Clicks/pops on playback**:
- Ensure `is_final` parameter is used correctly
- Check that `detach()` is used, not `stop()` in engine.rs
- Verify envelope is applied to all sources

**Voice leading doesn't work**:
- Call `reset_voicing()` before starting new progression
- Check that `voicingStyle: 'lead'` is specified
- Verify music theory engine is returning correct notes

## Performance Characteristics

- **Startup Time**: Lazy init on first play (~50-100ms)
- **Latency**: ~10-30ms from command to sound (hardware dependent)
- **Memory**: ~2-5 MB for embedded samples
- **CPU**: Minimal (<1% on modern hardware)
- **Thread Count**: +1 dedicated audio thread

## Known Limitations

1. **Sample range**: Only A1-C5 supported (49 notes)
2. **Format**: Only OGG Vorbis (not MP3 or WAV)
3. **Mobile**: iOS requires user interaction before first play (Tauri limitation)
4. **No streaming**: All samples must fit in memory

## Future Improvements

Potential enhancements (not yet implemented):

- [ ] Configurable sample rate conversion
- [ ] Reverb/delay effects
- [ ] MIDI-based synthesis (no samples needed)
- [ ] Pitch shifting for missing samples
- [ ] Compressed sample format for smaller binaries

---

## References

- **Canon Project**: Original audio architecture source
- **Rodio Documentation**: https://docs.rs/rodio/
- **Tauri State Management**: https://tauri.app/v1/guides/features/state-management
- **Voice Leading Theory**: See `src-tauri/src/music/voice_leading.rs`

## Credits

Audio engine architecture adapted from the [Canon](https://github.com/user/canon) chord progression tool, which demonstrates production-ready audio playback patterns for music applications.
