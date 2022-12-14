pub mod particles;
pub mod utils;
pub mod capture_point;
pub mod debug;
pub mod camera;
pub mod ui;
pub mod controls;

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
