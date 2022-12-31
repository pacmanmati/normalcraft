use std::f32::consts::PI;

use glam::{Mat4, Vec2, Vec3};
use winit::dpi::PhysicalSize;

#[rustfmt::skip]
const OPENGL_TO_WGPU: Mat4 = Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
]);

/// Only applies to orthographic projections.
pub enum ResizeStrategy {
    KeepY,
    KeepX,
    Stretch,
}

#[derive(Clone, Copy)]
enum Projection {
    Perspective {
        fov_y: f32,
        aspect_ratio: f32,
        z_near: f32,
        z_far: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
}

impl Projection {
    pub fn set_aspect_ratio(
        self,
        size: PhysicalSize<u32>,
        resize_strategy: ResizeStrategy,
    ) -> Self {
        match self {
            Projection::Perspective {
                fov_y,
                z_near,
                z_far,
                ..
            } => Projection::Perspective {
                fov_y,
                aspect_ratio: (size.width / size.height) as f32,
                z_near,
                z_far,
            },
            Projection::Orthographic {
                mut left,
                mut right,
                mut bottom,
                mut top,
                near,
                far,
            } => {
                match resize_strategy {
                    ResizeStrategy::KeepX => {
                        let inverse_aspect_ratio = size.height as f32 / size.width as f32;
                        top *= inverse_aspect_ratio;
                        bottom *= inverse_aspect_ratio;
                    }
                    ResizeStrategy::KeepY => {
                        let aspect_ratio = size.width as f32 / size.height as f32;
                        right *= aspect_ratio;
                        left *= aspect_ratio;
                    }
                    ResizeStrategy::Stretch => {
                        // let aspect_ratio = size.width as f32 / size.height as f32;
                        // right *= aspect_ratio;
                        // left *= aspect_ratio;
                        // let inverse_aspect_ratio = size.height as f32 / size.width as f32;
                        // top *= inverse_aspect_ratio;
                        // bottom *= inverse_aspect_ratio;
                    }
                }
                Projection::Orthographic {
                    left,
                    right,
                    bottom,
                    top,
                    near,
                    far,
                }
            }
        }
    }
}

const UP: Vec3 = Vec3::Y;

pub struct Camera {
    original_projection: Projection,
    projection: Projection,
    position: Vec3,
    // look_dir: Vec3,
    pitch: f32, // up and down
    yaw: f32,   // left and right
}

impl Camera {
    pub fn new_projection(
        position: Vec3,
        fov_y: f32,
        aspect_ratio: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        let projection = Projection::Perspective {
            fov_y,
            aspect_ratio,
            z_near,
            z_far,
        };
        Self {
            original_projection: projection,
            projection,
            position,
            pitch: 0.0,
            yaw: PI, // look_dir: DEFAULT_LOOK_DIR,
        }
    }

    pub fn new_orthographic(
        position: Vec3,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let projection = Projection::Orthographic {
            left,
            right,
            bottom,
            top,
            near,
            far,
        };
        Self {
            original_projection: projection,
            projection,
            position,
            pitch: 0.0,
            yaw: PI,
            // look_dir: DEFAULT_LOOK_DIR,
        }
    }

    pub fn look_dir(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.cos(),
        )
    }

    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            0.0,
            self.pitch.cos() * self.yaw.cos(),
        )
    }

    pub fn up(&self) -> Vec3 {
        UP
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(UP)
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.position += translation;
    }

    // pub fn look_at(&mut self, direction: Vec3) {
    //     self.look_dir = direction;
    // }

    pub fn look_add(&mut self, other: Vec2) {
        self.pitch += other.y;
        self.pitch = self
            .pitch
            .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        self.yaw += other.x;
        // println!("{}, {}", self.pitch, self.yaw);
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>, resize_strategy: ResizeStrategy) {
        self.projection = self
            .original_projection
            .set_aspect_ratio(size, resize_strategy);
        // we calculate the current projection from our original projection (const) to avoid cumulative float errors
    }

    pub fn compute(&self) -> Mat4 {
        // let pitch be the angle on the z-plane, 0 if front facing, positive looking up
        // let yaw be the angle on the x-plane, 0 if front facing, positive looking right

        OPENGL_TO_WGPU
            * match self.projection {
                Projection::Perspective {
                    fov_y,
                    aspect_ratio,
                    z_near,
                    z_far,
                } => Mat4::perspective_rh(f32::to_radians(fov_y), aspect_ratio, z_near, z_far),
                Projection::Orthographic {
                    left,
                    right,
                    bottom,
                    top,
                    near,
                    far,
                } => Mat4::orthographic_rh(left, right, bottom, top, near, far),
            }
            * Mat4::look_to_rh(self.position, self.look_dir(), UP)
    }
}
