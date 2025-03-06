use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use iced::{
    widget::{column, container, keyed_column, row, text},
    Element,
};
use iced_aw::number_input;
use intonation_playground::{Ratio, WaveTable, WaveTableOscillator};

struct AudioEngine {
    sample_rate: usize,
    wavetable: Arc<RwLock<WaveTable>>,
    oscillators: BTreeMap<usize, WaveTableOscillator>,
    time: f32,
    volume_multiple: f32,
    latestid: usize,
}

impl AudioEngine {
    fn new(sample_rate: usize) -> Self {
        Self {
            sample_rate,
            wavetable: Arc::new(RwLock::new(WaveTable::sine())),
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

    /// Set the waveform
    fn set_waveform(&mut self, waveform: WaveForm) {
        let new_table = match waveform {
            WaveForm::Sine => WaveTable::sine(),
            WaveForm::Triangle => WaveTable::triangle(),
            WaveForm::Square => WaveTable::square(),
            WaveForm::Saw => WaveTable::saw(),
        };
        let mut guard = self.wavetable.write().unwrap();
        *guard = new_table;
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
    base_frequency_id: usize,
    ratio: Ratio,
}

struct AbsoluteFrequency {
    id: usize,
    frequency: f32,
}

impl AbsoluteFrequency {
    fn view<'a>(&'a self, id: usize) -> Element<'a, Message> {
        row![
            text(format!("id: {}", self.id)),
            number_input(&self.frequency, 1f32..=20000f32, move |n| {
                Message::FrequencyUpdated {
                    id: id,
                    new_frequency: n,
                }
            })
            .step(1.0),
        ]
        .into()
    }
}

struct State {
    engine: AudioEngine,
    absolute_frequencies: Vec<Arc<RwLock<AbsoluteFrequency>>>,
    active_frequencies: Vec<ActiveFrequency>,
}

#[derive(Debug, Clone)]
enum WaveForm {
    Sine,
    Triangle,
    Square,
    Saw,
}

#[derive(Debug, Clone)]
enum Message {
    FrequencyUpdated {
        id: usize,
        new_frequency: f32,
    },
    RelativeFrecuencyUpdated {
        id: usize,
        new_absolute_id: usize,
        new_ratio: Ratio,
    },
    WaveFormUpdated(WaveForm),
    VolumeUpdated(f32),
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::FrequencyUpdated { id, new_frequency } => {
            if let Some(a) = state.absolute_frequencies.get(id) {
                a.write().unwrap().0 = new_frequency;
            }
        }
        Message::RelativeFrecuencyUpdated { id, new_ratio } => {
            let active_frequency = &mut state.active_frequencies[id];
            active_frequency.ratio = new_ratio;
            match state
                .engine
                .oscillators
                .get_mut(&active_frequency.oscillator_id)
            {
                Some(osc) => {
                    osc.set_frequency(
                        active_frequency.base_frequency.read().unwrap().0
                            * new_ratio.multiplicand(),
                    );
                }
                None => {
                    dbg!("Shouldn't happen, but if it does, the fix is to add an oscillator at this line of code");
                }
            }
        }
        Message::WaveFormUpdated(waveform) => state.engine.set_waveform(waveform),
        Message::VolumeUpdated(volume) => state.engine.set_volume(volume),
    }
}

fn view(state: &State) -> Element<Message> {
    let absolute_frequencies = state
        .absolute_frequencies
        .iter()
        .enumerate()
        .map(|(index, a)| {
            let guard = a.read().unwrap();
            guard.view(index)
        })
        .collect::<Vec<_>>();

    container(column(absolute_frequencies)).into()
}

fn main() -> iced::Result {
    //iced::run("Intonation Playground", update, view)
    todo!()
}
