use iced::{
    widget::{column, container, row, text, vertical_slider},
    Alignment::Center,
    Border, Element, Length,
};
use iced_aw::number_input;

/// A struct for storing a gui element representing a frequency relative to a global frequency
pub struct RelativeFrequency {
    pub absolute_frequency_id: usize,
    pub ratio: Ratio,
    pub volume: f32,
}

impl RelativeFrequency {
    pub fn view(&self, max_id: usize) -> Element<RelativeFrequencyMessage> {
        container(
            row![
                column![
                    container(row![
                        text("id"),
                        iced::widget::Space::new(Length::Fill, Length::Shrink),
                        number_input(
                            &self.absolute_frequency_id,
                            0..=max_id + 1,
                            RelativeFrequencyMessage::AbsoluteFrequencyIdUpdated,
                        )
                        .width(40),
                    ])
                    .width(75),
                    container(
                        row![
                            text("ratio"),
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
                vertical_slider(
                    -6.0..=0.0,
                    self.volume,
                    RelativeFrequencyMessage::VolumeUpdated
                )
                .step(0.1)
            ]
            .spacing(5),
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

    pub fn update(&mut self, message: RelativeFrequencyMessage) -> RelativeFrequencyStateUpdate {
        match message {
            RelativeFrequencyMessage::AbsoluteFrequencyIdUpdated(id) => {
                self.absolute_frequency_id = id;
                RelativeFrequencyStateUpdate::FrequencyUpdated
            }
            RelativeFrequencyMessage::RatioUpdated(message) => {
                self.ratio.update(message);
                RelativeFrequencyStateUpdate::FrequencyUpdated
            }
            RelativeFrequencyMessage::VolumeUpdated(new_volume) => {
                self.volume = new_volume;
                RelativeFrequencyStateUpdate::VolumeUpdated
            }
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
}

/// A struct for storing a mathematical ratio
#[derive(Debug, Clone, Copy)]
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
