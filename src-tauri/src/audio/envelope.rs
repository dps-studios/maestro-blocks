use rodio::Source;
use std::time::Duration;

/// Two-stage envelope that matches Web Audio API behavior
/// Stage 1: Linear ramp 0 → 0.01 over 3ms (prevents discontinuity)
/// Stage 2: Exponential ramp 0.01 → 1.0 over 12ms (natural attack)
pub struct TwoStageEnvelope<I> {
    inner: I,
    sample_rate: u32,
    current_sample: u64,
    /// Samples for stage 1 (linear 0 → 0.01)
    stage1_samples: u64,
    /// Samples for stage 2 (exponential 0.01 → 1.0)
    stage2_samples: u64,
}

impl<I> TwoStageEnvelope<I>
where
    I: Source<Item = f32>,
{
    /// Create a new two-stage envelope
    /// - stage1_duration: Time for linear ramp to 0.01 (typically 3ms)
    /// - stage2_duration: Time for exponential ramp to 1.0 (typically 12ms)
    pub fn new(inner: I, stage1_duration: Duration, stage2_duration: Duration) -> Self {
        let sample_rate = inner.sample_rate();
        let channels = inner.channels() as u64;

        // Calculate samples per stage (accounting for channels)
        let stage1_samples = (sample_rate as f64 * stage1_duration.as_secs_f64()) as u64 * channels;
        let stage2_samples = (sample_rate as f64 * stage2_duration.as_secs_f64()) as u64 * channels;

        Self {
            inner,
            sample_rate,
            current_sample: 0,
            stage1_samples,
            stage2_samples,
        }
    }
}

impl<I> Iterator for TwoStageEnvelope<I>
where
    I: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.inner.next()?;

        let gain = if self.current_sample < self.stage1_samples {
            // Stage 1: Linear 0 → 0.01
            let progress = self.current_sample as f32 / self.stage1_samples as f32;
            progress * 0.01
        } else if self.current_sample < self.stage1_samples + self.stage2_samples {
            // Stage 2: Exponential 0.01 → 1.0
            // Using: 0.01 * (1.0/0.01)^progress = 0.01 * 100^progress
            let progress = (self.current_sample - self.stage1_samples) as f32
                         / self.stage2_samples as f32;
            0.01 * 100.0_f32.powf(progress)
        } else {
            // Envelope complete - full volume
            1.0
        };

        self.current_sample += 1;
        Some(sample * gain)
    }
}

impl<I> Source for TwoStageEnvelope<I>
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
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}

/// Extension trait to add two_stage_envelope to any Source
pub trait TwoStageEnvelopeExt: Source<Item = f32> + Sized {
    /// Apply a two-stage envelope matching Web Audio API behavior
    fn two_stage_envelope(self) -> TwoStageEnvelope<Self> {
        TwoStageEnvelope::new(
            self,
            Duration::from_millis(3),   // Stage 1: 3ms linear to 0.01
            Duration::from_millis(12),  // Stage 2: 12ms exponential to 1.0
        )
    }
}

impl<S> TwoStageEnvelopeExt for S
where
    S: Source<Item = f32> + Sized,
{
}
