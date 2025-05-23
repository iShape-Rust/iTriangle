use crate::test::test_0_star::SimpleStarTest;
use crate::test::test_1_star_with_hole::StarWithHoleTest;
use crate::test::test_2_rect_star_holes::RectStarHolesTest;
use crate::util::args::EnvArgs;

mod test;
mod util;

fn main() {
    #[cfg(debug_assertions)]
    {
        debug_run();
    }

    #[cfg(not(debug_assertions))]
    {

        release_run();
    }
}

#[cfg(not(debug_assertions))]
fn release_run() {
    let args = EnvArgs::new();
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
fn debug_run() {
    // let mut args = EnvArgs::new();
    // args.set_bool("complex", false);
    // args.set_usize("count", 4);
    // args.set_bool("complex", true);
    // star(&args);
    let test = SimpleStarTest {
        radius: 100.0,
        angle_steps_count: 100,
        points_per_corner: 10,
        radius_steps_count: 100,
        min_radius_scale: 0.0,
        max_radius_scale: 1.0,
    };
    let s0 = test.run_triangulator(8, 8, false);
    println!("s0: {}", s0);
}

#[allow(dead_code)]
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
        let n = 7;

        println!("unchecked raw: ");
        let mut s0 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s0 += test.run_unchecked_raw(count, repeat_count);
        }
        println!();

        println!("unchecked delaunay: ");
        let mut s1 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s1 += test.run_unchecked_delaunay(count, repeat_count);
        }
        println!();
        
        println!("raw: ");
        let mut s2 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s2 += test.run_raw(count, repeat_count);
        }
        println!();

        println!("delaunay: ");
        let mut s3 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s3 += test.run_delaunay(count, repeat_count);
        }
        println!();

        println!("triangulator raw: ");
        let mut s4 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s4 += test.run_triangulator(count, repeat_count, false);
        }
        println!();

        println!("triangulator delaunay: ");
        let mut s5 = 0;
        for i in 0..8 {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s5 += test.run_triangulator(count, repeat_count, true);
        }
        println!();

        println!("earcutr: ");
        let mut s6 = 0;
        for i in 0..n {
            let count = 4 << i;
            s6 += test.run_earcutr(count);
        }
        println!();

        println!("s0: {}, s1: {}, s2: {}, s3: {}, s4: {}, s5: {}, s6: {}", s0, s1, s2, s3, s4, s5, s6);
    } else {
        let count = args.get_usize("count");
        let repeat_count = (256 / count).max(1);

        println!("unchecked raw: ");
        let s0 = test.run_unchecked_raw(count, repeat_count);
        println!();

        println!("unchecked delaunay: ");
        let s1 = test.run_unchecked_delaunay(count, repeat_count);
        println!();
        
        println!("raw: ");
        let s2 = test.run_raw(count, repeat_count);
        println!();

        println!("delaunay: ");
        let s3 = test.run_delaunay(count, repeat_count);
        println!();

        println!("triangulator raw: ");
        let s4 = test.run_triangulator(count, repeat_count, false);
        println!();

        println!("triangulator delaunay: ");
        let s5 = test.run_triangulator(count, repeat_count, true);
        println!();

        println!("earcutr: ");
        let s6 = test.run_earcutr(count);
        println!();

        println!("s0: {}, s1: {}, s2: {}, s3: {}, s4: {}, s5: {}, s6: {}", s0, s1, s2, s3, s4, s5, s6);
    }
}

#[allow(dead_code)]
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
        let n = 6;

        println!("unchecked raw: ");
        let mut s0 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s0 += test.run_unchecked_raw(count, repeat_count);
        }
        println!();

        println!("unchecked delaunay: ");
        let mut s1 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s1 += test.run_unchecked_delaunay(count, repeat_count);
        }
        println!();

        println!("raw: ");
        let mut s2 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s2 += test.run_raw(count, repeat_count);
        }
        println!();

        println!("delaunay: ");
        let mut s3 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s3 += test.run_delaunay(count, repeat_count);
        }
        println!();

        println!("triangulator raw: ");
        let mut s4 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s4 += test.run_triangulator(count, repeat_count, false);
        }
        println!();

        println!("triangulator delaunay: ");
        let mut s5 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s5 += test.run_triangulator(count, repeat_count, true);
        }
        println!();

        println!("earcutr: ");
        let mut s6 = 0;
        for i in 0..n {
            let count = 4 << i;
            s6 += test.run_earcutr(count);
        }
        println!();

        println!("s0: {}, s1: {}, s2: {}, s3: {}, s4: {}, s5: {}, s6: {}", s0, s1, s2, s3, s4, s5, s6);
    } else {
        let count = args.get_usize("count");
        let repeat_count = (256 / count).max(1);
        
        println!("unchecked raw: ");
        let s0 = test.run_unchecked_raw(count, repeat_count);
        println!();

        println!("unchecked delaunay: ");
        let s1 = test.run_unchecked_delaunay(count, repeat_count);
        println!();

        println!("raw: ");
        let s2 = test.run_raw(count, repeat_count);
        println!();

        println!("delaunay: ");
        let s3 = test.run_delaunay(count, repeat_count);
        println!();

        println!("triangulator raw: ");
        let s4 = test.run_triangulator(count, repeat_count, false);
        println!();

        println!("triangulator delaunay: ");
        let s5 = test.run_triangulator(count, repeat_count, true);
        println!();

        println!("earcutr: ");
        let s6 = test.run_earcutr(count);
        println!();

        println!("s0: {}, s1: {}, s2: {}, s3: {}, s4: {}, s5: {}, s6: {}", s0, s1, s2, s3, s4, s5, s6);
    }
}

#[allow(dead_code)]
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
        let n = 6;
        println!("unchecked raw: ");
        let mut s0 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s0 += test.run_unchecked_raw(count, repeat_count);
        }
        println!();

        println!("unchecked delaunay: ");
        let mut s1 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s1 += test.run_unchecked_delaunay(count, repeat_count);
        }
        println!();

        println!("raw: ");
        let mut s2 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s2 += test.run_raw(count, repeat_count);
        }
        println!();

        println!("delaunay: ");
        let mut s3 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s3 += test.run_delaunay(count, repeat_count);
        }
        println!();

        println!("triangulator raw: ");
        let mut s4 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s4 += test.run_triangulator(count, repeat_count, false);
        }
        println!();

        println!("triangulator delaunay: ");
        let mut s5 = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s5 += test.run_triangulator(count, repeat_count, true);
        }
        println!();

        println!("earcutr: ");
        let mut s6 = 0;
        for i in 0..n {
            let count = 4 << i;
            s6 += test.run_earcutr(count);
        }
        println!();

        println!("s0: {}, s1: {}, s2: {}, s3: {}, s4: {}, s5: {}, s6: {}", s0, s1, s2, s3, s4, s5, s6);
    } else {
        let count = args.get_usize("count");
        let repeat_count = (256 / count).max(1);
        
        println!("unchecked raw: ");
        let s0 = test.run_unchecked_raw(count, repeat_count);
        println!();

        println!("unchecked raw: ");
        let s1 = test.run_unchecked_delaunay(count, repeat_count);
        println!();

        println!("raw: ");
        let s2 = test.run_raw(count, repeat_count);
        println!();

        println!("delaunay: ");
        let s3 = test.run_delaunay(count, repeat_count);
        println!();

        println!("triangulator raw: ");
        let s4 = test.run_triangulator(count, repeat_count, false);
        println!();

        println!("triangulator delaunay: ");
        let s5 = test.run_triangulator(count, repeat_count, true);
        println!();

        println!("earcutr: ");
        let s6 = test.run_earcutr(count);
        println!();

        println!("s0: {}, s1: {}, s2: {}, s3: {}, s4: {}, s5: {}, s6: {}", s0, s1, s2, s3, s4, s5, s6);
    }
}