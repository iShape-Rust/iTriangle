use crate::data::triangle::TriangleResource;

pub struct AppResource {
    pub(crate) triangle: TriangleResource,
}

impl AppResource {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_paths(triangle: &str) -> Self {
        Self {
            triangle: TriangleResource::with_path(triangle),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn with_content(triangle: String) -> Self {
        Self {
            triangle: TriangleResource::with_content(triangle)
        }
    }

}