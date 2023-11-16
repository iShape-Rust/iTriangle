use i_shape::fix_path::FixPath;
use i_shape::fix_shape::FixShape;

pub struct FlipShape {
    paths: Vec<FixPath>
}

impl FlipShape {

    pub fn paths(&self) -> &Vec<FixPath> {
        &self.paths
    }

    pub fn with_paths(paths: Vec<FixPath>) -> Self {
        Self {
            paths
        }
    }

}

pub trait Flip {
    fn to_flip(&self) -> FlipShape;
    fn into_flip(self) -> FlipShape;
}

impl Flip for FixShape {

    fn to_flip(&self) -> FlipShape {
        let mut paths = Vec::new();

        paths.push(self.contour().clone());

        let n = self.paths.len();
        if n > 1 {
            for i in 1..n {
                let mut path = self.paths[i].clone();
                path.reverse();
                paths.push(path);
            }
        }

        FlipShape::with_paths(paths)
    }

    fn into_flip(self) -> FlipShape {
        let mut paths = self.paths;
        let n = paths.len();
        for i in 1..n {
            paths[i].reverse();
        }

        FlipShape::with_paths(paths)
    }

}
