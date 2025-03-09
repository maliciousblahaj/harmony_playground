use std::iter::once;

use harmony_playground::{
    audio::{engine::AudioEngine, synthesizer::WaveForm},
    gui::{
        global_frequency::{GlobalFrequency, GlobalFrequencyMessage},
        relative_frequency::{Ratio, RelativeFrequency, RelativeFrequencyMessage},
    },
};
use iced::{
    advanced::Widget,
    border::Radius,
    widget::{
        button::{self, Style},
        column, container, row,
        scrollable::{Direction, Scrollbar},
        text, vertical_slider, Text,
    },
    Alignment::Center,
    Border, Color, Element, Length, Shadow, Task,
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
            latest_id: 1,
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
    AddGlobalFrequency,
    AddRelativeFrequency,
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
        Message::AddGlobalFrequency => {
            state.add_global_frequency(42.0);
        }
        Message::AddRelativeFrequency => state.relative_frequencies.push(RelativeFrequency {
            absolute_frequency_id: 1,
            ratio: Ratio::new(1, 1),
        }),
        Message::WaveFormUpdated(waveform) => state.engine.set_waveform(waveform),
        Message::VolumeUpdated(volume) => state.engine.set_volume(volume),
    }
}

fn plus_button(width: f32, height: f32, on_press: Message) -> Element<'static, Message> {
    iced::widget::button(
        Text::new("+")
            .size(f32::min(width, height) / 1.5)
            .align_x(Center)
            .align_y(Center),
    )
    .width(width)
    .height(height)
    .style(|_, _| iced::widget::button::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.9, 0.2))),
        text_color: Color::WHITE,
        border: Border {
            color: Color::BLACK,
            width: 0.3,
            radius: Radius::new(2),
        },
        shadow: Shadow::default(),
    })
    .on_press(on_press)
    .into()
}

fn view(state: &State) -> Element<Message> {
    row![
        container(iced::widget::scrollable(column(
            state
                .global_frequencies
                .iter()
                .enumerate()
                .map(|(index, freq)| {
                    freq.view()
                        .map(move |message| Message::GlobalFrequencyUpdated { id: index, message })
                })
                .chain(once(plus_button(150.0, 35.0, Message::AddGlobalFrequency)))
        )))
        .max_width(150),
        container(
            iced::widget::scrollable(
                row(state
                    .relative_frequencies
                    .iter()
                    .enumerate()
                    .map(
                        |(index, freq)| freq.view(state.global_frequencies.len()).map(
                            move |message| Message::RelativeFrequencyUpdated { id: index, message }
                        )
                    )
                    .chain(once(plus_button(
                        35.0,
                        150.0,
                        Message::AddRelativeFrequency
                    ))))
                .spacing(1)
            )
            .direction(Direction::Horizontal(Scrollbar::new()))
        )
        .height(150)
        .max_width(765),
        container(column![
            text("volume"),
            vertical_slider(
                -16.0..=0.0,
                state.engine.get_volume(),
                Message::VolumeUpdated,
            )
        ])
        .height(150),
    ]
    .spacing(10)
    .into()
}

fn main() -> iced::Result {
    let mut state = State::new(AudioEngine::new(48000));
    state.add_global_frequency(220.0);

    iced::application("Harmony Playground", update, view).run_with(move || (state, Task::none()))
}
