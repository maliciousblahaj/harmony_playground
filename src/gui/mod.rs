use iced::{
    widget::{Button, Text},
    Alignment,
};

pub mod global_frequency;
pub mod relative_frequency;
pub mod theme;

pub fn icon_button<Message>(icon: Text, size: impl Into<iced::Pixels>) -> Button<Message> {
    iced::widget::button(
        icon.align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .size(size),
    )
}
