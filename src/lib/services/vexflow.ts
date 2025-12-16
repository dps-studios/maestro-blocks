/**
 * VexFlow Rendering Service
 * 
 * Provides pixel-perfect music notation rendering with precise coordinate tracking
 * for cursor/ghost note positioning.
 */

import {
  Renderer,
  Stave,
  StaveNote,
  Voice,
  Formatter,
  Accidental,
  type RenderContext,
} from 'vexflow';
import type { 
  WorksheetSection, 
  Pitch, 
  ClefType, 
  NoteName,
  Accidental as AccidentalType,
  TimeSignature,
} from '../types/score';

// ============================================================================
// LAYOUT CONSTANTS - All measurements in pixels at scale 1.0
// ============================================================================

export const LAYOUT = {
  // Staff dimensions - US Letter proportions (8.5:11 aspect ratio)
  // At 96 DPI: 816 x 1056 pixels, but we use a scaled version
  STAFF_WIDTH: 680,
  STAFF_HEIGHT: 200,   // Height per system (staff line) - extra space for ledger lines and answer boxes
  STAVE_Y: 50,         // Treble clef extends ~40px above top line, plus padding
  
  // Multi-system layout
  MEASURES_PER_SYSTEM: 4,  // How many measures per staff line
  SYSTEM_SPACING: 10,      // Vertical gap between systems (reduced for compact layout)
  
  // Margins
  LEFT_MARGIN: 10,     // Minimal left margin - VexFlow handles clef spacing
  RIGHT_MARGIN: 10,
  
  // Staff line spacing (VexFlow default is 10px between lines)
  STAFF_LINE_SPACING: 10,
  
  // Note positioning
  NOTE_HEAD_WIDTH: 12,
} as const;

// ============================================================================
// COORDINATE SYSTEM
// ============================================================================

export interface MeasureBound {
  startX: number;
  endX: number;
  centerX: number;
  /** Which system (staff line) this measure is on (0-indexed) */
  systemIndex: number;
  /** Y position of this system's stave */
  systemY: number;
  /** Y position of top staff line for this system */
  staffTopY: number;
  /** Y position of bottom staff line for this system */
  staffBottomY: number;
}

export interface StaffCoordinates {
  /** X position where measures begin (after clef/time sig) - first system */
  measureStartX: number;
  /** X position where measures end - first system */
  measureEndX: number;
  /** Width of each measure */
  measureWidth: number;
  /** Y position of top staff line (first system) */
  staffTopY: number;
  /** Y position of bottom staff line (first system) */
  staffBottomY: number;
  /** Spacing between staff lines */
  lineSpacing: number;
  /** Total width of the rendered staff */
  totalWidth: number;
  /** Total height of the rendered area */
  totalHeight: number;
  /** Array of measure bounds with system info */
  measureBounds: MeasureBound[];
  /** Number of systems (staff lines) */
  numSystems: number;
  /** Height of each system */
  systemHeight: number;
  /** Spacing between systems */
  systemSpacing: number;
}

/**
 * Calculate the precise coordinates for a staff with given parameters
 * Note: This is a simplified calculation for single-system layouts
 * For multi-system, use renderSection which calculates coords from actual VexFlow output
 */
export function calculateStaffCoordinates(
  numMeasures: number,
  scale: number = 1.0
): StaffCoordinates {
  const staffWidth = LAYOUT.STAFF_WIDTH * scale;
  const leftMargin = LAYOUT.LEFT_MARGIN * scale;
  const rightMargin = LAYOUT.RIGHT_MARGIN * scale;
  const lineSpacing = LAYOUT.STAFF_LINE_SPACING * scale;
  const systemHeight = LAYOUT.STAFF_HEIGHT * scale;
  const systemSpacing = LAYOUT.SYSTEM_SPACING * scale;
  
  const measuresPerSystem = LAYOUT.MEASURES_PER_SYSTEM;
  const numSystems = Math.ceil(numMeasures / measuresPerSystem);
  
  const measureStartX = leftMargin;
  const measureEndX = staffWidth - rightMargin;
  
  // VexFlow places staff at staveY, with 4 spaces (5 lines)
  const staveY = LAYOUT.STAVE_Y * scale;
  const staffTopY = staveY;
  const staffBottomY = staveY + (4 * lineSpacing);
  
  // Calculate each measure's bounds with system info
  const measureBounds: MeasureBound[] = [];
  for (let i = 0; i < numMeasures; i++) {
    const systemIndex = Math.floor(i / measuresPerSystem);
    const measureInSystem = i % measuresPerSystem;
    const measuresInThisSystem = Math.min(measuresPerSystem, numMeasures - (systemIndex * measuresPerSystem));
    const measureWidth = (measureEndX - measureStartX) / measuresInThisSystem;
    
    const systemY = staveY + (systemIndex * (systemHeight + systemSpacing));
    const sysStaffTopY = systemY;
    const sysStaffBottomY = systemY + (4 * lineSpacing);
    
    const startX = measureStartX + (measureInSystem * measureWidth);
    const endX = startX + measureWidth;
    
    measureBounds.push({
      startX,
      endX,
      centerX: startX + measureWidth / 2,
      systemIndex,
      systemY,
      staffTopY: sysStaffTopY,
      staffBottomY: sysStaffBottomY,
    });
  }
  
  const totalHeight = (systemHeight * numSystems) + (systemSpacing * (numSystems - 1));
  const measureWidth = measureBounds[0] ? (measureBounds[0].endX - measureBounds[0].startX) : (measureEndX - measureStartX) / measuresPerSystem;
  
  return {
    measureStartX,
    measureEndX,
    measureWidth,
    staffTopY,
    staffBottomY,
    lineSpacing,
    totalWidth: staffWidth,
    totalHeight,
    measureBounds,
    numSystems,
    systemHeight,
    systemSpacing,
  };
}

/**
 * Convert a Y position to a pitch on the staff
 */
export function yPositionToPitch(
  y: number,
  coords: StaffCoordinates,
  clef: ClefType
): Pitch {
  // Calculate which staff position (line/space) the Y is closest to
  // Staff positions: 0 = top line, positive = below, negative = above
  const relativeY = y - coords.staffTopY;
  const halfLineSpacing = coords.lineSpacing / 2;
  
  // Each half-line-spacing is one staff position (line or space)
  const staffPosition = Math.round(relativeY / halfLineSpacing);
  
  // Map staff position to pitch based on clef
  return staffPositionToPitch(staffPosition, clef);
}

/**
 * Convert a pitch to a Y position on the staff
 */
export function pitchToYPosition(
  pitch: Pitch,
  coords: StaffCoordinates,
  clef: ClefType
): number {
  const staffPosition = pitchToStaffPosition(pitch, clef);
  return coords.staffTopY + (staffPosition * coords.lineSpacing / 2);
}

/**
 * Get the measure index from an X position
 */
export function xPositionToMeasure(
  x: number,
  coords: StaffCoordinates
): number {
  if (x < coords.measureStartX) return 0;
  if (x > coords.measureEndX) return coords.measureBounds.length - 1;
  
  const relativeX = x - coords.measureStartX;
  const measureIndex = Math.floor(relativeX / coords.measureWidth);
  return Math.max(0, Math.min(measureIndex, coords.measureBounds.length - 1));
}

// ============================================================================
// PITCH <-> STAFF POSITION CONVERSION
// ============================================================================

const NOTE_ORDER: NoteName[] = ['c', 'd', 'e', 'f', 'g', 'a', 'b'];

// Reference pitches for each clef (pitch at the top line, staff position 0)
const CLEF_REFERENCES: Record<ClefType, { note: NoteName; octave: number }> = {
  treble: { note: 'f', octave: 5 },  // Top line is F5
  bass: { note: 'a', octave: 3 },    // Top line is A3
  alto: { note: 'g', octave: 4 },    // Top line is G4
  tenor: { note: 'a', octave: 4 },   // Top line is A4
  both: { note: 'f', octave: 5 },    // Default to treble (actual clef determined per-system)
};

function staffPositionToPitch(staffPosition: number, clef: ClefType): Pitch {
  const ref = CLEF_REFERENCES[clef];
  const refNoteIndex = NOTE_ORDER.indexOf(ref.note);
  const refAbsolutePosition = ref.octave * 7 + refNoteIndex;
  
  // Staff position 0 = top line = reference pitch
  // Each increment in staff position = one step down in pitch
  const absolutePosition = refAbsolutePosition - staffPosition;
  
  const octave = Math.floor(absolutePosition / 7);
  const noteIndex = ((absolutePosition % 7) + 7) % 7; // Handle negative modulo
  
  return {
    note: NOTE_ORDER[noteIndex],
    octave: Math.max(0, Math.min(8, octave)) as Pitch['octave'],
    accidental: null,
  };
}

function pitchToStaffPosition(pitch: Pitch, clef: ClefType): number {
  const ref = CLEF_REFERENCES[clef];
  const refNoteIndex = NOTE_ORDER.indexOf(ref.note);
  const refAbsolutePosition = ref.octave * 7 + refNoteIndex;
  
  const pitchNoteIndex = NOTE_ORDER.indexOf(pitch.note);
  const pitchAbsolutePosition = pitch.octave * 7 + pitchNoteIndex;
  
  // Staff position = how many steps below the reference
  return refAbsolutePosition - pitchAbsolutePosition;
}

// ============================================================================
// VEXFLOW NOTE CONVERSION
// ============================================================================

function pitchToVexKey(pitch: Pitch): string {
  // VexFlow format: "c/4", "f#/5", "bb/3"
  let key = pitch.note;
  if (pitch.accidental === 'sharp') key += '#';
  else if (pitch.accidental === 'flat') key += 'b';
  else if (pitch.accidental === 'natural') key += 'n';
  return `${key}/${pitch.octave}`;
}

function accidentalToVex(acc: AccidentalType | undefined): string | null {
  if (!acc) return null;
  switch (acc) {
    case 'sharp': return '#';
    case 'flat': return 'b';
    case 'double-sharp': return '##';
    case 'double-flat': return 'bb';
    case 'natural': return 'n';
    default: return null;
  }
}

function clefToVex(clef: ClefType): string {
  return clef; // VexFlow uses same names
}

// ============================================================================
// RENDERING
// ============================================================================

export interface RenderResult {
  svg: SVGElement;
  coordinates: StaffCoordinates;
}

/**
 * Render a worksheet section to SVG using VexFlow
 * Supports multi-line rendering with MEASURES_PER_SYSTEM measures per line
 */
export function renderSection(
  section: WorksheetSection,
  showAnswers: boolean,
  timeSignature: TimeSignature,
  keySignature: number = 0,
  scale: number = 1.0,
  selectedChordId: string | null = null
): RenderResult {
  const numMeasures = section.staff.measures.length;
  const measuresPerSystem = LAYOUT.MEASURES_PER_SYSTEM;
  const numSystems = Math.ceil(numMeasures / measuresPerSystem);
  
  const totalWidth = LAYOUT.STAFF_WIDTH * scale;
  const systemHeight = LAYOUT.STAFF_HEIGHT * scale;
  const systemSpacing = LAYOUT.SYSTEM_SPACING * scale;
  const totalHeight = (systemHeight * numSystems) + (systemSpacing * (numSystems - 1));
  
  // Create SVG container
  const div = document.createElement('div');
  const renderer = new Renderer(div, Renderer.Backends.SVG);
  renderer.resize(totalWidth, totalHeight);
  const context = renderer.getContext();
  
  // Track all measure bounds across all systems for coordinate lookup
  const allMeasureBounds: StaffCoordinates['measureBounds'] = [];
  let firstSystemCoords: { staffTopY: number; staffBottomY: number; lineSpacing: number } | null = null;
  
  // Render each system (staff line)
  for (let systemIndex = 0; systemIndex < numSystems; systemIndex++) {
    const systemY = (systemIndex * (systemHeight + systemSpacing)) + (LAYOUT.STAVE_Y * scale);
    const startMeasure = systemIndex * measuresPerSystem;
    const endMeasure = Math.min(startMeasure + measuresPerSystem, numMeasures);
    const measuresInSystem = endMeasure - startMeasure;
    
    // Determine clef for this system
    // For "both" mode, use the first chord's clefOverride on this line
    let systemClef: 'treble' | 'bass' = section.staff.clef === 'bass' ? 'bass' : 'treble';
    
    // Look for first chord in this system to get its clef override
    for (let i = startMeasure; i < endMeasure; i++) {
      const measure = section.staff.measures[i];
      const firstChord = measure.elements.find(e => e.type === 'chord');
      if (firstChord && firstChord.type === 'chord' && firstChord.clefOverride) {
        systemClef = firstChord.clefOverride;
        break;
      }
    }
    
    // Create the stave for this system
    const stave = new Stave(0, systemY, totalWidth);
    
    // Only show clef and time signature on first system
    if (systemIndex === 0) {
      stave.addClef(clefToVex(systemClef));
      stave.addTimeSignature(`${timeSignature.beats}/${timeSignature.beatType}`);
      if (keySignature !== 0) {
        stave.addKeySignature(fifthsToKeyName(keySignature));
      }
    } else {
      // Subsequent systems: just add clef (standard notation practice)
      stave.addClef(clefToVex(systemClef));
    }
    
    stave.setContext(context).draw();
    
    // Get actual coordinates from this stave
    const actualTopLineY = stave.getYForLine(0);
    const actualBottomLineY = stave.getYForLine(4);
    const actualLineSpacing = (actualBottomLineY - actualTopLineY) / 4;
    
    // Store first system's Y coordinates for ghost note positioning
    if (systemIndex === 0) {
      firstSystemCoords = { staffTopY: actualTopLineY, staffBottomY: actualBottomLineY, lineSpacing: actualLineSpacing };
    }
    
    // Calculate measure positions for this system
    const noteStartX = stave.getNoteStartX();
    const noteEndX = stave.getNoteEndX();
    const measureWidth = (noteEndX - noteStartX) / measuresInSystem;
    
    // Build measure bounds for this system
    const systemMeasureBounds: Array<{ startX: number; endX: number; centerX: number; systemIndex: number; staveY: number }> = [];
    for (let i = 0; i < measuresInSystem; i++) {
      const startX = noteStartX + (i * measureWidth);
      const endX = startX + measureWidth;
      systemMeasureBounds.push({
        startX,
        endX,
        centerX: startX + measureWidth / 2,
        systemIndex,
        staveY: systemY,
      });
    }
    
    // Draw bar lines for this system
    drawSystemMeasureLines(context, systemMeasureBounds, actualTopLineY, actualBottomLineY, noteEndX);
    
    // Track chord positions for answer lines (with answers for display and bottom Y for spacing)
    const chordPositions: Array<{ x: number; answer: string; bottomY: number }> = [];
    
    // Render notes for measures in this system
    for (let i = 0; i < measuresInSystem; i++) {
      const globalMeasureIndex = startMeasure + i;
      const measure = section.staff.measures[globalMeasureIndex];
      const measureBound = systemMeasureBounds[i];
      
      // Find chord elements in this measure
      const chordElements = measure.elements.filter(e => e.type === 'chord');
      
      if (chordElements.length > 0) {
        // Create a temporary stave positioned at this measure's X
        // Must set clef so VexFlow positions notes correctly for bass/treble
        const measureStave = new Stave(measureBound.startX, systemY, measureBound.endX - measureBound.startX);
        measureStave.setClef(clefToVex(systemClef));
        measureStave.setContext(context);
        
        for (const element of chordElements) {
          if (element.type === 'chord') {
            const isSelected = element.id === selectedChordId;
            // Use chord's clefOverride if available, otherwise use systemClef
            const chordClef = element.clefOverride || systemClef;
            const staveNote = createChordStaveNote(element.pitches, element.duration, showAnswers, isSelected, chordClef);
            
            const voice = new Voice({ numBeats: timeSignature.beats, beatValue: timeSignature.beatType }).setStrict(false);
            voice.addTickables([staveNote]);
            
            const formatter = new Formatter();
            formatter.joinVoices([voice]);
            formatter.format([voice], measureBound.endX - measureBound.startX - 30);
            
            staveNote.setStave(measureStave);
            staveNote.setContext(context);
            voice.draw(context, measureStave);
            
            // Track chord X position, answer, and bottom Y for answer line positioning
            const chordX = staveNote.getAbsoluteX();
            const boundingBox = staveNote.getBoundingBox();
            const chordBottomY = boundingBox ? boundingBox.getY() + boundingBox.getH() : actualBottomLineY;
            chordPositions.push({ x: chordX, answer: element.displayName || '', bottomY: chordBottomY });
          }
        }
      }
      
      // Add to global measure bounds with full system info
      allMeasureBounds.push({
        startX: measureBound.startX,
        endX: measureBound.endX,
        centerX: measureBound.centerX,
        systemIndex,
        systemY,
        staffTopY: actualTopLineY,
        staffBottomY: actualBottomLineY,
      });
    }
    
    // Draw answer lines below this system for students to write chord names
    drawAnswerLines(context, chordPositions, actualBottomLineY, showAnswers);
  }
  
  // Create coordinates object using first system's coordinates
  const coords: StaffCoordinates = {
    measureStartX: allMeasureBounds[0]?.startX ?? 0,
    measureEndX: allMeasureBounds[allMeasureBounds.length - 1]?.endX ?? totalWidth,
    measureWidth: allMeasureBounds[0] ? (allMeasureBounds[0].endX - allMeasureBounds[0].startX) : totalWidth / measuresPerSystem,
    staffTopY: firstSystemCoords?.staffTopY ?? LAYOUT.STAVE_Y * scale,
    staffBottomY: firstSystemCoords?.staffBottomY ?? (LAYOUT.STAVE_Y + 40) * scale,
    lineSpacing: firstSystemCoords?.lineSpacing ?? LAYOUT.STAFF_LINE_SPACING * scale,
    totalWidth,
    totalHeight,
    measureBounds: allMeasureBounds,
    numSystems,
    systemHeight,
    systemSpacing,
  };
  
  const svg = div.querySelector('svg') as SVGElement;
  return { svg, coordinates: coords };
}

/**
 * Draw bar lines for a single system
 */
function drawSystemMeasureLines(
  context: RenderContext,
  measureBounds: Array<{ startX: number; endX: number }>,
  staffTopY: number,
  staffBottomY: number,
  noteEndX: number
): void {
  context.save();
  context.setStrokeStyle('#2C2416');
  context.setLineWidth(1.5);
  
  // Draw vertical bar lines between measures
  for (let i = 1; i < measureBounds.length; i++) {
    const x = measureBounds[i].startX;
    context.beginPath();
    context.moveTo(x, staffTopY);
    context.lineTo(x, staffBottomY);
    context.stroke();
  }
  
  // Draw final double bar line (thin + thick)
  context.setLineWidth(1.5);
  context.beginPath();
  context.moveTo(noteEndX - 4, staffTopY);
  context.lineTo(noteEndX - 4, staffBottomY);
  context.stroke();
  
  context.setLineWidth(3);
  context.beginPath();
  context.moveTo(noteEndX, staffTopY);
  context.lineTo(noteEndX, staffBottomY);
  context.stroke();
  
  context.restore();
}

/**
 * Draw answer lines below measures for students to write chord names
 * When showAnswers is true, also display the correct chord name
 */
function drawAnswerLines(
  context: RenderContext,
  chordPositions: Array<{ x: number; answer: string; bottomY: number }>,
  staffBottomY: number,
  showAnswers: boolean
): void {
  context.save();
  context.setStrokeStyle('#999999'); // Light gray for answer lines
  context.setLineWidth(1);
  
  // Find the lowest point of any chord in this system
  const maxChordBottomY = chordPositions.length > 0
    ? Math.max(...chordPositions.map(c => c.bottomY))
    : staffBottomY;
  
  // Position answer lines below the lowest chord (or staff bottom if no low chords)
  // Use at least 35px below the lowest element for adequate spacing
  const answerLineY = Math.max(staffBottomY + 40, maxChordBottomY + 35);
  const lineWidth = 40; // Width of answer line (narrower)
  const xOffset = 8; // Shift right to center under chord
  
  // Draw a line below each chord
  for (const chord of chordPositions) {
    const lineStartX = chord.x - (lineWidth / 2) + xOffset;
    const lineEndX = chord.x + (lineWidth / 2) + xOffset;
    
    context.beginPath();
    context.moveTo(lineStartX, answerLineY);
    context.lineTo(lineEndX, answerLineY);
    context.stroke();
    
    // Draw the answer text if showAnswers is enabled
    if (showAnswers && chord.answer) {
      // Get the SVG element to add properly centered text
      const svg = (context as unknown as { svg: SVGSVGElement }).svg;
      if (svg) {
        const lineCenterX = chord.x + xOffset;
        const textY = answerLineY - 5; // 5px above the line
        
        // Parse the answer format: "Cm|6|4" -> base="Cm", top figure="6", bottom figure="4"
        const parts = chord.answer.split('|');
        const baseText = parts[0];
        const topFigure = parts[1] || '';
        const bottomFigure = parts[2] || '';
        
        // Estimate base text width for positioning (and centering adjustment)
        const baseWidth = baseText.length * 7.5; // ~7.5px per char at 14px font
        const hasFigures = topFigure || bottomFigure;
        const figuresWidth = hasFigures ? 8 : 0; // width of figured bass column
        const totalWidth = baseWidth + figuresWidth;
        
        // Adjust X to center the entire chord symbol + figures
        const adjustedCenterX = lineCenterX - (figuresWidth / 2);
        
        // Render base chord name
        const textElement = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        textElement.setAttribute('x', String(adjustedCenterX));
        textElement.setAttribute('y', String(textY));
        textElement.setAttribute('text-anchor', 'middle');
        textElement.setAttribute('font-family', "'Libre Baskerville', Georgia, serif");
        textElement.setAttribute('font-size', '14px');
        textElement.setAttribute('font-weight', 'normal');
        textElement.setAttribute('fill', '#2C2416');
        textElement.textContent = baseText;
        svg.appendChild(textElement);
        
        // Render figured bass as vertically stacked numbers to the right of base text
        if (hasFigures) {
          const figuresX = adjustedCenterX + (baseWidth / 2) + 3; // 3px gap after base text
          
          if (topFigure) {
            const topElement = document.createElementNS('http://www.w3.org/2000/svg', 'text');
            topElement.setAttribute('x', String(figuresX));
            topElement.setAttribute('y', String(textY - 5)); // above baseline
            topElement.setAttribute('font-family', "'Libre Baskerville', Georgia, serif");
            topElement.setAttribute('font-size', '9px');
            topElement.setAttribute('font-weight', 'normal');
            topElement.setAttribute('fill', '#2C2416');
            topElement.textContent = topFigure;
            svg.appendChild(topElement);
          }
          
          if (bottomFigure) {
            const bottomElement = document.createElementNS('http://www.w3.org/2000/svg', 'text');
            bottomElement.setAttribute('x', String(figuresX));
            bottomElement.setAttribute('y', String(textY + 6)); // below baseline
            bottomElement.setAttribute('font-family', "'Libre Baskerville', Georgia, serif");
            bottomElement.setAttribute('font-size', '9px');
            bottomElement.setAttribute('font-weight', 'normal');
            bottomElement.setAttribute('fill', '#2C2416');
            bottomElement.textContent = bottomFigure;
            svg.appendChild(bottomElement);
          }
        }
      }
    }
  }
  
  context.restore();
}

function createChordStaveNote(
  pitches: Pitch[],
  duration: { value: number; dots: number },
  showAnswers: boolean,
  isSelected: boolean = false,
  clef: 'treble' | 'bass' = 'treble'
): StaveNote {
  // Convert duration to VexFlow format
  const durationMap: Record<number, string> = {
    1: 'w',   // whole
    2: 'h',   // half
    4: 'q',   // quarter
    8: '8',   // eighth
    16: '16', // sixteenth
  };
  
  const vexDuration = durationMap[duration.value] || 'w';
  
  // Create the chord (multiple pitches on one stem)
  const keys = pitches.map(pitchToVexKey);
  
  // Must pass clef to StaveNote so VexFlow positions notes correctly
  const note = new StaveNote({
    keys,
    duration: vexDuration,
    clef,
  });
  
  // Add accidentals
  pitches.forEach((pitch, index) => {
    const acc = accidentalToVex(pitch.accidental);
    if (acc) {
      note.addModifier(new Accidental(acc), index);
    }
  });
  
  // Apply selection styling if this chord is selected
  if (isSelected) {
    note.setStyle({
      fillStyle: '#B8860B',   // Dark goldenrod - visible on both light/dark
      strokeStyle: '#B8860B',
    });
  }
  
  return note;
}

function fifthsToKeyName(fifths: number): string {
  const sharpKeys = ['C', 'G', 'D', 'A', 'E', 'B', 'F#', 'C#'];
  const flatKeys = ['C', 'F', 'Bb', 'Eb', 'Ab', 'Db', 'Gb', 'Cb'];
  
  if (fifths >= 0) {
    return sharpKeys[Math.min(fifths, 7)];
  } else {
    return flatKeys[Math.min(Math.abs(fifths), 7)];
  }
}

// ============================================================================
// SIMPLE RENDERING (for initial empty state)
// Note: This function is deprecated - use renderSection instead
// ============================================================================

/**
 * @deprecated Use renderSection instead - this doesn't support multi-system layout
 */
export function renderEmptyStaff(
  numMeasures: number,
  clef: ClefType,
  timeSignature: TimeSignature,
  keySignature: number = 0,
  scale: number = 1.0
): RenderResult {
  // Create a minimal section and delegate to renderSection
  const section = {
    id: 'empty',
    type: 'chord-naming' as const,
    staff: {
      id: 'empty-staff',
      clef,
      measures: Array.from({ length: numMeasures }, (_, i) => ({
        id: `measure-${i}`,
        number: i + 1,
        elements: [],
      })),
    },
    answerBoxes: [],
    autoExpand: false,
    autoExpandCount: 0,
    maxMeasures: numMeasures,
  };
  
  return renderSection(section, false, timeSignature, keySignature, scale);
}
