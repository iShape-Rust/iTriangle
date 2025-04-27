use crate::tessellation::split::Split;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::{IntOverlayOptions, Overlay, ShapeType};
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::i_shape::int::shape::{IntContour, IntShape, IntShapes};
use i_overlay::mesh::outline::offset::OutlineOffset;
use i_overlay::mesh::style::OutlineStyle;
use i_tree::key::exp::KeyExpCollection;
use i_tree::key::list::KeyExpList;
use crate::int::v_segment::VSegment;
use crate::tessellation::convert::{ToFloatShape, ToIntShape};

pub(super) struct Seeder {
    radius: u64,
    overlay: Overlay,
}

impl Seeder {
    #[inline]
    pub(super) fn new(radius: i32, capacity: usize, options: IntOverlayOptions) -> Self {
        let r = radius.unsigned_abs().max(1) as u64;
        Self {
            radius: r,
            overlay: Overlay::with_options(capacity, options),
        }
    }

    #[inline]
    pub(super) fn add_contour(&mut self, contour: &IntContour) {
        self.overlay
            .add_contour(&contour.split_contour(self.radius), ShapeType::Subject);
    }

    #[inline]
    pub(super) fn add_shape(&mut self, shape: &IntShape) {
        self.overlay
            .add_shape(&shape.split_contour(self.radius), ShapeType::Subject);
    }

    #[inline]
    pub(super) fn add_shapes(&mut self, shapes: &IntShapes) {
        self.overlay
            .add_shapes(&shapes.split_contour(self.radius), ShapeType::Subject);
    }

    pub(super) fn seed(self, fill_rule: FillRule) -> (IntShapes, Vec<Vec<IntPoint>>) {
        let r = self.radius as f64;
        let shapes = self.overlay.overlay(OverlayRule::Subject, fill_rule);
        if shapes.is_empty() {
            return (shapes, Vec::new());
        }

        let mut group = Vec::with_capacity(shapes.len());

        for shape in shapes.iter() {
            let points = seed_in_shape(r, shape);
            group.push(points);
        }

        (shapes, group)
    }
}

fn seed_in_shape(
    radius: f64,
    shape: &IntShape,
) -> Vec<IntPoint> {
    let style = OutlineStyle::new(-radius);
    let float_shape = shape.to_float_shape();
    let outline = float_shape.outline(&style);

    let mut result = Vec::new();
    for sub_shape in outline {
        let int_shape = sub_shape.to_int_shape();
        seed_in_outline(radius as i64, &int_shape, &mut result);
    }

    result
}

fn seed_in_outline(radius: i64, outline: &IntShape, output: &mut Vec<IntPoint>) {

    let mut segments = Vec::new();

    for contour in outline.iter() {
        let mut a = *contour.last().unwrap();
        for &b in contour.iter() {
            if a.x == b.x {
                a = b;
                continue;
            };

            let v_segment = if a.x < b.x {
                VSegment { a, b }
            } else {
                VSegment { a: b, b: a }
            };
            segments.push(v_segment);

            a = b;
        }
    }
    if segments.is_empty() {
        return;
    }

    segments.sort_unstable_by(|s0, s1| s0.a.cmp(&s1.a));

    let mut fx = segments.first().unwrap().a.x as f64;

    let mut i = 0;

    let mut list = KeyExpList::new(16);

    let r = radius as f64;
    let step_x = r;
    let step_y = f64::sqrt(3.0_f64) * r;

    fx -= 0.5 * step_x;

    while i < segments.len() || !list.is_empty() {
        fx += step_x;
        let x = fx as i32;

        while i < segments.len() && segments[i].a.x <= x {
            let s = segments[i];
            if x < s.b.x {
                list.insert(s, s, x);
            }
            i += 1;
        }

        list.clear_expired(x);

        if list.is_empty() {
            continue;
        }

        let mut iter = list.ordered_values();
        while let (Some(s0), Some(s1)) = (iter.next(), iter.next()) {
            let offset = if x & 1 == 0 {
                1.0
            } else {
                1.0 + 0.5 * step_y
            };

            let y_min = s0.y_for_x(x as f64) + offset;
            let y_max = s1.y_for_x(x as f64) - offset;

            if y_max < y_min {
                continue;
            }

            output.push(IntPoint::new(x, y_min as i32));

            let mut fy = y_min + step_y;

            while fy < y_max {
                let y = fy as i32;
                output.push(IntPoint::new(x, y));
                fy += step_y;
            }
        }
    }
}

impl VSegment {
    #[inline]
    fn y_for_x(&self, x: f64) -> f64 {
        // x0 != x1 by design
        let x0 = self.a.x as f64;
        let x1 = self.b.x as f64;
        let y0 = self.a.y as f64;
        let y1 = self.b.y as f64;

        let dx = x0 - x1;
        let dy = y0 - y1;

        (dy * x + x0 * y1 - y0 * x1) / dx
    }
}

#[cfg(test)]
mod tests {
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::core::overlay::IntOverlayOptions;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_shape::int::path::IntPath;
    use crate::tessellation::seeder::Seeder;

    fn path(slice: &[[i32; 2]]) -> IntPath {
        slice.iter().map(|p| IntPoint::new(p[0], p[1])).collect()
    }

    #[test]
    fn test_0() {
        let contour = path(&[[0, 0], [20, 0], [20, 20], [0, 20]]);

        let mut seeder = Seeder::new(3, 8, IntOverlayOptions::keep_all_points());
        seeder.add_contour(&contour);
        let (shapes, groups) = seeder.seed(FillRule::NonZero);


        assert_eq!(shapes.len(), 1);
        assert_eq!(groups.len(), 1);
    }

    #[test]
    fn test_1() {
        let main = path(&[[0, 0], [20, 0], [20, 20], [0, 20]]);
        let hole = path(&[[5, 5], [15, 5], [15, 15], [5, 15]]);
        let shape = vec![main, hole];

        let mut seeder = Seeder::new(3, 8, IntOverlayOptions::keep_all_points());
        seeder.add_shape(&shape);
        let (shapes, groups) = seeder.seed(FillRule::NonZero);


        assert_eq!(shapes.len(), 1);
        assert_eq!(groups.len(), 1);
    }
}


