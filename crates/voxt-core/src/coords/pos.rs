use std::{
    fmt::{Debug, Display},
    ops::RangeInclusive,
};

use num_traits::{AsPrimitive, CheckedAdd, CheckedSub, FromPrimitive};

use crate::prelude::Dir;

mod coord {
    pub trait Coord:
        num_traits::PrimInt + num_traits::AsPrimitive<isize> + num_traits::FromPrimitive
    {
    }

    impl Coord for u8 {}
    impl Coord for i16 {}
    impl Coord for i32 {}
}

pub trait Pos: Debug + Clone + Copy + PartialEq + Eq + Display {
    type T: coord::Coord;

    const MIN: Self::T;
    const MAX: Self::T;
    const RANGE: RangeInclusive<Self::T>;

    const MIN_HEIGHT: Self::T;
    const MAX_HEIGHT: Self::T;
    const RANGE_HEIGHT: RangeInclusive<Self::T>;

    fn from_raw(x: Self::T, y: Self::T, z: Self::T) -> Self;

    fn try_new(x: Self::T, y: Self::T, z: Self::T) -> Option<Self>
    where
        Self: Sized,
    {
        if Self::RANGE.contains(&x) && Self::RANGE_HEIGHT.contains(&y) && Self::RANGE.contains(&z) {
            Some(Self::from_raw(x, y, z))
        } else {
            None
        }
    }

    fn pos(&self) -> (Self::T, Self::T, Self::T);

    fn x(&self) -> Self::T {
        self.pos().0
    }

    fn y(&self) -> Self::T {
        self.pos().1
    }

    fn z(&self) -> Self::T {
        self.pos().2
    }

    fn neighbor(&self, dir: Dir) -> Option<Self>
    where
        Self: Sized,
    {
        let (dx, dy, dz) = dir.as_off_tuple();

        let rx: isize = self.x().as_() + dx as isize;
        let ry: isize = self.y().as_() + dy as isize;
        let rz: isize = self.z().as_() + dz as isize;

        let nx: Self::T = FromPrimitive::from_isize(rx)?;
        let ny: Self::T = FromPrimitive::from_isize(ry)?;
        let nz: Self::T = FromPrimitive::from_isize(rz)?;

        if Self::RANGE.contains(&nx)
            && Self::RANGE_HEIGHT.contains(&ny)
            && Self::RANGE.contains(&nz)
        {
            Some(Self::from_raw(nx, ny, nz))
        } else {
            None
        }
    }

    fn neighborhood(&self) -> [Option<Self>; 6]
    where
        Self: Sized,
    {
        Dir::LIST.map(|d| self.neighbor(d))
    }

    fn checked_add(&self, rhs: &Self) -> Option<Self>
    where
        Self: Sized,
    {
        let nx: Self::T = self.x().checked_add(&rhs.x())?;
        let ny: Self::T = self.y().checked_add(&rhs.y())?;
        let nz: Self::T = self.z().checked_add(&rhs.z())?;

        if Self::RANGE.contains(&nx)
            && Self::RANGE_HEIGHT.contains(&ny)
            && Self::RANGE.contains(&nz)
        {
            Some(Self::from_raw(nx, ny, nz))
        } else {
            None
        }
    }

    fn checked_sub(&self, rhs: &Self) -> Option<Self>
    where
        Self: Sized,
    {
        let nx: Self::T = self.x().checked_sub(&rhs.x())?;
        let ny: Self::T = self.y().checked_sub(&rhs.y())?;
        let nz: Self::T = self.z().checked_sub(&rhs.z())?;

        if Self::RANGE.contains(&nx)
            && Self::RANGE_HEIGHT.contains(&ny)
            && Self::RANGE.contains(&nz)
        {
            Some(Self::from_raw(nx, ny, nz))
        } else {
            None
        }
    }

    fn dist_euclid_2(&self, rhs: &Self) -> usize {
        let dx: isize = self.x().as_() - rhs.x().as_();
        let dy: isize = self.y().as_() - rhs.y().as_();
        let dz: isize = self.z().as_() - rhs.z().as_();

        (dx * dx) as usize + (dy * dy) as usize + (dz * dz) as usize
    }

    fn dist_manhattan(&self, rhs: &Self) -> usize {
        let dx: isize = self.x().as_() - rhs.x().as_();
        let dy: isize = self.y().as_() - rhs.y().as_();
        let dz: isize = self.z().as_() - rhs.z().as_();

        dx.unsigned_abs() + dy.unsigned_abs() + dz.unsigned_abs()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        constants::CHUNK_SIZE,
        coords::{ChunkPos, VoxelPos, WorldPos, pos::Pos},
    };

    #[test]
    fn from_raw_preserves_coordinates() {
        let pos = WorldPos::from_raw(1, -2, 3);

        assert_eq!(pos.pos(), (1, -2, 3));
        assert_eq!(pos.x(), 1);
        assert_eq!(pos.y(), -2);
        assert_eq!(pos.z(), 3);
    }

    #[test]
    fn tuple_conversion_preserves_coordinates() {
        let pos = WorldPos::from((1, 2, 3));

        assert_eq!(pos.pos(), (1, 2, 3));
    }

    #[test]
    fn array_conversion_preserves_coordinates() {
        let pos = WorldPos::from([1, 2, 3]);

        assert_eq!(pos.pos(), (1, 2, 3));
    }

    #[test]
    fn converts_to_tuple() {
        let pos = WorldPos::from_raw(1, 2, 3);

        let tuple: (i32, i32, i32) = pos.into();

        assert_eq!(tuple, (1, 2, 3));
    }

    #[test]
    fn converts_to_array() {
        let pos = WorldPos::from_raw(1, 2, 3);

        let array: [i32; 3] = pos.into();

        assert_eq!(array, [1, 2, 3]);
    }

    fn takes_chunk_pos(_: ChunkPos) {}

    fn takes_world_voxel_pos(_: WorldPos) {}

    #[test]
    fn position_aliases_are_usable_as_distinct_types() {
        let chunk = ChunkPos::from_raw(1, 2, 3);
        let world = WorldPos::from_raw(1, 2, 3);

        takes_chunk_pos(chunk);
        takes_world_voxel_pos(world);
    }

    #[test]
    fn try_new_accepts_valid_bounds() {
        assert!(VoxelPos::try_new(0, 0, 0).is_some());
        assert!(VoxelPos::try_new(31, 31, 31).is_some());
    }

    #[test]
    fn try_new_accepts_interior_coordinates() {
        let pos = VoxelPos::try_new(1, 2, 3).unwrap();

        assert_eq!(pos.pos(), (1, 2, 3));
    }

    #[test]
    fn try_new_rejects_invalid_x() {
        assert!(VoxelPos::try_new(32, 0, 0).is_none());
    }

    #[test]
    fn try_new_rejects_invalid_y() {
        assert!(VoxelPos::try_new(0, 32, 0).is_none());
    }

    #[test]
    fn try_new_rejects_invalid_z() {
        assert!(VoxelPos::try_new(0, 0, 32).is_none());
    }

    #[test]
    fn try_new_rejects_invalid_coordinates() {
        assert!(VoxelPos::try_new(32, 32, 32).is_none());
    }

    #[test]
    fn try_new_rejects_values_above_chunk_bounds() {
        assert!(VoxelPos::try_new(u8::MAX, 0, 0).is_none());
        assert!(VoxelPos::try_new(0, u8::MAX, 0).is_none());
        assert!(VoxelPos::try_new(0, 0, u8::MAX).is_none());
    }

    #[test]
    fn from_raw_accepts_valid_coordinates() {
        let pos = VoxelPos::from_raw(31, 10, 0);

        assert_eq!(pos.pos(), (31, 10, 0));
    }

    #[test]
    fn world_origin_decomposes_correctly() {
        let world = WorldPos::from_raw(0, 0, 0);

        assert_eq!(ChunkPos::from(&world), ChunkPos::from_raw(0, 0, 0));
        assert_eq!(VoxelPos::from(&world), VoxelPos::from_raw(0, 0, 0));
    }

    #[test]
    fn last_voxel_of_first_chunk_decomposes_correctly() {
        let world = WorldPos::from_raw(31, 31, 31);

        assert_eq!(ChunkPos::from(&world), ChunkPos::from_raw(0, 0, 0));
        assert_eq!(VoxelPos::from(&world), VoxelPos::from_raw(31, 31, 31));
    }

    #[test]
    fn first_voxel_of_next_chunk_decomposes_correctly() {
        let world = WorldPos::from_raw(32, 32, 32);

        assert_eq!(ChunkPos::from(&world), ChunkPos::from_raw(1, 1, 1));
        assert_eq!(VoxelPos::from(&world), VoxelPos::from_raw(0, 0, 0));
    }

    #[test]
    fn negative_one_decomposes_to_previous_chunk_last_voxel() {
        let world = WorldPos::from_raw(-1, -1, -1);

        assert_eq!(ChunkPos::from(&world), ChunkPos::from_raw(-1, -1, -1));
        assert_eq!(VoxelPos::from(&world), VoxelPos::from_raw(31, 31, 31));
    }

    #[test]
    fn negative_chunk_boundary_decomposes_correctly() {
        let world = WorldPos::from_raw(-32, -32, -32);

        assert_eq!(ChunkPos::from(&world), ChunkPos::from_raw(-1, -1, -1));
        assert_eq!(VoxelPos::from(&world), VoxelPos::from_raw(0, 0, 0));
    }

    #[test]
    fn negative_coordinate_just_beyond_chunk_boundary_decomposes_correctly() {
        let world = WorldPos::from_raw(-33, -33, -33);

        assert_eq!(ChunkPos::from(&world), ChunkPos::from_raw(-2, -2, -2));
        assert_eq!(VoxelPos::from(&world), VoxelPos::from_raw(31, 31, 31));
    }

    #[test]
    fn mixed_sign_coordinates_decompose_independently() {
        let world = WorldPos::from_raw(-1, 0, 32);

        assert_eq!(ChunkPos::from(&world), ChunkPos::from_raw(-1, 0, 1));
        assert_eq!(VoxelPos::from(&world), VoxelPos::from_raw(31, 0, 0));
    }

    #[test]
    fn coordinate_decomposition_round_trips() {
        let cases = [
            (0, 0, 0),
            (1, 2, 3),
            (31, 31, 31),
            (32, 32, 32),
            (33, 47, 33),
            (-1, -1, -1),
            (-32, -32, -32),
            (-33, -33, -33),
            (-33, 42, 97),
        ];

        for (x, y, z) in cases {
            let original = WorldPos::from_raw(x, y, z);

            let chunk = ChunkPos::from(&original);
            let voxel = VoxelPos::from(&original);

            let reconstructed = WorldPos::from((chunk, voxel));

            assert_eq!(reconstructed, original, "round-trip failed for {original}");
        }
    }

    #[test]
    fn world_conversion_always_produces_valid_voxel_positions() {
        for x in -100..=100 {
            for y in -100..=100 {
                for z in -100..=100 {
                    let world = WorldPos::from_raw(x, y, z);
                    let voxel = VoxelPos::from(world);

                    assert!(voxel.x() < CHUNK_SIZE as u8);
                    assert!(voxel.y() < CHUNK_SIZE as u8);
                    assert!(voxel.z() < CHUNK_SIZE as u8);
                }
            }
        }
    }

    #[test]
    fn chunk_pos_is_suitable_as_hash_map_key() {
        let mut chunks = std::collections::HashMap::new();

        chunks.insert(ChunkPos::from_raw(-4, 7, 12), "chunk");

        assert_eq!(chunks.get(&ChunkPos::from_raw(-4, 7, 12)), Some(&"chunk"));
    }
}
