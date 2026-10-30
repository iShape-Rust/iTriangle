//! Minkowski operations on prepared, filled integer regions.
//!
//! Preparation resolves the selected [`FillRule`], triangulates the normalized
//! region, and groups triangles into convex parts. Sums and differences merge
//! each pair's edge directions and union the resulting polygons. Concavities,
//! holes, and disconnected components are handled through this decomposition.
//!
//! Difference means `A + (-B) = {a - b | a in A, b in B}`, not erosion or
//! Boolean subtraction. It is not the inverse of a Minkowski sum.
//!
//! # Reuse and empty regions
//!
//! ```
//! use i_triangle::int::minkowski::{IntMinkowskiSolver, MinkowskiError};
//! use i_triangle::i_overlay::core::fill_rule::FillRule;
//! use i_triangle::i_overlay::i_float::int::point::IntPoint;
//! use i_triangle::i_overlay::i_shape::int_path;
//!
//! let source = int_path![[0, 0], [4, 0], [4, 4], [0, 4]];
//! let kernel = int_path![[0, 0], [1, 0], [1, 1], [0, 1]];
//! let mut solver = IntMinkowskiSolver::<i32>::default();
//! let a = solver.prepare(&source, FillRule::NonZero).unwrap();
//! let b = solver.prepare(&kernel, FillRule::NonZero).unwrap();
//! let sum = solver.sum(&a, &b)?;
//! solver.clear();
//! let difference = solver.difference(&a, &b)?;
//! assert!(sum[0][0].contains(&(source[2] + kernel[2])));
//! assert!(difference[0][0].contains(&IntPoint::new(-1, -1)));
//! let line = int_path![[0, 0], [1, 0], [2, 0]];
//! assert!(solver.prepare(&line, FillRule::NonZero).is_none());
//! # Ok::<(), MinkowskiError>(())
//! ```
//!
//! Only two-dimensional filled regions are represented: points, segments,
//! empty sources, and sources with no filled area produce `None` in preparation.
//! Integer operands own their parts and can be used with any integer solver of
//! the same coordinate type. The floating-point API adds solver ownership to
//! protect its fixed coordinate transform.
//!
//! # Coordinate range and rounding
//!
//! Input coordinates must satisfy the [overlay integer range](i_overlay::core::integer).
//! Preparation does not validate this arithmetic precondition. Operations check
//! conservative result bounds before generating pairs, requiring every result
//! coordinate to be strictly between `-2^(I::BITS - 2)` and `2^(I::BITS - 2)`.
//! Bounds come from the original source, including contours removed by filling.
//! Intersections are rounded to the integer grid, so small features can change
//! during normalization and union. Increase coordinate precision before
//! preparation if one grid unit is too coarse for the application.

use crate::advanced::delaunay::IntDelaunay;
use crate::int::minkowski_sum::ConvexSum;
use crate::int::triangulator::IntTriangulator;
use alloc::vec::Vec;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::integer::OverlayInt;
use i_overlay::core::overlay::{IntOverlayOptions, ShapeType};
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::rect::IntRect;
use i_overlay::i_shape::int::shape::{IntContour, IntShapes};
use i_overlay::i_shape::source::int::resource::IntShapeResource;

/// A numeric, domain, or operand-ownership failure in a Minkowski operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinkowskiError {
    /// A coordinate or conservative result bound exceeds the supported range.
    /// Floating-point input must also be finite.
    CoordinateOutOfRange,
    /// Original source or result bounds extend outside a float solver's domain.
    OutsideDomain,
    /// A floating-point operand was prepared by a different solver instance.
    ForeignOperand,
}

/// A reusable integer triangulator and union workspace for Minkowski operations.
///
/// Prepared operands are independent of the workspace and survive clearing it.
/// `I` can be `i16`, `i32`, or `i64`; callers must respect the module's input
/// [coordinate-range contract](self#coordinate-range-and-rounding).
pub struct IntMinkowskiSolver<I: OverlayInt> {
    triangulator: IntTriangulator<I, usize>,
    output_options: IntOverlayOptions<I::WideUInt>,
}

/// An owned convex decomposition of a normalized, nonempty filled region.
///
/// Prepare with [`Self::with_source`] or [`IntMinkowskiSolver::prepare`]. The
/// original source may be dropped, and the operand may be reused with any
/// [`IntMinkowskiSolver`] of the same integer type without further triangulation.
pub struct IntMinkowskiOperand<I: OverlayInt> {
    convex_polygons: Vec<IntContour<I>>,
    bbox: IntRect<I>,
}

impl<I: OverlayInt> IntMinkowskiOperand<I> {
    /// Prepares a source using a temporary default solver and `fill_rule`.
    ///
    /// Returns `None` for an empty or unfilled region, including points and
    /// segments. Input must satisfy the module's coordinate-range contract.
    /// For repeated preparations, reuse [`IntMinkowskiSolver::prepare`].
    pub fn with_source<R>(source: &R, fill_rule: FillRule) -> Option<Self>
    where
        R: IntShapeResource<I> + ?Sized,
    {
        IntMinkowskiSolver::default().prepare(source, fill_rule)
    }

    /// Borrows the prepared convex parts in counter-clockwise order.
    ///
    /// These describe the filled region after normalization and integer rounding,
    /// rather than the original contour grouping or vertex sequence.
    pub fn convex_polygons(&self) -> &[IntContour<I>] {
        &self.convex_polygons
    }

    /// Returns the original source bounds, before fill filtering.
    /// These conservative bounds are used for result-range checks.
    pub fn bbox(&self) -> IntRect<I> {
        self.bbox
    }
}

impl<I: OverlayInt> IntMinkowskiSolver<I> {
    /// Creates a solver with an initial capacity hint and simplified collinear output.
    /// Capacity is not a limit on source or result size.
    pub fn new(capacity: usize) -> Self {
        Self::with_options(capacity, false)
    }

    /// Creates a solver with an initial capacity hint and a final-output policy.
    ///
    /// `preserve_output_collinear` retains collinear vertices in the final union.
    /// Collinear edges of each convex pair are always merged first, so enabling
    /// this option does not preserve every original source vertex. Outer output
    /// contours are counter-clockwise; hole contours are clockwise.
    pub fn with_options(capacity: usize, preserve_output_collinear: bool) -> Self {
        Self {
            output_options: IntOverlayOptions {
                preserve_output_collinear,
                ..Default::default()
            },
            triangulator: IntTriangulator::new(capacity, Default::default(), Default::default()),
        }
    }

    /// Clears temporary overlay geometry while retaining its allocations.
    /// Prepared operands and the configured output policy remain usable.
    pub fn clear(&mut self) {
        self.triangulator.overlay.clear();
    }

    /// Normalizes a source with `fill_rule` and prepares its convex parts once.
    ///
    /// Accepts contours, shapes with holes, and collections through
    /// [`IntShapeResource`]. Returns `None` if no filled area remains, including
    /// empty, point, and segment input. Existing operands remain valid.
    ///
    /// Input coordinates must satisfy the [overlay integer range](i_overlay::core::integer);
    /// this method does not perform numeric range validation.
    pub fn prepare<R>(&mut self, source: &R, fill_rule: FillRule) -> Option<IntMinkowskiOperand<I>>
    where
        R: IntShapeResource<I> + ?Sized,
    {
        self.clear();
        let bbox = IntRect::with_iter(source.iter_paths().flatten())?;
        self.triangulator.overlay.options = IntOverlayOptions::keep_output_points();
        let shapes = self.triangulator.overlay.simplify_source(source, fill_rule);
        if shapes.is_empty() {
            return None;
        }

        let mut delaunay = IntDelaunay {
            points: Vec::new(),
            triangles: Vec::new(),
        };
        self.triangulator
            .uncheck_delaunay_shapes_into(&shapes, &mut delaunay);
        let convex_polygons = delaunay.to_convex_polygons();
        debug_assert!(!convex_polygons.is_empty());

        Some(IntMinkowskiOperand {
            convex_polygons,
            bbox,
        })
    }

    /// Returns `{a + b | a in lhs, b in rhs}` using the prepared convex parts.
    ///
    /// # Errors
    /// Returns [`MinkowskiError::CoordinateOutOfRange`] when the sum of the
    /// original source bounds reaches or exceeds the strict coordinate limits
    /// documented in this module. No pairwise points are generated on that error,
    /// and the solver remains reusable.
    pub fn sum(
        &mut self,
        lhs: &IntMinkowskiOperand<I>,
        rhs: &IntMinkowskiOperand<I>,
    ) -> Result<IntShapes<I>, MinkowskiError> {
        self.sum_parts(
            &lhs.convex_polygons,
            lhs.bbox,
            &rhs.convex_polygons,
            rhs.bbox,
        )
    }

    pub(crate) fn sum_parts<R0, R1>(
        &mut self,
        lhs: &R0,
        lhs_bbox: IntRect<I>,
        rhs: &R1,
        rhs_bbox: IntRect<I>,
    ) -> Result<IntShapes<I>, MinkowskiError>
    where
        R0: IntShapeResource<I> + ?Sized,
        R1: IntShapeResource<I> + ?Sized,
    {
        self.clear();
        self.triangulator.overlay.options = self.output_options;
        self.triangulator.overlay.options.preserve_input_collinear = true;

        Self::validate_bounds([
            lhs_bbox.min_x.to_wide() + rhs_bbox.min_x.to_wide(),
            lhs_bbox.max_x.to_wide() + rhs_bbox.max_x.to_wide(),
            lhs_bbox.min_y.to_wide() + rhs_bbox.min_y.to_wide(),
            lhs_bbox.max_y.to_wide() + rhs_bbox.max_y.to_wide(),
        ])?;

        for a in lhs.iter_paths() {
            for b in rhs.iter_paths() {
                if let Some(path) = ConvexSum::new(a, b) {
                    self.triangulator
                        .overlay
                        .add_path_iter(path, ShapeType::Subject);
                }
            }
        }

        Ok(self
            .triangulator
            .overlay
            .overlay(OverlayRule::Subject, FillRule::NonZero))
    }

    /// Returns `{a - b | a in lhs, b in rhs}`, reflecting `rhs` through the origin.
    ///
    /// This is `lhs + (-rhs)`, not erosion, Boolean subtraction, or an inverse of
    /// [`Self::sum`]. No repeated triangulation is performed.
    ///
    /// # Errors
    /// Returns [`MinkowskiError::CoordinateOutOfRange`] if the difference of the
    /// original source bounds reaches or exceeds the module's strict coordinate
    /// limits. The solver remains reusable after an error.
    pub fn difference(
        &mut self,
        lhs: &IntMinkowskiOperand<I>,
        rhs: &IntMinkowskiOperand<I>,
    ) -> Result<IntShapes<I>, MinkowskiError> {
        self.difference_parts(
            &lhs.convex_polygons,
            lhs.bbox,
            &rhs.convex_polygons,
            rhs.bbox,
        )
    }

    pub(crate) fn difference_parts<R0, R1>(
        &mut self,
        lhs: &R0,
        lhs_bbox: IntRect<I>,
        rhs: &R1,
        rhs_bbox: IntRect<I>,
    ) -> Result<IntShapes<I>, MinkowskiError>
    where
        R0: IntShapeResource<I> + ?Sized,
        R1: IntShapeResource<I> + ?Sized,
    {
        self.clear();
        self.triangulator.overlay.options = self.output_options;
        self.triangulator.overlay.options.preserve_input_collinear = true;

        Self::validate_bounds([
            lhs_bbox.min_x.to_wide() - rhs_bbox.max_x.to_wide(),
            lhs_bbox.max_x.to_wide() - rhs_bbox.min_x.to_wide(),
            lhs_bbox.min_y.to_wide() - rhs_bbox.max_y.to_wide(),
            lhs_bbox.max_y.to_wide() - rhs_bbox.min_y.to_wide(),
        ])?;

        for a in lhs.iter_paths() {
            for b in rhs.iter_paths() {
                if let Some(path) = ConvexSum::difference(a, b) {
                    self.triangulator
                        .overlay
                        .add_path_iter(path, ShapeType::Subject);
                }
            }
        }

        Ok(self
            .triangulator
            .overlay
            .overlay(OverlayRule::Subject, FillRule::NonZero))
    }

    pub(crate) fn validate_bounds(bounds: [I::Wide; 4]) -> Result<(), MinkowskiError> {
        let limit = I::Wide::ONE << (I::BITS - 2);
        if bounds
            .iter()
            .any(|&value| value <= -limit || value >= limit)
        {
            Err(MinkowskiError::CoordinateOutOfRange)
        } else {
            Ok(())
        }
    }
}

impl<I: OverlayInt> Default for IntMinkowskiSolver<I> {
    fn default() -> Self {
        Self::new(64)
    }
}

#[cfg(test)]
mod tests {
    use super::IntMinkowskiSolver;
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_float::int::rect::IntRect;
    use i_overlay::i_shape::int::shape::IntContour;
    use i_overlay::i_shape::int_path;

    #[test]
    fn repeated_operations_preserve_results() {
        let contour: IntContour<i32> = (-32..=32).map(|x| IntPoint::new(x, x * x)).collect();
        let mut solver = IntMinkowskiSolver::new(0);
        let operand = solver.prepare(&contour, FillRule::NonZero).unwrap();
        assert_eq!(operand.convex_polygons.len(), 1);
        let expected = solver.sum(&operand, &operand).unwrap();
        for _ in 0..3 {
            let operand = solver.prepare(&contour, FillRule::NonZero).unwrap();
            assert_eq!(solver.sum(&operand, &operand).unwrap(), expected);
        }
    }

    #[test]
    fn degenerate_parts_are_combined_before_discarding_zero_area_results() {
        let point = int_path![[2, 3], [2, 3], [2, 3]];
        let horizontal = int_path![[0, 0], [2, 0], [2, 0]];
        let vertical = int_path![[0, 0], [0, 2], [0, 2]];
        let point_bbox = IntRect::with_points(&point).unwrap();
        let h_bbox = IntRect::with_points(&horizontal).unwrap();
        let v_bbox = IntRect::with_points(&vertical).unwrap();
        let mut solver = IntMinkowskiSolver::default();
        assert!(solver
            .sum_parts(&point, point_bbox, &point, point_bbox)
            .unwrap()
            .is_empty());
        assert!(solver
            .difference_parts(&horizontal, h_bbox, &horizontal, h_bbox)
            .unwrap()
            .is_empty());
        let sum = solver
            .sum_parts(&horizontal, h_bbox, &vertical, v_bbox)
            .unwrap();
        let difference = solver
            .difference_parts(&horizontal, h_bbox, &vertical, v_bbox)
            .unwrap();
        let expected_sum = int_path![[0, 0], [2, 0], [2, 2], [0, 2]];
        let expected_difference = int_path![[0, -2], [2, -2], [2, 0], [0, 0]];
        assert_eq!(sum[0][0].len(), 4);
        assert!(sum[0][0].iter().all(|p| expected_sum.contains(p)));
        assert_eq!(difference[0][0].len(), 4);
        assert!(difference[0][0]
            .iter()
            .all(|p| expected_difference.contains(p)));
    }
}
