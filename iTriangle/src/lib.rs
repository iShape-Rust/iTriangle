#![no_std]
extern crate alloc;

#[cfg(test)]
extern crate std;

#[cfg(test)]
pub(crate) mod test_util {
    const QUICK_RANDOM_DIVISOR: usize = 20;
    const FULL_RANDOM_TESTS_ENV: &str = "ITRIANGLE_FULL_RANDOM_TESTS";

    pub(crate) fn random_cases(full_count: usize) -> usize {
        if std::env::var_os(FULL_RANDOM_TESTS_ENV).is_some() {
            full_count
        } else {
            full_count.div_ceil(QUICK_RANDOM_DIVISOR)
        }
    }
}

pub mod advanced;
pub mod float;
pub mod geom;
mod index;
pub mod int;
pub mod location;
pub mod tessellation;

pub use i_overlay;
