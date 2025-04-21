use crate::float::triangulation::Triangulation;

pub struct TriangulationBuilder<P> {
    points: Vec<P>,
    indices: Vec<usize>,
}

impl<P> TriangulationBuilder<P> {
    /// Appends another `Triangulation` to the builder.
    ///
    /// This method correctly offsets the indices of the appended triangulation
    /// based on the current number of points in the builder.
    pub fn append(&mut self, triangulation: Triangulation<P>) -> &mut Self {
        let offset = self.points.len();
        self.points.extend(triangulation.points);
        self.indices
            .extend(triangulation.indices.iter().map(|&i| i + offset));
        self
    }

    /// Builds and returns the final `Triangulation`.
    pub fn build(self) -> Triangulation<P> {
        Triangulation {
            points: self.points,
            indices: self.indices,
        }
    }
}

impl<P> Default for TriangulationBuilder<P> {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            indices: Vec::new(),
        }
    }
}
