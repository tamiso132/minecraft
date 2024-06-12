use ash::vk;
use glm::{Mat4, Vec3};

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
}
