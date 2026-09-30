use crate::math::Vec3;

const GAUSS: [f32; 5] = [0.227_027, 0.194_594_6, 0.121_621_6, 0.054_054, 0.016_216];
const DS: usize = 4;

pub struct Post {
    w: usize,
    h: usize,
    accum: Vec<Vec3>,
    count: u32,
    bw: usize,
    bh: usize,
    bloom: Vec<Vec3>,
    tmp: Vec<Vec3>,
}

impl Post {
    pub fn new(w: usize, h: usize) -> Post {
        let bw = (w / DS).max(1);
        let bh = (h / DS).max(1);
        Post {
            w,
            h,
            accum: vec![Vec3::ZERO; w * h],
            count: 0,
            bw,
            bh,
            bloom: vec![Vec3::ZERO; bw * bh],
            tmp: vec![Vec3::ZERO; bw * bh],
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
    }

    pub fn samples(&self) -> u32 {
        self.count
    }

    pub fn resolve(&mut self, hdr: &[Vec3], accumulate: bool, bloom_on: bool, out: &mut [u8]) {
        let (w, h) = (self.w, self.h);

        if !accumulate || self.count == 0 {
            self.accum.copy_from_slice(hdr);
            self.count = 1;
        } else {
            self.count = (self.count + 1).min(4096);
            let k = 1.0 / self.count as f32;
            for (a, c) in self.accum.iter_mut().zip(hdr.iter()) {
                *a = *a + (*c - *a) * k;
            }
        }

        let (bw, bh) = (self.bw, self.bh);
        if bloom_on {
            let inv = 1.0 / (DS * DS) as f32;
            for by in 0..bh {
                for bx in 0..bw {
                    let mut s = Vec3::ZERO;
                    for dy in 0..DS {
                        for dx in 0..DS {
                            let x = (bx * DS + dx).min(w - 1);
                            let y = (by * DS + dy).min(h - 1);
                            let c = self.accum[y * w + x];
                            let l = c.max_elem();
                            if l > 0.85 {
                                s += c * ((l - 0.85) / l);
                            }
                        }
                    }
                    self.bloom[by * bw + bx] = s * inv;
                }
            }
            for _ in 0..2 {
                blur(&self.bloom, &mut self.tmp, bw, bh, true);
                blur(&self.tmp, &mut self.bloom, bw, bh, false);
            }
        }

        let threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let rows_per = ((h + threads - 1) / threads).max(1);
        let accum = &self.accum;
        let bloom = &self.bloom;
        std::thread::scope(|s| {
            for (ci, chunk) in out.chunks_mut(rows_per * w * 4).enumerate() {
                s.spawn(move || {
                    for (ri, row) in chunk.chunks_mut(w * 4).enumerate() {
                        let y = ci * rows_per + ri;
                        let sy = 1.0 - ((y as f32 + 0.5) / h as f32) * 2.0;
                        let by = (y / DS).min(bh - 1);
                        for x in 0..w {
                            let mut c = accum[y * w + x];
                            if bloom_on {
                                let bx = (x / DS).min(bw - 1);
                                c += bloom[by * bw + bx] * 0.9;
                            }
                            let sx = ((x as f32 + 0.5) / w as f32) * 2.0 - 1.0;
                            c *= 1.0 - 0.22 * (sx * sx * 0.6 + sy * sy * 0.8);
                            let o = x * 4;
                            row[o] = tonemap(c.x);
                            row[o + 1] = tonemap(c.y);
                            row[o + 2] = tonemap(c.z);
                            row[o + 3] = 255;
                        }
                    }
                });
            }
        });
    }
}

#[inline]
fn tonemap(x: f32) -> u8 {
    let y = if x <= 0.8 {
        x.max(0.0)
    } else {
        let t = (x - 0.8) * 5.0;
        0.8 + 0.2 * (t / (1.0 + t))
    };
    (y * 255.0 + 0.5) as u8
}

fn blur(src: &[Vec3], dst: &mut [Vec3], w: usize, h: usize, horizontal: bool) {
    for y in 0..h {
        for x in 0..w {
            let mut s = src[y * w + x] * GAUSS[0];
            for (k, &g) in GAUSS.iter().enumerate().skip(1) {
                let (a, b) = if horizontal {
                    (y * w + x.saturating_sub(k), y * w + (x + k).min(w - 1))
                } else {
                    (y.saturating_sub(k) * w + x, (y + k).min(h - 1) * w + x)
                };
                s += (src[a] + src[b]) * g;
            }
            dst[y * w + x] = s;
        }
    }
}
