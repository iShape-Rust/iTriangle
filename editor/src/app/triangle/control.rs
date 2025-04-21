use crate::app::triangle::content::IntersectMessage;
use crate::app::main::{EditorApp, AppMessage};
use iced::{Alignment, Length};
use iced::widget::{Column, Container, pick_list, Row, Text};

impl EditorApp {
    pub(crate) fn triangle_control(&self) -> Column<AppMessage> {
        let mode_pick_list =
            Row::new()
                .push(Text::new("Mode:")
                    .width(Length::Fixed(90.0))
                    .height(Length::Fill)
                    .align_y(Alignment::Center))
                .push(
                    Container::new(
                        pick_list(
                            &ModeOption::ALL[..],
                            Some(self.state.triangle.mode),
                            on_select_mode,
                        ).width(Length::Fixed(160.0))
                    )
                        .height(Length::Fill)
                        .align_y(Alignment::Center)
                ).height(Length::Fixed(40.0));

        Column::new()
            .push(mode_pick_list)
    }
}

fn on_select_mode(option: ModeOption) -> AppMessage {
    AppMessage::Triangle(IntersectMessage::ModeSelected(option))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ModeOption {
    #[default]
    Raw,
    Delaunay,
}

impl ModeOption {
    const ALL: [ModeOption; 2] = [
        ModeOption::Raw,
        ModeOption::Delaunay,
    ];
}

impl std::fmt::Display for ModeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ModeOption::Raw => "Raw",
                ModeOption::Delaunay => "Delaunay",
            }
        )
    }
}