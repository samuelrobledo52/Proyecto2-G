use crate::math::{hash2, Vec3};

pub const TEX_SIZE: usize = 16;

pub const T_GRASS_TOP: usize = 0;
pub const T_GRASS_SIDE: usize = 1;
pub const T_DIRT: usize = 2;
pub const T_STONE: usize = 3;
pub const T_COBBLE: usize = 4;
pub const T_PLANKS: usize = 5;
pub const T_LOG_SIDE: usize = 6;
pub const T_LOG_TOP: usize = 7;
pub const T_LEAVES: usize = 8;
pub const T_GLASS: usize = 9;
pub const T_WATER: usize = 10;
pub const T_SAND: usize = 11;
pub const T_GOLD: usize = 12;
pub const T_GLOW: usize = 13;
pub const T_OBSIDIAN: usize = 14;
pub const T_PORTAL: usize = 15;
pub const T_NETHERRACK: usize = 16;
pub const T_END_STONE: usize = 17;
pub const T_DIAMOND_ORE: usize = 18;

#[derive(Clone)]
pub struct Texture {
    pub size: usize,
    pub rgb: Vec<Vec3>,
    pub alpha: Vec<f32>,
}

impl Texture {
    #[inline]
    fn index(&self, u: f32, v: f32) -> usize {
        let s = self.size;
        let x = (u.rem_euclid(1.0) * s as f32) as usize;
        let y = (v.rem_euclid(1.0) * s as f32) as usize;
        x.min(s - 1) + y.min(s - 1) * s
    }
    #[inline]
    pub fn color(&self, u: f32, v: f32) -> Vec3 {
        self.rgb[self.index(u, v)]
    }
    #[inline]
    pub fn alpha(&self, u: f32, v: f32) -> f32 {
        self.alpha[self.index(u, v)]
    }
}

pub fn build_textures() -> Vec<Texture> {
    vec![
        grass_top(),
        grass_side(),
        dirt(),
        stone(),
        cobble(),
        planks(),
        log_side(),
        log_top(),
        leaves(),
        glass(),
        water(),
        sand(),
        gold(),
        glowstone(),
        obsidian(),
        portal(),
        netherrack(),
        end_stone(),
        diamond_ore(),
    ]
}

fn gen<F: Fn(i32, i32) -> (Vec3, f32)>(f: F) -> Texture {
    let n = TEX_SIZE * TEX_SIZE;
    let mut t = Texture {
        size: TEX_SIZE,
        rgb: vec![Vec3::ZERO; n],
        alpha: vec![1.0; n],
    };
    for y in 0..TEX_SIZE as i32 {
        for x in 0..TEX_SIZE as i32 {
            let (c, a) = f(x, y);
            let i = y as usize * TEX_SIZE + x as usize;
            t.rgb[i] = c;
            t.alpha[i] = a;
        }
    }
    t
}

fn dirt_px(x: i32, y: i32) -> Vec3 {
    let h = hash2(x, y, 11);
    if h > 0.9 {
        Vec3::rgb(102, 72, 50)
    } else if h < 0.07 {
        Vec3::rgb(165, 124, 90)
    } else {
        Vec3::rgb(134, 96, 67) * (0.86 + 0.24 * hash2(x, y, 12))
    }
}

fn grass_px(x: i32, y: i32) -> Vec3 {
    let h = hash2(x, y, 21);
    if h > 0.92 {
        Vec3::rgb(78, 128, 44)
    } else {
        Vec3::rgb(104, 168, 62) * (0.8 + 0.35 * hash2(x, y, 22))
    }
}

fn grass_top() -> Texture {
    gen(|x, y| (grass_px(x, y), 1.0))
}

fn grass_side() -> Texture {
    gen(|x, y| {
        let edge = 3 + (hash2(x, 0, 23) > 0.45) as i32 + (hash2(x, 1, 24) > 0.8) as i32;
        if y < edge {
            (grass_px(x, y) * 0.92, 1.0)
        } else {
            (dirt_px(x, y), 1.0)
        }
    })
}

fn dirt() -> Texture {
    gen(|x, y| (dirt_px(x, y), 1.0))
}

fn stone() -> Texture {
    gen(|x, y| {
        let blob = hash2(x / 3, y / 2, 31);
        let h = hash2(x, y, 32);
        let mut c = Vec3::rgb(128, 128, 128) * (0.78 + 0.18 * blob + 0.1 * h);
        if h > 0.93 {
            c = Vec3::rgb(98, 98, 100);
        }
        (c, 1.0)
    })
}

fn cobble() -> Texture {
    gen(|x, y| {
        let row = y / 4;
        let off = if row % 2 == 1 { 2 } else { 0 };
        let lx = (x + off) % 4;
        let col = (x + off) / 4;
        if y % 4 == 0 || lx == 0 {
            return (Vec3::rgb(72, 72, 74) * (0.9 + 0.2 * hash2(x, y, 41)), 1.0);
        }
        let tone = 0.78 + 0.3 * hash2(col, row, 42);
        let mut c = Vec3::rgb(122, 122, 122) * tone * (0.92 + 0.12 * hash2(x, y, 43));
        if y % 4 == 1 || lx == 1 {
            c = c * 1.12;
        }
        (c, 1.0)
    })
}

fn planks() -> Texture {
    gen(|x, y| {
        let row = y / 4;
        if y % 4 == 3 {
            return (Vec3::rgb(104, 80, 48), 1.0);
        }
        if x == (row * 7 + 3) % 16 {
            return (Vec3::rgb(118, 92, 56), 1.0);
        }
        let grain = hash2(x / 4 + row * 5, y, 51);
        (
            Vec3::rgb(162, 130, 80) * (0.86 + 0.14 * grain + 0.08 * hash2(x, y, 52)),
            1.0,
        )
    })
}

fn log_side() -> Texture {
    gen(|x, y| {
        let s = hash2(x, 0, 61);
        let c = Vec3::rgb(106, 84, 52) * (0.68 + 0.42 * s) * (0.9 + 0.15 * hash2(x, y / 3, 62));
        (c, 1.0)
    })
}

fn log_top() -> Texture {
    gen(|x, y| {
        let dx = x as f32 - 7.5;
        let dy = y as f32 - 7.5;
        let d = (dx * dx + dy * dy).sqrt();
        let h = hash2(x, y, 71);
        if d > 6.6 {
            return (Vec3::rgb(106, 84, 52) * (0.75 + 0.3 * h), 1.0);
        }
        let ring = ((d * 0.85) as i32) % 2 == 0;
        let c = if ring {
            Vec3::rgb(186, 152, 98)
        } else {
            Vec3::rgb(152, 120, 74)
        };
        (c * (0.92 + 0.12 * h), 1.0)
    })
}

fn leaves() -> Texture {
    gen(|x, y| {
        let h = hash2(x, y, 81);
        let a = if hash2(x, y, 82) < 0.22 { 0.0 } else { 1.0 };
        (Vec3::rgb(56, 128, 38) * (0.6 + 0.55 * h), a)
    })
}

fn glass() -> Texture {
    gen(|x, y| {
        if x == 0 || y == 0 || x == 15 || y == 15 {
            return (Vec3::rgb(236, 244, 248), 1.0);
        }
        let d = x - y;
        if (d == 4 && x < 11) || (d == 5 && x < 10) || (d == -6 && x > 5 && x < 12) {
            return (Vec3::rgb(255, 255, 255), 1.0);
        }
        (Vec3::rgb(196, 226, 238), 1.0)
    })
}

fn water() -> Texture {
    gen(|x, y| {
        let tau = std::f32::consts::TAU;
        let fx = x as f32 / 16.0;
        let fy = y as f32 / 16.0;
        let w = ((fx * 2.0 + fy) * tau).sin() * 0.5 + ((fy * 2.0 - fx) * tau).sin() * 0.5;
        (
            Vec3::rgb(44, 96, 196) * (0.86 + 0.12 * w + 0.08 * hash2(x, y, 91)),
            1.0,
        )
    })
}

fn sand() -> Texture {
    gen(|x, y| {
        let h = hash2(x, y, 101);
        let c = if h > 0.9 {
            Vec3::rgb(194, 180, 134)
        } else {
            Vec3::rgb(222, 210, 164) * (0.88 + 0.16 * hash2(x, y, 102))
        };
        (c, 1.0)
    })
}

fn gold() -> Texture {
    gen(|x, y| {
        if x == 0 || y == 0 {
            return (Vec3::rgb(255, 244, 150), 1.0);
        }
        if x == 15 || y == 15 {
            return (Vec3::rgb(196, 146, 28), 1.0);
        }
        let h = hash2(x, y, 111);
        let mut c = Vec3::rgb(246, 204, 60) * (0.9 + 0.14 * h);
        if (x + y) % 6 == 0 && x > 2 && x < 13 {
            c = Vec3::rgb(255, 236, 130);
        }
        (c, 1.0)
    })
}

fn glowstone() -> Texture {
    gen(|x, y| {
        let cell = hash2(x / 3, y / 3, 121);
        let h = hash2(x, y, 122);
        if h > 0.88 {
            return (Vec3::rgb(132, 86, 40), 1.0);
        }
        (
            Vec3::rgb(176, 112, 52).lerp(Vec3::rgb(255, 228, 150), cell) * (0.88 + 0.2 * h),
            1.0,
        )
    })
}

fn obsidian() -> Texture {
    gen(|x, y| {
        let h = hash2(x, y, 131);
        if hash2(x / 2, y / 2, 132) > 0.86 {
            return (Vec3::rgb(64, 42, 96) * (0.8 + 0.4 * h), 1.0);
        }
        (Vec3::rgb(22, 15, 34) * (0.7 + 0.6 * h), 1.0)
    })
}

fn portal() -> Texture {
    gen(|x, y| {
        let tau = std::f32::consts::TAU;
        let fx = x as f32 / 16.0;
        let fy = y as f32 / 16.0;
        let w = ((fx + fy) * tau * 2.0).sin() * 0.5 + ((fx * 2.0 - fy) * tau).sin() * 0.5;
        let c = Vec3::rgb(90, 20, 170).lerp(Vec3::rgb(195, 115, 255), 0.5 + 0.5 * w);
        (c * (0.85 + 0.3 * hash2(x, y, 141)), 1.0)
    })
}

fn netherrack() -> Texture {
    gen(|x, y| {
        let h = hash2(x, y, 151);
        let blob = hash2(x / 2, y / 3, 152);
        if h > 0.9 {
            return (Vec3::rgb(70, 22, 24), 1.0);
        }
        (Vec3::rgb(114, 42, 42) * (0.72 + 0.25 * blob + 0.2 * h), 1.0)
    })
}

fn end_stone() -> Texture {
    gen(|x, y| {
        let h = hash2(x, y, 161);
        if h > 0.88 {
            return (Vec3::rgb(182, 184, 126), 1.0);
        }
        (Vec3::rgb(222, 224, 164) * (0.86 + 0.2 * hash2(x, y, 162)), 1.0)
    })
}

fn diamond_ore() -> Texture {
    gen(|x, y| {
        let h = hash2(x, y, 171);
        let gem = hash2(x / 2, y / 2, 172) > 0.8 && hash2(x, y, 173) > 0.25;
        if gem {
            let c = if h > 0.7 {
                Vec3::rgb(210, 255, 250)
            } else {
                Vec3::rgb(70, 215, 210)
            };
            return (c, 1.0);
        }
        let blob = hash2(x / 3, y / 2, 31);
        (Vec3::rgb(128, 128, 128) * (0.78 + 0.18 * blob + 0.1 * h), 1.0)
    })
}
