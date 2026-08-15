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
    core::{
        fill_rule::FillRule, overlay::IntOverlayOptions, point_location::IntPointContainment,
        simplify::Simplify,
    },
    i_float::{
        adapter::FloatPointAdapter,
        float::rect::FloatRect,
        int::{point::IntPoint, rect::IntRect},
    },
    i_shape::{
        float::adapter::PathToInt,
        int::shape::{IntShape, IntShapes},
    },
};
use i_triangle::{
    float::{
        relax::RelaxationOptions, triangulation::Triangulation, uniform::UniformTriangulatable,
    },
    tessellation::{split::SliceContour, uniform::IntUniformGrid},
};

const PANEL_WIDTH: f32 = 270.0;
const TRIANGLE_HEIGHT_NUMERATOR: i64 = 28_378;
const TRIANGLE_HEIGHT_SHIFT: u32 = 15;

struct MeshResult {
    mesh: Triangulation<Point, u32>,
    resampled_boundary: Vec<PolygonShape>,
    contained_candidates: Vec<Point>,
    clearance_points: Vec<Point>,
    max_boundary_edge: f32,
    relaxation: Option<RelaxationStats>,
}

struct RelaxationStats {
    iterations: usize,
    converged: bool,
}

struct UniformGridApp {
    camera: Camera,
    grid: Grid,
    examples: Vec<GridExample>,
    active_example: usize,
    edge_length: f32,
    relax_enabled: bool,
    relax_iterations: usize,
    show_fill: bool,
    show_triangles: bool,
    show_boundary: bool,
    show_resampled_boundary: bool,
    show_contained_candidates: bool,
    show_clearance_points: bool,
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
            relax_enabled: false,
            relax_iterations: 40,
            examples,
            active_example: 0,
            show_fill: true,
            show_triangles: true,
            show_boundary: true,
            show_resampled_boundary: true,
            show_contained_candidates: true,
            show_clearance_points: true,
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
        if edge_changed {
            self.refresh_result();
        }

        let relax_changed = ui.checkbox(&mut self.relax_enabled, "relax mesh").changed();
        let iterations_changed = ui
            .add_enabled(
                self.relax_enabled,
                egui::DragValue::new(&mut self.relax_iterations)
                    .prefix("relax iterations  ")
                    .range(0..=1_000)
                    .speed(1),
            )
            .changed();
        if relax_changed || iterations_changed {
            self.refresh_result();
        }

        ui.add_space(8.0);
        ui.separator();
        ui.label("Layers");
        ui.checkbox(&mut self.show_fill, "triangle fill");
        ui.checkbox(&mut self.show_triangles, "Delaunay edges");
        ui.checkbox(&mut self.show_boundary, "input boundary");
        ui.checkbox(&mut self.show_resampled_boundary, "resampled boundary");
        ui.checkbox(
            &mut self.show_contained_candidates,
            "lattice after containment",
        );
        ui.checkbox(&mut self.show_clearance_points, "after edge clearance");
        ui.checkbox(&mut self.show_vertices, "all mesh vertices");

        ui.add_space(8.0);
        ui.separator();
        match &self.result {
            Ok(result) => {
                ui.label(format!("Contours: {}", self.active_example().shape.len()));
                ui.label(format!(
                    "After containment: {}",
                    result.contained_candidates.len()
                ));
                ui.label(format!(
                    "After clearance: {}",
                    result.clearance_points.len()
                ));
                ui.label(format!(
                    "Removed near edges: {}",
                    result.contained_candidates.len() - result.clearance_points.len()
                ));
                ui.label(format!(
                    "Boundary samples: {}",
                    result
                        .resampled_boundary
                        .iter()
                        .flatten()
                        .map(Vec::len)
                        .sum::<usize>()
                ));
                ui.colored_label(
                    Color32::from_rgb(128, 212, 156),
                    format!(
                        "Max boundary edge: {:.3} ≤ {:.3}",
                        result.max_boundary_edge, self.edge_length
                    ),
                );
                ui.label(format!(
                    "Clearance: edge_length / 3 = {:.3}",
                    self.edge_length / 3.0
                ));
                ui.label(format!("Mesh vertices: {}", result.mesh.points.len()));
                ui.label(format!("Triangles: {}", result.mesh.indices.len() / 3));
                if let Some(relaxation) = &result.relaxation {
                    ui.label(format!(
                        "Relax: {} iterations, converged: {}",
                        relaxation.iterations, relaxation.converged
                    ));
                }
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

            if self.show_contained_candidates {
                paint_points(
                    &painter,
                    rect,
                    &self.camera,
                    &result.contained_candidates,
                    3.5,
                    Color32::from_rgba_unmultiplied(190, 130, 255, 135),
                );
            }

            if self.show_clearance_points {
                paint_points(
                    &painter,
                    rect,
                    &self.camera,
                    &result.clearance_points,
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

        if self.show_resampled_boundary
            && let Ok(result) = &self.result
        {
            paint_contours(
                &painter,
                rect,
                &self.camera,
                result.resampled_boundary.iter().flatten(),
                Stroke::new(1.25_f32, Color32::from_rgb(80, 225, 220)),
            );
            paint_resampled_boundary_points(
                &painter,
                rect,
                &self.camera,
                &result.resampled_boundary,
            );
        }

        paint_camera_readout(&painter, rect, &self.camera);
    }

    fn active_example(&self) -> &GridExample {
        &self.examples[self.active_example]
    }

    fn select_example(&mut self, index: usize) {
        self.active_example = index;
        self.edge_length = self.examples[index].edge_length;
        self.refresh_result();
        self.fit_active_example();
    }

    fn refresh_result(&mut self) {
        let shape = self.active_example().shape.clone();
        let edge_length = self.edge_length;
        let relax_enabled = self.relax_enabled;
        let relax_iterations = self.relax_iterations;

        self.result = match std::panic::catch_unwind(move || {
            build_mesh_result(&shape, edge_length, relax_enabled, relax_iterations)
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
    relax_enabled: bool,
    relax_iterations: usize,
) -> Result<MeshResult, String> {
    if !edge_length.is_finite() || edge_length <= 0.0 {
        return Err("edge_length must be finite and positive".to_owned());
    }
    // This is the public high-level API under test.
    let mut delaunay = shape.uniform_triangulate(edge_length);
    let relaxation = relax_enabled.then(|| {
        let result = delaunay.relax_mut(RelaxationOptions::new(relax_iterations));
        RelaxationStats {
            iterations: result.iterations,
            converged: result.converged,
        }
    });
    let mesh = delaunay.to_triangulation::<u32>();

    // Reproduce the public float wrapper's single conversion into the integer pipeline.
    let rect = FloatRect::with_iter(shape.iter().flatten())
        .ok_or_else(|| "input shape is empty".to_owned())?;
    let adapter = FloatPointAdapter::<Point, i32>::new(rect);
    let int_edge_length = adapter.round_len_to_int(edge_length);
    if int_edge_length <= 1 {
        return Err("edge_length is below integer adapter precision".to_owned());
    }

    let int_shape: IntShape<i32> = shape.iter().map(|path| path.to_int(&adapter)).collect();

    // These are the same integer stages used by IntUniformTriangulatable.
    let split_boundary = int_shape.slice_contour(int_edge_length as u64);
    let resampled_boundary = vec![int_shape_to_float(&split_boundary, &adapter)];
    let max_boundary_edge = max_contour_edge(&resampled_boundary);
    let normalized =
        split_boundary.simplify(FillRule::NonZero, IntOverlayOptions::keep_all_points());
    let contained_int = lattice_after_containment(&normalized, int_edge_length as u64);
    let clearance_int = normalized.uniform_grid(int_edge_length as u64);
    let contained_candidates = int_points_to_float(&contained_int, &adapter);
    let clearance_points = int_points_to_float(&clearance_int, &adapter);

    Ok(MeshResult {
        mesh,
        resampled_boundary,
        contained_candidates,
        clearance_points,
        max_boundary_edge,
        relaxation,
    })
}

fn lattice_after_containment(shapes: &IntShapes<i32>, edge_length: u64) -> Vec<IntPoint<i32>> {
    let Some(rect) = IntRect::with_iter(shapes.iter().flatten().flatten()) else {
        return Vec::new();
    };
    let Ok(step) = i64::try_from(edge_length) else {
        return Vec::new();
    };
    if step <= 1 {
        return Vec::new();
    }

    // Keep this generator identical to IntUniformGrid's lattice stage. Only the
    // subsequent edge-clearance filter is intentionally omitted here.
    let row_step = (step * TRIANGLE_HEIGHT_NUMERATOR + (1_i64 << (TRIANGLE_HEIGHT_SHIFT - 1)))
        >> TRIANGLE_HEIGHT_SHIFT;
    if row_step <= 0 {
        return Vec::new();
    }

    let half_step = step / 2;
    let min_x = i64::from(rect.min_x);
    let max_x = i64::from(rect.max_x);
    let max_y = i64::from(rect.max_y);
    let mut candidates = Vec::new();
    let mut row = 0usize;
    let mut y = i64::from(rect.min_y) + row_step / 2;

    while y < max_y {
        let row_offset = if row & 1 == 0 { half_step } else { step };
        let mut x = min_x + row_offset;

        while x < max_x {
            candidates.push(IntPoint::new(
                i32::try_from(x).expect("lattice x stays inside i32 bounds"),
                i32::try_from(y).expect("lattice y stays inside i32 bounds"),
            ));
            x += step;
        }

        row += 1;
        y += row_step;
    }

    let contains = shapes.contains_points(&candidates);
    candidates
        .into_iter()
        .zip(contains)
        .filter_map(|(point, is_inside)| is_inside.then_some(point))
        .collect()
}

fn int_shape_to_float(
    shape: &IntShape<i32>,
    adapter: &FloatPointAdapter<Point, i32>,
) -> PolygonShape {
    shape
        .iter()
        .map(|contour| int_points_to_float(contour, adapter))
        .collect()
}

fn int_points_to_float(
    points: &[IntPoint<i32>],
    adapter: &FloatPointAdapter<Point, i32>,
) -> Vec<Point> {
    points
        .iter()
        .map(|point| adapter.int_to_float(point))
        .collect()
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

fn paint_resampled_boundary_points(
    painter: &Painter,
    rect: Rect,
    camera: &Camera,
    shapes: &[PolygonShape],
) {
    for contour in shapes.iter().flatten() {
        paint_points(
            painter,
            rect,
            camera,
            contour,
            2.75,
            Color32::from_rgb(80, 225, 220),
        );
    }
}

fn max_contour_edge(shapes: &[PolygonShape]) -> f32 {
    shapes
        .iter()
        .flatten()
        .filter(|contour| contour.len() > 1)
        .flat_map(|contour| {
            contour
                .iter()
                .zip(contour.iter().cycle().skip(1))
                .take(contour.len())
                .map(|(a, b)| {
                    let dx = b[0] - a[0];
                    let dy = b[1] - a[1];
                    (dx * dx + dy * dy).sqrt()
                })
        })
        .fold(0.0_f32, f32::max)
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
            let result = build_mesh_result(&example.shape, example.edge_length, false, 40)
                .unwrap_or_else(|error| panic!("{}: {error}", example.name));
            assert!(
                !result.mesh.indices.is_empty(),
                "{} has no triangles",
                example.name
            );
            assert!(
                result.max_boundary_edge <= example.edge_length + 0.001,
                "{} has a resampled boundary edge {} longer than edge_length {}",
                example.name,
                result.max_boundary_edge,
                example.edge_length
            );
            assert!(
                result
                    .clearance_points
                    .iter()
                    .all(|point| result.contained_candidates.contains(point))
            );
        }
    }

    #[test]
    fn hole_case_has_no_lattice_points_or_triangles_in_hole() {
        let example = load_examples()
            .into_iter()
            .find(|example| example.name == "shape with hole")
            .expect("hole example");
        let result = build_mesh_result(&example.shape, example.edge_length, false, 40)
            .expect("hole example mesh");

        for points in [&result.contained_candidates, &result.clearance_points] {
            assert!(points.iter().all(|point| {
                point[0] <= -95.0 || point[0] >= 105.0 || point[1] <= -65.0 || point[1] >= 75.0
            }));
        }

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
        let result = build_mesh_result(&example.shape, example.edge_length, false, 40)
            .expect("narrow example mesh");

        assert!(result.clearance_points.len() <= result.contained_candidates.len());
        assert!(!result.mesh.indices.is_empty());
    }

    #[test]
    fn clearance_stage_removes_near_boundary_candidates() {
        let example = load_examples()
            .into_iter()
            .find(|example| example.name == "shape with hole")
            .expect("hole example");
        let result = build_mesh_result(&example.shape, example.edge_length, false, 40)
            .expect("hole example mesh");

        assert!(result.contained_candidates.len() > result.clearance_points.len());
    }

    #[test]
    fn optional_relax_uses_requested_iteration_limit() {
        let example = load_examples().remove(0);
        let result = build_mesh_result(&example.shape, example.edge_length, true, 40)
            .expect("relaxed example mesh");
        let relaxation = result.relaxation.expect("relaxation stats");

        assert!(relaxation.iterations <= 40);
        assert!(!result.mesh.indices.is_empty());
    }
}
