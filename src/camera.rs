use std::{collections::HashMap, mem::transmute, ops::Mul};

use ash::vk::{self, Extent2D};
use glm::{Mat4, Vec3};
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, SmolStr},
};

#[repr(u8)]
pub enum Alphabet {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

impl From<SmolStr> for Alphabet {
    /// Will only work for characters
    fn from(value: SmolStr) -> Self {
        unsafe {
            let char = value.as_str().chars().next().unwrap() as u8;
            let letter: Alphabet = transmute(char - 97);
            letter
        }
    }
}

pub struct Controls {
    letters: [bool; 193],
}
impl Controls {
    pub fn new() -> Controls {
        let letters = [false; 193];

        Self { letters }
    }

    pub fn update_key(&mut self, letter: KeyCode, state: bool) {
        self.letters[letter as usize] = state;
    }

    pub fn get_state(&self, letter: KeyCode) -> bool {
        self.letters[letter as usize]
    }

    pub fn reset_state(&mut self) {
        self.letters = [false; 193];
    }
}
#[repr(C, align(16))]
pub struct GPUCamera {
    viewproj: Mat4,
    pos: Vec3,
}
#[derive(Debug)]
pub struct Camera {
    pub pos: glm::Vec3,
    front: glm::Vec3,
    up: glm::Vec3,

    extent: vk::Extent2D,
    projection: glm::Mat4,

    yaw: f32,
    pitch: f32,

    fovy: f32,
    near: f32,
    far: f32,
}

impl Camera {
    pub fn new(extent: vk::Extent2D) -> Self {
        let aspect = extent.width as f32 / extent.height as f32;
        let fovy = f32::from(70.0).to_radians();
        let near = 0.1;
        let far = 200.0;
        let yaw = 0.0;
        let pitch = 0.0;

        let mut projection: glm::Mat4 = glm::projection::perspective_vk(fovy, aspect, near, far);

        Self {
            pos: Vec3::new(0.0, 0.0, 3.0),
            front: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            extent,
            projection,
            yaw,
            pitch,
            fovy,
            near,
            far,
        }
    }

    pub fn resize_window(&mut self, extent: Extent2D) {
        let aspect = extent.width as f32 / extent.height as f32;
        let fovy = f32::from(70.0).to_radians();
        let near = 0.1;
        let far = 200.0;

        self.projection = glm::projection::perspective_vk(fovy, aspect, near, far);
    }

    pub fn process_keyboard(&mut self, controls: &Controls, delta_time: f64) {
        let speed_mul = 6.0;

        let cam_speed = Vec3::new(speed_mul * delta_time as f32, speed_mul * delta_time as f32, speed_mul * delta_time as f32);
        if controls.get_state(KeyCode::KeyW) {
            self.pos += cam_speed * self.front;
        }

        if controls.get_state(KeyCode::KeyS) {
            self.pos -= cam_speed * self.front;
        }

        if controls.get_state(KeyCode::KeyD) {
            self.pos += self.front.cross(self.up).normalized() * cam_speed;
        }
        if controls.get_state(KeyCode::KeyA) {
            self.pos -= self.front.cross(self.up).normalized() * cam_speed;
        }
    }

    pub fn process_mouse(&mut self, mut mouse_delta: (f64, f64)) {
        let sensitivity = 0.06;

        mouse_delta = (mouse_delta.0 * sensitivity, mouse_delta.1 * sensitivity);

        self.yaw += mouse_delta.0 as f32;
        self.pitch += mouse_delta.1 as f32 * -1.0;

        if self.pitch > 89.0 {
            self.pitch = 89.0;
        } else if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        self.front.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        self.front.y = self.pitch.to_radians().sin();
        self.front.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();

        self.front.normalize();
    }

    pub fn get_view(&self) -> glm::Mat4 {
        Mat4::look_at(self.pos, self.pos + self.front, self.up)
    }

    pub fn ortho(max_right: f32, max_top: f32) -> glm::Mat4 {
        glm::projection::orthographic_vk(0.0, max_right, 0.0, max_top, -1.0, 1.0)
    }

    pub fn get_projection(&self) -> glm::Mat4 {
        self.projection
    }

    pub fn get_pos(&self) -> glm::Vec3 {
        self.pos
    }

    pub fn get_gpu_camera(&self) -> GPUCamera {
        let viewproj = self.get_projection().mul(self.get_view());

        GPUCamera { viewproj, pos: self.pos }
    }

    pub fn get_shader_format(&self) -> GPUCamera {
        let view = self.get_view();

        let view_proj = view * self.projection;

        GPUCamera { viewproj: view_proj, pos: self.pos }
    }
}
