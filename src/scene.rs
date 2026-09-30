use crate::material::*;
use crate::math::{hash2, hash3, Vec3};

pub const SX: i32 = 40;
pub const SY: i32 = 28;
pub const SZ: i32 = 40;

pub const MAIN_TOP: i32 = 12;
pub const CRYSTAL_POS: Vec3 = Vec3::new(32.0, 19.4, 8.0);
pub const PORTAL_LIGHT: Vec3 = Vec3::new(7.0, 18.5, 7.5);

const OX: i32 = 8;
const OY: i32 = 7;
const OZ: i32 = 8;

pub struct PointLight {
    pub pos: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub radius: f32,
    pub scaled: bool,
}

pub struct Scene {
    blocks: Vec<u8>,
    dist: Vec<u8>,
    pub lights: Vec<PointLight>,
}

impl Scene {
    #[inline]
    fn idx(x: i32, y: i32, z: i32) -> usize {
        (x + SX * (z + SZ * y)) as usize
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32, z: i32) -> u8 {
        if x < 0 || y < 0 || z < 0 || x >= SX || y >= SY || z >= SZ {
            AIR
        } else {
            self.blocks[Self::idx(x, y, z)]
        }
    }

    #[inline]
    pub fn empty_dist(&self, x: i32, y: i32, z: i32) -> u8 {
        self.dist[Self::idx(x, y, z)]
    }

    fn compute_dist(&mut self) {
        let mut d: Vec<u8> = self
            .blocks
            .iter()
            .map(|&b| if b == AIR { 255 } else { 0 })
            .collect();
        let dims = [SX, SY, SZ];
        for axis in 0..3 {
            let (a1, a2) = match axis {
                0 => (1, 2),
                1 => (0, 2),
                _ => (0, 1),
            };
            let len = dims[axis] as usize;
            let mut line = vec![0u8; len];
            let mut out = vec![0u8; len];
            for u in 0..dims[a1] {
                for w in 0..dims[a2] {
                    let mut c = [0i32; 3];
                    c[a1] = u;
                    c[a2] = w;
                    for i in 0..len {
                        c[axis] = i as i32;
                        line[i] = d[Self::idx(c[0], c[1], c[2])];
                    }
                    for i in 0..len {
                        let mut best = 255u8;
                        for (j, &lv) in line.iter().enumerate() {
                            let v = lv.max(i.abs_diff(j).min(255) as u8);
                            if v < best {
                                best = v;
                            }
                        }
                        out[i] = best;
                    }
                    for i in 0..len {
                        c[axis] = i as i32;
                        d[Self::idx(c[0], c[1], c[2])] = out[i];
                    }
                }
            }
        }
        self.dist = d;
    }

    fn set(&mut self, x: i32, y: i32, z: i32, b: u8) {
        if x < 0 || y < 0 || z < 0 || x >= SX || y >= SY || z >= SZ {
            return;
        }
        self.blocks[Self::idx(x, y, z)] = b;
    }

    fn get_o(&self, x: i32, y: i32, z: i32) -> u8 {
        self.get(x + OX, y + OY, z + OZ)
    }

    fn set_o(&mut self, x: i32, y: i32, z: i32, b: u8) {
        self.set(x + OX, y + OY, z + OZ, b);
    }

    pub fn center(&self) -> Vec3 {
        Vec3::new(20.0, 13.0, 20.0)
    }

    pub fn build() -> Scene {
        let mut s = Scene {
            blocks: vec![AIR; (SX * SY * SZ) as usize],
            dist: Vec::new(),
            lights: Vec::new(),
        };
        let fall_x = s.build_main();
        s.build_lower(fall_x);
        s.build_end();
        s.build_portal();
        s.collect_lights();
        s.compute_dist();
        s
    }

    fn build_main(&mut self) -> i32 {
        for x in 0..24 {
            for z in 0..24 {
                let dx = x as f32 + 0.5 - 12.0;
                let dz = z as f32 + 0.5 - 12.0;
                let r = (dx * dx + dz * dz).sqrt();
                let rr = 10.6 + (hash2(x, z, 7) - 0.5) * 1.4;
                if r > rr {
                    continue;
                }
                self.set_o(x, 5, z, GRASS);
                self.set_o(x, 4, z, DIRT);
                self.set_o(x, 3, z, DIRT);
                let depth = ((1.0 - r / rr) * 10.0 + hash2(x, z, 8) * 2.0) as i32;
                for y in (3 - depth)..3 {
                    let b = if hash3(x, y, z, 9) < 0.07 { DIAMOND_ORE } else { STONE };
                    self.set_o(x, y, z, b);
                }
            }
        }

        let (pcx, pcz) = (8.0f32, 15.5f32);
        for x in 0..24 {
            for z in 0..24 {
                if self.get_o(x, 5, z) == AIR {
                    continue;
                }
                let dx = x as f32 + 0.5 - pcx;
                let dz = z as f32 + 0.5 - pcz;
                let d = (dx * dx + dz * dz).sqrt();
                if d < 2.9 {
                    self.set_o(x, 5, z, WATER);
                    if d < 2.0 {
                        self.set_o(x, 4, z, WATER);
                        self.set_o(x, 3, z, SAND);
                    } else {
                        self.set_o(x, 4, z, SAND);
                    }
                } else if d < 3.9 {
                    self.set_o(x, 5, z, SAND);
                }
            }
        }

        let cz = 15;
        let mut x = 4;
        while x >= -OX && self.get_o(x, 5, cz) != AIR {
            self.set_o(x, 5, cz, WATER);
            self.set_o(x, 4, cz, SAND);
            x -= 1;
        }
        let fall_x = x + OX;
        let fall_z = cz + OZ;
        for y in 4..=MAIN_TOP {
            if self.get(fall_x, y, fall_z) == AIR {
                self.set(fall_x, y, fall_z, WATER);
            }
        }

        let (x0, x1, z0, z1) = (12, 18, 5, 11);
        for x in x0..=x1 {
            for z in z0..=z1 {
                self.set_o(x, 5, z, COBBLE);
            }
        }
        for y in 6..=9 {
            for x in x0..=x1 {
                for z in z0..=z1 {
                    let edge_x = x == x0 || x == x1;
                    let edge_z = z == z0 || z == z1;
                    if !(edge_x || edge_z) {
                        continue;
                    }
                    let b = if edge_x && edge_z { LOG } else { PLANKS };
                    self.set_o(x, y, z, b);
                }
            }
        }
        for y in 7..=8 {
            for x in 14..=16 {
                self.set_o(x, y, z0, GLASS);
                self.set_o(x, y, z1, GLASS);
            }
            for z in 7..=9 {
                self.set_o(x1, y, z, GLASS);
            }
            self.set_o(x0, y, 6, GLASS);
            self.set_o(x0, y, 10, GLASS);
        }
        self.set_o(x0, 6, 8, AIR);
        self.set_o(x0, 7, 8, AIR);
        for x in (x0 - 1)..=(x1 + 1) {
            for z in (z0 - 1)..=(z1 + 1) {
                self.set_o(x, 10, z, PLANKS);
            }
        }
        for k in 1..=3 {
            for x in (x0 - 1 + k)..=(x1 + 1 - k) {
                for z in (z0 - 1 + k)..=(z1 + 1 - k) {
                    self.set_o(x, 10 + k, z, COBBLE);
                }
            }
        }
        self.set_o(15, 9, 8, GLOWSTONE);
        self.set_o(16, 6, 6, GOLD);

        for &(x, z) in &[(11, 8), (11, 9), (10, 10), (10, 11), (10, 12)] {
            if self.get_o(x, 5, z) == GRASS {
                self.set_o(x, 5, z, COBBLE);
            }
        }

        self.tree(5 + OX, 6 + OY, 8 + OZ, 5, 3);
        self.tree(18 + OX, 6 + OY, 17 + OZ, 4, 9);

        self.set_o(11, 6, 15, LOG);
        self.set_o(11, 7, 15, LOG);
        self.set_o(11, 8, 15, GLOWSTONE);
        self.set_o(4, 6, 14, GOLD);
        self.set_o(4, 7, 14, GOLD);
        self.set_o(16, 6, 16, STONE);
        self.set_o(17, 6, 16, STONE);
        self.set_o(16, 7, 16, COBBLE);
        self.set_o(16, 6, 17, COBBLE);

        fall_x
    }

    fn build_lower(&mut self, fx: i32) {
        let (cx, cz, top) = (fx as f32 - 1.5, 25.0f32, 3);
        for x in 0..SX {
            for z in 0..SZ {
                let dx = x as f32 + 0.5 - cx;
                let dz = z as f32 + 0.5 - cz;
                let r = (dx * dx + dz * dz).sqrt();
                let rr = 5.0 + (hash2(x, z, 17) - 0.5) * 1.0;
                if r > rr {
                    continue;
                }
                self.set(x, top, z, GRASS);
                self.set(x, top - 1, z, DIRT);
                let depth = ((1.0 - r / rr) * 4.0 + hash2(x, z, 18)) as i32;
                for y in (top - 1 - depth).max(0)..(top - 1) {
                    let b = if hash3(x, y, z, 19) < 0.08 { DIAMOND_ORE } else { STONE };
                    self.set(x, y, z, b);
                }
            }
        }
        let (px, pz) = (fx as f32 + 0.5, 23.5f32);
        for x in 0..SX {
            for z in 0..SZ {
                if self.get(x, top, z) == AIR {
                    continue;
                }
                let dx = x as f32 + 0.5 - px;
                let dz = z as f32 + 0.5 - pz;
                let d = (dx * dx + dz * dz).sqrt();
                if d < 1.7 {
                    self.set(x, top, z, WATER);
                    if d < 1.0 {
                        self.set(x, top - 1, z, WATER);
                        self.set(x, top - 2, z, SAND);
                    } else {
                        self.set(x, top - 1, z, SAND);
                    }
                } else if d < 2.5 {
                    self.set(x, top, z, SAND);
                }
            }
        }
        let tx = (cx - 2.0) as i32;
        let tz = (cz + 2.0) as i32;
        if self.get(tx, top, tz) == GRASS {
            self.tree(tx, top + 1, tz, 3, 21);
        }
    }

    fn build_end(&mut self) {
        let (cx, cz, top) = (32.0f32, 8.0f32, 9);
        for x in 0..SX {
            for z in 0..SZ {
                let dx = x as f32 + 0.5 - cx;
                let dz = z as f32 + 0.5 - cz;
                let r = (dx * dx + dz * dz).sqrt();
                let rr = 4.3 + (hash2(x, z, 27) - 0.5) * 0.8;
                if r > rr {
                    continue;
                }
                let depth = ((1.0 - r / rr) * 5.0 + hash2(x, z, 28) * 1.2) as i32;
                for y in (top - depth).max(0)..=top {
                    self.set(x, y, z, END_STONE);
                }
            }
        }
        for y in (top + 1)..=(top + 8) {
            for x in 31..=32 {
                for z in 7..=8 {
                    self.set(x, y, z, OBSIDIAN);
                }
            }
        }
        self.set(29, top + 1, 10, OBSIDIAN);
        self.set(34, top + 1, 6, OBSIDIAN);
    }

    fn build_portal(&mut self) {
        let (cx, cz, top) = (7.0f32, 7.0f32, 15);
        for x in 0..SX {
            for z in 0..SZ {
                let dx = x as f32 + 0.5 - cx;
                let dz = z as f32 + 0.5 - cz;
                let r = (dx * dx + dz * dz).sqrt();
                let rr = 3.8 + (hash2(x, z, 37) - 0.5) * 0.8;
                if r > rr {
                    continue;
                }
                let depth = ((1.0 - r / rr) * 5.0 + hash2(x, z, 38)) as i32;
                for y in (top - depth).max(0)..=top {
                    self.set(x, y, z, NETHERRACK);
                }
            }
        }
        for x in 5..=8 {
            for y in 16..=20 {
                let frame = x == 5 || x == 8 || y == 16 || y == 20;
                self.set(x, y, 7, if frame { OBSIDIAN } else { PORTAL });
            }
        }
    }

    fn tree(&mut self, x: i32, base_y: i32, z: i32, h: i32, seed: u32) {
        let top = base_y + h - 1;
        for y in base_y..=top {
            self.set(x, y, z, LOG);
        }
        for (layer, y) in ((top - 1)..=(top + 2)).enumerate() {
            let rad: i32 = if layer < 2 { 2 } else { 1 };
            for dx in -rad..=rad {
                for dz in -rad..=rad {
                    let corner = dx.abs() == rad && dz.abs() == rad;
                    if corner && (layer == 1 || hash2(x + dx, z + dz + y * 31, seed) > 0.5) {
                        continue;
                    }
                    if layer == 3 && dx.abs() + dz.abs() > 1 {
                        continue;
                    }
                    if self.get(x + dx, y, z + dz) == AIR {
                        self.set(x + dx, y, z + dz, LEAVES);
                    }
                }
            }
        }
    }

    fn collect_lights(&mut self) {
        for y in 0..SY {
            for z in 0..SZ {
                for x in 0..SX {
                    if self.get(x, y, z) == GLOWSTONE {
                        self.lights.push(PointLight {
                            pos: Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5),
                            color: Vec3::new(1.0, 0.78, 0.45),
                            intensity: 2.4,
                            radius: 9.0,
                            scaled: true,
                        });
                    }
                }
            }
        }
        self.lights.push(PointLight {
            pos: PORTAL_LIGHT,
            color: Vec3::new(0.75, 0.35, 1.0),
            intensity: 2.2,
            radius: 8.0,
            scaled: false,
        });
        self.lights.push(PointLight {
            pos: CRYSTAL_POS,
            color: Vec3::new(1.0, 0.45, 0.85),
            intensity: 2.0,
            radius: 9.0,
            scaled: false,
        });
    }
}
