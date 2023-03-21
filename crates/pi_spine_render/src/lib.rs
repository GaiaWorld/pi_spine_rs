use pi_hash::XHashMap;
use renderer::Renderer;


pub mod binds;
pub mod bind_groups;
pub mod shaders;
pub mod vertex_buffer;
pub mod renderer;

pub enum EPrimitive {
    POINT,
    LINE,
    TILLED,
}
impl EPrimitive {
    pub fn mode(&self) -> wgpu::PrimitiveState {
        match self {
            EPrimitive::POINT => wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint16),
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            EPrimitive::LINE => todo!(),
            EPrimitive::TILLED => todo!(),
        }
    }
}
