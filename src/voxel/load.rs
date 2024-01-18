use std::collections::VecDeque;

use bevy::prelude::*;

use super::generation::{
    VoxelChunk, VoxelChunkBundle, VoxelChunkMap, VoxelChunkPosition, VoxelChunkWidth,
};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

pub(super) struct VoxelChunkLoadingPlugin;

impl Plugin for VoxelChunkLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkRenderQueue>()
            .init_resource::<ChunkLoadQueue>()
            .register_type::<ChunkRenderQueue>()
            .register_type::<ChunkLoadQueue>()
            .add_plugins((
                ResourceInspectorPlugin::<ChunkRenderQueue>::default(),
                ResourceInspectorPlugin::<ChunkLoadQueue>::default(),
            ))
            .add_systems(
                Update,
                (
                    systems::enqueue_chunks_in_render_distance,
                    systems::unload_chunks_out_of_render_distance,
                    systems::handle_chunk_unloading,
                    systems::handle_chunk_loading,
                    systems::handle_chunk_rendering,
                )
                    .chain(),
            );
    }
}

#[derive(Component)]
pub(crate) struct RenderDistance {
    pub(crate) val: u32,
    pub(crate) unload_margin: u32,
}

impl RenderDistance {
    pub(crate) fn new(val: u32, unload_margin: u32) -> Self {
        Self { val, unload_margin }
    }
}

/// This is the queue responsible for loading in voxel chunk entities.
///
/// It should be noted that chunks are just loaded in as entitites, but are not rendered.
/// Rendering is handled by [ChunkRenderQueue]
#[derive(Resource, Default, Clone, Reflect)]
pub(super) struct ChunkLoadQueue {
    /// Chunks to be loaded.
    load: VecDeque<VoxelChunkPosition>,
    /// Chunks to be unloaded.
    unload: VecDeque<(VoxelChunkPosition, Entity)>,
}

pub(super) enum ChunkLoadQueueInput {
    Load(VoxelChunkPosition),
    Unload((VoxelChunkPosition, Entity)),
}

impl ChunkLoadQueue {
    pub(super) fn push_chunk(&mut self, input: ChunkLoadQueueInput) {
        match input {
            ChunkLoadQueueInput::Load(pos) => self.load.push_back(pos),
            ChunkLoadQueueInput::Unload((chunk_pos, entity)) => {
                self.unload.push_back((chunk_pos, entity))
            }
        }
    }
}

/// This is the queue responsible for rendering chunks / creating the meshes.
#[derive(Resource, Default, Reflect)]
pub(super) struct ChunkRenderQueue {
    /// Chunks to be rendered.
    queue: VecDeque<Entity>,
}

impl ChunkRenderQueue {
    pub(super) fn push_chunk(&mut self, entity: Entity) {
        self.queue.push_back(entity);
    }
}

mod systems {
    use crate::voxel::{noise::TerrainNoise, VoxelChunkCoordinate};

    use super::*;

    pub(super) fn enqueue_chunks_in_render_distance(
        render_dist_query: Query<(&Transform, &RenderDistance)>,
        chunk_width: Res<VoxelChunkWidth>,
        mut chunk_load_queue: ResMut<ChunkLoadQueue>,
        voxel_chunk_map: Res<VoxelChunkMap>,
    ) {
        for (transform, render_distance) in render_dist_query.iter() {
            let origin_chunk_pos = transform.translation.as_chunk_pos(&chunk_width);
            let min_bound = origin_chunk_pos.0 - render_distance.val as i32;
            let max_bound = origin_chunk_pos.0 + render_distance.val as i32;

            for x in min_bound.x..=max_bound.x {
                for y in min_bound.y..=max_bound.y {
                    for z in min_bound.z..=max_bound.z {
                        let chunk_pos = &VoxelChunkPosition::new(x, y, z);

                        if voxel_chunk_map.0.contains_key(chunk_pos)
                            || chunk_load_queue.load.contains(chunk_pos)
                        {
                            continue;
                        }

                        let distance = (*chunk_pos - origin_chunk_pos).0.abs();

                        if distance.as_vec3().length() <= render_distance.val as f32 {
                            chunk_load_queue.push_chunk(ChunkLoadQueueInput::Load(*chunk_pos));
                        }
                    }
                }
            }
        }
    }

    pub(super) fn unload_chunks_out_of_render_distance(
        render_dist_query: Query<(&Transform, &RenderDistance)>,
        chunk_width: Res<VoxelChunkWidth>,
        mut chunk_load_queue: ResMut<ChunkLoadQueue>,
        voxel_chunk_map: Res<VoxelChunkMap>,
    ) {
        for (chunk_pos, entity) in voxel_chunk_map.0.iter() {
            if render_dist_query
                .iter()
                .all(|(transform, render_distance)| {
                    let origin_chunk_pos = transform.translation.as_chunk_pos(&chunk_width);

                    let distance = (*chunk_pos - origin_chunk_pos).0.abs();

                    distance.as_vec3().length()
                        > (render_distance.val + render_distance.unload_margin) as f32
                })
            {
                chunk_load_queue.push_chunk(ChunkLoadQueueInput::Unload((*chunk_pos, *entity)));
            }
        }
    }

    /// This system is responsible for empyting the [ChunkLoadQueue] resource, by loading in chunks.
    pub(super) fn handle_chunk_loading(
        mut commands: Commands,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut chunk_load_queue: ResMut<ChunkLoadQueue>,
        mut chunk_render_queue: ResMut<ChunkRenderQueue>,
        mut voxel_map: ResMut<VoxelChunkMap>,
        chunk_width: Res<VoxelChunkWidth>,
        terrain_noise: Res<TerrainNoise>,
    ) {
        loop {
            // TODO: this could lead to performance issues. Needs to be changed to something where it loads a variable
            // amount of chunks every frame, instead of ALL of them.
            let Some(chunk_pos) = chunk_load_queue.load.front() else {
                break;
            };

            let chunk = VoxelChunk::from_noise(chunk_pos, &chunk_width, &terrain_noise);

            let chunk_entity = commands
                .spawn(VoxelChunkBundle {
                    transform: Transform::from_translation(chunk_pos.as_world_pos(&chunk_width)),
                    material: materials.add(Color::GREEN.into()),
                    chunk,
                    chunk_pos: *chunk_pos,
                    ..default()
                })
                .id();

            if let Err(_) = voxel_map.insert_chunk(*chunk_pos, chunk_entity) {
                commands.entity(chunk_entity).despawn();
                break;
            }

            chunk_render_queue.push_chunk(chunk_entity);

            chunk_load_queue.load.pop_front();
        }
    }

    pub(super) fn handle_chunk_unloading(
        mut commands: Commands,
        mut chunk_load_queue: ResMut<ChunkLoadQueue>,
        mut voxel_chunk_map: ResMut<VoxelChunkMap>,
    ) {
        loop {
            let Some((chunk_pos, chunk_entity)) = chunk_load_queue.unload.front() else {
                break;
            };

            let Some(entity_commands) = commands.get_entity(*chunk_entity) else {
                break;
            };

            entity_commands.despawn_recursive();
            voxel_chunk_map.0.remove(chunk_pos);
            chunk_load_queue.unload.pop_front();
        }
    }

    pub(super) fn handle_chunk_rendering(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut chunk_render_queue: ResMut<ChunkRenderQueue>,
        chunk_width: Res<VoxelChunkWidth>,
        chunk_query: Query<&VoxelChunk>,
        voxel_chunk_map: Res<VoxelChunkMap>,
    ) {
        loop {
            let Some(chunk_entity) = chunk_render_queue.queue.front() else {
                break;
            };
            let Ok(chunk) = chunk_query.get(*chunk_entity) else {
                break;
            };

            let mesh = chunk.generate_mesh(&chunk_width, &voxel_chunk_map, &chunk_query);

            if let Some(mut chunk_commands) = commands.get_entity(*chunk_entity) {
                chunk_commands.insert(meshes.add(mesh));
            } else {
                break;
            };

            chunk_render_queue.queue.pop_front();
        }
    }
}
