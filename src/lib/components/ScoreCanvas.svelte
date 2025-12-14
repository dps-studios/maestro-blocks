<script lang="ts">
  import { onMount } from 'svelte';
  import { scoreStore, editorStore, renderSectionCached } from '../stores/score';
  import { initVerovio, pitchFromStaffPosition } from '../services/verovio';
  import type { WorksheetSection, Pitch, NoteName, ClefType } from '../types/score';

  // Reactive state
  let isVerovioReady = $state(false);
  let isLoading = $state(true);
  let loadError = $state<string | null>(null);
  let renderedSvgs = $state<Record<string, string>>({});
  let isRendering = $state(false);
  
  // Ghost note state - now uses percentage-based positioning
  let ghostNote = $state<{ 
    xPercent: number;  // 0-100% across the container
    yPercent: number;  // 0-100% down the container
    pitch: Pitch;
    measureIndex: number;
    sectionId: string;
  } | null>(null);

  // Initialize Verovio on mount
  onMount(() => {
    initializeVerovio();
  });

  async function initializeVerovio() {
    try {
      console.log('[ScoreCanvas] Initializing Verovio...');
      await initVerovio();
      console.log('[ScoreCanvas] Verovio ready');
      isVerovioReady = true;
      isLoading = false;
    } catch (error) {
      console.error('[ScoreCanvas] Failed to initialize Verovio:', error);
      loadError = error instanceof Error ? error.message : 'Unknown error';
      isLoading = false;
    }
  }

  // Render sections when score changes
  let sections = $derived($scoreStore.sections);
  let keyFifths = $derived($scoreStore.keySignature.fifths);
  let showAnswers = $derived($scoreStore.showAnswers);

  $effect(() => {
    const currentSections = sections;
    const currentKeyFifths = keyFifths;
    const currentShowAnswers = showAnswers;
    
    if (isVerovioReady && currentSections.length > 0) {
      setTimeout(() => {
        renderAllSections(currentSections, currentKeyFifths, currentShowAnswers);
      }, 0);
    }
  });

  async function renderAllSections(
    sectionsToRender: WorksheetSection[], 
    keyFifthsValue: number,
    showAnswersValue: boolean
  ) {
    if (isRendering) return;
    
    isRendering = true;

    try {
      const newSvgs: Record<string, string> = {};
      
      for (const section of sectionsToRender) {
        try {
          const svg = await renderSectionCached(section, keyFifthsValue, showAnswersValue);
          newSvgs[section.id] = svg;
        } catch (error) {
          console.error('[ScoreCanvas] Error rendering section:', section.id, error);
          newSvgs[section.id] = `<svg><text x="10" y="30" fill="red">Render error</text></svg>`;
        }
      }
      
      renderedSvgs = newSvgs;
    } catch (error) {
      console.error('[ScoreCanvas] Render error:', error);
    } finally {
      isRendering = false;
    }
  }

  let activeTool = $derived($editorStore.activeTool);

  // Layout constants for Verovio output at scale 100
  // These are approximate ratios based on typical Verovio rendering
  const STAFF_START_X_PERCENT = 8;   // Staff starts ~8% from left (after clef + time sig)
  const STAFF_TOP_Y_PERCENT = 30;    // Staff top line at ~30% from top (shifted up)
  const STAFF_BOTTOM_Y_PERCENT = 50; // Staff bottom line at ~50% from top
  const STAFF_HEIGHT_PERCENT = STAFF_BOTTOM_Y_PERCENT - STAFF_TOP_Y_PERCENT;

  // Calculate Y percentage for a pitch
  function calculateNoteYPercent(pitch: Pitch, clef: ClefType): number {
    const noteOrder: NoteName[] = ['c', 'd', 'e', 'f', 'g', 'a', 'b'];
    
    // Reference: top line of staff for each clef
    const clefTopLine: Record<ClefType, { note: NoteName; octave: number }> = {
      treble: { note: 'f', octave: 5 },
      bass: { note: 'a', octave: 3 },
      alto: { note: 'g', octave: 4 },
      tenor: { note: 'a', octave: 4 },
    };
    
    const ref = clefTopLine[clef];
    const refIndex = noteOrder.indexOf(ref.note) + ref.octave * 7;
    const pitchIndex = noteOrder.indexOf(pitch.note) + pitch.octave * 7;
    
    // Steps from top line (positive = below, negative = above)
    const stepsFromTop = refIndex - pitchIndex;
    
    // Staff has 4 spaces (8 half-steps visible), each step is 1/8 of staff height
    const stepPercent = STAFF_HEIGHT_PERCENT / 8;
    
    return STAFF_TOP_Y_PERCENT + (stepsFromTop * stepPercent);
  }

  async function handleStaffClick(_event: MouseEvent, section: WorksheetSection) {
    if (activeTool.type !== 'chord') return;
    if (!ghostNote || ghostNote.sectionId !== section.id) return;

    const measure = section.staff.measures[ghostNote.measureIndex];
    if (measure && activeTool.type === 'chord') {
      // Log the click details
      console.log('[ScoreCanvas] Click detected:', {
        measureIndex: ghostNote.measureIndex,
        measureId: measure.id,
        pitch: ghostNote.pitch,
        quality: activeTool.quality,
      });

      const chordDef = {
        root: ghostNote.pitch.note,
        rootAccidental: ghostNote.pitch.accidental,
        quality: activeTool.quality,
        inversion: 'root' as const,
      };

      console.log('[ScoreCanvas] Creating chord with definition:', chordDef);
      console.log('[ScoreCanvas] Root octave:', ghostNote.pitch.octave);

      // Use the octave from the ghost note position (root = bottom note of chord)
      const elementId = await scoreStore.addChordToMeasure(
        section.id,
        measure.id,
        chordDef,
        ghostNote.pitch.octave,
        { value: 1, dots: 0 }
      );

      console.log('[ScoreCanvas] Chord added with element ID:', elementId);
    }
  }

  function handleMouseMove(event: MouseEvent, section: WorksheetSection) {
    if (activeTool.type !== 'chord') {
      ghostNote = null;
      return;
    }

    const target = event.currentTarget as HTMLElement;
    const rect = target.getBoundingClientRect();
    const xPercent = ((event.clientX - rect.left) / rect.width) * 100;

    // Calculate pitch from Y position
    const staffTop = rect.height * (STAFF_TOP_Y_PERCENT / 100);
    const staffHeight = rect.height * (STAFF_HEIGHT_PERCENT / 100);
    const yInContainer = event.clientY - rect.top;
    const pitch = pitchFromStaffPosition(yInContainer, staffTop, staffHeight, section.staff.clef);

    // Calculate measure index from X position
    // Only trigger if we're in the staff area (past the clef/time signature)
    const measureAreaStart = STAFF_START_X_PERCENT;
    const measureAreaEnd = 98; // Right margin
    const measureAreaWidth = measureAreaEnd - measureAreaStart;
    const measureWidth = measureAreaWidth / section.staff.measures.length;
    
    // Determine which measure the mouse is over
    let measureIndex: number;
    if (xPercent < measureAreaStart) {
      measureIndex = 0; // Snap to first measure if before staff
    } else if (xPercent > measureAreaEnd) {
      measureIndex = section.staff.measures.length - 1; // Snap to last if past end
    } else {
      measureIndex = Math.floor((xPercent - measureAreaStart) / measureWidth);
      measureIndex = Math.max(0, Math.min(measureIndex, section.staff.measures.length - 1));
    }

    // Calculate ghost note position - ALWAYS centered in the hovered measure
    const noteXPercent = measureAreaStart + (measureIndex + 0.5) * measureWidth;
    const noteYPercent = calculateNoteYPercent(pitch, section.staff.clef);

    ghostNote = {
      xPercent: noteXPercent,
      yPercent: noteYPercent,
      pitch,
      measureIndex,
      sectionId: section.id,
    };
  }

  function handleMouseLeave() {
    ghostNote = null;
  }

  // Get measures that have chords (for underline rendering)
  function getMeasuresWithChords(section: WorksheetSection): number[] {
    return section.staff.measures
      .map((m, i) => ({ index: i, hasChord: m.elements.some(e => e.type === 'chord') }))
      .filter(m => m.hasChord)
      .map(m => m.index);
  }

  // Calculate underline positions (percentage-based)
  function getUnderlinePositions(section: WorksheetSection) {
    const measuresWithChords = getMeasuresWithChords(section);
    
    const measureAreaStart = STAFF_START_X_PERCENT;
    const measureAreaWidth = 100 - measureAreaStart - 2;
    const measureWidth = measureAreaWidth / section.staff.measures.length;
    
    const underlineYPercent = STAFF_BOTTOM_Y_PERCENT + 12; // Below staff
    const underlineWidthPercent = measureWidth * 0.4;
    
    return measuresWithChords.map(measureIndex => {
      const centerX = measureAreaStart + (measureIndex + 0.5) * measureWidth;
      return {
        leftPercent: centerX - underlineWidthPercent / 2,
        widthPercent: underlineWidthPercent,
        topPercent: underlineYPercent,
      };
    });
  }
</script>

<div class="score-canvas paper-texture">
  {#if isLoading}
    <div class="loading-state">
      <div class="spinner spinner-lg"></div>
      <p class="loading-text">Loading notation engine...</p>
    </div>
  {:else if loadError}
    <div class="error-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <p class="error-title">Failed to load notation engine</p>
      <code class="error-code">{loadError}</code>
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

          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
          <div
            class="staff-container"
            class:chord-mode={activeTool.type === 'chord'}
            onclick={(e) => handleStaffClick(e, section)}
            onkeydown={(e) => e.key === 'Enter' && handleStaffClick(e as unknown as MouseEvent, section)}
            onmousemove={(e) => handleMouseMove(e, section)}
            onmouseleave={handleMouseLeave}
            role="application"
            tabindex="0"
            aria-label="Music staff - click to place chords"
          >
            {#if renderedSvgs[section.id]}
              <div class="svg-wrapper">
                {@html renderedSvgs[section.id]}
              </div>
              
              <!-- Ghost note overlay (CSS positioned) -->
              {#if ghostNote && ghostNote.sectionId === section.id && activeTool.type === 'chord'}
                <div 
                  class="ghost-note"
                  style="left: {ghostNote.xPercent}%; top: {ghostNote.yPercent}%;"
                ></div>
              {/if}
              
              <!-- Underlines for answer blanks -->
              {#if !showAnswers}
                {#each getUnderlinePositions(section) as pos}
                  <div 
                    class="answer-underline"
                    style="left: {pos.leftPercent}%; top: {pos.topPercent}%; width: {pos.widthPercent}%;"
                  ></div>
                {/each}
              {/if}
            {:else}
              <div class="rendering-state">
                <div class="spinner spinner-sm"></div>
                <span>Rendering...</span>
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if isRendering}
    <div class="render-indicator">
      <div class="spinner spinner-sm"></div>
      <span>Updating...</span>
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

  /* Error State */
  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    min-height: 400px;
    gap: var(--space-3);
    color: var(--color-error);
  }

  .error-title {
    margin: 0;
    font-family: var(--font-sans);
    font-size: var(--text-base);
    font-weight: var(--font-medium);
  }

  .error-code {
    padding: var(--space-2) var(--space-4);
    background-color: var(--color-error-bg);
    border-radius: var(--radius-md);
    font-family: monospace;
    font-size: var(--text-sm);
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

  /* Sections */
  .sections {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .section {
    border: 1px solid var(--color-accent-line);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    background-color: var(--color-paper);
    box-shadow: var(--shadow-card);
  }

  .section-title {
    margin: 0 0 var(--space-1);
    font-family: var(--font-serif);
    font-size: var(--text-lg);
    font-weight: var(--font-normal);
    color: var(--color-ink);
  }

  .section-instructions {
    margin: 0 0 var(--space-3);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    font-style: italic;
    color: var(--color-ink-light);
  }

  /* Staff Container */
  .staff-container {
    position: relative;
    background-color: var(--color-paper);
    border-radius: var(--radius-md);
    min-height: 120px;
    transition: background-color var(--transition-base);
  }

  .staff-container.chord-mode {
    cursor: crosshair;
  }

  .staff-container.chord-mode:hover {
    background-color: var(--color-tier-colorful-bg);
  }

  .staff-container:focus-visible {
    outline: 2px solid var(--color-accent-gold);
    outline-offset: 2px;
  }

  .svg-wrapper {
    position: relative;
    width: 100%;
  }

  .svg-wrapper :global(svg) {
    width: 100%;
    height: auto;
    display: block;
  }

  /* Ghost Note */
  .ghost-note {
    position: absolute;
    width: 14px;
    height: 10px;
    background-color: var(--color-accent-gold);
    border-radius: 50%;
    transform: translate(-50%, -50%) rotate(-20deg);
    pointer-events: none;
    transition: left 0.05s ease, top 0.05s ease;
    box-shadow: 
      0 0 0 2px rgba(184, 149, 108, 0.3),
      0 0 12px 2px rgba(184, 149, 108, 0.25);
    opacity: 0.85;
  }

  :root.dark .ghost-note {
    box-shadow: 
      0 0 0 2px rgba(201, 168, 108, 0.4),
      0 0 12px 2px rgba(201, 168, 108, 0.3);
  }

  /* Answer Underline */
  .answer-underline {
    position: absolute;
    height: 2px;
    background-color: var(--color-ink);
    pointer-events: none;
    border-radius: 1px;
  }

  /* Rendering State */
  .rendering-state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    height: 150px;
    color: var(--color-ink-muted);
    font-size: var(--text-sm);
  }

  /* Render Indicator */
  .render-indicator {
    position: absolute;
    bottom: var(--space-4);
    right: var(--space-4);
    display: flex;
    align-items: center;
    gap: var(--space-2);
    background-color: var(--color-ink);
    color: var(--color-paper);
    padding: var(--space-1-5) var(--space-3);
    border-radius: var(--radius-full);
    font-size: var(--text-xs);
    box-shadow: var(--shadow-md);
    animation: fadeIn 0.2s ease-out;
  }

  .render-indicator .spinner {
    border-color: var(--color-paper);
    border-top-color: transparent;
  }

  /* Spinner sizes */
  .spinner {
    border: 2px solid var(--color-accent-line);
    border-top-color: var(--color-ink);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .spinner-sm {
    width: 14px;
    height: 14px;
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

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
