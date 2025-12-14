/**
 * Verovio service for real-time music notation rendering
 * Handles MEI generation and SVG output
 */

import createVerovioModule from 'verovio/wasm';
import { VerovioToolkit } from 'verovio/esm';
import type { Score, WorksheetSection, Pitch, ChordElement, MusicElement, ClefType } from '../types/score';

let toolkit: VerovioToolkit | null = null;
let initPromise: Promise<VerovioToolkit> | null = null;

/**
 * Initialize Verovio WASM module (singleton)
 */
export async function initVerovio(): Promise<VerovioToolkit> {
  if (toolkit) return toolkit;
  
  if (initPromise) return initPromise;
  
  initPromise = (async () => {
    console.log('[Verovio] Loading WASM module...');
    const module = await createVerovioModule();
    console.log('[Verovio] WASM loaded, creating toolkit...');
    toolkit = new VerovioToolkit(module);
    
    // Set default options for worksheet rendering
    console.log('[Verovio] Setting options...');
    toolkit.setOptions({
      scale: 100,  // Larger scale for better visibility
      pageWidth: 1800,
      pageHeight: 400,
      adjustPageHeight: true,
      pageMarginTop: 10,
      pageMarginBottom: 50,  // Extra space for answer underlines
      pageMarginLeft: 10,
      pageMarginRight: 10,
      // Add data attributes for interactive selection
      svgAdditionalAttribute: ['note@pname', 'note@oct', 'chord@xml:id'],
    });
    
    console.log('[Verovio] Initialization complete');
    return toolkit;
  })();
  
  return initPromise;
}

/**
 * Get the initialized toolkit (throws if not initialized)
 */
export function getToolkit(): VerovioToolkit {
  if (!toolkit) {
    throw new Error('Verovio not initialized. Call initVerovio() first.');
  }
  return toolkit;
}

/**
 * Convert a pitch to MEI note attributes
 */
function pitchToMei(pitch: Pitch): string {
  const accidentalMap: Record<string, string> = {
    'sharp': 's',
    'flat': 'f',
    'natural': 'n',
    'double-sharp': 'ss',
    'double-flat': 'ff',
  };
  
  const accid = pitch.accidental ? ` accid="${accidentalMap[pitch.accidental]}"` : '';
  return `pname="${pitch.note}" oct="${pitch.octave}"${accid}`;
}

/**
 * Convert duration to MEI dur attribute
 */
function durationToMei(value: number): string {
  return `dur="${value}"`;
}

/**
 * Generate MEI for a single chord element
 */
function chordToMei(chord: ChordElement): string {
  const dur = durationToMei(chord.duration.value);
  const dots = chord.duration.dots > 0 ? ` dots="${chord.duration.dots}"` : '';
  
  if (chord.pitches.length === 1) {
    // Single note
    const pitch = chord.pitches[0];
    return `<note xml:id="${chord.id}" ${dur}${dots} ${pitchToMei(pitch)} />`;
  }
  
  // Multiple notes = chord
  const notes = chord.pitches
    .map((p, i) => `<note xml:id="${chord.id}-n${i}" ${pitchToMei(p)} />`)
    .join('\n            ');
  
  return `<chord xml:id="${chord.id}" ${dur}${dots}>
            ${notes}
          </chord>`;
}

/**
 * Generate MEI for a music element
 */
function elementToMei(element: MusicElement): string {
  switch (element.type) {
    case 'chord':
      return chordToMei(element);
    case 'note':
      return `<note xml:id="${element.id}" ${durationToMei(element.duration.value)} ${pitchToMei(element.pitch)} />`;
    case 'rest':
      return `<rest xml:id="${element.id}" ${durationToMei(element.duration.value)} />`;
    default:
      return '';
  }
}

/**
 * Get MEI clef shape and line
 */
function clefToMei(clef: ClefType): { shape: string; line: string } {
  switch (clef) {
    case 'treble': return { shape: 'G', line: '2' };
    case 'bass': return { shape: 'F', line: '4' };
    case 'alto': return { shape: 'C', line: '3' };
    case 'tenor': return { shape: 'C', line: '4' };
    default: return { shape: 'G', line: '2' };
  }
}

/**
 * Generate MEI document for a worksheet section
 * @param section - The worksheet section to render
 * @param keyFifths - Key signature (positive = sharps, negative = flats)
 * @param showAnswers - If true, show chord names; if false, answers will be overlaid as SVG lines
 */
export function sectionToMei(section: WorksheetSection, keyFifths: number = 0, showAnswers: boolean = false): string {
  const { staff } = section;
  const clef = clefToMei(staff.clef);
  
  // Generate measures
  const measures = staff.measures.map((measure, idx) => {
    const elements = measure.elements.map(elementToMei).join('\n          ');
    const hasElements = measure.elements.length > 0;
    
    // If no elements, add a whole rest as placeholder
    const content = hasElements ? elements : `<mRest xml:id="rest-${measure.id}" />`;
    
    // Find chord element to get its display name for <harm>
    const chordElement = measure.elements.find(el => el.type === 'chord') as ChordElement | undefined;
    
    // Add <harm> element if showing answers and there's a chord with a display name
    let harmElement = '';
    if (showAnswers && chordElement?.displayName) {
      harmElement = `\n        <harm tstamp="1" place="below" staff="1">${chordElement.displayName}</harm>`;
    }
    
    return `<measure xml:id="${measure.id}" n="${idx + 1}">
        <staff n="1">
          <layer n="1">
            ${content}
          </layer>
        </staff>${harmElement}
      </measure>`;
  }).join('\n      ');
  
  return `<?xml version="1.0" encoding="UTF-8"?>
<?xml-model href="https://music-encoding.org/schema/5.0/mei-basic.rng" type="application/xml" schematypens="http://relaxng.org/ns/structure/1.0"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <meiHead>
    <fileDesc>
      <titleStmt>
        <title>${section.title || 'Worksheet'}</title>
      </titleStmt>
      <pubStmt></pubStmt>
    </fileDesc>
  </meiHead>
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="${clef.shape}" clef.line="${clef.line}" 
                        meter.count="4" meter.unit="4" key.sig="${keyFifths >= 0 ? keyFifths + 's' : Math.abs(keyFifths) + 'f'}" />
            </staffGrp>
          </scoreDef>
          <section>
            ${measures}
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>`;
}

/**
 * Render a worksheet section to SVG
 * @param section - The worksheet section to render
 * @param keyFifths - Key signature (positive = sharps, negative = flats)
 * @param showAnswers - If true, show chord names below staff
 */
export async function renderSection(section: WorksheetSection, keyFifths: number = 0, showAnswers: boolean = false): Promise<string> {
  console.log('[Verovio] renderSection called for:', section.id, 'showAnswers:', showAnswers);
  
  try {
    const tk = await initVerovio();
    const mei = sectionToMei(section, keyFifths, showAnswers);
    
    console.log('[Verovio] Loading MEI data...');
    const loaded = tk.loadData(mei);
    console.log('[Verovio] MEI loaded:', loaded);
    
    if (!loaded) {
      console.error('[Verovio] Failed to load MEI:', mei.substring(0, 500));
      throw new Error('Failed to load MEI data');
    }
    
    console.log('[Verovio] Rendering to SVG...');
    const svg = tk.renderToSVG(1, {});
    console.log('[Verovio] SVG rendered, length:', svg.length);
    
    return svg;
  } catch (error) {
    console.error('[Verovio] Render error:', error);
    throw error;
  }
}

/**
 * Render a complete score to SVG (all sections)
 */
export async function renderScore(score: Score): Promise<string[]> {
  const svgs: string[] = [];
  
  for (const section of score.sections) {
    const svg = await renderSection(section, score.keySignature.fifths);
    svgs.push(svg);
  }
  
  return svgs;
}

/**
 * Get element ID from SVG click event
 */
export function getElementIdFromClick(event: MouseEvent): string | null {
  const target = event.target as SVGElement;
  
  // Walk up the DOM to find an element with an ID
  let current: Element | null = target;
  while (current && current.tagName !== 'svg') {
    if (current.id && current.classList.contains('note')) {
      return current.id;
    }
    if (current.id && current.classList.contains('chord')) {
      return current.id;
    }
    current = current.parentElement;
  }
  
  return null;
}

/**
 * Calculate pitch from staff Y position
 * Returns the pitch that corresponds to clicking at a given Y coordinate
 */
export function pitchFromStaffPosition(
  y: number, 
  staffTop: number, 
  staffHeight: number, 
  clef: ClefType
): Pitch {
  // Staff has 5 lines and 4 spaces = 9 positions visible
  // Each position is staffHeight / 8
  const positionHeight = staffHeight / 8;
  
  // Calculate which line/space was clicked (0 = top line, 8 = bottom line)
  const rawPosition = Math.round((y - staffTop) / positionHeight);
  const position = Math.max(-4, Math.min(12, rawPosition)); // Allow ledger lines
  
  // Map position to pitch based on clef
  // Treble clef: line 0 (top) = F5, line 4 (middle) = B4, line 8 (bottom) = E4
  // Bass clef: line 0 (top) = A3, line 4 (middle) = D3, line 8 (bottom) = G2
  
  const treblePitches: Array<{ note: Pitch['note']; octave: Pitch['octave'] }> = [
    { note: 'f', octave: 5 }, // line 0
    { note: 'e', octave: 5 }, // space
    { note: 'd', octave: 5 }, // line 1
    { note: 'c', octave: 5 }, // space
    { note: 'b', octave: 4 }, // line 2
    { note: 'a', octave: 4 }, // space
    { note: 'g', octave: 4 }, // line 3
    { note: 'f', octave: 4 }, // space
    { note: 'e', octave: 4 }, // line 4
  ];
  
  const bassPitches: Array<{ note: Pitch['note']; octave: Pitch['octave'] }> = [
    { note: 'a', octave: 3 }, // line 0
    { note: 'g', octave: 3 }, // space
    { note: 'f', octave: 3 }, // line 1
    { note: 'e', octave: 3 }, // space
    { note: 'd', octave: 3 }, // line 2
    { note: 'c', octave: 3 }, // space
    { note: 'b', octave: 2 }, // line 3
    { note: 'a', octave: 2 }, // space
    { note: 'g', octave: 2 }, // line 4
  ];
  
  const pitches = clef === 'bass' ? bassPitches : treblePitches;
  
  // Handle ledger lines above and below
  const adjustedPosition = position + 4; // Shift so 0-8 is the main staff
  
  if (adjustedPosition < 0) {
    // Above staff - extrapolate upward
    const { note, octave } = pitches[0];
    const steps = -adjustedPosition;
    return adjustPitchBySteps({ note, octave, accidental: null }, steps);
  } else if (adjustedPosition >= pitches.length) {
    // Below staff - extrapolate downward
    const { note, octave } = pitches[pitches.length - 1];
    const steps = -(adjustedPosition - pitches.length + 1);
    return adjustPitchBySteps({ note, octave, accidental: null }, steps);
  }
  
  const { note, octave } = pitches[adjustedPosition];
  return { note, octave, accidental: null };
}

/**
 * Adjust a pitch by a number of diatonic steps
 */
function adjustPitchBySteps(pitch: Pitch, steps: number): Pitch {
  const noteOrder: Pitch['note'][] = ['c', 'd', 'e', 'f', 'g', 'a', 'b'];
  const currentIndex = noteOrder.indexOf(pitch.note);
  
  let newIndex = currentIndex + steps;
  let octaveAdjust = 0;
  
  while (newIndex < 0) {
    newIndex += 7;
    octaveAdjust -= 1;
  }
  while (newIndex >= 7) {
    newIndex -= 7;
    octaveAdjust += 1;
  }
  
  return {
    note: noteOrder[newIndex],
    octave: Math.max(0, Math.min(8, pitch.octave + octaveAdjust)) as Pitch['octave'],
    accidental: pitch.accidental,
  };
}
