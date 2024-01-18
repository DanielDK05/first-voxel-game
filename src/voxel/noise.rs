use bevy::{app::Plugin, ecs::system::Resource};
use noise::{Fbm, NoiseFn, Simplex};
use rand::Rng;

use super::Voxel;

pub(super) struct VoxelTerrainNoisePlugin;

impl Plugin for VoxelTerrainNoisePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<TerrainNoise>();
    }
}

#[derive(Resource)]
pub(super) struct TerrainNoise(Fbm<Simplex>);

impl TerrainNoise {
    pub(super) fn rand() -> Self {
        let mut rng = rand::thread_rng();

        Self(Fbm::new(rng.gen::<u32>()))
    }

    pub(super) fn get_voxel(&self, x: i32, y: i32, z: i32) -> Voxel {
        let scalar = 0.01;
        let noise_value = self
            .0
            .get([x as f64 * scalar, y as f64 * scalar, z as f64 * scalar]);

        if noise_value < 0.0 {
            Voxel::STONE
        } else {
            Voxel::AIR
        }
    }
}

impl Default for TerrainNoise {
    fn default() -> Self {
        Self::rand()
    }
}
