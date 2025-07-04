use iced::{
    Alignment::Center,
    Border, Color, Element, Length,
    alignment::Horizontal,
    widget::{button, column, container, row, text, vertical_slider, vertical_space},
};
use iced_aw::number_input;
use serde::{Deserialize, Serialize};

use crate::{audio::theory::Note, icon};

use super::icon_button;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// A struct for storing a gui element representing a frequency relative to a global frequency
pub struct RelativeFrequency {
    absolute_frequency_id: usize,
    ratio: Ratio,
    volume: f32,
}

impl RelativeFrequency {
    pub fn new(absolute_frequency_id: usize, ratio: Ratio, volume: f32) -> Self {
        Self {
            absolute_frequency_id,
            ratio,
            volume,
        }
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn absolute_frequency_id(&self) -> usize {
        self.absolute_frequency_id
    }

    pub fn ratio(&self) -> Ratio {
        self.ratio
    }

    pub fn view(&self, max_id: usize, played_frequency: f32) -> Element<RelativeFrequencyMessage> {
        let note = Note::from_frequency(played_frequency);

        let delete_button = icon_button(icon::cancel(), 12)
            .on_press(RelativeFrequencyMessage::Deleted)
            .style(|theme: &iced::Theme, status| button::Style {
                background: Some(iced::Background::Color({
                    let palette = theme.extended_palette();
                    match status {
                        button::Status::Active => palette.danger.base.color,
                        button::Status::Hovered | button::Status::Pressed => {
                            palette.danger.strong.color
                        }
                        button::Status::Disabled => palette.danger.weak.color,
                    }
                })),
                ..Default::default()
            });

        let right_column = column![
            delete_button.width(24),
            vertical_slider(
                -6.0..=0.0,
                self.volume,
                RelativeFrequencyMessage::VolumeUpdated
            )
            .step(0.1),
            vertical_space().height(5)
        ]
        .width(Length::Shrink)
        .spacing(10)
        .align_x(Center);

        container(
            column![
                row![
                    column![
                        container(row![
                            text("id").width(Length::Shrink),
                            iced::widget::Space::new(Length::Fill, Length::Shrink),
                            number_input(
                                &self.absolute_frequency_id,
                                0..=max_id,
                                RelativeFrequencyMessage::AbsoluteFrequencyIdUpdated,
                            )
                            .width(40),
                        ])
                        .width(75),
                        container(
                            row![
                                text("ratio").width(Length::Shrink),
                                iced::widget::Space::new(Length::Fill, Length::Shrink),
                                self.ratio
                                    .view()
                                    .map(RelativeFrequencyMessage::RatioUpdated)
                            ]
                            .align_y(Center)
                            .spacing(5)
                        )
                        .width(75),
                    ]
                    .align_x(Center)
                    .spacing(20),
                    right_column,
                ]
                .spacing(10),
                text(if self.absolute_frequency_id == 0 {
                    String::new()
                } else {
                    note.to_string()
                })
                .color(Color::from_rgb(0.5, 0.5, 0.5))
                .size(10),
            ]
            .spacing(2)
            .align_x(Horizontal::Center),
        )
        .padding(10)
        .height(180)
        .style(|theme: &iced::Theme| {
            iced::widget::container::Style::default().border(
                Border::default()
                    .width(1)
                    .rounded(2)
                    .color(theme.palette().background.inverse().scale_alpha(0.1)),
            )
        })
        .into()
    }

    pub fn update(
        &mut self,
        message: RelativeFrequencyMessage,
    ) -> Option<RelativeFrequencyStateUpdate> {
        match message {
            RelativeFrequencyMessage::AbsoluteFrequencyIdUpdated(id) => {
                self.absolute_frequency_id = id;
                Some(RelativeFrequencyStateUpdate::FrequencyUpdated)
            }
            RelativeFrequencyMessage::RatioUpdated(message) => {
                self.ratio.update(message);
                Some(RelativeFrequencyStateUpdate::FrequencyUpdated)
            }
            RelativeFrequencyMessage::VolumeUpdated(new_volume) => {
                self.volume = new_volume;
                Some(RelativeFrequencyStateUpdate::VolumeUpdated)
            }
            RelativeFrequencyMessage::Deleted => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RelativeFrequencyStateUpdate {
    FrequencyUpdated,
    VolumeUpdated,
}

#[derive(Debug, Clone)]
pub enum RelativeFrequencyMessage {
    AbsoluteFrequencyIdUpdated(usize),
    RatioUpdated(RatioMessage),
    VolumeUpdated(f32),
    Deleted,
}

/// A struct for storing a mathematical ratio
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Ratio {
    pub numerator: u32,
    pub denominator: u32,
}

impl Ratio {
    pub fn new(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    /// Calculate the multiplicand of the ratio
    pub const fn multiplicand(&self) -> f32 {
        self.numerator as f32 / self.denominator as f32
    }

    pub fn update(&mut self, message: RatioMessage) {
        match message {
            RatioMessage::NumeratorUpdated(n) => {
                self.numerator = n;
            }
            RatioMessage::DenominatorUpdated(n) => {
                self.denominator = n;
            }
        }
    }

    pub fn view(&self) -> Element<RatioMessage> {
        column![
            number_input(
                &self.numerator,
                1..=u32::MAX,
                RatioMessage::NumeratorUpdated,
            )
            .width(40),
            text("-----"),
            number_input(
                &self.denominator,
                1..=u32::MAX,
                RatioMessage::DenominatorUpdated,
            )
            .width(40),
        ]
        .into()
    }
}

#[derive(Debug, Clone)]
pub enum RatioMessage {
    NumeratorUpdated(u32),
    DenominatorUpdated(u32),
}
