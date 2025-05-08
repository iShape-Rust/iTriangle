use i_triangle::i_overlay::i_float::int::point::IntPoint;
use i_triangle::i_overlay::i_shape::int::path::IntPath;
use crate::compat::convert::Convert;
use crate::geom::camera::Camera;
use crate::mesh::path_builder::PathBuilder;
use crate::path_editor::color::PathEditorColorSchema;
use crate::path_editor::state::{ActivePoint, MeshCache, PathEditorState, SelectState};
use iced::advanced::graphics::Mesh;
use iced::advanced::widget::tree::State;
use iced::advanced::widget::{Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, renderer};
use iced::{
    Color, Element, Event, Length, Point, Rectangle, Renderer, Size, Theme, Vector, event, mouse,
};

pub(crate) struct PathEditorWidget<'a, Message> {
    pub(super) id: usize,
    pub(super) path: &'a IntPath,
    pub(super) camera: Camera,
    pub(super) schema: PathEditorColorSchema,
    pub(super) mesh_radius: f32,
    pub(super) hover_radius: f32,
    pub(super) split_factor: usize,
    on_update: Box<dyn Fn(PathEditorUpdateEvent) -> Message + 'a>,
}

#[derive(Debug, Clone)]
pub(crate) struct PathEditorUpdateEvent {
    pub(crate) curve_index: usize,
    pub(crate) point_index: usize,
    pub(crate) point: IntPoint,
}

impl<'a, Message> PathEditorWidget<'a, Message> {
    pub(crate) fn new(
        id: usize,
        path: &'a IntPath,
        camera: Camera,
        on_update: impl Fn(PathEditorUpdateEvent) -> Message + 'a,
    ) -> Self {
        Self {
            id,
            path,
            camera,
            mesh_radius: 6.0,
            hover_radius: 12.0,
            split_factor: 5,
            schema: PathEditorColorSchema::with_theme(Theme::default()),
            on_update: Box::new(on_update),
        }
    }

    pub(crate) fn set_schema(mut self, schema: PathEditorColorSchema) -> Self {
        self.schema = schema;
        self
    }
}

impl<Message> Widget<Message, Theme, Renderer> for PathEditorWidget<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<PathEditorState>()
    }

    fn state(&self) -> State {
        State::new(PathEditorState::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        if let State::Some(state_box) = &mut tree.state {
            state_box
                .downcast_mut::<PathEditorState>()
                .unwrap()
                .update_mesh(self.mesh_radius, self.schema)
        };

        layout::Node::new(limits.max())
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let state = tree.state.downcast_mut::<PathEditorState>();

        let bounds = layout.bounds();
        if let Event::Mouse(mouse_event) = event {
            match mouse_event {
                mouse::Event::CursorMoved { position } => {
                    if bounds.contains(position) {
                        let view_cursor = position - bounds.position();
                        if let Some(updated_point) = state.mouse_move(&*self, view_cursor) {
                            shell.publish((self.on_update)(updated_point));
                            return event::Status::Captured;
                        }
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    let position = cursor.position().unwrap_or(Point::ORIGIN);
                    if bounds.contains(position) {
                        let view_cursor = position - bounds.position();
                        if state.mouse_press(&*self, view_cursor) {
                            return event::Status::Captured;
                        }
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    let position = cursor.position().unwrap_or(Point::ORIGIN);
                    let view_cursor = position - bounds.position();
                    if state.mouse_release(&*self, view_cursor) {
                        return event::Status::Captured;
                    }
                }
                _ => {}
            }
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<PathEditorState>();

        let mesh_cache = if let Some(mesh) = &state.mesh_cache {
            mesh
        } else {
            return;
        };

        use iced::advanced::Renderer as _;
        use iced::advanced::graphics::mesh::Renderer as _;

        let offset = layout.position();
        let offset_vec = Vector::new(offset.x, offset.y);
        let radius_offset = offset - Point::new(self.mesh_radius, self.mesh_radius);

        let mut contour_builder = PathBuilder::new(self.camera, offset_vec.convert());
        contour_builder.add_paths(&self.path, true, 4.0);
        if let Some(mesh) = contour_builder.into_mesh(Color::new(1.0, 1.0, 1.0, 0.2)) {
            renderer.with_translation(Vector::new(0.0, 0.0), |renderer| renderer.draw_mesh(mesh));
        }

        for (index, point) in self.path.iter().enumerate() {
            let main_screen = self.camera.world_to_screen(radius_offset, point.convert());

            if let Some(active) = &state.active_point {
                let mesh = if active.index == index {
                    mesh_cache.active_point(active)
                } else {
                    mesh_cache.point.clone()
                };

                renderer.with_translation(main_screen, |renderer| renderer.draw_mesh(mesh));
            }
        }
    }
}

impl MeshCache {
    fn active_point(&self, point: &ActivePoint) -> Mesh {
        match point.select_state {
            SelectState::Hover => self.hover.clone(),
            SelectState::Drag(_) => self.hover.clone(),
        }
    }
}

impl<'a, Message: 'a> From<PathEditorWidget<'a, Message>> for Element<'a, Message> {
    fn from(editor: PathEditorWidget<'a, Message>) -> Self {
        Self::new(editor)
    }
}
