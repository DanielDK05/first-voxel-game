use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::hashbrown::HashMap,
};
use rayon::prelude::*;

use crate::voxel::cube_mesh::CubeFace;

use super::{
    cube_mesh::DIRECT_CUBE_NEIGHBOURS, load::VoxelChunkLoadingPlugin, noise::TerrainNoise, Voxel,
    VoxelChunkCoordinate,
};

/// Default value for [VoxelChunkWidth].
const DEFAULT_CHUNK_WIDTH: u8 = 16;

/// This is the plugin responsible for voxel terrain generation (like the name implies :D)
pub(super) struct VoxelTerrainGeneratorPlugin;

impl Plugin for VoxelTerrainGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VoxelChunkLoadingPlugin)
            .init_resource::<VoxelChunkWidth>()
            .init_resource::<VoxelChunkMap>();
    }
}

/// This struct represents a voxel position, local to it's chunk.
/// Because of this, the complete world position cannot be computed without a [VoxelChunkPosition].
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(super) struct LocalVoxelPosition {
    x: u8,
    y: u8,
    z: u8,
}

impl LocalVoxelPosition {
    fn new(x: u8, y: u8, z: u8) -> Self {
        Self { x, y, z }
    }

    /// Calculates a [LocalVoxelPosition] from a given index in the 3-dimensional flat voxel array [VoxelChunk].voxels.
    /// This is calculated based on the chunk width.
    pub(super) fn from_index(index: usize, chunk_width: &VoxelChunkWidth) -> Self {
        let cw = chunk_width.0 as u32;

        let x = index as u32 % cw;
        let y = (index as u32 / cw) % cw;
        let z = index as u32 / (cw * cw);

        Self::new(x as u8, y as u8, z as u8)
    }

    /// Calculates the index in the 3-dimensional flat voxel array [VoxelChunk].voxels based on the [LocalVoxelPosition]
    pub(super) fn to_index(&self, chunk_width: &VoxelChunkWidth) -> usize {
        let index = self.z as usize * chunk_width.0 as usize * chunk_width.0 as usize
            + self.y as usize * chunk_width.0 as usize
            + self.x as usize;

        index
    }
}

/// A HashMap containing all the [VoxelChunk]s currently spawned.
/// Keyed by the [VoxelChunkPosition] of a chunk, and the value is the entity id.
#[derive(Resource, Default, Debug)]
pub(super) struct VoxelChunkMap(pub(super) HashMap<VoxelChunkPosition, Entity>);

impl VoxelChunkMap {
    /// Inserts a new chunk to the map.
    ///
    /// If the chunk already exists, it returns an error.
    pub(super) fn insert_chunk(
        &mut self,
        chunk_position: VoxelChunkPosition,
        entity: Entity,
    ) -> Result<(), ()> {
        if self.0.contains_key(&chunk_position) {
            Err(()) //TODO: fix error type
        } else {
            self.0.insert(chunk_position, entity);
            Ok(())
        }
    }

    /// Gets a specific voxel from the map
    fn get_voxel(
        &self,
        chunk_position: &VoxelChunkPosition,
        local_voxel_position: &LocalVoxelPosition,
        chunk_width: &VoxelChunkWidth,
        voxel_chunk_query: &Query<&VoxelChunk>,
    ) -> Option<Voxel> {
        let Some(chunk_entity) = self.0.get(chunk_position) else {
            return None;
        };

        let Ok(chunk) = voxel_chunk_query.get(*chunk_entity) else {
            return None;
        };

        chunk
            .voxels
            .get(local_voxel_position.to_index(chunk_width))
            .and_then(|v| Some(*v))
    }
}

/// Decorative struct that represents a chunk position as an [IVec3].
/// This is also a component used in [VoxelChunkBundle]
#[derive(Component, Default, Debug, Eq, PartialEq, Hash, Copy, Clone, Reflect)]
pub(super) struct VoxelChunkPosition(pub(super) IVec3);

impl VoxelChunkPosition {
    pub(super) fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }
}

impl VoxelChunkCoordinate for VoxelChunkPosition {
    fn from_world_pos(world_pos: Vec3, chunk_width: &VoxelChunkWidth) -> Self {
        VoxelChunkPosition::new(world_pos.x as i32, world_pos.y as i32, world_pos.z as i32)
            / chunk_width.0 as i32
    }

    fn from_chunk_pos(chunk_pos: &VoxelChunkPosition, _chunk_width: &VoxelChunkWidth) -> Self {
        *chunk_pos
    }

    fn as_world_pos(&self, chunk_width: &VoxelChunkWidth) -> Vec3 {
        Vec3::from_chunk_pos(self, chunk_width)
    }

    fn as_chunk_pos(&self, _chunk_width: &VoxelChunkWidth) -> VoxelChunkPosition {
        *self
    }
}

impl std::ops::Div<i32> for VoxelChunkPosition {
    type Output = VoxelChunkPosition;

    fn div(self, rhs: i32) -> Self::Output {
        Self::new(self.0.x / rhs, self.0.y / rhs, self.0.z / rhs)
    }
}

impl std::ops::Mul<i32> for VoxelChunkPosition {
    type Output = VoxelChunkPosition;

    fn mul(self, rhs: i32) -> Self::Output {
        Self::new(self.0.x * rhs, self.0.y * rhs, self.0.z * rhs)
    }
}

impl<'a> std::ops::Mul<i32> for &'a VoxelChunkPosition {
    type Output = VoxelChunkPosition;

    fn mul(self, rhs: i32) -> VoxelChunkPosition {
        VoxelChunkPosition::new(self.0.x * rhs, self.0.y * rhs, self.0.z * rhs)
    }
}

impl std::ops::Sub<VoxelChunkPosition> for VoxelChunkPosition {
    type Output = VoxelChunkPosition;

    fn sub(self, rhs: VoxelChunkPosition) -> Self::Output {
        Self::new(self.0.x - rhs.0.x, self.0.y - rhs.0.y, self.0.z - rhs.0.z)
    }
}

/// Resource representing how many voxels wide a chunk is.
#[derive(Resource)]
pub(super) struct VoxelChunkWidth(pub(super) u8);

impl Default for VoxelChunkWidth {
    fn default() -> Self {
        Self(DEFAULT_CHUNK_WIDTH)
    }
}

/// The voxel chunk component.
#[derive(Component, Default, Clone)]
pub(super) struct VoxelChunk {
    /// A 3 dimensional flat vector of all the voxels. Refer to [LocalVoxelPosition]'s methods to
    /// find a specific voxel inside the vector.
    voxels: Vec<Voxel>,
}

impl VoxelChunk {
    pub(super) fn from_noise(
        chunk_pos: &VoxelChunkPosition,
        chunk_width: &VoxelChunkWidth,
        terrain_noise: &TerrainNoise,
    ) -> Self {
        let range_size = chunk_width.0 as usize * chunk_width.0 as usize * chunk_width.0 as usize;
        let voxels = std::sync::Mutex::new(vec![Voxel::AIR; range_size]);

        (0..range_size).into_par_iter().for_each(|i| {
            let position = LocalVoxelPosition::from_index(i, chunk_width);

            let voxel = terrain_noise.get_voxel(
                chunk_pos.0.x * chunk_width.0 as i32 + position.x as i32,
                chunk_pos.0.y * chunk_width.0 as i32 + position.y as i32,
                chunk_pos.0.z * chunk_width.0 as i32 + position.z as i32,
            );

            loop {
                if let Ok(mut voxels) = voxels.try_lock() {
                    voxels[i] = voxel;
                    break;
                }
            }
        });

        let voxels = voxels.into_inner().unwrap();
        Self { voxels }
    }

    pub(super) fn generate_mesh(
        &self,
        chunk_width: &VoxelChunkWidth,
        voxel_map: &VoxelChunkMap,
        voxel_chunk_query: &Query<&VoxelChunk>,
    ) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut vertices_pushed = 0;

        for (i, voxel) in self.voxels.iter().enumerate() {
            if !voxel.is_solid() {
                continue;
            }

            let local_voxel_pos = LocalVoxelPosition::from_index(i, &chunk_width);

            let mut faces = Vec::new();

            for neighbour in DIRECT_CUBE_NEIGHBOURS {
                let Some(x) = local_voxel_pos.x.checked_add_signed(neighbour.x as i8) else {
                    continue;
                };
                let Some(y) = local_voxel_pos.y.checked_add_signed(neighbour.y as i8) else {
                    continue;
                };
                let Some(z) = local_voxel_pos.z.checked_add_signed(neighbour.z as i8) else {
                    continue;
                };

                let face = CubeFace::from_ivec3(neighbour);

                // This looks kind of weird, but it's simply like this:
                // - if there is a neighbour, and the neighbour isn't a solid voxel, render face. if there is no neighbour, render face.
                if let Some(voxel) = voxel_map.get_voxel(
                    &VoxelChunkPosition::new(0, 0, 0),
                    &LocalVoxelPosition::new(x, y, z),
                    &chunk_width,
                    &voxel_chunk_query,
                ) {
                    if !voxel.is_solid() {
                        faces.push(face);
                    }
                } else {
                    faces.push(face);
                }
            }

            for face in faces {
                for index in face.indices(vertices_pushed) {
                    indices.push(index);
                }

                for vertex in face.vertices() {
                    let vertex_pos = Vec3::new(
                        local_voxel_pos.x as f32,
                        local_voxel_pos.y as f32,
                        local_voxel_pos.z as f32,
                    ) + vertex;

                    vertices.push(vertex_pos);
                    vertices_pushed += 1;
                }

                for normal in face.normals() {
                    normals.push(normal);
                }
            }
        }

        Mesh::new(PrimitiveTopology::TriangleList)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_indices(Some(Indices::U32(indices)))
    }
}

/// This is the bundle used for a voxel chunk. This is used when spawning in chunks.
#[derive(Bundle, Default)]
pub(super) struct VoxelChunkBundle {
    pub(super) visibility: Visibility,
    pub(super) inherited_visibility: InheritedVisibility,
    pub(super) view_visibility: ViewVisibility,
    pub(super) transform: Transform,
    pub(super) global_transform: GlobalTransform,
    pub(super) mesh: Handle<Mesh>,
    pub(super) material: Handle<StandardMaterial>,
    pub(super) chunk: VoxelChunk,
    pub(super) chunk_pos: VoxelChunkPosition,
}
