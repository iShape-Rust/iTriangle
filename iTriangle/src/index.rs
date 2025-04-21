pub(crate) const NIL_INDEX: usize = usize::MAX;

pub(crate) trait Index {
    fn is_not_nil(&self) -> bool;
}

impl Index for usize {
    fn is_not_nil(&self) -> bool {
        *self != NIL_INDEX
    }
}
