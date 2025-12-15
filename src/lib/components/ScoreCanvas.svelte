<script lang="ts">
  import { onMount } from 'svelte';
  import { scoreStore, editorStore } from '../stores/score';
  import { 
    renderSection, 
    yPositionToPitch,
    pitchToYPosition,
    xPositionToMeasure,
    type StaffCoordinates,
  } from '../services/vexflow';
  import type { WorksheetSection, Pitch } from '../types/score';

  // Reactive state
  let isLoading = $state(true);
  let staffCoordinates = $state<Map<string, StaffCoordinates>>(new Map());
  
  // Ghost note state - pixel-perfect positioning
  let ghostNote = $state<{ 
    x: number;           // Pixel X position (centered in measure)
    y: number;           // Pixel Y position (snapped to staff line/space)
    pitch: Pitch;
    measureIndex: number;
    sectionId: string;
  } | null>(null);

  // Initialize on mount
  onMount(() => {
    isLoading = false;
    console.log('[ScoreCanvas] VexFlow renderer ready');
  });

  // Reactive store subscriptions
  let sections = $derived($scoreStore.sections);
  let keyFifths = $derived($scoreStore.keySignature.fifths);
  let timeSignature = $derived($scoreStore.timeSignature);
  let showAnswers = $derived($scoreStore.showAnswers);
  let activeTool = $derived($editorStore.activeTool);

  // Re-render when score changes
  $effect(() => {
    if (!isLoading && sections.length > 0) {
      // Delay to ensure DOM is ready
      setTimeout(() => renderAllSections(), 0);
    }
  });

  function renderAllSections() {
    console.log('[ScoreCanvas] Rendering', sections.length, 'sections');
    
    const containers = getStaffContainers();
    
    for (const section of sections) {
      const container = containers.get(section.id);
      if (!container) {
        console.warn('[ScoreCanvas] No container for section:', section.id);
        continue;
      }
      
      // Clear previous content
      container.innerHTML = '';
      
      try {
        const result = renderSection(section, showAnswers, timeSignature, keyFifths);
        container.appendChild(result.svg);
        staffCoordinates.set(section.id, result.coordinates);
        
        console.log('[ScoreCanvas] Rendered section', section.id, 'coords:', result.coordinates);
      } catch (error) {
        console.error('[ScoreCanvas] Error rendering section:', section.id, error);
      }
    }
  }

  function getStaffContainers() {
    // Query all VexFlow containers by data attribute
    const containers = document.querySelectorAll<HTMLDivElement>('.vexflow-container[data-section-id]');
    const map = new Map<string, HTMLDivElement>();
    containers.forEach(el => {
      const id = el.getAttribute('data-section-id');
      if (id) map.set(id, el);
    });
    return map;
  }

  // Handle staff click - place chord at ghost note position
  async function handleStaffClick(event: MouseEvent, section: WorksheetSection) {
    if (activeTool.type !== 'chord') return;
    if (!ghostNote || ghostNote.sectionId !== section.id) return;

    const measure = section.staff.measures[ghostNote.measureIndex];
    if (!measure) return;

    console.log('[ScoreCanvas] === CHORD PLACEMENT ===');
    console.log('[ScoreCanvas] Click position:', { x: event.offsetX, y: event.offsetY });
    console.log('[ScoreCanvas] Ghost note:', ghostNote);
    console.log('[ScoreCanvas] Measure index:', ghostNote.measureIndex);
    console.log('[ScoreCanvas] Pitch:', ghostNote.pitch);
    console.log('[ScoreCanvas] Quality:', activeTool.quality);

    const chordDef = {
      root: ghostNote.pitch.note,
      rootAccidental: ghostNote.pitch.accidental,
      quality: activeTool.quality,
      inversion: 'root' as const,
    };

    console.log('[ScoreCanvas] Chord definition:', chordDef);

    try {
      const elementId = await scoreStore.addChordToMeasure(
        section.id,
        measure.id,
        chordDef,
        ghostNote.pitch.octave,
        { value: 1, dots: 0 }
      );
      console.log('[ScoreCanvas] Chord added successfully, element ID:', elementId);
    } catch (error) {
      console.error('[ScoreCanvas] Failed to add chord:', error);
    }
  }

  // Handle mouse move - update ghost note position with pixel-perfect accuracy
  function handleMouseMove(event: MouseEvent, section: WorksheetSection) {
    if (activeTool.type !== 'chord') {
      ghostNote = null;
      return;
    }

    const coords = staffCoordinates.get(section.id);
    if (!coords || coords.measureBounds.length === 0) return;

    // Get the staff-container (event target) and the SVG element
    const staffContainer = event.currentTarget as HTMLElement;
    const svg = staffContainer.querySelector('svg');
    if (!svg) return;

    // Get bounding rects for coordinate transformation
    const staffContainerRect = staffContainer.getBoundingClientRect();
    const svgRect = svg.getBoundingClientRect();
    
    // Calculate scale factors (SVG internal coords vs rendered size)
    const scaleX = coords.totalWidth / svgRect.width;
    const scaleY = coords.totalHeight / svgRect.height;

    // Convert mouse position to SVG coordinate space
    const mouseXInSvg = event.clientX - svgRect.left;
    const mouseYInSvg = event.clientY - svgRect.top;
    const svgX = mouseXInSvg * scaleX;
    const svgY = mouseYInSvg * scaleY;

    // Determine which system (staff line) the mouse is on based on Y position
    // Find the system whose vertical center is closest to the mouse Y
    let targetSystemIndex = 0;
    if (coords.numSystems > 1) {
      const systemTotalHeight = coords.systemHeight + coords.systemSpacing;
      // Calculate which system based on Y, snapping at halfway point between systems
      targetSystemIndex = Math.floor((svgY + coords.systemSpacing / 2) / systemTotalHeight);
      targetSystemIndex = Math.max(0, Math.min(targetSystemIndex, coords.numSystems - 1));
    }

    // Find measures that belong to this system
    const systemMeasures = coords.measureBounds
      .map((bound, index) => ({ bound, index }))
      .filter(({ bound }) => bound.systemIndex === targetSystemIndex);
    
    if (systemMeasures.length === 0) return;

    // Find which measure within this system based on X position
    let measureIndex = systemMeasures[0].index;
    for (const { bound, index } of systemMeasures) {
      if (svgX >= bound.startX && svgX < bound.endX) {
        measureIndex = index;
        break;
      }
      // Default to last measure if past the end
      if (svgX >= bound.endX) {
        measureIndex = index;
      }
    }

    const measureBound = coords.measureBounds[measureIndex];
    
    // Get pitch from Y position using this system's staff coordinates
    const systemCoords = {
      ...coords,
      staffTopY: measureBound.staffTopY,
      staffBottomY: measureBound.staffBottomY,
    };
    const pitch = yPositionToPitch(svgY, systemCoords, section.staff.clef);
    
    // Calculate snapped Y position on this system's staff
    const snappedSvgX = measureBound.startX + 20;
    const snappedSvgY = pitchToYPosition(pitch, systemCoords, section.staff.clef);
    
    // Convert SVG coords back to screen coords relative to staff-container
    const svgOffsetX = svgRect.left - staffContainerRect.left;
    const svgOffsetY = svgRect.top - staffContainerRect.top;
    
    const screenX = (snappedSvgX / scaleX) + svgOffsetX;
    const screenY = (snappedSvgY / scaleY) + svgOffsetY;

    ghostNote = {
      x: screenX,
      y: screenY,
      pitch,
      measureIndex,
      sectionId: section.id,
    };
  }

  function handleMouseLeave() {
    ghostNote = null;
  }

  // Format pitch for display
  function formatPitch(pitch: Pitch): string {
    let str = pitch.note.toUpperCase();
    if (pitch.accidental === 'sharp') str += '#';
    else if (pitch.accidental === 'flat') str += 'b';
    return `${str}${pitch.octave}`;
  }
</script>

<div class="score-canvas paper-texture">
  {#if isLoading}
    <div class="loading-state">
      <div class="spinner spinner-lg"></div>
      <p class="loading-text">Loading notation engine...</p>
    </div>
  {:else if sections.length === 0}
    <div class="empty-state">
      <div class="empty-icon">
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round">
          <path d="M9 18V5l12-2v13"/>
          <circle cx="6" cy="18" r="3"/>
          <circle cx="18" cy="16" r="3"/>
        </svg>
      </div>
      <h2 class="empty-title">No worksheet sections yet</h2>
      <p class="empty-description">Add a chord naming section from the sidebar to get started.</p>
    </div>
  {:else}
    <div class="sections">
      {#each sections as section (section.id)}
        <div class="section">
          {#if section.title}
            <h3 class="section-title">{section.title}</h3>
          {/if}
          {#if section.instructions}
            <p class="section-instructions">{section.instructions}</p>
          {/if}

          <div
            class="staff-container"
            class:chord-mode={activeTool.type === 'chord'}
            role="application"
            tabindex="0"
            aria-label="Music staff - click to place chords"
            onclick={(e) => handleStaffClick(e, section)}
            onmousemove={(e) => handleMouseMove(e, section)}
            onmouseleave={handleMouseLeave}
          >
            <!-- VexFlow SVG renders here -->
            <div 
              class="vexflow-container"
              data-section-id={section.id}
            ></div>
            
            <!-- Ghost note overlay (pixel-perfect positioned) -->
            {#if ghostNote && ghostNote.sectionId === section.id && activeTool.type === 'chord'}
              <div 
                class="ghost-note"
                style="left: {ghostNote.x}px; top: {ghostNote.y}px;"
              >
                <div class="ghost-note-head"></div>
                <span class="ghost-note-label">{formatPitch(ghostNote.pitch)}</span>
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .score-canvas {
    flex: 1;
    background-color: var(--color-paper);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-md);
    overflow: auto;
    padding: var(--space-6);
    position: relative;
  }

  /* Loading State */
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    min-height: 400px;
    gap: var(--space-4);
  }

  .loading-text {
    margin: 0;
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    color: var(--color-ink-muted);
  }

  /* Empty State */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    min-height: 400px;
    text-align: center;
    padding: var(--space-8);
  }

  .empty-icon {
    color: var(--color-accent-line);
    margin-bottom: var(--space-4);
  }

  .empty-title {
    margin: 0 0 var(--space-2);
    font-family: var(--font-serif);
    font-size: var(--text-xl);
    font-weight: var(--font-normal);
    color: var(--color-ink);
  }

  .empty-description {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--color-ink-muted);
    max-width: 300px;
  }

  /* Sections - Paper-like worksheet layout */
  .sections {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-6);
  }

  /* Section styled as US Letter paper (8.5:11 aspect ratio) */
  .section {
    background-color: #FFFFFF;
    box-shadow: 
      0 1px 3px rgba(0, 0, 0, 0.12),
      0 4px 12px rgba(0, 0, 0, 0.08);
    width: 100%;
    max-width: 816px;  /* US Letter width at 96 DPI */
    min-height: 400px;
    padding: 48px 56px; /* ~0.5" margins scaled */
  }

  .section-title {
    margin: 0 0 4px;
    font-family: var(--font-serif);
    font-size: 24px;
    font-weight: 600;
    color: #1a1a1a;
    text-align: center;
  }

  .section-instructions {
    margin: 0 0 32px;
    font-family: var(--font-sans);
    font-size: 14px;
    color: #666666;
    text-align: center;
  }

  /* Staff Container - Clean notation area */
  .staff-container {
    position: relative;
    background-color: #FFFFFF;
    min-height: 140px;
    overflow: hidden; /* Prevent any staff line bleeding */
  }

  .staff-container.chord-mode {
    cursor: crosshair;
  }

  .staff-container:focus-visible {
    outline: 2px solid var(--color-accent-gold);
    outline-offset: 2px;
  }

  .vexflow-container {
    width: 100%;
    display: flex;
    justify-content: center;
  }

  .vexflow-container :global(svg) {
    display: block;
    max-width: 100%;
    height: auto;
    margin: 0 auto;
  }

  /* Ghost Note - Pixel Perfect
   * The note head is positioned exactly at the target coordinates.
   * The container uses translate(-50%, 0) to center horizontally only.
   * The note head is vertically centered on the target Y via negative margin.
   */
  .ghost-note {
    position: absolute;
    pointer-events: none;
    transform: translateX(-50%); /* Center horizontally only */
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .ghost-note-head {
    width: 14px;
    height: 10px;
    background-color: var(--color-accent-gold);
    border-radius: 50%;
    transform: rotate(-20deg);
    box-shadow: 
      0 0 0 2px rgba(184, 149, 108, 0.4),
      0 0 12px 2px rgba(184, 149, 108, 0.3);
    opacity: 0.9;
    /* Offset upward by half the note head height to center on staff line */
    margin-top: -5px;
  }

  .ghost-note-label {
    font-family: var(--font-sans);
    font-size: 10px;
    font-weight: var(--font-semibold);
    color: var(--color-accent-gold);
    background-color: rgba(255, 255, 255, 0.95);
    padding: 1px 4px;
    border-radius: var(--radius-sm);
    white-space: nowrap;
    margin-top: 2px;
  }

  :root.dark .ghost-note-head {
    box-shadow: 
      0 0 0 2px rgba(201, 168, 108, 0.5),
      0 0 12px 2px rgba(201, 168, 108, 0.4);
  }

  /* Spinner */
  .spinner {
    border: 2px solid var(--color-accent-line);
    border-top-color: var(--color-ink);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .spinner-lg {
    width: 40px;
    height: 40px;
    border-width: 3px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
