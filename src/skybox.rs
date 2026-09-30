use crate::math::{fbm2, hash3, smoothstep, Vec3};
use std::f32::consts::TAU;

#[derive(Clone, Copy)]
pub enum Preset {
    Day,
    Sunset,
    Night,
}

#[allow(dead_code)]
struct Params {
    name: &'static str,
    zenith: Vec3,
    horizon: Vec3,
    ground: Vec3,
    sun_dir: Vec3,
    disc: Vec3,
    disc_cos: f32,
    glow: Vec3,
    cloud_lit: Vec3,
    cloud_dark: Vec3,
    cloud_lo: f32,
    cloud_hi: f32,
    stars: f32,
    mount_near: Vec3,
    mount_far: Vec3,
    light_color: Vec3,
    amb_sky: Vec3,
    amb_ground: Vec3,
}

fn params(p: Preset) -> Params {
    match p {
        Preset::Day => Params {
            name: "Dia",
            zenith: Vec3::new(0.22, 0.45, 0.85),
            horizon: Vec3::new(0.70, 0.83, 0.95),
            ground: Vec3::new(0.30, 0.40, 0.52),
            sun_dir: Vec3::new(0.45, 0.75, 0.35).normalize(),
            disc: Vec3::new(1.0, 1.0, 0.95),
            disc_cos: 0.9994,
            glow: Vec3::new(1.0, 0.9, 0.7) * 0.35,
            cloud_lit: Vec3::new(1.0, 1.0, 1.0),
            cloud_dark: Vec3::new(0.72, 0.78, 0.86),
            cloud_lo: 0.50,
            cloud_hi: 0.70,
            stars: 0.0,
            mount_near: Vec3::new(0.42, 0.52, 0.66),
            mount_far: Vec3::new(0.56, 0.66, 0.80),
            light_color: Vec3::new(1.0, 0.95, 0.85) * 0.95,
            amb_sky: Vec3::new(0.42, 0.50, 0.62),
            amb_ground: Vec3::new(0.25, 0.23, 0.20),
        },
        Preset::Sunset => Params {
            name: "Atardecer",
            zenith: Vec3::new(0.18, 0.16, 0.38),
            horizon: Vec3::new(1.0, 0.55, 0.28),
            ground: Vec3::new(0.25, 0.15, 0.20),
            sun_dir: Vec3::new(-0.8, 0.14, 0.4).normalize(),
            disc: Vec3::new(1.0, 0.85, 0.5),
            disc_cos: 0.999,
            glow: Vec3::new(1.0, 0.5, 0.2) * 0.9,
            cloud_lit: Vec3::new(1.0, 0.6, 0.45),
            cloud_dark: Vec3::new(0.35, 0.25, 0.40),
            cloud_lo: 0.50,
            cloud_hi: 0.72,
            stars: 0.3,
            mount_near: Vec3::new(0.28, 0.20, 0.32),
            mount_far: Vec3::new(0.50, 0.32, 0.40),
            light_color: Vec3::new(1.0, 0.6, 0.35),
            amb_sky: Vec3::new(0.35, 0.28, 0.40),
            amb_ground: Vec3::new(0.20, 0.12, 0.12),
        },
        Preset::Night => Params {
            name: "Noche",
            zenith: Vec3::new(0.01, 0.02, 0.07),
            horizon: Vec3::new(0.06, 0.09, 0.20),
            ground: Vec3::new(0.02, 0.03, 0.06),
            sun_dir: Vec3::new(0.3, 0.6, -0.7).normalize(),
            disc: Vec3::new(0.9, 0.92, 1.0),
            disc_cos: 0.9992,
            glow: Vec3::new(0.4, 0.5, 0.8) * 0.3,
            cloud_lit: Vec3::new(0.15, 0.18, 0.28),
            cloud_dark: Vec3::new(0.04, 0.05, 0.10),
            cloud_lo: 0.60,
            cloud_hi: 0.80,
            stars: 1.0,
            mount_near: Vec3::new(0.03, 0.04, 0.08),
            mount_far: Vec3::new(0.05, 0.07, 0.13),
            light_color: Vec3::new(0.35, 0.42, 0.65) * 0.5,
            amb_sky: Vec3::new(0.08, 0.10, 0.18),
            amb_ground: Vec3::new(0.03, 0.03, 0.05),
        },
    }
}

pub struct Skybox {
    n: usize,
    faces: Vec<Vec<Vec3>>,
}

impl Skybox {
    pub fn generate(preset: Preset, n: usize) -> Skybox {
        let p = params(preset);
        let mut faces: Vec<Vec<Vec3>> = vec![vec![Vec3::ZERO; n * n]; 6];
        std::thread::scope(|s| {
            for (f, face) in faces.iter_mut().enumerate() {
                let p = &p;
                s.spawn(move || {
                    for j in 0..n {
                        for i in 0..n {
                            let u = (i as f32 + 0.5) / n as f32 * 2.0 - 1.0;
                            let v = (j as f32 + 0.5) / n as f32 * 2.0 - 1.0;
                            face[j * n + i] = sky_color(face_dir(f, u, v).normalize(), p);
                        }
                    }
                });
            }
        });
        Skybox { n, faces }
    }

    pub fn sample(&self, d: Vec3) -> Vec3 {
        let (f, u, v) = dir_to_face(d);
        let n = self.n as f32;
        let x = ((u + 1.0) * 0.5 * n - 0.5).clamp(0.0, n - 1.0);
        let y = ((v + 1.0) * 0.5 * n - 0.5).clamp(0.0, n - 1.0);
        let x0 = x.floor() as usize;
        let y0 = y.floor() as usize;
        let x1 = (x0 + 1).min(self.n - 1);
        let y1 = (y0 + 1).min(self.n - 1);
        let fx = x - x0 as f32;
        let fy = y - y0 as f32;
        let face = &self.faces[f];
        let a = face[y0 * self.n + x0].lerp(face[y0 * self.n + x1], fx);
        let b = face[y1 * self.n + x0].lerp(face[y1 * self.n + x1], fx);
        a.lerp(b, fy)
    }
}

pub struct SkyState {
    pub w: [f32; 3],
    pub sun_dir: Vec3,
    pub moon_dir: Vec3,
    pub key_dir: Vec3,
    pub key_color: Vec3,
    pub amb_sky: Vec3,
    pub amb_ground: Vec3,
    pub night: f32,
    pub lamp: f32,
    sun_vis: f32,
    moon_vis: f32,
    sun_disc: Vec3,
    sun_glow: Vec3,
}

pub struct Sky {
    cubes: Vec<Skybox>,
}

impl Sky {
    pub fn new() -> Sky {
        Sky {
            cubes: vec![
                Skybox::generate(Preset::Day, 256),
                Skybox::generate(Preset::Sunset, 256),
                Skybox::generate(Preset::Night, 256),
            ],
        }
    }

    pub fn state(&self, tod: f32) -> SkyState {
        let th = tod * TAU;
        let sun_dir = Vec3::new(th.cos(), th.sin() * 0.92, 0.38).normalize();
        let e = sun_dir.y;
        let day = smoothstep(0.05, 0.35, e);
        let night = smoothstep(-0.05, -0.3, e);
        let sunset = (1.0 - day - night).max(0.0);
        let pd = params(Preset::Day);
        let ps = params(Preset::Sunset);
        let pn = params(Preset::Night);
        let sun_vis = smoothstep(-0.04, 0.06, e);
        let moon_vis = smoothstep(-0.04, 0.06, -e);
        let ds = (day + sunset).max(1e-3);
        let sun_col = (pd.light_color * day + ps.light_color * sunset) / ds * sun_vis;
        let moon_col = Vec3::new(0.35, 0.42, 0.65) * 0.45 * moon_vis;
        let (key_dir, key_color) = if e >= 0.0 {
            (sun_dir, sun_col)
        } else {
            (-sun_dir, moon_col)
        };
        let sk = sunset / ds;
        SkyState {
            w: [day, sunset, night],
            sun_dir,
            moon_dir: -sun_dir,
            key_dir,
            key_color,
            amb_sky: pd.amb_sky * day + ps.amb_sky * sunset + pn.amb_sky * night,
            amb_ground: pd.amb_ground * day + ps.amb_ground * sunset + pn.amb_ground * night,
            night,
            lamp: 0.5 + 1.3 * night + 0.5 * sunset,
            sun_vis,
            moon_vis,
            sun_disc: Vec3::new(1.7, 1.6, 1.4).lerp(Vec3::new(1.8, 1.0, 0.45), sk),
            sun_glow: (Vec3::new(1.0, 0.9, 0.7) * 0.35).lerp(Vec3::new(1.0, 0.5, 0.2) * 0.9, sk),
        }
    }

    pub fn phase_name(st: &SkyState, tod: f32) -> &'static str {
        if st.w[0] >= 0.5 {
            "Dia"
        } else if st.w[2] >= 0.5 {
            "Noche"
        } else if tod < 0.25 || tod > 0.9 {
            "Amanecer"
        } else {
            "Atardecer"
        }
    }

    pub fn color(&self, d: Vec3, st: &SkyState) -> Vec3 {
        let mut c = Vec3::ZERO;
        for i in 0..3 {
            if st.w[i] > 0.001 {
                c += self.cubes[i].sample(d) * st.w[i];
            }
        }
        let fade = smoothstep(-0.01, 0.04, d.y);
        let cs = d.dot(st.sun_dir).max(0.0);
        c += st.sun_glow * ((cs.powf(6.0) * 0.35 + cs.powf(48.0) * 0.6) * st.sun_vis);
        let disc = smoothstep(0.9990, 0.9994, cs) * st.sun_vis * fade;
        c = c.lerp(st.sun_disc, disc);
        if st.moon_vis > 0.0 {
            let cm = d.dot(st.moon_dir).max(0.0);
            c += Vec3::new(0.35, 0.45, 0.75) * (cm.powf(24.0) * 0.15 * st.moon_vis);
            let md = smoothstep(0.9991, 0.9994, cm) * st.moon_vis * fade;
            if md > 0.0 {
                let q = d * 900.0;
                let k = 0.82
                    + 0.18 * hash3(q.x.floor() as i32, q.y.floor() as i32, q.z.floor() as i32, 404);
                c = c.lerp(Vec3::new(1.0, 1.0, 1.08) * (k * 1.3), md);
            }
        }
        c
    }
}

fn dir_to_face(d: Vec3) -> (usize, f32, f32) {
    let (ax, ay, az) = (d.x.abs(), d.y.abs(), d.z.abs());
    if ax >= ay && ax >= az {
        if d.x > 0.0 {
            (0, -d.z / ax, -d.y / ax)
        } else {
            (1, d.z / ax, -d.y / ax)
        }
    } else if ay >= az {
        if d.y > 0.0 {
            (2, d.x / ay, d.z / ay)
        } else {
            (3, d.x / ay, -d.z / ay)
        }
    } else if d.z > 0.0 {
        (4, d.x / az, -d.y / az)
    } else {
        (5, -d.x / az, -d.y / az)
    }
}

fn face_dir(f: usize, u: f32, v: f32) -> Vec3 {
    match f {
        0 => Vec3::new(1.0, -v, -u),
        1 => Vec3::new(-1.0, -v, u),
        2 => Vec3::new(u, 1.0, v),
        3 => Vec3::new(u, -1.0, -v),
        4 => Vec3::new(u, -v, 1.0),
        _ => Vec3::new(-u, -v, -1.0),
    }
}

fn sky_color(d: Vec3, p: &Params) -> Vec3 {
    let y = d.y;
    let mut c = if y >= 0.0 {
        p.horizon.lerp(p.zenith, y.powf(0.5))
    } else {
        p.horizon.lerp(p.ground, smoothstep(0.0, 0.3, -y))
    };
    if p.stars > 0.0 && y > 0.0 {
        let q = d * 180.0;
        let h = hash3(q.x.floor() as i32, q.y.floor() as i32, q.z.floor() as i32, 5);
        if h > 0.9965 {
            let b = (h - 0.9965) / 0.0035;
            c += Vec3::splat(b * p.stars * smoothstep(0.0, 0.25, y));
        }
    }
    let cs = d.dot(p.sun_dir).max(0.0);
    if y > 0.015 {
        let k = 1.0 / (y + 0.08);
        let px = d.x * k * 0.9 + 13.7;
        let pz = d.z * k * 0.9 + 4.1;
        let nse = fbm2(px, pz, 5, 77);
        let cover = smoothstep(p.cloud_lo, p.cloud_hi, nse) * smoothstep(0.015, 0.18, y);
        let shade = smoothstep(0.35, 0.75, fbm2(px * 1.7 + 2.0, pz * 1.7, 3, 78));
        let cc = p.cloud_dark.lerp(p.cloud_lit, (shade * 0.6 + cs * 0.6).min(1.0));
        c = c.lerp(cc, cover * 0.92);
    }
    let lxz = (d.x * d.x + d.z * d.z).sqrt().max(1e-5);
    let (ax, az) = (d.x / lxz, d.z / lxz);
    let far_h = 0.035 + 0.075 * fbm2(ax * 2.2 + 31.0, az * 2.2 + 17.0, 4, 91);
    let near_h = 0.01 + 0.06 * fbm2(ax * 3.5 + 7.0, az * 3.5 + 3.0, 4, 92);
    let base_mask = smoothstep(-0.12, -0.04, y);
    if y < far_h {
        c = c.lerp(p.mount_far.lerp(c, 0.25), base_mask);
    }
    if y < near_h {
        let t = ((y + 0.08) / (near_h + 0.08)).clamp(0.0, 1.0);
        c = c.lerp(p.mount_near * (0.8 + 0.3 * t), base_mask);
    }
    c
}
