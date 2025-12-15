/**
 * Worksheet store for document generation
 * 
 * SolidJS implementation using createStore for fine-grained reactivity
 */

import { createStore } from 'solid-js/store';
import { createMemo } from 'solid-js';
import type { 
  WorksheetConfig, 
  WorksheetSection, 
  EditableElement, 
  WorksheetType,
} from '../types/worksheet';
import { invoke } from '../utils/tauri-api';

const [worksheet, setWorksheet] = createStore<{ config: WorksheetConfig | null }>({
  config: null,
});

export const worksheetStore = {
  /** Get the current worksheet config (reactive) */
  get state() {
    return worksheet.config;
  },
  
  /** Generate worksheet from template */
  async generateFromTemplate(type: WorksheetType, params: unknown): Promise<WorksheetConfig | null> {
    try {
      let config: WorksheetConfig;
      
      switch (type) {
        case 'chord-naming':
          config = await invoke('generate_chord_naming_template', { params });
          break;
        default:
          throw new Error(`Unsupported worksheet type: ${type}`);
      }
      
      setWorksheet('config', config);
      return config;
    } catch (error) {
      console.error('Failed to generate worksheet:', error);
      throw error;
    }
  },
  
  /** Generate final worksheet document */
  async generateDocument(config: WorksheetConfig) {
    try {
      const response = await invoke('generate_worksheet', { 
        config 
      });
      return response;
    } catch (error) {
      console.error('Failed to generate worksheet document:', error);
      throw error;
    }
  },
  
  /** Update worksheet configuration */
  updateConfig(updates: Partial<WorksheetConfig>) {
    setWorksheet('config', (config) => {
      if (!config) return null;
      return { ...config, ...updates };
    });
  },
  
  /** Update section */
  updateSection(sectionId: string, updates: Partial<WorksheetSection>) {
    setWorksheet('config', (config) => {
      if (!config) return null;
      return {
        ...config,
        sections: config.sections.map((section) =>
          section.id === sectionId 
            ? { ...section, ...updates }
            : section
        ),
      };
    });
  },
  
  /** Update element */
  updateElement(elementId: string, updates: Partial<EditableElement>) {
    setWorksheet('config', (config) => {
      if (!config) return null;
      return {
        ...config,
        sections: config.sections.map((section) => ({
          ...section,
          elements: section.elements.map((element) =>
            element.id === elementId 
              ? { ...element, ...updates }
              : element
          ),
        })),
      };
    });
  },
  
  /** Add new element */
  addElement(sectionId: string, element: Omit<EditableElement, 'id'>): string | null {
    const newElement: EditableElement = {
      ...element,
      id: crypto.randomUUID(),
    };
    
    setWorksheet('config', (config) => {
      if (!config) return null;
      return {
        ...config,
        sections: config.sections.map((section) =>
          section.id === sectionId 
            ? { ...section, elements: [...section.elements, newElement] }
            : section
        ),
      };
    });
    
    return newElement.id;
  },
  
  /** Remove element */
  removeElement(elementId: string) {
    setWorksheet('config', (config) => {
      if (!config) return null;
      return {
        ...config,
        sections: config.sections.map((section) => ({
          ...section,
          elements: section.elements.filter((element) => element.id !== elementId),
        })),
      };
    });
  },
  
  /** Toggle answers visibility */
  toggleAnswers() {
    setWorksheet('config', (config) => {
      if (!config) return null;
      return {
        ...config,
        globalSettings: {
          ...config.globalSettings,
          showAnswers: !config.globalSettings.showAnswers,
        },
      };
    });
  },
  
  /** Clear worksheet */
  clear() {
    setWorksheet('config', null);
  },
  
  /** Set worksheet directly */
  setWorksheet(config: WorksheetConfig) {
    setWorksheet('config', config);
  },
};

// Derived stores for convenience
export const currentWorksheet = createMemo(() => worksheet.config);
export const worksheetSections = createMemo(() => worksheet.config?.sections || []);
export const globalSettings = createMemo(() => worksheet.config?.globalSettings);
export const showAnswers = createMemo(() => worksheet.config?.globalSettings?.showAnswers || false);

// Helper functions for creating elements
export const createChordElement = (
  measure: number, 
  beat: number, 
  chord: string, 
  isAnswer = false
): Omit<EditableElement, 'id'> => ({
  type: 'chord',
  position: { measure, beat },
  content: chord,
  isAnswer: isAnswer,
  isInteractive: true,
});

export const createNoteElement = (
  measure: number, 
  beat: number, 
  note: string, 
  isAnswer = false
): Omit<EditableElement, 'id'> => ({
  type: 'note',
  position: { measure, beat },
  content: note,
  isAnswer: isAnswer,
  isInteractive: true,
});

export const createRestElement = (
  measure: number, 
  beat: number, 
  restDuration: string
): Omit<EditableElement, 'id'> => ({
  type: 'rest',
  position: { measure, beat },
  content: restDuration,
  isAnswer: false,
  isInteractive: false,
});
