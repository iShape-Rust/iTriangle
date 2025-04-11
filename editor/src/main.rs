mod data;
mod app;
mod draw;
mod sheet;
mod geom;
mod path_editor;
mod compat;
mod mesh;

use iced::application;
use crate::app::main::EditorApp;
use crate::data::resource::AppResource;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> iced::Result {
    run_desktop()
}

#[cfg(not(target_arch = "wasm32"))]
fn run_desktop() -> iced::Result {

    let app_resource = AppResource::with_paths(
        "./tests/triangle",
    );

    let app_initializer = || {
        let app = EditorApp::new(app_resource);
        (app, iced::Task::none())
    };

    application("iTriangle", EditorApp::update, EditorApp::view)
        .subscription(EditorApp::subscription)
        .run_with(app_initializer)
}