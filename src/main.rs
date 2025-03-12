use std::{
    collections::BTreeMap,
    iter::once,
    sync::{Arc, Mutex},
};

use harmony_playground::{
    audio::{
        engine::{AudioEngine, SharedFrequency, SharedVolumeMultiplier, Volume},
        synthesizer::WaveForm,
    },
    gui::{
        self,
        global_frequency::{GlobalFrequency, GlobalFrequencyMessage},
        relative_frequency::{
            Ratio, RelativeFrequency, RelativeFrequencyMessage, RelativeFrequencyStateUpdate,
        },
    },
};
use iced::{
    alignment::Horizontal,
    widget::{
        button, column, combo_box, container, horizontal_space, radio, row,
        scrollable::{Direction, Scrollbar},
        text, vertical_slider, vertical_space, Text,
    },
    Alignment::Center,
    Element, Length, Task,
};
use rodio::{OutputStream, Source};

struct State {
    engine: Arc<Mutex<AudioEngine>>,
    // Will never be None
    waveform: Option<WaveForm>,
    volume: Volume,

    global_frequencies: BTreeMap<usize, GlobalFrequency>,
    /// Stores the relative frequency, its corresponding oscillator id for future possible deletion,
    /// and its corresponding shared frequency for simply updating the oscillator
    relative_frequencies: Vec<(
        RelativeFrequency,
        Option<usize>,
        SharedFrequency,
        SharedVolumeMultiplier,
    )>,
    latest_id: usize,
    theme: iced::Theme,
    theme_selector_state: iced::widget::combo_box::State<iced::Theme>,
}

impl State {
    pub fn new(engine: Arc<Mutex<AudioEngine>>) -> Self {
        let volume = engine.lock().unwrap().get_volume();
        Self {
            engine,
            waveform: Some(WaveForm::Sine),
            volume,
            global_frequencies: BTreeMap::new(),
            relative_frequencies: Vec::new(),
            latest_id: 1,
            theme: iced::Theme::Dark,
            theme_selector_state: iced::widget::combo_box::State::new(Vec::from(iced::Theme::ALL)),
        }
    }

    pub fn add_global_frequency(&mut self, frequency: f32) {
        self.global_frequencies.insert(
            self.latest_id,
            GlobalFrequency::new(self.latest_id, frequency),
        );
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
                global_frequency_id,
                relative_frequency_id: relative_frequency.absolute_frequency_id,
            });
        }
        // global frequency id must be valid
        let Some(global_frequency) = self.global_frequencies.get(&global_frequency_id) else {
            self.relative_frequencies.push((
                relative_frequency,
                None,
                SharedFrequency::new(220.0),
                SharedVolumeMultiplier::new(0.25),
            ));
            return Ok(());
        };
        let shared_frequency = SharedFrequency::new(
            global_frequency.frequency() * relative_frequency.ratio.multiplicand(),
        );
        let shared_volume_multiplier = SharedVolumeMultiplier::new(0.25);
        let oscillator_id = self
            .engine
            .lock()
            .unwrap()
            // this initializes a shared channel for updating the frequency of the oscillator remotely
            //  so you don't have to lock the entire audio engine for that
            .add_oscillator(shared_frequency.clone(), shared_volume_multiplier.clone());
        self.relative_frequencies.push((
            relative_frequency,
            Some(oscillator_id),
            shared_frequency,
            shared_volume_multiplier,
        ));
        Ok(())
    }

    pub fn set_waveform(&mut self, waveform: WaveForm) {
        self.engine.lock().unwrap().set_waveform(waveform);
        self.waveform = Some(waveform);
    }

    pub fn set_volume(&mut self, mut volume: f32) {
        if volume == -10.0 {
            volume = f32::NEG_INFINITY
        }
        let volume = Volume::new(volume);
        self.engine.lock().unwrap().set_volume(volume);
        self.volume = volume;
    }

    pub fn update_global_frequency(&mut self, id: usize, message: GlobalFrequencyMessage) {
        let Some(global_frequency) = self.global_frequencies.get_mut(&id) else {
            return;
        };
        global_frequency.update(message);

        for (relative_frequency, _, shared_frequency, _) in self.relative_frequencies.iter_mut() {
            if relative_frequency.absolute_frequency_id != id {
                continue;
            }
            shared_frequency
                .set(global_frequency.frequency() * relative_frequency.ratio.multiplicand());
        }
    }

    pub fn update_relative_frequency(&mut self, id: usize, message: RelativeFrequencyMessage) {
        let Some((
            relative_frequency,
            oscillator_id_option,
            shared_frequency,
            shared_volume_multiplier,
        )) = self.relative_frequencies.get_mut(id)
        else {
            return;
        };
        // if relative frequency updated to a valid state, add an oscillator
        let oscillator_id = match oscillator_id_option {
            Some(oscillator_id) => *oscillator_id,
            None => {
                let oscillator_id = self
                    .engine
                    .lock()
                    .unwrap()
                    .add_oscillator(shared_frequency.clone(), shared_volume_multiplier.clone());
                *oscillator_id_option = Some(oscillator_id);
                oscillator_id
            }
        };
        // update the ui element representing the relative frequency
        let state_update = relative_frequency.update(message);

        // update the audio engine to reflect these changes
        match state_update {
            RelativeFrequencyStateUpdate::FrequencyUpdated => {
                // update the frequency
                match self
                    .global_frequencies
                    .get(&relative_frequency.absolute_frequency_id)
                {
                    Some(global_frequency) => {
                        shared_frequency.set(
                            global_frequency.frequency() * relative_frequency.ratio.multiplicand(),
                        );
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
            RelativeFrequencyStateUpdate::VolumeUpdated => {
                let volume_multiplier = Volume::new(relative_frequency.volume).multiple();
                shared_volume_multiplier.set(volume_multiplier)
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
    ThemeUpdated(iced::Theme),
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
        Message::AddRelativeFrequency => state
            .add_relative_frequency(
                0,
                RelativeFrequency {
                    absolute_frequency_id: 0,
                    ratio: Ratio::new(1, 1),
                    volume: -2.0,
                },
            )
            .unwrap(),
        Message::WaveFormUpdated(waveform) => {
            state.set_waveform(waveform);
        }
        Message::VolumeUpdated(volume) => {
            state.set_volume(volume);
        }
        Message::ThemeUpdated(theme) => {
            state.theme = theme;
        }
    }
}

fn view(state: &State) -> Element<Message> {
    let global_frequencies = iced::widget::scrollable(
        column(
            state
                .global_frequencies
                .iter()
                .map(|(index, freq)| {
                    freq.view()
                        .map(move |message| Message::GlobalFrequencyUpdated {
                            id: index.to_owned(),
                            message,
                        })
                })
                .chain(once(
                    iced::widget::button(Text::new("+").align_x(Center).align_y(Center).size(20))
                        .on_press(Message::AddGlobalFrequency)
                        .width(200)
                        .into(),
                )),
        )
        .spacing(1),
    );

    let relative_frequencies = iced::widget::scrollable(
        row(state
            .relative_frequencies
            .iter()
            .enumerate()
            .map(|(index, (relative_frequency, _, _, _))| {
                relative_frequency
                    .view(state.global_frequencies.len() - 1)
                    .map(move |message| Message::RelativeFrequencyUpdated { id: index, message })
            })
            .chain(once(
                iced::widget::button(Text::new("+").align_x(Center).align_y(Center).size(20))
                    .on_press(Message::AddRelativeFrequency)
                    .height(200)
                    .into(),
            )))
        .spacing(1),
    )
    .direction(Direction::Horizontal(Scrollbar::new()));

    let master_volume_slider = column![
        text("master"),
        vertical_slider(-10.0..=0.0, state.volume.get(), Message::VolumeUpdated,).step(0.1)
    ]
    .align_x(Horizontal::Center)
    .height(Length::Fill);

    let waveform_selection = column![
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
    ]
    .spacing(10);

    let theme_selection = combo_box(
        &state.theme_selector_state,
        "Select theme",
        Some(&state.theme),
        Message::ThemeUpdated,
    );

    let top_bar = row![
        button("Load"), // TODO implement loading saves
        button("Save"), // TODO implement saving
        horizontal_space().width(Length::Fill),
        container(theme_selection).width(150)
    ]
    .spacing(10);
    container(
        column![
            top_bar,
            row![
                container(
                    column![text("Frequencies"), global_frequencies].align_x(Horizontal::Center)
                )
                .padding(5)
                .width(200)
                .style(|theme: &iced::Theme| {
                    iced::widget::container::Style::default().border(
                        iced::Border::default()
                            .width(1)
                            .rounded(2)
                            .color(theme.palette().background.inverse().scale_alpha(0.4)),
                    )
                }),
                container(
                    column![
                        text("Frequency Ratios"),
                        container(relative_frequencies).width(Length::Fill)
                    ]
                    .align_x(Horizontal::Center)
                )
                .padding(5)
                .height(200)
                .style(|theme: &iced::Theme| {
                    iced::widget::container::Style::default().border(
                        iced::Border::default()
                            .width(1)
                            .rounded(2)
                            .color(theme.palette().background.inverse().scale_alpha(0.4)),
                    )
                }),
                container(
                    column![
                        text("Waveform"),
                        vertical_space().height(Length::Fill),
                        waveform_selection
                    ]
                    .align_x(Horizontal::Center)
                )
                .max_height(200)
                .padding(10)
                .style(|theme: &iced::Theme| {
                    iced::widget::container::Style::default().border(
                        iced::Border::default()
                            .width(1)
                            .rounded(2)
                            .color(theme.palette().background.inverse().scale_alpha(0.4)),
                    )
                }),
                container(master_volume_slider)
                    .max_height(200)
                    .padding(10)
                    .style(|theme: &iced::Theme| {
                        iced::widget::container::Style::default().border(
                            iced::Border::default()
                                .width(1)
                                .rounded(2)
                                .color(theme.palette().background.inverse().scale_alpha(0.4)),
                        )
                    }),
            ]
            //.height(150)
            .spacing(10)
        ]
        .spacing(10),
    )
    .padding(10)
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
        None //TODO: maybe research this more
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

    let audio_source = AudioSource(engine.clone()).low_pass(2000);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let _ = stream_handle.play_raw(audio_source.convert_samples());

    let mut state = State::new(engine);
    state.add_global_frequency(220.0);

    iced::application("Harmony Playground", update, view)
        .theme(|state| state.theme.clone())
        .run_with(move || (state, Task::none()))
}
