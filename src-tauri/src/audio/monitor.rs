use rodio::Source;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Statistics collected by the audio monitor
#[derive(Debug, Clone)]
pub struct AudioStats {
    /// Peak absolute sample value (0.0 - infinity, >1.0 = clipping)
    pub peak: f32,
    /// RMS (root mean square) level - perceived loudness
    pub rms: f32,
    /// Crest factor (peak/RMS ratio in dB) - dynamic range indicator
    pub crest_factor_db: f32,
    /// Number of samples that exceeded 1.0 (hard clipped)
    pub clipped_samples: u64,
    /// Total samples processed
    pub total_samples: u64,
    /// Percentage of samples that clipped
    pub clip_percentage: f32,
}

impl AudioStats {
    /// Convert peak to dBFS (decibels relative to full scale)
    pub fn peak_dbfs(&self) -> f32 {
        if self.peak > 0.0 {
            20.0 * self.peak.log10()
        } else {
            f32::NEG_INFINITY
        }
    }

    /// Convert RMS to dBFS
    pub fn rms_dbfs(&self) -> f32 {
        if self.rms > 0.0 {
            20.0 * self.rms.log10()
        } else {
            f32::NEG_INFINITY
        }
    }

    /// Log a summary of the audio statistics
    pub fn log(&self, label: &str) {
        let clip_indicator = if self.clipped_samples > 0 {
            " [CLIPPING]"
        } else if self.peak > 0.95 {
            " [NEAR CLIP]"
        } else {
            ""
        };

        eprintln!(
            "[Audio:{}]{} peak={:.3} ({:.1}dB) rms={:.3} ({:.1}dB) crest={:.1}dB clips={}/{} ({:.2}%)",
            label,
            clip_indicator,
            self.peak,
            self.peak_dbfs(),
            self.rms,
            self.rms_dbfs(),
            self.crest_factor_db,
            self.clipped_samples,
            self.total_samples,
            self.clip_percentage
        );
    }
}

/// Shared state for collecting audio statistics across the source lifetime
struct MonitorState {
    peak: AtomicU64,           // Stored as f32 bits
    sum_squares: AtomicU64,    // Running sum for RMS (stored as f64 bits)
    clipped_samples: AtomicU64,
    total_samples: AtomicU64,
}

impl MonitorState {
    fn new() -> Self {
        Self {
            peak: AtomicU64::new(0),
            sum_squares: AtomicU64::new(0),
            clipped_samples: AtomicU64::new(0),
            total_samples: AtomicU64::new(0),
        }
    }

    fn update(&self, sample: f32) {
        let abs_sample = sample.abs();

        // Update peak (atomic max)
        let mut current = self.peak.load(Ordering::Relaxed);
        loop {
            let current_f32 = f32::from_bits(current as u32);
            if abs_sample <= current_f32 {
                break;
            }
            match self.peak.compare_exchange_weak(
                current,
                abs_sample.to_bits() as u64,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current = x,
            }
        }

        // Update sum of squares for RMS (atomic add)
        let square = (sample as f64) * (sample as f64);
        let mut current = self.sum_squares.load(Ordering::Relaxed);
        loop {
            let current_f64 = f64::from_bits(current);
            let new_val = current_f64 + square;
            match self.sum_squares.compare_exchange_weak(
                current,
                new_val.to_bits(),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current = x,
            }
        }

        // Count clipped samples
        if abs_sample > 1.0 {
            self.clipped_samples.fetch_add(1, Ordering::Relaxed);
        }

        self.total_samples.fetch_add(1, Ordering::Relaxed);
    }

    fn get_stats(&self) -> AudioStats {
        let peak = f32::from_bits(self.peak.load(Ordering::Relaxed) as u32);
        let sum_squares = f64::from_bits(self.sum_squares.load(Ordering::Relaxed));
        let total_samples = self.total_samples.load(Ordering::Relaxed);
        let clipped_samples = self.clipped_samples.load(Ordering::Relaxed);

        let rms = if total_samples > 0 {
            (sum_squares / total_samples as f64).sqrt() as f32
        } else {
            0.0
        };

        let crest_factor_db = if rms > 0.0 {
            20.0 * (peak / rms).log10()
        } else {
            0.0
        };

        let clip_percentage = if total_samples > 0 {
            (clipped_samples as f32 / total_samples as f32) * 100.0
        } else {
            0.0
        };

        AudioStats {
            peak,
            rms,
            crest_factor_db,
            clipped_samples,
            total_samples,
            clip_percentage,
        }
    }
}

/// Audio monitor that wraps a Source and collects statistics
pub struct AudioMonitor<I> {
    inner: I,
    state: Arc<MonitorState>,
    label: String,
    logged: bool,
}

impl<I> AudioMonitor<I>
where
    I: Source<Item = f32>,
{
    pub fn new(inner: I, label: impl Into<String>) -> Self {
        let label = label.into();
        eprintln!("[Audio] Monitor created: {}", label);
        Self {
            inner,
            state: Arc::new(MonitorState::new()),
            label,
            logged: false,
        }
    }

    /// Get current statistics (can be called while source is playing)
    #[allow(dead_code)]
    pub fn stats(&self) -> AudioStats {
        self.state.get_stats()
    }
}

impl<I> Iterator for AudioMonitor<I>
where
    I: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(sample) => {
                self.state.update(sample);
                Some(sample)
            }
            None => {
                // Source exhausted - log final stats once
                if !self.logged {
                    self.logged = true;
                    let stats = self.state.get_stats();
                    stats.log(&self.label);
                }
                None
            }
        }
    }
}

impl<I> Source for AudioMonitor<I>
where
    I: Source<Item = f32>,
{
    fn current_span_len(&self) -> Option<usize> {
        self.inner.current_span_len()
    }

    fn channels(&self) -> u16 {
        self.inner.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}

impl<I> Drop for AudioMonitor<I> {
    fn drop(&mut self) {
        // Log stats when monitor is dropped (even if source wasn't exhausted)
        if !self.logged {
            self.logged = true;
            let stats = self.state.get_stats();
            if stats.total_samples > 0 {
                stats.log(&self.label);
            }
        }
    }
}

/// Extension trait to add monitoring to any Source
pub trait AudioMonitorExt: Source<Item = f32> + Sized {
    /// Wrap this source with an audio monitor that logs statistics
    fn monitor(self, label: impl Into<String>) -> AudioMonitor<Self> {
        AudioMonitor::new(self, label)
    }
}

impl<S> AudioMonitorExt for S where S: Source<Item = f32> + Sized {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_stats_dbfs() {
        let stats = AudioStats {
            peak: 1.0,
            rms: 0.707,
            crest_factor_db: 3.0,
            clipped_samples: 0,
            total_samples: 1000,
            clip_percentage: 0.0,
        };

        assert!((stats.peak_dbfs() - 0.0).abs() < 0.01);
        assert!((stats.rms_dbfs() - (-3.0)).abs() < 0.1);
    }

    #[test]
    fn test_clipping_detection() {
        let stats = AudioStats {
            peak: 1.5,
            rms: 0.5,
            crest_factor_db: 9.5,
            clipped_samples: 100,
            total_samples: 10000,
            clip_percentage: 1.0,
        };

        assert!(stats.clipped_samples > 0);
        assert!(stats.peak > 1.0);
    }
}
