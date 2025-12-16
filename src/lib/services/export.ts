import { invoke } from '@tauri-apps/api/core';

// PDF page dimensions in points (72 DPI)
// 8.5 x 11 inches = 612 x 792 points
const PDF_WIDTH = 612;
const PDF_HEIGHT = 792;

// Worksheet paper dimensions in pixels (96 DPI screen)
// 8.5 x 11 inches = 816 x 1056 pixels
const PAPER_WIDTH = 816;

/**
 * Capture the worksheet paper element as an SVG string.
 * 
 * This creates a composite SVG that includes all VexFlow-rendered sections
 * with proper positioning, sized for 8.5x11 inch PDF output.
 * 
 * Note: The Bravura music font is loaded in Rust via fontdb, not embedded here.
 * 
 * @returns SVG markup string ready for PDF conversion
 */
export function captureWorksheetSvg(): string | null {
  const paper = document.querySelector('.worksheet-paper');
  if (!paper) {
    console.error('[export] No .worksheet-paper element found');
    return null;
  }

  // Get all VexFlow SVG containers
  const vexflowContainers = paper.querySelectorAll('.vexflow-container');
  
  // Scale factor from screen pixels (96 DPI) to PDF points (72 DPI)
  const scale = PDF_WIDTH / PAPER_WIDTH;
  
  // Get paper element's position for relative calculations
  const paperRect = paper.getBoundingClientRect();
  
  // Start building composite SVG
  let svgContent = `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" 
     viewBox="0 0 ${PDF_WIDTH} ${PDF_HEIGHT}" 
     width="${PDF_WIDTH}" height="${PDF_HEIGHT}">
`;
  
  // Add white background
  svgContent += `<rect width="100%" height="100%" fill="white"/>`;
  
  // Add worksheet title and instructions
  const header = paper.querySelector('.worksheet-header');
  if (header) {
    const title = header.querySelector('.worksheet-title');
    const instructions = header.querySelector('.worksheet-instructions');
    
    if (title && title.textContent) {
      const titleRect = title.getBoundingClientRect();
      const titleY = (titleRect.top - paperRect.top + titleRect.height * 0.8) * scale;
      const titleX = PDF_WIDTH / 2;
      svgContent += `<text x="${titleX}" y="${titleY}" text-anchor="middle" font-family="Georgia, Times, serif" font-size="18" font-weight="bold">`;
      svgContent += escapeXml(title.textContent);
      svgContent += `</text>`;
      console.log('[export] Added title:', title.textContent, 'at y:', titleY);
    }
    
    if (instructions && instructions.textContent) {
      const instrRect = instructions.getBoundingClientRect();
      const instrY = (instrRect.top - paperRect.top + instrRect.height * 0.8) * scale;
      const instrX = PDF_WIDTH / 2;
      svgContent += `<text x="${instrX}" y="${instrY}" text-anchor="middle" font-family="Georgia, Times, serif" font-size="12" fill="#666666">`;
      svgContent += escapeXml(instructions.textContent);
      svgContent += `</text>`;
      console.log('[export] Added instructions:', instructions.textContent);
    }
  } else {
    console.log('[export] No .worksheet-header found');
  }
  
  // Embed each VexFlow SVG using XMLSerializer for complete serialization
  const serializer = new XMLSerializer();
  
  vexflowContainers.forEach((container, index) => {
    const svg = container.querySelector('svg');
    if (!svg) {
      console.log(`[export] Section ${index}: No SVG found`);
      return;
    }
    
    const containerRect = container.getBoundingClientRect();
    
    // Calculate position relative to paper, scaled to PDF coordinates
    const x = (containerRect.left - paperRect.left) * scale;
    const y = (containerRect.top - paperRect.top) * scale;
    const width = containerRect.width * scale;
    const height = containerRect.height * scale;
    
    console.log(`[export] Section ${index}: x=${x.toFixed(1)}, y=${y.toFixed(1)}, w=${width.toFixed(1)}, h=${height.toFixed(1)}`);
    
    // Get the original SVG dimensions
    const svgWidth = parseFloat(svg.getAttribute('width') || String(containerRect.width));
    const svgHeight = parseFloat(svg.getAttribute('height') || String(containerRect.height));
    
    // Clone the SVG to avoid modifying the original
    const svgClone = svg.cloneNode(true) as SVGSVGElement;
    
    // Remove any existing width/height attributes to let viewBox control sizing
    svgClone.removeAttribute('width');
    svgClone.removeAttribute('height');
    
    // Set viewBox if not present
    if (!svgClone.getAttribute('viewBox')) {
      svgClone.setAttribute('viewBox', `0 0 ${svgWidth} ${svgHeight}`);
    }
    
    // Set new dimensions and position
    svgClone.setAttribute('x', String(x));
    svgClone.setAttribute('y', String(y));
    svgClone.setAttribute('width', String(width));
    svgClone.setAttribute('height', String(height));
    
    // Serialize the complete SVG (preserves all internal structure, styles, defs)
    let svgString = serializer.serializeToString(svgClone);
    
    // Remove the XML declaration if present (we already have one at the top)
    svgString = svgString.replace(/<\?xml[^?]*\?>/g, '');
    
    // Remove redundant xmlns declarations on nested SVG (keep first one clean)
    // The outer SVG already declares the namespace
    svgString = svgString.replace(/xmlns="http:\/\/www\.w3\.org\/2000\/svg"/g, '');
    
    svgContent += svgString;
  });
  
  svgContent += `</svg>`;
  
  return svgContent;
}

/**
 * Escape special XML characters in text content.
 */
function escapeXml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;');
}

/**
 * Export SVG content to a PDF file using native save dialog.
 * 
 * @param svgContent - The SVG markup to convert to PDF
 * @param title - The worksheet title, used for default filename
 * @returns true if export succeeded, false if user cancelled
 */
export async function exportPdf(svgContent: string, title: string): Promise<boolean> {
  const defaultFilename = `${sanitizeFilename(title) || 'worksheet'}.pdf`;
  return await invoke<boolean>('export_pdf', {
    svgContent,
    defaultFilename,
  });
}

/**
 * Export SVG content to a PNG file using native save dialog.
 * 
 * @param svgContent - The SVG markup to convert to PNG
 * @param title - The worksheet title, used for default filename
 * @returns true if export succeeded, false if user cancelled
 */
export async function exportPng(svgContent: string, title: string): Promise<boolean> {
  const defaultFilename = `${sanitizeFilename(title) || 'worksheet'}.png`;
  return await invoke<boolean>('export_png', {
    svgContent,
    defaultFilename,
  });
}

/**
 * Sanitize a string for use as a filename.
 * Removes or replaces characters that are invalid in filenames.
 */
function sanitizeFilename(name: string): string {
  return name
    .replace(/[<>:"/\\|?*]/g, '') // Remove invalid chars
    .replace(/\s+/g, '-')          // Replace spaces with dashes
    .replace(/-+/g, '-')           // Collapse multiple dashes
    .replace(/^-|-$/g, '')         // Remove leading/trailing dashes
    .substring(0, 100);            // Limit length
}
