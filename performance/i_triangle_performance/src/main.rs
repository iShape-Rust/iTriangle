use crate::test::test_0_star::SimpleStarTest;
use crate::test::test_1_star_with_hole::StarWithHoleTest;
use crate::util::args::EnvArgs;

mod test;
mod util;

fn main() {
    let args = EnvArgs::new();
    #[cfg(debug_assertions)]
    {
        debug_run(&args);
    }

    #[cfg(not(debug_assertions))]
    {
        release_run(&args);
    }
}

#[cfg(not(debug_assertions))]
fn release_run(args: &EnvArgs) {
    let test = args.get_usize("test");
    match test {
        0 => star(&args),
        1 => star_with_hole(&args),
        _ => {
            panic!("No test found")
        }
    }
}

#[cfg(debug_assertions)]
fn debug_run(_args: &EnvArgs) {
    SimpleStarTest::run_raw(4);
}

fn star(args: &EnvArgs) {
    let complex = args.get_bool("complex");

    if complex {
        println!("unchecked: ");
        let mut s0 = 0;
        for i in 0..8 {
            let count = 4 << i;
            s0 += SimpleStarTest::run_unchecked(count);
        }
        println!();

        println!("raw: ");
        let mut s1 = 0;
        for i in 0..8 {
            let count = 4 << i;
            s1 += SimpleStarTest::run_raw(count);
        }
        println!();

        println!("delaunay: ");
        let mut s2 = 0;
        for i in 0..8 {
            let count = 4 << i;
            s2 += SimpleStarTest::run_delaunay(count);
        }
        println!();

        println!("s0: {}, s1: {}, s2: {}", s0, s1, s2);
    } else {
        let count = args.get_usize("count");

        println!("unchecked: ");
        let s0 = SimpleStarTest::run_unchecked(count);
        println!();

        println!("raw: ");
        let s1 = SimpleStarTest::run_raw(count);
        println!();

        println!("delaunay: ");
        let s2 = SimpleStarTest::run_delaunay(count);
        println!();

        println!("s0: {}, s1: {}, s2: {}", s0, s1, s2);
    }
}

fn star_with_hole(args: &EnvArgs) {
    let complex = args.get_bool("complex");

    if complex {
        println!("unchecked: ");
        let mut s0 = 0;
        for i in 0..7 {
            let count = 4 << i;
            s0 += StarWithHoleTest::run_unchecked(count);
        }
        println!();

        println!("raw: ");
        let mut s1 = 0;
        for i in 0..7 {
            let count = 4 << i;
            s1 += StarWithHoleTest::run_raw(count);
        }
        println!();

        println!("delaunay: ");
        let mut s2 = 0;
        for i in 0..7 {
            let count = 4 << i;
            s2 += StarWithHoleTest::run_delaunay(count);
        }
        println!();

        println!("s0: {}, s1: {}, s2: {}", s0, s1, s2);
    } else {
        let count = args.get_usize("count");

        println!("unchecked: ");
        let s0 = StarWithHoleTest::run_unchecked(count);
        println!();

        println!("raw: ");
        let s1 = StarWithHoleTest::run_raw(count);
        println!();

        println!("delaunay: ");
        let s2 = StarWithHoleTest::run_delaunay(count);
        println!();

        println!("s0: {}, s1: {}, s2: {}", s0, s1, s2);
    }
}