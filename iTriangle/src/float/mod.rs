//! Floating-point triangulation uses the same coordinate contract as iOverlay.
//!
//! Coordinates must be finite, with absolute values at most `2^60` for `f32`
//! or `2^500` for `f64`. Invalid input bounds panic; empty geometry produces
//! an empty mesh. Automatic conversion uses the conservative adapter constructors,
//! reserving `I::BITS - 3` coordinate bits for integer differences, products, and
//! rounding, including unchecked APIs.
//! Unchecked APIs skip topology validation. Reusable `Triangulator` methods
//! validate individual points only in debug builds and final bounds in all builds;
//! callers must ensure every input point satisfies the coordinate contract.

pub mod builder;
pub mod centroid_net;
pub mod circumcenter;
pub mod convex;
pub mod custom;
pub mod delaunay;
pub mod locator;
pub mod relax;
pub mod triangulatable;
pub mod triangulation;
pub mod triangulator;
pub mod unchecked;
pub mod uniform;
