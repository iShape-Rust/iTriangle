use iced::{Color, Theme};

#[derive(Debug, Clone, Copy)]
pub(crate) struct MeshViewerColorSchema {
    pub(crate) main: Color,
}

impl MeshViewerColorSchema {

    pub(crate) fn with_theme(theme: Theme) -> Self {
        let palette = theme.extended_palette();

        if palette.is_dark {
            Self {
                main: Color::from_rgb8(255, 100, 100),
            }
        } else {
            Self {
                main: Color::from_rgb8(255, 100, 100),
            }
        }
    }
}