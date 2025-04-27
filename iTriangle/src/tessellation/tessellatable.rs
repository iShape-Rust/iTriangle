use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::IntOverlayOptions;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};
use crate::advanced::delaunay::IntDelaunay;
use crate::int::triangulator::Triangulator;
use crate::tessellation::seeder::Seeder;

pub trait IntTessellatable {
    fn tessellate(&self, radius: i32) -> IntDelaunay;
}

impl IntTessellatable for IntContour {
    fn tessellate(&self, radius: i32) -> IntDelaunay {
        let mut seeder = Seeder::new(radius, self.len(), IntOverlayOptions::keep_all_points());
        seeder.add_contour(self);
        let (shapes, groups) = seeder.seed(FillRule::NonZero);
        Triangulator::default()
            .unchecked_triangulate_shapes_with_steiner_points(&shapes, &groups)
            .into_delaunay()
    }
}

impl IntTessellatable for IntShape {
    fn tessellate(&self, radius: i32) -> IntDelaunay {
        let mut seeder = Seeder::new(radius, self.len(), IntOverlayOptions::keep_all_points());
        seeder.add_shape(self);
        let (shapes, groups) = seeder.seed(FillRule::NonZero);
        Triangulator::default()
            .unchecked_triangulate_shapes_with_steiner_points(&shapes, &groups)
            .into_delaunay()
    }
}

impl IntTessellatable for IntShapes {
    fn tessellate(&self, radius: i32) -> IntDelaunay {
        let mut seeder = Seeder::new(radius, self.len(), IntOverlayOptions::keep_all_points());
        seeder.add_shapes(self);
        let (shapes, groups) = seeder.seed(FillRule::NonZero);
        Triangulator::default()
            .unchecked_triangulate_shapes_with_steiner_points(&shapes, &groups)
            .into_delaunay()
    }
}