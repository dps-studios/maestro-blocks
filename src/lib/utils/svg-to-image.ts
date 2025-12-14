/**
 * SVG to Image Conversion Utilities
 * 
 * Converts SVG strings from LilyPond output into image URLs
 * that Konva's Image component can display.
 */

/**
 * Convert SVG string to data URL for Konva Image component
 * @param svgString - Raw SVG content from LilyPond
 * @returns Promise<string> - Data URL that Konva can use
 */
export async function svgStringToImageUrl(svgString: string): Promise<string> {
  return new Promise((resolve, reject) => {
    try {
      // Create blob from SVG string
      const blob = new Blob([svgString], { type: 'image/svg+xml;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      
      // Don't revoke the URL here - we need it for the Image component
      // The URL will be cleaned up later when the component is destroyed
      resolve(url);
    } catch (error) {
      reject(new Error('Failed to create blob from SVG'));
    }
  });
}

/**
 * Clean up object URLs to prevent memory leaks
 * @param url - Object URL to revoke
 */
export function cleanupImageUrl(url: string): void {
  if (url.startsWith('blob:')) {
    URL.revokeObjectURL(url);
  }
}

/**
 * Extract dimensions from SVG viewBox
 * @param svgString - SVG content string
 * @returns {width: number, height: number} - Extracted dimensions
 */
export function extractSvgDimensions(svgString: string): { width: number; height: number } {
  const viewBoxMatch = svgString.match(/viewBox="[^"]*"/);
  if (!viewBoxMatch) {
    return { width: 200, height: 80 }; // Default dimensions
  }
  
  const viewBox = viewBoxMatch[0];
  const numbers = viewBox.match(/[\d.]+/g);
  if (!numbers || numbers.length < 4) {
    return { width: 200, height: 80 };
  }
  
  const [, , width, height] = numbers.map(Number);
  return { width: width || 200, height: height || 80 };
}