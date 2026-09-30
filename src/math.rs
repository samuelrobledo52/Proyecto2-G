use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    pub const ONE: Vec3 = Vec3::new(1.0, 1.0, 1.0);
    pub const UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }
    #[inline]
    pub fn splat(v: f32) -> Self {
        Vec3::new(v, v, v)
    }
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Vec3::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }
    #[inline]
    pub fn dot(self, o: Vec3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    #[inline]
    pub fn cross(self, o: Vec3) -> Vec3 {
        Vec3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }
    #[inline]
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
    #[inline]
    pub fn normalize(self) -> Vec3 {
        let l = self.length();
        if l > 1e-12 {
            self / l
        } else {
            self
        }
    }
    #[inline]
    pub fn lerp(self, o: Vec3, t: f32) -> Vec3 {
        self + (o - self) * t
    }
    #[inline]
    pub fn get(self, i: usize) -> f32 {
        match i {
            0 => self.x,
            1 => self.y,
            _ => self.z,
        }
    }
    #[inline]
    pub fn max_elem(self) -> f32 {
        self.x.max(self.y).max(self.z)
    }
    #[inline]
    pub fn is_black(self) -> bool {
        self.x <= 0.0 && self.y <= 0.0 && self.z <= 0.0
    }
    #[inline]
    pub fn reflect(self, n: Vec3) -> Vec3 {
        self - n * (2.0 * self.dot(n))
    }
    pub fn refract(self, n: Vec3, eta: f32) -> Option<Vec3> {
        let cosi = (-self.dot(n)).clamp(-1.0, 1.0);
        let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
        if k < 0.0 {
            None
        } else {
            Some((self * eta + n * (eta * cosi - k.sqrt())).normalize())
        }
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    #[inline]
    fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}
impl Sub for Vec3 {
    type Output = Vec3;
    #[inline]
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}
impl Mul<f32> for Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, s: f32) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
}
impl Mul<Vec3> for Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x * o.x, self.y * o.y, self.z * o.z)
    }
}
impl Div<f32> for Vec3 {
    type Output = Vec3;
    #[inline]
    fn div(self, s: f32) -> Vec3 {
        Vec3::new(self.x / s, self.y / s, self.z / s)
    }
}
impl Neg for Vec3 {
    type Output = Vec3;
    #[inline]
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}
impl AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, o: Vec3) {
        self.x += o.x;
        self.y += o.y;
        self.z += o.z;
    }
}
impl MulAssign<f32> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, s: f32) {
        self.x *= s;
        self.y *= s;
        self.z *= s;
    }
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn schlick(cos_i: f32, n1: f32, n2: f32) -> f32 {
    let mut r0 = (n1 - n2) / (n1 + n2);
    r0 *= r0;
    let mut c = cos_i;
    if n1 > n2 {
        let ratio = n1 / n2;
        let sin2t = ratio * ratio * (1.0 - c * c);
        if sin2t > 1.0 {
            return 1.0;
        }
        c = (1.0 - sin2t).sqrt();
    }
    let x = 1.0 - c;
    r0 + (1.0 - r0) * x * x * x * x * x
}

#[inline]
pub fn hash2(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343)
        ^ (y as u32).wrapping_mul(0xd816_3841)
        ^ seed.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0x00ff_ffff) as f32 / 16_777_216.0
}

#[inline]
pub fn hash3(x: i32, y: i32, z: i32, seed: u32) -> f32 {
    hash2(x, y.wrapping_mul(7919) ^ z.wrapping_mul(104_729), seed)
}

pub fn value_noise2(x: f32, y: f32, seed: u32) -> f32 {
    let xi = x.floor();
    let yi = y.floor();
    let fx = x - xi;
    let fy = y - yi;
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);
    let (ix, iy) = (xi as i32, yi as i32);
    let a = hash2(ix, iy, seed);
    let b = hash2(ix + 1, iy, seed);
    let c = hash2(ix, iy + 1, seed);
    let d = hash2(ix + 1, iy + 1, seed);
    lerp(lerp(a, b, ux), lerp(c, d, ux), uy)
}

pub fn fbm2(x: f32, y: f32, octaves: u32, seed: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut norm = 0.0;
    for i in 0..octaves {
        sum += value_noise2(x * freq, y * freq, seed.wrapping_add(i * 1013)) * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.03;
    }
    sum / norm
}

#[derive(Clone, Copy, Debug)]
pub struct Mat3 {
    pub c0: Vec3,
    pub c1: Vec3,
    pub c2: Vec3,
}

impl Mat3 {
    pub const IDENTITY: Mat3 = Mat3 {
        c0: Vec3::new(1.0, 0.0, 0.0),
        c1: Vec3::new(0.0, 1.0, 0.0),
        c2: Vec3::new(0.0, 0.0, 1.0),
    };
    pub fn rot_x(a: f32) -> Mat3 {
        let (s, c) = a.sin_cos();
        Mat3 {
            c0: Vec3::new(1.0, 0.0, 0.0),
            c1: Vec3::new(0.0, c, s),
            c2: Vec3::new(0.0, -s, c),
        }
    }
    pub fn rot_y(a: f32) -> Mat3 {
        let (s, c) = a.sin_cos();
        Mat3 {
            c0: Vec3::new(c, 0.0, -s),
            c1: Vec3::new(0.0, 1.0, 0.0),
            c2: Vec3::new(s, 0.0, c),
        }
    }
    pub fn rot_z(a: f32) -> Mat3 {
        let (s, c) = a.sin_cos();
        Mat3 {
            c0: Vec3::new(c, s, 0.0),
            c1: Vec3::new(-s, c, 0.0),
            c2: Vec3::new(0.0, 0.0, 1.0),
        }
    }
    #[inline]
    pub fn mul_vec(&self, v: Vec3) -> Vec3 {
        self.c0 * v.x + self.c1 * v.y + self.c2 * v.z
    }
    #[inline]
    pub fn tmul_vec(&self, v: Vec3) -> Vec3 {
        Vec3::new(self.c0.dot(v), self.c1.dot(v), self.c2.dot(v))
    }
    pub fn mul(&self, o: &Mat3) -> Mat3 {
        Mat3 {
            c0: self.mul_vec(o.c0),
            c1: self.mul_vec(o.c1),
            c2: self.mul_vec(o.c2),
        }
    }
}

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }
    pub fn next_f32(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        ((x >> 40) as f32) / 16_777_216.0
    }
}

pub fn halton(mut i: u32, b: u32) -> f32 {
    let mut f = 1.0f32;
    let mut r = 0.0f32;
    while i > 0 {
        f /= b as f32;
        r += f * (i % b) as f32;
        i /= b;
    }
    r
}
