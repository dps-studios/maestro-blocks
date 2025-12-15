# Piano Audio Samples

This directory should contain piano samples in OGG Vorbis format for the audio engine.

## Required Samples

The audio engine expects 49 piano samples covering the range A1 to C5:

### File Naming Convention
- Format: `{Note}{Octave}_bip.ogg`
- Examples: `C3_bip.ogg`, `Cs4_bip.ogg`, `As2_bip.ogg`
- Sharp notes use 's' suffix: C# = Cs, F# = Fs, etc.

### Complete List of Required Files

**Octave 1**: A1_bip.ogg, As1_bip.ogg, B1_bip.ogg
**Octave 2**: C2_bip.ogg, Cs2_bip.ogg, D2_bip.ogg, Ds2_bip.ogg, E2_bip.ogg, F2_bip.ogg, Fs2_bip.ogg, G2_bip.ogg, Gs2_bip.ogg, A2_bip.ogg, As2_bip.ogg, B2_bip.ogg
**Octave 3**: C3_bip.ogg, Cs3_bip.ogg, D3_bip.ogg, Ds3_bip.ogg, E3_bip.ogg, F3_bip.ogg, Fs3_bip.ogg, G3_bip.ogg, Gs3_bip.ogg, A3_bip.ogg, As3_bip.ogg, B3_bip.ogg
**Octave 4**: C4_bip.ogg, Cs4_bip.ogg, D4_bip.ogg, Ds4_bip.ogg, E4_bip.ogg, F4_bip.ogg, Fs4_bip.ogg, G4_bip.ogg, Gs4_bip.ogg, A4_bip.ogg, As4_bip.ogg, B4_bip.ogg
**Octave 5**: C5_bip.ogg

### Optional Sound Effects
- `swoosh.ogg` - UI sound effect

## Sample Sources

You can:
1. Copy samples from the Canon project at `/tmp/canon-analysis/src-tauri/resources/samples/`
2. Generate your own using a sampler or synthesizer
3. Use royalty-free piano samples from online sources

## Audio Requirements

- **Format**: OGG Vorbis
- **Sample Rate**: 44.1kHz or 48kHz recommended
- **Bit Depth**: 16-bit minimum
- **Duration**: 2-3 seconds with natural decay
- **Dynamics**: Clean piano tone without excessive compression

## Build Integration

The `build.rs` script automatically embeds these samples into the binary at compile time. The samples are loaded via `include_bytes!()` and stored in a static HashMap for fast access.

If samples are missing, the build will fail with a clear error message indicating which files are needed.
