<!-- 
  MusicBlock Component
  Draggable music notation block with LilyPond rendering
  
  Props:
  - block: MusicBlock - Block data including position, type, and notation
-->

<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  // import { invoke } from '@tauri-apps/api/core';
  import { Group, Image, Rect, Text, Circle } from 'svelte-konva';
  import { svgStringToImageUrl, cleanupImageUrl, extractSvgDimensions } from '../utils/svg-to-image';
  import type { MusicBlock } from '../types/blocks';
  
  let { block }: { block: MusicBlock } = $props();
  
  const dispatch = createEventDispatcher();
  
  // Component state
  let imageUrl = $state<string | null>(null);
  let isLoading = $state(true);
  let hasError = $state(false);
  let dimensions = $state({ width: 200, height: 80 });
  let konvaImage = $state<any>(null);
  
  // Update dimensions when block changes
  $effect(() => {
    dimensions.width = block.width;
    dimensions.height = block.height;
  });
  
// Render LilyPond notation to SVG
  async function renderNotation() {
    if (block.blockType !== 'measure') return;
    
    isLoading = true;
    hasError = false;
    
    try {
      // Generate LilyPond notation from measure content
      const { generateMeasureLilyPond } = await import('../utils/notation-templates');
      const lilypondNotation = generateMeasureLilyPond(
        block.content,
        block.clef,
        block.timeSignature
      );
      
      // Call Rust backend for SVG generation
      // TODO: Fix Tauri import issue
      const svgContent = await new Promise<string>((resolve, reject) => {
        // Mock implementation for now
        setTimeout(() => {
          resolve(`<svg width="150" height="100" xmlns="http://www.w3.org/2000/svg">
            <rect width="150" height="100" fill="white" stroke="black" stroke-width="1"/>
            <text x="75" y="50" text-anchor="middle" font-family="Arial" font-size="12">
              ${block.content.type === 'chord' ? block.content.symbol : 
                block.content.type === 'empty' ? 'Empty' : 
                block.content.type}
            </text>
          </svg>`);
        }, 100);
      });
      
      // const svgContent = await invoke<string>('render_lilypond', {
      //   notation: lilypondNotation
      // });
      
      // Convert SVG to image URL for Konva
      imageUrl = await svgStringToImageUrl(svgContent);
      
      // Extract dimensions from SVG
      const svgDims = extractSvgDimensions(svgContent);
      dimensions = {
        width: Math.max(svgDims.width, 100),
        height: Math.max(svgDims.height, 60)
      };
      
      // Create Konva image
      const imageObj = new window.Image();
      imageObj.onload = () => {
        konvaImage = imageObj;
        isLoading = false;
      };
      imageObj.onerror = () => {
        hasError = true;
        isLoading = false;
      };
      imageObj.src = imageUrl;
      
      // Update block with rendered content
      dispatch('rendered', {
        id: block.id,
        svgContent,
        dimensions
      });
      
    } catch (error) {
      hasError = true;
      isLoading = false;
      console.error('LilyPond rendering failed:', error);
    }
  }
  
  // Handle drag events
  function handleDragEnd(event: any) {
    const { target } = event;
    const newPos = target.position();
    
    dispatch('moved', {
      id: block.id,
      x: newPos.x,
      y: newPos.y
    });
  }
  
  // Handle click events
  function handleClick() {
    dispatch('selected', { id: block.id });
  }
  
  // Effect for lifecycle and reactive rendering
  $effect(() => {
    // Render on mount
    if (!block.svgContent) {
      renderNotation();
    }
    
    // Cleanup on destroy
    return () => {
      if (imageUrl) {
        cleanupImageUrl(imageUrl);
      }
    };
  });
</script>

{#if hasError}
  <!-- Error state -->
  <Group>
    <Rect
      x={block.x}
      y={block.y}
      width={block.width}
      height={block.height}
      fill="#fee"
      stroke="#f00"
      strokeWidth={2}
      cornerRadius={4}
    />
    <Text
      x={block.x + block.width / 2}
      y={block.y + block.height / 2}
      text="Rendering Error"
      align="center"
      verticalAlign="middle"
      fontSize={12}
      fill="#f00"
    />
  </Group>
{:else if isLoading}
  <!-- Loading state -->
  <Group>
    <Rect
      x={block.x}
      y={block.y}
      width={block.width}
      height={block.height}
      fill="#f0f0f0"
      stroke="#ccc"
      strokeWidth={1}
      cornerRadius={4}
    />
    <Text
      x={block.x + block.width / 2}
      y={block.y + block.height / 2}
      text="Loading..."
      align="center"
      verticalAlign="middle"
      fontSize={12}
      fill="#666"
    />
    <!-- Loading indicator -->
    <Circle
      x={block.x + block.width - 10}
      y={block.y + 10}
      radius={3}
      fill="#007acc"
    />
  </Group>
{:else if konvaImage}
  <!-- Rendered music block -->
  <Group
    x={block.x}
    y={block.y}
    draggable={true}
    on:dragend={handleDragEnd}
    on:click={handleClick}
  >
    <!-- Music notation image -->
    <Image
      image={konvaImage}
      width={dimensions.width}
      height={dimensions.height}
    />
    
    <!-- Transparent drag handle overlay -->
    <Rect
      x={0}
      y={0}
      width={dimensions.width}
      height={dimensions.height}
      fill="transparent"
      stroke="#007acc"
      strokeWidth={0}
      dash={[5, 5]}
      opacity={0}
    />
  </Group>
{/if}