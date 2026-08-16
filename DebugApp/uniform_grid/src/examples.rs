pub type Point = [f32; 2];
pub type Shape = Vec<Vec<Point>>;

#[derive(Clone)]
pub struct GridExample {
    pub name: &'static str,
    pub shape: Shape,
    pub edge_length: f32,
}

pub fn load_examples() -> Vec<GridExample> {
    vec![
        GridExample {
            name: "concave polygon",
            shape: vec![vec![
                [-220.0, -135.0],
                [35.0, -175.0],
                [220.0, -70.0],
                [80.0, 5.0],
                [205.0, 155.0],
                [-25.0, 105.0],
                [-205.0, 175.0],
                [-115.0, 10.0],
            ]],
            edge_length: 42.0,
        },
        GridExample {
            name: "shape with hole",
            shape: vec![
                vec![
                    [-225.0, -170.0],
                    [225.0, -170.0],
                    [225.0, 170.0],
                    [-225.0, 170.0],
                ],
                // Clockwise winding makes this contour a hole for NonZero fill.
                vec![[-95.0, -65.0], [-95.0, 75.0], [105.0, 75.0], [105.0, -65.0]],
            ],
            edge_length: 38.0,
        },
        GridExample {
            name: "narrow contour",
            shape: vec![vec![
                [-235.0, -28.0],
                [235.0, -28.0],
                [235.0, 28.0],
                [-235.0, 28.0],
            ]],
            edge_length: 72.0,
        },
        GridExample {
            name: "narrow passage",
            shape: vec![vec![
                [-230.0, -175.0],
                [-25.0, -175.0],
                [-25.0, 45.0],
                [25.0, 45.0],
                [25.0, -175.0],
                [230.0, -175.0],
                [230.0, 175.0],
                [25.0, 175.0],
                [25.0, 95.0],
                [-25.0, 95.0],
                [-25.0, 175.0],
                [-230.0, 175.0],
            ]],
            edge_length: 50.0,
        },
        GridExample {
            name: "two holes",
            shape: vec![
                vec![
                    [-240.0, -175.0],
                    [240.0, -175.0],
                    [215.0, 175.0],
                    [-215.0, 175.0],
                ],
                vec![
                    [-150.0, -55.0],
                    [-150.0, 70.0],
                    [-45.0, 70.0],
                    [-45.0, -55.0],
                ],
                vec![[45.0, -80.0], [45.0, 45.0], [160.0, 45.0], [160.0, -80.0]],
            ],
            edge_length: 34.0,
        },
    ]
}
