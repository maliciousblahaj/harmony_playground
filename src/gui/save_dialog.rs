use iced::{
    Border, Color, Element, Length,
    alignment::Horizontal,
    border::Radius,
    widget::{button, center, column, container, opaque, row, stack, text},
};

pub fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    stack![
        base.into(),
        opaque(center(opaque(content)).style(|_| {
            container::Style {
                background: Some(
                    Color {
                        a: 0.8,
                        ..Color::BLACK
                    }
                    .into(),
                ),
                ..container::Style::default()
            }
        }))
    ]
    .into()
}

pub struct SaveDialog;

#[derive(Clone, Debug, Copy)]
pub enum SaveDialogMessage {
    CancelPressed,
    DontSavePressed,
    SavePressed,
}
impl SaveDialog {
    pub fn view<'a>() -> Element<'a, SaveDialogMessage> {
        container(
            column![
                text("Do you want to save the project before closing?"),
                row![
                    button("Cancel")
                        .width(Length::Fill)
                        .on_press(SaveDialogMessage::CancelPressed),
                    button("Don't Save")
                        .width(Length::Fill)
                        .on_press(SaveDialogMessage::DontSavePressed),
                    button("Save")
                        .width(Length::Fill)
                        .on_press(SaveDialogMessage::SavePressed),
                ]
                .spacing(10)
            ]
            .spacing(10)
            .align_x(Horizontal::Center),
        )
        .max_width(350)
        .max_height(150)
        .style(|theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme.palette().background)),

            border: Border {
                radius: Radius::new(5),
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(10)
        .into()
    }
}
