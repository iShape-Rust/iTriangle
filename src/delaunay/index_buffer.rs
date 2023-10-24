use crate::index::{Index, NIL_INDEX};

pub(super) struct IndexBuffer {
    array: Vec<Link>,
    first: usize,
}

#[derive(Debug, Clone, Copy)]
struct Link {
    empty: bool,
    next: usize,
}

impl Link {
    fn empty() -> Self {
        Self { empty: true, next: NIL_INDEX }
    }
}

impl IndexBuffer {
    fn new(count: usize) -> Self {
        if count == 0 {
            return Self {
                array: vec![],
                first: NIL_INDEX,
            };
        }
        let mut array = vec![Link::empty(); count];
        for i in 0..count {
            array[i] = Link {
                empty: false,
                next: i + 1,
            };
        }
        array[count - 1].next = NIL_INDEX;
        Self { array, first: 0 }
    }

    fn has_next(&self) -> bool {
        self.first.is_not_nil()
    }

    fn next(&mut self) -> usize {
        let index = self.first;
        self.first = self.array[index].next;
        self.array[index] = Link::empty();
        index
    }

    pub(super) fn add(&mut self, index: usize) {
        let is_overflow = index >= self.array.len();
        if is_overflow || self.array[index].empty {
            if is_overflow {
                let n = index - self.array.len();
                self.array.resize(self.array.len() + n + 1, Link::empty());
            }
            self.array[index] = Link {
                empty: false,
                next: self.first
            };
            self.first = index;
        }
    }
}
