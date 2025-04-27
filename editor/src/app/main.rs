use iced::Subscription;
use iced::keyboard::Key::Named as NamedBox;
use iced::event::{self, Event as MainEvent};
use iced::widget::{Space, vertical_rule};
use iced::{Alignment, Element, Length};
use iced::keyboard::Event as KeyboardEvent;
use iced::keyboard::key::Named;
use iced::widget::{Button, Column, Container, Row, Text};
use crate::app::triangle::content::TriangleMessage;
use crate::app::triangle::content::TriangleState;


use crate::app::design::style_separator;
use crate::app::design::{style_sidebar_button, style_sidebar_button_selected, Design};
use crate::data::resource::AppResource;

pub struct EditorApp {
    main_actions: Vec<MainAction>,
    pub(super) state: MainState,
    pub(super) app_resource: AppResource,
    pub(super) design: Design,
}

pub(super) struct MainState {
    selected_action: MainAction,
    pub(super) triangle: TriangleState,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MainAction {
    Intersect
}

impl MainAction {
    fn title(&self) -> &str {
        match self {
            MainAction::Intersect => "Triangle",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum MainMessage {
    ActionSelected(MainAction),
}

#[derive(Debug, Clone)]
pub(crate) enum AppMessage {
    Main(MainMessage),
    Triangle(TriangleMessage),
    EventOccurred(MainEvent),
}

impl EditorApp {

    pub fn new(mut app_resource: AppResource) -> Self {
        Self {
            main_actions: vec![MainAction::Intersect],
            state: MainState {
                selected_action: MainAction::Intersect,
                triangle: TriangleState::new(&mut app_resource.triangle),
            },
            app_resource,
            design: Design::new(),
        }
    }

    pub fn update(&mut self, message: AppMessage) {
        match message {
            AppMessage::Main(msg) => self.update_main(msg),
            AppMessage::Triangle(msg) => self.triangle_update(msg),
            AppMessage::EventOccurred(MainEvent::Keyboard(KeyboardEvent::KeyPressed { key: NamedBox(named @ (Named::ArrowDown | Named::ArrowUp)), .. })) => {
                match (named, self.state.selected_action.clone()) {
                    (Named::ArrowDown, MainAction::Intersect) => self.triangle_next_test(),
                    (Named::ArrowUp, MainAction::Intersect) => self.triangle_prev_test(),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn subscription(&self) -> Subscription<AppMessage> {
        event::listen().map(AppMessage::EventOccurred)
    }

    fn update_main(&mut self, message: MainMessage) {
        match message {
            MainMessage::ActionSelected(action) => {
                self.state.selected_action = action;
                match self.state.selected_action {
                    MainAction::Intersect => self.triangle_init(),
                }
            }
        }
    }

    pub fn view(&self) -> Element<AppMessage> {
        let content = Row::new()
            .push(Container::new(self.main_navigation())
                .width(Length::Fixed(160.0))
                .height(Length::Shrink)
                .align_x(Alignment::Start));

        let content = match self.state.selected_action {
            MainAction::Intersect => {
                content
                    .push(
                        vertical_rule(1).style(style_separator)
                    )
                    .push(self.triangle_content())
            }
        };

        content.height(Length::Fill).into()
    }

    fn main_navigation(&self) -> Column<AppMessage> {
        self.main_actions.iter().fold(
            Column::new().push(Space::new(Length::Fill, Length::Fixed(2.0))),
            |column, item| {
                let is_selected = self.state.selected_action.eq(item);
                column.push(
                    Container::new(
                        Button::new(Text::new(item.title()))
                            .width(Length::Fill)
                            .on_press(AppMessage::Main(MainMessage::ActionSelected(item.clone())))
                            .style(if is_selected { style_sidebar_button_selected } else { style_sidebar_button })
                    ).padding(self.design.action_padding())
                )
            },
        )
    }
}