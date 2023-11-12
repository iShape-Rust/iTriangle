pub mod triangulation {
    use std::path::PathBuf;
    use i_shape::fix_path::FixPath;
    use i_shape::fix_shape::FixShape;
    use i_triangle::delaunay::convex::{ConvexPath, ConvexSide};
    use serde::{Deserialize, Deserializer};

    #[derive(Debug)]
    pub struct Test {
        pub shape: FixShape,
        pub points: FixPath,
        pub indices: Vec<usize>,
        pub polygons: Vec<ConvexPath>,
    }

    #[derive(Debug, Deserialize)]
    struct TestData {
        pub shape: FixShape,
        pub points: FixPath,
        pub indices: Vec<usize>,
        pub polygons: Vec<ConvexPathData>
    }

    #[derive(Debug, Deserialize)]
    struct ConvexPathData {
        path: FixPath,
        side: Vec<ConvexSideData>,
    }

    #[derive(Debug)]
    enum ConvexSideData {
        Inner,
        Outer,
    }

    impl<'de> Deserialize<'de> for ConvexSideData {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
        {
            let value = i32::deserialize(deserializer)?;
            match value {
                0 => Ok(ConvexSideData::Outer),
                1 => Ok(ConvexSideData::Inner),
                _ => Err(serde::de::Error::custom("Unexpected value")),
            }
        }
    }

    impl From<ConvexPathData> for ConvexPath {
        fn from(data: ConvexPathData) -> Self {
            let side: Vec<ConvexSide> = data.side.into_iter().map(ConvexSide::from).collect();
            ConvexPath { path: data.path, side }
        }
    }

    impl From<ConvexSideData> for ConvexSide {
        fn from(data: ConvexSideData) -> Self {
            match data {
                ConvexSideData::Inner => ConvexSide::Inner,
                ConvexSideData::Outer => ConvexSide::Outer,
            }
        }
    }

    impl From<TestData> for Test {
        fn from(data: TestData) -> Self {
            Self {
                shape: data.shape,
                points: data.points,
                indices: data.indices,
                polygons: data.polygons.into_iter().map(|it| ConvexPath::from(it)).collect(),
            }
        }
    }

    impl Test {
        pub fn load(index: usize) -> Self {
            let file_name = format!("triangle_test_{}.json", index);
            let mut path_buf = PathBuf::from("./tests/data");
            path_buf.push(file_name);

            let data = match std::fs::read_to_string(path_buf.as_path()) {
                Ok(data) => {
                    data
                },
                Err(e) => {
                    panic!("{:?}", e);
                }
            };

            let result: Result<TestData, _> = serde_json::from_str(&data);
            match result {
                Ok(test_data) => Test::from(test_data),
                Err(e) => {
                    eprintln!("Failed to parse JSON: {}", e);
                    panic!("can not parse file")
                }
            }
        }
    }
}


