import { writable, derived } from 'svelte/store';
import type { 
  WorksheetConfig, 
  WorksheetSection, 
  EditableElement, 
  WorksheetType,
  ChordNamingParams,
  EditableElementType 
} from '../types/worksheet';
import { invoke } from '../utils/tauri-api';

function createWorksheetStore() {
  const { subscribe, set, update } = writable<WorksheetConfig | null>(null);

  return {
    subscribe,
    
    // Generate worksheet from template
    generateFromTemplate: async (type: WorksheetType, params: any) => {
      try {
        let config: WorksheetConfig;
        
        switch (type) {
          case 'chord-naming':
            config = await invoke('generate_chord_naming_template', { params });
            break;
          default:
            throw new Error(`Unsupported worksheet type: ${type}`);
        }
        
        set(config);
        return config;
      } catch (error) {
        console.error('Failed to generate worksheet:', error);
        throw error;
      }
    },
    
    // Generate final worksheet document
    generateDocument: async (config: WorksheetConfig) => {
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
    
    // Update worksheet configuration
    updateConfig: (updates: Partial<WorksheetConfig>) => {
      update(config => {
        if (!config) return null;
        return { ...config, ...updates };
      });
    },
    
    // Update section
    updateSection: (sectionId: string, updates: Partial<WorksheetSection>) => {
      update(config => {
        if (!config) return null;
        return {
          ...config,
          sections: config.sections.map(section =>
            section.id === sectionId 
              ? { ...section, ...updates }
              : section
          )
        };
      });
    },
    
    // Update element
    updateElement: (elementId: string, updates: Partial<EditableElement>) => {
      update(config => {
        if (!config) return null;
        return {
          ...config,
          sections: config.sections.map(section => ({
            ...section,
            elements: section.elements.map(element =>
              element.id === elementId 
                ? { ...element, ...updates }
                : element
            )
          }))
        };
      });
    },
    
    // Add new element
    addElement: (sectionId: string, element: Omit<EditableElement, 'id'>) => {
      const newElement: EditableElement = {
        ...element,
        id: crypto.randomUUID(),
      };
      
      update(config => {
        if (!config) return null;
        return {
          ...config,
          sections: config.sections.map(section =>
            section.id === sectionId 
              ? { ...section, elements: [...section.elements, newElement] }
              : section
          )
        };
      });
      
      return newElement.id;
    },
    
    // Remove element
    removeElement: (elementId: string) => {
      update(config => {
        if (!config) return null;
        return {
          ...config,
          sections: config.sections.map(section => ({
            ...section,
            elements: section.elements.filter(element => element.id !== elementId)
          }))
        };
      });
    },
    
    // Toggle answers visibility
    toggleAnswers: () => {
      update(config => {
        if (!config) return null;
        return {
          ...config,
          globalSettings: {
            ...config.globalSettings,
            showAnswers: !config.globalSettings.showAnswers
          }
        };
      });
    },
    
    // Clear worksheet
    clear: () => set(null),
    
    // Set worksheet directly
    setWorksheet: (config: WorksheetConfig) => set(config)
  };
}

export const worksheetStore = createWorksheetStore();

// Derived stores for convenience
export const currentWorksheet = derived(worksheetStore, $worksheet => $worksheet);
export const worksheetSections = derived(worksheetStore, $worksheet => $worksheet?.sections || []);
export const globalSettings = derived(worksheetStore, $worksheet => $worksheet?.globalSettings);
export const showAnswers = derived(globalSettings, $settings => $settings?.showAnswers || false);

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