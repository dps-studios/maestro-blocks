import { onMount, Show, createSignal } from 'solid-js';
import ScoreCanvas from './lib/components/ScoreCanvas';
import Sidebar from './lib/components/Sidebar';
import WorksheetModeSidebar from './lib/components/WorksheetModeSidebar';
import { scoreStore, editorStore } from './lib/stores/score';
import { theme, isDark } from './lib/hooks/useTheme';
import { captureWorksheetSvg, exportPdf } from './lib/services/export';

export default function App() {
  const score = () => scoreStore.state;
  const [sidebarMode, setSidebarMode] = createSignal<'edit' | 'worksheet'>('worksheet');

  onMount(() => {
    console.log('Maestro Blocks - Paper & Ink Edition loaded');
    theme.init();
  });

  function toggleTheme() {
    theme.toggle();
  }

  // Worksheet mode handlers
  async function handleGenerateWorksheet() {
    await scoreStore.generateRandomWorksheet();
  }

  function handleClearAll() {
    scoreStore.clearAllChords();
  }

  function handleToggleAnswers() {
    scoreStore.toggleAnswers();
  }

  // Check if there's a selected chord
  const hasSelection = () => editorStore.state.selectedChordId !== null;

  // Export handlers
  const [isExporting, setIsExporting] = createSignal(false);
  
  async function handleExportPdf() {
    if (isExporting()) return;
    
    setIsExporting(true);
    try {
      const svgContent = captureWorksheetSvg();
      if (!svgContent) {
        console.error('Failed to capture worksheet SVG');
        return;
      }
      
      const title = score().metadata.title || 'worksheet';
      const success = await exportPdf(svgContent, title);
      
      if (success) {
        console.log('PDF exported successfully');
      }
    } catch (error) {
      console.error('Export failed:', error);
    } finally {
      setIsExporting(false);
    }
  }

  // Detect macOS for traffic light spacer
  const isMacOS = typeof navigator !== 'undefined' && navigator.platform.toLowerCase().includes('mac');

  return (
    <div class="app paper-texture">
      {/* Header */}
      <header class="app-header">
        <Show when={isMacOS}>
          <div class="traffic-light-spacer"></div>
        </Show>
        
        <div class="logo">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 18V5l12-2v13"/>
            <circle cx="6" cy="18" r="3"/>
            <circle cx="18" cy="16" r="3"/>
          </svg>
          <h1>Maestro Blocks</h1>
        </div>
        
        <div class="header-center">
          <input 
            type="text" 
            class="title-input"
            value={score().metadata.title}
            onInput={(e) => scoreStore.updateMetadata({ title: e.currentTarget.value })}
            placeholder="Untitled Worksheet"
          />
        </div>
        
        <div class="header-actions">
          <button class="header-btn" onClick={toggleTheme} title="Toggle theme">
            <Show 
              when={isDark()}
              fallback={
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
                </svg>
              }
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="5"/>
                <line x1="12" y1="1" x2="12" y2="3"/>
                <line x1="12" y1="21" x2="12" y2="23"/>
                <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
                <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
                <line x1="1" y1="12" x2="3" y2="12"/>
                <line x1="21" y1="12" x2="23" y2="12"/>
                <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
                <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
              </svg>
            </Show>
          </button>
          <button 
            class="header-btn" 
            title="Export as PDF" 
            onClick={handleExportPdf}
            disabled={isExporting()}
          >
            <Show
              when={!isExporting()}
              fallback={
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="spin">
                  <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
                </svg>
              }
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
                <line x1="12" y1="18" x2="12" y2="12"/>
                <line x1="9" y1="15" x2="15" y2="15"/>
              </svg>
            </Show>
            <span class="btn-label">{isExporting() ? 'Exporting...' : 'Export'}</span>
          </button>
        </div>
      </header>

      {/* Main Layout */}
      <div class="app-body">
        <Show 
          when={sidebarMode() === 'worksheet'}
          fallback={<Sidebar />}
        >
          <WorksheetModeSidebar
            onGenerateWorksheet={handleGenerateWorksheet}
            onClearAll={handleClearAll}
            onToggleAnswers={handleToggleAnswers}
            hasSelection={hasSelection()}
          />
        </Show>
        <main class="main-content paper-texture">
          <ScoreCanvas />
        </main>
      </div>

      {/* Status Bar */}
      <footer class="status-bar">
        <div class="status-left">
          <span class="status-item">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M9 18V5l12-2v13"/>
              <circle cx="6" cy="18" r="3"/>
              <circle cx="18" cy="16" r="3"/>
            </svg>
            {score().sections.length} section{score().sections.length !== 1 ? 's' : ''}
          </span>
          <span class="status-divider"></span>
          <span class="status-item">
            {score().keySignature.fifths === 0 ? 'C Major' : 
             score().keySignature.fifths > 0 ? `${score().keySignature.fifths} sharp${score().keySignature.fifths !== 1 ? 's' : ''}` :
             `${Math.abs(score().keySignature.fifths)} flat${Math.abs(score().keySignature.fifths) !== 1 ? 's' : ''}`}
          </span>
          <span class="status-divider"></span>
          <span class="status-item">
            {score().timeSignature.beats}/{score().timeSignature.beatType}
          </span>
        </div>
        <div class="status-right">
          <span class="status-item muted">
            VexFlow rendering
          </span>
        </div>
      </footer>
    </div>
  );
}
