use crate::camera::Camera;
use eframe::egui::{
    Align2, Color32, FontId, Painter, PointerButton, Rect, Response, Stroke, Ui, Vec2,
};

#[derive(Clone, Copy, Debug)]
pub struct Grid {
    pub base_step: f32,
    pub min_screen_step: f32,
    pub max_screen_step: f32,
    pub minor_stroke: Stroke,
    pub major_stroke: Stroke,
    pub axis_stroke: Stroke,
    pub background: Color32,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            base_step: 16.0,
            min_screen_step: 12.0,
            max_screen_step: 48.0,
            minor_stroke: Stroke::new(1.0_f32, Color32::from_gray(36)),
            major_stroke: Stroke::new(1.0_f32, Color32::from_gray(56)),
            axis_stroke: Stroke::new(1.5_f32, Color32::from_rgb(88, 102, 124)),
            background: Color32::from_rgb(18, 20, 24),
        }
    }
}

impl Grid {
    pub fn handle_input(&self, ui: &Ui, response: &Response, rect: Rect, camera: &mut Camera) {
        if response.hovered() {
            let scroll_y = ui.input(|input| input.smooth_scroll_delta.y);

            if scroll_y.abs() > f32::EPSILON {
                let factor = (scroll_y * 0.0018).exp();

                if let Some(pointer_pos) = ui.input(|input| input.pointer.hover_pos()) {
                    camera.zoom_at_screen_pos(rect, pointer_pos, factor);
                }
            }
        }

        if response.dragged_by(PointerButton::Middle)
            || response.dragged_by(PointerButton::Secondary)
        {
            camera.pan_by_screen_delta(response.drag_delta());
        }
    }

    pub fn paint(&self, painter: &Painter, rect: Rect, camera: &Camera) {
        painter.rect_filled(rect, 0.0, self.background);

        let world = camera.visible_world_rect(rect);
        let step = self.step_for_zoom(camera.zoom);
        let min_x_index = (world.left() / step).floor() as i32 - 1;
        let max_x_index = (world.right() / step).ceil() as i32 + 1;
        let min_y_index = (world.top() / step).floor() as i32 - 1;
        let max_y_index = (world.bottom() / step).ceil() as i32 + 1;

        for index in min_x_index..=max_x_index {
            let x = index as f32 * step;
            let stroke = self.stroke_for_index(index);
            let a = camera.screen_from_world(rect, eframe::egui::pos2(x, world.bottom()));
            let b = camera.screen_from_world(rect, eframe::egui::pos2(x, world.top()));
            painter.line_segment([a, b], stroke);
        }

        for index in min_y_index..=max_y_index {
            let y = index as f32 * step;
            let stroke = self.stroke_for_index(index);
            let a = camera.screen_from_world(rect, eframe::egui::pos2(world.left(), y));
            let b = camera.screen_from_world(rect, eframe::egui::pos2(world.right(), y));
            painter.line_segment([a, b], stroke);
        }
    }

    fn step_for_zoom(&self, zoom: f32) -> f32 {
        let mut step = self.base_step;

        while step * zoom < self.min_screen_step {
            step *= 2.0;
        }

        while step * zoom > self.max_screen_step {
            step *= 0.5;
        }

        step
    }

    fn stroke_for_index(&self, index: i32) -> Stroke {
        if index == 0 {
            self.axis_stroke
        } else if index.rem_euclid(5) == 0 {
            self.major_stroke
        } else {
            self.minor_stroke
        }
    }
}

pub fn paint_camera_readout(painter: &Painter, rect: Rect, camera: &Camera) {
    let text = format!(
        "center ({:.1}, {:.1})  zoom {:.2}x",
        camera.center.x, camera.center.y, camera.zoom
    );

    painter.text(
        rect.left_top() + Vec2::new(12.0, 10.0),
        Align2::LEFT_TOP,
        text,
        FontId::monospace(12.0),
        Color32::from_rgb(196, 202, 214),
    );
}
