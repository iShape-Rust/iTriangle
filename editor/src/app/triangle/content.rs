use i_mesh::i_triangle::i_overlay::i_shape::int::reverse::IntContourReverse;
use crate::app::design;
use crate::app::main::{AppMessage, EditorApp};
use crate::app::triangle::control::ModeOption;
use crate::app::triangle::workspace::WorkspaceState;
use crate::data::triangle::TriangleResource;
use crate::geom::camera::Camera;
use crate::path_editor::widget::PathEditorUpdateEvent;
use i_mesh::i_triangle::i_overlay::core::fill_rule::FillRule;
use i_mesh::i_triangle::i_overlay::core::simplify::Simplify;
use i_mesh::i_triangle::i_overlay::i_float::int::rect::IntRect;
use i_mesh::i_triangle::i_overlay::i_shape::int::path::IntPath;
use iced::widget::scrollable;
use iced::widget::{Button, Column, Container, Row, Space, Text};
use iced::{Alignment, Length, Padding, Size, Vector};
use std::collections::HashMap;

pub(crate) struct TriangleState {
    pub(crate) test: usize,
    pub(crate) mode: ModeOption,
    pub(crate) workspace: WorkspaceState,
    pub(crate) size: Size,
    pub(crate) cameras: HashMap<usize, Camera>,
}

#[derive(Debug, Clone)]
pub(crate) enum IntersectMessage {
    TestSelected(usize),
    ModeSelected(ModeOption),
    PathEdited(PathEditorUpdateEvent),
    WorkspaceSized(Size),
    WorkspaceZoomed(Camera),
    WorkspaceDragged(Vector<f32>),
}

impl EditorApp {
    fn triangle_sidebar(&self) -> Column<AppMessage> {
        let count = self.app_resource.triangle.count;
        let mut column = Column::new().push(Space::new(Length::Fill, Length::Fixed(2.0)));
        for index in 0..count {
            let is_selected = self.state.triangle.test == index;

            column = column.push(
                Container::new(
                    Button::new(
                        Text::new(format!("test_{}", index))
                            .style(if is_selected {
                                design::style_sidebar_text_selected
                            } else {
                                design::style_sidebar_text
                            })
                            .size(14),
                    )
                    .width(Length::Fill)
                    .on_press(AppMessage::Triangle(IntersectMessage::TestSelected(index)))
                    .style(if is_selected {
                        design::style_sidebar_button_selected
                    } else {
                        design::style_sidebar_button
                    }),
                )
                .padding(self.design.action_padding()),
            );
        }

        column
    }

    pub(crate) fn triangle_content(&self) -> Row<AppMessage> {
        Row::new()
            .push(
                scrollable(
                    Container::new(self.triangle_sidebar())
                        .width(Length::Fixed(160.0))
                        .height(Length::Shrink)
                        .align_x(Alignment::Start)
                        .padding(Padding::new(0.0).right(8))
                        .style(design::style_sidebar_background),
                )
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::new()
                        .width(4)
                        .margin(0)
                        .scroller_width(4)
                        .anchor(scrollable::Anchor::Start),
                )),
            )
            .push(self.triangle_workspace())
    }

    pub(crate) fn triangle_update(&mut self, message: IntersectMessage) {
        match message {
            IntersectMessage::TestSelected(index) => self.triangle_set_test(index),
            IntersectMessage::ModeSelected(mode) => self.triangle_update_mode(mode),
            IntersectMessage::PathEdited(update) => self.triangle_update_anchor(update),
            IntersectMessage::WorkspaceSized(size) => self.triangle_update_size(size),
            IntersectMessage::WorkspaceZoomed(zoom) => self.triangle_update_zoom(zoom),
            IntersectMessage::WorkspaceDragged(drag) => self.triangle_update_drag(drag),
        }
    }

    fn triangle_set_test(&mut self, index: usize) {
        self.state
            .triangle
            .load_test(index, &mut self.app_resource.triangle);
        self.state.triangle.update_solution();
    }

    pub(crate) fn triangle_init(&mut self) {
        self.triangle_set_test(self.state.triangle.test);
    }

    pub(crate) fn triangle_next_test(&mut self) {
        let next_test = self.state.triangle.test + 1;
        if next_test < self.app_resource.triangle.count {
            self.triangle_set_test(next_test);
        }
    }

    pub(crate) fn triangle_prev_test(&mut self) {
        let test = self.state.triangle.test;
        if test >= 1 {
            self.triangle_set_test(test - 1);
        }
    }

    fn triangle_update_size(&mut self, size: Size) {
        self.state.triangle.size = size;
        let curves = &self.state.triangle.workspace.paths;
        if self.state.triangle.workspace.camera.is_empty() && !curves.is_empty() {
            let camera = Camera::with_size_and_curves(size, curves);
            self.state.triangle.workspace.camera = camera;
        } else {
            self.state.triangle.workspace.camera.size = size;
        }
    }

    fn triangle_update_mode(&mut self, mode: ModeOption) {
        self.state.triangle.mode = mode;
        self.state.triangle.update_solution();
    }
}

impl TriangleState {
    pub(crate) fn new(resource: &mut TriangleResource) -> Self {
        let mut state = TriangleState {
            test: usize::MAX,
            mode: ModeOption::Edit,
            workspace: Default::default(),
            cameras: HashMap::with_capacity(resource.count),
            size: Size::ZERO,
        };

        state.load_test(0, resource);
        state.update_solution();
        state
    }

    fn load_test(&mut self, index: usize, resource: &mut TriangleResource) {
        if let Some(test) = resource.load(index) {
            self.workspace.paths.clear();

            self.workspace.paths = test.paths;

            self.cameras.insert(self.test, self.workspace.camera);
            let mut camera = *self.cameras.get(&index).unwrap_or(&Camera::empty());
            if camera.is_empty() && self.size.width > 0.001 {
                camera = Camera::with_size_and_curves(self.size, &self.workspace.paths);
            }

            self.workspace.camera = camera;

            self.test = index;
        }
    }

    fn update_solution(&mut self) {
        let mut shapes = self.workspace.paths.simplify(FillRule::NonZero, 0);
        shapes.reverse_contours();

        // self.workspace.triangulations = shapes
        //     .iter()
        //     .map(|s| Triangulator::default().triangulate(s))
        //     .collect();

        self.workspace.paths = shapes.into_iter().flat_map(|s| s).collect();

        match self.mode {
            ModeOption::Edit => {}
            ModeOption::Debug => {
                // self.workspace.vectors = Overlay::with_contours(subj, clip).into_separate_vectors(FillRule::NonZero, Default::default());
            }
        }
    }

    pub(super) fn triangle_update_point(&mut self, update: PathEditorUpdateEvent) {
        self.workspace.paths[update.curve_index][update.point_index] = update.point;
        self.update_solution();
    }
}

impl Camera {
    fn with_size_and_curves(size: Size, paths: &Vec<IntPath>) -> Self {
        let rect = if paths.is_empty() {
            IntRect::new(-10_000, 10_000, -10_000, 10_000)
        } else {
            let mut rect = IntRect::new(i32::MAX, i32::MIN, i32::MAX, i32::MIN);
            for path in paths {
                for point in path.iter() {
                    rect.add_point(&point);
                }
            }
            rect
        };

        Self::new(rect, size)
    }
}
