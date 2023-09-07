use cgmath::*;
// use std::time::Duration;
use winit::dpi::PhysicalPosition;
use winit::event::*;

// const MAX_ROTATION_SPEED: f32 = 1000.0;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn update_aspect(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: OPENGL_TO_WGPU_MATRIX.into(),
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct CameraController {
    speed: f32,
    mouse_sensitivity: f32,
    scroll_sensitivity: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32, mouse_sensitivity: f32, scroll_sensitivity: f32) -> Self {
        Self {
            speed,
            mouse_sensitivity,
            scroll_sensitivity,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let is_pressed = if state == ElementState::Pressed {
            true
        } else {
            false
        };
        match key {
            VirtualKeyCode::Space => {
                self.is_up_pressed = is_pressed;
                true
            }
            VirtualKeyCode::LShift => {
                self.is_down_pressed = is_pressed;
                false
            }
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.is_forward_pressed = is_pressed;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.is_left_pressed = is_pressed;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.is_backward_pressed = is_pressed;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.is_right_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32 * self.mouse_sensitivity * 0.01;
        self.rotate_vertical = mouse_dy as f32 * self.mouse_sensitivity * 0.01;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * self.scroll_sensitivity,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => -*scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        let forward = (camera.target - camera.eye).normalize();
        let right = camera.up.cross(forward).normalize();
        // let up = forward.cross(right);

        if self.scroll != 0.0 {
            camera.eye += forward * self.scroll * self.speed;
            self.scroll = 0.0;
        }

        if self.rotate_horizontal != 0.0 || self.rotate_vertical != 0.0 {
            let direction = camera.target - camera.eye;
            let distance = direction.magnitude();
            let direction = direction.normalize();

            let horizontal_angle = Rad(self.rotate_horizontal);
            let vertical_angle = Rad(self.rotate_vertical);

            let horizontal_rotation = Quaternion::from_axis_angle(camera.up, horizontal_angle);
            let vertical_rotation = Quaternion::from_axis_angle(right, vertical_angle);

            let direction = horizontal_rotation * (vertical_rotation * direction);
            camera.eye = camera.target - direction * distance;

            self.rotate_horizontal = 0.0;
            self.rotate_vertical = 0.0;
        }

        if self.is_forward_pressed {
            // camera.eye += self.speed * forward;
            // camera.target += self.speed * forward;
        }
        if self.is_backward_pressed {
            // camera.eye -= self.speed * forward;
            // camera.target -= self.speed * forward;
        }
        if self.is_right_pressed {
            // camera.eye -= self.speed * right;
            // camera.target -= self.speed * right;
        }
        if self.is_left_pressed {
            // camera.eye += self.speed * right;
            // camera.target += self.speed * right;
        }
        if self.is_up_pressed {
            // camera.eye += self.speed * up;
            // camera.target += self.speed * up;
        }
        if self.is_down_pressed {
            // camera.eye -= self.speed * up;
            // camera.target -= self.speed * up;
        }
    }
}
