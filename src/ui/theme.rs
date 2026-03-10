use iced::{Color, Theme, theme};

pub fn app_theme(_state: &crate::app::Overlay) -> Theme {
    Theme::TokyoNightStorm
}

pub fn app_style(_state: &crate::app::Overlay, _theme: &Theme) -> theme::Style {
    theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: Color::WHITE,
    }
}
