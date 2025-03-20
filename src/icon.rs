// Generated automatically by iced_fontello at build time.
// Do not edit manually. Source: ../fonts/icons.toml
// fd8e5ba39c2ed7f64acfe886b56540f536e52e611182003b708be98a98f1416b
use iced::widget::{text, Text};
use iced::Font;

pub const FONT: &[u8] = include_bytes!("../fonts/icons.ttf");

pub fn cancel<'a>() -> Text<'a> {
    icon("\u{2715}")
}

pub fn play<'a>() -> Text<'a> {
    icon("\u{25B6}")
}

pub fn plus<'a>() -> Text<'a> {
    icon("\u{2B}")
}

pub fn stop<'a>() -> Text<'a> {
    icon("\u{25AA}")
}

fn icon(codepoint: &str) -> Text<'_> {
    text(codepoint).font(Font::with_name("icons"))
}
