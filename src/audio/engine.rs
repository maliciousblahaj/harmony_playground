use std::collections::BTreeMap;

use super::synthesizer::{WaveForm, WaveTable, WaveTableOscillator};

/// A struct representing an audio engine, providing an api for things like creating, updating and deleting oscillators
pub struct AudioEngine {
    sample_rate: usize,
    wavetable: WaveTable,
    oscillators: BTreeMap<usize, WaveTableOscillator>,
    time: f32,
    volume_multiple: f32,
    volume: f32,
    latestid: usize,
}

impl AudioEngine {
    pub fn new(sample_rate: usize) -> Self {
        Self {
            sample_rate,
            wavetable: WaveTable::sine(),
            oscillators: BTreeMap::new(),
            time: 0.0,
            volume_multiple: 0.25,
            volume: -2.0,
            latestid: 0,
        }
    }

    /// Get the current volume
    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    /// new_volume should be a base 2 gain
    pub fn set_volume(&mut self, new_volume: f32) {
        self.volume_multiple = new_volume.clamp(f32::NEG_INFINITY, 0.0).exp2();
        self.volume = new_volume;
    }

    /// Adds an oscillator to the engine, and returns the id
    pub fn add_oscillator(&mut self, frequency: f32) -> usize {
        let id = self.latestid;
        self.latestid += 1;

        self.oscillators.insert(
            id,
            WaveTableOscillator::new(self.sample_rate, frequency, self.wavetable.clone()),
        );
        id
    }

    /// Get a reference to all active oscillators
    pub fn get_oscillators(&self) -> &BTreeMap<usize, WaveTableOscillator> {
        &self.oscillators
    }

    /// Remove an oscillator from the engine by its id if it exists
    pub fn remove_oscillator(&mut self, id: &usize) {
        self.oscillators.remove(id);
    }

    /// Sets the oscillator with the provided id's frequency
    pub fn set_oscillator_frequency(&mut self, id: &usize, frequency: f32) {
        match self.oscillators.get_mut(id) {
            Some(osc) => osc.set_frequency(frequency),
            None => {}
        }
    }

    /// Set the waveform
    pub fn set_waveform(&mut self, waveform: WaveForm) {
        let new_table = match waveform {
            WaveForm::Sine => WaveTable::sine(),
            WaveForm::Triangle => WaveTable::triangle(),
            WaveForm::Square => WaveTable::square(),
            WaveForm::Saw => WaveTable::saw(),
        };
        for osc in self.oscillators.values_mut() {
            osc.set_wavetable(new_table.clone());
        }
        self.wavetable = new_table;
    }
}

impl Iterator for AudioEngine {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let mut sum = 0.0;
        for osc in self.oscillators.values_mut() {
            sum += osc.next().unwrap_or(0.0);
        }
        Some(sum * self.volume_multiple)
    }
}
