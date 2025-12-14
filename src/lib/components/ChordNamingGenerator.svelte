<!-- 
  Chord Naming Worksheet Generator
  Interactive interface for creating chord identification worksheets with real-time updates
-->

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { worksheetStore, createChordElement } from '../stores/worksheet';
  import WorksheetViewer from './WorksheetViewer.svelte';
  import type { WorksheetConfig } from '../types/worksheet';
  import { debounce, WorksheetUpdateManager } from '../utils/real-time-updates';
  
  // Form state
  let title = 'Chord Identification Worksheet';
  let instructions = 'Identify the following chords';
  let chordsPerLine = 4;
  let showStaffLines = true;
  let showAnswers = false;
  
  // Chord list
  let chords = [
    { root: 'C', quality: 'major' as const, showAnswer: true },
    { root: 'G', quality: 'major' as const, showAnswer: true },
    { root: 'D', quality: 'minor' as const, showAnswer: true },
    { root: 'A', quality: 'minor' as const, showAnswer: true },
    { root: 'F', quality: 'major' as const, showAnswer: true },
    { root: 'C', quality: 'dominant7' as const, showAnswer: true },
    { root: 'G', quality: 'dominant7' as const, showAnswer: true },
    { root: 'D', quality: 'minor7' as const, showAnswer: true },
  ];
  
  let currentWorksheet: WorksheetConfig | null = null;
  let generating = false;
  let error: string | null = null;
  
  // Real-time update manager
  let updateManager: WorksheetUpdateManager;
  
  // Debounced worksheet generation
  const debouncedGenerate = debounce(async () => {
    await generateWorksheet();
  }, 500);
  
  async function generateWorksheet() {
    if (generating) return; // Prevent concurrent generation
    
    generating = true;
    error = null;
    
    try {
      // Convert chords to the format expected by the backend
      const chordParams = {
        title,
        chords: chords.map((chord, index) => ({
          root: chord.root,
          quality: chord.quality,
          position: {
            measure: Math.floor(index / chordsPerLine) + 1,
            beat: (index % chordsPerLine) + 1
          },
          showAnswer: chord.showAnswer
        })),
        instructions,
        layout: {
          chordsPerLine,
          showStaffLines
        }
      };
      
      const worksheet = await worksheetStore.generateFromTemplate('chord-naming', chordParams);
      if (worksheet) {
        // Update global settings
        worksheetStore.updateConfig({
          globalSettings: {
            ...worksheet.globalSettings,
            showAnswers
          }
        });
        
        currentWorksheet = worksheet;
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to generate worksheet';
      console.error('Failed to generate worksheet:', err);
    } finally {
      generating = false;
    }
  }
  
  // Real-time update functions
  function scheduleWorksheetUpdate(updateId: string) {
    updateManager.scheduleUpdate(updateId, debouncedGenerate);
  }
  
  function addChord() {
    chords = [...chords, {
      root: 'C',
      quality: 'major' as const,
      showAnswer: true
    }];
    scheduleWorksheetUpdate('add-chord');
  }
  
  function removeChord(index: number) {
    chords = chords.filter((_, i) => i !== index);
    scheduleWorksheetUpdate('remove-chord');
  }
  
  function updateChord(index: number, field: string, value: any) {
    chords = chords.map((chord, i) => 
      i === index ? { ...chord, [field]: value } : chord
    );
    scheduleWorksheetUpdate(`update-chord-${index}`);
  }
  
  function updateTitle() {
    scheduleWorksheetUpdate('title');
  }
  
  function updateInstructions() {
    scheduleWorksheetUpdate('instructions');
  }
  
  function updateChordsPerLine() {
    scheduleWorksheetUpdate('chordsPerLine');
  }
  
  function updateShowAnswers() {
    scheduleWorksheetUpdate('showAnswers');
  }
  
  // Initialize update manager and generate initial worksheet
  onMount(() => {
    updateManager = new WorksheetUpdateManager();
    generateWorksheet();
  });
  
  // Clean up on destroy
  onDestroy(() => {
    if (updateManager) {
      updateManager.cancelAll();
    }
  });
</script>

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
          bind:value={title}
          on:input={updateTitle}
          placeholder="Worksheet title"
        />
      </div>
      
      <div class="form-group">
        <label for="instructions">Instructions:</label>
        <textarea 
          id="instructions"
          bind:value={instructions}
          on:input={updateInstructions}
          placeholder="Instructions for students"
          rows="3"
        ></textarea>
      </div>
      
      <div class="form-group">
        <label for="chordsPerLine">Chords per line:</label>
        <input 
          id="chordsPerLine"
          type="number" 
          bind:value={chordsPerLine}
          on:input={updateChordsPerLine}
          min="1"
          max="8"
        />
      </div>
      
      <div class="form-group">
        <label>
          <input 
            type="checkbox" 
            bind:checked={showAnswers}
            on:change={updateShowAnswers}
          />
          Show answers
        </label>
      </div>
    </div>
    
    <div class="form-section">
      <h3>Chords</h3>
      
      <div class="chords-list">
        {#each chords as chord, index}
          <div class="chord-item">
            <select 
              bind:value={chord.root}
              on:change={() => updateChord(index, 'root', chord.root)}
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
              bind:value={chord.quality}
              on:change={() => updateChord(index, 'quality', chord.quality)}
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
                bind:checked={chord.showAnswer}
                on:change={() => updateChord(index, 'showAnswer', chord.showAnswer)}
              />
              Show Answer
            </label>
            
            <button 
              class="remove-btn"
              on:click={() => removeChord(index)}
              disabled={chords.length <= 1}
            >
              Remove
            </button>
          </div>
        {/each}
      </div>
      
      <button class="add-btn" on:click={addChord}>
        Add Chord
      </button>
    </div>
    
    <div class="actions">
      <button 
        class="generate-btn"
        on:click={generateWorksheet}
        disabled={generating}
      >
        {generating ? 'Generating...' : 'Generate Worksheet'}
      </button>
    </div>
  </div>
  
  <div class="preview">
    <h3>Worksheet Preview</h3>
    
    {#if error}
      <div class="error-banner">
        <p><strong>Error:</strong> {error}</p>
        <button on:click={generateWorksheet}>Retry</button>
      </div>
    {/if}
    
    <div class="worksheet-container" class:loading={generating}>
      {#if generating}
        <div class="loading-overlay">
          <div class="spinner"></div>
          <p>Generating worksheet...</p>
        </div>
      {/if}
      
      <WorksheetViewer 
        worksheet={currentWorksheet}
        width={600}
        height={800}
      />
    </div>
  </div>
</div>

<style>
  .chord-generator {
    display: grid;
    grid-template-columns: 400px 1fr;
    gap: 20px;
    height: 100vh;
    padding: 20px;
  }
  
  .controls {
    overflow-y: auto;
    border-right: 1px solid #e9ecef;
    padding-right: 20px;
  }
  
  .preview {
    overflow-y: auto;
    padding-left: 20px;
  }
  
  h2 {
    margin-top: 0;
    color: #333;
  }
  
  h3 {
    margin-bottom: 16px;
    color: #555;
    font-size: 16px;
  }
  
  .form-section {
    margin-bottom: 24px;
    padding: 16px;
    background: #f8f9fa;
    border-radius: 8px;
  }
  
  .form-group {
    margin-bottom: 16px;
  }
  
  .form-group label {
    display: block;
    margin-bottom: 4px;
    font-weight: 500;
    color: #333;
  }
  
  .form-group input[type="text"],
  .form-group textarea,
  .form-group input[type="number"] {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 14px;
  }
  
  .form-group input[type="checkbox"] {
    margin-right: 8px;
  }
  
  .chords-list {
    margin-bottom: 16px;
  }
  
  .chord-item {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-bottom: 8px;
    padding: 8px;
    background: white;
    border-radius: 4px;
    border: 1px solid #e9ecef;
  }
  
  .chord-item select {
    padding: 4px 8px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 14px;
  }
  
  .checkbox-label {
    display: flex;
    align-items: center;
    font-size: 14px;
    white-space: nowrap;
  }
  
  .remove-btn {
    padding: 4px 8px;
    background: #dc3545;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }
  
  .remove-btn:hover:not(:disabled) {
    background: #c82333;
  }
  
  .remove-btn:disabled {
    background: #6c757d;
    cursor: not-allowed;
  }
  
  .add-btn {
    padding: 8px 16px;
    background: #28a745;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 14px;
    cursor: pointer;
  }
  
  .add-btn:hover {
    background: #218838;
  }
  
  .actions {
    margin-top: 24px;
  }
  
  .generate-btn {
    width: 100%;
    padding: 12px 24px;
    background: #007bff;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 16px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }
  
  .generate-btn:hover:not(:disabled) {
    background: #0056b3;
  }
  
  .generate-btn:disabled {
    background: #6c757d;
    cursor: not-allowed;
  }
  
  .error-banner {
    background: #f8d7da;
    border: 1px solid #f5c6cb;
    color: #721c24;
    padding: 12px;
    border-radius: 4px;
    margin-bottom: 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .error-banner button {
    background: #721c24;
    color: white;
    border: none;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }
  
  .worksheet-container {
    position: relative;
  }
  
  .loading-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(255, 255, 255, 0.9);
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    z-index: 10;
    border-radius: 8px;
  }
  
  .loading-overlay .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid #f3f3f3;
    border-top: 3px solid #007bff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 12px;
  }
  
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
  
  .loading-overlay p {
    color: #666;
    font-size: 14px;
    margin: 0;
  }
</style>