use bytemuck::{self};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: super::OPENGL_TO_WGPU_MATRIX.into(),
        }
    }

    pub fn update(&mut self, camera: &super::Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
