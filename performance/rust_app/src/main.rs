use crate::test::runner::Runner;
use crate::test::test::TestData;
use crate::test::test_2_star_with_hole::StarWithHoleTest;
use crate::test::test_3_star_with_8_holes::StarWith8HolesTest;
use crate::test::test_1_spiral::SpiralTest;
use crate::test::test_0_star::StarTest;
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
        0 => test_0(),
        1 => test_1(),
        2 => test_2(),
        3 => test_3(),
        _ => {
            panic!("No test found")
        }
    }
}

#[cfg(debug_assertions)]
fn debug_run() {
    let mut s = 0;
    let t = TestData::new(64, 100_000);
    let resource = SpiralTest::resource(t.count);
    s += Runner::run_triangle(&resource, &t, false, true);

    println!("s: {}", s);
}

#[allow(dead_code)]
fn test_0() {
        let mut tests = vec![
            TestData::new(8, 10_000),
            TestData::new(16, 10_000),
            TestData::new(32, 10_000),
            TestData::new(64, 10_000),
        ];

        for i in 1..9 {
            tests.push(TestData::new(64 << i, 256 >> i));
        }

        let mut s;

        println!("flat + earcut: ");
        s = 0;
        for t in tests.iter() {
            let contour = StarTest::contour(t.count);
            s += Runner::run_triangle(&contour, t, false, true);
        }
        println!("s: {}", s);

        println!("flat + monotone: ");
        s = 0;
        for t in tests.iter() {
            let contour = StarTest::contour(t.count);
            s += Runner::run_triangle(&contour, t, false, false);
        }
        println!("s: {}", s);

        println!("flat + delaunay: ");
        s = 0;
        for t in tests.iter() {
            let contour = StarTest::contour(t.count);
            s += Runner::run_triangle(&contour, t, true, true);
        }
        println!("s: {}", s);

        println!("rust earcut: ");
        s = 0;
        for t in tests.iter() {
            let points = StarTest::points(t.count);
            s += Runner::run_earcut(points, t);
        }
        println!("s: {}", s);
}

#[allow(dead_code)]
fn test_1() {
    let mut tests = vec![
        TestData::new(8, 10_000),
        TestData::new(16, 10_000),
        TestData::new(32, 10_000),
        TestData::new(64, 10_000),
    ];

    for i in 1..9 {
        tests.push(TestData::new(64 << i, 256 >> i));
    }

    let mut s;

    println!("flat + earcut: ");
    s = 0;
    for t in tests.iter() {
        let resource = SpiralTest::resource(t.count);
        s += Runner::run_triangle(&resource, t, false, true);
    }
    println!("s: {}", s);

    println!("flat + monotone: ");
    s = 0;
    for t in tests.iter() {
        let resource = SpiralTest::resource(t.count);
        s += Runner::run_triangle(&resource, t, false, false);
    }
    println!("s: {}", s);

    println!("flat + delaunay: ");
    s = 0;
    for t in tests.iter() {
        let resource = SpiralTest::resource(t.count);
        s += Runner::run_triangle(&resource, t, true, true);
    }
    println!("s: {}", s);

    println!("rust earcut: ");
    s = 0;
    for t in tests.iter() {
        let points = SpiralTest::points(t.count);
        s += Runner::run_earcut(points, t);
    }
    println!("s: {}", s);
}

#[allow(dead_code)]
fn test_2() {
    let mut tests = Vec::new();

    for i in 1..9 {
        tests.push(TestData::new(64 << i, 256 >> i));
    }

    let mut s;

    println!("flat + earcut: ");
    s = 0;
    for t in tests.iter() {
        let resource = StarWithHoleTest::resource(t.count);
        s += Runner::run_triangle(&resource, t, false, true);
    }
    println!("s: {}", s);

    println!("flat + monotone: ");
    s = 0;
    for t in tests.iter() {
        let resource = StarWithHoleTest::resource(t.count);
        s += Runner::run_triangle(&resource, t, false, false);
    }
    println!("s: {}", s);

    println!("flat + delaunay: ");
    s = 0;
    for t in tests.iter() {
        let resource = StarWithHoleTest::resource(t.count);
        s += Runner::run_triangle(&resource, t, true, true);
    }
    println!("s: {}", s);

    println!("rust earcut: ");
    s = 0;
    for t in tests.iter() {
        let points = StarWithHoleTest::points(t.count);
        s += Runner::run_earcut(points, t);
    }
    println!("s: {}", s);
}

#[allow(dead_code)]
fn test_3() {
    let mut tests = Vec::new();

    for i in 1..8 {
        tests.push(TestData::new(128 << i, 256 >> i));
    }

    let mut s;

    println!("flat + earcut: ");
    s = 0;
    for t in tests.iter() {
        let resource = StarWith8HolesTest::resource(t.count);
        s += Runner::run_triangle(&resource, t, false, true);
    }
    println!("s: {}", s);

    println!("flat + monotone: ");
    s = 0;
    for t in tests.iter() {
        let resource = StarWith8HolesTest::resource(t.count);
        s += Runner::run_triangle(&resource, t, false, false);
    }
    println!("s: {}", s);

    println!("flat + delaunay: ");
    s = 0;
    for t in tests.iter() {
        let resource = StarWith8HolesTest::resource(t.count);
        s += Runner::run_triangle(&resource, t, true, true);
    }
    println!("s: {}", s);

    println!("rust earcut: ");
    s = 0;
    for t in tests.iter() {
        let points = StarWith8HolesTest::points(t.count);
        s += Runner::run_earcut(points, t);
    }
    println!("s: {}", s);
}