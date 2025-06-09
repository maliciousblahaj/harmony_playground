use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};

use super::synthesizer::{WaveForm, WaveTable, WaveTableOscillator};

/// A struct representing an audio engine, providing an api for things like creating, updating and deleting oscillators
pub struct AudioEngine {
    sample_rate: usize,
    wavetable: WaveTable,
    oscillators: BTreeMap<usize, WaveTableOscillator>,
    _time: f32, // eventually used in the future for syncing oscillators
    volume_multiple: f32,
    volume: Volume,
    latestid: usize,
    is_playing: bool,
    // // DEBUG: used for measuring if the sample_rate is correct
    // last_sample_time: std::time::Instant,
    // last_sample_index: usize, // mod 48000
    // last_sample_durations: [f32; 48000],
}

impl AudioEngine {
    pub fn new(sample_rate: usize) -> Self {
        let volume = Volume::new(-4.0);
        Self {
            sample_rate,
            wavetable: WaveTable::default(),
            oscillators: BTreeMap::new(),
            _time: 0.0,
            volume_multiple: volume.multiple(),
            volume,
            latestid: 0,
            is_playing: false,
            // last_sample_time: std::time::Instant::now(),
            // last_sample_index: 0,
            // last_sample_durations: [1.0; 48000],
        }
    }

    /// Reset the volume, wavetable and oscillators to their default values
    pub fn reset(&mut self) {
        let volume = Volume::new(-4.0);
        self.volume = volume;
        self.volume_multiple = volume.multiple();
        self.wavetable = WaveTable::default();
        self._time = 0.0;
    }

    /// Make the audio engine play
    pub fn play(&mut self) {
        self.is_playing = true;
    }

    /// Make the audio engine stop playing
    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    /// Get the current volume
    pub fn get_volume(&self) -> Volume {
        self.volume
    }

    /// new_volume should be a base 2 gain
    pub fn set_volume(&mut self, new_volume: Volume) {
        self.volume_multiple = new_volume.multiple();
        self.volume = new_volume;
        dbg!("Volume set: ", new_volume,);
    }

    /// Adds an oscillator to the engine, and returns the id
    pub fn add_oscillator(
        &mut self,
        frequency: SharedFrequency,
        volume_multiplier: SharedVolumeMultiplier,
    ) -> usize {
        let id = self.latestid;
        self.latestid += 1;

        self.oscillators.insert(
            id,
            WaveTableOscillator::new(
                self.sample_rate,
                frequency,
                volume_multiplier,
                self.wavetable.clone(),
            ),
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

    /// Remove all oscillators from the engine
    pub fn clear_oscillators(&mut self) {
        self.oscillators.clear();
    }

    ///// Sets the oscillator with the provided id's frequency
    //pub fn set_oscillator_frequency(&mut self, id: &usize, frequency: f32) {
    //    match self.oscillators.get_mut(id) {
    //        Some(osc) => osc.set_frequency(frequency),
    //        None => {}
    //    }
    //}

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
        // {
        //     // DEBUG
        //     let new_sample_time = std::time::Instant::now();
        //     let elapsed = new_sample_time
        //         .duration_since(self.last_sample_time)
        //         .as_secs_f32();
        //     self.last_sample_time = new_sample_time;
        //     self.last_sample_durations[self.last_sample_index] = elapsed;
        //     self.last_sample_index += 1;
        //     self.last_sample_index %= 48000;
        //     if self.last_sample_index == 47999 {
        //         let average = self.last_sample_durations.iter().sum::<f32>() / 48000.0; // Why does code SIGILL when this is changed to into_iter
        //         println!("Average sample rate: {}", average.recip());
        //     }
        // }

        if !self.is_playing {
            return Some(0.0);
        }
        let mut sum = 0.0;
        for osc in self.oscillators.values_mut() {
            sum += osc.next().unwrap_or(0.0);
        }
        Some(sum * self.volume_multiple)
    }
}

/// A struct representing a volume in base 2 gain. It is always clamped to be less than or equal to zero
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Volume(f32);

impl Volume {
    pub fn new(volume: f32) -> Self {
        let volume = volume.clamp(f32::NEG_INFINITY, 0.0);
        Self(volume)
    }

    pub fn get(&self) -> f32 {
        self.0
    }

    pub fn multiple(&self) -> f32 {
        self.0.exp2()
    }
}

/// A frequency shared between the gui thread and the audio thread to not have to put
/// locks on all data, only what's required
#[derive(Clone)]
pub struct SharedFrequency(Arc<RwLock<f32>>);

impl SharedFrequency {
    pub fn new(frequency: f32) -> Self {
        Self(Arc::new(RwLock::new(frequency)))
    }

    pub fn get(&self) -> f32 {
        *self.0.read().unwrap()
    }

    pub fn set(&self, frequency: f32) {
        *self.0.write().unwrap() = frequency;
    }
}

#[derive(Clone)]
pub struct SharedVolumeMultiplier(Arc<RwLock<f32>>);

impl SharedVolumeMultiplier {
    pub fn new(volume_multiplier: f32) -> Self {
        Self(Arc::new(RwLock::new(volume_multiplier)))
    }

    pub fn get(&self) -> f32 {
        *self.0.read().unwrap()
    }

    pub fn set(&self, volume_multiplier: f32) {
        let volume_multiplier = volume_multiplier.clamp(0.0, 1.0);
        *self.0.write().unwrap() = volume_multiplier;
    }
}
