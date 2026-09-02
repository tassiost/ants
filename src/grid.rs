use crate::utils::calc_weighted_midpoint;
use crate::*;
use bevy::prelude::*;
use kd_tree::{KdTree, KdPoint};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridKey(pub i32, pub i32);

impl KdPoint for GridKey {
    type Scalar = f32;
    type Dim = typenum::U2;

    fn at(&self, i: usize) -> Self::Scalar {
        match i {
            0 => self.0 as f32,
            1 => self.1 as f32,
            _ => panic!("GridKey is 2D"),
        }
    }
}

#[derive(Resource)]
pub struct DecayGrid {
    pub values: HashMap<GridKey, f32>,
    pub max_value: f32,
}

impl DecayGrid {
    pub fn new(values: HashMap<GridKey, f32>, max_value: f32) -> Self {
        Self {
            values,
            max_value,
        }
    }

    pub fn emit_signal(&mut self, key: &GridKey, value: f32) {
        if key.0 == 0 && key.1 == 0 {
            return;
        }
        let entry = self.values.entry(*key).or_insert(0.0);
        *entry = (*entry + value).min(self.max_value);
    }

    pub fn decay_values(&mut self, rate: f32) {
        for value in self.values.values_mut() {
            *value = (*value - rate).max(0.0);
        }
    }

    pub fn drop_zero_values(&mut self) {
        self.values.retain(|_, v| *v > 0.0);
    }

    pub fn get_values(&self) -> &HashMap<GridKey, f32> {
        &self.values
    }

    pub fn add_value(&mut self, key: &GridKey, value: f32, max_value: f32) {
        let entry = self.values.entry(*key).or_insert(0.0);
        *entry = (*entry + value).min(max_value);
    }
}

#[derive(Resource)]
pub struct WorldGrid {
    pub signals: DecayGrid,
    pub color: (u8, u8, u8),
    pub tree: Option<KdTree<GridKey>>,
    pub steer_cache: HashMap<GridKey, Vec2>,
}

impl WorldGrid {
    pub fn new(color: (u8, u8, u8), signals: HashMap<GridKey, f32>) -> Self {
        Self {
            signals: DecayGrid::new(signals, MAX_PHEROMONE_STRENGTH),
            color,
            tree: None,
            steer_cache: HashMap::new(),
        }
    }

    pub fn emit_signal(&mut self, key: &GridKey, value: f32) {
        self.signals.emit_signal(key, value);
    }

    pub fn decay_signals(&mut self) {
        self.signals.decay_values(PH_DECAY_RATE);
    }

    pub fn drop_zero_signals(&mut self) {
        self.signals.drop_zero_values();
    }

    pub fn get_signals(&self) -> &HashMap<GridKey, f32> {
        self.signals.get_values()
    }

    pub fn get_signals_size(&self) -> usize {
        self.signals.values.len()
    }

    pub fn update_tree(&mut self) {
        let points: Vec<GridKey> = self.signals.values.keys().copied().collect();
        if points.is_empty() {
            self.tree = None;
        } else {
            self.tree = Some(KdTree::build_by_ordered_float(points));
        }
    }

    pub fn get_ph_in_range(&self, center: &Vec2, radius: f32) -> Vec<(GridKey, f32)> {
        let tree = match &self.tree {
            Some(tree) => tree,
            None => return Vec::new(),
        };

        let query = [center.x as f32, center.y as f32];
        let mut results = Vec::new();
        for point in tree.within_radius(&query, radius) {
            let key = GridKey(point.at(0) as i32, point.at(1) as i32);
            if let Some(value) = self.signals.values.get(&key) {
                results.push((key, *value));
            }
        }

        results
    }

    pub fn get_steer_target(&mut self, pos: &Vec2, radius: f32) -> Option<Vec2> {
        let cache_key = GridKey(
            (pos.x / PH_CACHE_GRID_SIZE as f32).floor() as i32,
            (pos.y / PH_CACHE_GRID_SIZE as f32).floor() as i32,
        );

        if let Some(cached) = self.steer_cache.get(&cache_key) {
            return Some(*cached);
        }

        let points = self.get_ph_in_range(pos, radius);
        if points.is_empty() {
            return None;
        }

        let weighted = points
            .iter()
            .map(|(k, v)| (k.0 as f32, k.1 as f32, *v))
            .collect::<Vec<_>>();
        let target = calc_weighted_midpoint(&weighted);
        self.steer_cache.insert(cache_key, target);
        Some(target)
    }

    pub fn clear_steer_cache(&mut self) -> u32 {
        let size = self.steer_cache.len() as u32;
        self.steer_cache.clear();
        size
    }
}

pub fn add_map_to_grid_img(
    signals: &HashMap<GridKey, f32>,
    color: &(u8, u8, u8),
    bytes: &mut [u8],
    use_opacity: bool,
) {
    for (key, value) in signals {
        let x = key.0;
        let y = key.1;
        let px = x as usize;
        let py = y as usize;

        let intensity = if *value > MAX_PHEROMONE_STRENGTH / 2.0 {
            PH_GRID_OPACITY
        } else {
            PH_GRID_VIZ_MIN_STRENGTH
        };

        let idx = (py * (W as usize / PH_UNIT_GRID_SIZE) + px) * 4;
        if idx + 3 < bytes.len() {
            bytes[idx] = color.0;
            bytes[idx + 1] = color.1;
            bytes[idx + 2] = color.2;
            bytes[idx + 3] = if use_opacity { intensity } else { PH_GRID_OPACITY };
        }
    }
}
