use i_mesh::i_triangle::i_overlay::i_float::int::point::IntPoint;
use i_mesh::i_triangle::i_overlay::i_shape::int::path::IntPath;
use iced::advanced::graphics::color::pack;
use iced::{Rectangle, Transformation, Vector};
use crate::path_editor::color::PathEditorColorSchema;
use crate::path_editor::widget::{PathEditorUpdateEvent, PathEditorWidget};
use crate::geom::camera::Camera;
use crate::geom::vector::VectorExt;
use iced::advanced::graphics::Mesh;
use iced::advanced::graphics::mesh::{Indexed, SolidVertex2D};

#[derive(Clone)]
pub(super) struct MeshCache {
    radius: f32,
    pub(super) point: Mesh,
    pub(super) drag: Mesh,
    pub(super) hover: Mesh,
}

#[derive(Clone, Copy)]
pub(super) struct ActivePoint {
    pub(super) index: usize,
    pub(super) select_state: SelectState,
}


#[derive(Clone, Copy)]
pub(super) struct DragData {
    start_cursor: Vector<f32>,
    start_world: IntPoint
}

#[derive(Clone, Copy)]
pub(super) enum SelectState {
    Hover,
    Drag(DragData),
}

pub(crate) struct PathEditorState {
    pub(super) mesh_cache: Option<MeshCache>,
    pub(super) active_point: Option<ActivePoint>,
}

impl PathEditorState {
    pub(crate) fn update_mesh(&mut self, r: f32, schema: PathEditorColorSchema) {
        let radius = if let Some(cache) = &self.mesh_cache {
            cache.radius
        } else {
            0.0
        };

        if (radius - r).abs() < 0.1 {
            return;
        }

        let sr = 1.2 * r;

        let mut main_vertices = Vec::with_capacity(4);
        let mut hover_vertices = Vec::with_capacity(4);
        let mut drag_vertices = Vec::with_capacity(4);
        let mut indices = Vec::with_capacity(6);
        let main_pack = pack(schema.main);
        let hover_pack = pack(schema.hover);
        let drag_pack = pack(schema.drag);

        main_vertices.push(SolidVertex2D {
            position: [0.0, r],
            color: main_pack,
        });
        main_vertices.push(SolidVertex2D {
            position: [r, 2.0 * r],
            color: main_pack,
        });
        main_vertices.push(SolidVertex2D {
            position: [2.0 * r, r],
            color: main_pack,
        });
        main_vertices.push(SolidVertex2D {
            position: [r, 0.0],
            color: main_pack,
        });

        hover_vertices.push(SolidVertex2D {
            position: [0.0, sr],
            color: hover_pack,
        });
        hover_vertices.push(SolidVertex2D {
            position: [r, 2.0 * sr],
            color: hover_pack,
        });
        hover_vertices.push(SolidVertex2D {
            position: [2.0 * sr, sr],
            color: hover_pack,
        });
        hover_vertices.push(SolidVertex2D {
            position: [sr, 0.0],
            color: hover_pack,
        });

        drag_vertices.push(SolidVertex2D {
            position: [0.0, r],
            color: drag_pack,
        });
        drag_vertices.push(SolidVertex2D {
            position: [r, 2.0 * r],
            color: drag_pack,
        });
        drag_vertices.push(SolidVertex2D {
            position: [2.0 * r, r],
            color: drag_pack,
        });
        drag_vertices.push(SolidVertex2D {
            position: [r, 0.0],
            color: drag_pack,
        });

        indices.push(0);
        indices.push(1);
        indices.push(2);

        indices.push(0);
        indices.push(2);
        indices.push(3);

        self.mesh_cache = Some(MeshCache {
            radius: r,
            point: Mesh::Solid {
                buffers: Indexed {
                    vertices: main_vertices,
                    indices: indices.clone(),
                },
                transformation: Transformation::IDENTITY,
                clip_bounds: Rectangle::INFINITE,
            },
            hover: Mesh::Solid {
                buffers: Indexed {
                    vertices: hover_vertices,
                    indices: indices.clone(),
                },
                transformation: Transformation::translate(r - sr, r - sr),
                clip_bounds: Rectangle::INFINITE,
            },
            drag: Mesh::Solid {
                buffers: Indexed {
                    vertices: drag_vertices,
                    indices,
                },
                transformation: Transformation::IDENTITY,
                clip_bounds: Rectangle::INFINITE,
            },
        });
    }

    pub(super) fn mouse_press<M>(
        &mut self,
        widget: &PathEditorWidget<M>,
        cursor: Vector<f32>,
    ) -> bool {
        let closet_point = if let Some(point) = Self::find_closest_point(widget.camera, widget.hover_radius, &widget.path, cursor) {
            point
        } else {
            // self.active_anchor = None;
            return false;
        };

        let drag = DragData {
            start_cursor: cursor,
            start_world: closet_point.point
        };

        self.active_point = Some(ActivePoint {
            index: closet_point.index,
            select_state: SelectState::Drag(drag),
        });

        true
    }

    pub(super) fn mouse_release<M>(
        &mut self,
        widget: &PathEditorWidget<M>,
        cursor: Vector<f32>,
    ) -> bool {
        let active_anchor = if let Some(active) = self.active_point {
            active
        } else {
            return false;
        };

        if let SelectState::Drag(_) = &active_anchor.select_state {
            self.active_point = None;
            self.mouse_hover(widget.camera, widget.hover_radius, widget.path, cursor);
            true
        } else {
            false
        }
    }

    pub(super) fn mouse_move<M>(
        &mut self,
        widget: &PathEditorWidget<M>,
        cursor: Vector<f32>,
    ) -> Option<PathEditorUpdateEvent> {
        let active_state = &self.active_point?;
        if let SelectState::Drag(drag) = &active_state.select_state {
            Self::mouse_drag(
                widget.id,
                active_state.index, drag, widget.camera, widget.path, cursor)
        } else {
            self.mouse_hover(widget.camera, widget.hover_radius, widget.path, cursor);
            None
        }
    }

    fn mouse_drag(
        curve_index: usize,
        point_index: usize,
        drag: &DragData,
        camera: Camera,
        path: &IntPath,
        cursor: Vector<f32>,
    ) -> Option<PathEditorUpdateEvent> {
        let translate = cursor - drag.start_cursor;
        let world_dist = camera.view_distance_to_world(translate).round();
        let world_point = world_dist + drag.start_world;
        if world_point != path[point_index] {
            return Some(PathEditorUpdateEvent {
                curve_index,
                point_index,
                point: world_point,
            });
        }

        None
    }

    fn mouse_hover(&mut self, camera: Camera, radius: f32, path: &IntPath, cursor: Vector<f32>) {
        let closet_point = if let Some(close_point) = Self::find_closest_point(camera, radius, &path, cursor) {
            close_point
        } else {
            // self.active_anchor = None;
            return;
        };
        self.active_point = Some(ActivePoint {
            index: closet_point.index,
            select_state: SelectState::Hover,
        });
    }

    fn sqr_length(a: &Vector, b: &Vector) -> f32 {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        dx * dx + dy * dy
    }

    fn find_closest_point(camera: Camera, radius: f32, path: &IntPath, cursor: Vector<f32>) -> Option<ClosestPoint> {
        let mut min_ds = radius.powi(2);
        let mut closest_point = ClosestPoint {
            index: usize::MAX,
            point: IntPoint { x: 0, y: 0 }
        };

        for (i, &point) in path.iter().enumerate() {
            let view_pos = camera.int_world_to_view(point);
            let ds = Self::sqr_length(&cursor, &view_pos);
            if ds <= min_ds {
                min_ds = ds;
                closest_point.index = i;
                closest_point.point = point;
            }
        }

        if closest_point.index == usize::MAX {
            return None;
        }

        Some(closest_point)
    }
}

struct ClosestPoint {
    index: usize,
    point: IntPoint,
}

impl Default for PathEditorState {
    fn default() -> Self {
        Self {
            mesh_cache: None,
            active_point: Default::default()
        }
    }
}
