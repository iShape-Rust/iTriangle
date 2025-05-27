mod data;
mod app;
mod draw;
mod sheet;
mod geom;
mod path_editor;
mod compat;
mod mesh;
mod mesh_viewer;

use iced::application;
use crate::app::main::EditorApp;
use crate::data::resource::AppResource;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> iced::Result {
    run_desktop()
}

#[cfg(not(target_arch = "wasm32"))]
fn run_desktop() -> iced::Result {
    let app_initializer = move || {
        let app = EditorApp::new(AppResource::with_paths(
            "./tests/triangle",
        ));
        (app, iced::Task::none())
    };

    application(app_initializer, EditorApp::update, EditorApp::view)
        .resizable(true)
        .centered()
        .title("iTriangle Editor")
        .subscription(EditorApp::subscription)
        .run()
}