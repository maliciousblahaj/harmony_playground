use iced::{
    widget::{column, row, slider, text, vertical_slider},
    Element, Task,
};
use intonation_playground::{
    audio::{engine::AudioEngine, synthesizer::WaveForm},
    gui::{
        global_frequency::{GlobalFrequency, GlobalFrequencyMessage},
        relative_frequency::{Ratio, RelativeFrequency, RelativeFrequencyMessage},
    },
};

struct State {
    engine: AudioEngine,
    global_frequencies: Vec<GlobalFrequency>,
    relative_frequencies: Vec<RelativeFrequency>,
    latest_id: usize,
}

impl State {
    pub fn new(engine: AudioEngine) -> Self {
        Self {
            engine,
            global_frequencies: Vec::new(),
            relative_frequencies: Vec::new(),
            latest_id: 0,
        }
    }

    pub fn add_global_frequency(&mut self, frequency: f32) {
        self.global_frequencies
            .push(GlobalFrequency::new(self.latest_id, frequency));
        self.latest_id += 1;
    }

    pub fn add_relative_frequency(&mut self, frequency: RelativeFrequency) {
        self.relative_frequencies.push(frequency);
    }
}

#[derive(Debug, Clone)]
enum Message {
    GlobalFrequencyUpdated {
        id: usize,
        message: GlobalFrequencyMessage,
    },
    RelativeFrequencyUpdated {
        id: usize,
        message: RelativeFrequencyMessage,
    },
    WaveFormUpdated(WaveForm),
    VolumeUpdated(f32),
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::GlobalFrequencyUpdated { id, message } => {
            if let Some(frequency) = state.global_frequencies.get_mut(id) {
                frequency.update(message);
            }
        }
        Message::RelativeFrequencyUpdated { id, message } => {
            if let Some(frequency) = state.relative_frequencies.get_mut(id) {
                frequency.update(message);
            }
        }
        Message::WaveFormUpdated(waveform) => state.engine.set_waveform(waveform),
        Message::VolumeUpdated(volume) => state.engine.set_volume(volume),
    }
}

fn view(state: &State) -> Element<Message> {
    row![
        column(
            state
                .global_frequencies
                .iter()
                .enumerate()
                .map(|(index, freq)| {
                    freq.view()
                        .map(move |message| Message::GlobalFrequencyUpdated { id: index, message })
                })
        ),
        row(state
            .relative_frequencies
            .iter()
            .enumerate()
            .map(|(index, freq)| freq
                .view()
                .map(move |message| Message::RelativeFrequencyUpdated { id: index, message })))
        .spacing(1),
        column![
            text("volume"),
            vertical_slider(
                -16.0..=0.0,
                state.engine.get_volume(),
                Message::VolumeUpdated,
            )
            .height(100),
        ],
    ]
    .spacing(50)
    .into()
}

fn main() -> iced::Result {
    let mut state = State::new(AudioEngine::new(48000));
    state.add_global_frequency(220.0);
    state.add_global_frequency(253.4622);
    state.add_relative_frequency(RelativeFrequency {
        absolute_frequency_id: 0,
        ratio: Ratio {
            numerator: 5,
            denominator: 3,
        },
    });
    state.add_relative_frequency(RelativeFrequency {
        absolute_frequency_id: 0,
        ratio: Ratio {
            numerator: 1,
            denominator: 1,
        },
    });
    state.add_relative_frequency(RelativeFrequency {
        absolute_frequency_id: 1,
        ratio: Ratio {
            numerator: 3,
            denominator: 2,
        },
    });

    iced::application("Intonation Playground", update, view).run_with(move || (state, Task::none()))
}
