use ants::{
    ant::{AntFollowCameraPos, AntPlugin},
    benchmark::{BenchmarkConfig, BenchmarkPlugin},
    configs::{DeterministicRng, SpriteHandles},
    gui::{GuiPlugin, SimSettings},
    pathviz::PathVizPlugin,
    pheromone::PheromonePlugin,
    *,
};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::vec3,
    prelude::*,
    render::diagnostic::RenderDiagnosticsPlugin,
};
use bevy_pancam::{PanCam, PanCamPlugin};
use rand::SeedableRng;

#[derive(Component)]
struct FollowCamera;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut seed: u64 = 42;
    let mut ant_count: Option<u32> = None;
    let mut benchmark_enabled: bool = false;
    let mut benchmark_duration: f32 = 30.0;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--seed" if i + 1 < args.len() => {
                seed = args[i + 1].parse().unwrap_or(42);
                i += 2;
            }
            "--ants" if i + 1 < args.len() => {
                ant_count = args[i + 1].parse().ok();
                i += 2;
            }
            "--benchmark" => {
                benchmark_enabled = true;
                i += 1;
            }
            "--duration" if i + 1 < args.len() => {
                benchmark_duration = args[i + 1].parse().unwrap_or(30.0);
                i += 2;
            }
            _ => i += 1,
        }
    }

    let ant_count = ant_count.unwrap_or(NUM_ANTS);

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resizable: false,
                        focused: true,
                        resolution: (W as u32, H as u32).into(),
                        title: "Ants".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(DeterministicRng {
            inner: rand::rngs::StdRng::seed_from_u64(seed),
        })
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(BenchmarkConfig {
            enabled: benchmark_enabled,
            ant_count,
            duration_secs: benchmark_duration,
            log_interval_secs: 5.0,
        })
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(RenderDiagnosticsPlugin::default())
        .add_systems(Update, close_on_esc)
        .add_plugins(PanCamPlugin)
        .insert_resource(ClearColor(Color::srgba_u8(
            BG_COLOR.0, BG_COLOR.1, BG_COLOR.2, 0,
        )))
        .add_systems(Startup, (setup, load_sprites).chain())
        .add_systems(Update, ant_follow_camera)
        .add_plugins(AntPlugin)
        .add_plugins(PheromonePlugin)
        .add_plugins(PathVizPlugin)
        .add_plugins(GuiPlugin)
        .add_plugins(BenchmarkPlugin)
        .run();
}

fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }
        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}

fn ant_follow_camera(
    ant_pos: Res<AntFollowCameraPos>,
    sim_settings: Res<SimSettings>,
    mut camera_query: Query<&mut Transform, With<FollowCamera>>,
) {
    if !sim_settings.is_camera_follow {
        return;
    }
    let Ok(mut transform) = camera_query.single_mut() else { return };
    transform.translation = vec3(ant_pos.0.x, ant_pos.0.y, ANT_Z_INDEX);
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Camera2d,
            Camera::default(),
            Transform::from_xyz(0.0, 0.0, 9999999.0),
            FollowCamera,
        ))
        .insert(PanCam::default());

    commands.spawn((
        Sprite {
            image: asset_server.load(SPRITE_ANT_COLONY),
            ..default()
        },
        Transform::from_xyz(HOME_LOCATION.0, HOME_LOCATION.1, 2.0)
            .with_scale(Vec3::splat(HOME_SPRITE_SCALE)),
    ));

    commands.spawn((
        Sprite {
            image: asset_server.load(SPRITE_FOOD),
            ..default()
        },
        Transform::from_xyz(FOOD_LOCATION.0, FOOD_LOCATION.1, 2.0)
            .with_scale(Vec3::splat(FOOD_SPRITE_SCALE)),
    ));
}

fn load_sprites(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SpriteHandles {
        ant: asset_server.load(SPRITE_ANT),
        ant_with_food: asset_server.load(SPRITE_ANT_WITH_FOOD),
    });
}
