/*
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
                array: Vec::new(),
                first: NIL_INDEX,
            };
        }
        let mut array = Vec::with_capacity(count);
        for i in 1..count {
            array.push(Link {
                empty: false,
                next: i,
            });
        }
        array.push(Link {
            empty: false,
            next: NIL_INDEX,
        });
        Self { array, first: 0 }
    }

    fn has_next(&self) -> bool {
        self.first.is_not_nil()
    }

    fn next(&mut self) -> usize {
        let index = self.first;
        unsafe {
            let pnt = self.array.get_unchecked_mut(index);
            self.first = pnt.next;
            *pnt = Link::empty();
        }
        index
    }

    pub(super) fn add(&mut self, index: usize) {
        if index >= self.array.len() {
            self.array.resize(index + 1, Link::empty());
        }

        unsafe {
            let pnt = self.array.get_unchecked_mut(index);
            if pnt.empty {
                *pnt = Link {
                    empty: false,
                    next: self.first,
                };
                self.first = index;
            }
        }
    }
}
*/