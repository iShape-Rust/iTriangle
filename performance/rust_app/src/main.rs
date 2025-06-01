use crate::test::test::Test;
use crate::test::test_0_star::SimpleStarTest;
use crate::test::test_1_star_with_hole::StarWithHoleTest;
use crate::test::test_2_rect_star_holes::RectStarHolesTest;
use crate::test::test_3_spiral::SpiralTest;
use crate::test::test_4_spike::SpikeTest;
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
        3 => spiral(&args),
        4 => spike(&args),
        _ => {
            panic!("No test found")
        }
    }
}

#[cfg(debug_assertions)]
fn debug_run() {
    let mut s;
    // let mut args = EnvArgs::new();
    // args.set_bool("complex", false);
    // args.set_usize("count", 4);
    // args.set_bool("complex", true);
    // star(&args);
    // let test = SimpleStarTest {
    //     radius: 100.0,
    //     angle_steps_count: 100,
    //     points_per_corner: 10,
    //     radius_steps_count: 100,
    //     min_radius_scale: 0.0,
    //     max_radius_scale: 1.0,
    // };
    let test = StarWithHoleTest {
        radius: 100.0,
        angle_steps_count: 100,
        points_per_corner: 10,
        radius_steps_count: 100,
        min_radius_scale: 0.1, // must be > 0 to prevent intersection!
        max_radius_scale: 1.0,
    };
    // s = test.run_unchecked_raw(32, 2);
    // println!("s: {}", s);
    s = test.run_unchecked_triangulator(32, 2, false);
    println!("s: {}", s);
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
        let mut s;

        println!("unchecked raw: ");
        s = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s += test.run_unchecked_raw(count, repeat_count);
        }
        println!("s: {}", s);

        println!("unchecked delaunay: ");
        s = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s += test.run_unchecked_delaunay(count, repeat_count);
        }
        println!("s: {}", s);

        // println!("raw: ");
        // s = 0;
        // for i in 0..n {
        //     let count = 4 << i;
        //     let repeat_count = (256 / count).max(1);
        //     s += test.run_raw(count, repeat_count);
        // }
        // println!("s: {}", s);
        //
        // println!("delaunay: ");
        // s = 0;
        // for i in 0..n {
        //     let count = 4 << i;
        //     let repeat_count = (256 / count).max(1);
        //     s += test.run_delaunay(count, repeat_count);
        // }
        // println!("s: {}", s);
        //
        // println!("triangulator raw: ");
        // s = 0;
        // for i in 0..n {
        //     let count = 4 << i;
        //     let repeat_count = (256 / count).max(1);
        //     s += test.run_triangulator(count, repeat_count, false);
        // }
        // println!("s: {}", s);
        //
        // println!("triangulator delaunay: ");
        // s = 0;
        // for i in 0..8 {
        //     let count = 4 << i;
        //     let repeat_count = (256 / count).max(1);
        //     s += test.run_triangulator(count, repeat_count, true);
        // }
        // println!("s: {}", s);

        println!("triangulator uncheck raw: ");
        s = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s += test.run_unchecked_triangulator(count, repeat_count, false);
        }
        println!("s: {}", s);

        println!("triangulator uncheck delaunay: ");
        s = 0;
        for i in 0..8 {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s += test.run_unchecked_triangulator(count, repeat_count, true);
        }
        println!("s: {}", s);

        println!("earcutr: ");
        s = 0;
        for i in 0..n {
            let count = 4 << i;
            s += test.run_earcutr(count);
        }
        println!("s: {}", s);
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

        println!("triangulator uncheck raw: ");
        let s6 = test.run_triangulator(count, repeat_count, false);
        println!();

        println!("triangulator uncheck delaunay: ");
        let s7 = test.run_triangulator(count, repeat_count, true);
        println!();

        println!("earcutr: ");
        let s8 = test.run_earcutr(count);
        println!();

        println!(
            "s0: {}, s1: {}, s2: {}, s3: {}, s4: {}, s5: {}, s6: {}, s7: {}, s8: {}",
            s0, s1, s2, s3, s4, s5, s6, s7, s8
        );
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
        let mut s;

        println!("unchecked raw: ");
        s = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s += test.run_unchecked_raw(count, repeat_count);
        }
        println!("s: {}", s);

        println!("unchecked delaunay: ");
        s = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s += test.run_unchecked_delaunay(count, repeat_count);
        }
        println!("s: {}", s);

        // println!("raw: ");
        // s = 0;
        // for i in 0..n {
        //     let count = 4 << i;
        //     let repeat_count = (256 / count).max(1);
        //     s += test.run_raw(count, repeat_count);
        // }
        // println!("s: {}", s);
        //
        // println!("delaunay: ");
        // s = 0;
        // for i in 0..n {
        //     let count = 4 << i;
        //     let repeat_count = (256 / count).max(1);
        //     s += test.run_delaunay(count, repeat_count);
        // }
        // println!("s: {}", s);
        //
        // println!("triangulator raw: ");
        // s = 0;
        // for i in 0..n {
        //     let count = 4 << i;
        //     let repeat_count = (256 / count).max(1);
        //     s += test.run_triangulator(count, repeat_count, false);
        // }
        // println!("s: {}", s);
        //
        // println!("triangulator delaunay: ");
        // s = 0;
        // for i in 0..n {
        //     let count = 4 << i;
        //     let repeat_count = (256 / count).max(1);
        //     s += test.run_triangulator(count, repeat_count, true);
        // }
        // println!("s: {}", s);

        println!("triangulator uncheck raw: ");
        s = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s += test.run_unchecked_triangulator(count, repeat_count, false);
        }
        println!("s: {}", s);

        println!("triangulator uncheck delaunay: ");
        s = 0;
        for i in 0..8 {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s += test.run_unchecked_triangulator(count, repeat_count, true);
        }
        println!("s: {}", s);

        println!("earcutr: ");
        s = 0;
        for i in 0..n {
            let count = 4 << i;
            s += test.run_earcutr(count);
        }
        println!("s: {}", s);
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

        println!("triangulator uncheck raw: ");
        let s6 = test.run_triangulator(count, repeat_count, false);
        println!();

        println!("triangulator uncheck delaunay: ");
        let s7 = test.run_triangulator(count, repeat_count, true);
        println!();

        println!("earcutr: ");
        let s8 = test.run_earcutr(count);
        println!();

        println!(
            "s0: {}, s1: {}, s2: {}, s3: {}, s4: {}, s5: {}, s6: {}, s7: {}, s8: {}",
            s0, s1, s2, s3, s4, s5, s6, s7, s8
        );
    }
}

#[allow(dead_code)]
fn rect_with_star_holes(args: &EnvArgs) {
    let complex = args.get_bool("complex");

    let test = RectStarHolesTest {
        radius: 100.0,
        angle_steps_count: 5,
        points_per_corner: 10,
        radius_steps_count: 5,
        min_radius_scale: 0.0,
        max_radius_scale: 1.0,
        corners_count: 5,
    };

    if complex {
        let n = 5;
        let mut s;

        println!("unchecked raw: ");
        s = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s += test.run_unchecked_raw(count, repeat_count);
        }
        println!("s: {}", s);

        println!("unchecked delaunay: ");
        s = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s += test.run_unchecked_delaunay(count, repeat_count);
        }
        println!("s: {}", s);

        // println!("raw: ");
        // s = 0;
        // for i in 0..n {
        //     let count = 4 << i;
        //     let repeat_count = (256 / count).max(1);
        //     s += test.run_raw(count, repeat_count);
        // }
        // println!("s: {}", s);
        //
        // println!("delaunay: ");
        // s = 0;
        // for i in 0..n {
        //     let count = 4 << i;
        //     let repeat_count = (256 / count).max(1);
        //     s += test.run_delaunay(count, repeat_count);
        // }
        // println!("s: {}", s);
        //
        // println!("triangulator raw: ");
        // s = 0;
        // for i in 0..n {
        //     let count = 4 << i;
        //     let repeat_count = (256 / count).max(1);
        //     s += test.run_triangulator(count, repeat_count, false);
        // }
        // println!("s: {}", s);
        //
        // println!("triangulator delaunay: ");
        // s = 0;
        // for i in 0..n {
        //     let count = 4 << i;
        //     let repeat_count = (256 / count).max(1);
        //     s += test.run_triangulator(count, repeat_count, true);
        // }
        // println!("s: {}", s);

        println!("triangulator uncheck raw: ");
        s = 0;
        for i in 0..n {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s += test.run_unchecked_triangulator(count, repeat_count, false);
        }
        println!("s: {}", s);

        println!("triangulator uncheck delaunay: ");
        s = 0;
        for i in 0..8 {
            let count = 4 << i;
            let repeat_count = (256 / count).max(1);
            s += test.run_unchecked_triangulator(count, repeat_count, true);
        }
        println!("s: {}", s);

        println!("earcutr: ");
        s = 0;
        for i in 0..n {
            let count = 4 << i;
            s += test.run_earcutr(count);
        }
        println!("s: {}", s);
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

        println!("triangulator uncheck raw: ");
        let s6 = test.run_triangulator(count, repeat_count, false);
        println!();

        println!("triangulator uncheck delaunay: ");
        let s7 = test.run_triangulator(count, repeat_count, true);
        println!();

        println!("earcutr: ");
        let s8 = test.run_earcutr(count);
        println!();

        println!(
            "s0: {}, s1: {}, s2: {}, s3: {}, s4: {}, s5: {}, s6: {}, s7: {}, s8: {}",
            s0, s1, s2, s3, s4, s5, s6, s7, s8
        );
    }
}

#[allow(dead_code)]
fn spiral(args: &EnvArgs) {
    let complex = args.get_bool("complex");

    let test = SpiralTest { width: 10.0 };

    if complex {
        let mut tests = vec![
            Test::new(8, 10_000),
            Test::new(16, 10_000),
            Test::new(24, 10_000),
            Test::new(32, 10_000),
            Test::new(40, 10_000),
            Test::new(48, 10_000),
            Test::new(64, 10_000),
        ];
        for i in 1..6 {
            tests.push(Test::new(64 << i, 256 >> i));
        }

        let mut s;

        println!("flat + earcut: ");
        s = 0;
        for t in tests.iter() {
            s += test.run_triangle(t, false, true);
        }
        println!("s: {}", s);

        println!("flat + monotone: ");
        s = 0;
        for t in tests.iter() {
            s += test.run_triangle(t, false, false);
        }
        println!("s: {}", s);

        println!("flat + delaunay: ");
        s = 0;
        for t in tests.iter() {
            s += test.run_triangle(t, true, true);
        }
        println!("s: {}", s);

        println!("rust earcut: ");
        s = 0;
        for t in tests.iter() {
            s += test.run_earcutr(t);
        }
        println!("s: {}", s);
    } else {
        println!("not implemented");
    }
}

#[allow(dead_code)]
fn spike(args: &EnvArgs) {
    let complex = args.get_bool("complex");

    let test = SpikeTest {
        inner_radius: 80.0,
        outer_radius: 160.0,
    };

    if complex {
        let mut tests = vec![
            Test::new(8, 10_000),
            Test::new(16, 10_000),
            Test::new(24, 10_000),
            Test::new(32, 10_000),
            Test::new(40, 10_000),
            Test::new(48, 10_000),
            Test::new(64, 10_000),
        ];
        for i in 1..6 {
            tests.push(Test::new(64 << i, 256 >> i));
        }

        let mut s;

        println!("flat + earcut: ");
        s = 0;
        for t in tests.iter() {
            s += test.run_triangle(t, false, true);
        }
        println!("s: {}", s);

        println!("flat + monotone: ");
        s = 0;
        for t in tests.iter() {
            s += test.run_triangle(t, false, false);
        }
        println!("s: {}", s);

        println!("flat + delaunay: ");
        s = 0;
        for t in tests.iter() {
            s += test.run_triangle(t, true, true);
        }
        println!("s: {}", s);

        println!("rust earcut: ");
        s = 0;
        for t in tests.iter() {
            s += test.run_earcutr(t);
        }
        println!("s: {}", s);
    } else {
        println!("not implemented");
    }
}
