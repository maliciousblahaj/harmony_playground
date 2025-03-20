use iced_aw::iced_fonts;
use postcard::to_allocvec;
use serde::{Deserialize, Serialize};

use std::{
    collections::BTreeMap,
    io,
    iter::once,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use harmony_playground::{
    audio::{
        engine::{AudioEngine, SharedFrequency, SharedVolumeMultiplier, Volume},
        synthesizer::WaveForm,
    },
    gui::{
        global_frequency::{GlobalFrequency, GlobalFrequencyMessage},
        icon_button,
        relative_frequency::{
            Ratio, RelativeFrequency, RelativeFrequencyMessage, RelativeFrequencyStateUpdate,
        },
    },
    icon,
};
use iced::{
    alignment::Horizontal,
    widget::{
        button, column, combo_box, container, horizontal_space, radio, row,
        scrollable::{Direction, Scrollbar},
        text, vertical_slider, vertical_space,
    },
    Element, Length, Task,
};
use rodio::{OutputStream, Source};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StateSave {
    volume: Volume,
    waveform: WaveForm,
    global_frequencies: BTreeMap<usize, GlobalFrequency>,
    relative_frequencies: Vec<RelativeFrequency>,
}

struct State {
    engine: Arc<Mutex<AudioEngine>>,
    // Will never be None
    waveform: Option<WaveForm>,
    volume: Volume,

    global_frequencies: BTreeMap<usize, GlobalFrequency>,
    /// Stores the relative frequency, its corresponding oscillator id for future possible deletion,
    /// and its corresponding shared frequency for simply updating the oscillator
    relative_frequencies: BTreeMap<
        usize,
        (
            RelativeFrequency,
            Option<usize>,
            SharedFrequency,
            SharedVolumeMultiplier,
        ),
    >,
    theme: iced::Theme,
    theme_selector_state: iced::widget::combo_box::State<iced::Theme>,
    is_loading: bool,
    /// If a file is open, this contains the path and a boolean indicating if the file is saved or not
    file: Option<(PathBuf, bool)>,
    current_error: Option<Error>,
}

impl State {
    pub fn new(engine: Arc<Mutex<AudioEngine>>) -> Self {
        let volume = engine.lock().unwrap().get_volume();
        let waveform = WaveForm::default();
        {
            let mut engine = engine.lock().unwrap();
            engine.clear_oscillators();
            engine.set_waveform(waveform);
        }
        Self {
            engine,
            waveform: Some(waveform),
            volume,
            global_frequencies: BTreeMap::new(),
            relative_frequencies: BTreeMap::new(),
            theme: iced::Theme::Dark,
            theme_selector_state: iced::widget::combo_box::State::new(Vec::from(iced::Theme::ALL)),
            is_loading: false,
            file: None,
            current_error: None,
        }
    }

    pub fn to_save(&self) -> StateSave {
        StateSave {
            volume: self.volume,
            waveform: self.waveform.unwrap_or(WaveForm::Sine),
            global_frequencies: self.global_frequencies.clone(),
            relative_frequencies: self
                .relative_frequencies
                .iter()
                .map(|(_, (relative_frequency, _, _, _))| relative_frequency.clone())
                .collect(),
        }
    }

    /// If an error occurs, run it through this function to display it to the user
    pub fn set_error(&mut self, error: Error) {
        match error {
            Error::FileDialogClosed => {}
            Error::IO(_) | Error::Postcard(_) => {
                self.current_error = Some(error);
            }
        };
    }

    /// Function that should be called after every change to display to the user that they need to save
    pub fn unsave(&mut self) {
        if let Some((_, ref mut is_saved)) = self.file {
            *is_saved = false;
        }
    }

    /// Returns the oscillator id and shared frequency if it succeeds to initialize, else None
    pub fn initialize_oscillator(
        engine: &Arc<Mutex<AudioEngine>>,
        global_frequencies: &BTreeMap<usize, GlobalFrequency>,
        relative_frequency: &RelativeFrequency,
    ) -> (Option<(usize, SharedFrequency)>, SharedVolumeMultiplier) {
        let shared_volume_multiplier =
            SharedVolumeMultiplier::new(Volume::new(relative_frequency.volume()).multiple());
        // if global frequency doesn't exist, don't create an oscillator
        let Some(global_frequency) =
            global_frequencies.get(&relative_frequency.absolute_frequency_id())
        else {
            return (None, shared_volume_multiplier);
        };
        let shared_frequency = SharedFrequency::new(
            global_frequency.frequency() * relative_frequency.ratio().multiplicand(),
        );
        let oscillator_id = engine
            .lock()
            .unwrap()
            // this initializes a shared channel for updating the frequency of the oscillator remotely
            //  so you don't have to lock the entire audio engine for that
            .add_oscillator(shared_frequency.clone(), shared_volume_multiplier.clone());
        (
            Some((oscillator_id, shared_frequency)),
            shared_volume_multiplier,
        )
    }

    pub fn from_save(
        engine: Arc<Mutex<AudioEngine>>,
        save: Arc<StateSave>,
        file_path: Option<PathBuf>,
        theme: iced::Theme,
    ) -> Self {
        let save = Arc::unwrap_or_clone(save);
        {
            let mut engine = engine.lock().unwrap();
            engine.clear_oscillators();
            engine.set_volume(save.volume);
            engine.set_waveform(save.waveform);
        }

        let relative_frequencies = save
            .relative_frequencies
            .into_iter()
            .enumerate()
            .map(|(index, relative_frequency)| {
                let (oscillator_id, shared_frequency, shared_volume_multiplier) =
                    match Self::initialize_oscillator(
                        &engine,
                        &save.global_frequencies,
                        &relative_frequency,
                    ) {
                        (Some((oscillator_id, shared_frequency)), shared_volume_multiplier) => (
                            Some(oscillator_id),
                            shared_frequency,
                            shared_volume_multiplier,
                        ),
                        (None, shared_volume_multiplier) => {
                            println!("failed to initialize oscillator");
                            (None, SharedFrequency::new(220.0), shared_volume_multiplier)
                        }
                    };

                (
                    index,
                    (
                        relative_frequency,
                        oscillator_id,
                        shared_frequency,
                        shared_volume_multiplier,
                    ),
                )
            })
            .collect();

        Self {
            engine,
            volume: save.volume,
            waveform: Some(save.waveform),
            global_frequencies: save.global_frequencies,
            relative_frequencies,
            theme,
            theme_selector_state: iced::widget::combo_box::State::new(Vec::from(iced::Theme::ALL)),
            is_loading: false,
            file: file_path.map(|path| (path, true)),
            current_error: None,
        }
    }

    pub fn add_global_frequency(&mut self, frequency: f32) {
        let latest_id = self
            .global_frequencies
            .last_key_value()
            .map(|(id, _)| id + 1)
            .unwrap_or(1);

        self.global_frequencies
            .insert(latest_id, GlobalFrequency::new(latest_id, frequency));
    }

    pub fn add_relative_frequency(&mut self, relative_frequency: RelativeFrequency) {
        let (oscillator_id, shared_frequency, shared_volume_multiplier) =
            match Self::initialize_oscillator(
                &self.engine,
                &self.global_frequencies,
                &relative_frequency,
            ) {
                (Some((oscillator_id, shared_frequency)), shared_volume_multiplier) => (
                    Some(oscillator_id),
                    shared_frequency,
                    shared_volume_multiplier,
                ),
                (None, shared_volume_multiplier) => {
                    (None, SharedFrequency::new(220.0), shared_volume_multiplier)
                }
            };

        let latest_id = self
            .relative_frequencies
            .last_key_value()
            .map(|(id, _)| id + 1)
            .unwrap_or(0);

        self.relative_frequencies.insert(
            latest_id,
            (
                relative_frequency,
                oscillator_id,
                shared_frequency,
                shared_volume_multiplier,
            ),
        );
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

        for (relative_frequency, _, shared_frequency, _) in self.relative_frequencies.values_mut() {
            if relative_frequency.absolute_frequency_id() != id {
                continue;
            }
            shared_frequency
                .set(global_frequency.frequency() * relative_frequency.ratio().multiplicand());
        }
    }

    pub fn update_relative_frequency(&mut self, id: usize, message: RelativeFrequencyMessage) {
        let Some((
            relative_frequency,
            oscillator_id_option,
            shared_frequency,
            shared_volume_multiplier,
        )) = self.relative_frequencies.get_mut(&id)
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
            Some(RelativeFrequencyStateUpdate::FrequencyUpdated) => {
                // update the frequency
                match self
                    .global_frequencies
                    .get(&relative_frequency.absolute_frequency_id())
                {
                    Some(global_frequency) => {
                        shared_frequency.set(
                            global_frequency.frequency()
                                * relative_frequency.ratio().multiplicand(),
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
            Some(RelativeFrequencyStateUpdate::VolumeUpdated) => {
                let volume_multiplier = Volume::new(relative_frequency.volume()).multiple();
                shared_volume_multiplier.set(volume_multiplier)
            }
            None => {}
        }
    }

    pub fn delete_relative_frequency(&mut self, id: usize) {
        let Some((_, oscillator_id_option, _, _)) = self.relative_frequencies.remove(&id) else {
            println!("deleted non-existent relative frequency");
            return;
        };
        if let Some(oscillator_id) = oscillator_id_option {
            self.engine
                .lock()
                .unwrap()
                .remove_oscillator(&oscillator_id);
        }
    }
}

/// The main error type
#[derive(Debug, Clone)]
enum Error {
    FileDialogClosed,
    #[allow(dead_code)]
    IO(io::ErrorKind), // TODO: handle these errors by displaying them, then remove the allow macro
    /// Errors related to serialization and deserialization of the postcard binary format
    #[allow(dead_code)]
    Postcard(postcard::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::FileDialogClosed => String::from("File dialog closed"),
                Error::IO(error_kind) => error_kind.to_string(),
                Error::Postcard(error) => error.to_string(),
            }
        )
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
    RelativeFrequencyDeleted(usize),
    AddGlobalFrequency,
    AddRelativeFrequency,
    WaveFormUpdated(WaveForm),
    VolumeUpdated(f32),
    PlayPressed,
    StopPressed,
    ThemeUpdated(iced::Theme),
    NewFile,
    SaveFile,
    OpenFile,
    SaveLoaded(Result<(PathBuf, Arc<StateSave>), Error>),
    StateSaved(Result<PathBuf, Error>),
}
impl State {
    fn title(&self) -> String {
        let (path, has_saved) = self
            .file
            .as_ref()
            .map(|(path, saved)| {
                (
                    path.as_os_str()
                        .to_str()
                        .unwrap_or("Unable to display file path"),
                    *saved,
                )
            })
            .unwrap_or(("New file", false));
        format!(
            "{}Harmony playground - {path}",
            if has_saved { "" } else { "*" }
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GlobalFrequencyUpdated { id, message } => {
                self.update_global_frequency(id, message);
                self.unsave();
                Task::none()
            }
            Message::RelativeFrequencyUpdated { id, message } => {
                self.update_relative_frequency(id, message);
                self.unsave();
                Task::none()
            }
            Message::RelativeFrequencyDeleted(id) => {
                self.delete_relative_frequency(id);
                self.unsave();
                Task::none()
            }
            Message::AddGlobalFrequency => {
                self.add_global_frequency(42.0);
                self.unsave();
                Task::none()
            }
            Message::AddRelativeFrequency => {
                self.add_relative_frequency(RelativeFrequency::new(0, Ratio::new(1, 1), -2.0));
                self.unsave();
                Task::none()
            }
            Message::WaveFormUpdated(waveform) => {
                self.set_waveform(waveform);
                self.unsave();
                Task::none()
            }
            Message::VolumeUpdated(volume) => {
                self.set_volume(volume);
                self.unsave();
                Task::none()
            }
            Message::ThemeUpdated(theme) => {
                self.theme = theme;
                Task::none()
            }

            Message::NewFile => {
                if !self.is_loading {
                    self.engine.lock().unwrap().reset();
                    *self = Self::new(self.engine.clone());
                }
                Task::none()
            }
            Message::SaveFile => {
                if self.is_loading {
                    Task::none()
                } else {
                    self.is_loading = true;

                    Task::perform(
                        save_file(None, Arc::new(self.to_save())),
                        Message::StateSaved,
                    )
                }
            }
            Message::OpenFile => {
                if self.is_loading {
                    Task::none()
                } else {
                    self.is_loading = true;

                    Task::perform(open_file(), Message::SaveLoaded)
                }
            }
            Message::SaveLoaded(result) => {
                self.is_loading = false;

                match result {
                    Ok((path, save)) => {
                        *self = Self::from_save(
                            self.engine.clone(),
                            save,
                            Some(path),
                            self.theme.clone(),
                        );
                    }
                    Err(error) => {
                        self.set_error(error);
                    }
                }

                Task::none()
            }
            Message::StateSaved(result) => {
                self.is_loading = false;

                match result {
                    Ok(path) => {
                        self.file = Some((path, true));
                    }
                    Err(error) => {
                        self.set_error(error);
                    }
                }

                Task::none()
            }
            Message::PlayPressed => {
                self.engine.lock().unwrap().play();
                Task::none()
            }
            Message::StopPressed => {
                self.engine.lock().unwrap().stop();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let global_frequencies = iced::widget::scrollable(
            column(
                self.global_frequencies
                    .iter()
                    .map(|(index, freq)| {
                        freq.view()
                            .map(move |message| Message::GlobalFrequencyUpdated {
                                id: index.to_owned(),
                                message,
                            })
                    })
                    .chain(once(
                        icon_button(icon::plus(), 14)
                            .on_press(Message::AddGlobalFrequency)
                            .width(200)
                            .into(),
                    )),
            )
            .spacing(1),
        );

        let relative_frequencies = iced::widget::scrollable(
            row(self
                .relative_frequencies
                .iter()
                .map(|(id, (relative_frequency, _, _, _))| {
                    relative_frequency
                        .view(self.global_frequencies.len())
                        .map(move |message| match message {
                            RelativeFrequencyMessage::Deleted => {
                                Message::RelativeFrequencyDeleted(*id)
                            }
                            message => Message::RelativeFrequencyUpdated { id: *id, message },
                        })
                })
                .chain(once(
                    icon_button(icon::plus(), 14)
                        .on_press(Message::AddRelativeFrequency)
                        .height(200)
                        .into(),
                )))
            .spacing(1),
        )
        .direction(Direction::Horizontal(Scrollbar::new()));

        let master_volume_slider = column![
            text("Master"),
            vertical_slider(-10.0..=0.0, self.volume.get(), Message::VolumeUpdated,).step(0.1)
        ]
        .align_x(Horizontal::Center)
        .height(Length::Fill);

        let waveform_selection = column![
            radio(
                "Sine",
                WaveForm::Sine,
                self.waveform,
                Message::WaveFormUpdated
            ),
            radio(
                "Triangle",
                WaveForm::Triangle,
                self.waveform,
                Message::WaveFormUpdated
            ),
            radio(
                "Square",
                WaveForm::Square,
                self.waveform,
                Message::WaveFormUpdated
            ),
            radio(
                "Saw",
                WaveForm::Saw,
                self.waveform,
                Message::WaveFormUpdated
            ),
        ]
        .spacing(12.5);

        let theme_selection = combo_box(
            &self.theme_selector_state,
            "Select theme",
            Some(&self.theme),
            Message::ThemeUpdated,
        );

        let audio_button = |icon, message| {
            icon_button(icon, 18)
                .on_press(message)
                .style(|theme: &iced::Theme, status| button::Style {
                    background: None,
                    text_color: {
                        let palette = theme.extended_palette();
                        match status {
                            button::Status::Active => palette.secondary.base.color,
                            button::Status::Disabled => palette.secondary.weak.color,
                            button::Status::Hovered | button::Status::Pressed => {
                                palette.secondary.strong.color
                            }
                        }
                    },
                    ..Default::default()
                })
        };

        let top_bar = row![
            // TODO: add a dialog asking if the user wants to save before creating a new one or opening
            button("New").on_press(Message::NewFile),
            button("Open").on_press(Message::OpenFile),
            button("Save").on_press(Message::SaveFile),
            horizontal_space().width(Length::Fill),
            audio_button(icon::play(), Message::PlayPressed),
            audio_button(icon::stop(), Message::StopPressed),
            horizontal_space().width(Length::Fill),
            container(theme_selection).width(150)
        ]
        .spacing(10);

        let bottom_bar = row![
            text(match self.current_error {
                Some(_) => "ERROR", //TODO: make this more informative
                None => "",
            })
            .style(|theme: &iced::Theme| {
                text::Style {
                    color: Some(theme.palette().danger),
                }
            })
            .size(15),
            horizontal_space().width(Length::Fill),
        ];

        container(
            column![
                top_bar,
                row![
                    container(
                        column![text("Frequencies"), global_frequencies]
                            .align_x(Horizontal::Center)
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
                .spacing(10),
                bottom_bar,
            ]
            .spacing(10),
        )
        .padding(10)
        .into()
    }
}
async fn open_file() -> Result<(PathBuf, Arc<StateSave>), Error> {
    let picked_file = rfd::AsyncFileDialog::new()
        .set_title("Open a file...")
        .add_filter("Harmony playground file", &["harm"])
        .pick_file()
        .await
        .ok_or(Error::FileDialogClosed)?;

    load_file(picked_file).await
}

async fn load_file(path: impl Into<PathBuf>) -> Result<(PathBuf, Arc<StateSave>), Error> {
    let path = path.into();

    let contents = tokio::fs::read(&path)
        .await
        .map(|bytevec| postcard::from_bytes(&bytevec))
        .map_err(|tokio_fs_error| Error::IO(tokio_fs_error.kind()))?
        .map(Arc::new)
        .map_err(Error::Postcard)?;

    Ok((path, contents))
}

async fn save_file(path: Option<PathBuf>, state_save: Arc<StateSave>) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .add_filter("Harmony playground file", &["harm"])
            .save_file()
            .await
            .as_ref()
            .map(rfd::FileHandle::path)
            .map(std::path::Path::to_owned)
            .ok_or(Error::FileDialogClosed)?
    };

    let contents = to_allocvec(&*state_save).map_err(Error::Postcard)?;

    tokio::fs::write(&path, contents)
        .await
        .map_err(|error| Error::IO(error.kind()))?;

    println!("saved to file {path:?}");

    Ok(path)
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

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let engine = Arc::new(Mutex::new(AudioEngine::new(48000)));

    let audio_source = AudioSource(engine.clone()).low_pass(2000);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let _ = stream_handle.play_raw(audio_source.convert_samples());

    let state = State::new(engine);

    iced::application(State::title, State::update, State::view)
        .theme(|state| state.theme.clone())
        .font(icon::FONT)
        .font(iced_fonts::REQUIRED_FONT_BYTES)
        .run_with(move || (state, Task::none()))
        .unwrap();
    Ok(())
}
