use naia_shared::Protocolize;

mod auth;
mod entity_assignment;
mod key_command;
mod network_position;

pub use auth::Auth;
pub use entity_assignment::EntityAssignment; 
pub use key_command::KeyCommand;    
pub use network_position::NetworkPosition;

#[derive(Protocolize)]
pub enum Protocol {
    NetworkPosition(NetworkPosition),
    KeyCommand(KeyCommand),
    EntityAssignment(EntityAssignment),
    Auth(Auth),
}
