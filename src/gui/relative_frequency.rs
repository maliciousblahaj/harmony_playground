use iced::{
    widget::{column, row, text},
    Alignment::Center,
    Element,
};
use iced_aw::number_input;

/// A struct for storing a gui element representing a frequency relative to a global frequency
pub struct RelativeFrequency {
    pub absolute_frequency_id: usize,
    pub ratio: Ratio,
}

impl RelativeFrequency {
    pub fn view(&self) -> Element<RelativeFrequencyMessage> {
        column![
            row![
                text("id:"),
                number_input(
                    &self.absolute_frequency_id,
                    0..usize::MAX,
                    RelativeFrequencyMessage::AbsoluteFrequencyIdUpdated,
                )
                .width(40),
            ],
            row![
                text("ratio"),
                self.ratio
                    .view()
                    .map(|message| RelativeFrequencyMessage::RatioUpdated(message))
            ]
            .align_y(Center)
            .spacing(5),
        ]
        .spacing(20)
        .into()
    }

    pub fn update(&mut self, message: RelativeFrequencyMessage) {
        match message {
            RelativeFrequencyMessage::AbsoluteFrequencyIdUpdated(id) => {
                self.absolute_frequency_id = id;
            }
            RelativeFrequencyMessage::RatioUpdated(message) => {
                self.ratio.update(message);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum RelativeFrequencyMessage {
    AbsoluteFrequencyIdUpdated(usize),
    RatioUpdated(RatioMessage),
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
