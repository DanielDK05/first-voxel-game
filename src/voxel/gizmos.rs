use bevy::prelude::*;

const CHUNK_BORDER_COLOR: Color = Color::ORANGE;

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
pub(super) enum ChunkBorderState {
    Enabled,
    #[default]
    Disabled,
}

pub(super) struct VoxelGizmosPlugin;

impl Plugin for VoxelGizmosPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state::<ChunkBorderState>().add_systems(
            Update,
            (
                systems::toggle_chunk_borders,
                systems::chunk_borders.run_if(in_state(ChunkBorderState::Enabled)),
            ),
        );
    }
}

mod systems {
    use bevy::{gizmos::gizmos::Gizmos, prelude::*};

    use crate::voxel::{
        generation::{VoxelChunk, VoxelChunkPosition, VoxelChunkWidth},
        VoxelChunkCoordinate,
    };

    use super::{ChunkBorderState, CHUNK_BORDER_COLOR};

    pub(super) fn chunk_borders(
        mut gizmos: Gizmos,
        chunk_query: Query<&VoxelChunkPosition, With<VoxelChunk>>,
        chunk_width: Res<VoxelChunkWidth>,
    ) {
        for chunk_pos in &chunk_query {
            gizmos.cuboid(
                Transform::from_translation(chunk_pos.as_world_pos(&chunk_width) / 2.0 - 0.5)
                    .with_scale(Vec3::splat(chunk_width.0 as f32)),
                CHUNK_BORDER_COLOR,
            )
        }
    }

    pub(super) fn toggle_chunk_borders(
        input: Res<Input<KeyCode>>,
        mut next_state: ResMut<NextState<ChunkBorderState>>,
        cur_state: Res<State<ChunkBorderState>>,
    ) {
        if input.just_pressed(KeyCode::B) {
            next_state.set(match **cur_state {
                ChunkBorderState::Enabled => ChunkBorderState::Disabled,
                ChunkBorderState::Disabled => ChunkBorderState::Enabled,
            })
        }
    }
}
