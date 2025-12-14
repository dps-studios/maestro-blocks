// Debouncing utility for real-time worksheet updates
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout>;
  
  return function executedFunction(...args: Parameters<T>) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

// Throttling utility for high-frequency updates
export function throttle<T extends (...args: any[]) => any>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean;
  
  return function executedFunction(...args: Parameters<T>) {
    if (!inThrottle) {
      func(...args);
      inThrottle = true;
      setTimeout(() => inThrottle = false, limit);
    }
  };
}

// Utility to batch multiple updates
export class UpdateBatcher {
  private updates: Array<() => void> = [];
  private timeout: ReturnType<typeof setTimeout> | null = null;
  
  constructor(private delay: number = 100) {}
  
  add(update: () => void) {
    this.updates.push(update);
    
    if (this.timeout) {
      clearTimeout(this.timeout);
    }
    
    this.timeout = setTimeout(() => {
      this.flush();
    }, this.delay);
  }
  
  private flush() {
    // Execute all pending updates
    this.updates.forEach(update => update());
    this.updates = [];
    this.timeout = null;
  }
  
  cancel() {
    if (this.timeout) {
      clearTimeout(this.timeout);
      this.timeout = null;
    }
    this.updates = [];
  }
}

// Real-time update manager for worksheets
export class WorksheetUpdateManager {
  private batcher: UpdateBatcher;
  private pendingUpdates = new Set<string>();
  
  constructor(delay: number = 300) {
    this.batcher = new UpdateBatcher(delay);
  }
  
  scheduleUpdate(updateId: string, updateFn: () => void) {
    if (this.pendingUpdates.has(updateId)) {
      return; // Already scheduled
    }
    
    this.pendingUpdates.add(updateId);
    this.batcher.add(() => {
      try {
        updateFn();
      } finally {
        this.pendingUpdates.delete(updateId);
      }
    });
  }
  
  cancelUpdate(updateId: string) {
    this.pendingUpdates.delete(updateId);
  }
  
  cancelAll() {
    this.batcher.cancel();
    this.pendingUpdates.clear();
  }
}