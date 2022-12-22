use glam::{Mat4, Quat, Vec3};

use crate::texture::TextureHandle;

pub struct Instance {
    position: Vec3,
    rotation: Quat,
    pub texture: TextureHandle,
}

impl Instance {
    pub fn new(position: Vec3, rotation: Quat, texture: TextureHandle) -> Self {
        Self {
            position,
            rotation,
            texture,
        }
    }

    pub fn raw(&self) -> [f32; 16] {
        Mat4::from_rotation_translation(self.rotation, self.position).to_cols_array()
    }
}
