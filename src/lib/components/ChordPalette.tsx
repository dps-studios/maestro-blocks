import { createSignal, For } from 'solid-js';

export default function ChordPalette() {
  const commonChords = [
    'C', 'Dm', 'Em', 'F', 'G', 'Am', 'Bdim',
    'Cmaj7', 'Dm7', 'Em7', 'Fmaj7', 'G7', 'Am7', 'Bm7b5',
    'Cmaj9', 'Dm9', 'Em9', 'Fmaj9', 'G9', 'Am9'
  ];

  const [draggedChord, setDraggedChord] = createSignal<string | null>(null);

  function handleDragStart(event: DragEvent, chord: string) {
    setDraggedChord(chord);
    if (event.dataTransfer) {
      event.dataTransfer.setData('text/plain', chord);
      event.dataTransfer.effectAllowed = 'copy';
    }
  }

  function handleDragEnd() {
    setDraggedChord(null);
  }

  return (
    <div class="chord-palette">
      <h3>Chord Palette</h3>
      <div class="chord-grid">
        <For each={commonChords}>
          {(chord) => (
            <div
              class={`chord-item ${draggedChord() === chord ? 'dragging' : ''}`}
              draggable={true}
              role="button"
              tabIndex={0}
              aria-label={`Chord: ${chord}`}
              onDragStart={(e) => handleDragStart(e, chord)}
              onDragEnd={handleDragEnd}
            >
              {chord}
            </div>
          )}
        </For>
      </div>
    </div>
  );
}
