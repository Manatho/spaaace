extern crate log;

pub mod behavior;
pub mod protocol;

mod channels;
pub mod projectiles;
pub use channels::Channels;

mod shared;
pub use shared::shared_config;
