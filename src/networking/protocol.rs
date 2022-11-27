use naia_shared::Protocolize;

pub use self::{
    auth::Auth, entity_assignment::EntityAssignment, key_command::KeyCommand,
    network_position::NetworkPosition,
};

mod auth;
mod entity_assignment;
mod key_command;
mod network_position;

#[derive(Protocolize)]
pub enum Protocol {
    NetworkPosition(NetworkPosition),
    KeyCommand(KeyCommand),
    EntityAssignment(EntityAssignment),
    Auth(Auth),
    /*
    Color(Color), */
}
