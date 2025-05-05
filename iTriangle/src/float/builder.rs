use crate::float::triangulation::Triangulation;
use crate::int::triangulation::IndexType;

pub struct TriangulationBuilder<P, I> {
    points: Vec<P>,
    indices: Vec<I>,
}

impl<P, I: IndexType> TriangulationBuilder<P, I> {
    /// Appends another `Triangulation` to the builder.
    ///
    /// This method correctly offsets the indices of the appended triangulation
    /// based on the current number of points in the builder.
    pub fn append(&mut self, triangulation: Triangulation<P, I>) -> &mut Self {
        let points_count = self.points.len() + triangulation.points.len();
        if points_count > I::MAX {
            panic!(
                "Index type `{}` cannot hold {} points",
                std::any::type_name::<I>(),
                points_count
            );
        }
        
        let offset = I::try_from(self.points.len()).unwrap_or(I::ZERO);
        self.points.extend(triangulation.points);
        self.indices
            .extend(triangulation.indices.iter().map(|&i|i.add(offset)));
        self
    }

    /// Builds and returns the final `Triangulation`.
    pub fn build(self) -> Triangulation<P, I> {
        Triangulation {
            points: self.points,
            indices: self.indices,
        }
    }
}

impl<P, I> Default for TriangulationBuilder<P, I> {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            indices: Vec::new(),
        }
    }
}
