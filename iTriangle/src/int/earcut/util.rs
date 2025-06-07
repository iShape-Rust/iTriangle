use core::cmp::Ordering;
use i_overlay::i_float::fix_vec::FixVec;
use i_overlay::i_float::int::point::IntPoint;

#[derive(PartialEq, Eq)]
pub(super) enum ABCExcludeResult {
    Inside,
    Outside,
    OutsideEdge,
}

// a, b, c - counter clock wised points
pub(super) struct ABC {
    a: IntPoint,
    b: IntPoint,
    c: IntPoint,
    ab: FixVec,
    bc: FixVec,
    ca: FixVec,
}

impl ABC {
    #[inline(always)]
    pub(super) fn new(a: IntPoint, b: IntPoint, c: IntPoint) -> Self {
        let ab = b.subtract(a);
        let bc = c.subtract(b);
        let ca = a.subtract(c);
        Self {
            a,
            b,
            c,
            ab,
            bc,
            ca,
        }
    }

    #[inline(always)]
    pub(super) fn contains(&self, p: IntPoint) -> bool {
        let ap = p.subtract(self.a);
        let a_cross = ap.cross_product(self.ab);
        if a_cross >= 0 {
            return false;
        }

        let bp = p.subtract(self.b);
        let b_cross = bp.cross_product(self.bc);
        if b_cross >= 0 {
            return false;
        }

        let cp = p.subtract(self.c);
        let c_cross = cp.cross_product(self.ca);

        c_cross < 0
    }

    #[inline(always)]
    pub(super) fn contains_exclude_ca(&self, p: IntPoint) -> ABCExcludeResult {
        let ap = p.subtract(self.a);
        let a_cross = ap.cross_product(self.ab);
        if a_cross >= 0 {
            return ABCExcludeResult::Outside;
        }

        let bp = p.subtract(self.b);
        let b_cross = bp.cross_product(self.bc);
        if b_cross >= 0 {
            return ABCExcludeResult::Outside;
        }

        let cp = p.subtract(self.c);
        let c_cross = cp.cross_product(self.ca);

        match c_cross.cmp(&0) {
            Ordering::Less => ABCExcludeResult::Inside,
            Ordering::Equal => {
                if AB::contains(self.a, self.c, p) {
                    ABCExcludeResult::Inside
                } else {
                    ABCExcludeResult::OutsideEdge
                }
            }
            Ordering::Greater => ABCExcludeResult::OutsideEdge,
        }
    }
}

pub(super) struct AB;

impl AB {
    #[inline(always)]
    pub(super) fn contains(a: IntPoint, b: IntPoint, p: IntPoint) -> bool {
        // a, b, p already on one line
        // not including ends
        let ap = a.subtract(p);
        let bp = b.subtract(p);

        // must have opposite direction
        ap.dot_product(bp) < 0
    }
}
