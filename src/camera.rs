use ash::vk;
use glm::TVec3;

pub struct Camera {
    pos: glm::TVec3<f32>,
    front: glm::TVec3<f32>,
    up: glm::TVec3<f32>,

    extent: vk::Extent2D,
    projection: glm::TMat4<f32>,

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

        let mut projection: glm::TMat4<f32> = glm::perspective(aspect, fovy, near, far);
        projection[(1, 1)] *= -1.0;

        Self {
            pos: TVec3::new(0.0, 0.0, 3.0),
            front: TVec3::new(0.0, 0.0, 1.0),
            up: TVec3::new(0.0, 1.0, 0.0),
            extent,
            projection,
            yaw,
            pitch,
            fovy,
            near,
            far,
        }
    }

    pub fn get_view(&self) -> glm::TMat4<f32> {
        glm::look_at(&self.pos, &(self.pos + self.front), &self.up)
    }

    pub fn ortho(max_right: f32, max_top: f32) -> glm::TMat4<f32> {
        glm::ortho(0.0, max_right, 0.0, max_top, -1.0, 1.0)
    }

    pub fn get_projection(&self) -> glm::TMat4<f32> {
        self.projection
    }

    pub fn get_pos(&self) -> glm::TVec3<f32> {
        self.pos
    }
}
