use crate::geom::point::IndexPoint;

#[derive(Debug, Clone)]
pub struct Abc {
    pub v0: ABCVertex,
    pub v1: ABCVertex,
    pub v2: ABCVertex,
}

#[derive(Debug, Clone, Copy)]
pub struct ABCVertex {
    pub vertex: IndexPoint,
    pub position: usize,
    pub neighbor: usize,
}

#[derive(Debug, Clone)]
pub struct IntTriangle {
    pub vertices: [IndexPoint; 3],
    pub neighbors: [usize; 3],
}

impl IntTriangle {
    #[inline]
    pub fn abc(a: IndexPoint, b: IndexPoint, c: IndexPoint) -> Self {
        Self {
            vertices: [a, b, c],
            neighbors: [usize::MAX; 3],
        }
    }

    #[inline]
    pub fn set_neighbor(&mut self, order: usize, neighbor: usize) {
        debug_assert!(order < 3);
        unsafe {
            *self.neighbors.get_unchecked_mut(order) = neighbor;
        }
    }

    #[inline]
    pub fn remove_neighbor(&mut self, order: usize) {
        debug_assert!(order < 3);
        unsafe {
            *self.neighbors.get_unchecked_mut(order) = usize::MAX;
        }
    }

    #[inline]
    pub fn other_vertex(&self, a: usize, b: usize) -> usize {
        if self.vertices[0].index != a && self.vertices[0].index != b {
            0
        } else if self.vertices[1].index != a && self.vertices[1].index != b {
            1
        } else {
            2
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
    
    #[inline]
    pub(crate) fn abc_by_neighbor(&self, neighbor: usize) -> Abc {
        if neighbor == self.neighbors[0] {
            self.abc_by_a()
        } else if neighbor == self.neighbors[1] {
            self.abc_by_b()
        } else {
            self.abc_by_c()
        }
    }

    #[inline]
    pub(crate) fn abc_by_a(&self) -> Abc {
        let a = ABCVertex {
            vertex: self.vertices[0],
            position: 0,
            neighbor: self.neighbors[0],
        };
        let b = ABCVertex {
            vertex: self.vertices[1],
            position: 1,
            neighbor: self.neighbors[1],
        };
        let c = ABCVertex {
            vertex: self.vertices[2],
            position: 2,
            neighbor: self.neighbors[2],
        };
        Abc { v0: a, v1: b, v2: c }
    }

    #[inline]
    pub(crate) fn abc_by_b(&self) -> Abc {
        let a = ABCVertex {
            vertex: self.vertices[1],
            position: 1,
            neighbor: self.neighbors[1],
        };
        let b = ABCVertex {
            vertex: self.vertices[2],
            position: 2,
            neighbor: self.neighbors[2],
        };
        let c = ABCVertex {
            vertex: self.vertices[0],
            position: 0,
            neighbor: self.neighbors[0],
        };
        Abc { v0: a, v1: b, v2: c }
    }

    #[inline]
    pub(crate) fn abc_by_c(&self) -> Abc {
        let a = ABCVertex {
            vertex: self.vertices[2],
            position: 2,
            neighbor: self.neighbors[2],
        };
        let b = ABCVertex {
            vertex: self.vertices[0],
            position: 0,
            neighbor: self.neighbors[0],
        };
        let c = ABCVertex {
            vertex: self.vertices[1],
            position: 1,
            neighbor: self.neighbors[1],
        };
        Abc { v0: a, v1: b, v2: c }
    }
}
