use std::{collections::HashMap, mem::transmute, ops::Mul};

use ash::vk;
use glm::{Mat4, Vec3};
use winit::{event::WindowEvent, keyboard::SmolStr};

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
    letters: [bool; 26],
}
impl Controls {
    pub fn new() -> Controls {
        let letters = [false; 26];

        Self { letters }
    }

    pub fn update_key(&mut self, letter: Alphabet, state: bool) {
        self.letters[letter as usize] = state;
    }

    pub fn get_state(&self, letter: Alphabet) -> bool {
        self.letters[letter as usize]
    }

    pub fn reset_state(&mut self) {
        self.letters = [false; 26];
    }
}
#[repr(C, align(16))]
pub struct GPUCamera {
    viewproj: Mat4,
    pos: Vec3,
}

pub struct Camera {
    pos: glm::Vec3,
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

        let mut projection: glm::Mat4 = glm::projection::perspective_vk(aspect, fovy, near, far);
        projection[1][1] *= -1.0;

        Self {
            pos: Vec3::new(0.0, 0.0, 3.0),
            front: Vec3::new(0.0, 0.0, 1.0),
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
    pub fn process_keyboard(&mut self, controls: &Controls, delta_time: f64) {
        let cam_speed = Vec3::new(2.5 * delta_time as f32, 2.5 * delta_time as f32, 2.5 * delta_time as f32);

        if controls.get_state(Alphabet::W) {
            self.pos += cam_speed * self.front;
        }

        if controls.get_state(Alphabet::S) {
            self.pos -= cam_speed * self.front;
        }

        if controls.get_state(Alphabet::D) {
            self.pos += self.front.cross(self.up).normalized() * cam_speed;
        }
        if controls.get_state(Alphabet::A) {
            self.pos -= self.front.cross(self.up).normalized() * cam_speed;
        }
    }

    pub fn process_mouse(&mut self, mut mouse_delta: (f64, f64)) {
        let sensitivity = 0.1;

        mouse_delta = (mouse_delta.0 * sensitivity, mouse_delta.1 * sensitivity);

        self.yaw += mouse_delta.0 as f32;
        self.pitch += mouse_delta.1 as f32 * -1.0;

        if self.pitch > 89.0 {
            self.pitch = 89.0;
        } else if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        let mut direction = glm::Vec3::one();

        direction.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        direction.y = self.pitch.sin();
        direction.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();

        self.front = direction.normalized();
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
        let view = Mat4::look_at(self.pos, self.pos + self.front, self.up);

        let view_proj = view * self.projection;

        GPUCamera { viewproj: view_proj, pos: self.pos }
    }
}
