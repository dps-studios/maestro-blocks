const MEASURE_WIDTH = 150;
const MEASURE_HEIGHT = 100;
const SNAP_THRESHOLD = 20; // pixels

export function snapToGrid(x: number, y: number): { x: number; y: number } {
  const snappedX = Math.round(x / MEASURE_WIDTH) * MEASURE_WIDTH;
  const snappedY = Math.round(y / MEASURE_HEIGHT) * MEASURE_HEIGHT;
  return { x: snappedX, y: snappedY };
}

export function isAdjacent(
  block1: { x: number; y: number },
  block2: { x: number; y: number }
): boolean {
  // Check if measures are horizontally adjacent
  const horizontallyAdjacent = 
    Math.abs(block1.y - block2.y) < SNAP_THRESHOLD &&
    Math.abs(block1.x - block2.x - MEASURE_WIDTH) < SNAP_THRESHOLD;
  
  return horizontallyAdjacent;
}