<!-- 
  Worksheet Viewer Component
  Displays complete LilyPond-generated worksheet documents with interactive elements
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { worksheetStore, showAnswers } from '../stores/worksheet';
  import type { WorksheetConfig, InteractiveElement } from '../types/worksheet';
  
  export let worksheet: WorksheetConfig | null = null;
  export let width = 800;
  export let height = 1000;
  
  let svgContent = '';
  let loading = false;
  let error: string | null = null;
  let interactiveElements: InteractiveElement[] = [];
  let svgElement: HTMLElement | null = null;
  
  // Reactive to worksheet changes and showAnswers
  $: if (worksheet && worksheet.sections && worksheet.sections.length > 0) {
    generateWorksheet();
  }
  
  async function generateWorksheet() {
    if (!worksheet || !worksheet.sections || worksheet.sections.length === 0) {
      console.log('Skipping worksheet generation - no valid worksheet data');
      return;
    }
    
    loading = true;
    error = null;
    
    try {
      console.log('Generating worksheet with config:', worksheet);
      const response = await worksheetStore.generateDocument(worksheet);
      if (response) {
        svgContent = response.svg_content;
        interactiveElements = response.interactive_elements || [];
        
        // Wait for SVG to be rendered, then add interactivity
        setTimeout(() => {
          addInteractiveElements();
        }, 100);
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to generate worksheet';
      console.error('Worksheet generation error:', err);
    } finally {
      loading = false;
    }
  }
  
  function addInteractiveElements() {
    if (!svgElement) return;
    
    // Find all elements with interactive classes
    const interactiveNodes = svgElement.querySelectorAll('.interactive-note, .interactive-chord, .interactive-rest');
    
    interactiveNodes.forEach((node) => {
      // Add click handlers and visual feedback
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
      // TODO: Open modal for editing element content
      // Could dispatch a custom event for parent component to handle
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
  
  function handleSvgLoad() {
    svgElement = document.querySelector('#worksheet-svg');
    addInteractiveElements();
  }
  
  function toggleAnswers() {
    worksheetStore.toggleAnswers();
  }
  
  function downloadWorksheet() {
    if (!svgContent) return;
    
    const blob = new Blob([svgContent], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `worksheet-${worksheet?.title || 'untitled'}.svg`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }
</script>

<div class="worksheet-viewer" style="width: {width}px; height: {height}px;">
  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Generating worksheet...</p>
    </div>
  {:else if error}
    <div class="error">
      <p><strong>Error:</strong> {error}</p>
      <button on:click={generateWorksheet}>Try Again</button>
    </div>
  {:else if svgContent}
    <div class="toolbar">
      <button 
        class="answers-toggle" 
        class:active={$showAnswers}
        on:click={toggleAnswers}
      >
        {$showAnswers ? 'Hide Answers' : 'Show Answers'}
      </button>
      <button on:click={downloadWorksheet}>
        Download SVG
      </button>
    </div>
    
    <div class="svg-container">
      {@html svgContent}
    </div>
  {:else}
    <div class="empty">
      <p>No worksheet to display. Generate a worksheet to get started.</p>
    </div>
  {/if}
</div>

<style>
  .worksheet-viewer {
    border: 1px solid #ddd;
    border-radius: 8px;
    overflow: hidden;
    background: white;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    display: flex;
    flex-direction: column;
  }
  
  .toolbar {
    padding: 12px;
    background: #f8f9fa;
    border-bottom: 1px solid #e9ecef;
    display: flex;
    gap: 8px;
    align-items: center;
  }
  
  .toolbar button {
    padding: 8px 16px;
    border: 1px solid #dee2e6;
    border-radius: 4px;
    background: white;
    cursor: pointer;
    font-size: 14px;
    transition: all 0.2s ease;
  }
  
  .toolbar button:hover {
    background: #e9ecef;
  }
  
  .answers-toggle.active {
    background: #007bff;
    color: white;
    border-color: #007bff;
  }
  
  .svg-container {
    flex: 1;
    overflow: auto;
    padding: 20px;
    display: flex;
    justify-content: center;
    align-items: flex-start;
  }
  
  .svg-container :global(svg) {
    max-width: 100%;
    height: auto;
  }
  
  .loading, .error, .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    padding: 40px;
    text-align: center;
  }
  
  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid #f3f3f3;
    border-top: 4px solid #007bff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 16px;
  }
  
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
  
  .error {
    color: #dc3545;
  }
  
  .error button {
    margin-top: 16px;
    padding: 8px 16px;
    background: #dc3545;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .error button:hover {
    background: #c82333;
  }
</style>