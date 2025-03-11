use std::f32::consts::TAU;

use super::engine::SharedFrequency;

pub const WAVETABLE_SIZE: usize = 1024;

#[derive(Clone)]
pub struct WaveTable([f32; WAVETABLE_SIZE]);

impl WaveTable {
    pub fn sine() -> Self {
        Self::from_fn(|time| (time * TAU).sin())
    }

    pub fn triangle() -> Self {
        Self::from_fn(|time| 4.0 * (time + 0.25 - (time + 0.75).floor()).abs() - 1.0)
    }

    pub fn square() -> Self {
        Self::from_fn(|time| if time <= 0.5 { 1.0 } else { -1.0 })
    }

    pub fn saw() -> Self {
        Self::from_fn(|time| (2.0 * time + 3.0) % 2.0 - 1.0)
    }

    /// Create a wavetable from a periodic function of period 1
    pub fn from_fn(f: fn(f32) -> f32) -> Self {
        let mut table = [0f32; WAVETABLE_SIZE];
        for i in 0..WAVETABLE_SIZE {
            table[i] = f(i as f32 * ((WAVETABLE_SIZE as f32).recip()));
        }
        Self(table)
    }

    pub fn get_index(&self, index: usize) -> f32 {
        self.0[index]
    }

    pub fn get_interpolated_value(&self, index: f32) -> f32 {
        let index = index % (WAVETABLE_SIZE as f32);
        let floor = index as usize;
        let (sample0, sample1) = (
            self.0[floor],
            self.0[if floor == WAVETABLE_SIZE - 1 {
                0
            } else {
                floor + 1
            }],
        );
        lerp(sample0, sample1, index - (floor as f32))
    }
}

const fn lerp(sample0: f32, sample1: f32, t: f32) -> f32 {
    (1.0 - t) * sample0 + t * sample1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveForm {
    Sine,
    Triangle,
    Square,
    Saw,
}

pub struct WaveTableOscillator {
    sample_rate: usize,
    sample_rate_recip: f32,
    frequency: SharedFrequency,
    wavetable: WaveTable,
    time: f32,
}

impl WaveTableOscillator {
    pub fn new(sample_rate: usize, frequency: SharedFrequency, wavetable: WaveTable) -> Self {
        Self {
            sample_rate,
            sample_rate_recip: (sample_rate as f32).recip(),
            frequency,
            wavetable,
            time: 0f32,
        }
    }

    //pub fn set_frequency(&mut self, new_frequency: f32) {
    //    self.frequency = new_frequency;
    //}

    pub fn set_wavetable(&mut self, wavetable: WaveTable) {
        self.wavetable = wavetable;
    }
}

impl Iterator for WaveTableOscillator {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self
            .wavetable
            .get_interpolated_value(self.time * (WAVETABLE_SIZE as f32));
        self.time += self.frequency.get() * self.sample_rate_recip;
        Some(sample)
    }
}
