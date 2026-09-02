use crate::{
    gui::SimStatistics,
    *,
};
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use std::time::Instant;

#[derive(Resource)]
pub struct BenchmarkConfig {
    pub enabled: bool,
    pub ant_count: u32,
    pub duration_secs: f32,
    pub log_interval_secs: f32,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ant_count: NUM_ANTS,
            duration_secs: 30.0,
            log_interval_secs: 5.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct BenchmarkStats {
    pub start_time: Option<Instant>,
    pub tick_count: u64,
    pub sim_time_ms: f64,
    pub ai_time_ms: f64,
    pub spatial_query_time_ms: f64,
    pub pheromone_time_ms: f64,
    pub collision_time_ms: f64,
    pub last_log_time: f32,
}

pub struct BenchmarkPlugin;

impl Plugin for BenchmarkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BenchmarkConfig::default())
            .insert_resource(BenchmarkStats::default())
            .add_systems(Update, (
                benchmark_tick_count,
                benchmark_log_results,
                benchmark_exit_on_complete,
            ));
    }
}

fn benchmark_tick_count(mut stats: ResMut<BenchmarkStats>) {
    if stats.start_time.is_none() {
        stats.start_time = Some(Instant::now());
    }
    stats.tick_count += 1;
}

fn benchmark_log_results(
    time: Res<Time>,
    mut stats: ResMut<BenchmarkStats>,
    sim_stats: Res<SimStatistics>,
    diagnostics: Res<DiagnosticsStore>,
) {
    if time.elapsed_secs() - stats.last_log_time < 5.0 {
        return;
    }

    let frame_time = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.average())
        .unwrap_or(0.0);

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.average())
        .unwrap_or(0.0);

    info!(
        "BENCHMARK tick={} ants={} ph_home={} ph_food={} fps={:.1} frame_time={:.2}ms sim={:.2}ms ai={:.2}ms spatial={:.2}ms ph={:.2}ms collision={:.2}ms",
        stats.tick_count,
        sim_stats.num_ants,
        sim_stats.ph_home_size,
        sim_stats.ph_food_size,
        fps,
        frame_time * 1000.0,
        stats.sim_time_ms,
        stats.ai_time_ms,
        stats.spatial_query_time_ms,
        stats.pheromone_time_ms,
        stats.collision_time_ms,
    );

    stats.last_log_time = time.elapsed_secs();
}

fn benchmark_exit_on_complete(
    time: Res<Time>,
    stats: Res<BenchmarkStats>,
    config: Res<BenchmarkConfig>,
) {
    if config.enabled && time.elapsed_secs() >= config.duration_secs as f32 {
        info!(
            "BENCHMARK_COMPLETE ticks={} sim_time={:.2}ms ai_time={:.2}ms spatial={:.2}ms ph={:.2}ms collision={:.2}ms",
            stats.tick_count,
            stats.sim_time_ms,
            stats.ai_time_ms,
            stats.spatial_query_time_ms,
            stats.pheromone_time_ms,
            stats.collision_time_ms,
        );
        std::process::exit(0);
    }
}
