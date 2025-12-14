<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  const commonChords = [
    'C', 'Dm', 'Em', 'F', 'G', 'Am', 'Bdim',
    'Cmaj7', 'Dm7', 'Em7', 'Fmaj7', 'G7', 'Am7', 'Bm7b5',
    'Cmaj9', 'Dm9', 'Em9', 'Fmaj9', 'G9', 'Am9'
  ];
  
  let draggedChord: string | null = null;
  
  function handleDragStart(event: DragEvent, chord: string) {
    draggedChord = chord;
    if (event.dataTransfer) {
      event.dataTransfer.setData('text/plain', chord);
      event.dataTransfer.effectAllowed = 'copy';
    }
  }
  
  function handleDragEnd() {
    draggedChord = null;
  }
</script>

<div class="chord-palette">
  <h3>Chord Palette</h3>
  <div class="chord-grid">
    {#each commonChords as chord}
      <div 
        class="chord-item"
        class:dragging={draggedChord === chord}
        draggable={true}
        role="button"
        tabindex="0"
        aria-label={`Chord: ${chord}`}
        on:dragstart={(e) => handleDragStart(e, chord)}
        on:dragend={handleDragEnd}
      >
        {chord}
      </div>
    {/each}
  </div>
</div>

<style>
  .chord-palette {
    padding: 16px;
    background: #f5f5f5;
    border: 1px solid #ddd;
    border-radius: 8px;
    min-width: 200px;
  }
  
  .chord-palette h3 {
    margin: 0 0 12px 0;
    font-size: 16px;
    color: #333;
  }
  
  .chord-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(60px, 1fr));
    gap: 8px;
  }
  
  .chord-item {
    padding: 8px 12px;
    background: white;
    border: 1px solid #ccc;
    border-radius: 4px;
    text-align: center;
    font-size: 12px;
    cursor: grab;
    transition: all 0.2s ease;
    user-select: none;
  }
  
  .chord-item:hover {
    background: #007acc;
    color: white;
    border-color: #005a9e;
    transform: translateY(-1px);
  }
  
  .chord-item.dragging {
    opacity: 0.5;
    cursor: grabbing;
  }
</style>