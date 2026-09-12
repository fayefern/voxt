#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlatIdx(u16);

impl FlatIdx {
    #[inline]
    pub const fn idx(&self) -> u16 {
        self.0
    }

    #[inline]
    pub const fn encode(x: u8, y: u8, z: u8) -> Self {
        Self(x as u16 + y as u16 * 32 + z as u16 * 1024)
    }

    #[inline]
    pub const fn decode(&self) -> (u8, u8, u8) {
        (
            (self.0 & 0x1F) as u8,
            ((self.0 >> 5) & 0x1F) as u8,
            ((self.0 >> 10) & 0x1F) as u8,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MortonIdx(u16);

impl MortonIdx {
    #[inline]
    pub const fn idx(&self) -> u16 {
        self.0
    }

    #[inline]
    const fn spread(mut bits: u32) -> u32 {
        bits &= 0x1F;
        bits = (bits | (bits << 8)) & 0x100F;
        bits = (bits | (bits << 4)) & 0x10C3;

        (bits | (bits << 2)) & 0x1249
    }

    #[inline]
    pub const fn encode(x: u8, y: u8, z: u8) -> Self {
        let mx = Self::spread(x as u32);
        let my = Self::spread(y as u32);
        let mz = Self::spread(z as u32);

        Self((mx | (my << 1) | (mz << 2)) as u16)
    }

    #[inline]
    const fn compact(mut bits: u32) -> u32 {
        bits &= 0x1249;
        bits = (bits | (bits >> 2)) & 0x10C3;
        bits = (bits | (bits >> 4)) & 0x100F;

        (bits | (bits >> 8)) & 0x001F
    }

    #[inline]
    pub const fn decode(&self) -> (u8, u8, u8) {
        let morton = self.0 as u32;

        let x = Self::compact(morton) as u8;
        let y = Self::compact(morton >> 1) as u8;
        let z = Self::compact(morton >> 2) as u8;

        (x, y, z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HilbertLinearIdx(u16);

impl HilbertLinearIdx {
    #[inline]
    pub const fn idx(&self) -> u16 {
        self.0
    }

    #[inline]
    pub const fn encode(x: u8, y: u8, z: u8) -> Self {
        let mut ux = x;
        let mut uz = z;
        let mut idx = 0;

        let mut bit = 4;
        loop {
            let rx = ((ux >> bit) & 1) as u32;
            let rz = ((uz >> bit) & 1) as u32;

            idx = (idx << 2) | ((3 * rx) ^ rz);

            if rz == 0 {
                if rx == 1 {
                    ux ^= 31;
                    uz ^= 31;
                }
                std::mem::swap(&mut ux, &mut uz);
            }
            if bit == 0 {
                break;
            }
            bit -= 1;
        }
        Self((y as u32 + (idx << 5)) as u16)
    }

    #[inline]
    pub const fn decode(&self) -> (u8, u8, u8) {
        let y = (self.0 & 31) as u8;

        let mut idx = self.0 >> 5;
        let mut ux = 0;
        let mut uz = 0;

        let mut bit = 0;
        loop {
            let rx = (1 & (idx >> 1)) as u8;
            let rz = (1 & (idx ^ (rx as u16))) as u8;

            if rz == 0 {
                if rx == 1 {
                    let mask = (1 << bit) - 1;
                    ux ^= mask;
                    uz ^= mask;
                }
                std::mem::swap(&mut ux, &mut uz);
            }
            ux |= rx << bit;
            uz |= rz << bit;

            idx >>= 2;
            bit += 1;

            if bit == 5 {
                break;
            }
        }
        (ux, y, uz)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HilbertIdx(u16);

impl HilbertIdx {
    #[inline]
    pub const fn idx(&self) -> u16 {
        self.0
    }

    #[inline]
    pub const fn encode(x: u8, z: u8) -> Self {
        let mut ux = x;
        let mut uz = z;
        let mut idx = 0;

        let mut bit = 7;
        loop {
            let rx = ((ux >> bit) & 1) as u16;
            let rz = ((uz >> bit) & 1) as u16;

            idx = (idx << 2) | ((3 * rx) ^ rz);

            if rz == 0 {
                if rx == 1 {
                    ux = !ux;
                    uz = !uz;
                }
                std::mem::swap(&mut ux, &mut uz);
            }
            if bit == 0 {
                break;
            }
            bit -= 1;
        }
        Self(idx)
    }

    #[inline]
    pub const fn decode(&self) -> (u8, u8) {
        let mut idx = self.0;
        let mut ux = 0;
        let mut uz = 0;

        let mut bit = 0;
        loop {
            let rx = (1 & (idx >> 1)) as u8;
            let rz = (1 & (idx ^ (rx as u16))) as u8;

            if rz == 0 {
                if rx == 1 {
                    let mask = (1 << bit) - 1;
                    ux ^= mask;
                    uz ^= mask;
                }
                std::mem::swap(&mut ux, &mut uz);
            }
            ux |= rx << bit;
            uz |= rz << bit;

            idx >>= 2;
            bit += 1;

            if bit == 8 {
                break;
            }
        }
        (ux, uz)
    }
}

#[cfg(test)]
mod tests {
    use crate::coords::{FlatIdx, HilbertIdx, HilbertLinearIdx, MortonIdx};

    #[test]
    fn flat_roundtrip() {
        let input = [
            (11, 31, 5),
            (5, 1, 9),
            (4, 1, 11),
            (0, 8, 0),
            (14, 28, 14),
            (11, 11, 12),
        ];

        for (x, y, z) in input {
            let f = FlatIdx::encode(x, y, z);
            let (nx, ny, nz) = f.decode();
            assert_eq!((x, y, z), (nx, ny, nz));
        }
    }

    #[test]
    fn morton_roundtrip() {
        let input = [
            (10, 0, 15),
            (5, 1, 8),
            (0, 0, 2),
            (11, 15, 15),
            (30, 31, 14),
            (0, 10, 1),
        ];

        for (x, y, z) in input {
            let m = MortonIdx::encode(x, y, z);
            let (nx, ny, nz) = m.decode();
            assert_eq!((x, y, z), (nx, ny, nz));
        }
    }

    #[test]
    fn hilbert_linear_roundtrip() {
        let input = [
            (0, 31, 31),
            (15, 15, 15),
            (0, 0, 0),
            (1, 1, 20),
            (1, 7, 5),
            (15, 2, 8),
        ];

        for (x, y, z) in input {
            let h = HilbertLinearIdx::encode(x, y, z);
            let (nx, ny, nz) = h.decode();

            assert_eq!((x, y, z), (nx, ny, nz))
        }
    }

    #[test]
    fn hilbert_roundtrip() {
        let input = [(0, 0), (127, 127), (63, 64), (1, 1), (100, 50), (112, 89)];

        for (x, z) in input {
            let h = HilbertIdx::encode(x, z);
            let (nx, nz) = h.decode();

            assert_eq!((x, z), (nx, nz))
        }
    }
}
