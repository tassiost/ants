use crate::{
    benchmark::BenchmarkConfig,
    configs::SpriteHandles,
    grid::GridKey,
    gui::SimStatistics,
    pheromone::Pheromones,
    utils::{calc_rotation_angle, get_rand_unit_vec2},
    *,
};
use bevy::{
    math::{vec2, vec3},
    prelude::*,
};
use rand::Rng;
use std::f32::consts::PI;

pub struct AntPlugin;

pub enum AntTask {
    FindFood,
    FindHome,
}

#[derive(Component)]
pub struct Ant;
#[derive(Component)]
pub struct CurrentTask(pub AntTask);
#[derive(Component)]
struct Velocity(Vec2);
#[derive(Component)]
struct Acceleration(Vec2);
#[derive(Component)]
struct PhStrength(f32);

#[derive(Resource)]
struct AntScanRadius(f32);
#[derive(Resource)]
pub struct AntFollowCameraPos(pub Vec2);

impl Plugin for AntPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .insert_resource(AntScanRadius(INITIAL_ANT_PH_SCAN_RADIUS))
            .insert_resource(AntFollowCameraPos(Vec2::ZERO))
            .add_systems(FixedUpdate, drop_pheromone)
            .add_systems(FixedUpdate, check_collisions)
            .add_systems(Update, update_camera_follow_pos)
            .add_systems(FixedUpdate, periodic_direction_update)
            .add_systems(Update, update_stats)
            .add_systems(FixedUpdate, update_scan_radius)
            .add_systems(FixedUpdate, decay_ph_strength)
            .add_systems(FixedUpdate, update_position);
    }
}

fn setup(
    mut commands: Commands,
    mut rng: ResMut<DeterministicRng>,
    benchmark_config: Option<Res<BenchmarkConfig>>,
    sprite_handles: Res<SpriteHandles>,
) {
    let count = benchmark_config
        .map(|c| c.ant_count)
        .unwrap_or(NUM_ANTS);

    for _ in 0..count {
        commands.spawn((
            Sprite {
                image: sprite_handles.ant.clone(),
                ..default()
            },
            Transform::from_xyz(HOME_LOCATION.0, HOME_LOCATION.1, ANT_Z_INDEX)
                .with_scale(Vec3::splat(ANT_SPRITE_SCALE)),
            Ant,
            CurrentTask(AntTask::FindFood),
            Velocity(get_rand_unit_vec2(&mut rng.inner)),
            Acceleration(Vec2::ZERO),
            PhStrength(ANT_INITIAL_PH_STRENGTH),
        ));
    }
}

fn drop_pheromone(
    mut ant_query: Query<(&Transform, &CurrentTask, &PhStrength), With<Ant>>,
    mut pheromones: ResMut<Pheromones>,
) {
    for (transform, ant_task, ph_strength) in ant_query.iter_mut() {
        let x = transform.translation.x as i32;
        let y = transform.translation.y as i32;

        match ant_task.0 {
            AntTask::FindFood => pheromones.to_home.emit_signal(&GridKey(x, y), ph_strength.0),
            AntTask::FindHome => pheromones.to_food.emit_signal(&GridKey(x, y), ph_strength.0),
        }
    }
}

fn update_scan_radius(mut scan_radius: ResMut<AntScanRadius>) {
    if scan_radius.0 > INITIAL_ANT_PH_SCAN_RADIUS * ANT_PH_SCAN_RADIUS_SCALE {
        return;
    }
    scan_radius.0 += ANT_PH_SCAN_RADIUS_INCREMENT;
}

fn update_camera_follow_pos(
    ant_query: Query<&Transform, With<Ant>>,
    mut follow_pos: ResMut<AntFollowCameraPos>,
) {
    if let Some(transform) = ant_query.iter().next() {
        follow_pos.0 = transform.translation.truncate();
    }
}

fn update_stats(
    mut stats: ResMut<SimStatistics>,
    scan_radius: Res<AntScanRadius>,
    ant_query: Query<Entity, With<Ant>>,
) {
    stats.scan_radius = scan_radius.0;
    stats.num_ants = ant_query.iter().count();
}

fn decay_ph_strength(mut ant_query: Query<&mut PhStrength, With<Ant>>) {
    for mut ph_strength in ant_query.iter_mut() {
        ph_strength.0 = f32::max(ph_strength.0 - ANT_PH_STRENGTH_DECAY_RATE, 0.0);
    }
}

fn get_steering_force(target: Vec2, current: Vec2, velocity: Vec2) -> Vec2 {
    let desired = target - current;
    let steering = desired - velocity;
    steering * 0.05
}

fn periodic_direction_update(
    mut ant_query: Query<(&mut Acceleration, &Transform, &CurrentTask, &Velocity), With<Ant>>,
    mut pheromones: ResMut<Pheromones>,
    mut stats: ResMut<SimStatistics>,
    scan_radius: Res<AntScanRadius>,
    mut rng: ResMut<DeterministicRng>,
) {
    (stats.food_cache_size, stats.home_cache_size) = pheromones.clear_cache();

    for (mut acceleration, transform, current_task, velocity) in ant_query.iter_mut() {
        let current_pos = transform.translation;
        let mut target = None;

        match current_task.0 {
            AntTask::FindFood => {
                let dist_to_food = transform.translation.distance_squared(vec3(
                    FOOD_LOCATION.0,
                    FOOD_LOCATION.1,
                    0.0,
                ));
                if dist_to_food <= ANT_TARGET_AUTO_PULL_RADIUS * ANT_TARGET_AUTO_PULL_RADIUS {
                    target = Some(vec2(FOOD_LOCATION.0, FOOD_LOCATION.1));
                }
            }
            AntTask::FindHome => {
                let dist_to_home = transform.translation.distance_squared(vec3(
                    HOME_LOCATION.0,
                    HOME_LOCATION.1,
                    0.0,
                ));
                if dist_to_home <= ANT_TARGET_AUTO_PULL_RADIUS * ANT_TARGET_AUTO_PULL_RADIUS {
                    target = Some(vec2(HOME_LOCATION.0, HOME_LOCATION.1));
                }
            }
        };

        if target.is_none() {
            match current_task.0 {
                AntTask::FindFood => {
                    target = pheromones
                        .to_food
                        .get_steer_target(&current_pos.truncate(), scan_radius.0);
                }
                AntTask::FindHome => {
                    target = pheromones
                        .to_home
                        .get_steer_target(&current_pos.truncate(), scan_radius.0);
                }
            }
        }

        if target.is_none() {
            acceleration.0 += get_rand_unit_vec2(&mut rng.inner) * 0.2;
            continue;
        }

        let steering_force = get_steering_force(
            target.unwrap(),
            transform.translation.truncate(),
            velocity.0,
        );

        acceleration.0 += steering_force * rng.inner.gen_range(0.4..=ANT_STEERING_FORCE_FACTOR);
    }
}

fn check_collisions(
    mut ant_query: Query<
        (
            &Transform,
            &mut Sprite,
            &mut Velocity,
            &mut CurrentTask,
            &mut PhStrength,
        ),
        With<Ant>,
    >,
    sprite_handles: Res<SpriteHandles>,
    mut rng: ResMut<DeterministicRng>,
) {
    for (transform, mut sprite, mut velocity, mut ant_task, mut ph_strength) in
        ant_query.iter_mut()
    {
        let border = 20.0;
        let top_left = (-W / 2.0, H / 2.0);
        let bottom_right = (W / 2.0, -H / 2.0);
        let x_bound = transform.translation.x < top_left.0 + border
            || transform.translation.x >= bottom_right.0 - border;
        let y_bound = transform.translation.y >= top_left.1 - border
            || transform.translation.y < bottom_right.1 + border;
        if x_bound || y_bound {
            let target = vec2(
                rng.inner.gen_range(-200.0..200.0),
                rng.inner.gen_range(-200.0..200.0),
            );
            let vel = velocity.0;
            velocity.0 += get_steering_force(target, transform.translation.truncate(), vel);
        }

        let dist_to_home =
            transform
                .translation
                .distance_squared(vec3(HOME_LOCATION.0, HOME_LOCATION.1, 0.0));
        if dist_to_home < HOME_RADIUS * HOME_RADIUS {
            match ant_task.0 {
                AntTask::FindFood => {}
                AntTask::FindHome => {
                    velocity.0 *= -1.0;
                }
            }
            ant_task.0 = AntTask::FindFood;
            ph_strength.0 = ANT_INITIAL_PH_STRENGTH;
            sprite.image = sprite_handles.ant.clone();
            sprite.color = Color::srgb(1.0, 1.0, 2.5);
        }

        let dist_to_food =
            transform
                .translation
                .distance_squared(vec3(FOOD_LOCATION.0, FOOD_LOCATION.1, 0.0));
        if dist_to_food < FOOD_PICKUP_RADIUS * FOOD_PICKUP_RADIUS {
            match ant_task.0 {
                AntTask::FindFood => {
                    velocity.0 *= -1.0;
                }
                AntTask::FindHome => {}
            }
            ant_task.0 = AntTask::FindHome;
            ph_strength.0 = ANT_INITIAL_PH_STRENGTH;
            sprite.image = sprite_handles.ant_with_food.clone();
            sprite.color = Color::srgb(1.0, 2.0, 1.0);
        }
    }
}

fn update_position(
    mut ant_query: Query<(&mut Transform, &mut Velocity, &mut Acceleration), With<Ant>>,
) {
    for (mut transform, mut velocity, mut acceleration) in ant_query.iter_mut() {
        let old_pos = transform.translation;

        if !acceleration.0.is_nan() {
            velocity.0 = (velocity.0 + acceleration.0).normalize();
            let new_translation =
                transform.translation + vec3(velocity.0.x, velocity.0.y, 0.0) * ANT_SPEED;
            if !new_translation.is_nan() {
                transform.translation = new_translation;
            }
        }

        acceleration.0 = Vec2::ZERO;
        transform.rotation =
            Quat::from_rotation_z(calc_rotation_angle(old_pos, transform.translation) + PI / 2.0);
    }
}
