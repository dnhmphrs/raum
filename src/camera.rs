use cgmath::*;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use winit::dpi::PhysicalPosition;
use winit::event::*;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

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
        proj * view
    }

    pub fn update_aspect(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }
}

pub struct CameraController {
    speed: f32,
    sensitivity: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
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

    pub fn update_camera(&mut self, camera: &mut Camera) {
        let forward = (camera.target - camera.eye).normalize();
        let right = camera.up.cross(forward).normalize();
        let up = forward.cross(right);

        if self.is_forward_pressed {
            camera.eye += self.speed * forward;
            camera.target += self.speed * forward;
        }
        if self.is_backward_pressed {
            camera.eye -= self.speed * forward;
            camera.target -= self.speed * forward;
        }
        if self.is_right_pressed {
            camera.eye += self.speed * right;
            camera.target += self.speed * right;
        }
        if self.is_left_pressed {
            camera.eye -= self.speed * right;
            camera.target -= self.speed * right;
        }
        if self.is_up_pressed {
            camera.eye += self.speed * up;
            camera.target += self.speed * up;
        }
        if self.is_down_pressed {
            camera.eye -= self.speed * up;
            camera.target -= self.speed * up;
        }
    }
}

// pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
//     self.yaw += Rad(mouse_dx as f32 * self.sensitivity); // * dt.as_secs_f32());
//     self.pitch += Rad(-mouse_dy as f32 * self.sensitivity); // * dt.as_secs_f32());

//     if self.pitch < -Rad(FRAC_PI_2) {
//         self.pitch = -Rad(FRAC_PI_2);
//     } else if self.pitch > Rad(FRAC_PI_2) {
//         self.pitch = Rad(FRAC_PI_2);
//     }
// }

// pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
//     match delta {
//         MouseScrollDelta::LineDelta(_, scroll) => self.scroll = scroll * 20.0,
//         MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
//             self.scroll = *scroll as f32
//         }
//     };

//     self.radius -= self.scroll * self.speed * self.sensitivity; // * dt.as_secs_f32();
//     if self.radius < 0.0 {
//         self.radius = 0.0;
//     }
// }

// pub fn update_camera(&mut self, camera: &mut Camera) {
//     // Calculate the new eye position based on yaw and pitch
//     let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();
//     let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
//     let eye_offset = Vector3::new(
//         self.radius * cos_pitch * cos_yaw,
//         self.radius * sin_pitch,
//         self.radius * cos_pitch * sin_yaw,
//     );

//     camera.eye = Point3::from_vec(camera.target.to_vec() + eye_offset);

//     // Update target position based on keyboard input
//     if self.is_forward_pressed > 0.0 {
//         camera.target += self.speed * Vector3::unit_z();
//     }
//     if self.is_backward_pressed > 0.0 {
//         camera.target -= self.speed * Vector3::unit_z();
//     }
//     if self.is_right_pressed > 0.0 {
//         camera.target -= self.speed * Vector3::unit_x();
//     }
//     if self.is_left_pressed > 0.0 {
//         camera.target += self.speed * Vector3::unit_x();
//     }
//     if self.is_up_pressed > 0.0 {
//         camera.target += self.speed * Vector3::unit_y();
//     }
//     if self.is_down_pressed > 0.0 {
//         camera.target -= self.speed * Vector3::unit_y();
//     }
// }
// }
