use bevy::{
    asset::diagnostic,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::{Res, ResMut},
};
use bevy_egui::{egui::Window, EguiContext};

pub fn fps_gui(mut egui_context: ResMut<EguiContext>, diagnostics: Res<Diagnostics>) {
    Window::new("Hello").show(egui_context.ctx_mut(), |ui| {
        ui.label(format!(
            "{}",
            diagnostics
                .get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
                .and_then(|fps| fps.smoothed())
                .unwrap_or(0.0)
        ));
    });
}
