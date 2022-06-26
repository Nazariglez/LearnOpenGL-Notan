use notan::math::{vec3, Mat4, Vec3};

const YAW: f32 = -90.0;
const PITCH: f32 = 0.0;
const SPEED: f32 = 2.5;
const SENSITIVITY: f32 = 0.1;
const ZOOM: f32 = 45.0;

pub enum CameraMovement {
    Forward,
    Backward,
    Left,
    Right,
}

pub struct Camera {
    pub position: Vec3,
    pub front: Vec3,
    up: Vec3,
    right: Vec3,
    world_up: Vec3,
    yaw: f32,
    pitch: f32,
    movement_speed: f32,
    mouse_sensitivity: f32,
    pub zoom: f32,
}

impl Camera {
    pub fn new(position: Vec3) -> Self {
        let mut camera = Self {
            position,
            front: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
            right: Default::default(),
            world_up: Default::default(),
            yaw: YAW,
            pitch: PITCH,
            movement_speed: SPEED,
            mouse_sensitivity: SENSITIVITY,
            zoom: ZOOM,
        };
        update_camera_vectors(&mut camera);
        camera
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.front, self.up)
    }

    pub fn process_keyboard(&mut self, direction: CameraMovement, delta: f32) {
        let velocity = self.movement_speed * delta;
        match direction {
            CameraMovement::Forward => self.position += self.front * velocity,
            CameraMovement::Backward => self.position -= self.front * velocity,
            CameraMovement::Left => self.position -= self.right * velocity,
            CameraMovement::Right => self.position += self.right * velocity,
        }
    }

    pub fn process_mouse_movement(&mut self, xoffset: f32, yoffset: f32, constrain_pitch: bool) {
        self.yaw += xoffset * self.mouse_sensitivity;
        self.pitch += yoffset * self.mouse_sensitivity;

        if constrain_pitch {
            self.pitch = self.pitch.clamp(-89.0, 89.0);
        }

        update_camera_vectors(self);
    }

    pub fn process_mouse_scroll(&mut self, yoffset: f32) {
        if yoffset != 0.0 {
            self.zoom -= yoffset;
            self.zoom = self.zoom.clamp(1.0, 45.0);
        }
    }
}

fn update_camera_vectors(camera: &mut Camera) {
    let front = vec3(
        camera.yaw.to_radians().cos() * camera.pitch.to_radians().cos(),
        camera.pitch.to_radians().sin(),
        camera.yaw.to_radians().sin() * camera.pitch.to_radians().cos(),
    );

    println!(": yaw:{} pitch:{}", camera.yaw.to_radians().cos(), camera.pitch.to_radians().cos());

    camera.front = front.normalize();
    camera.right = camera.front.cross(camera.world_up).normalize();
    camera.up = camera.right.cross(camera.front).normalize();
}
