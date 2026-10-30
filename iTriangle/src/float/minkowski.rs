//! Minkowski sums and differences on a shared, fixed integer grid.
//!
//! The sum is `A + B = {a + b | a in A, b in B}`. Here, **difference** means
//! `A + (-B) = {a - b | a in A, b in B}`. It is neither polygon subtraction
//! nor morphological erosion, and does not undo a sum.
//!
//! Use [`minkowski_sum`] or [`minkowski_difference`] for a single operation.
//! They choose a domain containing both sources and their result and use `i32`
//! coordinates. For repeated operations, choose a domain once and prepare each
//! source with [`MinkowskiSolver::prepare`]. Preparation applies the selected
//! [`FillRule`], resolves intersections, triangulates, and decomposes the filled
//! region into convex parts. Operations reuse these parts without converting
//! or triangulating the sources again.
//!
//! # Fixed domain and reusable operands
//!
//! The domain must contain the original source bounds and the bounds of every
//! requested result. Bounds checks use the original sources, including points
//! in contours later removed by the fill rule. An operand belongs to the solver
//! that prepared it; another solver rejects it even with an identical domain.
//! Moving or clearing a solver does not invalidate its operands.
//!
//! ```
//! use i_triangle::float::minkowski::{MinkowskiError, MinkowskiSolver};
//! use i_triangle::i_overlay::core::fill_rule::FillRule;
//! use i_triangle::i_overlay::i_float::float::rect::FloatRect;
//!
//! let domain = FloatRect::new(-16.0, 16.0, -16.0, 16.0).unwrap();
//! let mut solver = MinkowskiSolver::<[f64; 2]>::new(domain);
//! let source = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
//! let kernel = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
//! let a = solver.prepare(&source, FillRule::NonZero)?.unwrap();
//! let b = solver.prepare(&kernel, FillRule::NonZero)?.unwrap();
//! let sum = solver.sum(&a, &b)?;
//! solver.clear();
//! let difference = solver.difference(&a, &b)?;
//! assert_eq!(sum[0][0].len(), 4);
//! assert!(sum[0][0].contains(&[5.0, 5.0]));
//! assert!(difference[0][0].contains(&[-1.0, -1.0]));
//! # Ok::<(), MinkowskiError>(())
//! ```
//!
//! # Empty regions and errors
//!
//! Only filled, two-dimensional regions are represented. Empty input, an
//! unfilled source, or a region collapsed by quantization produces `Ok(None)`
//! during preparation. Points and segments are not operands. The one-shot
//! functions return an empty vector when either prepared region is empty.
//! Invalid coordinates are still validated; an empty source does not exempt
//! the other source from coordinate validation.
//!
//! ```
//! use i_triangle::float::minkowski::{MinkowskiError, MinkowskiSolver};
//! use i_triangle::i_overlay::core::fill_rule::FillRule;
//! use i_triangle::i_overlay::i_float::float::rect::FloatRect;
//!
//! let domain = FloatRect::new(-2.0, 2.0, -2.0, 2.0).unwrap();
//! let mut solver = MinkowskiSolver::<[f64; 2]>::new(domain);
//! let empty: Vec<[f64; 2]> = Vec::new();
//! assert!(solver.prepare(&empty, FillRule::NonZero)?.is_none());
//! let square = [[1.0, 1.0], [2.0, 1.0], [2.0, 2.0], [1.0, 2.0]];
//! let a = solver.prepare(&square, FillRule::NonZero)?.unwrap();
//! assert!(matches!(solver.sum(&a, &a), Err(MinkowskiError::OutsideDomain)));
//! let mut other = MinkowskiSolver::<[f64; 2]>::new(domain);
//! let b = other.prepare(&square, FillRule::NonZero)?.unwrap();
//! assert!(matches!(solver.sum(&a, &b), Err(MinkowskiError::ForeignOperand)));
//! let invalid = [[f64::NAN, 0.0], [1.0, 0.0], [0.0, 1.0]];
//! assert!(matches!(solver.prepare(&invalid, FillRule::NonZero),
//!     Err(MinkowskiError::CoordinateOutOfRange)));
//! # Ok::<(), MinkowskiError>(())
//! ```
//!
//! # Precision and collinear vertices
//!
//! All operands share the solver's power-of-two scale and origin. A larger
//! domain generally means a coarser grid; a larger integer type can retain more
//! detail, subject to the precision of the floating-point scalar. The grid step
//! is [`FloatPointAdapter::inv_scale`], available through
//! [`MinkowskiSolver::adapter`]. Input rounding and integer intersection rounding
//! can alter small features or collapse them. Results approximate the operation
//! on the original floating-point regions; they are not exact real arithmetic.
//! Snapped output can extend slightly beyond the original domain bounds.
//!
//! [`MinkowskiSolver::with_options`] controls collinear vertices in the final
//! union. Enabling preservation retains vertices introduced by that union; it
//! does not retain every source vertex. Collinear edges of each convex pair
//! are merged before union, regardless of this setting. Output shapes contain
//! a counter-clockwise outer contour followed by clockwise hole contours.

pub use crate::int::minkowski::MinkowskiError;
use crate::int::minkowski::{IntMinkowskiOperand, IntMinkowskiSolver};
use alloc::sync::Arc;
use alloc::vec::Vec;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::integer::OverlayInt;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::rect::IntRect;
use i_overlay::i_shape::base::data::Shapes;
use i_overlay::i_shape::flat::buffer::FlatContoursBuffer;
use i_overlay::i_shape::int::shape::IntShapes;
use i_overlay::i_shape::source::float::resource::ShapeResource;

/// An owned, prepared filled region on its creating solver's fixed grid.
///
/// Create operands with [`MinkowskiSolver::prepare`]. The source need not remain
/// alive. Operands remain usable after further preparations, operations, solver
/// moves, and [`MinkowskiSolver::clear`], but only with their original solver.
pub struct MinkowskiOperand<P: FloatPointCompatible, I: OverlayInt = i32> {
    owner: Arc<()>,
    bbox: FloatRect<P::Scalar>,
    int_operand: IntMinkowskiOperand<I>,
}

impl<P: FloatPointCompatible, I: OverlayInt> MinkowskiOperand<P, I> {
    /// Borrows the prepared integer geometry in adapter coordinates.
    ///
    /// These coordinates have the adapter's origin subtracted and scale applied;
    /// they are not coordinates in the original floating-point space.
    pub fn int_operand(&self) -> &IntMinkowskiOperand<I> {
        &self.int_operand
    }

    /// Returns the original source bounds, before quantization or fill filtering.
    ///
    /// Operations use these conservative bounds for domain and range checks.
    pub fn bbox(&self) -> FloatRect<P::Scalar> {
        self.bbox
    }
}

/// Reuses prepared regions and work buffers within one fixed coordinate domain.
///
/// `P` may be a compatible `f32` or `f64` point type. `I` selects the integer
/// engine (`i16`, `i32`, or `i64`), with `i32` as the default. The adapter never
/// rescales after construction. See the [module documentation](self) for reuse,
/// ownership, empty-region, and precision contracts.
pub struct MinkowskiSolver<P: FloatPointCompatible, I: OverlayInt = i32> {
    owner: Arc<()>,
    adapter: FloatPointAdapter<P, I>,
    int_solver: IntMinkowskiSolver<I>,
    source_buffer: FlatContoursBuffer<I>,
}

impl<P: FloatPointCompatible, I: OverlayInt> MinkowskiSolver<P, I> {
    /// Creates a solver with initial capacity 64 and collinear output simplified.
    ///
    /// `rect` must contain both sources and all requested result bounds.
    ///
    /// # Panics
    /// Panics if `rect` has reversed bounds, non-finite coordinates, or absolute
    /// coordinates above `2^60` for `f32` or `2^500` for `f64`.
    pub fn new(rect: FloatRect<P::Scalar>) -> Self {
        Self::with_options(rect, 64, false)
    }

    /// Creates a solver with a fixed domain, capacity hint, and output policy.
    ///
    /// `capacity` is an initial allocation hint, not a limit. When
    /// `preserve_output_collinear` is `true`, collinear vertices from the final
    /// union are retained. Pairwise convex sums still merge collinear edges,
    /// so this option does not preserve all original source vertices.
    ///
    /// The grid reserves an extra coordinate bit for adding or subtracting two
    /// prepared operands: input magnitude is bounded by `2^(I::BITS - 4)`.
    ///
    /// # Panics
    /// Panics for invalid `rect`, as in [`Self::new`].
    pub fn with_options(
        rect: FloatRect<P::Scalar>,
        capacity: usize,
        preserve_output_collinear: bool,
    ) -> Self {
        Self {
            owner: Arc::new(()),
            adapter: FloatPointAdapter::with_coordinate_bits(rect, I::BITS - 4),
            int_solver: IntMinkowskiSolver::with_options(capacity, preserve_output_collinear),
            source_buffer: FlatContoursBuffer::with_capacity(capacity),
        }
    }

    /// Returns the fixed adapter, including its domain, origin, and grid scale.
    ///
    /// Operations restore sums with twice the adapter origin and differences
    /// with zero origin. Directly applying `int_to_float` to an integer sum or
    /// difference therefore does not in general reproduce the solver's output.
    pub fn adapter(&self) -> &FloatPointAdapter<P, I> {
        &self.adapter
    }

    /// Clears temporary geometry while retaining allocations and the fixed grid.
    /// Previously prepared operands remain valid for this solver.
    pub fn clear(&mut self) {
        self.int_solver.clear();
        self.source_buffer.clear_and_reserve(0, 0);
    }

    /// Quantizes, normalizes with `fill_rule`, and decomposes a source once.
    ///
    /// Accepts contours, shapes with holes, and collections of shapes through
    /// [`ShapeResource`]. The returned operand owns its convex parts and can be
    /// reused in any number of operations with this solver. Each source may use
    /// a different fill rule.
    ///
    /// Returns `Ok(None)` when no filled area survives normalization on the
    /// fixed grid, including empty, point, segment, and collapsed input.
    ///
    /// # Errors
    /// Returns [`MinkowskiError::CoordinateOutOfRange`] for any non-finite source
    /// coordinate or coordinate beyond the scalar's supported limit. Returns
    /// [`MinkowskiError::OutsideDomain`] if the original source bounds extend
    /// outside the solver's domain, even if those points would later be removed.
    pub fn prepare<R>(
        &mut self,
        source: &R,
        fill_rule: FillRule,
    ) -> Result<Option<MinkowskiOperand<P, I>>, MinkowskiError>
    where
        R: ShapeResource<P> + ?Sized,
    {
        self.clear();
        let Some(bbox) = source_bbox(source)? else {
            return Ok(None);
        };
        self.check_rect(bbox)?;
        let (points, paths) = source
            .iter_paths()
            .fold((0, 0), |(n, c), path| (n + path.len(), c + 1));
        self.source_buffer.clear_and_reserve(points, paths);
        for path in source.iter_paths() {
            let start = self.source_buffer.points.len();
            self.source_buffer
                .points
                .extend(path.iter().map(|p| self.adapter.float_to_int(p)));
            self.source_buffer
                .ranges
                .push(start..self.source_buffer.points.len());
        }
        Ok(self
            .int_solver
            .prepare(&self.source_buffer, fill_rule)
            .map(|int_operand| MinkowskiOperand {
                owner: Arc::clone(&self.owner),
                bbox,
                int_operand,
            }))
    }

    /// Returns `lhs + rhs = {a + b | a in lhs, b in rhs}`.
    ///
    /// Reuses the prepared parts and unions all pairwise sums. No source
    /// conversion or triangulation is repeated.
    ///
    /// # Errors
    /// Returns [`MinkowskiError::ForeignOperand`] if either operand was prepared
    /// by another solver, [`MinkowskiError::OutsideDomain`] if the sum of the
    /// original source bounds exceeds the fixed domain, or
    /// [`MinkowskiError::CoordinateOutOfRange`] if numeric result bounds exceed
    /// the supported coordinate range. The solver remains reusable on error.
    pub fn sum(
        &mut self,
        lhs: &MinkowskiOperand<P, I>,
        rhs: &MinkowskiOperand<P, I>,
    ) -> Result<Shapes<P>, MinkowskiError> {
        self.clear();
        self.check_operand(lhs)?;
        self.check_operand(rhs)?;
        self.check_rect(result_bbox(lhs.bbox, rhs.bbox, false)?)?;
        let a = lhs.int_operand.bbox();
        let b = rhs.int_operand.bbox();
        let origin = self.adapter.offset();
        let offset = P::from_xy(origin.x() + origin.x(), origin.y() + origin.y());
        self.check_output_bounds(
            IntRect::new(
                a.min_x + b.min_x,
                a.max_x + b.max_x,
                a.min_y + b.min_y,
                a.max_y + b.max_y,
            ),
            offset,
        )?;
        let shapes = self.int_solver.sum(&lhs.int_operand, &rhs.int_operand)?;
        Ok(self.to_float(shapes, offset))
    }

    /// Returns `lhs + (-rhs) = {a - b | a in lhs, b in rhs}`.
    ///
    /// The right operand is reflected through the coordinate origin. This is
    /// not erosion, Boolean subtraction, or an inverse of [`Self::sum`]. It
    /// generally depends on operand order. Sources are not triangulated again.
    ///
    /// # Errors
    /// Uses the same ownership, domain, and numeric checks as [`Self::sum`],
    /// with result bounds computed by subtracting the original source bounds.
    pub fn difference(
        &mut self,
        lhs: &MinkowskiOperand<P, I>,
        rhs: &MinkowskiOperand<P, I>,
    ) -> Result<Shapes<P>, MinkowskiError> {
        self.clear();
        self.check_operand(lhs)?;
        self.check_operand(rhs)?;
        self.check_rect(result_bbox(lhs.bbox, rhs.bbox, true)?)?;
        let a = lhs.int_operand.bbox();
        let b = rhs.int_operand.bbox();
        let offset = P::from_xy(P::Scalar::ZERO, P::Scalar::ZERO);
        self.check_output_bounds(
            IntRect::new(
                a.min_x - b.max_x,
                a.max_x - b.min_x,
                a.min_y - b.max_y,
                a.max_y - b.min_y,
            ),
            offset,
        )?;
        let shapes = self
            .int_solver
            .difference(&lhs.int_operand, &rhs.int_operand)?;
        Ok(self.to_float(shapes, offset))
    }

    fn check_rect(&self, bbox: FloatRect<P::Scalar>) -> Result<(), MinkowskiError> {
        let rect = self.adapter.rect();
        if bbox.min_x < rect.min_x
            || bbox.max_x > rect.max_x
            || bbox.min_y < rect.min_y
            || bbox.max_y > rect.max_y
        {
            Err(MinkowskiError::OutsideDomain)
        } else {
            Ok(())
        }
    }

    fn check_operand(&self, operand: &MinkowskiOperand<P, I>) -> Result<(), MinkowskiError> {
        if !Arc::ptr_eq(&self.owner, &operand.owner) {
            return Err(MinkowskiError::ForeignOperand);
        }
        self.check_rect(operand.bbox)
    }

    fn check_output_bounds(&self, bbox: IntRect<I>, offset: P) -> Result<(), MinkowskiError> {
        let inv = self.adapter.inv_scale();
        FloatRect::new(
            P::Scalar::from_int(bbox.min_x) * inv + offset.x(),
            P::Scalar::from_int(bbox.max_x) * inv + offset.x(),
            P::Scalar::from_int(bbox.min_y) * inv + offset.y(),
            P::Scalar::from_int(bbox.max_y) * inv + offset.y(),
        )
        .map(|_| ())
        .map_err(|_| MinkowskiError::CoordinateOutOfRange)
    }

    fn to_float(&self, shapes: IntShapes<I>, offset: P) -> Shapes<P> {
        let inv = self.adapter.inv_scale();
        shapes
            .into_iter()
            .map(|shape| {
                shape
                    .into_iter()
                    .map(|contour| {
                        contour
                            .into_iter()
                            .map(|p| {
                                P::from_xy(
                                    P::Scalar::from_int(p.x) * inv + offset.x(),
                                    P::Scalar::from_int(p.y) * inv + offset.y(),
                                )
                            })
                            .collect()
                    })
                    .collect()
            })
            .collect()
    }
}

/// Computes a one-shot Minkowski sum using an automatically selected `i32` grid.
///
/// Both sources use `fill_rule`. The domain encloses their original bounds and
/// the sum bounds. Use [`MinkowskiSolver`] to choose a domain or integer type,
/// retain output collinear vertices, or reuse prepared operands.
///
/// Returns an empty vector if either normalized source has no filled area,
/// including geometry collapsed by the common grid.
///
/// # Errors
/// Returns [`MinkowskiError::CoordinateOutOfRange`] for invalid source
/// coordinates or nonempty result bounds beyond the supported numeric range.
///
/// # Example
/// ```
/// use i_triangle::float::minkowski::{minkowski_sum, MinkowskiError};
/// use i_triangle::i_overlay::core::fill_rule::FillRule;
///
/// let square = [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]];
/// let result = minkowski_sum(&square, &square, FillRule::NonZero)?;
/// assert!(result[0][0].contains(&[4.0, 4.0]));
/// let empty: Vec<[f64; 2]> = Vec::new();
/// assert!(minkowski_sum(&square, &empty, FillRule::NonZero)?.is_empty());
/// # Ok::<(), MinkowskiError>(())
/// ```
pub fn minkowski_sum<P, R0, R1>(
    lhs: &R0,
    rhs: &R1,
    fill_rule: FillRule,
) -> Result<Shapes<P>, MinkowskiError>
where
    P: FloatPointCompatible,
    R0: ShapeResource<P> + ?Sized,
    R1: ShapeResource<P> + ?Sized,
{
    calculate(lhs, rhs, fill_rule, false)
}

/// Computes `{a - b | a in lhs, b in rhs}` on an automatically selected `i32` grid.
///
/// This is the sum with `rhs` reflected through the origin, not erosion or
/// Boolean subtraction. It does not undo [`minkowski_sum`]. Both sources use
/// `fill_rule`; the domain encloses both original sources and their difference.
/// Empty-region behavior and numeric errors are the same as [`minkowski_sum`].
///
/// # Example
/// ```
/// use i_triangle::float::minkowski::{minkowski_difference, MinkowskiError};
/// use i_triangle::i_overlay::core::fill_rule::FillRule;
///
/// let square = [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]];
/// let result = minkowski_difference(&square, &square, FillRule::NonZero)?;
/// assert!(result[0][0].contains(&[-2.0, -2.0]));
/// assert!(result[0][0].contains(&[2.0, 2.0]));
/// # Ok::<(), MinkowskiError>(())
/// ```
pub fn minkowski_difference<P, R0, R1>(
    lhs: &R0,
    rhs: &R1,
    fill_rule: FillRule,
) -> Result<Shapes<P>, MinkowskiError>
where
    P: FloatPointCompatible,
    R0: ShapeResource<P> + ?Sized,
    R1: ShapeResource<P> + ?Sized,
{
    calculate(lhs, rhs, fill_rule, true)
}

fn source_bbox<P, R>(source: &R) -> Result<Option<FloatRect<P::Scalar>>, MinkowskiError>
where
    P: FloatPointCompatible,
    R: ShapeResource<P> + ?Sized,
{
    FloatRect::with_iter(source.iter_paths().flatten())
        .map_err(|_| MinkowskiError::CoordinateOutOfRange)
}

fn result_bbox<F: FloatNumber>(
    a: FloatRect<F>,
    b: FloatRect<F>,
    difference: bool,
) -> Result<FloatRect<F>, MinkowskiError> {
    let rect = if difference {
        FloatRect::new(
            a.min_x - b.max_x,
            a.max_x - b.min_x,
            a.min_y - b.max_y,
            a.max_y - b.min_y,
        )
    } else {
        FloatRect::new(
            a.min_x + b.min_x,
            a.max_x + b.max_x,
            a.min_y + b.min_y,
            a.max_y + b.max_y,
        )
    };
    rect.map_err(|_| MinkowskiError::CoordinateOutOfRange)
}

fn calculate<P, R0, R1>(
    lhs: &R0,
    rhs: &R1,
    fill_rule: FillRule,
    difference: bool,
) -> Result<Shapes<P>, MinkowskiError>
where
    P: FloatPointCompatible,
    R0: ShapeResource<P> + ?Sized,
    R1: ShapeResource<P> + ?Sized,
{
    let a = source_bbox(lhs)?;
    let b = source_bbox(rhs)?;
    let (Some(a), Some(b)) = (a, b) else {
        return Ok(Vec::new());
    };
    let inputs = FloatRect::with_rects(a, b).map_err(|_| MinkowskiError::CoordinateOutOfRange)?;
    let rect = match result_bbox(a, b, difference) {
        Ok(output) => FloatRect::with_rects(inputs, output)
            .map_err(|_| MinkowskiError::CoordinateOutOfRange)?,
        Err(_) => inputs,
    };
    let mut solver = MinkowskiSolver::<P>::new(rect);
    let a = solver.prepare(lhs, fill_rule)?;
    let b = solver.prepare(rhs, fill_rule)?;
    let (Some(a), Some(b)) = (a, b) else {
        return Ok(Vec::new());
    };
    if difference {
        solver.difference(&a, &b)
    } else {
        solver.sum(&a, &b)
    }
}
