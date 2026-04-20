use crate::apu::NesAPU;
use sdl2::AudioSubsystem;
use sdl2::audio::{AudioQueue, AudioSpecDesired};

const CPU_CLOCK_HZ: f64 = 1_789_773.0;
pub const SAMPLE_RATE: i32 = 44_100;
const CPU_CYCLES_PER_SAMPLE: f64 = CPU_CLOCK_HZ / SAMPLE_RATE as f64;
const BUFFER_SAMPLES: u16 = 1024;
const FLUSH_SAMPLES: usize = 512;
const MAX_QUEUED_SAMPLES: usize = 4096;

pub struct AudioPlayer {
    queue: AudioQueue<f32>,
    sample_cycles: f64,
    pending_samples: Vec<f32>,
}

impl AudioPlayer {
    pub fn new(audio_subsystem: &AudioSubsystem) -> Result<Self, String> {
        let desired_spec = AudioSpecDesired {
            freq: Some(SAMPLE_RATE),
            channels: Some(1),
            samples: Some(BUFFER_SAMPLES),
        };

        let queue = audio_subsystem.open_queue::<f32, _>(None, &desired_spec)?;
        queue.resume();

        Ok(Self {
            queue,
            sample_cycles: 0.0,
            pending_samples: Vec::with_capacity(FLUSH_SAMPLES),
        })
    }

    pub fn tick(&mut self, apu: &NesAPU) {
        self.sample_cycles += 1.0;
        while self.sample_cycles >= CPU_CYCLES_PER_SAMPLE {
            self.sample_cycles -= CPU_CYCLES_PER_SAMPLE;
            self.pending_samples.push(Self::mix_sample(apu));
        }

        if self.pending_samples.len() >= FLUSH_SAMPLES {
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        if self.pending_samples.is_empty() {
            return;
        }

        if self.queued_samples() >= MAX_QUEUED_SAMPLES {
            self.pending_samples.clear();
            return;
        }

        self.queue.queue(&self.pending_samples);
        self.pending_samples.clear();
    }

    fn queued_samples(&self) -> usize {
        self.queue.size() as usize / std::mem::size_of::<f32>()
    }

    fn mix_sample(apu: &NesAPU) -> f32 {
        let sample = apu.output() as f32 / u8::MAX as f32;
        sample * 2.0 - 1.0
    }
}
