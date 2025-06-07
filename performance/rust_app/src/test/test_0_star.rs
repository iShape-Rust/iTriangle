use std::f32::consts::PI;

pub(crate) struct StarTest;

impl StarTest {

    pub(crate) fn contour(count: usize) -> Vec<[f32; 2]> {
        Self::star(count)
    }

    pub(crate) fn points(count: usize) -> Vec<f32> {
        Self::star(count)
            .into_iter()
            .flat_map(|p| p)
            .collect()
    }
}

impl StarTest {
    fn star(count: usize) -> Vec<[f32; 2]> {
        let r0 = 160.0;
        let r1 = 80.0;
        
        let n = count / 2;
        let da = PI / (n as f32);
        let mut a = 0.0f32;

        let mut path = Vec::with_capacity(count);
        for _ in 0..n {
            let (s, c) = a.sin_cos();
            let x = c * r0;
            let y = s * r0;
            path.push([x, y]);

            a += da;

            let (s, c) = a.sin_cos();
            let x = c * r1;
            let y = s * r1;

            path.push([x, y]);

            a += da;
        }

        path
    }
}