use crate::test::test_0_star::SimpleStarTest;
use crate::test::test_1_star_with_hole::StarWithHoleTest;
use crate::test::test_2_rect_star_holes::RectStarHolesTest;
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
        2 => rect_with_star_holes(&args),
        _ => {
            panic!("No test found")
        }
    }
}

#[cfg(debug_assertions)]
fn debug_run(_args: &EnvArgs) {
    SimpleStarTest {
        radius: 100.0,
        angle_steps_count: 100,
        points_per_corner: 10,
        radius_steps_count: 100,
        min_radius_scale: 0.0,
        max_radius_scale: 1.0,
    }.run_raw(4);
}

fn star(args: &EnvArgs) {
    let test = SimpleStarTest {
        radius: 100.0,
        angle_steps_count: 100,
        points_per_corner: 10,
        radius_steps_count: 100,
        min_radius_scale: 0.0,
        max_radius_scale: 1.0,
    };

    let complex = args.get_bool("complex");

    if complex {
        println!("unchecked: ");
        let mut s0 = 0;
        for i in 0..8 {
            let count = 4 << i;
            s0 += test.run_unchecked(count);
        }
        println!();

        println!("raw: ");
        let mut s1 = 0;
        for i in 0..8 {
            let count = 4 << i;
            s1 += test.run_raw(count);
        }
        println!();

        println!("delaunay: ");
        let mut s2 = 0;
        for i in 0..8 {
            let count = 4 << i;
            s2 += test.run_delaunay(count);
        }
        println!();

        println!("s0: {}, s1: {}, s2: {}", s0, s1, s2);

        println!("earcutr: ");
        let mut s3 = 0;
        for i in 0..8 {
            let count = 4 << i;
            s3 += test.run_earcutr(count);
        }
        println!();

        println!("s0: {}, s1: {}, s2: {}, s3: {}", s0, s1, s2, s3);
    } else {
        let count = args.get_usize("count");

        println!("unchecked: ");
        let s0 = test.run_unchecked(count);
        println!();

        println!("raw: ");
        let s1 = test.run_raw(count);
        println!();

        println!("delaunay: ");
        let s2 = test.run_delaunay(count);
        println!();

        println!("s0: {}, s1: {}, s2: {}", s0, s1, s2);
    }
}

fn star_with_hole(args: &EnvArgs) {
    let complex = args.get_bool("complex");

    let test = StarWithHoleTest {
        radius: 100.0,
        angle_steps_count: 100,
        points_per_corner: 10,
        radius_steps_count: 100,
        min_radius_scale: 0.1, // must be > 0 to prevent intersection!
        max_radius_scale: 1.0,
    };

    if complex {
        println!("unchecked: ");
        let mut s0 = 0;
        for i in 0..7 {
            let count = 4 << i;
            s0 += test.run_unchecked(count);
        }
        println!();

        println!("raw: ");
        let mut s1 = 0;
        for i in 0..7 {
            let count = 4 << i;
            s1 += test.run_raw(count);
        }
        println!();

        println!("delaunay: ");
        let mut s2 = 0;
        for i in 0..7 {
            let count = 4 << i;
            s2 += test.run_delaunay(count);
        }
        println!();

        println!("earcutr: ");
        let mut s3 = 0;
        for i in 0..7 {
            let count = 4 << i;
            s3 += test.run_earcutr(count);
        }
        println!();

        println!("s0: {}, s1: {}, s2: {}, s3: {}", s0, s1, s2, s3);
    } else {
        let count = args.get_usize("count");

        println!("unchecked: ");
        let s0 = test.run_unchecked(count);
        println!();

        println!("raw: ");
        let s1 = test.run_raw(count);
        println!();

        println!("delaunay: ");
        let s2 = test.run_delaunay(count);
        println!();

        println!("earcutr: ");
        let s3 = test.run_earcutr(count);
        println!();

        println!("s0: {}, s1: {}, s2: {}, s3: {}", s0, s1, s2, s3);
    }
}


fn rect_with_star_holes(args: &EnvArgs) {
    let complex = args.get_bool("complex");

    let test = RectStarHolesTest{
        radius: 100.0,
        angle_steps_count: 5,
        points_per_corner: 10,
        radius_steps_count: 5,
        min_radius_scale: 0.0,
        max_radius_scale: 1.0,
        corners_count: 5,
    };

    if complex {
        println!("unchecked: ");
        let mut s0 = 0;
        for i in 0..7 {
            let count = 4 << i;
            s0 += test.run_unchecked(count);
        }
        println!();

        println!("raw: ");
        let mut s1 = 0;
        for i in 0..7 {
            let count = 4 << i;
            s1 += test.run_raw(count);
        }
        println!();

        println!("delaunay: ");
        let mut s2 = 0;
        for i in 0..7 {
            let count = 4 << i;
            s2 += test.run_delaunay(count);
        }
        println!();

        println!("earcutr: ");
        let mut s3 = 0;
        for i in 0..7 {
            let count = 4 << i;
            s3 += test.run_earcutr(count);
        }
        println!();

        println!("s0: {}, s1: {}, s2: {}, s3: {}", s0, s1, s2, s3);
    } else {
        let count = args.get_usize("count");

        println!("unchecked: ");
        let s0 = test.run_unchecked(count);
        println!();

        println!("raw: ");
        let s1 = test.run_raw(count);
        println!();

        println!("delaunay: ");
        let s2 = test.run_delaunay(count);
        println!();

        println!("earcutr: ");
        let s3 = test.run_earcutr(count);
        println!();

        println!("s0: {}, s1: {}, s2: {}, s3: {}", s0, s1, s2, s3);
    }
}