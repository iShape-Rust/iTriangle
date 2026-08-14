use eframe::egui::{Pos2, Rect, Vec2};

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub center: Pos2,
    pub zoom: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            center: Pos2::ZERO,
            zoom: 1.0,
            min_zoom: 0.02,
            max_zoom: 256.0,
        }
    }
}

impl Camera {
    pub fn screen_from_world(&self, rect: Rect, world: Pos2) -> Pos2 {
        let origin = rect.center();

        Pos2::new(
            origin.x + (world.x - self.center.x) * self.zoom,
            origin.y - (world.y - self.center.y) * self.zoom,
        )
    }

    pub fn world_from_screen(&self, rect: Rect, screen: Pos2) -> Pos2 {
        let origin = rect.center();

        Pos2::new(
            self.center.x + (screen.x - origin.x) / self.zoom,
            self.center.y - (screen.y - origin.y) / self.zoom,
        )
    }

    pub fn world_delta_from_screen_delta(&self, delta: Vec2) -> Vec2 {
        Vec2::new(delta.x / self.zoom, -delta.y / self.zoom)
    }

    pub fn pan_by_screen_delta(&mut self, delta: Vec2) {
        self.center -= self.world_delta_from_screen_delta(delta);
    }

    pub fn zoom_at_screen_pos(&mut self, rect: Rect, screen_pos: Pos2, factor: f32) {
        let before = self.world_from_screen(rect, screen_pos);
        self.zoom = (self.zoom * factor).clamp(self.min_zoom, self.max_zoom);
        let after = self.world_from_screen(rect, screen_pos);
        self.center += before - after;
    }

    pub fn visible_world_rect(&self, rect: Rect) -> Rect {
        Rect::from_two_pos(
            self.world_from_screen(rect, rect.left_bottom()),
            self.world_from_screen(rect, rect.right_top()),
        )
    }
}
