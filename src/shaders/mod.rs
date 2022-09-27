use render_data_container::EVertexDataFormat;
use render_geometry::{geometry::GeometryBufferDesc};
use render_material::{material::{MaterialTextureDesc, MaterialUniformDesc, EUniformDataFormat}};
use render_pipeline_key::uniform_info::calc_uniform_size;

use crate::material::{SpineMaterialBlockKindKey, SpineVertexBufferKindKey};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EShader {
    Colored,
    ColoredTextured,
    TwoColoredTextured,
}

impl EShader {
    pub const VERTEX_COLORED: GeometryBufferDesc<SpineVertexBufferKindKey> = GeometryBufferDesc {
        slot: 0,
        format: EVertexDataFormat::F32,
        kind: SpineVertexBufferKindKey::Vertices,
        size_per_vertex: 2 + 4,
    };
    pub const VERTEX_COLORED_TEXTURED: GeometryBufferDesc<SpineVertexBufferKindKey> = GeometryBufferDesc {
        slot: 0,
        format: EVertexDataFormat::F32,
        kind: SpineVertexBufferKindKey::Vertices,
        size_per_vertex: 2 + 4 + 2,
    };
    pub const VERTEX_COLORED_TEXTURED_TWO: GeometryBufferDesc<SpineVertexBufferKindKey> = GeometryBufferDesc {
        slot: 0,
        format: EVertexDataFormat::F32,
        kind: SpineVertexBufferKindKey::Vertices,
        size_per_vertex: 2 + 4 + 2 + 4,
    };
}

pub struct SpineShader {
    pub vs_module: wgpu::ShaderModule,
    pub fs_module: wgpu::ShaderModule,
    pub attributes: Vec<wgpu::VertexAttribute>,
    pub attributes_bytes: u16,
    pub attributes_instanced: Vec<wgpu::VertexAttribute>,
    pub attributes_instanced_bytes: u16,
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub textures: Vec<MaterialTextureDesc<SpineMaterialBlockKindKey>>,
    pub uniform_bytes: wgpu::BufferAddress,
}

pub trait SpineShaderPool {
    fn record_spine_shader_colored(&mut self, shader: SpineShader);
    fn record_spine_shader_colored_textured(&mut self, shader: SpineShader);
    fn record_spine_shader_colored_textured_two(&mut self, shader: SpineShader);
    fn get_spine_shader_colored(& self) -> &SpineShader;
    fn get_spine_shader_colored_textured(& self) -> &SpineShader;
    fn get_spine_shader_colored_textured_two(& self) -> &SpineShader;
}

impl SpineShader {
    pub fn init<P: SpineShaderPool>(device: &wgpu::Device, pool: &mut P) {
        let attributes_bytes = 6 * std::mem::size_of::<f32>() as u16;
        let attributes_instanced_bytes = 0 * std::mem::size_of::<f32>() as u16;
        let attributes = vec![
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 2 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
                shader_location: 1,
            },
        ];
        let attributes_instanced = vec![];
        let uniform_bytes = calc_uniform_size(device, (16 + 4) * std::mem::size_of::<f32>() as u64) as wgpu::BufferAddress;
        let uniform_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Param
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            // min_binding_size: wgpu::BufferSize::new(uniform_size)
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            }
        );
        let shader = Self::load(device, include_str!("./colored.vert"), include_str!("./colored.frag"), "ColoredVS", "ColoredFS", attributes, attributes_bytes, attributes_instanced, attributes_instanced_bytes, uniform_layout, uniform_bytes, vec![], None);
        pool.record_spine_shader_colored(shader);

        
        let attributes_bytes = 8 * std::mem::size_of::<f32>() as u16;
        let attributes_instanced_bytes = 0 * std::mem::size_of::<f32>() as u16;
        let attributes = vec![
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: (0 + 2) * std::mem::size_of::<f32>() as wgpu::BufferAddress,
                shader_location: 1,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: (0 + 2 + 4) * std::mem::size_of::<f32>() as wgpu::BufferAddress,
                shader_location: 2,
            },
        ];
        let attributes_instanced = vec![];
        let uniform_bytes = calc_uniform_size(device, (16 + 4 + 2 + 2) * std::mem::size_of::<f32>() as u64) as wgpu::BufferAddress;
        let uniform_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Param
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            // min_binding_size: wgpu::BufferSize::new(uniform_size)
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
            }
        );
        let textures = vec![
            MaterialTextureDesc { kind: SpineMaterialBlockKindKey::Texture, bind: 1, bind_sampler: 0 }
        ];
        let texture_layout = Some(device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    }
                ],
            }
        ));
        let shader = Self::load(device, include_str!("./colored_textured.vert"), include_str!("./colored_textured.frag"), "ColoredTexturedVS", "ColoredTexturedFS", attributes, attributes_bytes, attributes_instanced, attributes_instanced_bytes, uniform_layout, uniform_bytes, textures, texture_layout);
        pool.record_spine_shader_colored_textured(shader);

        
        let attributes_bytes = 12 * std::mem::size_of::<f32>() as u16;
        let attributes_instanced_bytes = 0 * std::mem::size_of::<f32>() as u16;
        let attributes = vec![
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: (0 + 2) * std::mem::size_of::<f32>() as wgpu::BufferAddress,
                shader_location: 1,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: (0 + 2 + 4) * std::mem::size_of::<f32>() as wgpu::BufferAddress,
                shader_location: 2,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: (0 + 2 + 4 + 4) * std::mem::size_of::<f32>() as wgpu::BufferAddress,
                shader_location: 3,
            },
        ];
        let attributes_instanced = vec![];
        let uniform_bytes = calc_uniform_size(device, (16 + 4 + 2 + 2) * std::mem::size_of::<f32>() as u64) as wgpu::BufferAddress;
        let uniform_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Param
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            // min_binding_size: wgpu::BufferSize::new(uniform_size)
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
            }
        );
        let textures = vec![
            MaterialTextureDesc { kind: SpineMaterialBlockKindKey::Texture, bind: 1, bind_sampler: 0 }
        ];
        let texture_layout = Some(device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    }
                ],
            }
        ));
        let shader = Self::load(device, include_str!("./two_colored_textured.vert"), include_str!("./two_colored_textured.frag"), "TwoColoredTexturedVS", "TwoColoredTexturedFS", attributes, attributes_bytes, attributes_instanced, attributes_instanced_bytes, uniform_layout, uniform_bytes, textures, texture_layout);
        pool.record_spine_shader_colored_textured_two(shader);
    }
    fn load(
        device: &wgpu::Device,
        vs_text: &str,
        fs_text: &str,
        vs_label: &str,
        fs_label: &str,
        attributes: Vec<wgpu::VertexAttribute>,
        attributes_bytes: u16,
        attributes_instanced: Vec<wgpu::VertexAttribute>,
        attributes_instanced_bytes: u16,
        uniform_layout: wgpu::BindGroupLayout,
        uniform_bytes: wgpu::BufferAddress,
        textures: Vec<MaterialTextureDesc<SpineMaterialBlockKindKey>>,
        texture_layout: Option<wgpu::BindGroupLayout>,
    ) -> SpineShader {
        let vs_module = device.create_shader_module(
            &wgpu::ShaderModuleDescriptor {
                label: Some(vs_label),
                source: wgpu::ShaderSource::Glsl {
                    shader: std::borrow::Cow::Borrowed(vs_text),
                    stage: naga::ShaderStage::Vertex,
                    defines: naga::FastHashMap::default(),
                }
            }
        );

        let fs_module = device.create_shader_module(
            &wgpu::ShaderModuleDescriptor {
                label: Some(fs_label),
                source: wgpu::ShaderSource::Glsl {
                    shader: std::borrow::Cow::Borrowed(fs_text),
                    stage: naga::ShaderStage::Fragment,
                    defines: naga::FastHashMap::default(),
                }
            }
        );

        SpineShader {
            vs_module,
            fs_module,
            attributes,
            attributes_bytes,
            attributes_instanced,
            attributes_instanced_bytes,
            texture_bind_group_layout: texture_layout,
            uniform_bind_group_layout: uniform_layout,
            uniform_bytes,
            textures,
        }
    }
    pub fn get_texture_layout(
        &self,
    ) -> Option<&wgpu::BindGroupLayout> {
        self.texture_bind_group_layout.as_ref()
    }
    pub fn get_uniform_layout(
        &self,
    ) -> &wgpu::BindGroupLayout {
        &self.uniform_bind_group_layout
    }
    pub fn bind_group_layouts(
        &self,
    ) -> Vec<&wgpu::BindGroupLayout> {
        let mut result = vec![];

        result.push(&self.uniform_bind_group_layout);

        match self.texture_bind_group_layout.as_ref() {
            Some(layout) => result.push(layout),
            None => {},
        }

        result
    }
}