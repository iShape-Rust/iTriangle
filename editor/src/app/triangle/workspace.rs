use i_mesh::i_triangle::i_overlay::i_shape::int::path::IntPath;
use i_mesh::i_triangle::int::triangulation::Triangulation;
use crate::geom::camera::Camera;
use crate::sheet::widget::SheetWidget;
use crate::app::triangle::content::IntersectMessage;
use crate::app::design::{style_sheet_background, Design};
use crate::app::main::{EditorApp, AppMessage};
use iced::widget::Stack;
use iced::widget::Container;
use iced::{Length, Padding, Size, Vector};
use crate::mesh_viewer::widget::MeshViewerWidget;
use crate::path_editor::widget::{PathEditorUpdateEvent, PathEditorWidget};

pub(crate) struct WorkspaceState {
    pub(crate) camera: Camera,
    pub(crate) paths: Vec<IntPath>,
    pub(crate) triangulations: Vec<Triangulation>,
}

impl EditorApp {
    pub(crate) fn triangle_workspace(&self) -> Container<AppMessage> {
        Container::new({
            let mut stack = Stack::new();
            stack = stack.push(
                Container::new(SheetWidget::new(
                    self.state.triangle.workspace.camera,
                    Design::negative_color().scale_alpha(0.5),
                    on_update_size,
                    on_update_zoom,
                    on_update_drag,
                ))
                    .width(Length::Fill)
                    .height(Length::Fill)
            );
            for (id, curve) in self.state.triangle.workspace.paths.iter().enumerate() {
                stack = stack.push(
                    Container::new(PathEditorWidget::new(
                        id,
                        curve,
                        self.state.triangle.workspace.camera,
                        on_update_anchor
                    ))
                        .width(Length::Fill)
                        .height(Length::Fill)
                );
            }
            for triangulation in self.state.triangle.workspace.triangulations.iter() {
                stack = stack.push(
                    Container::new(MeshViewerWidget::new(
                        triangulation,
                        self.state.triangle.workspace.camera
                    ))
                        .width(Length::Fill)
                        .height(Length::Fill)
                );
            }

            stack.push(
                Container::new(self.triangle_control())
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .padding(Padding::new(8.0))
            )
        })
            .style(style_sheet_background)
    }

    pub(super) fn triangle_update_anchor(&mut self, update: PathEditorUpdateEvent) {
        self.state.triangle.triangle_update_point(update);
    }

    pub(super) fn triangle_update_zoom(&mut self, camera: Camera) {
        self.state.triangle.workspace.camera = camera;
    }

    pub(super) fn triangle_update_drag(&mut self, new_pos: Vector<f32>) {
        self.state.triangle.workspace.camera.pos = new_pos;
    }
}

fn on_update_anchor(event: PathEditorUpdateEvent) -> AppMessage {
    AppMessage::Triangle(IntersectMessage::PathEdited(event))
}

fn on_update_size(size: Size) -> AppMessage {
    AppMessage::Triangle(IntersectMessage::WorkspaceSized(size))
}

fn on_update_zoom(zoom: Camera) -> AppMessage {
    AppMessage::Triangle(IntersectMessage::WorkspaceZoomed(zoom))
}

fn on_update_drag(drag: Vector<f32>) -> AppMessage {
    AppMessage::Triangle(IntersectMessage::WorkspaceDragged(drag))
}

impl Default for WorkspaceState {
    fn default() -> Self {
        WorkspaceState { camera: Camera::empty(), paths: vec![], triangulations: vec![] }
    }
}