use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy_egui::EguiContexts;

#[derive(Resource)]
pub struct SimSettings {
    pub is_camera_follow: bool,
    pub is_show_ants_path: bool,
    pub is_show_food_ph: bool,
    pub is_show_home_ph: bool,
    pub is_show_ants: bool,
}

impl Default for SimSettings {
    fn default() -> Self {
        Self {
            is_camera_follow: false,
            is_show_ants_path: true,
            is_show_food_ph: true,
            is_show_home_ph: true,
            is_show_ants: true,
        }
    }
}

#[derive(Resource)]
pub struct SimStatistics {
    pub scan_radius: f32,
    pub num_ants: usize,
    pub ph_home_size: u32,
    pub ph_food_size: u32,
    pub food_cache_size: u32,
    pub home_cache_size: u32,
}

impl Default for SimStatistics {
    fn default() -> Self {
        Self {
            scan_radius: 0.0,
            num_ants: 0,
            ph_home_size: 0,
            ph_food_size: 0,
            food_cache_size: 0,
            home_cache_size: 0,
        }
    }
}

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimSettings::default())
            .insert_resource(SimStatistics::default())
            .add_systems(Update, settings_toggle)
            .add_systems(Update, settings_dialog);
    }
}

fn settings_toggle(
    mut sim_settings: ResMut<SimSettings>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F1) {
        sim_settings.is_camera_follow = !sim_settings.is_camera_follow;
    }
    if keyboard_input.just_pressed(KeyCode::F2) {
        sim_settings.is_show_ants_path = !sim_settings.is_show_ants_path;
    }
    if keyboard_input.just_pressed(KeyCode::F3) {
        sim_settings.is_show_food_ph = !sim_settings.is_show_food_ph;
    }
    if keyboard_input.just_pressed(KeyCode::F4) {
        sim_settings.is_show_home_ph = !sim_settings.is_show_home_ph;
    }
    if keyboard_input.just_pressed(KeyCode::F5) {
        sim_settings.is_show_ants = !sim_settings.is_show_ants;
    }
}

fn settings_dialog(mut contexts: EguiContexts, mut sim_settings: ResMut<SimSettings>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::Window::new("Settings").show(ctx, |ui| {
        ui.checkbox(&mut sim_settings.is_camera_follow, "Camera follow");
        ui.checkbox(&mut sim_settings.is_show_ants_path, "Show ant paths");
        ui.checkbox(&mut sim_settings.is_show_food_ph, "Show food pheromones");
        ui.checkbox(&mut sim_settings.is_show_home_ph, "Show home pheromones");
        ui.checkbox(&mut sim_settings.is_show_ants, "Show ants");
    });
}
