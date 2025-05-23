use crate::float::triangulation::Triangulation;
use crate::int::triangulation::{IndexType, IntTriangulation};
use crate::int::triangulator::IntTriangulator;
use crate::int::validation::Validation;
use i_overlay::core::solver::Solver;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_shape::flat::buffer::FlatContoursBuffer;
use i_overlay::i_shape::source::resource::ShapeResource;

pub struct Triangulator<I> {
    flat_buffer: Option<FlatContoursBuffer>,
    int_buffer: Option<IntTriangulation<I>>,
    int_triangulator: IntTriangulator<I>,
}

impl<I: IndexType> Triangulator<I> {
    #[inline]
    pub fn new(max_points_count: usize, validation: Validation, solver: Solver) -> Self {
        Self {
            flat_buffer: Some(FlatContoursBuffer::with_capacity(max_points_count)),
            int_buffer: Some(IntTriangulation::with_capacity(max_points_count)),
            int_triangulator: IntTriangulator::new(max_points_count, validation, solver),
        }
    }
}

impl<I: IndexType> Default for Triangulator<I> {
    #[inline]
    fn default() -> Self {
        Self::new(64, Default::default(), Default::default())
    }
}

impl<I: IndexType> Triangulator<I> {
    #[inline]
    pub fn triangulate<R, P, T>(&mut self, resource: &R, delaunay: bool) -> Triangulation<P, I>
    where
        R: ShapeResource<P, T> + ?Sized,
        P: FloatPointCompatible<T>,
        T: FloatNumber,
    {
        let mut flat_buffer = self.flat_buffer.take().unwrap_or_default();
        let mut int_buffer = self.int_buffer.take().unwrap_or_default();

        let adapter = flat_buffer.set_with_resource(resource);

        self.int_triangulator
            .triangulate_flat_into(&mut flat_buffer, delaunay, &mut int_buffer);

        let triangulation = int_buffer.to_float(&adapter);

        self.flat_buffer = Some(flat_buffer);
        self.int_buffer = Some(int_buffer);

        triangulation
    }

    #[inline]
    pub fn triangulate_into<R, P, T>(
        &mut self,
        resource: &R,
        delaunay: bool,
        triangulation: &mut Triangulation<P, I>,
    ) where
        R: ShapeResource<P, T> + ?Sized,
        P: FloatPointCompatible<T>,
        T: FloatNumber,
    {
        let mut flat_buffer = self.flat_buffer.take().unwrap_or_default();
        let mut int_buffer = self.int_buffer.take().unwrap_or_default();
        let adapter = flat_buffer.set_with_resource(resource);

        self.int_triangulator
            .triangulate_flat_into(&mut flat_buffer, delaunay, &mut int_buffer);

        triangulation.set_with_int(&int_buffer, &adapter);

        self.flat_buffer = Some(flat_buffer);
        self.int_buffer = Some(int_buffer);
    }

    #[inline]
    pub fn uncheck_triangulate<R, P, T>(
        &mut self,
        resource: &R,
        delaunay: bool,
    ) -> Triangulation<P, I>
    where
        R: ShapeResource<P, T> + ?Sized,
        P: FloatPointCompatible<T>,
        T: FloatNumber,
    {
        let mut flat_buffer = self.flat_buffer.take().unwrap_or_default();
        let mut int_buffer = self.int_buffer.take().unwrap_or_default();
        let adapter = flat_buffer.set_with_resource(resource);

        self.int_triangulator
            .triangulate_flat_into(&mut flat_buffer, delaunay, &mut int_buffer);

        let triangulation = int_buffer.to_float(&adapter);

        self.flat_buffer = Some(flat_buffer);
        self.int_buffer = Some(int_buffer);

        triangulation
    }

    #[inline]
    pub fn uncheck_triangulate_into<R, P, T>(
        &mut self,
        resource: &R,
        delaunay: bool,
        triangulation: &mut Triangulation<P, I>,
    ) where
        R: ShapeResource<P, T> + ?Sized,
        P: FloatPointCompatible<T>,
        T: FloatNumber,
    {
        let mut flat_buffer = self.flat_buffer.take().unwrap_or_default();
        let mut int_buffer = self.int_buffer.take().unwrap_or_default();
        let adapter = flat_buffer.set_with_resource(resource);

        self.int_triangulator
            .triangulate_flat_into(&mut flat_buffer, delaunay, &mut int_buffer);

        triangulation.set_with_int(&int_buffer, &adapter);

        self.flat_buffer = Some(flat_buffer);
        self.int_buffer = Some(int_buffer);
    }
}
