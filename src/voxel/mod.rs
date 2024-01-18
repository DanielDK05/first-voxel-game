mod cube_mesh;
mod generation;
mod gizmos;
pub(crate) mod load;
mod noise;

use bevy::{app::Plugin, math::Vec3};

use self::{
    generation::{VoxelChunkPosition, VoxelChunkWidth, VoxelTerrainGeneratorPlugin},
    gizmos::VoxelGizmosPlugin,
    noise::VoxelTerrainNoisePlugin,
};

pub(crate) struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            VoxelTerrainGeneratorPlugin,
            VoxelTerrainNoisePlugin,
            VoxelGizmosPlugin,
        ));
    }
}

#[derive(Clone, Copy, Debug)]
struct Voxel {
    id: u16,
}

impl Voxel {
    const AIR: Self = Self::new(0);
    const STONE: Self = Self::new(1);

    const fn new(id: u16) -> Self {
        Self { id }
    }

    fn is_solid(&self) -> bool {
        self.id != Self::AIR.id
    }
}

impl Default for Voxel {
    fn default() -> Self {
        Voxel::AIR
    }
}

/// Anything that implements this trait, is something that can be represented as a voxel chunk coordinate.
trait VoxelChunkCoordinate {
    fn from_world_pos(world_pos: Vec3, chunk_width: &VoxelChunkWidth) -> Self;
    fn from_chunk_pos(chunk_pos: &VoxelChunkPosition, chunk_width: &VoxelChunkWidth) -> Self;
    fn as_world_pos(&self, chunk_width: &VoxelChunkWidth) -> Vec3;
    fn as_chunk_pos(&self, chunk_width: &VoxelChunkWidth) -> VoxelChunkPosition;
}

impl VoxelChunkCoordinate for Vec3 {
    fn from_world_pos(world_pos: Vec3, _chunk_width: &VoxelChunkWidth) -> Self {
        world_pos
    }

    fn from_chunk_pos(chunk_pos: &VoxelChunkPosition, chunk_width: &VoxelChunkWidth) -> Self {
        Self::new(
            chunk_pos.0.x as f32,
            chunk_pos.0.y as f32,
            chunk_pos.0.z as f32,
        ) * chunk_width.0 as f32
    }

    fn as_world_pos(&self, _chunk_width: &VoxelChunkWidth) -> Vec3 {
        *self
    }

    fn as_chunk_pos(&self, chunk_width: &VoxelChunkWidth) -> VoxelChunkPosition {
        VoxelChunkPosition::from_world_pos(*self, chunk_width)
    }
}
