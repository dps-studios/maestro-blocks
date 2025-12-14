export type MeasureContent = 
  | { type: 'empty' }
  | { type: 'chord', symbol: string } // e.g., "Cmaj7"
  | { type: 'notes', notation: string } // LilyPond notation
  | { type: 'rest' };

export interface MusicBlock {
  id: string;
  blockType: 'measure';
  x: number;
  y: number;
  width: number; // Fixed measure width (150px)
  height: number; // Fixed staff height (100px)
  content: MeasureContent;
  timeSignature: string; // e.g., "4/4"
  clef: 'treble' | 'bass';
  svgContent?: string;
  isRendered: boolean;
}

export interface BlockProperties {
  title?: string;
  description?: string;
  timeSignature?: string;
  keySignature?: string;
  clef?: 'treble' | 'bass' | 'alto' | 'tenor';
}

export type BlockType = 'note' | 'chord' | 'rest' | 'time-signature' | 'key-signature' | 'clef' | 'staff';

export interface CanvasState {
  blocks: MusicBlock[];
  selectedBlockId: string | null;
  isDragging: boolean;
  scale: number;
}