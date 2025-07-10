use iced::widget::{button, checkbox, container, scrollable, text};
use iced::{application, border, color, Color, Theme};

// --- Theme Definition ---

#[derive(Debug, Clone, Copy, Default)]
pub struct Fluent;

// --- Colors ---

const ACCENT_BLUE: Color = color!(0x00, 0x78, 0xD4);
const ACCENT_BLUE_HOVER: Color = color!(0x00, 0x5A, 0x9E);
const TEXT_PRIMARY: Color = color!(0x00, 0x00, 0x00);
const BORDER_LIGHT: Color = color!(0xE0, 0xE0, 0xE0);
const CONTROL_FILL_HOVER: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.05);
const ERROR_BACKGROUND: Color = color!(0xFDE7E9);
const ERROR_FOREGROUND: Color = color!(0xA4262C);

// --- Implementations ---

impl application::StyleSheet for Fluent {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: color!(0xFA, 0xFA, 0xFA),
            text_color: TEXT_PRIMARY,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Text {
    #[default]
    Default,
    Error,
}

impl text::StyleSheet for Fluent {
    type Style = Text;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        match style {
            Text::Default => text::Appearance { color: None }, // Inherit from application
            Text::Error => text::Appearance {
                color: Some(ERROR_FOREGROUND),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Button {
    #[default]
    Secondary,
    Primary,
}

impl button::StyleSheet for Fluent {
    type Style = Button;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        match style {
            Button::Primary => button::Appearance {
                background: Some(ACCENT_BLUE.into()),
                text_color: Color::WHITE,
                border: border::Border::with_radius(4.0),
                ..Default::default()
            },
            Button::Secondary => button::Appearance {
                background: Some(Color::WHITE.into()),
                text_color: TEXT_PRIMARY,
                border: border::Border {
                    color: BORDER_LIGHT,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        match style {
            Button::Primary => button::Appearance {
                background: Some(ACCENT_BLUE_HOVER.into()),
                ..active
            },
            Button::Secondary => button::Appearance {
                background: Some(CONTROL_FILL_HOVER.into()),
                ..active
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Container {
    #[default]
    Default,
    Error,
}

impl container::StyleSheet for Fluent {
    type Style = Container;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            Container::Default => container::Appearance {
                background: Some(Color::WHITE.into()),
                border: border::Border {
                    color: BORDER_LIGHT,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            },
            Container::Error => container::Appearance {
                background: Some(ERROR_BACKGROUND.into()),
                border: border::Border {
                    color: ERROR_FOREGROUND,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                text_color: Some(ERROR_FOREGROUND),
                ..Default::default()
            },
        }
    }
}

impl scrollable::StyleSheet for Fluent {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> scrollable::Appearance {
        scrollable::Appearance {
            container: container::Appearance::default(),
            scrollbar: scrollable::Scrollbar {
                background: None,
                border: border::Border::with_radius(2.0),
                scroller: scrollable::Scroller {
                    color: color!(0xAE, 0xAE, 0xAE),
                    border: border::Border::with_radius(2.0),
                },
            },
            gap: None,
        }
    }

    fn hovered(&self, style: &Self::Style, _is_mouse_over_scrollbar: bool) -> scrollable::Appearance {
        self.active(style)
    }
}

impl checkbox::StyleSheet for Fluent {
    type Style = Theme;

    fn active(&self, _style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        checkbox::Appearance {
            background: (if is_checked { ACCENT_BLUE } else { Color::TRANSPARENT }).into(),
            icon_color: Color::WHITE,
            border: border::Border {
                radius: 3.0.into(),
                width: 1.0,
                color: if is_checked { ACCENT_BLUE } else { color!(0x70, 0x70, 0x70) },
            },
            text_color: None,
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let mut active = self.active(style, is_checked);
        active.background = (if is_checked { ACCENT_BLUE_HOVER } else { CONTROL_FILL_HOVER }).into();
        active
    }
}