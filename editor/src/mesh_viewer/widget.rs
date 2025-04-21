use i_mesh::i_triangle::int::triangulation::Triangulation;
use crate::compat::convert::Convert;
use crate::geom::camera::Camera;
use crate::mesh::path_builder::PathBuilder;
use iced::advanced::widget::Tree;
use iced::advanced::{Layout, Widget, layout, renderer};
use iced::{Element, Length, Rectangle, Renderer, Size, Theme, Vector, mouse};
use crate::mesh_viewer::color::MeshViewerColorSchema;

pub(crate) struct MeshViewerWidget<'a> {
    pub(super) triangulation: &'a Triangulation,
    pub(super) camera: Camera,
    pub(super) schema: MeshViewerColorSchema,
}

impl<'a> MeshViewerWidget<'a> {
    pub(crate) fn new(
        triangulation: &'a Triangulation,
        camera: Camera,
    ) -> Self {
        Self {
            triangulation,
            camera,
            schema: MeshViewerColorSchema::with_theme(Theme::default())
        }
    }
}

impl<Message> Widget<Message, Theme, Renderer> for MeshViewerWidget<'_> {

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        use iced::advanced::graphics::mesh::Renderer as _;

        let offset = layout.position();
        let offset_vec = Vector::new(offset.x, offset.y);

        let mut contour_builder = PathBuilder::new(self.camera, offset_vec.convert());
        let mut i = 0;
        while i < self.triangulation.indices.len() {
            let ai = self.triangulation.indices[i];
            let bi = self.triangulation.indices[i + 1];
            let ci = self.triangulation.indices[i+ 2];

            let a = self.triangulation.points[ai];
            let b = self.triangulation.points[bi];
            let c = self.triangulation.points[ci];
            contour_builder.add_int_segment(a, b, 1.0);
            contour_builder.add_int_segment(b, c, 1.0);
            contour_builder.add_int_segment(c, a, 1.0);
            i += 3;
        }
        if let Some(mesh) = contour_builder.into_mesh(self.schema.main) {
            renderer.with_translation(Vector::new(0.0, 0.0), |renderer| renderer.draw_mesh(mesh));
        }
    }
}

impl<'a, Message: 'a> From<MeshViewerWidget<'a>> for Element<'a, Message> {
    fn from(editor: MeshViewerWidget<'a>) -> Self {
        Self::new(editor)
    }
}
