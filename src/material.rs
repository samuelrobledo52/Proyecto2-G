use crate::math::Vec3;
use crate::texture::*;

pub const AIR: u8 = 0;
pub const GRASS: u8 = 1;
pub const DIRT: u8 = 2;
pub const STONE: u8 = 3;
pub const COBBLE: u8 = 4;
pub const PLANKS: u8 = 5;
pub const LOG: u8 = 6;
pub const LEAVES: u8 = 7;
pub const GLASS: u8 = 8;
pub const WATER: u8 = 9;
pub const SAND: u8 = 10;
pub const GOLD: u8 = 11;
pub const GLOWSTONE: u8 = 12;
pub const OBSIDIAN: u8 = 13;
pub const PORTAL: u8 = 14;
pub const NETHERRACK: u8 = 15;
pub const END_STONE: u8 = 16;
pub const DIAMOND_ORE: u8 = 17;

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub name: &'static str,
    pub tex_top: usize,
    pub tex_side: usize,
    pub tex_bottom: usize,
    pub albedo: f32,
    pub specular: f32,
    pub shininess: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub ior: f32,
    pub emission: f32,
    pub tint: Vec3,
    pub cutout: bool,
}

impl Material {
    #[inline]
    pub fn tex_for(&self, n: Vec3) -> usize {
        if n.y > 0.5 {
            self.tex_top
        } else if n.y < -0.5 {
            self.tex_bottom
        } else {
            self.tex_side
        }
    }
}

const fn mat(name: &'static str, top: usize, side: usize, bottom: usize) -> Material {
    Material {
        name,
        tex_top: top,
        tex_side: side,
        tex_bottom: bottom,
        albedo: 0.9,
        specular: 0.05,
        shininess: 8.0,
        transparency: 0.0,
        reflectivity: 0.0,
        ior: 1.0,
        emission: 0.0,
        tint: Vec3::ONE,
        cutout: false,
    }
}

pub fn materials() -> Vec<Material> {
    vec![
        Material {
            transparency: 1.0,
            albedo: 0.0,
            ..mat("Aire", T_GLASS, T_GLASS, T_GLASS)
        },
        Material {
            albedo: 0.92,
            specular: 0.04,
            shininess: 8.0,
            ..mat("Pasto", T_GRASS_TOP, T_GRASS_SIDE, T_DIRT)
        },
        Material {
            albedo: 0.9,
            specular: 0.02,
            shininess: 4.0,
            ..mat("Tierra", T_DIRT, T_DIRT, T_DIRT)
        },
        Material {
            albedo: 0.88,
            specular: 0.25,
            shininess: 24.0,
            reflectivity: 0.03,
            ..mat("Piedra", T_STONE, T_STONE, T_STONE)
        },
        Material {
            albedo: 0.86,
            specular: 0.15,
            shininess: 16.0,
            ..mat("Cobblestone", T_COBBLE, T_COBBLE, T_COBBLE)
        },
        Material {
            albedo: 0.88,
            specular: 0.12,
            shininess: 12.0,
            ..mat("Tablones de roble", T_PLANKS, T_PLANKS, T_PLANKS)
        },
        Material {
            albedo: 0.88,
            specular: 0.06,
            shininess: 8.0,
            ..mat("Tronco", T_LOG_TOP, T_LOG_SIDE, T_LOG_TOP)
        },
        Material {
            albedo: 0.9,
            specular: 0.15,
            shininess: 10.0,
            cutout: true,
            ..mat("Hojas", T_LEAVES, T_LEAVES, T_LEAVES)
        },
        Material {
            albedo: 1.0,
            specular: 1.2,
            shininess: 96.0,
            transparency: 0.88,
            reflectivity: 0.08,
            ior: 1.5,
            tint: Vec3::new(0.92, 1.0, 0.98),
            ..mat("Vidrio", T_GLASS, T_GLASS, T_GLASS)
        },
        Material {
            albedo: 1.0,
            specular: 1.4,
            shininess: 128.0,
            transparency: 0.78,
            reflectivity: 0.15,
            ior: 1.33,
            tint: Vec3::new(0.55, 0.8, 0.95),
            ..mat("Agua", T_WATER, T_WATER, T_WATER)
        },
        Material {
            albedo: 0.92,
            specular: 0.05,
            shininess: 6.0,
            ..mat("Arena", T_SAND, T_SAND, T_SAND)
        },
        Material {
            albedo: 0.8,
            specular: 1.0,
            shininess: 64.0,
            reflectivity: 0.45,
            ..mat("Oro", T_GOLD, T_GOLD, T_GOLD)
        },
        Material {
            albedo: 1.0,
            specular: 0.3,
            shininess: 16.0,
            emission: 0.6,
            ..mat("Glowstone", T_GLOW, T_GLOW, T_GLOW)
        },
        Material {
            albedo: 0.9,
            specular: 0.9,
            shininess: 64.0,
            reflectivity: 0.12,
            ..mat("Obsidiana", T_OBSIDIAN, T_OBSIDIAN, T_OBSIDIAN)
        },
        Material {
            albedo: 1.0,
            specular: 0.6,
            shininess: 32.0,
            transparency: 0.6,
            reflectivity: 0.05,
            ior: 1.2,
            emission: 0.9,
            tint: Vec3::new(0.75, 0.45, 1.0),
            ..mat("Portal del Nether", T_PORTAL, T_PORTAL, T_PORTAL)
        },
        Material {
            albedo: 0.9,
            specular: 0.05,
            shininess: 6.0,
            ..mat("Netherrack", T_NETHERRACK, T_NETHERRACK, T_NETHERRACK)
        },
        Material {
            albedo: 0.92,
            specular: 0.08,
            shininess: 8.0,
            ..mat("End Stone", T_END_STONE, T_END_STONE, T_END_STONE)
        },
        Material {
            albedo: 0.88,
            specular: 0.8,
            shininess: 48.0,
            reflectivity: 0.05,
            ..mat("Mena de diamante", T_DIAMOND_ORE, T_DIAMOND_ORE, T_DIAMOND_ORE)
        },
    ]
}
