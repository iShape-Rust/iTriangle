
pub(crate) struct SpiralTest;

impl SpiralTest {

    pub(crate) fn resource(count: usize) -> Vec<[f32; 2]> {
        Self::spiral(count)
    }

    pub(crate) fn points(count: usize) -> Vec<f32> {
        Self::spiral(count)
            .into_iter()
            .flat_map(|p| p)
            .collect()
    }
}

impl SpiralTest {
    fn spiral(count: usize) -> Vec<[f32; 2]> {
        let s = 10.0;
        
        let mut path_0: Vec<[f32; 2]> = Vec::with_capacity(count);
        let mut path_1: Vec<[f32; 2]> = Vec::with_capacity(count / 2);

        let mut s0 = s;
        let mut s1 = 2.0 * s;

        let mut x0 = 0.0f32;
        let mut y0 = 0.0f32;

        let mut x1 = 0.0f32;
        let mut y1 = 0.0f32;

        y0 += s0;
        path_0.push([x0, y0]);

        x0 += s0;
        path_0.push([x0, y0]);

        path_1.push([x1, y1]);

        x1 += s1;
        path_1.push([x1, y1]);
        s1 += s;

        let n = count - 4;

        for i in 0..n / 2 {
            match i % 4 {
                0 => {
                    y0 += s0;
                    y1 += s1;
                }
                1 => {
                    x0 -= s0;
                    x1 -= s1;
                }
                2 => {
                    y0 -= s0;
                    y1 -= s1;
                }
                _ => {
                    x0 += s0;
                    x1 += s1;
                }
            }
            path_0.push([x0, y0]);
            path_1.push([x1, y1]);

            s0 += s;
            s1 += s;
        }

        path_1.extend(path_0.into_iter().rev());
        path_1
    }
}
