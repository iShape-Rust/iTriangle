use iced::{Color, Theme};

#[derive(Debug, Clone, Copy)]
pub(crate) struct PathEditorColorSchema {
    pub(crate) main: Color,
    pub(crate) drag: Color,
    pub(crate) hover: Color,
}

impl PathEditorColorSchema {

    pub(crate) fn with_theme(theme: Theme) -> Self {
        let palette = theme.extended_palette();

        if palette.is_dark {
            Self {
                main: Color::WHITE,
                drag: palette.primary.base.color,
                hover: palette.primary.weak.color
            }
        } else {
            Self {
                main: Color::BLACK,
                drag: palette.primary.base.color,
                hover: palette.primary.weak.color
            }
        }
    }
}