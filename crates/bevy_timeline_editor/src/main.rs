use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_timeline_runtime::prelude::*;
use bevy_timeline_impls::prelude::*;

type TimelineSet = (TransformSet,StdMaterialSet,DirLightSet,PointLightSet,SpotLightSet,);
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins( TimelinePlugin::<TimelineSet>::new() )
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, draw_timeline)
    .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn draw_timeline(
    time: Res<Time>,
    mut egui_context: EguiContexts,
) -> Result {
    egui::Window::new("Hello").show(egui_context.ctx_mut()?, |ui| {
        ui.label("world");
    });
    egui::TopBottomPanel::bottom("timeline_bottom_panel")
        .max_height(500.0)
        .resizable(true)
        .show(egui_context.ctx_mut()?, |ui| {
            ui.heading("timeline_bottom_panel");
        });
    Ok(())
}