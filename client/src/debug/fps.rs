use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::{Res, ResMut},
};
use bevy_egui::{egui::Window, EguiContext};
use bevy_renet::renet::{DefaultChannel, RenetClient};
use spaaaace_shared::ClientMessages;

pub fn fps_gui(mut egui_context: ResMut<EguiContext>, diagnostics: Res<Diagnostics>) {
    Window::new("Fps").show(egui_context.ctx_mut(), |ui| {
        ui.label(format!(
            "{}",
            diagnostics
                .get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
                .and_then(|fps| fps.smoothed())
                .unwrap_or(0.0)
        ));
    });
}

pub fn team_swap_gui(
    mut egui_context: ResMut<EguiContext>,
    mut client: ResMut<RenetClient>,
) {
    Window::new("Team GUI").show(egui_context.ctx_mut(), |ui| {
        if ui.button("1").clicked() {
            send_message(&mut client, 1);
        }

        if ui.button("2").clicked() {
            send_message(&mut client, 2);
        }
    });
}

fn send_message(client: &mut ResMut<RenetClient>, team: u8) {
    let client_message = ClientMessages::Command {
        command: format!("swap_team {}", team),
    };
    let input_message = bincode::serialize(&client_message).unwrap();
    client.send_message(DefaultChannel::Reliable, input_message);
}
