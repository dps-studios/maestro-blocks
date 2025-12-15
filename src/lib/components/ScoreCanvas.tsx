import { createSignal, createEffect, onMount, on, Show, For } from 'solid-js';
import { scoreStore, editorStore } from '../stores/score';
import { 
  renderSection, 
  yPositionToPitch,
  pitchToYPosition,
  type StaffCoordinates,
} from '../services/vexflow';
import type { WorksheetSection, Pitch } from '../types/score';

export default function ScoreCanvas() {
  // Reactive state
  const [isLoading, setIsLoading] = createSignal(true);
  const [staffCoordinates, setStaffCoordinates] = createSignal<Map<string, StaffCoordinates>>(new Map());
  const [isPlacingChord, setIsPlacingChord] = createSignal(false);
  
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
  });

  // Reactive store accessors
  const sections = () => scoreStore.state.sections;
  const keyFifths = () => scoreStore.state.keySignature.fifths;
  const timeSignature = () => scoreStore.state.timeSignature;
  const showAnswers = () => scoreStore.state.showAnswers;
  const activeTool = () => editorStore.state.activeTool;

  // Re-render when score changes - use explicit dependency tracking
  // This prevents the effect from running on unrelated store changes
  createEffect(
    on(
      // Explicit dependencies - only re-run when these change
      () => [sections(), showAnswers(), timeSignature(), keyFifths()] as const,
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
        const result = renderSection(section, showAnswers(), timeSignature(), keyFifths());
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

  // Handle staff click - place chord at ghost note position
  async function handleStaffClick(_event: MouseEvent, section: WorksheetSection) {
    // Prevent multiple rapid clicks
    if (isPlacingChord()) {
      console.log('[ScoreCanvas] Click ignored - chord placement in progress');
      return;
    }
    
    if (activeTool().type !== 'chord') return;
    
    const ghost = ghostNote();
    if (!ghost || ghost.sectionId !== section.id) return;

    const measure = section.staff.measures[ghost.measureIndex];
    if (!measure) return;

    // Set guard flag
    setIsPlacingChord(true);

    console.log('[ScoreCanvas] === CHORD PLACEMENT ===');
    console.log('[ScoreCanvas] Measure index:', ghost.measureIndex, 'Pitch:', ghost.pitch.note + ghost.pitch.octave);

    const chordDef = {
      root: ghost.pitch.note,
      rootAccidental: ghost.pitch.accidental,
      quality: (activeTool() as { type: 'chord'; quality: string }).quality as any,
      inversion: 'root' as const,
    };

    try {
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
  }

  function formatPitch(pitch: Pitch): string {
    let str = pitch.note.toUpperCase();
    if (pitch.accidental === 'sharp') str += '#';
    else if (pitch.accidental === 'flat') str += 'b';
    return `${str}${pitch.octave}`;
  }

  return (
    <div class="score-canvas paper-texture">
      <Show when={isLoading()}>
        <div class="loading-state">
          <div class="spinner spinner-lg"></div>
          <p class="loading-text">Loading notation engine...</p>
        </div>
      </Show>
      
      <Show when={!isLoading() && sections().length === 0}>
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
      </Show>
      
      <Show when={!isLoading() && sections().length > 0}>
        <div class="sections">
          <For each={sections()}>
            {(section) => (
              <div class="section">
                <Show when={section.title}>
                  <h3 class="section-title">{section.title}</h3>
                </Show>
                <Show when={section.instructions}>
                  <p class="section-instructions">{section.instructions}</p>
                </Show>

                <div
                  class={`staff-container ${activeTool().type === 'chord' ? 'chord-mode' : ''}`}
                  role="application"
                  tabIndex={0}
                  aria-label="Music staff - click to place chords"
                  onClick={(e) => handleStaffClick(e, section)}
                  onMouseMove={(e) => handleMouseMove(e, section)}
                  onMouseLeave={handleMouseLeave}
                >
                  <div 
                    class="vexflow-container"
                    data-section-id={section.id}
                  ></div>
                  
                  <Show when={ghostNote() && ghostNote()!.sectionId === section.id && activeTool().type === 'chord'}>
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
      </Show>
    </div>
  );
}
