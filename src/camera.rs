use crate::math::Vec3;

pub struct OrbitCamera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub fov_deg: f32,
}

pub struct CamFrame {
    pub eye: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    tan_half: f32,
    aspect: f32,
}

impl OrbitCamera {
    pub fn new(target: Vec3) -> Self {
        OrbitCamera {
            target,
            yaw: 2.35,
            pitch: 0.35,
            distance: 55.0,
            fov_deg: 50.0,
        }
    }

    pub fn eye(&self) -> Vec3 {
        let cp = self.pitch.cos();
        self.target
            + Vec3::new(cp * self.yaw.cos(), self.pitch.sin(), cp * self.yaw.sin()) * self.distance
    }

    pub fn rotate(&mut self, dyaw: f32, dpitch: f32) {
        self.yaw += dyaw;
        self.pitch = (self.pitch + dpitch).clamp(-1.2, 1.45);
    }

    pub fn zoom(&mut self, factor: f32) {
        self.distance = (self.distance * factor).clamp(8.0, 110.0);
    }

    pub fn frame(&self, aspect: f32) -> CamFrame {
        let eye = self.eye();
        let forward = (self.target - eye).normalize();
        let right = forward.cross(Vec3::UP).normalize();
        let up = right.cross(forward);
        CamFrame {
            eye,
            forward,
            right,
            up,
            tan_half: (self.fov_deg.to_radians() * 0.5).tan(),
            aspect,
        }
    }
}

impl CamFrame {
    #[inline]
    pub fn ray(&self, sx: f32, sy: f32) -> Vec3 {
        (self.forward
            + self.right * (sx * self.tan_half * self.aspect)
            + self.up * (sy * self.tan_half))
            .normalize()
    }
}
