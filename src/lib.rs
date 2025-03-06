pub mod audio;
pub mod gui;

use std::{
    collections::HashSet,
    f32::consts::TAU,
    sync::{Arc, RwLock},
};

pub const WAVETABLE_SIZE: usize = 1024;

#[derive(Debug)]
pub struct Chord(HashSet<Frequency>);

#[derive(Debug, Clone, Copy)]
pub struct AbsoluteFrequency(pub f32);

#[derive(Debug)]
pub struct Frequency {
    base: AbsoluteFrequency,
    ratio: Option<Ratio>,
}

impl Frequency {
    pub fn absolute(frequency: AbsoluteFrequency) -> Self {
        Self {
            base: frequency,
            ratio: None,
        }
    }

    pub fn ratio(base: AbsoluteFrequency, ratio: Ratio) -> Self {
        Self {
            base,
            ratio: Some(ratio),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ratio {
    numerator: u32,
    denominator: u32,
}

impl Ratio {
    pub fn new(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    pub const fn multiplicand(&self) -> f32 {
        self.numerator as f32 / self.denominator as f32
    }
}

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
            self.0[if floor == WAVETABLE_SIZE {
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

pub struct WaveTableOscillator {
    sample_rate: usize,
    sample_rate_recip: f32,
    frequency: f32,
    wavetable: Arc<RwLock<WaveTable>>,
    time: f32,
}

impl WaveTableOscillator {
    pub fn new(sample_rate: usize, frequency: f32, wavetable: Arc<RwLock<WaveTable>>) -> Self {
        Self {
            sample_rate,
            sample_rate_recip: (sample_rate as f32).recip(),
            frequency,
            wavetable,
            time: 0f32,
        }
    }

    pub fn set_frequency(&mut self, new_frequency: f32) {
        self.frequency = new_frequency;
    }
}

impl Iterator for WaveTableOscillator {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self
            .wavetable
            .read()
            .unwrap()
            .get_interpolated_value(self.time * (WAVETABLE_SIZE as f32));
        self.time += self.frequency * self.sample_rate_recip;
        Some(sample)
    }
}
