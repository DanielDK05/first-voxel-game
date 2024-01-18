use bevy::math::{IVec3, Vec3};

pub(super) const DIRECT_CUBE_NEIGHBOURS: [IVec3; 6] = [
    IVec3 { x: 0, y: 1, z: 0 },
    IVec3 { x: 0, y: -1, z: 0 },
    IVec3 { x: -1, y: 0, z: 0 },
    IVec3 { x: 1, y: 0, z: 0 },
    IVec3 { x: 0, y: 0, z: -1 },
    IVec3 { x: 0, y: 0, z: 1 },
];

pub(super) enum CubeFace {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

impl CubeFace {
    pub(super) fn from_ivec3(vec3: IVec3) -> Self {
        match vec3 {
            IVec3 { x: 0, y: 1, z: 0 } => CubeFace::Top,
            IVec3 { x: 0, y: -1, z: 0 } => CubeFace::Bottom,
            IVec3 { x: -1, y: 0, z: 0 } => CubeFace::Left,
            IVec3 { x: 1, y: 0, z: 0 } => CubeFace::Right,
            IVec3 { x: 0, y: 0, z: -1 } => CubeFace::Front,
            IVec3 { x: 0, y: 0, z: 1 } => CubeFace::Back,
            _ => panic!("CubeFaces::from_ivec3 failed: invalid IVec3"),
        }
    }

    pub(super) fn normals(&self) -> Vec<Vec3> {
        match self {
            CubeFace::Top => vec![Vec3::new(0.0, 1.0, 0.0); 4],
            CubeFace::Bottom => vec![Vec3::new(0.0, -1.0, 0.0); 4],
            CubeFace::Left => vec![Vec3::new(-1.0, 0.0, 0.0); 4],
            CubeFace::Right => vec![Vec3::new(1.0, 0.0, 0.0); 4],
            CubeFace::Front => vec![Vec3::new(0.0, 0.0, 1.0); 4],
            CubeFace::Back => vec![Vec3::new(0.0, 0.0, -1.0); 4],
        }
    }

    pub(super) fn indices(&self, vertices_pushed: u32) -> Vec<u32> {
        // DO NOT TOUCH THESE INDICES PLEASE ON GOD
        // I SPENT LITERALLY 3 HOURS ON THESE F**KING NUMBERS
        let base_indices = match self {
            CubeFace::Top => vec![2, 0, 1, 1, 3, 2],
            CubeFace::Bottom => vec![3, 1, 0, 0, 2, 3],
            CubeFace::Left => vec![0, 1, 3, 3, 2, 0],
            CubeFace::Right => vec![1, 0, 2, 2, 3, 1],
            CubeFace::Front => vec![1, 0, 2, 2, 3, 1],
            CubeFace::Back => vec![0, 1, 3, 3, 2, 0],
        };

        base_indices
            .into_iter()
            .map(|index| index + vertices_pushed)
            .collect()
    }

    pub(super) fn vertices(&self) -> Vec<Vec3> {
        match self {
            CubeFace::Top => vec![
                CubeCorner::TopLeftFront.vertex(),
                CubeCorner::TopLeftBack.vertex(),
                CubeCorner::TopRightFront.vertex(),
                CubeCorner::TopRightBack.vertex(),
            ],
            CubeFace::Bottom => vec![
                CubeCorner::BottomLeftFront.vertex(),
                CubeCorner::BottomLeftBack.vertex(),
                CubeCorner::BottomRightFront.vertex(),
                CubeCorner::BottomRightBack.vertex(),
            ],
            CubeFace::Left => vec![
                CubeCorner::BottomLeftFront.vertex(),
                CubeCorner::BottomLeftBack.vertex(),
                CubeCorner::TopLeftFront.vertex(),
                CubeCorner::TopLeftBack.vertex(),
            ],
            CubeFace::Right => vec![
                CubeCorner::BottomRightFront.vertex(),
                CubeCorner::BottomRightBack.vertex(),
                CubeCorner::TopRightFront.vertex(),
                CubeCorner::TopRightBack.vertex(),
            ],
            CubeFace::Front => vec![
                CubeCorner::BottomLeftFront.vertex(),
                CubeCorner::BottomRightFront.vertex(),
                CubeCorner::TopLeftFront.vertex(),
                CubeCorner::TopRightFront.vertex(),
            ],
            CubeFace::Back => vec![
                CubeCorner::BottomLeftBack.vertex(),
                CubeCorner::BottomRightBack.vertex(),
                CubeCorner::TopLeftBack.vertex(),
                CubeCorner::TopRightBack.vertex(),
            ],
        }
    }
}

pub(super) enum CubeCorner {
    BottomLeftFront,
    BottomLeftBack,
    BottomRightFront,
    BottomRightBack,
    TopLeftFront,
    TopLeftBack,
    TopRightFront,
    TopRightBack,
}

impl CubeCorner {
    const VERTICES: [Vec3; 8] = [
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(0.5, 0.5, 0.5),
    ];

    fn vertex(&self) -> Vec3 {
        match self {
            CubeCorner::BottomLeftFront => Self::VERTICES[0],
            CubeCorner::BottomLeftBack => Self::VERTICES[1],
            CubeCorner::BottomRightFront => Self::VERTICES[2],
            CubeCorner::BottomRightBack => Self::VERTICES[3],
            CubeCorner::TopLeftFront => Self::VERTICES[4],
            CubeCorner::TopLeftBack => Self::VERTICES[5],
            CubeCorner::TopRightFront => Self::VERTICES[6],
            CubeCorner::TopRightBack => Self::VERTICES[7],
        }
    }
}
