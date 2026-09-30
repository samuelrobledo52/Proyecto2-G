use crate::camera::CamFrame;
use crate::entities::{DynHit, Entities};
use crate::material::*;
use crate::math::{halton, hash3, lerp, schlick, Vec3};
use crate::scene::{Scene, SX, SY, SZ};
use crate::skybox::{Sky, SkyState};
use crate::texture::{build_textures, Texture};
use std::sync::Mutex;

const MAX_DEPTH: u32 = 3;
const EPS: f32 = 1e-3;

pub struct Hit {
    pub t: f32,
    pub pos: Vec3,
    pub normal: Vec3,
    pub axis: usize,
    pub cell: (i32, i32, i32),
    pub block: u8,
}

struct Ctx<'a> {
    st: &'a SkyState,
    time: f32,
    frame: u32,
    jitter: bool,
}

pub struct World {
    pub scene: Scene,
    pub mats: Vec<Material>,
    pub textures: Vec<Texture>,
    pub sky: Sky,
    pub ents: Entities,
}

#[inline]
fn safe_inv(v: f32) -> f32 {
    if v.abs() < 1e-9 {
        1e9
    } else {
        1.0 / v
    }
}

#[inline]
fn face_uv(h: &Hit) -> (f32, f32) {
    let lx = (h.pos.x - h.cell.0 as f32).clamp(1e-4, 0.9999);
    let ly = (h.pos.y - h.cell.1 as f32).clamp(1e-4, 0.9999);
    let lz = (h.pos.z - h.cell.2 as f32).clamp(1e-4, 0.9999);
    match h.axis {
        0 => (lz, 1.0 - ly),
        1 => (lx, lz),
        _ => (lx, 1.0 - ly),
    }
}

#[inline]
fn rand_vec(p: Vec3, frame: u32, salt: u32) -> Vec3 {
    let (x, y, z) = ((p.x * 131.0) as i32, (p.y * 131.0) as i32, (p.z * 131.0) as i32);
    let s = frame.wrapping_mul(7919).wrapping_add(salt);
    Vec3::new(
        hash3(x, y, z, s) - 0.5,
        hash3(x, y, z, s ^ 0xabc) - 0.5,
        hash3(x, y, z, s ^ 0x1234) - 0.5,
    ) * 2.0
}

impl World {
    pub fn new() -> World {
        let scene = Scene::build();
        let ents = Entities::new(&scene);
        World {
            scene,
            mats: materials(),
            textures: build_textures(),
            sky: Sky::new(),
            ents,
        }
    }

    pub fn update(&mut self, dt: f32, time: f32, night: f32) {
        self.ents.update(&self.scene, dt, time, night);
    }

    #[inline]
    fn medium_at(&self, p: Vec3) -> u8 {
        let b = self
            .scene
            .get(p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32);
        let m = &self.mats[b as usize];
        if b != AIR && m.transparency > 0.0 && !m.cutout {
            b
        } else {
            AIR
        }
    }

    fn trace_grid(&self, ro: Vec3, rd: Vec3, medium: u8, max_t: f32) -> Option<Hit> {
        let inv = [safe_inv(rd.x), safe_inv(rd.y), safe_inv(rd.z)];
        let o = [ro.x, ro.y, ro.z];
        let size = [SX, SY, SZ];

        let mut tmin = 0.0f32;
        let mut tmax = f32::INFINITY;
        let mut entry_axis: Option<usize> = None;
        for a in 0..3 {
            let mut t1 = (0.0 - o[a]) * inv[a];
            let mut t2 = (size[a] as f32 - o[a]) * inv[a];
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            if t1 > tmin {
                tmin = t1;
                entry_axis = Some(a);
            }
            if t2 < tmax {
                tmax = t2;
            }
        }
        if tmax <= tmin || tmin > max_t {
            return None;
        }
        let started_inside = entry_axis.is_none();
        let p = if started_inside {
            ro
        } else {
            ro + rd * (tmin + 1e-4)
        };
        let pa = [p.x, p.y, p.z];

        let mut cell = [0i32; 3];
        let mut step = [0i32; 3];
        let mut t_next = [0f32; 3];
        let mut t_delta = [0f32; 3];
        for a in 0..3 {
            cell[a] = (pa[a].floor() as i32).clamp(0, size[a] - 1);
            step[a] = if inv[a] >= 0.0 { 1 } else { -1 };
            t_delta[a] = inv[a].abs();
            let boundary = if step[a] > 0 {
                (cell[a] + 1) as f32
            } else {
                cell[a] as f32
            };
            t_next[a] = (boundary - o[a]) * inv[a];
        }

        let mut t = tmin;
        let mut axis = entry_axis.unwrap_or(0);
        let mut first = true;
        loop {
            if !(first && started_inside) {
                let b = self.scene.get(cell[0], cell[1], cell[2]);
                if b != medium {
                    let mut normal = Vec3::ZERO;
                    match axis {
                        0 => normal.x = -step[0] as f32,
                        1 => normal.y = -step[1] as f32,
                        _ => normal.z = -step[2] as f32,
                    }
                    let hit = Hit {
                        t,
                        pos: ro + rd * t,
                        normal,
                        axis,
                        cell: (cell[0], cell[1], cell[2]),
                        block: b,
                    };
                    let m = &self.mats[b as usize];
                    if !m.cutout {
                        return Some(hit);
                    }
                    let (u, v) = face_uv(&hit);
                    if self.textures[m.tex_for(normal)].alpha(u, v) >= 0.5 {
                        return Some(hit);
                    }
                }
            }
            first = false;

            if medium == AIR {
                let k = self.scene.empty_dist(cell[0], cell[1], cell[2]) as i32;
                if k >= 3 {
                    let r = (k - 1) as f32;
                    let mut t_exit = f32::INFINITY;
                    for a in 0..3 {
                        let bound = if step[a] > 0 {
                            cell[a] as f32 + 1.0 + r
                        } else {
                            cell[a] as f32 - r
                        };
                        let ta = (bound - o[a]) * inv[a];
                        if ta < t_exit {
                            t_exit = ta;
                        }
                    }
                    let tj = t_exit - 1e-3;
                    if tj > t + 0.5 {
                        if tj > max_t {
                            return None;
                        }
                        let pj = ro + rd * tj;
                        for a in 0..3 {
                            let c = pj.get(a).floor() as i32;
                            if c < 0 || c >= size[a] {
                                return None;
                            }
                            cell[a] = c;
                            let boundary = if step[a] > 0 { (c + 1) as f32 } else { c as f32 };
                            t_next[a] = (boundary - o[a]) * inv[a];
                        }
                        t = tj;
                        continue;
                    }
                }
            }

            let a = if t_next[0] < t_next[1] {
                if t_next[0] < t_next[2] {
                    0
                } else {
                    2
                }
            } else if t_next[1] < t_next[2] {
                1
            } else {
                2
            };
            t = t_next[a];
            if t > max_t {
                return None;
            }
            cell[a] += step[a];
            if cell[a] < 0 || cell[a] >= size[a] {
                return None;
            }
            t_next[a] += t_delta[a];
            axis = a;
        }
    }

    fn shadow(&self, origin: Vec3, dir: Vec3, max_t: f32, to_point: bool) -> Vec3 {
        if self.ents.intersect(origin, dir, max_t, true).is_some() {
            return Vec3::ZERO;
        }
        let mut trans = Vec3::ONE;
        let start = self.medium_at(origin);
        if start != AIR {
            let m = &self.mats[start as usize];
            trans = trans * m.tint * m.transparency;
        }
        let mut o = origin;
        let mut remaining = max_t;
        for _ in 0..8 {
            let medium = self.medium_at(o);
            let hit = match self.trace_grid(o, dir, medium, remaining) {
                None => return trans,
                Some(h) => h,
            };
            if hit.block != AIR {
                let m = &self.mats[hit.block as usize];
                if to_point && m.emission > 0.0 {
                    return trans;
                }
                if m.transparency <= 0.0 {
                    return Vec3::ZERO;
                }
                trans = trans * m.tint * m.transparency;
                if trans.max_elem() < 0.02 {
                    return Vec3::ZERO;
                }
            }
            o = hit.pos + dir * EPS;
            remaining -= hit.t + EPS;
            if remaining <= 0.0 {
                return trans;
            }
        }
        trans
    }

    #[inline]
    fn occludes(&self, x: i32, y: i32, z: i32) -> bool {
        let b = self.scene.get(x, y, z);
        b != AIR && self.mats[b as usize].transparency <= 0.0
    }

    fn ao(&self, h: &Hit) -> f32 {
        let n = [h.normal.x as i32, h.normal.y as i32, h.normal.z as i32];
        let front = [h.cell.0 + n[0], h.cell.1 + n[1], h.cell.2 + n[2]];
        let (a1, a2) = match h.axis {
            0 => (1, 2),
            1 => (0, 2),
            _ => (0, 1),
        };
        let local = [
            h.pos.x - h.cell.0 as f32,
            h.pos.y - h.cell.1 as f32,
            h.pos.z - h.cell.2 as f32,
        ];
        let fu = local[a1].clamp(0.0, 1.0);
        let fv = local[a2].clamp(0.0, 1.0);
        let mut c = [[1.0f32; 2]; 2];
        for (i, s1) in [-1i32, 1].iter().enumerate() {
            for (j, s2) in [-1i32, 1].iter().enumerate() {
                let mut p1 = front;
                p1[a1] += s1;
                let mut p2 = front;
                p2[a2] += s2;
                let mut pc = front;
                pc[a1] += s1;
                pc[a2] += s2;
                let side1 = self.occludes(p1[0], p1[1], p1[2]);
                let side2 = self.occludes(p2[0], p2[1], p2[2]);
                let corner = self.occludes(pc[0], pc[1], pc[2]);
                c[i][j] = if side1 && side2 {
                    0.0
                } else {
                    (3 - (side1 as i32 + side2 as i32 + corner as i32)) as f32 / 3.0
                };
            }
        }
        let a = lerp(c[0][0], c[1][0], fu);
        let b = lerp(c[0][1], c[1][1], fu);
        lerp(a, b, fv)
    }

    fn surface_color(&self, h: &Hit, mid: u8, ctx: &Ctx) -> Vec3 {
        let m = &self.mats[mid as usize];
        let (mut u, mut v) = face_uv(h);
        let t = ctx.time;
        if mid == WATER {
            if h.axis == 1 {
                u += t * 0.12;
                v += t * 0.07;
            } else {
                v -= t * 1.6;
            }
        } else if mid == PORTAL {
            u += (v * 6.0 + t * 2.0).sin() * 0.08;
            v += t * 0.3;
        }
        self.textures[m.tex_for(h.normal)].color(u, v) * m.albedo
    }

    #[allow(clippy::too_many_arguments)]
    fn lighting(
        &self,
        pos: Vec3,
        geo_n: Vec3,
        n: Vec3,
        view: Vec3,
        ks: f32,
        shin: f32,
        ao: f32,
        ctx: &Ctx,
    ) -> (Vec3, Vec3) {
        let st = ctx.st;
        let p = pos + geo_n * EPS;
        let up = n.y * 0.5 + 0.5;
        let mut diff = st.amb_ground.lerp(st.amb_sky, up) * (0.3 + 0.7 * ao);
        let mut spec = Vec3::ZERO;

        if !st.key_color.is_black() {
            let mut kd = st.key_dir;
            if ctx.jitter {
                kd = (kd + rand_vec(p, ctx.frame, 1) * 0.02).normalize();
            }
            let ndl = n.dot(kd);
            if ndl > 0.0 && geo_n.dot(kd) > 0.0 {
                let sh = self.shadow(p, kd, f32::INFINITY, false);
                if !sh.is_black() {
                    diff += st.key_color * sh * ndl;
                    let hv = (kd + view).normalize();
                    spec += st.key_color * sh * (hv.dot(n).max(0.0).powf(shin) * ks);
                }
            }
        }

        for l in &self.scene.lights {
            let mut lpos = l.pos;
            if ctx.jitter {
                lpos = lpos + rand_vec(p, ctx.frame, 2) * 0.35;
            }
            let to = lpos - p;
            let dist = to.length();
            if dist > l.radius || dist < 1e-4 {
                continue;
            }
            let ld = to / dist;
            let ndl = n.dot(ld);
            if ndl <= 0.0 || geo_n.dot(ld) <= 0.0 {
                continue;
            }
            let x = dist / l.radius;
            let k = if l.scaled { st.lamp } else { 1.0 };
            let fall = (1.0 - x * x).powi(2) / (1.0 + 0.12 * dist * dist);
            let sh = self.shadow(p, ld, dist, true);
            if sh.is_black() {
                continue;
            }
            let lc = l.color * (l.intensity * k * fall) * sh;
            diff += lc * ndl;
            let hv = (ld + view).normalize();
            spec += lc * (hv.dot(n).max(0.0).powf(shin) * ks);
        }
        (diff, spec)
    }

    fn shade_dynamic(&self, d: &DynHit, rd: Vec3, ctx: &Ctx) -> Vec3 {
        let s = self.ents.surface(d, ctx.time);
        if s.emission > 0.0 {
            return s.color * (1.0 + s.emission);
        }
        let (diff, spec) =
            self.lighting(d.pos, d.normal, d.normal, -rd, s.specular, s.shininess, 1.0, ctx);
        s.color * diff + spec
    }

    fn trace_color(&self, ro: Vec3, rd: Vec3, depth: u32, ctx: &Ctx) -> Vec3 {
        let medium = self.medium_at(ro);
        let dh = self.ents.intersect(ro, rd, f32::INFINITY, false);
        let max_t = dh.as_ref().map_or(f32::INFINITY, |d| d.t);
        let h = match self.trace_grid(ro, rd, medium, max_t) {
            Some(h) => h,
            None => {
                return match dh {
                    Some(d) => self.shade_dynamic(&d, rd, ctx),
                    None => self.sky.color(rd, ctx.st),
                };
            }
        };
        let view = -rd;
        let mb = &self.mats[h.block as usize];

        if mb.transparency <= 0.0 {
            let base = self.surface_color(&h, h.block, ctx);
            if mb.emission > 0.0 {
                return base * (1.0 + mb.emission);
            }
            let ao = self.ao(&h);
            let (diff, spec) =
                self.lighting(h.pos, h.normal, h.normal, view, mb.specular, mb.shininess, ao, ctx);
            let mut c = base * diff + spec;
            if mb.reflectivity > 0.0 && depth < MAX_DEPTH {
                let r = rd.reflect(h.normal);
                let rc = self.trace_color(h.pos + h.normal * EPS, r, depth + 1, ctx);
                c = c * (1.0 - mb.reflectivity) + rc * mb.reflectivity;
            }
            return c;
        }

        let mm = &self.mats[medium as usize];
        let surf_id = if h.block != AIR { h.block } else { medium };
        let surf = &self.mats[surf_id as usize];

        let mut n = h.normal;
        if surf_id == WATER {
            let t = ctx.time;
            if h.axis == 1 {
                let wx = (h.pos.x * 2.1 + t * 1.7).sin() * 0.05 + (h.pos.z * 3.3 + t * 1.1).cos() * 0.03;
                let wz = (h.pos.z * 2.5 - t * 1.4).sin() * 0.05 + (h.pos.x * 2.9 - t * 0.9).cos() * 0.03;
                n = (n + Vec3::new(wx, 0.0, wz)).normalize();
            } else {
                let wy = (h.pos.y * 3.0 - t * 8.0).sin() * 0.08;
                n = (n + Vec3::new(0.0, wy, 0.0)).normalize();
            }
        }

        let base = self.surface_color(&h, surf_id, ctx);
        let (diff, spec) =
            self.lighting(h.pos, h.normal, n, view, surf.specular, surf.shininess, 1.0, ctx);
        let glow = base * surf.emission;
        if depth >= MAX_DEPTH {
            return base * diff + spec + glow;
        }

        let cosi = (-rd.dot(n)).clamp(0.0, 1.0);
        let fres = schlick(cosi, mm.ior, mb.ior);
        let refr = rd.refract(n, mm.ior / mb.ior);

        let mut wr = surf.reflectivity.max(fres);
        let mut wt = surf.transparency * (1.0 - wr);
        if refr.is_none() {
            wr = (wr + wt).min(1.0);
            wt = 0.0;
        }
        let wl = (1.0 - wr - wt).max(0.0);

        let mut col = base * diff * wl + spec + glow;
        if wr > 0.001 {
            let r = rd.reflect(n);
            col += self.trace_color(h.pos + h.normal * EPS, r, depth + 1, ctx) * wr;
        }
        if let Some(d) = refr {
            if wt > 0.001 {
                let mut rc = self.trace_color(h.pos - h.normal * EPS, d, depth + 1, ctx);
                if h.block != AIR {
                    rc = rc * mb.tint;
                }
                col += rc * wt;
            }
        }
        col
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        hdr: &mut [Vec3],
        w: usize,
        h: usize,
        cam: &CamFrame,
        st: &SkyState,
        time: f32,
        frame: u32,
        jitter: bool,
    ) {
        let ctx = Ctx {
            st,
            time,
            frame,
            jitter,
        };
        let (jx, jy) = if jitter {
            (halton(frame + 1, 2), halton(frame + 1, 3))
        } else {
            (0.5, 0.5)
        };
        let threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let band = 4usize;
        let jobs: Vec<(usize, &mut [Vec3])> = hdr.chunks_mut(w * band).enumerate().collect();
        let queue = Mutex::new(jobs);

        std::thread::scope(|s| {
            for _ in 0..threads {
                s.spawn(|| loop {
                    let job = queue.lock().unwrap().pop();
                    let Some((bi, chunk)) = job else {
                        break;
                    };
                    for (ri, row) in chunk.chunks_mut(w).enumerate() {
                        let y = bi * band + ri;
                        for (x, px) in row.iter_mut().enumerate() {
                            let sx = ((x as f32 + jx) / w as f32) * 2.0 - 1.0;
                            let sy = 1.0 - ((y as f32 + jy) / h as f32) * 2.0;
                            *px = self.trace_color(cam.eye, cam.ray(sx, sy), 0, &ctx);
                        }
                    }
                });
            }
        });
    }
}
