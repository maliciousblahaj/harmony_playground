use std::{
    iter::once,
    sync::{Arc, Mutex},
};

use harmony_playground::{
    audio::{
        engine::{AudioEngine, SharedFrequency, Volume},
        synthesizer::WaveForm,
    },
    gui::{
        self,
        global_frequency::{self, GlobalFrequency, GlobalFrequencyMessage},
        relative_frequency::{Ratio, RelativeFrequency, RelativeFrequencyMessage},
    },
};
use iced::{
    widget::{
        column, container, radio, row,
        scrollable::{Direction, Scrollbar},
        text, vertical_slider, Text,
    },
    Alignment::Center,
    Element, Task, Theme,
};
use rodio::{OutputStream, Source};

struct State {
    engine: Arc<Mutex<AudioEngine>>,
    // Will never be None
    waveform: Option<WaveForm>,
    volume: Volume,

    global_frequencies: Vec<GlobalFrequency>,
    /// Stores the relative frequency, its corresponding oscillator id for future possible deletion,
    /// and its corresponding shared frequency for simply updating the oscillator
    relative_frequencies: Vec<(RelativeFrequency, Option<usize>, SharedFrequency)>,
    latest_id: usize,
}

impl State {
    pub fn new(engine: Arc<Mutex<AudioEngine>>) -> Self {
        let volume = engine.lock().unwrap().get_volume();
        Self {
            engine,
            waveform: Some(WaveForm::Sine),
            volume,
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

    pub fn add_relative_frequency(
        &mut self,
        global_frequency_id: usize,
        relative_frequency: RelativeFrequency,
    ) -> Result<(), gui::error::Error> {
        // ids must match
        if relative_frequency.absolute_frequency_id != global_frequency_id {
            return Err(gui::error::Error::MismatchedFrequencyIds {
                global_frequency_id: global_frequency_id,
                relative_frequency_id: relative_frequency.absolute_frequency_id,
            });
        }
        // global frequency id must be valid
        let Some(global_frequency) = self.global_frequencies.get(global_frequency_id) else {
            return Err(gui::error::Error::InvalidGlobalFrequencyId(
                global_frequency_id,
            ));
        };
        let shared_frequency = SharedFrequency::new(
            global_frequency.frequency() * relative_frequency.ratio.multiplicand(),
        );
        let oscillator_id = self
            .engine
            .lock()
            .unwrap()
            // this initializes a shared channel for updating the frequency of the oscillator remotely
            //  so you don't have to lock the entire audio engine for that
            .add_oscillator(shared_frequency.clone());
        self.relative_frequencies
            .push((relative_frequency, Some(oscillator_id), shared_frequency));
        Ok(())
    }

    pub fn set_waveform(&mut self, waveform: WaveForm) {
        self.engine.lock().unwrap().set_waveform(waveform);
        self.waveform = Some(waveform);
    }

    pub fn set_volume(&mut self, volume: f32) {
        let volume = Volume::new(volume);
        self.engine.lock().unwrap().set_volume(volume);
        self.volume = volume;
    }

    pub fn update_global_frequency(&mut self, id: usize, message: GlobalFrequencyMessage) {
        let Some(global_frequency) = self.global_frequencies.get_mut(id) else {
            return;
        };
        global_frequency.update(message);
        // TODO for all relative frequencies, update them
    }

    pub fn update_relative_frequency(&mut self, id: usize, message: RelativeFrequencyMessage) {
        let Some((relative_frequency, oscillator_id_option, shared_frequency)) =
            self.relative_frequencies.get_mut(id)
        else {
            return;
        };
        // if relative frequency updated to a valid state, add an oscillator
        let Some(oscillator_id) = oscillator_id_option else {
            todo!()
        };
        // update the ui element representing the relative frequency
        relative_frequency.update(message);

        // update the audio engine to reflect these changes
        match self
            .global_frequencies
            .get(relative_frequency.absolute_frequency_id)
        {
            Some(global_frequency) => {
                shared_frequency
                    .set(global_frequency.frequency() * relative_frequency.ratio.multiplicand());
            }
            // if referencing an invalid global frequency, remove the oscillator so no sound is produced
            None => {
                self.engine
                    .lock()
                    .unwrap()
                    .remove_oscillator(&oscillator_id);
                *oscillator_id_option = None;
            }
        }
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
            state.update_global_frequency(id, message);
        }
        Message::RelativeFrequencyUpdated { id, message } => {
            state.update_relative_frequency(id, message);
        }
        Message::AddGlobalFrequency => {
            state.add_global_frequency(42.0);
        }
        Message::AddRelativeFrequency => {
            state
                .add_relative_frequency(
                    0,
                    RelativeFrequency {
                        absolute_frequency_id: 0,
                        ratio: Ratio::new(1, 1),
                    },
                )
                .unwrap();
        }
        Message::WaveFormUpdated(waveform) => state.set_waveform(waveform),
        Message::VolumeUpdated(volume) => state.set_volume(volume),
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
    //.style(|_, _| iced::widget::button::Style {
    //    background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.9, 0.2))),
    //    text_color: Color::WHITE,
    //    border: Border {
    //        color: Color::BLACK,
    //        width: 0.3,
    //        radius: Radius::new(2),
    //    },
    //    shadow: Shadow::default(),
    //})
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
                    .map(|(index, (relative_frequency, _, _))| relative_frequency
                        .view(state.global_frequencies.len())
                        .map(move |message| Message::RelativeFrequencyUpdated {
                            id: index,
                            message
                        }))
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
        .max_width(672),
        container(column![
            text("volume"),
            vertical_slider(-16.0..=0.0, state.volume.get(), Message::VolumeUpdated,)
        ])
        .height(150),
        container(column![
            radio(
                "Sine",
                WaveForm::Sine,
                state.waveform,
                Message::WaveFormUpdated
            ),
            radio(
                "Triangle",
                WaveForm::Triangle,
                state.waveform,
                Message::WaveFormUpdated
            ),
            radio(
                "Square",
                WaveForm::Square,
                state.waveform,
                Message::WaveFormUpdated
            ),
            radio(
                "Saw",
                WaveForm::Saw,
                state.waveform,
                Message::WaveFormUpdated
            ),
        ])
    ]
    .spacing(10)
    .into()
}

struct AudioSource(Arc<Mutex<AudioEngine>>);

impl Iterator for AudioSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.lock().unwrap().next()
    }
}

impl Source for AudioSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(256) // TODO: maybe set to None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        48000
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

fn main() -> iced::Result {
    let engine = Arc::new(Mutex::new(AudioEngine::new(48000)));

    let audio_source = AudioSource(engine.clone());

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    stream_handle.play_raw(audio_source.convert_samples());

    let mut state = State::new(engine);
    state.add_global_frequency(220.0);

    iced::application("Harmony Playground", update, view)
        .theme(|_| Theme::Dark)
        .run_with(move || (state, Task::none()))
}
