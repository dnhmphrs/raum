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

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[derive(Debug)]
pub struct CameraController {
    radius: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
    is_up_pressed: f32,
    is_down_pressed: f32,
    is_forward_pressed: f32,
    is_backward_pressed: f32,
    is_left_pressed: f32,
    is_right_pressed: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32, radius: f32) -> Self {
        Self {
            radius,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: -100.0,
            speed,
            sensitivity,
            is_up_pressed: 0.0,
            is_down_pressed: 0.0,
            is_forward_pressed: 0.0,
            is_backward_pressed: 0.0,
            is_left_pressed: 0.0,
            is_right_pressed: 0.0,
        }
    }

    // ... process_keyboard, process_mouse, process_scroll functions ...

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            VirtualKeyCode::Space => {
                self.is_up_pressed = amount;
            }
            VirtualKeyCode::LShift => {
                self.is_down_pressed = amount;
            }
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.is_forward_pressed = amount;
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.is_left_pressed = amount;
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.is_backward_pressed = amount;
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.is_right_pressed = amount;
            }
            _ => (),
        }
        false
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 20.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Rotate
        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }

        // Change radius (distance from the origin) based on scroll
        self.radius -= self.scroll * self.speed * self.sensitivity * dt;
        if self.radius < 0.0 {
            self.radius = 0.0;
        }

        // Convert spherical coordinates (radius, yaw, pitch) to cartesian coordinates
        let (sin_yaw, cos_yaw) = camera.yaw.0.sin_cos();
        let (sin_pitch, cos_pitch) = camera.pitch.0.sin_cos();
        camera.position = Point3::new(
            -self.radius * cos_pitch * cos_yaw,
            -self.radius * sin_pitch,
            -self.radius * cos_pitch * sin_yaw,
        );

        // Reset scroll and rotations
        self.scroll = 0.0;
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
    }
}
