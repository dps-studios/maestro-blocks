/**
 * Chord Naming Worksheet Generator
 * Interactive interface for creating chord identification worksheets with real-time updates
 */

import { createSignal, onMount, onCleanup, For, Show } from 'solid-js';
import { worksheetStore } from '../stores/worksheet';
import WorksheetViewer from './WorksheetViewer';
import type { WorksheetConfig } from '../types/worksheet';
import { debounce, WorksheetUpdateManager } from '../utils/real-time-updates';

type ChordQualityType = 'major' | 'minor' | 'diminished' | 'augmented' | 'dominant7' | 'major7' | 'minor7';

interface ChordEntry {
  root: string;
  quality: ChordQualityType;
  showAnswer: boolean;
}

export default function ChordNamingGenerator() {
  // Form state
  const [title, setTitle] = createSignal('Chord Identification Worksheet');
  const [instructions, setInstructions] = createSignal('Identify the following chords');
  const [chordsPerLine, setChordsPerLine] = createSignal(4);
  const [showAnswersState, setShowAnswersState] = createSignal(false);

  // Chord list
  const [chords, setChords] = createSignal<ChordEntry[]>([
    { root: 'C', quality: 'major', showAnswer: true },
    { root: 'G', quality: 'major', showAnswer: true },
    { root: 'D', quality: 'minor', showAnswer: true },
    { root: 'A', quality: 'minor', showAnswer: true },
    { root: 'F', quality: 'major', showAnswer: true },
    { root: 'C', quality: 'dominant7', showAnswer: true },
    { root: 'G', quality: 'dominant7', showAnswer: true },
    { root: 'D', quality: 'minor7', showAnswer: true },
  ]);

  const [currentWorksheet, setCurrentWorksheet] = createSignal<WorksheetConfig | null>(null);
  const [generating, setGenerating] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  let updateManager: WorksheetUpdateManager;

  // Debounced worksheet generation
  const debouncedGenerate = debounce(async () => {
    await generateWorksheet();
  }, 500);

  async function generateWorksheet() {
    if (generating()) return;

    setGenerating(true);
    setError(null);

    try {
      const chordParams = {
        title: title(),
        chords: chords().map((chord, index) => ({
          root: chord.root,
          quality: chord.quality,
          position: {
            measure: Math.floor(index / chordsPerLine()) + 1,
            beat: (index % chordsPerLine()) + 1
          },
          showAnswer: chord.showAnswer
        })),
        instructions: instructions(),
        layout: {
          chordsPerLine: chordsPerLine(),
          showStaffLines: true
        }
      };

      const worksheet = await worksheetStore.generateFromTemplate('chord-naming', chordParams);
      if (worksheet) {
        worksheetStore.updateConfig({
          globalSettings: {
            ...worksheet.globalSettings,
            showAnswers: showAnswersState()
          }
        });

        setCurrentWorksheet(worksheet);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to generate worksheet');
      console.error('Failed to generate worksheet:', err);
    } finally {
      setGenerating(false);
    }
  }

  function scheduleWorksheetUpdate(updateId: string) {
    updateManager.scheduleUpdate(updateId, debouncedGenerate);
  }

  function addChord() {
    setChords([...chords(), {
      root: 'C',
      quality: 'major',
      showAnswer: true
    }]);
    scheduleWorksheetUpdate('add-chord');
  }

  function removeChord(index: number) {
    setChords(chords().filter((_, i) => i !== index));
    scheduleWorksheetUpdate('remove-chord');
  }

  function updateChord(index: number, field: keyof ChordEntry, value: string | boolean) {
    setChords(chords().map((chord, i) =>
      i === index ? { ...chord, [field]: value } : chord
    ));
    scheduleWorksheetUpdate(`update-chord-${index}`);
  }

  onMount(() => {
    updateManager = new WorksheetUpdateManager();
    generateWorksheet();
  });

  onCleanup(() => {
    if (updateManager) {
      updateManager.cancelAll();
    }
  });

  return (
    <div class="chord-generator">
      <div class="controls">
        <h2>Chord Naming Worksheet Generator</h2>

        <div class="form-section">
          <h3>Worksheet Settings</h3>

          <div class="form-group">
            <label for="title">Title:</label>
            <input
              id="title"
              type="text"
              value={title()}
              onInput={(e) => {
                setTitle(e.currentTarget.value);
                scheduleWorksheetUpdate('title');
              }}
              placeholder="Worksheet title"
            />
          </div>

          <div class="form-group">
            <label for="instructions">Instructions:</label>
            <textarea
              id="instructions"
              value={instructions()}
              onInput={(e) => {
                setInstructions(e.currentTarget.value);
                scheduleWorksheetUpdate('instructions');
              }}
              placeholder="Instructions for students"
              rows="3"
            />
          </div>

          <div class="form-group">
            <label for="chordsPerLine">Chords per line:</label>
            <input
              id="chordsPerLine"
              type="number"
              value={chordsPerLine()}
              onInput={(e) => {
                setChordsPerLine(parseInt(e.currentTarget.value) || 4);
                scheduleWorksheetUpdate('chordsPerLine');
              }}
              min="1"
              max="8"
            />
          </div>

          <div class="form-group">
            <label>
              <input
                type="checkbox"
                checked={showAnswersState()}
                onChange={(e) => {
                  setShowAnswersState(e.currentTarget.checked);
                  scheduleWorksheetUpdate('showAnswers');
                }}
              />
              Show answers
            </label>
          </div>
        </div>

        <div class="form-section">
          <h3>Chords</h3>

          <div class="chords-list">
            <For each={chords()}>
              {(chord, index) => (
                <div class="chord-item">
                  <select
                    value={chord.root}
                    onChange={(e) => updateChord(index(), 'root', e.currentTarget.value)}
                  >
                    <option value="C">C</option>
                    <option value="C#">C#</option>
                    <option value="Db">Db</option>
                    <option value="D">D</option>
                    <option value="D#">D#</option>
                    <option value="Eb">Eb</option>
                    <option value="E">E</option>
                    <option value="F">F</option>
                    <option value="F#">F#</option>
                    <option value="Gb">Gb</option>
                    <option value="G">G</option>
                    <option value="G#">G#</option>
                    <option value="Ab">Ab</option>
                    <option value="A">A</option>
                    <option value="A#">A#</option>
                    <option value="Bb">Bb</option>
                    <option value="B">B</option>
                  </select>

                  <select
                    value={chord.quality}
                    onChange={(e) => updateChord(index(), 'quality', e.currentTarget.value)}
                  >
                    <option value="major">Major</option>
                    <option value="minor">Minor</option>
                    <option value="diminished">Diminished</option>
                    <option value="augmented">Augmented</option>
                    <option value="dominant7">Dominant 7</option>
                    <option value="major7">Major 7</option>
                    <option value="minor7">Minor 7</option>
                  </select>

                  <label class="checkbox-label">
                    <input
                      type="checkbox"
                      checked={chord.showAnswer}
                      onChange={(e) => updateChord(index(), 'showAnswer', e.currentTarget.checked)}
                    />
                    Show Answer
                  </label>

                  <button
                    class="remove-btn"
                    onClick={() => removeChord(index())}
                    disabled={chords().length <= 1}
                  >
                    Remove
                  </button>
                </div>
              )}
            </For>
          </div>

          <button class="add-btn" onClick={addChord}>
            Add Chord
          </button>
        </div>

        <div class="actions">
          <button
            class="generate-btn"
            onClick={generateWorksheet}
            disabled={generating()}
          >
            {generating() ? 'Generating...' : 'Generate Worksheet'}
          </button>
        </div>
      </div>

      <div class="preview">
        <h3>Worksheet Preview</h3>

        <Show when={error()}>
          <div class="error-banner">
            <p><strong>Error:</strong> {error()}</p>
            <button onClick={generateWorksheet}>Retry</button>
          </div>
        </Show>

        <div class={`worksheet-container ${generating() ? 'loading' : ''}`}>
          <Show when={generating()}>
            <div class="loading-overlay">
              <div class="spinner"></div>
              <p>Generating worksheet...</p>
            </div>
          </Show>

          <WorksheetViewer
            worksheet={currentWorksheet()}
            width={600}
            height={800}
          />
        </div>
      </div>
    </div>
  );
}
