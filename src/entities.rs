use crate::material::*;
use crate::math::{hash3, Mat3, Rng, Vec3};
use crate::scene::{Scene, CRYSTAL_POS, MAIN_TOP, SX, SZ};
use std::f32::consts::PI;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PartKind {
    ZombieHead,
    ZombieSkin,
    ZombieShirt,
    ZombiePants,
    CreeperHead,
    CreeperBody,
    EnderHead,
    EnderBody,
    DragonScale,
    DragonHead,
    DragonWing,
    DragonBone,
    Crystal,
    CrystalCage,
}

impl PartKind {
    fn casts_shadow(self) -> bool {
        !matches!(self, PartKind::Crystal | PartKind::CrystalCage)
    }
}

#[derive(Clone, Copy)]
pub struct Part {
    pub center: Vec3,
    pub half: Vec3,
    pub rot: Mat3,
    pub kind: PartKind,
}

struct Group {
    center: Vec3,
    radius: f32,
    start: usize,
    end: usize,
}

pub struct DynHit {
    pub t: f32,
    pub pos: Vec3,
    pub normal: Vec3,
    local_p: Vec3,
    local_n: Vec3,
    part: usize,
}

pub struct Surface {
    pub color: Vec3,
    pub emission: f32,
    pub specular: f32,
    pub shininess: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum MobKind {
    Zombie,
    Creeper,
    Enderman,
}

#[derive(Clone, Copy)]
struct Mob {
    kind: MobKind,
    x: f32,
    z: f32,
    yaw: f32,
    target_yaw: f32,
    timer: f32,
    phase: f32,
}

pub struct Entities {
    parts: Vec<Part>,
    groups: Vec<Group>,
    mobs: Vec<Mob>,
    rng: Rng,
    pub mobs_visible: bool,
}

#[inline]
fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

#[inline]
fn axis_vec(a: usize, s: f32) -> Vec3 {
    match a {
        0 => v(s, 0.0, 0.0),
        1 => v(0.0, s, 0.0),
        _ => v(0.0, 0.0, s),
    }
}

fn walkable(scene: &Scene, x: f32, z: f32) -> bool {
    let cx = x.floor() as i32;
    let cz = z.floor() as i32;
    let g = scene.get(cx, MAIN_TOP, cz);
    matches!(g, GRASS | SAND | COBBLE | DIRT | STONE)
        && scene.get(cx, MAIN_TOP + 1, cz) == AIR
        && scene.get(cx, MAIN_TOP + 2, cz) == AIR
        && scene.get(cx, MAIN_TOP + 3, cz) == AIR
}

fn intersect_part(p: &Part, ro: Vec3, rd: Vec3, max_t: f32) -> Option<(f32, Vec3, Vec3)> {
    let lo = p.rot.tmul_vec(ro - p.center);
    let ld = p.rot.tmul_vec(rd);
    let mut tn = f32::NEG_INFINITY;
    let mut tf = f32::INFINITY;
    let mut an = 0usize;
    let mut af = 0usize;
    for a in 0..3 {
        let o = lo.get(a);
        let d = ld.get(a);
        let h = p.half.get(a);
        if d.abs() < 1e-9 {
            if o.abs() > h {
                return None;
            }
            continue;
        }
        let inv = 1.0 / d;
        let mut t1 = (-h - o) * inv;
        let mut t2 = (h - o) * inv;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }
        if t1 > tn {
            tn = t1;
            an = a;
        }
        if t2 < tf {
            tf = t2;
            af = a;
        }
    }
    if tn > tf || tf < 1e-4 || tn > max_t {
        return None;
    }
    let cutout = p.kind == PartKind::CrystalCage;
    let cands = [(tn, an), (tf, af)];
    for (i, &(t, a)) in cands.iter().enumerate() {
        if i == 1 && !cutout {
            break;
        }
        if t < 1e-4 || t > max_t {
            continue;
        }
        let lp = lo + ld * t;
        if cutout {
            let (b1, b2) = match a {
                0 => (1, 2),
                1 => (0, 2),
                _ => (0, 1),
            };
            let u = (lp.get(b1) / p.half.get(b1)).abs();
            let w = (lp.get(b2) / p.half.get(b2)).abs();
            if u < 0.78 && w < 0.78 {
                continue;
            }
        }
        let ln = axis_vec(a, -ld.get(a).signum());
        return Some((t, lp, ln));
    }
    None
}

impl Entities {
    pub fn new(scene: &Scene) -> Self {
        let mut rng = Rng::new(0x5eed_1234);
        let mut spots: Vec<(f32, f32)> = Vec::new();
        for z in 0..SZ {
            for x in 0..SX {
                let (fx, fz) = (x as f32 + 0.5, z as f32 + 0.5);
                if walkable(scene, fx, fz) {
                    spots.push((fx, fz));
                }
            }
        }
        let kinds = [
            MobKind::Zombie,
            MobKind::Zombie,
            MobKind::Creeper,
            MobKind::Creeper,
            MobKind::Enderman,
        ];
        let mut mobs: Vec<Mob> = Vec::new();
        let mut tries = 0;
        while mobs.len() < kinds.len() && !spots.is_empty() && tries < 2000 {
            tries += 1;
            let i = ((rng.next_f32() * spots.len() as f32) as usize).min(spots.len() - 1);
            let (x, z) = spots[i];
            if mobs.iter().any(|m| (m.x - x).powi(2) + (m.z - z).powi(2) < 9.0) {
                continue;
            }
            let yaw = rng.next_f32() * 2.0 * PI;
            mobs.push(Mob {
                kind: kinds[mobs.len()],
                x,
                z,
                yaw,
                target_yaw: yaw,
                timer: 1.0 + rng.next_f32() * 2.0,
                phase: 0.0,
            });
        }
        Entities {
            parts: Vec::new(),
            groups: Vec::new(),
            mobs,
            rng,
            mobs_visible: false,
        }
    }

    pub fn update(&mut self, scene: &Scene, dt: f32, time: f32, night: f32) {
        self.mobs_visible = night > 0.55;
        if self.mobs_visible {
            for i in 0..self.mobs.len() {
                let r1 = self.rng.next_f32();
                let r2 = self.rng.next_f32();
                let m = &mut self.mobs[i];
                m.timer -= dt;
                if m.timer <= 0.0 {
                    m.target_yaw = m.yaw + (r1 - 0.5) * 2.6;
                    m.timer = 2.0 + r2 * 3.0;
                }
                let mut diff = m.target_yaw - m.yaw;
                while diff > PI {
                    diff -= 2.0 * PI;
                }
                while diff < -PI {
                    diff += 2.0 * PI;
                }
                m.yaw += diff * (dt * 3.0).min(1.0);
                let speed = match m.kind {
                    MobKind::Zombie => 0.9,
                    MobKind::Creeper => 0.7,
                    MobKind::Enderman => 1.1,
                };
                let (dx, dz) = (m.yaw.sin(), m.yaw.cos());
                let nx = m.x + dx * speed * dt;
                let nz = m.z + dz * speed * dt;
                if walkable(scene, nx + dx * 0.45, nz + dz * 0.45) && walkable(scene, nx, nz) {
                    m.x = nx;
                    m.z = nz;
                    m.phase += dt * speed * 6.0;
                } else {
                    m.target_yaw = m.yaw + PI * (0.6 + r1 * 0.8);
                    m.timer = 1.5;
                }
            }
        }
        self.rebuild(time);
    }

    fn rebuild(&mut self, time: f32) {
        self.parts.clear();
        self.groups.clear();
        self.build_dragon(time);
        self.build_crystal(time);
        if self.mobs_visible {
            for i in 0..self.mobs.len() {
                let m = self.mobs[i];
                self.build_mob(&m);
            }
        }
    }

    fn add(&mut self, bp: Vec3, br: &Mat3, off: Vec3, rot: Mat3, half: Vec3, kind: PartKind) {
        self.parts.push(Part {
            center: bp + br.mul_vec(off),
            half,
            rot: br.mul(&rot),
            kind,
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn add_piv(
        &mut self,
        bp: Vec3,
        br: &Mat3,
        pivot: Vec3,
        rot: Mat3,
        off: Vec3,
        half: Vec3,
        kind: PartKind,
    ) {
        let c = pivot + rot.mul_vec(off);
        self.add(bp, br, c, rot, half, kind);
    }

    fn group(&mut self, start: usize, center: Vec3, radius: f32) {
        self.groups.push(Group {
            center,
            radius,
            start,
            end: self.parts.len(),
        });
    }

    fn build_mob(&mut self, m: &Mob) {
        use PartKind::*;
        let b = v(m.x, MAIN_TOP as f32 + 1.0, m.z);
        let r = Mat3::rot_y(m.yaw);
        let sw = m.phase.sin() * 0.6;
        let start = self.parts.len();
        let id = Mat3::IDENTITY;
        match m.kind {
            MobKind::Zombie => {
                let leg = v(0.125, 0.375, 0.125);
                self.add_piv(b, &r, v(-0.125, 0.75, 0.0), Mat3::rot_x(sw), v(0.0, -0.375, 0.0), leg, ZombiePants);
                self.add_piv(b, &r, v(0.125, 0.75, 0.0), Mat3::rot_x(-sw), v(0.0, -0.375, 0.0), leg, ZombiePants);
                self.add(b, &r, v(0.0, 1.125, 0.0), id, v(0.25, 0.375, 0.125), ZombieShirt);
                let arm = v(0.125, 0.125, 0.375);
                self.add_piv(b, &r, v(-0.375, 1.375, 0.0), Mat3::rot_x(0.1 + sw * 0.15), v(0.0, 0.0, 0.3), arm, ZombieSkin);
                self.add_piv(b, &r, v(0.375, 1.375, 0.0), Mat3::rot_x(0.1 - sw * 0.15), v(0.0, 0.0, 0.3), arm, ZombieSkin);
                self.add(b, &r, v(0.0, 1.75, 0.0), id, v(0.25, 0.25, 0.25), ZombieHead);
                self.group(start, b + v(0.0, 1.0, 0.0), 1.3);
            }
            MobKind::Creeper => {
                let leg = v(0.125, 0.1875, 0.125);
                for (sx, sz, s) in [(-0.125, 0.25, 1.0), (0.125, 0.25, -1.0), (-0.125, -0.25, -1.0), (0.125, -0.25, 1.0)] {
                    self.add_piv(b, &r, v(sx, 0.375, sz), Mat3::rot_x(sw * s), v(0.0, -0.1875, 0.0), leg, CreeperBody);
                }
                self.add(b, &r, v(0.0, 0.75, 0.0), id, v(0.25, 0.375, 0.125), CreeperBody);
                self.add(b, &r, v(0.0, 1.375, 0.0), id, v(0.25, 0.25, 0.25), CreeperHead);
                self.group(start, b + v(0.0, 0.8, 0.0), 1.0);
            }
            MobKind::Enderman => {
                let leg = v(0.0625, 0.9375, 0.0625);
                self.add_piv(b, &r, v(-0.125, 1.875, 0.0), Mat3::rot_x(sw * 0.7), v(0.0, -0.9375, 0.0), leg, EnderBody);
                self.add_piv(b, &r, v(0.125, 1.875, 0.0), Mat3::rot_x(-sw * 0.7), v(0.0, -0.9375, 0.0), leg, EnderBody);
                self.add(b, &r, v(0.0, 2.25, 0.0), id, v(0.25, 0.375, 0.125), EnderBody);
                let arm = v(0.0625, 0.9, 0.0625);
                self.add_piv(b, &r, v(-0.3125, 2.6, 0.0), Mat3::rot_x(-sw * 0.5), v(0.0, -0.9, 0.0), arm, EnderBody);
                self.add_piv(b, &r, v(0.3125, 2.6, 0.0), Mat3::rot_x(sw * 0.5), v(0.0, -0.9, 0.0), arm, EnderBody);
                self.add(b, &r, v(0.0, 2.875, 0.0), id, v(0.25, 0.25, 0.25), EnderHead);
                self.group(start, b + v(0.0, 1.6, 0.0), 1.9);
            }
        }
    }

    fn build_dragon(&mut self, time: f32) {
        use PartKind::*;
        let a = time * 0.22;
        let radius = 25.0;
        let pos = v(20.0 + radius * a.cos(), 24.0 + (time * 0.6).sin() * 2.0, 20.0 + radius * a.sin());
        let yaw = (-a.sin()).atan2(a.cos());
        let r = Mat3::rot_y(yaw)
            .mul(&Mat3::rot_z(0.22))
            .mul(&Mat3::rot_x(-0.08 * (time * 0.6).cos()));
        let id = Mat3::IDENTITY;
        let start = self.parts.len();

        self.add(pos, &r, v(0.0, 0.0, 0.0), id, v(1.0, 0.8, 2.2), DragonScale);
        for i in 0..4 {
            self.add(pos, &r, v(0.0, 0.95, -1.5 + i as f32 * 1.0), id, v(0.08, 0.22, 0.18), DragonBone);
        }
        for i in 0..3 {
            let fi = i as f32;
            let y = 0.35 + fi * 0.18 + (time * 1.5 - fi * 0.5).sin() * 0.08;
            self.add(pos, &r, v(0.0, y, 2.5 + fi * 0.85), id, v(0.42, 0.42, 0.45), DragonScale);
        }
        let hy = 0.95 + (time * 1.5 - 1.8).sin() * 0.1;
        self.add(pos, &r, v(0.0, hy, 5.05), id, v(0.7, 0.55, 0.75), DragonHead);
        self.add(pos, &r, v(0.0, hy - 0.1, 6.25), id, v(0.45, 0.32, 0.5), DragonScale);
        self.add_piv(
            pos,
            &r,
            v(0.0, hy - 0.42, 5.4),
            Mat3::rot_x(0.15 + 0.12 * (time * 2.0).sin()),
            v(0.0, 0.0, 0.75),
            v(0.38, 0.1, 0.75),
            DragonScale,
        );
        for sx in [-0.4f32, 0.4] {
            self.add(pos, &r, v(sx, hy + 0.7, 4.7), Mat3::rot_x(-0.5), v(0.09, 0.3, 0.09), DragonBone);
        }
        for (sx, sz) in [(-1.0f32, 1.0f32), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
            self.add(pos, &r, v(0.75 * sx, -0.85, 1.3 * sz), id, v(0.22, 0.4, 0.3), DragonScale);
        }
        for i in 0..7 {
            let fi = i as f32;
            let z = -2.7 - fi * 0.8;
            let x = (time * 1.8 - fi * 0.55).sin() * 0.12 * fi;
            let y = -0.1 - fi * 0.05 + (time * 1.2 - fi * 0.4).sin() * 0.06 * fi;
            let s = 1.0 - fi * 0.09;
            self.add(pos, &r, v(x, y, z), id, v(0.42 * s, 0.36 * s, 0.42), DragonScale);
        }
        let flap = (time * 3.2).sin();
        for side in [-1.0f32, 1.0] {
            let roll1 = side * (0.15 + 0.55 * flap);
            let r1 = Mat3::rot_z(roll1);
            let pivot1 = v(side * 0.95, 0.55, 0.6);
            self.add_piv(pos, &r, pivot1, r1, v(side * 1.4, 0.0, 0.0), v(1.4, 0.07, 1.25), DragonWing);
            self.add_piv(pos, &r, pivot1, r1, v(side * 1.4, 0.06, 1.2), v(1.4, 0.1, 0.1), DragonBone);
            let pivot2 = pivot1 + r1.mul_vec(v(side * 2.8, 0.0, 0.0));
            let roll2 = roll1 + side * (0.25 * (time * 3.2 - 0.7).sin() - 0.1);
            let r2 = Mat3::rot_z(roll2);
            self.add_piv(pos, &r, pivot2, r2, v(side * 1.6, 0.0, -0.35), v(1.6, 0.05, 1.05), DragonWing);
            self.add_piv(pos, &r, pivot2, r2, v(side * 1.6, 0.05, 0.65), v(1.6, 0.09, 0.09), DragonBone);
        }
        self.group(start, pos, 9.5);
    }

    fn build_crystal(&mut self, time: f32) {
        use PartKind::*;
        let start = self.parts.len();
        let c = CRYSTAL_POS + v(0.0, (time * 1.5).sin() * 0.3, 0.0);
        let id = Mat3::IDENTITY;
        let rc = Mat3::rot_y(time).mul(&Mat3::rot_x(time * 0.7));
        self.add(c, &id, Vec3::ZERO, rc, v(0.28, 0.28, 0.28), Crystal);
        let r1 = Mat3::rot_y(-time * 0.8).mul(&Mat3::rot_z(0.6));
        self.add(c, &id, Vec3::ZERO, r1, v(0.55, 0.55, 0.55), CrystalCage);
        let r2 = Mat3::rot_x(time * 0.9).mul(&Mat3::rot_y(0.785));
        self.add(c, &id, Vec3::ZERO, r2, v(0.42, 0.42, 0.42), CrystalCage);
        self.group(start, c, 1.0);
    }

    pub fn intersect(&self, ro: Vec3, rd: Vec3, max_t: f32, shadow_only: bool) -> Option<DynHit> {
        let mut best: Option<DynHit> = None;
        let mut bt = max_t;
        for g in &self.groups {
            let oc = ro - g.center;
            let b = oc.dot(rd);
            let c = oc.dot(oc) - g.radius * g.radius;
            if c > 0.0 && b > 0.0 {
                continue;
            }
            let disc = b * b - c;
            if disc < 0.0 || -b - disc.sqrt() > bt {
                continue;
            }
            for i in g.start..g.end {
                let p = &self.parts[i];
                if shadow_only && !p.kind.casts_shadow() {
                    continue;
                }
                if let Some((t, lp, ln)) = intersect_part(p, ro, rd, bt) {
                    bt = t;
                    best = Some(DynHit {
                        t,
                        pos: ro + rd * t,
                        normal: p.rot.mul_vec(ln),
                        local_p: lp,
                        local_n: ln,
                        part: i,
                    });
                    if shadow_only {
                        return best;
                    }
                }
            }
        }
        best
    }

    pub fn surface(&self, h: &DynHit, time: f32) -> Surface {
        use PartKind::*;
        let p = &self.parts[h.part];
        let lp = h.local_p;
        let ln = h.local_n;
        let px = (lp.x * 16.0).floor() as i32;
        let py = (lp.y * 16.0).floor() as i32;
        let pz = (lp.z * 16.0).floor() as i32;
        let nz = hash3(px, py, pz, 301);
        let front = ln.z > 0.5;
        let col = (((lp.x / p.half.x) * 0.5 + 0.5) * 8.0).floor().clamp(0.0, 7.0) as i32;
        let row = ((0.5 - (lp.y / p.half.y) * 0.5) * 8.0).floor().clamp(0.0, 7.0) as i32;
        let rgb = Vec3::rgb;
        let mut s = Surface {
            color: Vec3::ZERO,
            emission: 0.0,
            specular: 0.1,
            shininess: 12.0,
        };
        match p.kind {
            ZombieHead | ZombieSkin => {
                s.color = rgb(82, 135, 70) * (0.82 + 0.3 * nz);
                if p.kind == ZombieHead && front {
                    if row == 4 && matches!(col, 1 | 2 | 5 | 6) {
                        s.color = rgb(28, 36, 30);
                    } else if row == 5 && matches!(col, 3 | 4) {
                        s.color = rgb(58, 98, 50);
                    } else if row == 6 && (2..=5).contains(&col) {
                        s.color = rgb(45, 72, 40);
                    }
                }
            }
            ZombieShirt => s.color = rgb(42, 158, 170) * (0.8 + 0.3 * nz),
            ZombiePants => s.color = rgb(62, 60, 150) * (0.8 + 0.3 * nz),
            CreeperHead | CreeperBody => {
                let pal = [rgb(70, 170, 60), rgb(100, 200, 90), rgb(52, 125, 46), rgb(150, 220, 140)];
                s.color = pal[((nz * 4.0) as usize).min(3)];
                if p.kind == CreeperHead && front {
                    let eye = (row == 2 || row == 3) && matches!(col, 1 | 2 | 5 | 6);
                    let nose = row == 4 && matches!(col, 3 | 4);
                    let mouth = (row == 5 || row == 6) && (2..=5).contains(&col);
                    let fangs = row == 7 && matches!(col, 2 | 5);
                    if eye || nose || mouth || fangs {
                        s.color = rgb(12, 12, 12);
                    }
                }
            }
            EnderHead | EnderBody => {
                s.color = rgb(22, 18, 30) * (0.8 + 0.4 * nz);
                s.specular = 0.3;
                if p.kind == EnderHead && front && row == 4 && matches!(col, 0 | 1 | 2 | 5 | 6 | 7) {
                    s.color = if matches!(col, 1 | 6) {
                        rgb(250, 190, 255)
                    } else {
                        rgb(200, 90, 240)
                    };
                    s.emission = 1.6;
                }
            }
            DragonScale | DragonHead => {
                s.color = rgb(32, 30, 38) * (0.75 + 0.5 * nz);
                if nz > 0.88 {
                    s.color = rgb(62, 44, 80);
                }
                s.specular = 0.5;
                s.shininess = 32.0;
                if p.kind == DragonHead && ln.x.abs() > 0.5 {
                    let ez = lp.z - p.half.z * 0.45;
                    let ey = lp.y - p.half.y * 0.3;
                    if ez.abs() < 0.14 && ey.abs() < 0.09 {
                        s.color = rgb(225, 120, 255);
                        s.emission = 2.0;
                    }
                }
            }
            DragonWing => {
                s.color = rgb(58, 50, 66) * (0.8 + 0.25 * nz);
                s.specular = 0.2;
            }
            DragonBone => s.color = rgb(205, 198, 182) * (0.85 + 0.2 * nz),
            Crystal => {
                s.color = rgb(255, 120, 225);
                s.emission = 1.8 * (0.8 + 0.2 * (time * 3.0).sin());
            }
            CrystalCage => {
                s.color = rgb(235, 225, 255);
                s.emission = 0.9;
            }
        }
        s
    }
}
