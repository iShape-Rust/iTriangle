mod examples;

use crate::examples::{GridExample, Point, Shape as PolygonShape, load_examples};
use debug_ui::{
    camera::Camera,
    egui::{
        self, Color32, CursorIcon, Id, Painter, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2,
        epaint::PathShape,
    },
    grid::{Grid, paint_camera_readout},
};
use i_overlay::{
    core::fill_rule::FillRule,
    float::simplify::SimplifyShape,
    i_float::{adapter::FloatPointAdapter, float::rect::FloatRect},
    i_shape::float::adapter::ShapesToInt,
    mesh::{outline::offset::OutlineOffset, style::OutlineStyle},
};
use i_triangle::{
    float::{triangulation::Triangulation, uniform::UniformTriangulatable},
    tessellation::uniform::IntUniformGrid,
};

const PANEL_WIDTH: f32 = 270.0;

struct MeshResult {
    mesh: Triangulation<Point, u32>,
    steiner_points: Vec<Point>,
    offset_shapes: Vec<PolygonShape>,
}

struct UniformGridApp {
    camera: Camera,
    grid: Grid,
    examples: Vec<GridExample>,
    active_example: usize,
    edge_length: f32,
    boundary_offset: f32,
    show_fill: bool,
    show_triangles: bool,
    show_boundary: bool,
    show_offset: bool,
    show_steiner: bool,
    show_vertices: bool,
    result: Result<MeshResult, String>,
}

impl Default for UniformGridApp {
    fn default() -> Self {
        let examples = load_examples();
        let mut app = Self {
            camera: Camera::default(),
            grid: Grid::default(),
            edge_length: examples[0].edge_length,
            boundary_offset: examples[0].boundary_offset,
            examples,
            active_example: 0,
            show_fill: true,
            show_triangles: true,
            show_boundary: true,
            show_offset: true,
            show_steiner: true,
            show_vertices: false,
            result: Err("not calculated".to_owned()),
        };
        app.refresh_result();
        app.fit_active_example();
        app
    }
}

impl eframe::App for UniformGridApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("uniform_grid_panel")
            .resizable(false)
            .default_size(PANEL_WIDTH)
            .frame(egui::Frame::default().fill(Color32::from_rgb(24, 27, 32)))
            .show_inside(ui, |ui| self.show_controls(ui));

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(Color32::from_rgb(18, 20, 24)))
            .show_inside(ui, |ui| self.show_canvas(ui));
    }
}

impl UniformGridApp {
    fn show_controls(&mut self, ui: &mut Ui) {
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
        ui.add_space(8.0);

        let mut selected = None;
        ui.label("Contour");
        for (index, example) in self.examples.iter().enumerate() {
            if ui
                .selectable_label(index == self.active_example, example.name)
                .clicked()
            {
                selected = Some(index);
            }
        }

        if let Some(index) = selected {
            self.select_example(index);
        }

        ui.add_space(8.0);
        ui.separator();
        ui.label("Uniform grid");

        let edge_changed = ui
            .add(
                egui::DragValue::new(&mut self.edge_length)
                    .prefix("edge_length  ")
                    .range(1.0..=500.0)
                    .speed(1.0),
            )
            .changed();
        let offset_changed = ui
            .add(
                egui::DragValue::new(&mut self.boundary_offset)
                    .prefix("boundary_offset  ")
                    .range(0.0..=500.0)
                    .speed(1.0),
            )
            .changed();

        if edge_changed || offset_changed {
            self.refresh_result();
        }

        ui.add_space(8.0);
        ui.separator();
        ui.label("Layers");
        ui.checkbox(&mut self.show_fill, "triangle fill");
        ui.checkbox(&mut self.show_triangles, "Delaunay edges");
        ui.checkbox(&mut self.show_boundary, "input boundary");
        ui.checkbox(&mut self.show_offset, "inner offset");
        ui.checkbox(&mut self.show_steiner, "uniform Steiner points");
        ui.checkbox(&mut self.show_vertices, "all mesh vertices");

        ui.add_space(8.0);
        ui.separator();
        match &self.result {
            Ok(result) => {
                ui.label(format!("Contours: {}", self.active_example().shape.len()));
                ui.label(format!("Steiner points: {}", result.steiner_points.len()));
                ui.label(format!("Mesh vertices: {}", result.mesh.points.len()));
                ui.label(format!("Triangles: {}", result.mesh.indices.len() / 3));
            }
            Err(error) => {
                ui.colored_label(Color32::from_rgb(240, 118, 118), error);
            }
        }

        if ui.button("Fit view").clicked() {
            self.fit_active_example();
        }
        if ui.button("Reset example").clicked() {
            let index = self.active_example;
            self.examples[index] = load_examples().remove(index);
            self.edge_length = self.examples[index].edge_length;
            self.boundary_offset = self.examples[index].boundary_offset;
            self.refresh_result();
            self.fit_active_example();
        }

        ui.add_space(8.0);
        ui.separator();
        ui.small("Drag yellow boundary vertices to edit.");
        ui.small("Wheel: zoom. Right/middle drag: pan.");
    }

    fn show_canvas(&mut self, ui: &mut Ui) {
        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(available_size, Sense::click_and_drag());
        let rect = response.rect;

        self.grid
            .handle_input(ui, &response, rect, &mut self.camera);
        self.grid.paint(&painter, rect, &self.camera);

        if let Ok(result) = &self.result {
            paint_mesh(
                &painter,
                rect,
                &self.camera,
                result,
                self.show_fill,
                self.show_triangles,
                self.show_vertices,
            );

            if self.show_offset {
                paint_contours(
                    &painter,
                    rect,
                    &self.camera,
                    result.offset_shapes.iter().flatten(),
                    Stroke::new(1.5_f32, Color32::from_rgb(240, 163, 72)),
                );
            }

            if self.show_steiner {
                paint_points(
                    &painter,
                    rect,
                    &self.camera,
                    &result.steiner_points,
                    3.0,
                    Color32::from_rgb(233, 92, 132),
                );
            }
        }

        let camera = self.camera;
        let show_boundary = self.show_boundary;
        let changed = edit_shape(
            ui,
            &painter,
            rect,
            &camera,
            &mut self.examples[self.active_example].shape,
            show_boundary,
        );
        if changed {
            self.refresh_result();
        }

        paint_camera_readout(&painter, rect, &self.camera);
    }

    fn active_example(&self) -> &GridExample {
        &self.examples[self.active_example]
    }

    fn select_example(&mut self, index: usize) {
        self.active_example = index;
        self.edge_length = self.examples[index].edge_length;
        self.boundary_offset = self.examples[index].boundary_offset;
        self.refresh_result();
        self.fit_active_example();
    }

    fn refresh_result(&mut self) {
        let shape = self.active_example().shape.clone();
        let edge_length = self.edge_length;
        let boundary_offset = self.boundary_offset;

        self.result = match std::panic::catch_unwind(move || {
            build_mesh_result(&shape, edge_length, boundary_offset)
        }) {
            Ok(result) => result,
            Err(payload) => Err(panic_message(payload)),
        };
    }

    fn fit_active_example(&mut self) {
        let Some(bounds) = shape_bounds(&self.active_example().shape) else {
            return;
        };

        self.camera.center = Pos2::new(
            0.5 * (bounds.min_x + bounds.max_x),
            0.5 * (bounds.min_y + bounds.max_y),
        );
    }
}

fn build_mesh_result(
    shape: &PolygonShape,
    edge_length: f32,
    boundary_offset: f32,
) -> Result<MeshResult, String> {
    if !edge_length.is_finite() || edge_length <= 0.0 {
        return Err("edge_length must be finite and positive".to_owned());
    }
    if !boundary_offset.is_finite() || boundary_offset < 0.0 {
        return Err("boundary_offset must be finite and non-negative".to_owned());
    }

    // This is the public high-level API under test.
    let mesh = shape
        .uniform_triangulate_with_offset(edge_length, boundary_offset)
        .to_triangulation::<u32>();

    // Reproduce the grid-seeding stage so its exact IntUniformGrid output can be inspected.
    let simplified = shape.simplify_shape_as::<i32>(FillRule::NonZero);
    let rect = FloatRect::with_iter(simplified.iter().flatten().flatten())
        .ok_or_else(|| "shape is empty after simplification".to_owned())?;
    let adapter = FloatPointAdapter::<Point, i32>::new(rect);
    let int_edge_length = adapter.round_len_to_int(edge_length);
    if int_edge_length <= 1 {
        return Err("edge_length is below integer adapter precision".to_owned());
    }

    let inner = if boundary_offset > 0.0 {
        simplified.outline_as::<i32>(&OutlineStyle::new(-boundary_offset))
    } else {
        simplified.clone()
    };
    let inner_int = inner.to_int(&adapter);
    let steiner_points = inner_int
        .uniform_grid(int_edge_length as u64)
        .iter()
        .map(|point| adapter.int_to_float(point))
        .collect();

    Ok(MeshResult {
        mesh,
        steiner_points,
        offset_shapes: inner,
    })
}

fn paint_mesh(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    result: &MeshResult,
    show_fill: bool,
    show_edges: bool,
    show_vertices: bool,
) {
    let edge_stroke = Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(106, 185, 255, 190));
    let fill = Color32::from_rgba_unmultiplied(73, 170, 255, 30);

    for triangle in result.mesh.indices.chunks_exact(3) {
        let points = [triangle[0], triangle[1], triangle[2]].map(|index| {
            let point = result.mesh.points[index as usize];
            camera.screen_from_world(rect, point_to_pos(point))
        });

        if show_fill {
            painter.add(Shape::convex_polygon(
                points.to_vec(),
                fill,
                Stroke::new(0.0_f32, Color32::TRANSPARENT),
            ));
        }
        if show_edges {
            painter.add(Shape::closed_line(points.to_vec(), edge_stroke));
        }
    }

    if show_vertices {
        paint_points(
            painter,
            rect,
            camera,
            &result.mesh.points,
            2.0,
            Color32::from_rgb(128, 212, 156),
        );
    }
}

fn edit_shape(
    ui: &mut Ui,
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    contours: &mut [Vec<Point>],
    show_boundary: bool,
) -> bool {
    let mut changed = false;

    for (contour_index, contour) in contours.iter_mut().enumerate() {
        for (point_index, point) in contour.iter_mut().enumerate() {
            let screen = camera.screen_from_world(rect, point_to_pos(*point));
            let hit_rect = Rect::from_center_size(screen, Vec2::splat(18.0));
            let response = ui
                .interact(
                    hit_rect,
                    Id::new("boundary_point")
                        .with(contour_index)
                        .with(point_index),
                    Sense::drag(),
                )
                .on_hover_cursor(CursorIcon::Grab);

            if response.dragged()
                && let Some(screen_position) = ui.input(|input| input.pointer.interact_pos())
            {
                let world = camera.world_from_screen(rect, screen_position);
                *point = [world.x, world.y];
                changed = true;
            }

            let fill = if response.dragged() || response.hovered() {
                Color32::WHITE
            } else {
                Color32::from_rgb(255, 206, 102)
            };
            painter.circle(
                camera.screen_from_world(rect, point_to_pos(*point)),
                4.5,
                fill,
                Stroke::new(1.0_f32, Color32::from_rgb(18, 20, 24)),
            );
        }
    }

    if show_boundary {
        paint_contours(
            painter,
            rect,
            camera,
            contours.iter(),
            Stroke::new(2.5_f32, Color32::from_rgb(255, 206, 102)),
        );
    }

    changed
}

fn paint_contours<'a>(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    contours: impl Iterator<Item = &'a Vec<Point>>,
    stroke: Stroke,
) {
    for contour in contours {
        if contour.len() < 2 {
            continue;
        }
        let screen_points = contour
            .iter()
            .map(|point| camera.screen_from_world(rect, point_to_pos(*point)))
            .collect();
        painter.add(PathShape::closed_line(screen_points, stroke));
    }
}

fn paint_points(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    points: &[Point],
    radius: f32,
    color: Color32,
) {
    for point in points {
        painter.circle_filled(
            camera.screen_from_world(rect, point_to_pos(*point)),
            radius,
            color,
        );
    }
}

#[derive(Clone, Copy)]
struct Bounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

fn shape_bounds(shape: &PolygonShape) -> Option<Bounds> {
    let mut points = shape.iter().flatten();
    let first = *points.next()?;
    let mut bounds = Bounds {
        min_x: first[0],
        max_x: first[0],
        min_y: first[1],
        max_y: first[1],
    };

    for point in points {
        bounds.min_x = bounds.min_x.min(point[0]);
        bounds.max_x = bounds.max_x.max(point[0]);
        bounds.min_y = bounds.min_y.min(point[1]);
        bounds.max_y = bounds.max_y.max(point[1]);
    }

    Some(bounds)
}

fn point_to_pos(point: Point) -> Pos2 {
    Pos2::new(point[0], point[1])
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    let message = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown panic");

    format!("triangulation panic: {message}")
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Uniform Triangular Grid")
            .with_inner_size(Vec2::new(1100.0, 780.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "Uniform Triangular Grid",
        native_options,
        Box::new(|_cc| Ok(Box::new(UniformGridApp::default()))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_examples_build_meshes() {
        for example in load_examples() {
            let result =
                build_mesh_result(&example.shape, example.edge_length, example.boundary_offset)
                    .unwrap_or_else(|error| panic!("{}: {error}", example.name));
            assert!(
                !result.mesh.indices.is_empty(),
                "{} has no triangles",
                example.name
            );
        }
    }

    #[test]
    fn hole_case_has_no_steiner_points_in_hole() {
        let example = load_examples()
            .into_iter()
            .find(|example| example.name == "shape with hole")
            .expect("hole example");
        let result =
            build_mesh_result(&example.shape, example.edge_length, example.boundary_offset)
                .expect("hole example mesh");

        assert!(result.steiner_points.iter().all(|point| {
            point[0] <= -95.0 || point[0] >= 105.0 || point[1] <= -65.0 || point[1] >= 75.0
        }));

        for triangle in result.mesh.indices.chunks_exact(3) {
            let a = result.mesh.points[triangle[0] as usize];
            let b = result.mesh.points[triangle[1] as usize];
            let c = result.mesh.points[triangle[2] as usize];
            let centroid = [(a[0] + b[0] + c[0]) / 3.0, (a[1] + b[1] + c[1]) / 3.0];
            assert!(
                centroid[0] <= -95.0
                    || centroid[0] >= 105.0
                    || centroid[1] <= -65.0
                    || centroid[1] >= 75.0,
                "triangle centroid lies inside the hole: {centroid:?}"
            );
        }
    }

    #[test]
    fn narrow_case_falls_back_to_boundary_mesh() {
        let example = load_examples()
            .into_iter()
            .find(|example| example.name == "narrow contour")
            .expect("narrow example");
        let result =
            build_mesh_result(&example.shape, example.edge_length, example.boundary_offset)
                .expect("narrow example mesh");

        assert!(result.steiner_points.is_empty());
        assert!(!result.mesh.indices.is_empty());
    }
}
