pub mod triangulation {
    use std::path::PathBuf;
    use i_shape::int::path::IntPath;
    use i_shape::int::shape::IntShape;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Test {
        pub shape: IntShape,
        pub points: IntPath,
        pub indices: Vec<usize>,
        pub polygons: Vec<IntPath>,
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

            let result: Result<Test, _> = serde_json::from_str(&data);
            match result {
                Ok(test) => Test::from(test),
                Err(e) => {
                    eprintln!("Failed to parse JSON: {}", e);
                    panic!("can not parse file")
                }
            }
        }
    }
}


