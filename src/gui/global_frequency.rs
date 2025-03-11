use iced::{
    widget::{container, row, text},
    Border, Element,
};
use iced_aw::number_input;

#[derive(Clone)]
/// A struct for storing the gui element representing a global frequency
pub struct GlobalFrequency {
    /// The id of the global frequency, used for showing the user which indexed id the global frequency has
    id: usize,
    frequency: f32,
}

impl GlobalFrequency {
    pub fn new(id: usize, frequency: f32) -> Self {
        Self { id, frequency }
    }

    /// Get the id of the global frequency
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn frequency(&self) -> f32 {
        self.frequency
    }

    pub fn view(&self) -> Element<GlobalFrequencyMessage> {
        container(
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
            .spacing(10),
        )
        .padding(10)
        .width(150)
        .style(|theme: &iced::Theme| {
            iced::widget::container::Style::default().border(
                Border::default()
                    .width(1)
                    .rounded(2)
                    .color(theme.palette().background.inverse().scale_alpha(0.2)),
            )
        })
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

//#[derive(Clone)]
///// Global frequency to be shared internally on the gui thread
//pub struct GuiSharedGlobalFrequency(Rc<Cell<f32>>);
//
//impl GuiSharedGlobalFrequency {
//    pub fn new(frequency: f32) -> Self {
//        Self(Rc::new(Cell::new(frequency)))
//    }
//
//    pub fn get(&self) -> f32 {
//        self.0.get()
//    }
//
//    pub fn set(&self, frequency: f32) {
//        self.0.set(frequency)
//    }
//}
//
