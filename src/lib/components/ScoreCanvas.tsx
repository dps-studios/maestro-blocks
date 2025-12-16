import { createSignal, createEffect, onMount, on, Show, For } from 'solid-js';
import { scoreStore, editorStore } from '../stores/score';
import { 
  renderSection, 
  yPositionToPitch,
  pitchToYPosition,
  type StaffCoordinates,
} from '../services/vexflow';
import { generateChordPitchesRust } from '../services/music';
import type { WorksheetSection, Pitch, ChordElement, ChordDefinition } from '../types/score';
import ChordEditorPopup from './ChordEditorPopup';
import { EditableHeader } from './EditableHeader';

export default function ScoreCanvas() {
  // Reactive state
  const [isLoading, setIsLoading] = createSignal(true);
  const [staffCoordinates, setStaffCoordinates] = createSignal<Map<string, StaffCoordinates>>(new Map());
  const [isPlacingChord, setIsPlacingChord] = createSignal(false);
  
  // Cursor/interaction mode state
  const [hoverHasChord, setHoverHasChord] = createSignal(false);
  const [isInsertMode, setIsInsertMode] = createSignal(false); // True after double-click on measure with chord
  
  // Ghost note state - pixel-perfect positioning
  const [ghostNote, setGhostNote] = createSignal<{ 
    x: number;
    y: number;
    pitch: Pitch;
    measureIndex: number;
    sectionId: string;
  } | null>(null);

  // Track last render to prevent excessive re-renders
  let lastRenderTime = 0;
  const MIN_RENDER_INTERVAL = 50; // ms

  // Initialize on mount
  onMount(() => {
    setIsLoading(false);
    console.log('[ScoreCanvas] VexFlow renderer ready');
    // Trigger initial render after DOM is ready - use double rAF to ensure
    // SolidJS has finished rendering the section containers
    requestAnimationFrame(() => {
      requestAnimationFrame(() => renderAllSections());
    });
  });

  // Reactive store accessors
  const sections = () => scoreStore.state.sections;
  const keyFifths = () => scoreStore.state.keySignature.fifths;
  const timeSignature = () => scoreStore.state.timeSignature;
  const showAnswers = () => scoreStore.state.showAnswers;
  const activeTool = () => editorStore.state.activeTool;
  const selectedChordId = () => editorStore.state.selectedChordId;

  // Re-render when score changes - use explicit dependency tracking
  // This prevents the effect from running on unrelated store changes
  createEffect(
    on(
      // Explicit dependencies - only re-run when these change
      // Include selectedChordId to re-render when selection changes
      // Include renderVersion to re-render after deep store mutations (e.g., updateChord)
      () => [sections(), showAnswers(), timeSignature(), keyFifths(), selectedChordId(), scoreStore.renderVersion] as const,
      () => {
        if (!isLoading() && sections().length > 0) {
          const now = Date.now();
          if (now - lastRenderTime >= MIN_RENDER_INTERVAL) {
            lastRenderTime = now;
            // Use requestAnimationFrame for smoother rendering
            requestAnimationFrame(() => renderAllSections());
          }
        }
      },
      { defer: true } // Don't run on initial mount, onMount handles that
    )
  );

  function renderAllSections() {
    const sectionList = sections();
    console.log('[ScoreCanvas] Rendering', sectionList.length, 'sections');
    
    const containers = getStaffContainers();
    const newCoords = new Map<string, StaffCoordinates>();
    
    for (const section of sectionList) {
      const container = containers.get(section.id);
      if (!container) {
        console.warn('[ScoreCanvas] No container for section:', section.id);
        continue;
      }
      
      // Clear previous content
      container.innerHTML = '';
      
      try {
        const result = renderSection(section, showAnswers(), timeSignature(), keyFifths(), 1.0, selectedChordId());
        container.appendChild(result.svg);
        newCoords.set(section.id, result.coordinates);
      } catch (error) {
        console.error('[ScoreCanvas] Error rendering section:', section.id, error);
      }
    }
    
    setStaffCoordinates(newCoords);
  }

  function getStaffContainers() {
    const containers = document.querySelectorAll<HTMLDivElement>('.vexflow-container[data-section-id]');
    const map = new Map<string, HTMLDivElement>();
    containers.forEach(el => {
      const id = el.getAttribute('data-section-id');
      if (id) map.set(id, el);
    });
    return map;
  }

  // Find chord element at click position
  function findChordAtPosition(section: WorksheetSection, measureIndex: number): ChordElement | null {
    const measure = section.staff.measures[measureIndex];
    if (!measure) return null;
    
    // For now, return the first chord in the measure (single chord per measure)
    // TODO: When supporting multiple chords per measure, use x-position to determine which
    const chord = measure.elements.find(el => el.type === 'chord');
    return chord as ChordElement | null;
  }

  // Get the optimal screen Y position for the popup (higher of note or staff top)
  function getChordPopupY(
    pitches: Pitch[],
    section: WorksheetSection,
    measureIndex: number
  ): number | null {
    const coords = staffCoordinates().get(section.id);
    if (!coords) return null;

    const containers = getStaffContainers();
    const container = containers.get(section.id);
    if (!container) return null;

    const svg = container.querySelector('svg');
    if (!svg) return null;

    const svgRect = svg.getBoundingClientRect();
    const measureBound = coords.measureBounds[measureIndex];
    if (!measureBound) return null;

    // 1. Find highest pitch (highest on staff = lowest Y value in screen coords)
    const topPitch = pitches.reduce((highest, p) => {
      // Compare by octave first, then by note position within octave
      const noteOrder: Record<string, number> = { c: 0, d: 1, e: 2, f: 3, g: 4, a: 5, b: 6 };
      const pValue = p.octave * 7 + noteOrder[p.note];
      const hValue = highest.octave * 7 + noteOrder[highest.note];
      return pValue > hValue ? p : highest;
    }, pitches[0]);

    // 2. Convert pitch to VexFlow Y coordinate
    const systemCoords = {
      ...coords,
      staffTopY: measureBound.staffTopY,
      staffBottomY: measureBound.staffBottomY,
    };
    const noteVexY = pitchToYPosition(topPitch, systemCoords, section.staff.clef);
    
    // 3. Get Staff Top Y (VexFlow coordinate)
    const staffTopVexY = measureBound.staffTopY;

    // 4. Scale to screen coordinates and pick the higher one (smaller Y)
    const scaleY = svgRect.height / coords.totalHeight;
    const noteScreenY = svgRect.top + noteVexY * scaleY;
    const staffScreenY = svgRect.top + staffTopVexY * scaleY;

    return Math.min(noteScreenY, staffScreenY);
  }

  // Get the horizontal center of a measure in screen coordinates
  function getMeasureCenterX(section: WorksheetSection, measureIndex: number): number | null {
    const coords = staffCoordinates().get(section.id);
    if (!coords) return null;

    const containers = getStaffContainers();
    const container = containers.get(section.id);
    if (!container) return null;

    const svg = container.querySelector('svg');
    if (!svg) return null;

    const svgRect = svg.getBoundingClientRect();
    const measureBound = coords.measureBounds[measureIndex];
    if (!measureBound) return null;

    // Calculate measure center in VexFlow coordinates
    const measureCenterX = (measureBound.startX + measureBound.endX) / 2;

    // Scale to screen coordinates
    const scaleX = svgRect.width / coords.totalWidth;
    return svgRect.left + measureCenterX * scaleX;
  }

  // Handle staff click - select existing chord or create new one
  async function handleStaffClick(event: MouseEvent, section: WorksheetSection) {
    // Prevent multiple rapid clicks
    if (isPlacingChord()) {
      console.log('[ScoreCanvas] Click ignored - chord placement in progress');
      return;
    }
    
    const ghost = ghostNote();
    if (!ghost || ghost.sectionId !== section.id) return;

    const measure = section.staff.measures[ghost.measureIndex];
    if (!measure) return;

    // Check if there's an existing chord at this position
    const existingChord = findChordAtPosition(section, ghost.measureIndex);
    
    if (existingChord && !isInsertMode()) {
      // Single click on existing chord → SELECT it
      // Position popup above top note of chord OR staff, centered on measure
      const popupY = getChordPopupY(existingChord.pitches, section, ghost.measureIndex);
      const anchorX = getMeasureCenterX(section, ghost.measureIndex) ?? event.clientX;
      const anchorY = popupY !== null ? popupY - 15 : event.clientY - 40;
      
      editorStore.selectChord(existingChord.id, {
        x: anchorX,
        y: anchorY,
        sectionId: section.id,
        measureId: measure.id,
      });
      
      console.log('[ScoreCanvas] Selected existing chord:', existingChord.id);
      return;
    }

    // No existing chord OR in insert mode - create new chord
    await placeNewChord(event, section, measure, ghost);
  }

  // Handle double-click - enter insert mode (to add chord even if one exists)
  function handleStaffDoubleClick(event: MouseEvent, section: WorksheetSection) {
    const ghost = ghostNote();
    if (!ghost || ghost.sectionId !== section.id) return;

    const existingChord = findChordAtPosition(section, ghost.measureIndex);
    
    if (existingChord) {
      // Double-click on measure with chord → enter insert mode
      setIsInsertMode(true);
      console.log('[ScoreCanvas] Entered insert mode for measure with existing chord');
      // The next click will place a new chord
    }
  }

  // Place a new chord at the ghost note position
  async function placeNewChord(
    event: MouseEvent,
    section: WorksheetSection,
    measure: { id: string },
    ghost: { pitch: Pitch; measureIndex: number }
  ) {
    // Shift+click = quick placement with last-used settings, no popup
    const isQuickPlace = event.shiftKey;
    
    // Set guard flag
    setIsPlacingChord(true);

    console.log('[ScoreCanvas] === CHORD PLACEMENT ===');
    console.log('[ScoreCanvas] Measure index:', ghost.measureIndex, 'Pitch:', ghost.pitch.note + ghost.pitch.octave);
    console.log('[ScoreCanvas] Quick place:', isQuickPlace);

    const { quality: lastQuality, inversion: lastInversion } = editorStore.lastUsedSettings;
    
    // Map inversion from '1'/'2'/'3' to 'first'/'second'/'third'
    const inversionMap: Record<string, 'root' | 'first' | 'second' | 'third'> = {
      'root': 'root',
      '1': 'first',
      '2': 'second', 
      '3': 'third',
    };
    
    const chordDef: ChordDefinition = {
      root: ghost.pitch.note,
      rootAccidental: ghost.pitch.accidental,
      quality: lastQuality,
      inversion: inversionMap[lastInversion] || 'root',
    };

    try {
      // Get chord pitches from Rust backend FIRST (for popup positioning)
      const { pitches } = await generateChordPitchesRust(
        chordDef,
        ghost.pitch.octave as 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8
      );

      const elementId = await scoreStore.addChordToMeasure(
        section.id,
        measure.id,
        chordDef,
        ghost.pitch.octave,
        { value: 1, dots: 0 }
      );
      console.log('[ScoreCanvas] Chord added:', elementId);
      
      // Directly trigger re-render after successful chord placement
      renderAllSections();
      
      // Exit insert mode after placing
      setIsInsertMode(false);
      
      // If not quick place, select the new chord and open popup above top note
      if (!isQuickPlace) {
        // Calculate top note Y from the pitches we just got
        const popupY = getChordPopupY(pitches, section, ghost.measureIndex);
        const anchorX = getMeasureCenterX(section, ghost.measureIndex) ?? event.clientX;
        const anchorY = popupY !== null ? popupY - 15 : event.clientY - 40;
        
        editorStore.selectChord(elementId, {
          x: anchorX,
          y: anchorY,
          sectionId: section.id,
          measureId: measure.id,
        });
      }
    } catch (error) {
      console.error('[ScoreCanvas] Failed to add chord:', error);
    } finally {
      // Release guard after a short delay to prevent double-clicks
      setTimeout(() => setIsPlacingChord(false), 100);
    }
  }

  // Handle mouse move - update ghost note position with pixel-perfect accuracy
  function handleMouseMove(event: MouseEvent, section: WorksheetSection) {
    if (activeTool().type !== 'chord') {
      setGhostNote(null);
      return;
    }

    const coords = staffCoordinates().get(section.id);
    if (!coords || coords.measureBounds.length === 0) return;

    const staffContainer = event.currentTarget as HTMLElement;
    const svg = staffContainer.querySelector('svg');
    if (!svg) return;

    const staffContainerRect = staffContainer.getBoundingClientRect();
    const svgRect = svg.getBoundingClientRect();
    
    const scaleX = coords.totalWidth / svgRect.width;
    const scaleY = coords.totalHeight / svgRect.height;

    const mouseXInSvg = event.clientX - svgRect.left;
    const mouseYInSvg = event.clientY - svgRect.top;
    const svgX = mouseXInSvg * scaleX;
    const svgY = mouseYInSvg * scaleY;

    let targetSystemIndex = 0;
    if (coords.numSystems > 1) {
      const systemTotalHeight = coords.systemHeight + coords.systemSpacing;
      targetSystemIndex = Math.floor((svgY + coords.systemSpacing / 2) / systemTotalHeight);
      targetSystemIndex = Math.max(0, Math.min(targetSystemIndex, coords.numSystems - 1));
    }

    const systemMeasures = coords.measureBounds
      .map((bound, index) => ({ bound, index }))
      .filter(({ bound }) => bound.systemIndex === targetSystemIndex);
    
    if (systemMeasures.length === 0) return;

    let measureIndex = systemMeasures[0].index;
    for (const { bound, index } of systemMeasures) {
      if (svgX >= bound.startX && svgX < bound.endX) {
        measureIndex = index;
        break;
      }
      if (svgX >= bound.endX) {
        measureIndex = index;
      }
    }

    const measureBound = coords.measureBounds[measureIndex];
    
    const systemCoords = {
      ...coords,
      staffTopY: measureBound.staffTopY,
      staffBottomY: measureBound.staffBottomY,
    };
    const pitch = yPositionToPitch(svgY, systemCoords, section.staff.clef);
    
    const snappedSvgX = measureBound.startX + 20;
    const snappedSvgY = pitchToYPosition(pitch, systemCoords, section.staff.clef);
    
    const svgOffsetX = svgRect.left - staffContainerRect.left;
    const svgOffsetY = svgRect.top - staffContainerRect.top;
    
    const screenX = (snappedSvgX / scaleX) + svgOffsetX;
    const screenY = (snappedSvgY / scaleY) + svgOffsetY;

    // Check if this measure already has a chord
    const measure = section.staff.measures[measureIndex];
    const measureHasChord = measure?.elements.some(el => el.type === 'chord') ?? false;
    setHoverHasChord(measureHasChord);

    setGhostNote({
      x: screenX,
      y: screenY,
      pitch,
      measureIndex,
      sectionId: section.id,
    });
  }

  function handleMouseLeave() {
    setGhostNote(null);
    setHoverHasChord(false);
    setIsInsertMode(false);
  }

  function formatPitch(pitch: Pitch): string {
    let str = pitch.note.toUpperCase();
    if (pitch.accidental === 'sharp') str += '#';
    else if (pitch.accidental === 'flat') str += 'b';
    return `${str}${pitch.octave}`;
  }

  return (
    <div class="score-canvas">
      {/* Chord Editor Popup - rendered at document level for proper positioning */}
      <ChordEditorPopup />
      
      <Show when={isLoading()}>
        <div class="loading-state">
          <div class="spinner spinner-lg"></div>
          <p class="loading-text">Loading notation engine...</p>
        </div>
      </Show>
      
      <Show when={!isLoading()}>
        {/* 8.5x11 Paper Worksheet */}
        <div class="worksheet-paper">
          <EditableHeader />
          
          <div class="worksheet-content">
            <Show when={sections().length === 0}>
              <div class="empty-worksheet">
                <p class="empty-worksheet-hint">Click "Add Staff Line" to begin adding chords to your worksheet.</p>
              </div>
            </Show>
            
            <div class="sections">
              <For each={sections()}>
                {(section) => (
                  <div class="section">
                    <div
                      class={`staff-container ${activeTool().type === 'chord' ? 'chord-mode' : ''} ${hoverHasChord() && !isInsertMode() ? 'select-cursor' : 'insert-cursor'}`}
                      role="application"
                      tabIndex={0}
                      aria-label="Music staff - click to place chords"
                      onClick={(e) => handleStaffClick(e, section)}
                      onDblClick={(e) => handleStaffDoubleClick(e, section)}
                      onMouseMove={(e) => handleMouseMove(e, section)}
                      onMouseLeave={handleMouseLeave}
                    >
                      <div 
                        class="vexflow-container"
                        data-section-id={section.id}
                      ></div>
                      
                      {/* Ghost note only visible when: empty measure OR in insert mode (after double-click) */}
                      <Show when={ghostNote() && ghostNote()!.sectionId === section.id && activeTool().type === 'chord' && (!hoverHasChord() || isInsertMode())}>
                        <div 
                          class="ghost-note"
                          style={`left: ${ghostNote()!.x}px; top: ${ghostNote()!.y}px;`}
                        >
                          <div class="ghost-note-head"></div>
                          <span class="ghost-note-label">{formatPitch(ghostNote()!.pitch)}</span>
                        </div>
                      </Show>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
