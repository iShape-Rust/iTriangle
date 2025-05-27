#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::panic;
    use wasm_bindgen::prelude::*;
    use iced::{application};

    use log::info;
    use crate::app::main::EditorApp;
    use crate::data::resource::AppResource;

    #[wasm_bindgen]
    pub struct WebApp;

    #[wasm_bindgen]
    impl WebApp {
        #[wasm_bindgen(constructor)]
        pub fn create() -> Self {
            Self {}
        }

        #[wasm_bindgen]
        pub fn start(&self, triangle_data: String) {
            panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Debug).expect("error initializing log");
            info!("Starting application...");

            let app_resource = AppResource::with_content(triangle_data);

            let app_initializer = move || {
                info!("wasm init");
                let app = EditorApp::with_resource(AppResource::with_content(&triangle_data));

                (app, iced::Task::none())
            };

            application(app_initializer, EditorApp::update, EditorApp::view)
                .resizable(true)
                .centered()
                .title("iTriangle Editor")
                .subscription(EditorApp::subscription)
                .run()
                .unwrap();
        }
    }
}
