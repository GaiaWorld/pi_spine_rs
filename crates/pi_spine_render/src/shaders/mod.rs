

use std::{sync::Arc, f32::consts::E, hash::{Hash, Hasher}};

use pi_assets::{asset::{Asset, Handle, GarbageEmpty}, mgr::AssetMgr};
use pi_hash::DefaultHasher;
use pi_render::{renderer::{shader::{TShaderSetBlock, Shader, KeyShader, KeyShaderMeta, KeyShaderSetBlocks}, attributes::{KeyShaderFromAttributes, EVertexDataKind, VertexAttribute}, vertex_buffer::{VertexBufferLayouts, VertexBufferLayout, KeyVertexBuffer}, vertex_buffer_desc::VertexBufferDesc, instance::EInstanceKind, pipeline::KeyRenderPipelineState}, rhi::{device::RenderDevice, asset::RenderRes, pipeline::RenderPipeline, RenderQueue, bind_group_layout::BindGroupLayout}, asset::ASSET_SIZE_FOR_UNKOWN};
use pi_share::Share;

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::binds::param::BindParam;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EKeySpineSet {
    Param,
    Texture,
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[cfg(feature = "pi_js_export")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeySpineShader {
    Colored,
    ColoredTextured,
    TwoColoredTextured,
}

impl KeySpineShader {
    pub fn key(&self) -> String {
        match self {
            Self::Colored => String::from("Colored"),
            Self::ColoredTextured => String::from("ColoredTextured"),
            Self::TwoColoredTextured => String::from("TwoColoredTextured"),
        }
    }
    pub fn vertices_bytes_per_element(&self) -> u32 {
        match self {
            KeySpineShader::Colored => (2 + 4) * 4,
            KeySpineShader::ColoredTextured => (2 + 4 + 2) * 4,
            KeySpineShader::TwoColoredTextured => (2 + 4 + 2 + 4) * 4,
        }
    }
    pub fn attributes(&self) -> Vec<wgpu::VertexAttribute> {
        match self {
            KeySpineShader::Colored => {
                vec![
                    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 00, shader_location: 0, },
                    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 08, shader_location: 1, },
                ]
            },
            KeySpineShader::ColoredTextured => {
                vec![
                    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 00, shader_location: 0, },
                    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 08, shader_location: 1, },
                    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 24, shader_location: 2, },
                ]
            },
            KeySpineShader::TwoColoredTextured => {
                vec![
                    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 00, shader_location: 0, },
                    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 08, shader_location: 1, },
                    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 24, shader_location: 2, },
                    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 32, shader_location: 3, },
                ]
            },
        }
    }
    pub fn bind_group_layout(&self, device: &RenderDevice) -> BindGroupLayout {
        match self {
            KeySpineShader::Colored => {
                device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[
                            // Param
                            BindParam::layout_entry(),
                        ],
                    }
                )
            },
            _ => {
                device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[
                            // Param
                            BindParam::layout_entry(),
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 2,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            },
                        ]
                    }
                )
            },
        }
    }
    pub fn shader(&self, device: &RenderDevice) -> SpineShader {
        match self {
            Self::Colored => {
                let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some((self.key() + "-VS").as_str()),
                    source: wgpu::ShaderSource::Glsl {
                        shader: std::borrow::Cow::Borrowed(include_str!("./colored.vert")),
                        stage: naga::ShaderStage::Vertex,
                        defines: naga::FastHashMap::default(),
                    },
                });
        
                let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some((self.key() + "-FS").as_str()),
                    source: wgpu::ShaderSource::Glsl {
                        shader: std::borrow::Cow::Borrowed(include_str!("./colored.frag")),
                        stage: naga::ShaderStage::Fragment,
                        defines: naga::FastHashMap::default(),
                    },
                });
        
                SpineShader { vs, vs_point: "main", fs, fs_point: "main"  }
            },
            Self::ColoredTextured => {
                let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some((self.key() + "-VS").as_str()),
                    source: wgpu::ShaderSource::Glsl {
                        shader: std::borrow::Cow::Borrowed(include_str!("./colored_textured.vert")),
                        stage: naga::ShaderStage::Vertex,
                        defines: naga::FastHashMap::default(),
                    },
                });
        
                let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some((self.key() + "-FS").as_str()),
                    source: wgpu::ShaderSource::Glsl {
                        shader: std::borrow::Cow::Borrowed(include_str!("./colored_textured.frag")),
                        stage: naga::ShaderStage::Fragment,
                        defines: naga::FastHashMap::default(),
                    },
                });
        
                SpineShader { vs, vs_point: "main", fs, fs_point: "main"  }
            },
            Self::TwoColoredTextured => {
                let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some((self.key() + "-VS").as_str()),
                    source: wgpu::ShaderSource::Glsl {
                        shader: std::borrow::Cow::Borrowed(include_str!("./two_colored_textured.vert")),
                        stage: naga::ShaderStage::Vertex,
                        defines: naga::FastHashMap::default(),
                    },
                });
        
                let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some((self.key() + "-FS").as_str()),
                    source: wgpu::ShaderSource::Glsl {
                        shader: std::borrow::Cow::Borrowed(include_str!("./two_colored_textured.frag")),
                        stage: naga::ShaderStage::Fragment,
                        defines: naga::FastHashMap::default(),
                    },
                });
        
                SpineShader { vs, vs_point: "main", fs, fs_point: "main"  }
            },
        }
    }
}

pub struct SpineShader {
    pub vs: wgpu::ShaderModule,
    pub vs_point: &'static str,
    pub fs: wgpu::ShaderModule,
    pub fs_point: &'static str,
}

pub struct SingleSpineShaderPool {
    pub colored: SpineShader,
    pub colored_textured: SpineShader,
    pub two_colored_textured: SpineShader,
}
impl SingleSpineShaderPool {
    pub fn new(device: &RenderDevice) -> Self {
        Self {
            colored: KeySpineShader::Colored.shader(device),
            colored_textured: KeySpineShader::ColoredTextured.shader(device),
            two_colored_textured: KeySpineShader::TwoColoredTextured.shader(device),
        }
    }
    fn shader(&self, key: &KeySpineShader) -> &SpineShader {
        match key {
            KeySpineShader::Colored => &self.colored,
            KeySpineShader::ColoredTextured => &self.colored_textured,
            KeySpineShader::TwoColoredTextured => &self.two_colored_textured,
        }
    }
}

pub struct SingleSpineBindGroupLayout {
    pub colored: BindGroupLayout,
    pub colored_textured: BindGroupLayout,
    pub two_colored_textured: BindGroupLayout,
}
impl SingleSpineBindGroupLayout {
    pub fn new(device: &RenderDevice) -> Self {
        Self {
            colored: KeySpineShader::Colored.bind_group_layout(device),
            colored_textured:  KeySpineShader::ColoredTextured.bind_group_layout(device),
            two_colored_textured:  KeySpineShader::TwoColoredTextured.bind_group_layout(device),
        }
    }
    pub fn value<'a>(&'a self, key: &KeySpineShader) -> Vec<&'a wgpu::BindGroupLayout> {
        match key {
            KeySpineShader::Colored => vec![&self.colored],
            KeySpineShader::ColoredTextured => vec![&self.colored_textured],
            KeySpineShader::TwoColoredTextured => vec![&self.two_colored_textured],
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeySpinePipeline {
    pub key_shader: KeySpineShader,
    pub key_state: KeyRenderPipelineState,
}
impl KeySpinePipeline {
    pub fn as_u64(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

pub struct SingleSpinePipelinePool {
    shaders: SingleSpineShaderPool,
    bind_group_layouts: SingleSpineBindGroupLayout,
    asset_mgr: Share<AssetMgr<RenderRes<RenderPipeline>>>
}
impl SingleSpinePipelinePool {
    pub fn new(device: &RenderDevice) -> Self {
        Self {
            shaders: SingleSpineShaderPool::new(device),
            bind_group_layouts: SingleSpineBindGroupLayout::new(device),
            asset_mgr: AssetMgr::<RenderRes::<RenderPipeline>>::new(GarbageEmpty(), false, 1 * 1024, 60 * 1000),
        }
    }
    pub fn get(
        &self,
        device: &RenderDevice, 
        key: &KeySpinePipeline,
    ) -> Option<Handle<RenderRes<RenderPipeline>>> {
        let key_u64 = key.as_u64();
        if let Some(pipeline) = self.asset_mgr.get(&key_u64) {
            Some(pipeline)
        } else {
            let pipeline = self.pipeline(device, key);
            self.asset_mgr.insert(key_u64, pipeline).ok()
        }
    }
    fn pipeline(
        &self,
        device: &RenderDevice, 
        key: &KeySpinePipeline,
    ) -> RenderRes<RenderPipeline> {
        let key_shader = &key.key_shader;
        let state = &key.key_state;
        let array_stride = key_shader.vertices_bytes_per_element();
        let attributes = key_shader.attributes();
        let shader = self.shaders.shader(key_shader);
        let bind_group_layouts = self.bind_group_layouts.value(key_shader);
        
        let vertex_layouts = vec![
            wgpu::VertexBufferLayout {
                array_stride: array_stride as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &attributes,
            },
        ];
        let vs_state = wgpu::VertexState {
            module: &shader.vs,
            entry_point: "main",
            buffers: &vertex_layouts,
        };
        let fs_state = wgpu::FragmentState {
            module: &shader.fs,
            entry_point: "main",
            targets: &state.target_state,
        };
        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &bind_group_layouts,
                push_constant_ranges: &[],
            }
        );

        let depth_stencil = if let Some(depth_stencil) = &state.depth_stencil {
            Some(depth_stencil.depth_stencil_state())
        } else {
            None
        };
        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: vs_state,
                fragment: Some(fs_state),
                primitive: state.primitive.clone(),
                depth_stencil,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false
                },
                multiview: None,
            }
        );

        RenderRes::new(pipeline, ASSET_SIZE_FOR_UNKOWN)
    }
}
