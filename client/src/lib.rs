pub mod camera;
pub mod capture_point;
pub mod controls;
pub mod debug;
pub mod game_state;
pub mod particles;
pub mod player;
pub mod ship_editor;
pub mod skybox;
pub mod ui;
pub mod utils;

#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {

        mod resources;
        mod systems;
        mod app;

        use wasm_bindgen::prelude::*;

        #[wasm_bindgen(start)]
        pub fn main() -> Result<(), JsValue> {
            app::run();

            Ok(())
        }
    }
}
