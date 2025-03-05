use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use iced::{widget::text, Element};
use intonation_playground::{AbsoluteFrequency, Ratio, WaveTable, WaveTableOscillator};

struct AudioEngine {
    sample_rate: usize,
    wavetable: Arc<WaveTable>,
    oscillators: BTreeMap<usize, WaveTableOscillator>,
    time: f32,
    volume_multiple: f32,
    latestid: usize,
}

impl AudioEngine {
    fn new(sample_rate: usize) -> Self {
        Self {
            sample_rate,
            wavetable: Arc::new(WaveTable::sine()),
            oscillators: BTreeMap::new(),
            time: 0.0,
            volume_multiple: -2.0,
            latestid: 0,
        }
    }

    /// new_volume should be a base 2 gain
    fn set_volume(&mut self, new_volume: f32) {
        self.volume_multiple = new_volume.clamp(f32::NEG_INFINITY, 0.0).exp2();
    }

    /// Adds an oscillator to the engine, and returns the id
    fn add_oscillator(&mut self, frequency: f32) -> usize {
        let id = self.latestid;
        self.latestid += 1;

        self.oscillators.insert(
            id,
            WaveTableOscillator::new(self.sample_rate, frequency, self.wavetable.clone()),
        );
        id
    }

    /// Get a reference to all active oscillators
    fn get_oscillators(&self) -> &BTreeMap<usize, WaveTableOscillator> {
        &self.oscillators
    }

    /// Remove an oscillator from the engine by its id if it exists
    fn remove_oscillator(&mut self, id: &usize) {
        self.oscillators.remove(id);
    }

    /// Sets the oscillator with the provided id's frequency
    fn set_oscillator_frequency(&mut self, id: &usize, frequency: f32) {
        match self.oscillators.get_mut(id) {
            Some(osc) => osc.set_frequency(frequency),
            None => {}
        }
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

struct ActiveFrequency {
    oscillator_id: usize,
    base_frequency: Arc<Mutex<AbsoluteFrequency>>,
    ratio: Option<Ratio>,
}

struct State {
    engine: AudioEngine,
    absolute_frequencies: Vec<Arc<Mutex<AbsoluteFrequency>>>,
    active_frequencies: Vec<ActiveFrequency>,
}

#[derive(Debug)]
enum WaveForm {
    Sine,
    Triangle,
    Square,
    Saw,
}

#[derive(Debug)]
enum Message {
    FrequencyUpdated { id: usize, new_frequency: f32 },
    RelativeFrecuencyUpdated { id: usize, new_ratio: Ratio },
    WaveFormUpdated(WaveForm),
    VolumeUpdated(f32),
}

fn update(_state: &mut State, _message: Message) {}

fn view(_state: &State) -> Element<Message> {
    text("hello world").into()
}

fn main() -> iced::Result {
    //iced::run("Intonation Playground", update, view)
    todo!()
}
