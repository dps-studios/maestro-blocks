<!-- 
  Canvas Component
  Main workspace for draggable music notation blocks
-->

<script lang="ts">
  
  import { Stage, Layer, Line, Rect } from 'svelte-konva';
  import { canvasStore } from '../stores/canvas';
  import MusicBlockComponent from './MusicBlock.svelte';
  import type { MusicBlock } from '../types/blocks';
  
  // Canvas configuration
  const CANVAS_WIDTH = 1200;
  const CANVAS_HEIGHT = 800;
  const MEASURE_WIDTH = 150;
  const MEASURE_HEIGHT = 100;
  const GRID_SIZE = 20; // Visual grid size for alignment
  
  // Store subscription using runes
  let blocks = $state<MusicBlock[]>([]);
  
  // Subscribe to canvas store
  $effect(() => {
    const unsubscribe = canvasStore.subscribe(value => {
      blocks = value;
    });
    return unsubscribe;
  });
  
  // Handle block events
  function handleBlockRendered(event: CustomEvent) {
    const { id, svgContent, dimensions } = event.detail;
    canvasStore.updateBlock(id, { 
      svgContent, 
      width: dimensions.width, 
      height: dimensions.height,
      isRendered: true 
    });
  }
  
  function handleBlockMoved(event: CustomEvent) {
    const { id, x, y } = event.detail;
    canvasStore.updatePosition(id, x, y);
  }
  
  function handleBlockSelected(event: CustomEvent) {
    const { id } = event.detail;
    console.log('Measure selected:', id);
  }
  
  // Handle canvas click to deselect
  function handleCanvasClick() {
    console.log('Canvas clicked');
  }
  
  // Handle chord drop on canvas
  function handleDrop(event: DragEvent) {
    event.preventDefault();
    const chord = event.dataTransfer?.getData('text/plain');
    if (!chord) return;
    
    // Find the measure at the drop position
    const stage = event.currentTarget as HTMLElement;
    if (!stage) return;
    
    const rect = stage.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;
    
    // Find which measure was dropped on
    const targetMeasure = blocks.find(block => 
      x >= block.x && x <= block.x + block.width &&
      y >= block.y && y <= block.y + block.height
    );
    
    if (targetMeasure) {
      canvasStore.updateMeasureContent(targetMeasure.id, {
        type: 'chord',
        symbol: chord
      });
    }
  }
  
  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    event.dataTransfer!.dropEffect = 'copy';
  }
</script>

<div class="canvas-container">
  <Stage 
    width={CANVAS_WIDTH} 
    height={CANVAS_HEIGHT}
    on:click={handleCanvasClick}
    on:drop={handleDrop}
    on:dragover={handleDragOver}
  >
    <Layer>
      <!-- Sheet music background -->
      <Rect
        x={0}
        y={0}
        width={CANVAS_WIDTH}
        height={CANVAS_HEIGHT}
        fill="#fafafa"
      />
      
      <!-- Grid lines for measure alignment -->
      {#each Array(Math.floor(CANVAS_WIDTH / MEASURE_WIDTH)) as _, gridX}
        <Line
          points={[gridX * MEASURE_WIDTH, 0, gridX * MEASURE_WIDTH, CANVAS_HEIGHT]}
          stroke="#f0f0f0"
          strokeWidth={1}
          dash={[5, 5]}
        />
      {/each}
      
      {#each Array(Math.floor(CANVAS_HEIGHT / MEASURE_HEIGHT)) as _, gridY}
        <Line
          points={[0, gridY * MEASURE_HEIGHT, CANVAS_WIDTH, gridY * MEASURE_HEIGHT]}
          stroke="#f0f0f0"
          strokeWidth={1}
          dash={[5, 5]}
        />
      {/each}
    </Layer>
    
    <Layer>
      <!-- Render all music blocks -->
      {#each blocks as block (block.id)}
        <MusicBlockComponent 
          {block}
          on:rendered={handleBlockRendered}
          on:moved={handleBlockMoved}
          on:selected={handleBlockSelected}
        />
      {/each}
    </Layer>
  </Stage>
</div>

<style>
  .canvas-container {
    border: 1px solid #ddd;
    border-radius: 8px;
    overflow: hidden;
    background: white;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }
</style>