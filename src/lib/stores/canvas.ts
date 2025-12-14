import { writable } from 'svelte/store';
import type { MusicBlock, MeasureContent } from '../types/blocks';
import { snapToGrid } from '../utils/snap-grid';

function createCanvasStore() {
  const { subscribe, set, update } = writable<MusicBlock[]>([]);

  return {
    subscribe,
    
    addMeasure: (x: number, y: number, clef: 'treble' | 'bass' = 'treble') => {
      const newBlock: MusicBlock = {
        id: crypto.randomUUID(),
        blockType: 'measure',
        x,
        y,
        width: 150,
        height: 100,
        content: { type: 'empty' },
        timeSignature: '4/4',
        clef,
        svgContent: undefined,
        isRendered: false
      };
      update(blocks => [...blocks, newBlock]);
      return newBlock.id;
    },
    
    updateMeasureContent: (id: string, content: MeasureContent) => {
      update(blocks => 
        blocks.map(block => 
          block.id === id 
            ? { ...block, content, svgContent: undefined, isRendered: false } // Clear SVG to trigger re-render
            : block
        )
      );
    },
    
    updatePosition: (id: string, x: number, y: number) => {
      const snapped = snapToGrid(x, y);
      update(blocks => 
        blocks.map(block => 
          block.id === id 
            ? { ...block, x: snapped.x, y: snapped.y }
            : block
        )
      );
    },
    
    updateBlock: (id: string, updates: Partial<MusicBlock>) => {
      update(blocks => 
        blocks.map(block => 
          block.id === id ? { ...block, ...updates } : block
        )
      );
    },
    
    removeBlock: (id: string) => {
      update(blocks => blocks.filter(block => block.id !== id));
    },
    
    getBlock: (id: string) => {
      let block: MusicBlock | undefined;
      subscribe(blocks => {
        block = blocks.find(b => b.id === id);
      })();
      return block;
    },
    
    setBlocks: (blocks: MusicBlock[]) => {
      set(blocks);
    },
    
    clear: () => set([])
  };
}

export const canvasStore = createCanvasStore();