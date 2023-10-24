use crate::delaunay::vertex::DVertex;

#[derive(Debug, Clone, Copy)]
pub struct DTriangle {
    pub index: usize,
    pub vertices: [DVertex; 3],
    pub neighbors: [usize; 3],
}

impl DTriangle {
    pub fn va(&self) -> DVertex {
        self.vertices[0]
    }

    pub fn vb(&self) -> DVertex {
        self.vertices[1]
    }

    pub fn vc(&self) -> DVertex {
        self.vertices[2]
    }

    pub fn na(&self) -> usize {
        self.neighbors[0]
    }

    pub fn nb(&self) -> usize {
        self.neighbors[1]
    }

    pub fn nc(&self) -> usize {
        self.neighbors[2]
    }

    pub fn new() -> Self {
        Self {
            index: usize::MAX,
            vertices: [DVertex::empty(); 3],
            neighbors: [usize::MAX; 3],
        }
    }

    pub fn abc(index: usize, a: DVertex, b: DVertex, c: DVertex) -> Self {
        Self {
            index,
            vertices: [a, b, c],
            neighbors: [usize::MAX; 3],
        }
    }

    pub fn abc_bc_ac_ab(index: usize, a: DVertex, b: DVertex, c: DVertex, bc: usize, ac: usize, ab: usize) -> Self {
        Self {
            index,
            vertices: [a, b, c],
            neighbors: [bc, ac, ab],
        }
    }

    pub fn neighbor(&self, vertex: usize) -> usize {
        #[cfg(debug_assertions)]
        {
            for i in 0..3 {
                if self.vertices[i].index == vertex {
                    return self.neighbors[i];
                }
            }

            panic!("Neighbor vertex is not present");
        }

        #[cfg(not(debug_assertions))]
        {
            for i in 0..2 {
                if self.vertices[i].index == vertex {
                    return self.neighbors[i];
                }
            }
            self.neighbors[2]
        }
    }

    pub fn opposite(&self, neighbor: usize) -> usize {
        #[cfg(debug_assertions)]
        {
            for i in 0..3 {
                if self.neighbors[i] == neighbor {
                    return i;
                }
            }

            panic!("Neighbor is not present");
        }

        #[cfg(not(debug_assertions))]
        {
            for i in 0..2 {
                if self.neighbors[i] == neighbor {
                    return i;
                }
            }

            2
        }
    }

    pub fn update_opposite(&mut self, old_neighbor: usize, new_neighbor: usize) {
        let index = self.opposite(old_neighbor);
        unsafe {
            *self.neighbors.get_unchecked_mut(index) = new_neighbor;
        }
    }
}