use iced::{
    widget::{row, text},
    Element,
};
use iced_aw::number_input;

/// A struct for storing the gui element representing a global frequency
pub struct GlobalFrequency {
    /// The id of the global frequency, used for referencing from local frequencies
    id: usize,
    pub frequency: f32,
}

impl GlobalFrequency {
    pub fn new(id: usize, frequency: f32) -> Self {
        Self { id, frequency }
    }

    /// Get the id of the global frequency
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn view(&self) -> Element<GlobalFrequencyMessage> {
        row![
            text(format!("id: {}", self.id)),
            number_input(
                &self.frequency,
                1f32..=20000f32,
                GlobalFrequencyMessage::FrequencyUpdated
            )
            .width(100)
            .step(1.0),
        ]
        .spacing(10)
        .into()
    }

    pub fn update(&mut self, message: GlobalFrequencyMessage) {
        match message {
            GlobalFrequencyMessage::FrequencyUpdated(frequency) => {
                self.frequency = frequency;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum GlobalFrequencyMessage {
    FrequencyUpdated(f32),
}
