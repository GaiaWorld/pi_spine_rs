use shaders::{SpineShaderPool};


pub mod error;
pub mod mesh;
pub mod shaders;
// pub mod shape_renderer;
// pub mod polygon_batcher;
pub mod mesh_renderer;
pub mod pipeline;
pub mod material;

pub const MAX_VERTICES: u16 = 10920;

pub fn vec_set<T: Copy>(vec: &mut Vec<T>, data: &[T], offset: usize) {
    let len = data.len();
    if offset + len <= vec.len() {
        for i in 0..len {
            vec[offset + i] = data[i];
        }
    }
}

pub struct SpineShaderPoolSimple {
    pub colored: Option<shaders::SpineShader>,
    pub colored_textured: Option<shaders::SpineShader>,
    pub colored_textured_two: Option<shaders::SpineShader>,
}

impl Default for SpineShaderPoolSimple {
    fn default() -> Self {
        Self {
            colored: None,
            colored_textured: None,
            colored_textured_two: None,
        }
    }
}

impl SpineShaderPoolSimple {
    pub fn init(&mut self, device: &wgpu::Device) {
        shaders::SpineShader::init(device, self);
    }
}

impl SpineShaderPool for SpineShaderPoolSimple {
    fn record_spine_shader_colored(&mut self, shader: shaders::SpineShader) {
        self.colored = Some(shader);
    }

    fn record_spine_shader_colored_textured(&mut self, shader: shaders::SpineShader) {
        self.colored_textured = Some(shader);
    }

    fn record_spine_shader_colored_textured_two(&mut self, shader: shaders::SpineShader) {
        self.colored_textured_two = Some(shader);
    }

    fn get_spine_shader_colored(& self) -> &shaders::SpineShader {
        self.colored.as_ref().unwrap()
    }

    fn get_spine_shader_colored_textured(& self) -> &shaders::SpineShader {
        self.colored_textured.as_ref().unwrap()
    }

    fn get_spine_shader_colored_textured_two(& self) -> &shaders::SpineShader {
        self.colored_textured_two.as_ref().unwrap()
    }
}