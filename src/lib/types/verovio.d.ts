/**
 * Type declarations for Verovio
 */

declare module 'verovio/wasm' {
  export interface VerovioModule {
    // WASM module instance
  }
  
  export default function createVerovioModule(): Promise<VerovioModule>;
}

declare module 'verovio/esm' {
  import type { VerovioModule } from 'verovio/wasm';
  
  export interface VerovioOptions {
    scale?: number;
    pageWidth?: number;
    pageHeight?: number;
    adjustPageHeight?: boolean;
    pageMarginTop?: number;
    pageMarginBottom?: number;
    pageMarginLeft?: number;
    pageMarginRight?: number;
    font?: string;
    svgAdditionalAttribute?: string[];
    breaks?: 'auto' | 'none' | 'line' | 'smart' | 'encoded';
    header?: 'none' | 'auto' | 'encoded';
    footer?: 'none' | 'auto' | 'encoded';
    spacingStaff?: number;
    spacingSystem?: number;
  }
  
  export class VerovioToolkit {
    constructor(module: VerovioModule);
    
    setOptions(options: VerovioOptions): void;
    getOptions(): VerovioOptions;
    
    loadData(data: string): boolean;
    renderToSVG(page?: number, options?: Record<string, unknown>): string;
    
    getPageCount(): number;
    getTimeForElement(elementId: string): number;
    getElementsAtTime(time: number): string[];
    getElementAttr(elementId: string): Record<string, string>;
    
    getMEI(options?: Record<string, unknown>): string;
    renderToMIDI(): string;
    
    edit(editorAction: Record<string, unknown>): boolean;
    getLog(): string;
    getVersion(): string;
  }
}
