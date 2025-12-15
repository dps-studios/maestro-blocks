/**
 * Worksheet Viewer Component
 * Displays complete LilyPond-generated worksheet documents with interactive elements
 */

import { createSignal, Show } from 'solid-js';
import { worksheetStore, showAnswers } from '../stores/worksheet';
import type { WorksheetConfig } from '../types/worksheet';

interface WorksheetViewerProps {
  worksheet: WorksheetConfig | null;
  width?: number;
  height?: number;
}

export default function WorksheetViewer(props: WorksheetViewerProps) {
  const [svgContent, setSvgContent] = createSignal('');
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  let svgElement: HTMLElement | null = null;

  // Generate worksheet when props change
  const generateWorksheet = async () => {
    const worksheet = props.worksheet;
    if (!worksheet || !worksheet.sections || worksheet.sections.length === 0) {
      console.log('Skipping worksheet generation - no valid worksheet data');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      console.log('Generating worksheet with config:', worksheet);
      const response = await worksheetStore.generateDocument(worksheet);
      if (response) {
        setSvgContent(response.svg_content);

        // Wait for SVG to be rendered, then add interactivity
        setTimeout(() => {
          addInteractiveElements();
        }, 100);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to generate worksheet');
      console.error('Worksheet generation error:', err);
    } finally {
      setLoading(false);
    }
  };

  function addInteractiveElements() {
    if (!svgElement) return;

    const interactiveNodes = svgElement.querySelectorAll('.interactive-note, .interactive-chord, .interactive-rest');

    interactiveNodes.forEach((node) => {
      (node as HTMLElement).style.cursor = 'pointer';
      (node as HTMLElement).addEventListener('click', handleElementClick);
      (node as HTMLElement).addEventListener('mouseenter', handleElementHover);
      (node as HTMLElement).addEventListener('mouseleave', handleElementLeave);
    });
  }

  function handleElementClick(event: Event) {
    const target = event.target as HTMLElement;
    const elementId = target.getAttribute('data-element-id');
    const elementType = target.getAttribute('data-element-type');

    if (elementId) {
      console.log('Clicked element:', { elementId, elementType });
      target.dispatchEvent(new CustomEvent('elementClick', {
        detail: { elementId, elementType, target },
        bubbles: true
      }));
    }
  }

  function handleElementHover(event: Event) {
    const target = event.target as HTMLElement;
    target.style.opacity = '0.7';
  }

  function handleElementLeave(event: Event) {
    const target = event.target as HTMLElement;
    target.style.opacity = '1';
  }

  function toggleAnswers() {
    worksheetStore.toggleAnswers();
  }

  function downloadWorksheet() {
    const content = svgContent();
    if (!content) return;

    const blob = new Blob([content], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `worksheet-${props.worksheet?.title || 'untitled'}.svg`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  return (
    <div class="worksheet-viewer" style={`width: ${props.width || 800}px; height: ${props.height || 1000}px;`}>
      <Show when={loading()}>
        <div class="loading">
          <div class="spinner"></div>
          <p>Generating worksheet...</p>
        </div>
      </Show>

      <Show when={!loading() && error()}>
        <div class="error">
          <p><strong>Error:</strong> {error()}</p>
          <button onClick={generateWorksheet}>Try Again</button>
        </div>
      </Show>

      <Show when={!loading() && !error() && svgContent()}>
        <div class="toolbar">
          <button
            class={`answers-toggle ${showAnswers() ? 'active' : ''}`}
            onClick={toggleAnswers}
          >
            {showAnswers() ? 'Hide Answers' : 'Show Answers'}
          </button>
          <button onClick={downloadWorksheet}>
            Download SVG
          </button>
        </div>

        <div class="svg-container">
          <div innerHTML={svgContent()} />
        </div>
      </Show>

      <Show when={!loading() && !error() && !svgContent()}>
        <div class="empty">
          <p>No worksheet to display. Generate a worksheet to get started.</p>
        </div>
      </Show>
    </div>
  );
}
